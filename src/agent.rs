use anyhow::Result;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::openai::{Message, OpenAIClient};
use crate::tools;
use crate::tools::todo::TodoStore;

const DEFAULT_SYSTEM_PROMPT: &str =
    "You are Kazahane, a capable and precise coding agent. \
     You have access to tools for reading, writing, and searching files, running commands, \
     and managing a todo list to plan your work. \
     Work step by step. Read files before editing them. \
     When done, summarize what you did concisely.";

pub struct Agent {
    config: Config,
    client: OpenAIClient,
    tools: Value,
}

impl Agent {
    pub fn new(config: Config) -> Self {
        let client = OpenAIClient::new(config.clone());
        let tools = tools::tool_definitions();
        Self { config, client, tools }
    }

    pub async fn run(&self, task: &str) -> Result<()> {
        let system_prompt = self
            .config
            .system_prompt
            .as_deref()
            .unwrap_or(DEFAULT_SYSTEM_PROMPT);

        let mut messages: Vec<Message> = vec![
            Message::system(system_prompt),
            Message::user(task),
        ];

        let todo_store = Arc::new(Mutex::new(TodoStore::new()));
        let mut tool_calls_count: u32 = 0;

        println!("Task: {task}");

        loop {
            let response = self.client.chat(&messages, &self.tools).await?;

            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No choices in API response"))?;

            let msg = choice.message;

            // Print text content if any
            if let Some(text) = msg.text() {
                println!("{text}");
            }

            let tool_calls = msg.tool_calls.clone();

            // Push assistant message with full original content (preserves reasoning_content blocks)
            messages.push(Message {
                role: "assistant".to_string(),
                content: msg.content,
                reasoning_content: msg.reasoning_content,
                tool_calls: msg.tool_calls,
                tool_call_id: None,
                name: None,
            });

            match tool_calls {
                Some(calls) if !calls.is_empty() => {
                    for tc in &calls {
                        if tool_calls_count >= self.config.max_tool_calls {
                            println!(
                                "Reached max_tool_calls limit ({}).",
                                self.config.max_tool_calls
                            );
                            return Ok(());
                        }
                        tool_calls_count += 1;

                        let args: Value = serde_json::from_str(&tc.function.arguments)
                            .unwrap_or(Value::Object(Default::default()));

                        let is_read = tc.function.name == "read";
                        println!("tool: {}  {}", tc.function.name, format_args_summary(&args));

                        let result = tools::dispatch(
                            &tc.function.name,
                            &args,
                            &self.config,
                            &todo_store,
                        )
                        .await;

                        let result_str = match result {
                            Ok(s) => s,
                            Err(e) => format!("Error: {e}"),
                        };

                        if !is_read {
                            println!("{}", preview_lines(&result_str, 8));
                        }

                        messages.push(Message::tool_result(&tc.id, result_str));
                    }
                }
                _ => {
                    println!("Done.");
                    break;
                }
            }
        }

        Ok(())
    }
}

fn format_args_summary(args: &Value) -> String {
    if let Value::Object(map) = args {
        map.iter()
            .map(|(k, v)| {
                let val = match v {
                    Value::String(s) => {
                        let s = s.replace('\n', "\\n");
                        if s.char_indices().nth(50).is_some() {
                            format!("\"{}...\"", s.chars().take(50).collect::<String>())
                        } else {
                            format!("\"{s}\"")
                        }
                    }
                    other => {
                        let s = other.to_string();
                        if s.char_indices().nth(50).is_some() {
                            format!("{}...", s.chars().take(50).collect::<String>())
                        } else {
                            s
                        }
                    }
                };
                format!("{k}={val}")
            })
            .collect::<Vec<_>>()
            .join("  ")
    } else {
        String::new()
    }
}

fn preview_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let truncated = lines.len() > n;
    let shown = &lines[..lines.len().min(n)];
    let indented = shown
        .iter()
        .map(|l| format!("  {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    if truncated {
        format!("{indented}\n  ... ({} more lines)", lines.len() - n)
    } else {
        indented
    }
}
