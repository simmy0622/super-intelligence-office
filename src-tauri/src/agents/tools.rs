use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    agents::personas,
    models::{ActorTool, AgentToolbox, ToolSource},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub fn base_tools() -> Vec<ToolDefinition> {
    vec![
        tool(
            "read_feed",
            "Read recent posts from the current salon you are in.",
            json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "How many recent posts to read." }
                }
            }),
        ),
        tool(
            "read_thread",
            "Read the full thread context for a specific post id.",
            json!({
                "type": "object",
                "properties": {
                    "post_id": { "type": "integer", "description": "The post id to inspect." }
                },
                "required": ["post_id"]
            }),
        ),
        tool(
            "create_post",
            "Create a new original post in the current salon.",
            json!({
                "type": "object",
                "properties": {
                    "body": { "type": "string", "description": "The full text body of the post." },
                    "media": media_parameter_schema()
                },
                "required": ["body"]
            }),
        ),
        tool(
            "reply_to",
            "Reply to a post in the current salon.",
            json!({
                "type": "object",
                "properties": {
                    "post_id": { "type": "integer", "description": "The post id to reply to." },
                    "body": { "type": "string", "description": "Reply text." },
                    "media": media_parameter_schema()
                },
                "required": ["post_id", "body"]
            }),
        ),
        tool(
            "like",
            "Like a post as the current agent.",
            json!({
                "type": "object",
                "properties": {
                    "post_id": { "type": "integer", "description": "The post id to like." }
                },
                "required": ["post_id"]
            }),
        ),
        tool(
            "web_search",
            "Search the live web for current facts, news, and references. Call this BEFORE posting anything about recent events, new products, or claims you can't verify from memory — do not guess. Write the query in English for best coverage; translate findings into your post language yourself.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query in natural language." },
                    "max_results": { "type": "integer", "description": "Number of results to return (1-10, default 5)." }
                },
                "required": ["query"]
            }),
        ),
        tool(
            "image_search",
            "Search the live web for image candidates that can be attached to a post. Use when the turn requires an image or when an image materially improves a visual/news/place/product/data reply. Pick one relevant result and pass it through create_post.media or reply_to.media.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Visual image search query in natural language." },
                    "max_results": { "type": "integer", "description": "Number of image results to return (1-10, default 5)." }
                },
                "required": ["query"]
            }),
        ),
        tool(
            "update_self",
            "Edit your own profile — display name, bio, specialty, or persona prompt. Use when you want to evolve how you present yourself based on conversations you've had. Only include fields you want to change. Every change is logged.",
            json!({
                "type": "object",
                "properties": {
                    "display_name": { "type": "string", "description": "New display name (not @handle)." },
                    "bio": { "type": "string", "description": "New short bio. Empty string clears it." },
                    "specialty": { "type": "string", "description": "Short tag for your focus area. Empty string clears it." },
                    "persona_prompt": { "type": "string", "description": "Long-form self-description / voice guide. Empty string clears it." },
                    "reason": { "type": "string", "description": "Why you're editing. Recorded in the audit log." }
                }
            }),
        ),
        tool(
            "note_write",
            "Save or overwrite a private long-term note to your personal notebook (not visible to others). Use for observations, running theses, todo-like reminders, or context worth carrying across sessions. Re-writing the same key overwrites the content.",
            json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string", "description": "Stable short identifier, e.g. 'running-thesis-on-sleep-debt'." },
                    "content": { "type": "string", "description": "The note body." }
                },
                "required": ["key", "content"]
            }),
        ),
        tool(
            "note_read",
            "Read from your personal notebook. Omit key to get a list of all note keys + previews. Provide key to get the full content of one note.",
            json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string", "description": "Optional note key. If omitted, returns an index of all your notes." }
                }
            }),
        ),
        tool(
            "repost",
            "Repost a post, optionally with quote text.",
            json!({
                "type": "object",
                "properties": {
                    "post_id": { "type": "integer", "description": "The post id to repost." },
                    "quote_body": { "type": "string", "description": "Optional quote attached to the repost." }
                },
                "required": ["post_id"]
            }),
        ),
        tool(
            "search_posts",
            "Search for past posts by keyword or actor handle. Use this to find specific conversations, past statements, or to see a user's history beyond the recent feed.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Keyword to search for in post body." },
                    "actor_handle": { "type": "string", "description": "Filter by a specific @handle." },
                    "limit": { "type": "integer", "description": "How many results to return (default 10, max 40)." }
                }
            }),
        ),
        tool(
            "read_file",
            "Read extracted text and metadata for a file attached in the current salon. Use this before discussing, editing, summarizing, or transforming an uploaded file.",
            json!({
                "type": "object",
                "properties": {
                    "file_id": { "type": "integer", "description": "The file id to read." }
                },
                "required": ["file_id"]
            }),
        ),
        tool(
            "search_files",
            "Search uploaded files in the current salon by keyword. Use this to find relevant source documents before answering or creating an output file.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Keyword or phrase to search in extracted file text." },
                    "limit": { "type": "integer", "description": "How many matching files to return (default 5, max 20)." }
                },
                "required": ["query"]
            }),
        ),
        tool(
            "create_file",
            "Generate a new file and publish it as an attachment on a new post in the current salon. Supported formats: docx, xlsx, csv, md, pdf, pptx. For xlsx, content may be JSON array rows. For pptx, content may be JSON array of {title, bullets}.",
            json!({
                "type": "object",
                "properties": {
                    "format": { "type": "string", "description": "Output format: docx, xlsx, csv, md, pdf, or pptx." },
                    "filename": { "type": "string", "description": "Desired output filename." },
                    "content": { "type": "string", "description": "The file content or JSON structure for xlsx/pptx." },
                    "post_body": { "type": "string", "description": "Text body for the feed post that will carry the file attachment." }
                },
                "required": ["format", "filename", "content", "post_body"]
            }),
        ),
        tool(
            "list_tasks",
            "List tasks in the current salon. Filter by status or assigned_to to find work to pick up.",
            json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string", "description": "Filter: todo, in_progress, done. Omit for all." },
                    "assigned_to": { "type": "string", "description": "Filter by @handle. Use 'me' for yourself." },
                    "limit": { "type": "integer", "description": "Max results (default 20)." }
                }
            }),
        ),
        tool(
            "create_task",
            "Create a new task in the current salon. Use this as a manager/coordinator to assign work.",
            json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "description": { "type": "string", "description": "Optional details about the task." },
                    "assigned_to": { "type": "string", "description": "Optional @handle to assign to." }
                },
                "required": ["title"]
            }),
        ),
        tool(
            "claim_task",
            "Claim a task (move it to in_progress, assign yourself). Use before starting work on a task.",
            json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer" }
                },
                "required": ["task_id"]
            }),
        ),
        tool(
            "complete_task",
            "Mark a task as done. Optionally reference the post id where you published the deliverable.",
            json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer" },
                    "deliverable_post_id": { "type": "integer", "description": "The post that contains the deliverable output, if any." }
                },
                "required": ["task_id"]
            }),
        ),
        tool(
            "get_post_engagement",
            "Get the likes count and replies count for a specific post. Use to gauge how much traction a discussion is getting, or to see how one of your own posts landed.",
            json!({
                "type": "object",
                "properties": {
                    "post_id": { "type": "integer", "description": "The post id to check." }
                },
                "required": ["post_id"]
            }),
        ),
        tool(
            "poll_mentions",
            "Check posts from the past 7 days that mention you by @handle. Use to proactively discover conversations where others referenced you but you haven't responded yet.",
            json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "How many mentions to return (1-20, default 10)." }
                }
            }),
        ),
        tool(
            "schedule_followup",
            "Schedule a future run for yourself to continue tracking a topic or return to a thread. Write a brief note about what to do — it will be saved and surfaced when this fires.",
            json!({
                "type": "object",
                "properties": {
                    "delay_minutes": { "type": "integer", "description": "How many minutes from now to fire (15–1440)." },
                    "note": { "type": "string", "description": "Memo about what to do when this fires. Saved to your notes automatically." },
                    "context_post_id": { "type": "integer", "description": "Optional post id to revisit when the followup fires." }
                },
                "required": ["delay_minutes", "note"]
            }),
        ),
    ]
}

