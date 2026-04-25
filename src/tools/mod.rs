pub mod append;
pub mod command;
pub mod edit;
pub mod glob_tool;
pub mod grep;
pub mod read;
pub mod todo;
pub mod write;

use anyhow::Result;
use serde_json::{json, Value};

use crate::config::Config;
use todo::SharedTodo;

pub fn tool_definitions() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "grep",
                "description": "Search file contents using a pattern. Supports literal and regex search. Returns matching lines with line numbers.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Search pattern (literal string or regex)" },
                        "path": { "type": "string", "description": "File or directory to search. If directory, searches recursively." },
                        "regex": { "type": "boolean", "description": "If true, treat pattern as regex. Default: false." },
                        "case_sensitive": { "type": "boolean", "description": "If false, case-insensitive. Default: true." },
                        "include_glob": { "type": "string", "description": "Only search files matching this glob, e.g. '*.rs'" }
                    },
                    "required": ["pattern", "path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "glob",
                "description": "Find files matching a glob pattern.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Glob pattern, e.g. 'src/**/*.rs'" }
                    },
                    "required": ["pattern"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read the contents of a file. Optionally specify a line range.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to the file" },
                        "start_line": { "type": "integer", "description": "First line to read (1-indexed). Omit for start of file." },
                        "end_line": { "type": "integer", "description": "Last line to read (1-indexed, inclusive). Omit for end of file." }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "write",
                "description": "Write content to a NEW file. Fails if file already exists.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path of the file to create" },
                        "content": { "type": "string", "description": "Content to write" }
                    },
                    "required": ["path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "edit",
                "description": "Edit an existing file by finding and replacing text. old_str must match exactly.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to the file" },
                        "old_str": { "type": "string", "description": "Exact string to find" },
                        "new_str": { "type": "string", "description": "String to replace it with" },
                        "occurrence": { "type": "integer", "description": "Which occurrence to replace (1-indexed). Default: all." }
                    },
                    "required": ["path", "old_str", "new_str"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "append",
                "description": "Append content to the end of an existing file.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to the file" },
                        "content": { "type": "string", "description": "Content to append" }
                    },
                    "required": ["path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "command",
                "description": "Execute a shell command via /bin/sh -c. Captures stdout and stderr.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "cmd": { "type": "string", "description": "Command to execute" },
                        "timeout_secs": { "type": "integer", "description": "Timeout in seconds. Uses config default if omitted." },
                        "working_dir": { "type": "string", "description": "Directory to run in. Uses agent working dir if omitted." }
                    },
                    "required": ["cmd"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "todo",
                "description": "Manage an in-memory todo list for planning the current task. Use this to track your progress.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["add", "list", "done", "delete", "edit"],
                            "description": "Action to perform"
                        },
                        "title": { "type": "string", "description": "Title of the todo item (required for add, optional for edit)" },
                        "id": { "type": "integer", "description": "Todo item ID (required for done, delete, edit)" },
                        "priority": {
                            "type": "string",
                            "enum": ["high", "medium", "low"],
                            "description": "Priority level (optional)"
                        }
                    },
                    "required": ["action"]
                }
            }
        }
    ])
}

pub async fn dispatch(name: &str, args: &Value, config: &Config, todo_store: &SharedTodo) -> Result<String> {
    let working_dir = config.working_dir.as_deref().unwrap_or(".").to_string();

    match name {
        "grep" => grep::run(args, &working_dir),
        "glob" => glob_tool::run(args, &working_dir),
        "read" => read::run(args, &working_dir),
        "write" => write::run(args, &working_dir),
        "edit" => edit::run(args, &working_dir),
        "append" => append::run(args, &working_dir),
        "command" => command::run(args, config).await,
        "todo" => todo::run(args, todo_store),
        _ => Ok(format!("Unknown tool: {name}")),
    }
}
