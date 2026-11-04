use crate::config::get_app_dir;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteItem {
    pub id: String,
    pub content: String,
    pub updated_at: String,
}

pub struct NoteManager {
    pub items: Vec<NoteItem>,
}

impl NoteManager {
    pub fn load() -> Self {
        let path = get_app_dir().join("notes.json");
        let items = if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { items }
    }

    pub fn save(&self) {
        let path = get_app_dir().join("notes.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.items) {
            let _ = fs::write(path, json);
        }
    }

    pub fn save_active_note(&mut self, content: String) {
        let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        if self.items.is_empty() {
            self.items.push(NoteItem {
                id: "note_1".to_string(),
                content,
                updated_at: now_str,
            });
        } else {
            self.items[0].content = content;
            self.items[0].updated_at = now_str;
        }
        self.save();
    }
}