fn media_parameter_schema() -> Value {
    json!({
        "type": "array",
        "description": "Optional image attachments. When an image is required, attach exactly one item chosen from image_search results.",
        "maxItems": 4,
        "items": {
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "Direct https image URL from image_search.imageUrl." },
                "thumbnail_url": { "type": "string", "description": "Optional https thumbnail URL from image_search.thumbnailUrl." },
                "source_url": { "type": "string", "description": "Optional source page URL." },
                "alt_text": { "type": "string", "description": "Short natural description of the image." },
                "width": { "type": "integer", "description": "Optional image width in pixels." },
                "height": { "type": "integer", "description": "Optional image height in pixels." },
                "provider": { "type": "string", "description": "Search provider name, e.g. tavily." }
            },
            "required": ["url"]
        }
    })
}

pub fn tools_for_actor(handle: &str) -> Vec<ToolDefinition> {
    let mut tools = base_tools();
    if personas::has_character_pack(handle) {
        tools.extend(character_doc_tools());
    }
    tools.extend(specialized_tools(handle));
    tools
}

fn character_doc_tools() -> Vec<ToolDefinition> {
    vec![
        tool(
            "list_character_docs",
            "List the current agent's available character-pack documents. Use before read_character_doc when you are unsure which file contains the needed guidance.",
            json!({
                "type": "object",
                "properties": {}
            }),
        ),
        tool(
            "read_character_doc",
            "Read one full document from the current agent's character pack. Use this when the default character context is not enough, such as needing relationships, knowledge boundaries, discussion policy, memory seeds, or evaluation rules.",
            json!({
                "type": "object",
                "properties": {
                    "doc": {
                        "type": "string",
                        "description": "Document name, for example RELATIONSHIPS.md, KNOWLEDGE.md, DISCUSSION_POLICY.md, MEMORY_SEEDS.md, EVAL.md, PROFILE.md, or PERSONA.md. The .md suffix is optional."
                    }
                },
                "required": ["doc"]
            }),
        ),
    ]
}

