use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub api_key: String,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default)]
    pub system_prompt: Option<String>,

    #[serde(default = "default_max_tool_calls")]
    pub max_tool_calls: u32,

    #[serde(default)]
    pub command: CommandConfig,

    #[serde(default)]
    pub working_dir: Option<String>,

    #[serde(default)]
    pub thinking: ThinkingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ThinkingConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_budget_tokens")]
    pub budget_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandConfig {
    #[serde(default = "default_timeout_secs")]
    pub default_timeout_secs: u64,

    #[serde(default = "default_max_timeout_secs")]
    pub max_timeout_secs: u64,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: default_timeout_secs(),
            max_timeout_secs: default_max_timeout_secs(),
        }
    }
}

fn default_base_url() -> String { "https://api.openai.com/v1".to_string() }
fn default_model() -> String { "gpt-4o".to_string() }
fn default_max_tool_calls() -> u32 { 100 }
fn default_timeout_secs() -> u64 { 30 }
fn default_max_timeout_secs() -> u64 { 300 }
fn default_budget_tokens() -> u32 { 8000 }

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let mut config: Config =
            serde_json::from_str(&content).context("Failed to parse config JSON")?;

        if let Ok(key) = std::env::var("KAZAHANE_API_KEY") {
            config.api_key = key;
        } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.api_key = key;
        }

        if config.api_key.is_empty() {
            anyhow::bail!(
                "API key not set. Use KAZAHANE_API_KEY, OPENAI_API_KEY, or 'api_key' in config."
            );
        }

        Ok(config)
    }
}
