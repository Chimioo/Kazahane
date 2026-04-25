use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: u32,
    pub title: String,
    pub status: TodoStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TodoPriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Todo,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Default)]
pub struct TodoStore {
    items: Vec<TodoItem>,
    next_id: u32,
}

impl TodoStore {
    pub fn new() -> Self {
        Self { items: Vec::new(), next_id: 1 }
    }

    pub fn add(&mut self, title: String, priority: Option<TodoPriority>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(TodoItem { id, title, status: TodoStatus::Todo, priority });
        id
    }

    pub fn list(&self) -> &[TodoItem] {
        &self.items
    }

    pub fn done(&mut self, id: u32) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = TodoStatus::Done;
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self, id: u32) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() < before
    }

    pub fn edit(&mut self, id: u32, title: Option<String>, priority: Option<TodoPriority>) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            if let Some(t) = title { item.title = t; }
            if let Some(p) = priority { item.priority = Some(p); }
            true
        } else {
            false
        }
    }
}

pub type SharedTodo = Arc<Mutex<TodoStore>>;

pub fn run(args: &Value, store: &SharedTodo) -> Result<String> {
    let action = args["action"].as_str().unwrap_or("");
    let mut store = store.lock().unwrap();

    match action {
        "add" => {
            let title = args["title"].as_str().unwrap_or("").to_string();
            if title.is_empty() {
                return Ok("Error: title is required for add".to_string());
            }
            let priority = parse_priority(args["priority"].as_str());
            let id = store.add(title.clone(), priority);
            Ok(format!("Added todo #{id}: {title}"))
        }

        "list" => {
            let items = store.list();
            if items.is_empty() {
                return Ok("Todo list is empty.".to_string());
            }
            let lines: Vec<String> = items
                .iter()
                .map(|item| {
                    let status = match item.status {
                        TodoStatus::Todo => "[ ]",
                        TodoStatus::Done => "[x]",
                    };
                    let priority = item
                        .priority
                        .as_ref()
                        .map(|p| format!(" ({p:?})").to_lowercase())
                        .unwrap_or_default();
                    format!("#{} {} {}{}", item.id, status, item.title, priority)
                })
                .collect();
            Ok(lines.join("\n"))
        }

        "done" => {
            let id = match args["id"].as_u64() {
                Some(n) => n as u32,
                None => return Ok("Error: id is required for done".to_string()),
            };
            if store.done(id) {
                Ok(format!("Marked #{id} as done"))
            } else {
                Ok(format!("Todo #{id} not found"))
            }
        }

        "delete" => {
            let id = match args["id"].as_u64() {
                Some(n) => n as u32,
                None => return Ok("Error: id is required for delete".to_string()),
            };
            if store.delete(id) {
                Ok(format!("Deleted #{id}"))
            } else {
                Ok(format!("Todo #{id} not found"))
            }
        }

        "edit" => {
            let id = match args["id"].as_u64() {
                Some(n) => n as u32,
                None => return Ok("Error: id is required for edit".to_string()),
            };
            let title = args["title"].as_str().map(|s| s.to_string());
            let priority = args["priority"].as_str().and_then(|s| parse_priority(Some(s)));
            if store.edit(id, title, priority) {
                Ok(format!("Updated #{id}"))
            } else {
                Ok(format!("Todo #{id} not found"))
            }
        }

        _ => Ok(format!("Unknown action: '{action}'. Use: add, list, done, delete, edit")),
    }
}

fn parse_priority(s: Option<&str>) -> Option<TodoPriority> {
    match s? {
        "high" => Some(TodoPriority::High),
        "medium" => Some(TodoPriority::Medium),
        "low" => Some(TodoPriority::Low),
        _ => None,
    }
}
