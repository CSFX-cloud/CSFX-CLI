use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub token: Option<String>,
    pub compose_dir: Option<String>,
    pub ghcr_org: Option<String>,
    #[serde(default)]
    pub ca_pem: Option<String>,
}

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("home dir not found");
    path.push(".csf");
    path.push("config.json");
    path
}

pub fn get_history_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("home dir not found");
    path.push(".csf");
    path.push("history");
    path
}

pub fn load_config() -> Option<Config> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(config_path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    // Erstelle das Verzeichnis falls es nicht existiert
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;

    Ok(())
}
