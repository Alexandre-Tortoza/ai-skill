use std::path::PathBuf;

use ai_skill_core::{CONFIG_DIR, CONFIG_FILE, ConfigStore, TuiConfig};
use serde::{Deserialize, Serialize};

fn config_path() -> Result<PathBuf, std::io::Error> {
    let home = std::env::var_os("HOME").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME is not set; set HOME or run ai-skill from a login shell",
        )
    })?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join(CONFIG_DIR)
        .join(CONFIG_FILE))
}

pub struct FsConfigStore {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct RawConfig {
    #[serde(default)]
    custom_agent_paths: std::collections::HashMap<String, PathBuf>,
    #[serde(default)]
    theme: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    keymap: std::collections::HashMap<String, String>,
    #[serde(default)]
    proxy: Option<String>,
    #[serde(default = "ai_skill_core::config::default_stale_after_days")]
    stale_after_days: u64,
    #[serde(default)]
    locale: Option<String>,
}

impl From<RawConfig> for TuiConfig {
    fn from(r: RawConfig) -> Self {
        TuiConfig {
            custom_agent_paths: r.custom_agent_paths,
            theme: r.theme,
            keymap: r.keymap,
            proxy: r.proxy,
            stale_after_days: r.stale_after_days,
            locale: r.locale,
        }
    }
}

impl From<TuiConfig> for RawConfig {
    fn from(c: TuiConfig) -> Self {
        RawConfig {
            custom_agent_paths: c.custom_agent_paths,
            theme: c.theme,
            keymap: c.keymap,
            proxy: c.proxy,
            stale_after_days: c.stale_after_days,
            locale: c.locale,
        }
    }
}

impl FsConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn from_env() -> Result<Self, std::io::Error> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self { path })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl ConfigStore for FsConfigStore {
    fn read(&self) -> Result<TuiConfig, Box<dyn std::error::Error>> {
        if !self.path.exists() {
            return Ok(TuiConfig::default());
        }
        let content = std::fs::read_to_string(&self.path)?;
        let raw: RawConfig = serde_json::from_str(&content)?;
        Ok(raw.into())
    }

    fn write(&self, config: &TuiConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw: RawConfig = config.clone().into();
        let content = serde_json::to_string_pretty(&raw)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, FsConfigStore) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        (dir, FsConfigStore::new(path))
    }

    #[test]
    fn read_nonexistent_returns_defaults() {
        let (_dir, store) = make_store();
        let config = store.read().unwrap();
        assert!(config.custom_agent_paths.is_empty());
        assert!(config.theme.is_none());
        assert!(config.keymap.is_empty());
        assert!(config.proxy.is_none());
    }

    #[test]
    fn write_then_read_roundtrip() {
        let (_dir, store) = make_store();
        let mut paths = std::collections::HashMap::new();
        paths.insert("cursor".into(), PathBuf::from("/custom/cursor/skills"));
        let config = TuiConfig {
            custom_agent_paths: paths,
            theme: Some([("primary".into(), "blue".into())].into_iter().collect()),
            keymap: [("quit".into(), "q".into())].into_iter().collect(),
            proxy: Some("http://proxy:8080".into()),
            stale_after_days: 30,
            locale: Some("pt-BR".into()),
        };
        store.write(&config).unwrap();
        let read_back = store.read().unwrap();
        assert_eq!(
            read_back.custom_agent_paths.get("cursor").unwrap(),
            &PathBuf::from("/custom/cursor/skills")
        );
        assert_eq!(
            read_back.theme.as_ref().unwrap().get("primary").unwrap(),
            "blue"
        );
        assert_eq!(read_back.keymap.get("quit").unwrap(), "q");
        assert_eq!(read_back.proxy.as_ref().unwrap(), "http://proxy:8080");
    }

    #[test]
    fn write_creates_parent_dir() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("sub").join("config.json");
        let store = FsConfigStore::new(path.clone());
        let config = TuiConfig::default();
        store.write(&config).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn read_existing_json_parses_correctly() {
        let (_dir, store) = make_store();
        let json = r#"{"custom_agent_paths":{"cursor":"/custom/path"},"theme":{"primary":"red"}}"#;
        std::fs::write(store.path(), json).unwrap();
        let config = store.read().unwrap();
        assert_eq!(
            config.custom_agent_paths.get("cursor").unwrap(),
            &PathBuf::from("/custom/path")
        );
        assert_eq!(config.theme.unwrap().get("primary").unwrap(), "red");
    }

    #[test]
    fn read_file_with_extra_fields_ignores_them() {
        let (_dir, store) = make_store();
        let json = r#"{"custom_agent_paths":{},"extraField":123}"#;
        std::fs::write(store.path(), json).unwrap();
        let config = store.read().unwrap();
        assert!(config.custom_agent_paths.is_empty());
    }

    #[test]
    fn default_config_has_empty_paths() {
        let default = TuiConfig::default();
        assert!(default.custom_agent_paths.is_empty());
    }
}
