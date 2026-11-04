use crate::config::get_app_dir;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub deadline: Option<String>, // format: YYYY-MM-DD
    pub done: bool,
}

pub struct TodoManager {
    pub items: Vec<TodoItem>,
}

impl TodoManager {
    pub fn load() -> Self {
        let path = get_app_dir().join("todos.json");
        let items = if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { items }
    }

    pub fn save(&self) {
        let path = get_app_dir().join("todos.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.items) {
            let _ = fs::write(path, json);
        }
    }

    pub fn add(&mut self, title: String, deadline: Option<String>) {
        let id = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        self.items.push(TodoItem {
            id,
            title,
            deadline,
            done: false,
        });
        self.save();
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.done = !item.done;
            self.save();
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            self.save();
        }
    }
}
