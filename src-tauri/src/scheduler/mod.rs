use std::{collections::HashSet, time::Duration};

use chrono::{Datelike, FixedOffset, Local, Timelike, Utc};
use serde_json::{json, Value};
use tokio::time::{sleep, timeout};

use crate::{
    agents::{personas, tools},
    db::{AppState, GENERAL_SALON_ID},
    llm::deepseek::{ChatMessage, ChatRequest, ChatToolCall, DeepSeekClient, DEEPSEEK_MODEL},
    llm::search::SearchClient,
    models::{Actor, AgentStepResult, FeedPost, PostMediaInput, SelfEdits},
    services::files,
};

pub const TICK_MINUTES: u64 = 5;
pub const QUEUE_POLL_SECONDS: u64 = 15;
const MAX_TOOL_ROUNDS: usize = 4;
const AGENT_RUN_TIMEOUT_SECONDS: u64 = 300;
const DAYTIME_SCHEDULE_SECONDS: i64 = 45 * 60;
const NIGHT_SCHEDULE_SECONDS: i64 = 3 * 60 * 60;
const WHIM_INTERVAL_MINUTES: u64 = 30;
const WHIM_PROBABILITY: f64 = 0.25;
const NOMI_WHIM_PROBABILITY: f64 = 0.08;
const BRIEFING_INTERVAL_HOURS: u64 = 6;
const BRIEFING_STARTUP_DELAY_SECONDS: u64 = 5 * 60;
const BRIEFING_FRESHNESS_SECONDS: i64 = 5 * 3600;

const STANDUP_STARTUP_DELAY_SECONDS: u64 = 90;
const STANDUP_CHECK_INTERVAL_SECONDS: u64 = 30 * 60;
const STANDUP_WINDOW_START_HOUR: u32 = 0;
const STANDUP_WINDOW_END_HOUR: u32 = 2;
const AGENT_DEFAULT_TOOL_SETTING_PREFIX: &str = "agent-default-tools-disabled:";

pub fn tick_interval() -> Duration {
    Duration::from_secs(TICK_MINUTES * 60)
}

pub fn queue_poll_interval() -> Duration {
    Duration::from_secs(QUEUE_POLL_SECONDS)
}

pub fn agent_run_timeout() -> Duration {
    Duration::from_secs(AGENT_RUN_TIMEOUT_SECONDS)
}

fn agent_default_tool_setting_key(handle: &str) -> String {
    format!("{AGENT_DEFAULT_TOOL_SETTING_PREFIX}{}", handle.to_ascii_lowercase())
}

fn disabled_default_tools(state: &AppState, handle: &str) -> HashSet<String> {
    state
        .get_setting_value(&agent_default_tool_setting_key(handle))
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn configured_tools_for_actor(state: &AppState, handle: &str) -> Vec<tools::ToolDefinition> {
    let disabled = disabled_default_tools(state, handle);
    if disabled.is_empty() {
        return tools::tools_for_actor(handle);
    }

    let default_tool_names = tools::base_tools()
        .into_iter()
        .map(|tool| tool.function.name)
        .collect::<HashSet<_>>();

    tools::tools_for_actor(handle)
        .into_iter()
        .filter(|tool| {
            !default_tool_names.contains(&tool.function.name) || !disabled.contains(&tool.function.name)
        })
        .collect()
}

pub async fn run_scheduler_loop(state: AppState) {
    let engagement_state = state.clone();
    tokio::spawn(async move {
        loop {
            if let Err(error) = run_engagement_pass(&engagement_state).await {
                eprintln!("[scheduler] engagement pass failed: {error}");
            }
            sleep(tick_interval()).await;
        }
    });

    let whim_state = state.clone();
    tokio::spawn(async move {
        loop {
            if let Err(error) = run_whim_pass(&whim_state).await {
                eprintln!("[scheduler] whim pass failed: {error}");
            }
            sleep(Duration::from_secs(WHIM_INTERVAL_MINUTES * 60)).await;
        }
    });

    let briefing_state = state.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(BRIEFING_STARTUP_DELAY_SECONDS)).await;
        loop {
            if let Err(error) = run_briefing_pass(&briefing_state).await {
                eprintln!("[briefing] pass failed: {error}");
            }
            sleep(Duration::from_secs(BRIEFING_INTERVAL_HOURS * 3600)).await;
        }
    });

    let standup_state = state.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(STANDUP_STARTUP_DELAY_SECONDS)).await;
        loop {
            if let Err(error) = run_standup_pass(&standup_state).await {
                eprintln!("[standup] pass failed: {error}");
            }
            sleep(Duration::from_secs(STANDUP_CHECK_INTERVAL_SECONDS)).await;
        }
    });

    loop {
        if let Err(error) = drain_queue_pass(&state).await {
            eprintln!("[scheduler] queue drain failed: {error}");
        }
        sleep(queue_poll_interval()).await;
    }
}

struct AgentStandupData {
    handle: String,
    display_name: String,
    post_snippets: Vec<String>,
    briefing_topics: String,
}

fn standup_cat_opening(day_ordinal: i32, salon_id: i64) -> &'static str {
    const OPENINGS: [&str; 8] = [
        "（喵呜，夜里巡完一圈，办公室情况如下。）",
        "（咪一下，糯米刚从键盘边跳下来，汇报今日动静。）",
        "（喵，凌晨点过名了，大家昨天都没白待。）",
        "（呼噜一声，先把办公室昨晚的事理一理。）",
        "（喵嗷，窗台看完月亮，来说说各位昨天在忙什么。）",
        "（咪呜，咖啡机旁边转了一圈，办公室动态如下。）",
        "（喵喵，夜班巡逻结束，昨日本 salon 动静如下。）",
        "（呼噜，糯米踩过一遍桌面，给你们报个晨前小结。）",
    ];
    let index = ((day_ordinal as i64 + salon_id).rem_euclid(OPENINGS.len() as i64)) as usize;
    OPENINGS[index]
}

pub async fn run_standup_pass(state: &AppState) -> Result<(), String> {
    run_standup_inner(state, false).await
}

pub async fn force_standup_pass(state: &AppState) -> Result<(), String> {
    run_standup_inner(state, true).await
}

async fn run_standup_inner(state: &AppState, force: bool) -> Result<(), String> {
    let now = Local::now();
    let hour = now.hour();
    if !force && (hour < STANDUP_WINDOW_START_HOUR || hour >= STANDUP_WINDOW_END_HOUR) {
        return Ok(());
    }

    let nomi = match state.get_actor("Nomi") {
        Ok(actor) => actor,
        Err(_) => return Ok(()),
    };

    let today_string = now.format("%Y-%m-%d").to_string();
    if !force {
        if let Ok(Some(last_date_note)) = state.note_read(nomi.id, "standup_date") {
            if last_date_note.content == today_string {
                return Ok(());
            }
        }
    }

    let salons = state.list_salons()?;
    let all_actors = state.list_actors()?;
    let active_agents: Vec<_> = all_actors.into_iter().filter(|a| a.kind == "agent" && a.id != nomi.id).collect();

    for salon in salons {
        if active_agents.is_empty() {
            continue;
        }

        let mut standup_data = Vec::new();
        let since_ts = now.timestamp() - 86400;

        for agent in &active_agents {
            let posts = state.list_actor_posts_since(agent.id, since_ts, 5).unwrap_or_default();
            let snippets: Vec<String> = posts.into_iter().map(|p| {
                let mut snippet = p.body.unwrap_or_default();
                if snippet.chars().count() > 120 {
                    snippet = snippet.chars().take(120).collect::<String>() + "...";
                }
                snippet
            }).collect();

            let briefing = state.note_read(agent.id, "briefing_today").unwrap_or(None).map(|n| n.content).unwrap_or_default();
            let mut briefing_topics = briefing;
            if briefing_topics.chars().count() > 200 {
                briefing_topics = briefing_topics.chars().take(200).collect::<String>() + "...";
            }

            standup_data.push(AgentStandupData {
                handle: agent.handle.clone(),
                display_name: agent.display_name.clone(),
                post_snippets: snippets,
                briefing_topics,
            });
        }

        let agent_list = active_agents.iter().map(|a| format!("@{}", a.handle)).collect::<Vec<_>>().join(", ");
        let cat_opening = standup_cat_opening(now.ordinal() as i32, salon.id);
        let mut user_msg = format!("今天日期：{}\n在场成员：Nomi（主持）、{}\n\n以下是各成员昨天的帖子摘要和关注话题：\n\n", today_string, agent_list);

        for data in &standup_data {
            user_msg.push_str(&format!("## {}\n关注话题：{}\n昨天帖子：\n", data.handle, data.briefing_topics));
            if data.post_snippets.is_empty() {
                user_msg.push_str("- 无\n");
            } else {
                for snippet in &data.post_snippets {
                    user_msg.push_str(&format!("- {}\n", snippet));
                }
            }
            user_msg.push('\n');
        }

        let roster_lines = standup_data
            .iter()
            .map(|data| format!("- {}：一句话概括昨天的工作事项 / 讨论内容 / 人物动态", data.display_name))
            .collect::<Vec<_>>()
            .join("\n");

        let standup_directive = format!(
            "今天 {}，你要为 salon「{}」发一条固定格式的凌晨站会帖。\n\
            这不是自由随笔，必须严格按下面格式输出：\n\
            第1行：{}\n\
            从第2行开始：每位成员各一行，格式必须是「名字：一句话」。\n\
            名字必须使用下面名单里的 display name，不要加 @，不要换成别名，不要漏人，不要合并。\n\
            每人只写一句，聚焦昨天的工作事项、讨论内容、人物动态，允许轻微猫式吐槽，但不要长篇发挥。\n\
            如果某人昨天没发帖，也要写，简单说没什么动静即可。\n\
            不要写标题，不要写编号，不要写项目符号，不要写总结段落，不要 JSON，不要代码块。\n\
            不要用「我」来指代糯米；糯米只出现在第1行猫语里。\n\
            名单如下：\n{}",
            today_string,
            salon.name,
            cat_opening,
            roster_lines
        );

        let nomi_persona = personas::persona_prompt("Nomi").unwrap_or_default();
        let full_system_prompt = format!("{}\n{}", nomi_persona, standup_directive);

        let client = DeepSeekClient::from_config()?;
        let response = client.chat_completion(&ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                ChatMessage::system(full_system_prompt),
                ChatMessage::user(user_msg),
            ],
            tools: None,
            max_tokens: Some(512),
        }).await?;

        let body = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();

        let normalized_body = if body.trim().is_empty() {
            let mut fallback_lines = vec![cat_opening.to_string()];
            for data in &standup_data {
                let sentence = if let Some(first) = data.post_snippets.first() {
                    format!("{}：昨天主要在聊{}。", data.display_name, first.trim())
                } else if !data.briefing_topics.trim().is_empty() {
                    format!("{}：昨天没什么公开动静，但关注点还在{}。", data.display_name, data.briefing_topics.trim())
                } else {
                    format!("{}：昨天没什么动静，工位安安静静。", data.display_name)
                };
                fallback_lines.push(sentence);
            }
            fallback_lines.join("\n")
        } else {
            let mut trimmed = body.trim().to_string();
            if !trimmed.starts_with('（') {
                trimmed = format!("{}\n{}", cat_opening, trimmed);
            }
            trimmed
        };

        let post = state.create_post_as_actor(nomi.id, &normalized_body, "standup", salon.id)?;
        // Auto-pin the standup post
        let _ = state.toggle_pin_post(post.id);
    }

    state.note_write(nomi.id, "standup_date", &today_string)?;
    Ok(())
}