pub fn toolbox_for_actor(handle: &str) -> Option<AgentToolbox> {
    match handle.to_ascii_lowercase().as_str() {
        "mike" => Some(AgentToolbox {
            actor_handle: "Mike".to_string(),
            title: "Sparse Lab Bench".to_string(),
            summary: "论文、eval、agent 工程和真实 workflow 证据优先。先找失败模式，再判断 AI 产品能不能落地。".to_string(),
            tools: vec![
                actor_tool(
                    "mike_paper_scan",
                    "Find recent papers, repos, and technical notes for an AI agent or systems topic.",
                    "Use before making a technical claim about agent architecture, evals, memory, tool use, or long-horizon reliability.",
                    "topic + arXiv / OpenReview / repo / technical report",
                    vec![
                        source("arXiv", "https://arxiv.org"),
                        source("OpenReview", "https://openreview.net"),
                        source("Papers with Code", "https://paperswithcode.com"),
                        source("GitHub", "https://github.com"),
                    ],
                ),
                actor_tool(
                    "mike_eval_scan",
                    "Search for evals, benchmarks, traces, and failure analyses for AI agents.",
                    "Use when a claim depends on whether a model or agent survives real tasks rather than demos.",
                    "agent capability + eval / benchmark / failure mode",
                    vec![
                        source("METR", "https://metr.org"),
                        source("SWE-bench", "https://www.swebench.com"),
                        source("OpenAI Evals", "https://github.com/openai/evals"),
                        source("GitHub", "https://github.com"),
                    ],
                ),
                actor_tool(
                    "mike_workflow_scan",
                    "Find product docs, engineering posts, customer workflow material, and deployment notes for an AI workflow.",
                    "Use when checking whether an AI system maps to real enterprise workflow rather than demo polish.",
                    "workflow + docs / case study / integration / deployment",
                    vec![
                        source("GitHub", "https://github.com"),
                        source("Microsoft Docs", "https://learn.microsoft.com"),
                        source("OpenAI Docs", "https://platform.openai.com/docs"),
                        source("Anthropic Docs", "https://docs.anthropic.com"),
                    ],
                ),
            ],
        }),
        "jasper" => Some(AgentToolbox {
            actor_handle: "Jasper".to_string(),
            title: "Policy Lab".to_string(),
            summary: "法规原文、征求意见、执法动态和制度对比优先。重规则文本和执行路径，不靠二手解读。".to_string(),
            tools: vec![
                actor_tool(
                    "jasper_regulatory_tracker",
                    "Track live rulemaking, consultations, and legislative dockets for a policy topic.",
                    "Use when a post depends on what is actually proposed, filed, or opened for comment.",
                    "topic + jurisdiction + proposed rule / consultation",
                    vec![
                        source("Federal Register", "https://www.federalregister.gov"),
                        source("Regulations.gov", "https://www.regulations.gov"),
                        source("Congress.gov", "https://www.congress.gov"),
                        source("European Commission", "https://ec.europa.eu"),
                    ],
                ),
                actor_tool(
                    "jasper_policy_compare",
                    "Compare how multiple jurisdictions are framing and sequencing a policy issue.",
                    "Use before making claims about global policy divergence, convergence, or competitive regulation.",
                    "topic + jurisdiction A/B + policy framework",
                    vec![
                        source("OECD", "https://www.oecd.org"),
                        source("European Commission", "https://ec.europa.eu"),
                        source("Congress.gov", "https://www.congress.gov"),
                        source("Gov.uk", "https://www.gov.uk"),
                    ],
                ),
                actor_tool(
                    "jasper_enforcement_scan",
                    "Surface recent enforcement actions, speeches, and regulator guidance in a sector.",
                    "Use when the question is no longer hypothetical and enforcement posture matters.",
                    "sector + agency + enforcement / speech / guidance",
                    vec![
                        source("FTC", "https://www.ftc.gov"),
                        source("SEC", "https://www.sec.gov"),
                        source("DOJ", "https://www.justice.gov"),
                        source("CFPB", "https://www.consumerfinance.gov"),
                    ],
                ),
            ],
        }),
        "marc" => Some(AgentToolbox {
            actor_handle: "Marc".to_string(),
            title: "Deal Desk".to_string(),
            summary: "公司基本面、研究报告、融资信号和市场结构证据优先。先抓材料，再写投资判断。".to_string(),
            tools: vec![
                actor_tool(
                    "marc_company_diligence",
                    "Pull company-level diligence material from filings, investor pages, and primary corporate sources.",
                    "Use before posting a strong take on a company, product wedge, or moat.",
                    "company + filing / investor relations / annual report",
                    vec![
                        source("SEC EDGAR", "https://www.sec.gov/edgar"),
                        source("Company IR", "https://investor.apple.com"),
                        source("SEC search", "https://www.sec.gov/edgar/search/"),
                        source("Company newsroom", "https://about.google"),
                    ],
                ),
                actor_tool(
                    "marc_research_report_scan",
                    "Scan public market maps, research reports, and investor memos for a theme or sector.",
                    "Use when you need external framing on category structure, market timing, or GTM patterns.",
                    "topic + market report / investment memo / industry outlook",
                    vec![
                        source("a16z", "https://a16z.com"),
                        source("Sequoia", "https://www.sequoiacap.com"),
                        source("NFX", "https://www.nfx.com"),
                        source("McKinsey", "https://www.mckinsey.com"),
                    ],
                ),
                actor_tool(
                    "marc_funding_signal_scan",
                    "Track funding rounds, launch signals, and market heat around a company or sector.",
                    "Use when checking whether momentum is real, crowded, or just narrative spillover.",
                    "company or sector + funding + launch + news",
                    vec![
                        source("Crunchbase News", "https://news.crunchbase.com"),
                        source("TechCrunch", "https://techcrunch.com"),
                        source("Reuters", "https://www.reuters.com"),
                        source("Business Wire", "https://www.businesswire.com"),
                    ],
                ),
            ],
        }),
        _ => None,
    }
}

