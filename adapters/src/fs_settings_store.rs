use std::collections::HashMap;
use std::path::PathBuf;

use ai_skill_core::{ProjectSettings, SettingsStore, SkillOverride};
use serde::{Deserialize, Serialize};

fn project_settings_path() -> Result<PathBuf, std::io::Error> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(".claude").join("settings.json"))
}

fn home_settings_path() -> Result<PathBuf, std::io::Error> {
    let home = std::env::var_os("HOME").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME is not set; set HOME or run ai-skill from a login shell",
        )
    })?;
    Ok(PathBuf::from(home).join(".claude").join("settings.json"))
}

#[derive(Serialize, Deserialize)]
struct ClaudeSettings {
    #[serde(default)]
    skills: Option<ClaudeSkillsSection>,
}

#[derive(Serialize, Deserialize, Default)]
struct ClaudeSkillsSection {
    #[serde(default = "default_true", rename = "autoTrigger")]
    auto_trigger: bool,
    #[serde(default, rename = "skillOverrides")]
    skill_overrides: HashMap<String, ClaudeSkillOverride>,
}

#[derive(Serialize, Deserialize)]
struct ClaudeSkillOverride {
    #[serde(default = "default_true", rename = "autoTrigger")]
    auto_trigger: bool,
}

fn default_true() -> bool {
    true
}

impl From<ClaudeSettings> for ProjectSettings {
    fn from(cs: ClaudeSettings) -> Self {
        let section = cs.skills.unwrap_or_default();
        ProjectSettings {
            auto_trigger: section.auto_trigger,
            skill_overrides: section
                .skill_overrides
                .into_iter()
                .map(|(name, override_)| SkillOverride {
                    skill_name: name,
                    auto_trigger: override_.auto_trigger,
                })
                .collect(),
        }
    }
}

impl From<ProjectSettings> for ClaudeSettings {
    fn from(ps: ProjectSettings) -> Self {
        let mut map = HashMap::new();
        for s in ps.skill_overrides {
            map.insert(
                s.skill_name,
                ClaudeSkillOverride {
                    auto_trigger: s.auto_trigger,
                },
            );
        }
        ClaudeSettings {
            skills: Some(ClaudeSkillsSection {
                auto_trigger: ps.auto_trigger,
                skill_overrides: map,
            }),
        }
    }
}

pub struct FsSettingsStore {
    path: PathBuf,
}

impl FsSettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn from_env() -> Result<Self, std::io::Error> {
        let p = project_settings_path()?
            .canonicalize()
            .unwrap_or_else(|_| project_settings_path().unwrap());
        Ok(Self { path: p })
    }

    pub fn from_home() -> Result<Self, std::io::Error> {
        Ok(Self {
            path: home_settings_path()?,
        })
    }

    pub fn project_path() -> Result<PathBuf, std::io::Error> {
        project_settings_path()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl SettingsStore for FsSettingsStore {
    fn read(&self) -> Result<ProjectSettings, Box<dyn std::error::Error>> {
        if !self.path.exists() {
            return Ok(ProjectSettings::default());
        }
        let content = std::fs::read_to_string(&self.path)?;
        let claude: ClaudeSettings = serde_json::from_str(&content)?;
        Ok(claude.into())
    }

    fn write(&self, settings: &ProjectSettings) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let claude: ClaudeSettings = settings.clone().into();
        let content = serde_json::to_string_pretty(&claude)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, FsSettingsStore) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".claude").join("settings.json");
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        (dir, FsSettingsStore::new(path))
    }

    #[test]
    fn read_nonexistent_returns_defaults() {
        let (_dir, store) = make_store();
        let settings = store.read().unwrap();
        assert!(settings.auto_trigger);
        assert!(settings.skill_overrides.is_empty());
    }

    #[test]
    fn write_then_read_roundtrip() {
        let (_dir, store) = make_store();
        let settings = ProjectSettings {
            auto_trigger: false,
            skill_overrides: vec![SkillOverride {
                skill_name: "my-skill".into(),
                auto_trigger: false,
            }],
        };
        store.write(&settings).unwrap();
        let read_back = store.read().unwrap();
        assert!(!read_back.auto_trigger);
        assert_eq!(read_back.skill_overrides.len(), 1);
        assert_eq!(read_back.skill_overrides[0].skill_name, "my-skill");
        assert!(!read_back.skill_overrides[0].auto_trigger);
    }

    #[test]
    fn write_creates_parent_dir() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("sub").join("settings.json");
        let store = FsSettingsStore::new(path.clone());
        let settings = ProjectSettings::default();
        store.write(&settings).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn read_existing_json_parses_correctly() {
        let (_dir, store) = make_store();
        let json =
            r#"{"skills":{"autoTrigger":false,"skillOverrides":{"alpha":{"autoTrigger":true}}}}"#;
        std::fs::write(store.path(), json).unwrap();
        let settings = store.read().unwrap();
        assert!(!settings.auto_trigger);
        assert_eq!(settings.skill_overrides.len(), 1);
        assert_eq!(settings.skill_overrides[0].skill_name, "alpha");
        assert!(settings.skill_overrides[0].auto_trigger);
    }

    #[test]
    fn read_file_with_extra_fields_ignores_them() {
        let (_dir, store) = make_store();
        let json = r#"{"skills":{"autoTrigger":true,"skillOverrides":{}},"extraField":123}"#;
        std::fs::write(store.path(), json).unwrap();
        let settings = store.read().unwrap();
        assert!(settings.auto_trigger);
    }

    #[test]
    fn roundtrip_preserves_all_overrides() {
        let (_dir, store) = make_store();
        let original = ProjectSettings {
            auto_trigger: false,
            skill_overrides: vec![
                SkillOverride {
                    skill_name: "a".into(),
                    auto_trigger: false,
                },
                SkillOverride {
                    skill_name: "b".into(),
                    auto_trigger: true,
                },
            ],
        };
        store.write(&original).unwrap();
        let read = store.read().unwrap();
        assert_eq!(read.auto_trigger, original.auto_trigger);
        assert_eq!(read.skill_overrides.len(), original.skill_overrides.len());
        assert_eq!(read.skill_overrides[0].skill_name, "a");
        assert!(read.skill_overrides[1].auto_trigger);
    }

    #[test]
    fn write_and_read_produces_json_with_skills_key() {
        let (dir, store) = make_store();
        store
            .write(&ProjectSettings {
                auto_trigger: true,
                skill_overrides: vec![],
            })
            .unwrap();
        let content = std::fs::read_to_string(store.path()).unwrap();
        assert!(content.contains("skills"));
        assert!(content.contains("autoTrigger"));
        let _ = dir;
    }

    #[test]
    fn default_settings_have_auto_trigger_enabled() {
        let default = ProjectSettings::default();
        assert!(default.auto_trigger);
        assert!(default.skill_overrides.is_empty());
    }
}
