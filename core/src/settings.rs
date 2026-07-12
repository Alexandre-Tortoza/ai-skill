use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectSettings {
    pub auto_trigger: bool,
    pub skill_overrides: Vec<SkillOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillOverride {
    pub skill_name: String,
    pub auto_trigger: bool,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_trigger: true,
            skill_overrides: vec![],
        }
    }
}

pub trait SettingsStore {
    fn read(&self) -> Result<ProjectSettings, Box<dyn std::error::Error>>;
    fn write(&self, settings: &ProjectSettings) -> Result<(), Box<dyn std::error::Error>>;
}