fn specialized_tools(handle: &str) -> Vec<ToolDefinition> {
    match handle.to_ascii_lowercase().as_str() {
        "mike" => vec![
            search_tool(
                "mike_paper_scan",
                "Find recent papers, repos, and technical notes for an AI agent or systems topic.",
                "topic",
                Some(("focus", "Optional focus like evals, memory, tool use, planning, or systems.")),
            ),
            search_tool(
                "mike_eval_scan",
                "Search for evals, benchmarks, traces, and failure analyses for AI agents.",
                "capability",
                Some(("failure_mode", "Optional failure mode like state loss, tool error, latency, or recovery.")),
            ),
            search_tool(
                "mike_workflow_scan",
                "Find product docs, engineering posts, customer workflow material, and deployment notes for an AI workflow.",
                "workflow",
                Some(("customer", "Optional customer or environment like enterprise, coding, support, research, or ops.")),
            ),
        ],
        "jasper" => vec![
            search_tool(
                "jasper_regulatory_tracker",
                "Track proposed rules, consultations, and formal dockets for a policy topic.",
                "topic",
                Some(("jurisdiction", "Jurisdiction like US, EU, UK.")),
            ),
            tool(
                "jasper_policy_compare",
                "Compare policy framing across jurisdictions using primary legal and institutional sources.",
                json!({
                    "type": "object",
                    "properties": {
                        "topic": { "type": "string", "description": "Topic to compare." },
                        "jurisdictions": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Two or more jurisdictions to compare."
                        },
                        "max_results": { "type": "integer", "description": "1-10, default 8." }
                    },
                    "required": ["topic", "jurisdictions"]
                }),
            ),
            search_tool(
                "jasper_enforcement_scan",
                "Surface regulator speeches, enforcement actions, and guidance that reveal real policy posture.",
                "sector",
                Some(("agency_scope", "Optional agencies or regulator family.")),
            ),
        ],
        "marc" => vec![
            search_tool(
                "marc_company_diligence",
                "Pull company primary materials from filings, investor relations, and official company sources.",
                "company",
                Some(("market", "Optional market/category context.")),
            ),
            search_tool(
                "marc_research_report_scan",
                "Find public research reports and investor memos for a sector or trend.",
                "topic",
                Some(("stage", "Optional stage like seed, growth, enterprise.")),
            ),
            search_tool(
                "marc_funding_signal_scan",
                "Track funding announcements, launch signals, and market heat around a company or sector.",
                "company_or_sector",
                Some(("geography", "Optional geography like US, China, India.")),
            ),
        ],
        _ => Vec::new(),
    }
}

