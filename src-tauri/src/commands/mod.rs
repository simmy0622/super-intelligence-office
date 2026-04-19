use tauri::State;

use crate::{
    agents::tools,
    db::{AppState, GENERAL_SALON_ID},
    models::{AgentStepResult, AgentToolbox, SettingEntry},
    scheduler,
};

#[tauri::command]
pub fn list_posts(
    state: State<'_, AppState>,
    salon_id: Option<i64>,
    before_timestamp: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<crate::models::FeedPost>, String> {
    state.list_posts(salon_id, before_timestamp, limit.unwrap_or(20))
}

#[tauri::command]
pub fn get_thread(state: State<'_, AppState>, post_id: i64) -> Result<Vec<crate::models::FeedPost>, String> {
    state.get_thread(post_id)
}

#[tauri::command]
pub fn create_human_post(
    state: State<'_, AppState>,
    body: String,
    salon_id: Option<i64>,
) -> Result<crate::models::FeedPost, String> {
    state.create_human_post(&body, salon_id.unwrap_or(GENERAL_SALON_ID))
}

#[tauri::command]
pub fn reply_as_human(
    state: State<'_, AppState>,
    parent_id: i64,
    body: String,
) -> Result<crate::models::FeedPost, String> {
    state.reply_as_human(parent_id, &body)
}

#[tauri::command]
pub fn like_toggle(state: State<'_, AppState>, post_id: i64) -> Result<bool, String> {
    state.toggle_like(post_id)
}

#[tauri::command]
pub fn repost_as_human(
    state: State<'_, AppState>,
    post_id: i64,
    quote_body: Option<String>,
) -> Result<crate::models::FeedPost, String> {
    state.repost_as_human(post_id, quote_body.as_deref())
}

#[tauri::command]
pub fn list_actors(state: State<'_, AppState>) -> Result<Vec<crate::models::Actor>, String> {
    state.list_actors()
}

#[tauri::command]
pub fn get_actor(state: State<'_, AppState>, handle: String) -> Result<crate::models::Actor, String> {
    state.get_actor(&handle)
}

#[tauri::command]
pub fn get_actor_toolbox(_state: State<'_, AppState>, handle: String) -> Result<AgentToolbox, String> {
    tools::toolbox_for_actor(&handle).ok_or_else(|| format!("No specialized toolbox for @{}", handle))
}

#[tauri::command]
pub fn list_agent_runs(state: State<'_, AppState>, limit: Option<i64>) -> Result<Vec<crate::models::AgentRun>, String> {
    state.list_agent_runs(limit.unwrap_or(5))
}

#[tauri::command]
pub fn set_api_key(state: State<'_, AppState>, provider: String, key: String) -> Result<(), String> {
    state.write_api_key(&provider, &key)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Vec<SettingEntry>, String> {
    state.get_settings()
}

#[tauri::command]
pub fn set_settings(state: State<'_, AppState>, settings: Vec<SettingEntry>) -> Result<(), String> {
    state.set_settings(&settings)
}

#[tauri::command]
pub async fn run_agent_step(
    state: State<'_, AppState>,
    handle: String,
    trigger: Option<String>,
    context_post_id: Option<i64>,
    salon_id: Option<i64>,
) -> Result<AgentStepResult, String> {
    let app_state = state.inner().clone();
    scheduler::run_agent_step(
        &app_state,
        &handle,
        trigger.as_deref().unwrap_or("manual"),
        context_post_id,
        salon_id,
    )
    .await
}