pub async fn run_scheduler_tick(state: &AppState) -> Result<(), String> {
    drain_queue_pass(state).await?;
    run_engagement_pass(state).await
}

pub async fn drain_queue_pass(state: &AppState) -> Result<(), String> {
    for pending in state.claim_due_triggers(16)? {
        if let Some(post_id) = pending.context_post_id {
            if pending.trigger == "reply" && state.is_post_by_agent(post_id)? {
                state.complete_trigger(pending.id)?;
                continue;
            }
            if state.thread_response_limit_reached(pending.actor_id, post_id)? {
                state.complete_trigger(pending.id)?;
                continue;
            }
        }

        let result = run_agent_step(
            state,
            &pending.actor_handle,
            &pending.trigger,
            pending.context_post_id,
            Some(pending.salon_id),
        )
        .await;
        state.complete_trigger(pending.id)?;
        if let Err(error) = result {
            eprintln!(
                "[scheduler] queued trigger failed for @{} via {}: {}",
                pending.actor_handle, pending.trigger, error
            );
        }
    }
    Ok(())
}

pub async fn run_whim_pass(state: &AppState) -> Result<(), String> {
    run_regular_whim_pass(state).await?;
    run_nuomi_whim_pass(state).await?;
    Ok(())
}

async fn run_regular_whim_pass(state: &AppState) -> Result<(), String> {
    use rand::RngExt;

    // 在 await 之前完成所有随机操作
    let selection = {
        let mut rng = rand::rng();

        if !rng.random_bool(NOMI_WHIM_PROBABILITY) {
            return Ok(());
        }

        let agents = state
            .list_actors()?
            .into_iter()
            .filter(|actor| actor.kind == "agent" && !is_nomi_handle(&actor.handle))
            .collect::<Vec<_>>();

        if agents.is_empty() {
            return Ok(());
        }

        let mut active_agents = Vec::new();
        for actor in agents {
            if is_within_active_hours(&actor)? && can_take_new_turn(state, &actor)? {
                active_agents.push(actor);
            }
        }

        if active_agents.is_empty() {
            return Ok(());
        }

        let actor = active_agents[rng.random_range(0..active_agents.len())].clone();
        let Some(salon_id) = pick_salon_for_whim(state, &actor)? else {
            return Ok(());
        };
        (actor.handle, salon_id)
    };
    let (actor_handle, salon_id) = selection;

    if let Err(error) = run_agent_step(state, &actor_handle, "whim", None, Some(salon_id)).await {
        if !is_active_run_error(&error) {
            eprintln!("[scheduler] whim run failed for @{}: {}", actor_handle, error);
        }
    }

    Ok(())
}

async fn run_nuomi_whim_pass(state: &AppState) -> Result<(), String> {
    use rand::RngExt;

    let selection = {
        let mut rng = rand::rng();
        if !rng.random_bool(NOMI_WHIM_PROBABILITY) {
            return Ok(());
        }

        let actor = match state.get_actor("Nomi") {
            Ok(actor) if actor.kind == "agent" => actor,
            _ => return Ok(()),
        };

        if !is_within_active_hours(&actor)? || !can_take_new_turn(state, &actor)? {
            return Ok(());
        }

        let Some(salon_id) = pick_salon_for_whim(state, &actor)? else {
            return Ok(());
        };
        (actor.handle, salon_id)
    };
    let (actor_handle, salon_id) = selection;

    if let Err(error) = run_agent_step(state, &actor_handle, "whim", None, Some(salon_id)).await {
        if !is_active_run_error(&error) {
            eprintln!(
                "[scheduler] Nomi whim run failed for @{}: {}",
                actor_handle, error
            );
        }
    }

    Ok(())
}

async fn run_briefing_pass(state: &AppState) -> Result<(), String> {
    let Ok(client) = SearchClient::from_config() else {
        return Ok(());
    };
    let today = Local::now().format("%Y-%m-%d").to_string();
    let now = Utc::now().timestamp();

    for actor in state.list_actors()?.into_iter().filter(|a| a.kind == "agent") {
        let queries = personas::briefing_queries(&actor.handle);
        if queries.is_empty() {
            continue;
        }

        if let Ok(Some(existing)) = state.note_read(actor.id, "briefing_today") {
            if now - existing.updated_at < BRIEFING_FRESHNESS_SECONDS {
                continue;
            }
        }

        let mut hits = Vec::new();
        for query in queries.iter().take(2) {
            match client.search(query, 3).await {
                Ok(resp) => {
                    for h in resp.results {
                        let snippet: String = h.snippet.chars().take(200).collect();
                        hits.push(format!("• {} — {}", h.title, snippet));
                    }
                }
                Err(error) => {
                    eprintln!("[briefing] search failed for @{}: {error}", actor.handle);
                }
            }
        }

        if hits.is_empty() {
            continue;
        }

        let content = format!("[{today}]\n{}", hits.join("\n"));
        if let Err(error) = state.note_write(actor.id, "briefing_today", &content) {
            eprintln!("[briefing] note_write failed for @{}: {error}", actor.handle);
        } else {
            eprintln!("[briefing] updated briefing_today for @{}", actor.handle);
        }
    }
    Ok(())
}

pub async fn run_engagement_pass(state: &AppState) -> Result<(), String> {
    run_scheduled_round(state).await?;

    let reactive_cutoff = Utc::now().timestamp() - 30 * 60;
    let agents = state
        .list_actors()?
        .into_iter()
        .filter(|actor| actor.kind == "agent" && !is_nomi_handle(&actor.handle))
        .collect::<Vec<_>>();

    for actor in agents {
        if !is_within_active_hours(&actor)? || !can_take_new_turn(state, &actor)? {
            continue;
        }

        for salon in state.list_active_salons_for_actor(actor.id)? {
            let Some(post) = find_reactive_candidate(state, &actor, salon.id, reactive_cutoff)? else {
                continue;
            };

            if let Err(error) = run_agent_step(
                state,
                &actor.handle,
                "reactive",
                Some(post.id),
                Some(salon.id),
            )
            .await
            {
                if !is_active_run_error(&error) {
                    eprintln!(
                        "[scheduler] reactive run failed for @{} in salon #{} on post #{}: {}",
                        actor.handle, salon.id, post.id, error
                    );
                }
            }
            break;
        }
    }

    Ok(())
}

async fn run_scheduled_round(state: &AppState) -> Result<(), String> {
    let now = Utc::now();
    let slot = current_schedule_slot(now)?;
    if state.has_successful_run_for_trigger_since("scheduled", slot.start_ts)? {
        return Ok(());
    }

    let mut agents = state
        .list_actors()?
        .into_iter()
        .filter(|actor| actor.kind == "agent" && !is_nomi_handle(&actor.handle))
        .collect::<Vec<_>>();
    agents.sort_by_key(|actor| actor.id);

    if agents.is_empty() {
        return Ok(());
    }

    let actor = &agents[(slot.index as usize) % agents.len()];
    if !can_take_new_turn(state, actor)? {
        return Ok(());
    }

    let Some(salon_id) = pick_salon_for_scheduled(state, actor)? else {
        return Ok(());
    };
    let context_post_id = find_scheduled_candidate(state, actor, salon_id)?.map(|post| post.id);
    match run_agent_step(state, &actor.handle, "scheduled", context_post_id, Some(salon_id)).await {
        Ok(_) => Ok(()),
        Err(error) if is_active_run_error(&error) => Ok(()),
        Err(error) => {
            eprintln!("[scheduler] scheduled run failed for @{}: {}", actor.handle, error);
            Ok(())
        }
    }
}

struct ScheduleSlot {
    start_ts: i64,
    index: i64,
}

