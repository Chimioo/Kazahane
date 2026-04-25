use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    /// Either a plain string or a content block array (thinking mode)
    pub content: Option<Value>,
    /// Reasoning content from thinking mode — must be passed back to API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(Value::String(content.into())),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(Value::String(content.into())),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(Value::String(content.into())),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            name: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
}

/// Assistant message from API — content may be a string or a block array
#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    /// Raw content value: string in normal mode, array of blocks in thinking mode
    pub content: Option<Value>,
    /// Reasoning content from thinking mode — must be passed back to API
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl ResponseMessage {
    /// Extract displayable text from content (works for both string and block array)
    pub fn text(&self) -> Option<String> {
        match &self.content {
            Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
            Some(Value::Array(blocks)) => {
                let text = blocks
                    .iter()
                    .filter_map(|b| {
                        if b["type"].as_str() == Some("text") {
                            b["text"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if text.is_empty() { None } else { Some(text) }
            }
            _ => None,
        }
    }
}

pub struct OpenAIClient {
    client: reqwest::Client,
    config: Config,
}

impl OpenAIClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn chat(&self, messages: &[Message], tools: &Value) -> Result<ChatResponse> {
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );

        let mut body = json!({
            "model": self.config.model,
            "messages": messages,
            "tools": tools,
            "tool_choice": "auto",
        });

        if self.config.thinking.enabled {
            body["thinking"] = json!({
                "type": "enabled",
                "budget_tokens": self.config.thinking.budget_tokens
            });
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to API")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("API error {status}: {text}");
        }

        resp.json().await.context("Failed to parse API response")
    }
}