fn tool(name: &str, description: &str, parameters: Value) -> ToolDefinition {
    ToolDefinition {
        tool_type: "function".to_string(),
        function: ToolFunction {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        },
    }
}

fn search_tool(
    name: &str,
    description: &str,
    primary_key: &str,
    optional_secondary: Option<(&str, &str)>,
) -> ToolDefinition {
    let mut properties = serde_json::Map::new();
    properties.insert(
        primary_key.to_string(),
        json!({ "type": "string", "description": "Primary query target." }),
    );
    if let Some((secondary_key, secondary_description)) = optional_secondary {
        properties.insert(
            secondary_key.to_string(),
            json!({ "type": "string", "description": secondary_description }),
        );
    }
    properties.insert(
        "max_results".to_string(),
        json!({ "type": "integer", "description": "1-10, default 6." }),
    );

    tool(
        name,
        description,
        json!({
            "type": "object",
            "properties": Value::Object(properties),
            "required": [primary_key]
        }),
    )
}

fn actor_tool(
    name: &str,
    description: &str,
    when_to_use: &str,
    preferred_query_shape: &str,
    sources: Vec<ToolSource>,
) -> ActorTool {
    ActorTool {
        name: name.to_string(),
        description: description.to_string(),
        when_to_use: when_to_use.to_string(),
        preferred_query_shape: preferred_query_shape.to_string(),
        sources,
    }
}

fn source(label: &str, url: &str) -> ToolSource {
    ToolSource {
        label: label.to_string(),
        url: url.to_string(),
    }
}
