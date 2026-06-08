use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TerminalProfile {
    pub guid: String,
    pub name: String,
    pub source: Option<String>,
    pub hidden: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaneConfig {
    pub enabled: bool,
    pub label: String,
    #[serde(default, alias = "distro")]
    pub profile: String, // Windows Terminal profile guid，空字串代表預設
    pub command: String,
}

impl Default for PaneConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            label: String::new(),
            profile: String::new(),
            command: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub panes: [PaneConfig; 4],
    #[serde(default)]
    pub dual_monitor: bool,
    #[serde(default = "default_panes2")]
    pub panes2: [PaneConfig; 4],
    #[serde(default)]
    pub default_profile: String,
}

pub fn import(path: &std::path::Path) -> Option<AppConfig> {
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn export(cfg: &AppConfig, path: &std::path::Path) -> Result<(), String> {
    let data = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())
}

fn default_panes2() -> [PaneConfig; 4] {
    [
        PaneConfig { enabled: true,  label: "Monitor2 - 1".into(), profile: String::new(), command: String::new() },
        PaneConfig { enabled: true,  label: "Monitor2 - 2".into(), profile: String::new(), command: String::new() },
        PaneConfig { enabled: true,  label: "Monitor2 - 3".into(), profile: String::new(), command: String::new() },
        PaneConfig { enabled: true,  label: "Monitor2 - 4".into(), profile: String::new(), command: String::new() },
    ]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            panes: [
                PaneConfig { enabled: true, label: "Terminal 1".into(), profile: String::new(), command: String::new() },
                PaneConfig { enabled: true, label: "Terminal 2".into(), profile: String::new(), command: String::new() },
                PaneConfig { enabled: true, label: "Terminal 3".into(), profile: String::new(), command: String::new() },
                PaneConfig { enabled: true, label: "Terminal 4".into(), profile: String::new(), command: String::new() },
            ],
            dual_monitor: false,
            panes2: default_panes2(),
            default_profile: String::new(),
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

pub fn list_terminal_profiles() -> Vec<TerminalProfile> {
    let settings = terminal_settings_candidates()
        .into_iter()
        .find_map(|path| std::fs::read_to_string(path).ok());

    let Some(settings) = settings else {
        return vec![];
    };

    let parsed: TerminalSettings = match serde_json::from_str(&settings) {
        Ok(parsed) => parsed,
        Err(_) => return vec![],
    };

    let mut profiles: Vec<TerminalProfile> = parsed
        .profiles
        .list
        .into_iter()
        .map(|p| TerminalProfile {
            guid: p.guid,
            name: p.name,
            source: p.source,
            hidden: p.hidden,
        })
        .collect();

    profiles.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.guid.cmp(&b.guid)));
    profiles
}

fn terminal_settings_candidates() -> Vec<PathBuf> {
    let local_app_data = std::env::var("LOCALAPPDATA").ok();
    let app_data = std::env::var("APPDATA").ok();
    let mut paths = Vec::new();

    if let Some(local) = &local_app_data {
        let local = PathBuf::from(local);
        paths.push(
            local
                .join("Packages")
                .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
                .join("LocalState")
                .join("settings.json"),
        );
        paths.push(
            local
                .join("Packages")
                .join("Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe")
                .join("LocalState")
                .join("settings.json"),
        );
        paths.push(
            local
                .join("Microsoft")
                .join("Windows Terminal")
                .join("settings.json"),
        );
    }

    if let Some(app) = &app_data {
        paths.push(
            PathBuf::from(app)
                .join("Microsoft")
                .join("Windows Terminal")
                .join("settings.json"),
        );
    }

    paths
}

#[derive(Deserialize)]
struct TerminalSettings {
    profiles: TerminalProfiles,
}

#[derive(Deserialize)]
struct TerminalProfiles {
    list: Vec<TerminalProfileRecord>,
}

#[derive(Deserialize)]
struct TerminalProfileRecord {
    guid: String,
    hidden: bool,
    name: String,
    source: Option<String>,
}
