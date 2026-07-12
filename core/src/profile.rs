//! Profile model, diff algorithm, and persistence port.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{Skill, ValidationState};

/// Development phase that a preset profile targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    Init,
    Dev,
    Test,
    Release,
}

/// A named profile referencing a set of skills by name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// Display name (e.g. `dev`, `ops`).
    pub name: String,
    /// Skill names that this profile should install.
    pub skill_names: Vec<String>,
    /// Optional phase tag for preset profiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<Phase>,
}

/// A single operation required to reconcile current state with a desired profile.
#[derive(Debug, Clone, PartialEq)]
pub enum ProfileOp {
    Install { name: String },
    Remove { name: String },
}

/// Returns the minimal set of install/remove ops to reach `desired` from `current`.
///
/// Only skills with [`ValidationState::Valid`] count as "installed";
/// disabled, broken, or duplicate skills are treated as absent.
pub fn diff_profile(current: &[Skill], desired: &Profile) -> Vec<ProfileOp> {
    let current_names: HashSet<&str> = current
        .iter()
        .filter(|s| s.validation == ValidationState::Valid)
        .map(|s| s.name.as_str())
        .collect();

    let desired_names: HashSet<&str> = desired.skill_names.iter().map(String::as_str).collect();

    let mut ops = Vec::new();

    for name in &desired.skill_names {
        if !current_names.contains(name.as_str()) {
            ops.push(ProfileOp::Install { name: name.clone() });
        }
    }

    for skill in current {
        if skill.validation == ValidationState::Valid
            && !desired_names.contains(skill.name.as_str())
        {
            ops.push(ProfileOp::Remove {
                name: skill.name.clone(),
            });
        }
    }

    ops
}

/// Persistence port for named skill profiles.
pub trait ProfileStore {
    /// Returns all saved profiles.
    fn list(&self) -> Result<Vec<Profile>, Box<dyn std::error::Error>>;
    /// Persists or overwrites a profile.
    fn save(&self, profile: &Profile) -> Result<(), Box<dyn std::error::Error>>;
    /// Deletes a profile by name.
    fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Scope, SkillMode, ValidationState};
    use std::path::PathBuf;

    fn valid_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: vec![],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: crate::DriftState::default(),
        }
    }

    fn broken_skill(name: &str) -> Skill {
        Skill {
            validation: ValidationState::BrokenSymlink,
            ..valid_skill(name)
        }
    }

    fn profile(name: &str, skills: &[&str]) -> Profile {
        Profile {
            name: name.to_string(),
            skill_names: skills.iter().map(|s| s.to_string()).collect(),
            phase: None,
        }
    }

    #[test]
    fn profile_fields_are_accessible() {
        let p = profile("dev", &["alpha", "beta"]);
        assert_eq!(p.name, "dev");
        assert_eq!(p.skill_names, vec!["alpha", "beta"]);
    }

    #[test]
    fn diff_empty_desired_produces_remove_for_all_valid() {
        let current = vec![valid_skill("alpha"), valid_skill("beta")];
        let desired = profile("empty", &[]);
        let ops = diff_profile(&current, &desired);
        assert_eq!(ops.len(), 2);
        assert!(ops.contains(&ProfileOp::Remove {
            name: "alpha".into()
        }));
        assert!(ops.contains(&ProfileOp::Remove {
            name: "beta".into()
        }));
    }

    #[test]
    fn diff_new_skills_in_desired_produces_install() {
        let current: Vec<Skill> = vec![];
        let desired = profile("dev", &["alpha", "beta"]);
        let ops = diff_profile(&current, &desired);
        assert!(ops.contains(&ProfileOp::Install {
            name: "alpha".into()
        }));
        assert!(ops.contains(&ProfileOp::Install {
            name: "beta".into()
        }));
    }

    #[test]
    fn diff_identical_produces_empty() {
        let current = vec![valid_skill("alpha"), valid_skill("beta")];
        let desired = profile("same", &["alpha", "beta"]);
        assert!(diff_profile(&current, &desired).is_empty());
    }

    #[test]
    fn diff_mixed_installs_and_removes() {
        let current = vec![valid_skill("alpha"), valid_skill("old")];
        let desired = profile("new", &["alpha", "new-skill"]);
        let ops = diff_profile(&current, &desired);
        assert!(ops.contains(&ProfileOp::Install {
            name: "new-skill".into()
        }));
        assert!(ops.contains(&ProfileOp::Remove { name: "old".into() }));
        assert!(!ops.iter().any(|op| matches!(op, ProfileOp::Install { name } | ProfileOp::Remove { name } if name == "alpha")));
    }

    #[test]
    fn diff_ignores_non_valid_skills() {
        let current = vec![valid_skill("alpha"), broken_skill("broken")];
        let desired = profile("p", &["alpha"]);
        // "broken" is not Valid so it should not trigger a Remove
        let ops = diff_profile(&current, &desired);
        assert!(ops.is_empty());
    }

    #[test]
    fn profile_store_is_object_safe() {
        struct FakeStore;
        impl ProfileStore for FakeStore {
            fn list(&self) -> Result<Vec<Profile>, Box<dyn std::error::Error>> {
                Ok(vec![])
            }
            fn save(&self, _p: &Profile) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
            fn delete(&self, _n: &str) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }
        let store: Box<dyn ProfileStore> = Box::new(FakeStore);
        assert!(store.list().unwrap().is_empty());
    }
}
