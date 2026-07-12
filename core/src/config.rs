use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TuiConfig {
    #[serde(default)]
    pub custom_agent_paths: HashMap<String, PathBuf>,
    #[serde(default)]
    pub theme: Option<HashMap<String, String>>,
    #[serde(default)]
    pub keymap: HashMap<String, String>,
    #[serde(default)]
    pub proxy: Option<String>,
}

pub trait ConfigStore {
    fn read(&self) -> Result<TuiConfig, Box<dyn std::error::Error>>;
    fn write(&self, config: &TuiConfig) -> Result<(), Box<dyn std::error::Error>>;
}

pub const CONFIG_DIR: &str = "ai-skill";
pub const CONFIG_FILE: &str = "config.json";