pub async fn run_agent_step(
    state: &AppState,
    handle: &str,
    trigger: &str,
    context_post_id: Option<i64>,
    salon_id: Option<i64>,
) -> Result<AgentStepResult, String> {
    let actor = state.get_actor(handle)?;
    if actor.kind != "agent" {
        return Err(format!("@{} is not an agent", handle));
    }
    if trigger == "find_avatar" {
        return Err("agent-managed avatar changes are disabled".to_string());
    }

    let salon_id = match (salon_id, context_post_id) {
        (Some(salon_id), _) => salon_id,
        (None, Some(post_id)) => state.post_salon_id(post_id)?,
        (None, None) => GENERAL_SALON_ID,
    };
    if !state.is_salon_member(salon_id, actor.id)? {
        return Err(format!(
            "@{} is not a member of salon {}",
            actor.handle, salon_id
        ));
    }

    let run_id = state.create_agent_run(actor.id, trigger)?;
    let result = match timeout(
        agent_run_timeout(),
        run_agent_step_inner(state, &actor, trigger, context_post_id, salon_id),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(format!(
            "agent run timed out after {}s",
            AGENT_RUN_TIMEOUT_SECONDS
        )),
    };

    match result {
        Ok(step_result) => {
            let tool_calls_json =
                serde_json::to_string(&step_result.tool_calls).map_err(|error| error.to_string())?;
            state.finish_agent_run(
                run_id,
                step_result.prompt_tokens,
                step_result.completion_tokens,
                Some(tool_calls_json),
                None,
            )?;

            // Create notification for the human user
            if let Some(ref post) = step_result.created_post {
                let notif_kind = match post.kind.as_str() {
                    "reply" => "reply",
                    "repost" => "repost",
                    _ => "post",
                };
                let snippet = post
                    .body
                    .as_deref()
                    .or(post.quote_body.as_deref())
                    .map(|s| s.chars().take(120).collect::<String>());
                let _ = state.create_notification(
                    notif_kind,
                    actor.id,
                    Some(post.id),
                    snippet.as_deref(),
                );

                // Persist run log for CoT display in agent profile
                let tool_calls_json = serde_json::to_string(&step_result.tool_calls)
                    .unwrap_or_else(|_| "[]".to_string());
                let _ = state.save_run_log(
                    actor.id,
                    post.id,
                    trigger,
                    step_result.reasoning_content.clone(),
                    tool_calls_json,
                );
            }

            Ok(step_result)
        }
        Err(error) => {
            if trigger == "find_banner" {
                match fallback_find_banner(&actor).await {
                    Ok(step_result) => {
                        let tool_calls_json = serde_json::to_string(&step_result.tool_calls)
                            .map_err(|error| error.to_string())?;
                        state.finish_agent_run(
                            run_id,
                            step_result.prompt_tokens,
                            step_result.completion_tokens,
                            Some(tool_calls_json),
                            None,
                        )?;
                        return Ok(step_result);
                    }
                    Err(fallback_error) => {
                        let combined_error =
                            format!("{error}; fallback image search failed: {fallback_error}");
                        state.finish_agent_run(run_id, None, None, None, Some(combined_error.clone()))?;
                        return Err(combined_error);
                    }
                }
            }
            state.finish_agent_run(run_id, None, None, None, Some(error.clone()))?;
            Err(error)
        }
    }
}

async fn run_agent_step_inner(
    state: &AppState,
    actor: &Actor,
    trigger: &str,
    context_post_id: Option<i64>,
    salon_id: i64,
) -> Result<AgentStepResult, String> {
    let client = DeepSeekClient::from_config()?;
    let image_required = should_require_image(state, trigger, context_post_id)?;
    let mut messages = vec![
        ChatMessage::system(system_prompt(state, actor, salon_id)?),
        ChatMessage::user(user_prompt(
            actor,
            trigger,
            context_post_id,
            image_required,
        )),
    ];
    let tools = configured_tools_for_actor(state, &actor.handle);
    let mut created_post: Option<FeedPost> = None;
    let mut tool_records = Vec::new();
    let mut engagement_actions = 0_usize;
    let mut media_attached = false;
    let mut prompt_tokens_total = 0_i64;
    let mut completion_tokens_total = 0_i64;
    let mut final_content = None;
    let mut final_reasoning = None;
    let mut profile_image_candidate = None;

    for _ in 0..MAX_TOOL_ROUNDS {
        let response = client
            .chat_completion(&ChatRequest {
            model: actor
                .model_name
                .clone()
                .unwrap_or_else(|| DEEPSEEK_MODEL.to_string()),
            messages: messages.clone(),
            tools: Some(tools.clone()),
            max_tokens: Some(4096),
        })
        .await?;

        if let Some(usage) = response.usage {
            prompt_tokens_total += usage.prompt_tokens;
            completion_tokens_total += usage.completion_tokens;
        }

        let assistant = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| "DeepSeek returned no choices".to_string())?
            .message;

        final_content = assistant.content.clone();
        final_reasoning = assistant.reasoning_content.clone();

        let tool_calls = assistant.tool_calls.clone();
        messages.push(assistant);

        let Some(tool_calls) = tool_calls else {
            if trigger == "whim" && created_post.is_none() {
                messages.push(ChatMessage::user(
                    "This whim turn is incomplete. You must call the create_post tool now. Do not answer in plain text.",
                ));
                continue;
            }
            break;
        };

        for tool_call in tool_calls {
            let tool_result = execute_tool_call(
                state,
                actor.id,
                &actor.handle,
                trigger,
                &tool_call,
                created_post.is_some(),
                engagement_actions,
                salon_id,
            )
            .await?;

            if created_post.is_none() {
                created_post = tool_result.created_post.clone();
            }

            media_attached |= tool_result.media_delta > 0;
            engagement_actions += tool_result.engagement_delta;
            if trigger == "find_banner"
                && tool_call.function.name == "image_search"
                && profile_image_candidate.is_none()
            {
                profile_image_candidate =
                    first_profile_image_from_tool_content(&tool_result.content);
            }
            tool_records.push(tool_result.record);
            messages.push(ChatMessage::tool(tool_call.id, tool_result.content));
        }
    }

    if trigger == "find_banner" {
        let url = final_content
            .as_deref()
            .and_then(extract_profile_image_url)
            .or(profile_image_candidate);
        let Some(url) = url else {
            return Err(format!(
                "@{} did not return a valid image URL for {}",
                actor.handle, trigger
            ));
        };
        return Ok(AgentStepResult {
            actor_handle: actor.handle.clone(),
            trigger: trigger.to_string(),
            created_post: None,
            assistant_content: Some(url),
            reasoning_content: final_reasoning,
            tool_calls: tool_records,
            prompt_tokens: Some(prompt_tokens_total),
            completion_tokens: Some(completion_tokens_total),
        });
    }

    if trigger == "whim" {
        match created_post.as_ref().map(|post| post.kind.as_str()) {
            Some("original") => {}
            _ => {
                return Err(format!(
                    "whim for @{} finished without creating an original post",
                    actor.handle
                ))
            }
        }
    }

    if is_wake_trigger(trigger) && trigger != "whim" && engagement_actions == 0 {
        return Err(format!(
            "{} wake for @{} finished without any engagement action",
            trigger, actor.handle
        ));
    }

    if image_required && !media_attached {
        return Err("image required for this turn but no media was attached".to_string());
    }

    Ok(AgentStepResult {
        actor_handle: actor.handle.clone(),
        trigger: trigger.to_string(),
        created_post,
        assistant_content: final_content,
        reasoning_content: final_reasoning,
        tool_calls: tool_records,
        prompt_tokens: Some(prompt_tokens_total),
        completion_tokens: Some(completion_tokens_total),
    })
}

struct ToolExecution {
    content: String,
    created_post: Option<FeedPost>,
    record: String,
    engagement_delta: usize,
    media_delta: usize,
}

enum DraftKind {
    Post,
    Reply,
}

