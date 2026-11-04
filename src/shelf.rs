use crate::config::get_app_dir;
use std::fs;
use std::process::Command;

pub struct ShelfManager {
    pub items: Vec<String>,
}

impl ShelfManager {
    pub fn load() -> Self {
        let path = get_app_dir().join("shelf.json");
        let items = if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { items }
    }

    pub fn save(&self) {
        let path = get_app_dir().join("shelf.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.items) {
            let _ = fs::write(path, json);
        }
    }

    pub fn add(&mut self, file_path: String) {
        if !self.items.contains(&file_path) {
            self.items.insert(0, file_path);
            self.save();
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            self.save();
        }
    }

    pub fn open(&self, index: usize) {
        if let Some(path) = self.items.get(index) {
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("explorer").arg(path).spawn();
            }
        }
    }
}
