use serde::{Deserialize, Serialize};
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = ".csfx";
const LEGACY_CONFIG_DIR: &str = ".csf";
const DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub token: Option<String>,
    pub compose_dir: Option<String>,
    pub ghcr_org: Option<String>,
    #[serde(default)]
    pub ca_pem: Option<String>,
}

fn home() -> PathBuf {
    dirs::home_dir().expect("home dir not found")
}

pub fn config_dir() -> PathBuf {
    home().join(CONFIG_DIR)
}

pub fn get_config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn get_history_path() -> PathBuf {
    config_dir().join("history")
}

fn restrict(path: &Path, mode: u32) -> io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

fn migrate_legacy_dir(target: &Path) -> io::Result<()> {
    let legacy = home().join(LEGACY_CONFIG_DIR);
    if target.exists() || !legacy.is_dir() {
        return Ok(());
    }
    fs::rename(legacy, target)
}

pub fn create_private_dir(path: &Path) -> io::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(DIRECTORY_MODE)
        .create(path)?;
    restrict(path, DIRECTORY_MODE)
}

pub fn ensure_private_dir() -> io::Result<PathBuf> {
    let directory = config_dir();
    migrate_legacy_dir(&directory)?;
    create_private_dir(&directory)?;
    for entry in [get_config_path(), get_history_path()] {
        if entry.exists() {
            restrict(&entry, FILE_MODE)?;
        }
    }
    Ok(directory)
}

pub fn write_private(path: &Path, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(FILE_MODE)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(FILE_MODE))?;
    file.write_all(content.as_bytes())
}

pub fn load_config() -> Option<Config> {
    ensure_private_dir().ok()?;
    let content = fs::read_to_string(get_config_path()).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    ensure_private_dir()?;
    let content = serde_json::to_string_pretty(config)?;
    write_private(&get_config_path(), &content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode_of(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn private_directories_and_files_are_owner_only() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("nested").join(".csfx");
        create_private_dir(&directory).unwrap();
        assert_eq!(mode_of(&directory), DIRECTORY_MODE);

        let file = directory.join("config.json");
        write_private(&file, "secret").unwrap();
        assert_eq!(mode_of(&file), FILE_MODE);
        assert_eq!(fs::read_to_string(&file).unwrap(), "secret");
    }

    #[test]
    fn existing_permissive_entries_are_tightened() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("shared");
        fs::create_dir(&directory).unwrap();
        restrict(&directory, 0o755).unwrap();
        create_private_dir(&directory).unwrap();
        assert_eq!(mode_of(&directory), DIRECTORY_MODE);

        let file = directory.join("config.json");
        fs::write(&file, "old").unwrap();
        restrict(&file, 0o644).unwrap();
        write_private(&file, "new").unwrap();
        assert_eq!(mode_of(&file), FILE_MODE);
    }
}