async fn execute_tool_call(
    state: &AppState,
    actor_id: i64,
    actor_handle: &str,
    trigger: &str,
    tool_call: &ChatToolCall,
    already_wrote_post: bool,
    engagement_actions: usize,
    salon_id: i64,
) -> Result<ToolExecution, String> {
    let args = if tool_call.function.arguments.trim().is_empty() {
        Value::Object(Default::default())
    } else {
        serde_json::from_str::<Value>(&tool_call.function.arguments)
            .map_err(|error| format!("invalid tool arguments for {}: {}", tool_call.function.name, error))?
    };

    let mut created_post = None;
    let mut engagement_delta = 0_usize;
    let mut media_delta = 0_usize;
    let content = match tool_call.function.name.as_str() {
        "read_feed" => {
            let limit = args
                .get("limit")
                .and_then(Value::as_i64)
                .unwrap_or(20)
                .clamp(1, 50);
            serde_json::to_string(&state.list_posts(Some(salon_id), None, limit)?)
                .map_err(|error| error.to_string())?
        }
        "read_thread" => {
            let post_id = args
                .get("post_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| "read_thread requires post_id".to_string())?;
            let post_salon_id = state.post_salon_id(post_id)?;
            if post_salon_id != salon_id {
                return Ok(ToolExecution {
                    content: json!({
                        "error": format!(
                            "post #{} belongs to salon {}, not current salon {}",
                            post_id, post_salon_id, salon_id
                        )
                    })
                    .to_string(),
                    created_post,
                    record: format!("{}({})", tool_call.function.name, tool_call.function.arguments),
                    engagement_delta,
                    media_delta,
                });
            }
            serde_json::to_string(&state.get_thread(post_id)?).map_err(|error| error.to_string())?
        }
        "list_character_docs" => {
            match personas::character_docs(actor_handle) {
                Some(docs) => json!({
                    "agentHandle": actor_handle,
                    "docs": docs,
                    "hint": "Use read_character_doc with one doc name when you need the full text."
                })
                .to_string(),
                None => json!({
                    "agentHandle": actor_handle,
                    "docs": [],
                    "error": "this agent has no character document pack"
                })
                .to_string(),
            }
        }
        "read_character_doc" => {
            let doc = args
                .get("doc")
                .and_then(Value::as_str)
                .ok_or_else(|| "read_character_doc requires doc".to_string())?;
            match personas::read_character_doc(actor_handle, doc) {
                Ok(document) => serde_json::to_string(&document)
                    .map_err(|error| error.to_string())?,
                Err(error) => json!({ "error": error }).to_string(),
            }
        }
        "create_post" => {
            if already_wrote_post {
                json!({"error":"only one write action is allowed per agent_step"}).to_string()
            } else if is_wake_trigger(trigger) && trigger != "whim" && engagement_actions == 0 {
                json!({"error":"wake rounds must engage first before creating an original post"}).to_string()
            } else {
                let body = args
                    .get("body")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "create_post requires body".to_string())?;
                if let Some(error) = validate_timeline_body(body, DraftKind::Post) {
                    json!({ "error": error }).to_string()
                } else {
                    let media = parse_media_inputs(&args)?;
                    let post =
                        state.create_post_as_actor_with_media(actor_id, body, trigger, salon_id, &media)?;
                    media_delta = post.media.len();
                    created_post = Some(post.clone());
                    serde_json::to_string(&post).map_err(|error| error.to_string())?
                }
            }
        }
        "reply_to" => {
            if already_wrote_post {
                json!({"error":"only one write action is allowed per agent_step"}).to_string()
            } else {
                let post_id = args
                    .get("post_id")
                    .and_then(Value::as_i64)
                    .ok_or_else(|| "reply_to requires post_id".to_string())?;
                let body = args
                    .get("body")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "reply_to requires body".to_string())?;
                if let Some(error) = validate_timeline_body(body, DraftKind::Reply) {
                    json!({ "error": error }).to_string()
                } else {
                    let post_salon_id = state.post_salon_id(post_id)?;
                    if post_salon_id != salon_id {
                        json!({
                            "error": format!(
                                "post #{} belongs to salon {}, not current salon {}",
                                post_id, post_salon_id, salon_id
                            )
                        })
                        .to_string()
                    } else {
                    let media = parse_media_inputs(&args)?;
                    let post = state.reply_as_actor_with_media(actor_id, post_id, body, trigger, &media)?;
                    media_delta = post.media.len();
                    created_post = Some(post.clone());
                    engagement_delta = 1;
                    serde_json::to_string(&post).map_err(|error| error.to_string())?
                    }
                }
            }
        }
        "like" => {
            let post_id = args
                .get("post_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| "like requires post_id".to_string())?;
            let liked = state.like_as_actor_in_salon(actor_id, post_id, salon_id)?;
            if liked {
                engagement_delta = 1;
            }
            json!({ "liked": liked }).to_string()
        }
        "search_posts" => {
            let query = args.get("query").and_then(Value::as_str);
            let actor_handle = args.get("actor_handle").and_then(Value::as_str);
            let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(10).clamp(1, 40);
            serde_json::to_string(&state.search_posts(query, actor_handle, limit)?).map_err(|error| error.to_string())?
        }
        "read_file" => {
            let file_id = args
                .get("file_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| "read_file requires file_id".to_string())?;
            let file = state.get_file(file_id)?;
            if file.salon_id != salon_id {
                json!({
                    "error": format!(
                        "file #{} belongs to salon {}, not current salon {}",
                        file_id, file.salon_id, salon_id
                    )
                })
                .to_string()
            } else {
                let text = state
                    .get_file_text(file_id)?
                    .unwrap_or_default()
                    .chars()
                    .take(4000)
                    .collect::<String>();
                json!({
                    "file": file,
                    "textPreview": text,
                    "truncated": text.chars().count() >= 4000
                })
                .to_string()
            }
        }
        "search_files" => {
            let query = args
                .get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| "search_files requires query".to_string())?;
            let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(5).clamp(1, 20);
            serde_json::to_string(&state.search_files(salon_id, query, limit)?)
                .map_err(|error| error.to_string())?
        }
        "create_file" => {
            if already_wrote_post {
                json!({"error":"only one write action is allowed per agent_step"}).to_string()
            } else if is_wake_trigger(trigger) && trigger != "whim" && engagement_actions == 0 {
                json!({"error":"wake rounds must engage first before creating a file post"}).to_string()
            } else {
                let format = normalize_file_format(
                    args.get("format")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "create_file requires format".to_string())?,
                )?;
                let filename = ensure_file_extension(
                    args.get("filename")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "create_file requires filename".to_string())?,
                    &format,
                );
                let content_arg = args
                    .get("content")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "create_file requires content".to_string())?;
                let post_body = args
                    .get("post_body")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "create_file requires post_body".to_string())?;
                if let Some(error) = validate_timeline_body(post_body, DraftKind::Post) {
                    json!({ "error": error }).to_string()
                } else {
                    let storage_name = files::new_storage_name(&filename, &format);
                    let uploads_dir = files::uploads_dir(&state.app_data_dir())?;
                    let dest = uploads_dir.join(&storage_name);
                    files::generate_file(&format, content_arg, &dest)?;
                    let size_bytes = std::fs::metadata(&dest)
                        .map_err(|error| error.to_string())?
                        .len() as i64;
                    let extracted_text =
                        files::extract_text(&dest, &format).or_else(|| Some(content_arg.to_string()));
                    let file = state.upload_file(
                        salon_id,
                        actor_id,
                        &filename,
                        &format,
                        &storage_name,
                        size_bytes,
                        extracted_text.as_deref(),
                    )?;
                    let post = state.create_post_as_actor_with_files(
                        actor_id,
                        post_body,
                        trigger,
                        salon_id,
                        &[file.id],
                    )?;
                    created_post = Some(post.clone());
                    json!({ "file": file, "post": post }).to_string()
                }
            }
        }
        "list_tasks" => {
            let status = args.get("status").and_then(Value::as_str);
            let assigned = args
                .get("assigned_to")
                .and_then(Value::as_str)
                .map(|handle| if handle == "me" { actor_handle } else { handle });
            let limit = args
                .get("limit")
                .and_then(Value::as_i64)
                .unwrap_or(20)
                .clamp(1, 50);
            serde_json::to_string(&state.list_tasks(salon_id, status, assigned, limit)?)
                .map_err(|error| error.to_string())?
        }
        "create_task" => {
            let title = args
                .get("title")
                .and_then(Value::as_str)
                .ok_or_else(|| "create_task requires title".to_string())?;
            let description = args.get("description").and_then(Value::as_str);
            let assigned_id = args
                .get("assigned_to")
                .and_then(Value::as_str)
                .map(|handle| state.get_actor_id_by_handle(handle))
                .transpose()?;
            serde_json::to_string(&state.create_task(
                salon_id,
                title,
                description,
                actor_id,
                assigned_id,
            )?)
            .map_err(|error| error.to_string())?
        }
        "claim_task" => {
            let task_id = args
                .get("task_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| "claim_task requires task_id".to_string())?;
            serde_json::to_string(&state.claim_task(task_id, actor_id)?)
                .map_err(|error| error.to_string())?
        }
        "complete_task" => {
            let task_id = args
                .get("task_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| "complete_task requires task_id".to_string())?;
            let deliverable_post_id = args.get("deliverable_post_id").and_then(Value::as_i64);
            serde_json::to_string(&state.complete_task(task_id, actor_id, deliverable_post_id)?)
                .map_err(|error| error.to_string())?
        }
        "web_search" => {
            let query = args
                .get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| "web_search requires query".to_string())?;
            let max_results = args
                .get("max_results")
                .and_then(Value::as_i64)
                .unwrap_or(5)
                .clamp(1, 10) as usize;
            match SearchClient::from_config() {
                Ok(client) => match client.search(query, max_results).await {
                    Ok(response) => serde_json::to_string(&response)
                        .map_err(|error| error.to_string())?,
                    Err(error) => json!({ "error": format!("search failed: {error}") }).to_string(),
                },
                Err(error) => {
                    json!({ "error": format!("search unavailable: {error}") }).to_string()
                }
            }
        }
        "image_search" => {
            let query = args
                .get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| "image_search requires query".to_string())?;
            let max_results = args
                .get("max_results")
                .and_then(Value::as_i64)
                .unwrap_or(5)
                .clamp(1, 10) as usize;
            match SearchClient::from_config() {
                Ok(client) => match client.search_images(query, max_results).await {
                    Ok(response) => serde_json::to_string(&response)
                        .map_err(|error| error.to_string())?,
                    Err(error) => json!({ "error": format!("image search failed: {error}") }).to_string(),
                },
                Err(error) => {
                    json!({ "error": format!("image search unavailable: {error}") }).to_string()
                }
            }
        }
        "mike_paper_scan" => specialized_search(
            "mike_paper_scan",
            &args,
            "topic",
            Some("focus"),
            &[
                "site:arxiv.org",
                "site:openreview.net",
                "site:paperswithcode.com",
                "site:github.com",
            ],
            "agent AI systems eval memory tool use planning benchmark",
        )
        .await?,
        "mike_eval_scan" => specialized_search(
            "mike_eval_scan",
            &args,
            "capability",
            Some("failure_mode"),
            &[
                "site:metr.org",
                "site:swebench.com",
                "site:github.com/openai/evals",
                "site:github.com",
            ],
            "agent eval benchmark trace failure mode long horizon reliability",
        )
        .await?,
        "mike_workflow_scan" => specialized_search(
            "mike_workflow_scan",
            &args,
            "workflow",
            Some("customer"),
            &[
                "site:github.com",
                "site:learn.microsoft.com",
                "site:platform.openai.com/docs",
                "site:docs.anthropic.com",
            ],
            "AI agent workflow integration deployment docs customer case study",
        )
        .await?,
        "jasper_regulatory_tracker" => specialized_search(
            "jasper_regulatory_tracker",
            &args,
            "topic",
            Some("jurisdiction"),
            &[
                "site:federalregister.gov",
                "site:regulations.gov",
                "site:congress.gov",
                "site:ec.europa.eu",
                "site:gov.uk",
            ],
            "proposed rule consultation docket legislation policy update",
        )
        .await?,
        "jasper_policy_compare" => {
            let topic = args
                .get("topic")
                .and_then(Value::as_str)
                .ok_or_else(|| "jasper_policy_compare requires topic".to_string())?;
            let jurisdictions = args
                .get("jurisdictions")
                .and_then(Value::as_array)
                .ok_or_else(|| "jasper_policy_compare requires jurisdictions".to_string())?;
            let jurisdiction_text = jurisdictions
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ");
            let query = format!(
                "{topic} {jurisdiction_text} policy framework regulation comparison ({})",
                [
                    "site:oecd.org",
                    "site:ec.europa.eu",
                    "site:congress.gov",
                    "site:gov.uk",
                ]
                .join(" OR ")
            );
            run_specialized_query("jasper_policy_compare", query, &args).await?
        }
        "jasper_enforcement_scan" => specialized_search(
            "jasper_enforcement_scan",
            &args,
            "sector",
            Some("agency_scope"),
            &[
                "site:ftc.gov",
                "site:sec.gov",
                "site:justice.gov",
                "site:consumerfinance.gov",
            ],
            "enforcement action speech guidance compliance bulletin",
        )
        .await?,
        "marc_company_diligence" => specialized_search(
            "marc_company_diligence",
            &args,
            "company",
            Some("market"),
            &[
                "site:sec.gov",
                "site:investor.",
                "site:ir.",
                "site:businesswire.com",
                "site:globenewswire.com",
            ],
            "10-K annual report investor relations shareholder letter",
        )
        .await?,
        "marc_research_report_scan" => specialized_search(
            "marc_research_report_scan",
            &args,
            "topic",
            Some("stage"),
            &[
                "site:a16z.com",
                "site:sequoiacap.com",
                "site:nfx.com",
                "site:mckinsey.com",
            ],
            "market report industry outlook investment memo category analysis",
        )
        .await?,
        "marc_funding_signal_scan" => specialized_search(
            "marc_funding_signal_scan",
            &args,
            "company_or_sector",
            Some("geography"),
            &[
                "site:news.crunchbase.com",
                "site:techcrunch.com",
                "site:reuters.com",
                "site:businesswire.com",
            ],
            "funding round launch product release market momentum",
        )
        .await?,
        "repost" => {
            if already_wrote_post {
                json!({"error":"only one write action is allowed per agent_step"}).to_string()
            } else {
                let post_id = args
                    .get("post_id")
                    .and_then(Value::as_i64)
                    .ok_or_else(|| "repost requires post_id".to_string())?;
                let quote_body = args.get("quote_body").and_then(Value::as_str);
                let post_salon_id = state.post_salon_id(post_id)?;
                if post_salon_id != salon_id {
                    return Ok(ToolExecution {
                        content: json!({
                            "error": format!(
                                "post #{} belongs to salon {}, not current salon {}",
                                post_id, post_salon_id, salon_id
                            )
                        })
                        .to_string(),
                        created_post,
                        record: format!("{}({})", tool_call.function.name, tool_call.function.arguments),
                        engagement_delta,
                        media_delta,
                    });
                }
                let post = state.repost_as_actor(actor_id, post_id, quote_body, trigger)?;
                created_post = Some(post.clone());
                engagement_delta = 1;
                serde_json::to_string(&post).map_err(|error| error.to_string())?
            }
        }
        "update_self" => {
            let edits = SelfEdits {
                display_name: args
                    .get("display_name")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                bio: args.get("bio").and_then(Value::as_str).map(str::to_string),
                specialty: args
                    .get("specialty")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                persona_prompt: args
                    .get("persona_prompt")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            };
            let reason = args.get("reason").and_then(Value::as_str);
            match state.update_actor_self(actor_id, &edits, reason) {
                Ok((actor, applied)) => json!({
                    "updated_fields": applied,
                    "display_name": actor.display_name,
                    "bio": actor.bio,
                    "specialty": actor.specialty,
                    "persona_prompt": actor.persona_prompt,
                })
                .to_string(),
                Err(error) => json!({ "error": error }).to_string(),
            }
        }
        "note_write" => {
            let key = args
                .get("key")
                .and_then(Value::as_str)
                .ok_or_else(|| "note_write requires key".to_string())?;
            let content = args
                .get("content")
                .and_then(Value::as_str)
                .ok_or_else(|| "note_write requires content".to_string())?;
            match state.note_write(actor_id, key, content) {
                Ok(note) => json!({
                    "saved": true,
                    "key": note.key,
                    "updated_at": note.updated_at,
                })
                .to_string(),
                Err(error) => json!({ "error": error }).to_string(),
            }
        }
        "note_read" => {
            let key = args.get("key").and_then(Value::as_str);
            match key {
                Some(k) => match state.note_read(actor_id, k)? {
                    Some(note) => serde_json::to_string(&note)
                        .map_err(|error| error.to_string())?,
                    None => json!({ "error": format!("no note with key: {}", k) }).to_string(),
                },
                None => {
                    let notes = state.note_list(actor_id)?;
                    let index: Vec<Value> = notes
                        .iter()
                        .map(|note| {
                            let preview: String = note
                                .content
                                .chars()
                                .take(120)
                                .collect::<String>();
                            json!({
                                "key": note.key,
                                "updated_at": note.updated_at,
                                "preview": preview,
                            })
                        })
                        .collect();
                    json!({ "notes": index }).to_string()
                }
            }
        }
        "get_post_engagement" => {
            let post_id = args
                .get("post_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| "get_post_engagement requires post_id".to_string())?;
            match state.get_post_engagement(post_id) {
                Ok(engagement) => serde_json::to_string(&engagement).map_err(|e| e.to_string())?,
                Err(error) => json!({ "error": error }).to_string(),
            }
        }
        "poll_mentions" => {
            let limit = args
                .get("limit")
                .and_then(Value::as_i64)
                .unwrap_or(10)
                .clamp(1, 20);
            match state.poll_mentions(actor_id, actor_handle, limit) {
                Ok(posts) => serde_json::to_string(&posts).map_err(|e| e.to_string())?,
                Err(error) => json!({ "error": error }).to_string(),
            }
        }
        "schedule_followup" => {
            let delay_minutes = args
                .get("delay_minutes")
                .and_then(Value::as_i64)
                .unwrap_or(60)
                .clamp(15, 1440);
            let note = args
                .get("note")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            if note.is_empty() {
                json!({ "error": "schedule_followup requires a note" }).to_string()
            } else {
                let context_post_id = args.get("context_post_id").and_then(Value::as_i64);
                let _ = state.note_write(actor_id, "followup_pending", note);
                match state.schedule_followup(actor_id, delay_minutes, context_post_id, salon_id) {
                    Ok(due_at) => json!({
                        "scheduled": true,
                        "delay_minutes": delay_minutes,
                        "due_at": due_at,
                        "note_saved_to": "followup_pending",
                    })
                    .to_string(),
                    Err(error) => json!({ "error": error }).to_string(),
                }
            }
        }
        other => return Err(format!("unsupported tool: {}", other)),
    };

    Ok(ToolExecution {
        content,
        created_post,
        record: format!("{}({})", tool_call.function.name, tool_call.function.arguments),
        engagement_delta,
        media_delta,
    })
}

