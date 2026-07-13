use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TuiConfig {
    #[serde(default)]
    pub custom_agent_paths: HashMap<String, PathBuf>,
    #[serde(default)]
    pub theme: Option<HashMap<String, String>>,
    #[serde(default)]
    pub keymap: HashMap<String, String>,
    #[serde(default)]
    pub proxy: Option<String>,
    /// Days of inactivity after which a skill is reported as stale.
    #[serde(default = "default_stale_after_days")]
    pub stale_after_days: u64,
    /// UI locale code (e.g. "en", "pt-BR"). Falls back to English when unset
    /// or unrecognized.
    #[serde(default)]
    pub locale: Option<String>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            custom_agent_paths: HashMap::new(),
            theme: None,
            keymap: HashMap::new(),
            proxy: None,
            stale_after_days: default_stale_after_days(),
            locale: None,
        }
    }
}

/// Default stale threshold: 30 days without observed usage.
pub fn default_stale_after_days() -> u64 {
    30
}

pub trait ConfigStore {
    fn read(&self) -> Result<TuiConfig, Box<dyn std::error::Error>>;
    fn write(&self, config: &TuiConfig) -> Result<(), Box<dyn std::error::Error>>;
}

pub const CONFIG_DIR: &str = "ai-skill";
pub const CONFIG_FILE: &str = "config.json";
