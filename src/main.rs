mod agent;
mod config;
mod openai;
mod tools;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Kazahane — lightweight AI coding agent
#[derive(Parser, Debug)]
#[command(name = "kazae", about = "Kazahane — lightweight AI coding agent", version)]
struct Cli {
    /// The task to perform
    task: String,

    /// Path to config JSON file
    #[arg(short, long, default_value = "Kazae.json")]
    config: PathBuf,

    /// Override model from config
    #[arg(short, long)]
    model: Option<String>,

    /// Override working directory
    #[arg(short, long)]
    dir: Option<String>,

    /// Override max tool calls
    #[arg(long)]
    max_tool_calls: Option<u32>,

    /// Enable thinking mode
    #[arg(long)]
    thinking: bool,

    /// Thinking budget tokens (implies --thinking)
    #[arg(long)]
    thinking_budget: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = config::Config::load(&cli.config)?;

    if let Some(model) = cli.model {
        config.model = model;
    }
    if let Some(dir) = cli.dir {
        config.working_dir = Some(dir);
    }
    if let Some(max_tool_calls) = cli.max_tool_calls {
        config.max_tool_calls = max_tool_calls;
    }
    if cli.thinking {
        config.thinking.enabled = true;
    }
    if let Some(budget) = cli.thinking_budget {
        config.thinking.enabled = true;
        config.thinking.budget_tokens = budget;
    }

    let agent = agent::Agent::new(config);
    agent.run(&cli.task).await?;

    Ok(())
}
