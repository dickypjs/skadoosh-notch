use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub city: String,
    pub province: String,
    pub scale: f32,
    pub minimal_mode: bool,
    pub auto_pause_adzan: bool,
    pub sound_adzan_path: String,
    pub screen_timeout_min: u32,
    pub sleep_timeout_min: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            city: "Jakarta".to_string(),
            province: "DKI Jakarta".to_string(),
            scale: 1.0,
            minimal_mode: true,
            auto_pause_adzan: true,
            sound_adzan_path: String::new(),
            screen_timeout_min: 10,
            sleep_timeout_min: 30,
        }
    }
}

pub fn get_app_dir() -> PathBuf {
    let mut dir = dirs_or_fallback();
    dir.push("SkadooshNotch");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn dirs_or_fallback() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata)
    } else {
        PathBuf::from(".")
    }
}

impl Config {
    pub fn load() -> Self {
        let path = get_app_dir().join("config.json");
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
        let default_cfg = Self::default();
        default_cfg.save();
        default_cfg
    }

    pub fn save(&self) {
        let path = get_app_dir().join("config.json");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}