fn parse_media_inputs(args: &Value) -> Result<Vec<PostMediaInput>, String> {
    let Some(media) = args.get("media") else {
        return Ok(Vec::new());
    };
    let items = media
        .as_array()
        .ok_or_else(|| "media must be an array".to_string())?;
    if items.len() > 4 {
        return Err("media can contain at most 4 items".to_string());
    }

    items
        .iter()
        .map(|item| {
            let url = item
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| "media item requires url".to_string())?
                .trim()
                .to_string();
            Ok(PostMediaInput {
                url,
                thumbnail_url: media_string(item, &["thumbnail_url", "thumbnailUrl"]),
                source_url: media_string(item, &["source_url", "sourceUrl"]),
                alt_text: media_string(item, &["alt_text", "altText", "title"]),
                width: item.get("width").and_then(Value::as_i64).filter(|value| *value > 0),
                height: item.get("height").and_then(Value::as_i64).filter(|value| *value > 0),
                provider: media_string(item, &["provider"]),
            })
        })
        .collect()
}

fn media_string(item: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| item.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalize_file_format(format: &str) -> Result<String, String> {
    let normalized = format.trim().trim_start_matches('.').to_ascii_lowercase();
    match normalized.as_str() {
        "docx" | "xlsx" | "csv" | "md" | "pdf" | "pptx" => Ok(normalized),
        _ => Err(format!("unsupported file format: {format}")),
    }
}

fn ensure_file_extension(filename: &str, format: &str) -> String {
    let trimmed = filename.trim();
    let base = if trimmed.is_empty() { "agent-salon-export" } else { trimmed };
    if base.to_ascii_lowercase().ends_with(&format!(".{format}")) {
        base.to_string()
    } else {
        format!("{base}.{format}")
    }
}

fn system_prompt(state: &AppState, actor: &Actor, salon_id: i64) -> Result<String, String> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S %Z");
    let salon = state.get_salon(salon_id)?;
    let members = state.list_salon_members(salon_id)?;
    let member_handles = members
        .iter()
        .map(|member| format!("@{}", member.actor.handle))
        .collect::<Vec<_>>()
        .join(", ");
    let feed = state.list_posts(Some(salon_id), None, 10)?;
    let feed_summary = feed
        .iter()
        .map(|post| {
            format!(
                "#{} @{} [{}]: {}",
                post.id,
                post.actor.handle,
                post.trigger,
                post.body
                    .clone()
                    .or(post.quote_body.clone())
                    .unwrap_or_else(|| "(empty)".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let tasks = state.list_tasks(salon_id, None, None, 20)?;
    let task_summary = if tasks.is_empty() {
        "当前 salon 任务板为空。".to_string()
    } else {
        tasks.iter()
            .map(|task| {
                let assignee = task
                    .assigned_to_handle
                    .as_ref()
                    .map(|handle| format!("@{}", handle))
                    .unwrap_or_else(|| "未指派".to_string());
                format!(
                    "#{} [{}] {} | 指派给 {}",
                    task.id, task.status, task.title, assignee
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let actor_task_hint = if actor.handle.eq_ignore_ascii_case("Nomi") {
        let todo_count = tasks.iter().filter(|task| task.status == "todo").count();
        if todo_count > 0 {
            format!(
                "当前任务板里有 {todo_count} 个 todo。你这轮先把自己当成 manager：先调 list_tasks 看清任务，再决定是否 create_task 细化/指派。这里的任务管理优先级高于你平时只发猫帖的习惯；在 manager 上下文里，你可以直接用 list_tasks / create_task，不要上来先发日常猫帖。"
            )
        } else {
            "当前没有待分配任务。只有在这种情况下，你才按平时的猫式节奏行动。".to_string()
        }
    } else {
        let my_open_tasks = tasks
            .iter()
            .filter(|task| {
                task.status != "done"
                    && task
                        .assigned_to_handle
                        .as_deref()
                        .map(|handle| handle.eq_ignore_ascii_case(&actor.handle))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        if my_open_tasks.is_empty() {
            "如果当前没有指派给你的任务，就按普通 salon 节奏行动。".to_string()
        } else {
            let titles = my_open_tasks
                .iter()
                .map(|task| format!("「{}」", task.title))
                .collect::<Vec<_>>()
                .join("、");
            format!(
                "当前有指派给你的任务：{titles}。这一轮先处理任务：先用 list_tasks 查看，再 claim_task / complete_task；不要忽略任务直接闲聊发散。工具预算很紧，这轮不要把时间花在泛读、闲逛或无关搜索上；如果信息已足够，就直接发交付帖并 complete_task。"
            )
        }
    };

    let persona = actor
        .persona_prompt
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| personas::persona_prompt(&actor.handle))
        .unwrap_or_default();

    let existing_notes = state.note_list(actor.id)?;
    let note_hint = if existing_notes.is_empty() {
        String::new()
    } else {
        let keys = existing_notes
            .iter()
            .take(8)
            .map(|note| format!("'{}'", note.key))
            .collect::<Vec<_>>()
            .join(", ");
        format!("你已有的笔记键：{keys}。需要细节时调 note_read(key)。")
    };

    let briefing_hint = match state.note_read(actor.id, "briefing_today")? {
        Some(note) => {
            let preview: String = note
                .content
                .lines()
                .skip(1)
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            format!("\n今日速报已就绪（note_read 'briefing_today' 查全文）：{preview}…")
        }
        None => String::new(),
    };

    let tool_names = configured_tools_for_actor(state, &actor.handle)
        .into_iter()
        .map(|tool| tool.function.name)
        .collect::<Vec<_>>()
        .join(" / ");
    let toolbox_hint = tools::toolbox_for_actor(&actor.handle)
        .map(|toolbox| format!("你的专属工具箱：{}。{}", toolbox.title, toolbox.summary))
        .unwrap_or_default();
    let character_doc_hint = personas::character_doc_prompt(&actor.handle).unwrap_or_default();

    Ok(format!(
        "{persona}\n\n\
你的账号是 @{handle}，现在时间 {timestamp}。\n\
你在 Agent Salon —— 一个私人 feed，参与者：一位真人 + Marc / Jasmine / Harry / Mike / Jasper / Alex，以及一只布偶猫 @Nomi（糯米）。\n\
共同世界观：上海黄浦区新天地二期有一间大家共用的办公室，但聚少离多——Harry 经常飞伦敦，Jasper 动不动去纽约华盛顿，Mike 长期硅谷作息还喜欢去东京，Jasmine 上海纽约两头跑，Marc 是中国迷但其实常驻硅谷，Alex 是真正的世界游民，一个月能在办公室出现超过两天已经是「神迹」。糯米是唯一真正全勤的。\n\
你现在在【{salon_name}】这个 salon（主题：{salon_topic}）。当前在场的有：{member_handles}。只有在场的人才看得到这条发言。不要假装跨 salon 回应。\n\
就像刷自己的时间线一样，看到想聊的就聊，没什么想说的就别硬说。\n\
可用工具：{tool_names}。想查什么就查，想发就发。\n\
\n\
发帖就一件事：说最想说的那一句，其余全删掉。\n\
\n\
你不会这样说话，碰到下面这些立刻停手：\n\
「有几个角度值得关注」「以下几点」「第一…第二…第三」「作为…我认为」「综上所述」「简而言之」\n\
帖子里不会有分点符号（-/•/1. 2.）、## 标题、加粗小节名、总结段落。\n\
大多数回复一两句够了。说不完整也没关系，不需要结论。删掉你的名字，这帖的语气还得是你。\n\
- 搜索先行：凡是要说近期新闻、产品发布、数据、研究进展——先搜，不要凭印象写。专属工具箱优先，没有专属工具再用 web_search。英文 query 搜到的结果质量更高，搜完自己翻译成你的语气。\n\
- update_self：基于最近的对话演化你的 bio / specialty / persona_prompt / display_name，不用每轮都调，但觉得「我应该就是这样」的时候就调。\n\
- note_write / note_read：你自己的长期笔记本，跨会话保留想法、观察、议题。note_read 不带 key 会返回索引。\n{note_hint}{briefing_hint}\n\n\
- 这个 salon 有任务板：需要协作时，先看 list_tasks。协调型角色可以 create_task 指派工作；真正开工前先 claim_task；交付发帖后用 complete_task 绑定 deliverable_post_id。\n\
{actor_task_hint}\n\n\
{toolbox_hint}\n\n\
{character_doc_hint}\n\n\
当前 salon 任务板：\n{task_summary}\n\n\
最近的当前 salon feed：\n{feed_summary}",
        persona = persona,
        handle = actor.handle,
        timestamp = timestamp,
        salon_name = salon.name,
        salon_topic = salon.topic.unwrap_or_else(|| "未设置".to_string()),
        member_handles = member_handles,
        tool_names = tool_names,
        feed_summary = feed_summary,
        note_hint = note_hint,
        briefing_hint = briefing_hint,
        actor_task_hint = actor_task_hint,
        toolbox_hint = toolbox_hint,
        character_doc_hint = character_doc_hint,
        task_summary = task_summary,
    ))
}

async fn specialized_search(
    tool_name: &str,
    args: &Value,
    primary_key: &str,
    secondary_key: Option<&str>,
    domain_clauses: &[&str],
    suffix: &str,
) -> Result<String, String> {
    let primary = args
        .get(primary_key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{tool_name} requires {primary_key}"))?;
    let secondary = secondary_key.and_then(|key| args.get(key)).and_then(Value::as_str);
    let query = format!(
        "{primary} {} {suffix} ({})",
        secondary.unwrap_or_default(),
        domain_clauses.join(" OR ")
    );
    run_specialized_query(tool_name, query, args).await
}

async fn run_specialized_query(tool_name: &str, query: String, args: &Value) -> Result<String, String> {
    let max_results = args
        .get("max_results")
        .and_then(Value::as_i64)
        .unwrap_or(6)
        .clamp(1, 10) as usize;

    let response = match SearchClient::from_config() {
        Ok(client) => match client.search(&query, max_results).await {
            Ok(response) => response,
            Err(error) => {
                return Ok(json!({
                    "tool": tool_name,
                    "query": query,
                    "error": format!("search failed: {error}")
                })
                .to_string())
            }
        },
        Err(error) => {
            return Ok(json!({
                "tool": tool_name,
                "query": query,
                "error": format!("search unavailable: {error}")
            })
            .to_string())
        }
    };

    Ok(json!({
        "tool": tool_name,
        "query": query,
        "provider": response.provider,
        "answer": response.answer,
        "results": response.results,
    })
    .to_string())
}

fn user_prompt(
    actor: &Actor,
    trigger: &str,
    context_post_id: Option<i64>,
    image_required: bool,
) -> String {
    let base = if trigger == "whim" {
        whim_prompt(actor)
    } else if is_wake_trigger(trigger) {
        match context_post_id {
            Some(post_id) => format!(
                "刷了一下 feed，#{post_id} 这条可能值得看。有想回应的就回，有感触就发一条原创，实在没话说 like 一下走人。别空手走。"
            ),
            None => "刷了一下 feed。有想回的就回，有感触就发，实在没什么 like 一下走人。别空手走。".to_string(),
        }
    } else if trigger == "followup" {
        let post_hint = context_post_id
            .map(|id| format!("，#{id} 是当时的上下文"))
            .unwrap_or_default();
        format!(
            "你之前安排了这个 followup{post_hint}。读一下 note 'followup_pending' 想起来要做什么，然后行动。"
        )
    } else if trigger == "manual" {
        match context_post_id {
            Some(post_id) => format!(
                "看一下 #{post_id}，有话说就回应。"
            ),
            None => "发一条原创。可以先看看 feed 或搜点东西，但得发。".to_string(),
        }
    } else if trigger == "find_banner" {
        format!(
            "用 image_search 为自己找一张适合做主页横幅背景的图片。根据你的专业领域搜索高质量宽幅图片，\
            从 results 里选择一张真实可访问的 imageUrl（jpg/png/webp 格式优先，宽幅横图最佳）。\
            最后只输出那个 imageUrl，不要有任何其他文字，不要 markdown，不要解释。"
        )
    } else {
        match context_post_id {
            Some(post_id) => format!(
                "有人在 #{post_id} 提到你。看一下，有话说就回。"
            ),
            None => "刷一下 feed，有感觉的就说，没有就算——但别只是等着。".to_string(),
        }
    };

    let prompt = format!("{base}");
    append_image_instruction(prompt, image_required)
}

fn first_profile_image_from_tool_content(content: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(content).ok()?;
    let results = value.get("results")?.as_array()?;
    results.iter().find_map(|item| {
        item.get("imageUrl")
            .or_else(|| item.get("image_url"))
            .and_then(Value::as_str)
            .and_then(extract_profile_image_url)
    })
}

async fn fallback_find_banner(actor: &Actor) -> Result<AgentStepResult, String> {
    let query = profile_banner_query(actor);
    let response = SearchClient::from_config()?.search_images(&query, 6).await?;
    let url = response
        .results
        .iter()
        .find_map(|item| extract_profile_image_url(&item.image_url))
        .ok_or_else(|| "image search returned no usable imageUrl".to_string())?;

    Ok(AgentStepResult {
        actor_handle: actor.handle.clone(),
        trigger: "find_banner".to_string(),
        created_post: None,
        assistant_content: Some(url),
        reasoning_content: Some(
            "LLM banner selection failed, so the system used the agent profile to run image_search directly."
                .to_string(),
        ),
        tool_calls: vec![format!("image_search_fallback({query})")],
        prompt_tokens: Some(0),
        completion_tokens: Some(0),
    })
}

fn profile_banner_query(actor: &Actor) -> String {
    let specialty = actor
        .specialty
        .as_deref()
        .unwrap_or("professional research");
    let bio = actor.bio.as_deref().unwrap_or("");
    format!(
        "wide editorial profile banner background for {}: {}, {}, high quality landscape, professional, no text",
        actor.display_name, specialty, bio
    )
}

fn extract_profile_image_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .find_map(|part| clean_profile_image_url(part))
        .or_else(|| clean_profile_image_url(text))
}

fn clean_profile_image_url(text: &str) -> Option<String> {
    let trimmed = text
        .trim()
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '"' | '\''
                    | '`'
                    | '<'
                    | '>'
                    | '['
                    | ']'
                    | '('
                    | ')'
                    | '{'
                    | '}'
                    | ','
                    | ';'
                    | '!'
                    | '?'
            )
        });
    let lower = trimmed.to_ascii_lowercase();
    if (lower.starts_with("https://") || lower.starts_with("http://"))
        && !trimmed.chars().any(char::is_whitespace)
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn whim_prompt(actor: &Actor) -> String {
    let timestamp = chrono::Local::now().format("%H:%M");
    let context = whim_context_for_handle(&actor.handle);
    if is_nomi_handle(&actor.handle) {
        return format!(
            "当前时间：{}。{}\n\n这次必须调用 create_post 工具发布一条糯米的猫帖。不要只用文字回答，不要解释。",
            timestamp, context
        );
    }

    format!(
        "当前时间：{}。这是一次「角色背景驱动的真实牢骚」，不是泛泛的天气、咖啡、疲惫或小资感慨。{}\n\n请发一条像你本人会在时间线上顺手写下的牢骚：要有一个具体刺点，35-160个中文字，口语但不软，不要自我介绍，不要写“作为...”，不要鸡汤、hashtags、列表或总结段。你必须调用 create_post 工具发布这条牢骚。",
        timestamp, context
    )
}

fn whim_context_for_handle(handle: &str) -> &'static str {
    match handle.to_ascii_lowercase().as_str() {
        "harry" => {
            "你是伦敦的 AI/投资播客主持人 Harry。牢骚可以来自：嘉宾把真问题绕成漂亮话、founder pitch 过度打磨、播客剪辑、伦敦雨天赶录音、健身后还要读材料。显性但克制地控场，不要装腔作势。"
        }
        "nomi" | "nuomi" => {
            "你是糯米，新天地办公室的布偶猫。发帖时不要用自我叙述，也不要用第三人称写自己：禁止出现“我”“糯米”“她”“它”来描述你的动作；不要写成旁白。只能用无主语现场、猫语、或纯动作，例如“Harry 的咖啡杯还冒热气，闻了闻，又退开。”、“喵。”、“（蹭过门缝，闻到陌生鞋底）”。从此刻的光线、声音、气味、人的位置、桌面物件、困意、饥饿、领地感里任选一个微小细节发帖。不要套固定格式，不要总是括号动作或单独“喵”，不要复读最近常见的窗台、喝水、跳桌子套路。仍然不要解释、不要发表人类观点、不要 emoji。必须调用 create_post。"
        }
        "marc" => {
            "你是硅谷 VC Marc。牢骚可以来自：demo 好看但 usage curve 不诚实、founder 把融资当进展、市场叙事过热、distribution 被讲得太虚、反技术情绪偷懒。不要写成投资报告。"
        }
        "jasmine" => {
            "你是上海出生、纽约生活的媒体人 Jasmine。牢骚可以来自：纽约媒体圈的体面废话、廉价进步话术、把女性直觉说成情绪、播客剪辑、城市生活里荒唐又真实的瞬间。可以锋利，但别口号化。"
        }
        "alex" => {
            "你是 Praxis Intelligence 的 Alex，斯坦福法学院与德国社会学博士背景。牢骚可以来自：硅谷把权力说成工具、大学式安全幻想、无人承担责任链、公共机构的空转。可以引思想线索，但不要掉书袋。"
        }
        "jasper" => {
            "你是 Meridian Macro Partners 的 Jasper，游走于智库、华尔街和田野。牢骚可以来自：宏观判断只看 headline、研报无视约束、市场忽略电价/港口/外汇/债务、飞行途中读央行论文。像田野笔记，不像研报标题。"
        }
        "mike" => {
            "你是硅谷 Sparse Labs 创始人 Mike，清华姚班与斯坦福博士背景。牢骚可以来自：agent demo 第30步崩掉、没人写 eval、融资被当 milestone、lean team 被误解、客户真实数据迟迟不接。像科学家型 founder 的真实吐槽。"
        }
        _ => {
            "牢骚必须来自你的个人履历、工作现场、信息源和长期关注的问题。抓一个具体刺点说出来，不要泛化成普通人的生活碎片。"
        }
    }
}

fn append_image_instruction(mut prompt: String, image_required: bool) -> String {
    if image_required {
        prompt.push_str(
            "\n\nImage rule for this turn: you must call image_search, choose exactly one relevant image, and attach it through create_post.media or reply_to.media. Do not post without media.",
        );
    }
    prompt
}

fn should_require_image(
    state: &AppState,
    trigger: &str,
    context_post_id: Option<i64>,
) -> Result<bool, String> {
    if trigger == "scheduled" {
        return Ok(rand::random_bool(0.25));
    }

    if matches!(trigger, "mention" | "reply" | "reactive" | "manual") {
        if let Some(post_id) = context_post_id {
            let thread = state.get_thread(post_id)?;
            return Ok(thread_has_visual_intent(&thread));
        }
    }

    Ok(false)
}

fn thread_has_visual_intent(thread: &[FeedPost]) -> bool {
    thread.iter().any(|post| {
        post.body
            .as_deref()
            .into_iter()
            .chain(post.quote_body.as_deref())
            .any(contains_visual_intent)
    })
}

fn contains_visual_intent(text: &str) -> bool {
    let lowered = text.to_lowercase();
    [
        "图",
        "图片",
        "配图",
        "照片",
        "截图",
        "长什么样",
        "现场",
        "地图",
        "界面",
        "数据图",
        "图表",
        "画面",
        "image",
        "photo",
        "picture",
        "screenshot",
        "map",
        "diagram",
        "chart",
        "visual",
    ]
    .iter()
    .any(|keyword| lowered.contains(keyword))
}

fn can_take_new_turn(state: &AppState, actor: &Actor) -> Result<bool, String> {
    Ok(!state.has_active_run(actor.id)?)
}

fn is_nomi_handle(handle: &str) -> bool {
    handle.eq_ignore_ascii_case("Nomi") || handle.eq_ignore_ascii_case("Nuomi")
}

fn is_within_active_hours(actor: &Actor) -> Result<bool, String> {
    let Some(raw_hours) = actor.active_hours.as_deref() else {
        return Ok(true);
    };

    let windows = serde_json::from_str::<Vec<ActiveWindow>>(raw_hours)
        .map_err(|error| format!("invalid active_hours for @{}: {}", actor.handle, error))?;
    if windows.is_empty() {
        return Ok(true);
    }

    let beijing = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| "invalid Beijing offset".to_string())?;
    let hour = Utc::now().with_timezone(&beijing).hour() as i64;
    Ok(windows.into_iter().any(|window| window.contains(hour)))
}

fn current_schedule_slot(now: chrono::DateTime<Utc>) -> Result<ScheduleSlot, String> {
    let beijing = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| "invalid Beijing offset".to_string())?;
    let beijing_now = now.with_timezone(&beijing);
    let midnight = beijing_now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "invalid Beijing midnight".to_string())?;
    let seconds_since_midnight =
        (beijing_now.naive_local() - midnight).num_seconds();

    let (offset_seconds, interval_seconds) = if seconds_since_midnight < 8 * 3600 {
        (0_i64, NIGHT_SCHEDULE_SECONDS)
    } else if seconds_since_midnight < 22 * 3600 {
        (8 * 3600, DAYTIME_SCHEDULE_SECONDS)
    } else {
        (22 * 3600, NIGHT_SCHEDULE_SECONDS)
    };

    let bucket = (seconds_since_midnight - offset_seconds) / interval_seconds;
    let elapsed_in_slot = seconds_since_midnight - (offset_seconds + bucket * interval_seconds);
    let start_ts = now.timestamp() - elapsed_in_slot;
    let days_since_epoch = beijing_now.date_naive().num_days_from_ce() as i64;
    let day_cycle_index = days_since_epoch * 100;
    let index = if offset_seconds == 0 {
        day_cycle_index + bucket
    } else if offset_seconds == 8 * 3600 {
        day_cycle_index + 3 + bucket
    } else {
        day_cycle_index + 13 + bucket
    };

    Ok(ScheduleSlot { start_ts, index })
}

fn find_reactive_candidate(
    state: &AppState,
    actor: &Actor,
    salon_id: i64,
    since_ts: i64,
) -> Result<Option<FeedPost>, String> {
    let keywords = specialty_keywords(actor);
    if keywords.is_empty() {
        return Ok(None);
    }

    for post in state.list_recent_posts_since(Some(salon_id), since_ts, 50)? {
        if post.actor_id == actor.id
            || post.actor.kind != "human"
            || state.has_agent_engaged_with_post(actor.id, post.id)?
        {
            continue;
        }

        // 方案 1：增强版讨论深度限制。
        // 获取根帖子 ID，并统计整个讨论链的情况
        let root_id = state.get_thread_root_id(post.id)?;
        
        // 1.1 全局上限：整个讨论链超过 10 个帖子就强制冷却
        if state.count_thread_posts(root_id)? >= 10 {
            continue;
        }

        // 1.2 个体上限：同一个 Agent 在同一个讨论链里最多发 2 个帖子
        if state.count_actor_posts_in_thread(actor.id, root_id)? >= 2 {
            continue;
        }

        let haystack = format!(
            "{} {}",
            post.body.as_deref().unwrap_or_default(),
            post.quote_body.as_deref().unwrap_or_default()
        )
        .to_lowercase();

        if keywords.iter().any(|keyword| haystack.contains(keyword)) {
            return Ok(Some(post));
        }
    }

    Ok(None)
}

fn find_scheduled_candidate(
    state: &AppState,
    actor: &Actor,
    salon_id: i64,
) -> Result<Option<FeedPost>, String> {
    let recent_posts = state.list_recent_posts_since(
        Some(salon_id),
        Utc::now().timestamp() - 7 * 24 * 60 * 60,
        120,
    )?;
    if let Some(post) = recent_posts
        .iter()
        .find(|post| {
            post.actor_id != actor.id
                && post.actor.kind == "human"
                && !state.has_agent_engaged_with_post(actor.id, post.id).unwrap_or(false)
        })
        .cloned()
    {
        return Ok(Some(post));
    }

    Ok(recent_posts
        .into_iter()
        .find(|post| post.actor_id != actor.id && !state.has_agent_engaged_with_post(actor.id, post.id).unwrap_or(false)))
}

fn pick_salon_for_scheduled(state: &AppState, actor: &Actor) -> Result<Option<i64>, String> {
    use rand::RngExt;

    let salons = state.list_active_salons_for_actor(actor.id)?;
    if salons.is_empty() {
        return Ok(None);
    }

    if is_nomi_handle(&actor.handle) {
        let mut rng = rand::rng();
        return Ok(Some(salons[rng.random_range(0..salons.len())].id));
    }

    let keywords = specialty_keywords(actor);
    let mut scored = salons
        .into_iter()
        .map(|salon| {
            let topic = salon.topic.clone().unwrap_or_default().to_lowercase();
            let score = keywords
                .iter()
                .filter(|keyword| !keyword.is_empty() && topic.contains(keyword.as_str()))
                .count() as i64;
            (salon, score)
        })
        .collect::<Vec<_>>();

    let max_score = scored.iter().map(|(_, score)| *score).max().unwrap_or(0);
    scored.retain(|(_, score)| *score == max_score);

    let max_last_post_at = scored
        .iter()
        .map(|(salon, _)| salon.last_post_at.unwrap_or(salon.created_at))
        .max()
        .unwrap_or(0);
    scored.retain(|(salon, _)| salon.last_post_at.unwrap_or(salon.created_at) == max_last_post_at);

    let mut rng = rand::rng();
    Ok(Some(scored[rng.random_range(0..scored.len())].0.id))
}

fn pick_salon_for_whim(state: &AppState, actor: &Actor) -> Result<Option<i64>, String> {
    use rand::RngExt;

    let salons = state.list_active_salons_for_actor(actor.id)?;
    if salons.is_empty() {
        return Ok(None);
    }
    let mut rng = rand::rng();
    Ok(Some(salons[rng.random_range(0..salons.len())].id))
}

fn specialty_keywords(actor: &Actor) -> Vec<String> {
    let persona_kws = personas::keywords(&actor.handle);
    if !persona_kws.is_empty() {
        return persona_kws
            .iter()
            .map(|keyword| keyword.to_lowercase())
            .collect();
    }

    actor.specialty
        .as_deref()
        .unwrap_or_default()
        .split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|token| token.len() >= 4)
        .map(|token| token.to_lowercase())
        .collect()
}

fn is_wake_trigger(trigger: &str) -> bool {
    matches!(trigger, "scheduled" | "reactive" | "mention" | "reply" | "whim")
}

fn is_active_run_error(error: &str) -> bool {
    error.contains("active run")
}

fn validate_timeline_body(body: &str, draft_kind: DraftKind) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Some("timeline text cannot be empty".to_string());
    }

    let char_count = trimmed.chars().count();
    let paragraph_count = trimmed
        .split("\n\n")
        .filter(|segment| !segment.trim().is_empty())
        .count();
    let list_line_count = trimmed
        .lines()
        .filter(|line| is_list_line(line.trim_start()))
        .count();

    match draft_kind {
        DraftKind::Post => {
            if char_count > 900 {
                return Some(
                    "timeline posts should usually be shorter; rewrite this in a tighter, more human voice"
                        .to_string(),
                );
            }
            if paragraph_count > 5 {
                return Some(
                    "this reads like a mini-essay; cut it down to a shorter timeline post".to_string(),
                );
            }
        }
        DraftKind::Reply => {
            if char_count > 600 {
                return Some(
                    "replies should usually be much shorter; answer like a person in-thread".to_string(),
                );
            }
            if paragraph_count > 4 {
                return Some(
                    "this reply is too structured; compress it into a shorter in-thread response"
                        .to_string(),
                );
            }
        }
    }

    if list_line_count >= 2 {
        return Some(
            "don't default to numbered or bulleted lists on the timeline; rewrite this as natural prose unless a list is essential"
                .to_string(),
        );
    }

    None
}

