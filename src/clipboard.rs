use crate::config::get_app_dir;
use std::fs;

pub struct ClipboardManager {
    pub history: Vec<String>,
    pub last_copied: String,
}

impl ClipboardManager {
    pub fn load() -> Self {
        let path = get_app_dir().join("clipboard.json");
        let history = if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Self {
            history,
            last_copied: String::new(),
        }
    }

    pub fn save(&self) {
        let path = get_app_dir().join("clipboard.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.history) {
            let _ = fs::write(path, json);
        }
    }

    pub fn check_and_add(&mut self, text: String) {
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed != self.last_copied {
            self.last_copied = trimmed.to_string();
            // Simpan max 100 item (tanpa batas 5 item versi Nusa Free)
            self.history.retain(|x| x != trimmed);
            self.history.insert(0, trimmed.to_string());
            if self.history.len() > 100 {
                self.history.truncate(100);
            }
            self.save();
        }
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.save();
    }
}
