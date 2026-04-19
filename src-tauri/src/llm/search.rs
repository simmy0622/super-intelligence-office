use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const TAVILY_ENDPOINT: &str = "https://api.tavily.com/search";
pub const EXA_ENDPOINT: &str = "https://api.exa.ai/search";
const HTTP_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AgentSalon/0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub title: String,
    pub url: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub provider: String,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    pub results: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchHit {
    pub title: String,
    pub image_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchResponse {
    pub provider: String,
    pub query: String,
    pub results: Vec<ImageSearchHit>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProviderKeys {
    providers: BTreeMap<String, String>,
}

pub struct SearchClient {
    tavily_key: Option<String>,
    exa_key: Option<String>,
    http: Client,
}

impl SearchClient {
    pub fn from_config() -> Result<Self, String> {
        let config_path = config_path();
        let content = fs::read_to_string(&config_path)
            .map_err(|error| format!("failed to read {}: {}", config_path.display(), error))?;
        let keys = toml::from_str::<ProviderKeys>(&content)
            .map_err(|error| format!("invalid keys.toml: {}", error))?;

        let tavily_key = keys.providers.get("tavily").cloned();
        let exa_key = keys.providers.get("exa").cloned();

        if tavily_key.is_none() && exa_key.is_none() {
            return Err("no search provider key (tavily/exa) configured".to_string());
        }

        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .http1_only()
            .user_agent(HTTP_USER_AGENT)
            .build()
            .map_err(|error| error.to_string())?;

        Ok(Self {
            tavily_key,
            exa_key,
            http,
        })
    }

    pub async fn search(&self, query: &str, max_results: usize) -> Result<SearchResponse, String> {
        let limit = max_results.clamp(1, 10);

        let mut errors = Vec::new();
        if let Some(key) = &self.tavily_key {
            match self.tavily_search(key, query, limit).await {
                Ok(response) => return Ok(response),
                Err(error) => errors.push(format!("tavily: {error}")),
            }
        }

        if let Some(key) = &self.exa_key {
            match self.exa_search(key, query, limit).await {
                Ok(response) => return Ok(response),
                Err(error) => errors.push(format!("exa: {error}")),
            }
        }

        Err(if errors.is_empty() {
            "no search provider configured".to_string()
        } else {
            errors.join("; ")
        })
    }

    pub async fn search_images(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<ImageSearchResponse, String> {
        let limit = max_results.clamp(1, 10);
        let Some(key) = &self.tavily_key else {
            return Err("image search requires a tavily provider key".to_string());
        };

        let body = json!({
            "api_key": key,
            "query": query,
            "max_results": limit,
            "search_depth": "basic",
            "include_images": true,
            "include_image_descriptions": true,
        });

        let response = self
            .http
            .post(TAVILY_ENDPOINT)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        let status = response.status();
        let text = response.text().await.map_err(|error| error.to_string())?;
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, text));
        }

        let value: Value = serde_json::from_str(&text)
            .map_err(|error| format!("decode tavily images: {error}. body={text}"))?;
        let results = parse_tavily_images(&value, limit);

        Ok(ImageSearchResponse {
            provider: "tavily".to_string(),
            query: query.to_string(),
            results,
        })
    }

    async fn tavily_search(
        &self,
        api_key: &str,
        query: &str,
        limit: usize,
    ) -> Result<SearchResponse, String> {
        let body = json!({
            "api_key": api_key,
            "query": query,
            "max_results": limit,
            "search_depth": "basic",
            "include_answer": true,
        });

        let response = self
            .http
            .post(TAVILY_ENDPOINT)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        let status = response.status();
        let text = response.text().await.map_err(|error| error.to_string())?;
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, text));
        }

        let value: Value = serde_json::from_str(&text)
            .map_err(|error| format!("decode tavily: {error}. body={text}"))?;

        let answer = value
            .get("answer")
            .and_then(Value::as_str)
            .map(str::to_string);
        let results = value
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| SearchHit {
                title: item
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                url: item
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                snippet: item
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                score: item.get("score").and_then(Value::as_f64),
            })
            .collect::<Vec<_>>();

        Ok(SearchResponse {
            provider: "tavily".to_string(),
            query: query.to_string(),
            answer,
            results,
        })
    }

    async fn exa_search(
        &self,
        api_key: &str,
        query: &str,
        limit: usize,
    ) -> Result<SearchResponse, String> {
        let body = json!({
            "query": query,
            "numResults": limit,
            "contents": { "text": { "maxCharacters": 800 } },
        });

        let response = self
            .http
            .post(EXA_ENDPOINT)
            .header("Content-Type", "application/json")
            .header("x-api-key", api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        let status = response.status();
        let text = response.text().await.map_err(|error| error.to_string())?;
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, text));
        }

        let value: Value = serde_json::from_str(&text)
            .map_err(|error| format!("decode exa: {error}. body={text}"))?;

        let results = value
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| SearchHit {
                title: item
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                url: item
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                snippet: item
                    .get("text")
                    .and_then(Value::as_str)
                    .or_else(|| item.get("snippet").and_then(Value::as_str))
                    .unwrap_or("")
                    .to_string(),
                score: item.get("score").and_then(Value::as_f64),
            })
            .collect::<Vec<_>>();

        Ok(SearchResponse {
            provider: "exa".to_string(),
            query: query.to_string(),
            answer: None,
            results,
        })
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("agent-salon")
        .join("keys.toml")
}

fn parse_tavily_images(value: &Value, limit: usize) -> Vec<ImageSearchHit> {
    value
        .get("images")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| parse_tavily_image_item(&item))
        .take(limit)
        .collect()
}

fn parse_tavily_image_item(item: &Value) -> Option<ImageSearchHit> {
    if let Some(url) = item.as_str().filter(|url| is_https_url(url)) {
        return Some(ImageSearchHit {
            title: String::new(),
            image_url: url.to_string(),
            thumbnail_url: None,
            source_url: None,
            width: None,
            height: None,
            provider: "tavily".to_string(),
        });
    }

    let image_url = first_string(item, &["url", "image_url", "imageUrl"])
        .filter(|url| is_https_url(url))?;
    let thumbnail_url = first_string(item, &["thumbnail_url", "thumbnailUrl", "thumbnail"])
        .filter(|url| is_https_url(url))
        .map(str::to_string);
    let source_url = first_string(item, &["source_url", "sourceUrl", "source"])
        .filter(|url| is_http_url(url))
        .map(str::to_string);
    let title = first_string(item, &["title", "description", "alt"])
        .unwrap_or("")
        .chars()
        .take(160)
        .collect::<String>();

    Some(ImageSearchHit {
        title,
        image_url: image_url.to_string(),
        thumbnail_url,
        source_url,
        width: item.get("width").and_then(Value::as_i64),
        height: item.get("height").and_then(Value::as_i64),
        provider: "tavily".to_string(),
    })
}

fn first_string<'a>(item: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| item.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn is_https_url(url: &str) -> bool {
    url.trim_start().starts_with("https://")
}

fn is_http_url(url: &str) -> bool {
    let trimmed = url.trim_start();
    trimmed.starts_with("https://") || trimmed.starts_with("http://")
}
