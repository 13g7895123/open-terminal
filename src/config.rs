use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaneConfig {
    pub label: String,
    pub command: String,
}

impl Default for PaneConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            command: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub panes: [PaneConfig; 4],
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            panes: [
                PaneConfig { label: "Terminal 1".into(), command: String::new() },
                PaneConfig { label: "Terminal 2".into(), command: String::new() },
                PaneConfig { label: "Terminal 3".into(), command: String::new() },
                PaneConfig { label: "Terminal 4".into(), command: String::new() },
            ],
        }
    }
}

fn config_path() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("open-terminal").join("config.json")
}

pub fn load() -> AppConfig {
    let path = config_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str(&data) {
            return cfg;
        }
    }
    AppConfig::default()
}

pub fn save(cfg: &AppConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(&path, data);
    }
}
