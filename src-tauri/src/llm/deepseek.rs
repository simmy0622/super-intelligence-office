use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::agents::tools::ToolDefinition;

pub const DEEPSEEK_ENDPOINT: &str = "https://api.deepseek.com/v1/chat/completions";
pub const DEEPSEEK_MODEL: &str = "deepseek-reasoner";
const HTTP_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AgentSalon/0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
    #[serde(default)]
    pub usage: Option<ChatUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content.into()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.into()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content.into()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ChatToolFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProviderKeys {
    providers: BTreeMap<String, String>,
}

pub struct DeepSeekClient {
    api_key: String,
    endpoint: String,
    http: Client,
}

impl DeepSeekClient {
    pub fn from_config() -> Result<Self, String> {
        let config_path = config_path();
        let content = fs::read_to_string(&config_path)
            .map_err(|error| format!("failed to read {}: {}", config_path.display(), error))?;
        let keys =
            toml::from_str::<ProviderKeys>(&content).map_err(|error| format!("invalid keys.toml: {}", error))?;
        let api_key = keys
            .providers
            .get("deepseek")
            .cloned()
            .ok_or_else(|| "missing [providers].deepseek in keys.toml".to_string())?;

        let http = Client::builder()
            .timeout(Duration::from_secs(90))
            .http1_only()
            .user_agent(HTTP_USER_AGENT)
            .build()
            .map_err(|error| error.to_string())?;

        Ok(Self {
            api_key,
            endpoint: DEEPSEEK_ENDPOINT.to_string(),
            http,
        })
    }

    pub async fn chat_completion(&self, request: &ChatRequest) -> Result<ChatResponse, String> {
        let response = self
            .http
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        let status = response.status();
        let text = response.text().await.map_err(|error| error.to_string())?;

        if !status.is_success() {
            return Err(format!("DeepSeek API {}: {}", status, text));
        }

        serde_json::from_str::<ChatResponse>(&text)
            .map_err(|error| format!("failed to decode DeepSeek response: {}. body={}", error, text))
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("agent-salon")
        .join("keys.toml")
}
