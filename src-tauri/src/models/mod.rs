use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorSummary {
    pub id: i64,
    pub kind: String,
    pub handle: String,
    pub display_name: String,
    pub avatar_seed: Option<String>,
    pub specialty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub id: i64,
    pub kind: String,
    pub handle: String,
    pub display_name: String,
    pub avatar_seed: Option<String>,
    pub bio: Option<String>,
    pub specialty: Option<String>,
    pub persona_prompt: Option<String>,
    pub model_provider: Option<String>,
    pub model_name: Option<String>,
    pub active_hours: Option<String>,
    pub posts_per_day: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSource {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorTool {
    pub name: String,
    pub description: String,
    pub when_to_use: String,
    pub preferred_query_shape: String,
    pub sources: Vec<ToolSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolbox {
    pub actor_handle: String,
    pub title: String,
    pub summary: String,
    pub tools: Vec<ActorTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostReference {
    pub id: i64,
    pub actor: ActorSummary,
    pub kind: String,
    pub parent_id: Option<i64>,
    pub salon_id: i64,
    pub quote_body: Option<String>,
    pub body: Option<String>,
    pub media: Vec<PostMedia>,
    pub files: Vec<FileInfo>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMedia {
    pub id: i64,
    pub post_id: i64,
    pub kind: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub source_url: Option<String>,
    pub alt_text: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub provider: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMediaInput {
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub source_url: Option<String>,
    pub alt_text: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub id: i64,
    pub salon_id: i64,
    pub uploader_id: i64,
    pub original_name: String,
    pub kind: String,
    pub size_bytes: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchResult {
    pub file: FileInfo,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedPost {
    pub id: i64,
    pub actor_id: i64,
    pub actor: ActorSummary,
    pub kind: String,
    pub parent_id: Option<i64>,
    pub salon_id: i64,
    pub quote_body: Option<String>,
    pub body: Option<String>,
    pub trigger: String,
    pub created_at: i64,
    pub pinned_at: Option<i64>,
    pub media: Vec<PostMedia>,
    pub files: Vec<FileInfo>,
    pub referenced_post: Option<PostReference>,
    pub like_count: i64,
    pub reply_count: i64,
    pub repost_count: i64,
    pub liked_by_you: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Salon {
    pub id: i64,
    pub name: String,
    pub topic: Option<String>,
    pub created_by: i64,
    pub created_at: i64,
    pub last_post_at: Option<i64>,
    pub member_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalonMember {
    pub salon_id: i64,
    pub actor: ActorSummary,
    pub joined_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub salon_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_by: i64,
    pub created_by_handle: String,
    pub assigned_to: Option<i64>,
    pub assigned_to_handle: Option<String>,
    pub deliverable_post_id: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLog {
    pub id: i64,
    pub task_id: i64,
    pub actor_id: i64,
    pub actor_handle: String,
    pub action: String,
    pub note: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    pub id: i64,
    pub actor_id: i64,
    pub actor_handle: String,
    pub actor_display_name: String,
    pub trigger: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub tool_calls: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStepResult {
    pub actor_handle: String,
    pub trigger: String,
    pub created_post: Option<FeedPost>,
    pub assistant_content: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_calls: Vec<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfEdits {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub specialty: Option<String>,
    pub persona_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunLog {
    pub reasoning: Option<String>,
    pub tool_calls_json: String,
    pub trigger: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentNote {
    pub id: i64,
    pub actor_id: i64,
    pub key: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i64,
    pub kind: String,
    pub actor_id: i64,
    pub actor: ActorSummary,
    pub post_id: Option<i64>,
    pub body: Option<String>,
    pub read: bool,
    pub created_at: i64,
}
