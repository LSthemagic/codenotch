use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default)]
    pub bar_x: Option<i32>,
    #[serde(default)]
    pub bar_y: Option<i32>,
    #[serde(default)]
    pub bar_w: Option<u32>,
    #[serde(default)]
    pub drag_enabled: bool,
    #[serde(default = "default_notch_y")]
    pub notch_y: f64,
}

fn default_notch_y() -> f64 { 0.5 }
fn default_port() -> u16 { 48666 }
fn default_lang() -> String { "auto".into() }

impl Default for Config {
    fn default() -> Self {
        Self { port: default_port(), lang: default_lang(), bar_x: None, bar_y: None, bar_w: None, drag_enabled: false, notch_y: default_notch_y() }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("nyrva").join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn save(cfg: &Config) {
    let path = config_path();
    if let Some(dir) = path.parent() { let _ = std::fs::create_dir_all(dir); }
    if let Ok(txt) = serde_json::to_string_pretty(cfg) { let _ = std::fs::write(path, txt); }
}