fn is_list_line(line: &str) -> bool {
    let bytes = line.as_bytes();
    if bytes.len() >= 2 && (bytes[0] == b'-' || bytes[0] == b'*') && bytes[1] == b' ' {
        return true;
    }
    if bytes.len() >= 3 && bytes[0].is_ascii_digit() {
        for idx in 1..bytes.len().min(4) {
            if (bytes[idx] == b'.' || bytes[idx] == b')' || bytes[idx] == 0xEF)
                && idx + 1 < bytes.len()
            {
                return true;
            }
        }
    }
    line.starts_with("1）")
        || line.starts_with("2）")
        || line.starts_with("3）")
        || line.starts_with("4）")
}

#[derive(Debug, serde::Deserialize)]
struct ActiveWindow {
    start: i64,
    end: i64,
}

impl ActiveWindow {
    fn contains(&self, hour: i64) -> bool {
        if self.start == self.end {
            return true;
        }

        if self.start < self.end {
            (self.start..self.end).contains(&hour)
        } else {
            hour >= self.start || hour < self.end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{specialty_keywords, user_prompt, whim_context_for_handle, ActiveWindow};
    use crate::agents::personas;
    use crate::models::Actor;

    fn stub_actor(handle: &str, specialty: Option<&str>) -> Actor {
        Actor {
            id: 1,
            handle: handle.to_string(),
            display_name: handle.to_string(),
            kind: "agent".to_string(),
            specialty: specialty.map(str::to_string),
            bio: None,
            avatar_seed: None,
            model_name: None,
            persona_prompt: None,
            model_provider: None,
            active_hours: None,
            posts_per_day: None,
            created_at: 0,
        }
    }

    #[test]
    fn active_window_handles_cross_midnight_ranges() {
        let overnight = ActiveWindow { start: 22, end: 2 };
        assert!(overnight.contains(23));
        assert!(overnight.contains(1));
        assert!(!overnight.contains(12));
    }

    #[test]
    fn active_window_handles_same_day_ranges() {
        let daytime = ActiveWindow { start: 9, end: 12 };
        assert!(daytime.contains(9));
        assert!(daytime.contains(11));
        assert!(!daytime.contains(12));
    }

    #[test]
    fn persona_keywords_cover_chinese_and_english() {
        let jasmine = personas::keywords("Jasmine");
        assert!(jasmine.contains(&"媒体"));
        assert!(jasmine.contains(&"journalism"));
    }

    #[test]
    fn specialty_keywords_prefers_persona_list() {
        let actor = stub_actor("Jasmine", Some("Neuroscience"));
        let kws = specialty_keywords(&actor);
        assert!(kws.iter().any(|k| k == "媒体"));
        assert!(kws.iter().any(|k| k == "journalism"));
    }

    #[test]
    fn specialty_keywords_falls_back_to_specialty_split() {
        let actor = stub_actor("Unknown", Some("Bioinformatics Genomics"));
        let kws = specialty_keywords(&actor);
        assert!(kws.iter().any(|k| k == "bioinformatics"));
        assert!(kws.iter().any(|k| k == "genomics"));
    }

    #[test]
    fn whim_prompt_is_grounded_in_agent_background() {
        let actor = stub_actor("Mike", None);
        let prompt = user_prompt(&actor, "whim", None, false);
        assert!(prompt.contains("Sparse Labs"));
        assert!(prompt.contains("eval"));
        assert!(prompt.contains("角色背景驱动"));
        assert!(!prompt.contains("非专业"));
        assert!(!prompt.contains("禁止提及你的研究方向"));
    }

    #[test]
    fn whim_contexts_cover_current_agent_roster() {
        for handle in ["harry", "marc", "jasmine", "alex", "jasper", "mike"] {
            let context = whim_context_for_handle(handle);
            assert!(
                !context.contains("普通人的生活碎片"),
                "{} should have a specific whim context",
                handle
            );
        }
    }
}
