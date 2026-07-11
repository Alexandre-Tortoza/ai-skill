//! Core domain model: [`Scope`] and [`Skill`].

use std::path::PathBuf;

use crate::{DriftState, ValidationState};
use serde::Serialize;

/// Whether a skill is installed globally or scoped to a project.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    /// Available to every project on the system.
    Global,
    /// Scoped to a single project directory.
    Project,
}

/// A single installed skill with its metadata, validation state, and drift info.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Skill {
    /// Display / unique name.
    pub name: String,
    /// Filesystem path to the skill directory or manifest.
    pub path: PathBuf,
    /// Installation scope.
    pub scope: Scope,
    /// Agent identifiers this skill is meant for (e.g. `claude`, `codex`).
    pub agents: Vec<String>,
    /// Free-form tags for categorisation.
    pub tags: Vec<String>,
    /// Whether ai-skill considers this skill under its management.
    pub managed: bool,
    /// Current health state.
    pub validation: ValidationState,
    /// Raw SKILL.md content, if loaded.
    pub manifest_content: Option<String>,
    /// Drift state relative to upstream.
    pub drift_state: DriftState,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, scope: Scope) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from("/tmp").join(name),
            scope,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: DriftState::default(),
        }
    }

    #[test]
    fn skill_fields_are_accessible() {
        let skill = make_skill("my-skill", Scope::Global);
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.path, PathBuf::from("/tmp/my-skill"));
        assert_eq!(skill.scope, Scope::Global);
        assert_eq!(skill.agents, vec!["claude"]);
        assert_eq!(skill.validation, ValidationState::Valid);
        assert!(skill.manifest_content.is_none());
    }

    #[test]
    fn scopes_are_not_equal() {
        assert_ne!(Scope::Global, Scope::Project);
    }

    #[test]
    fn clone_produces_independent_value() {
        let original = make_skill("alpha", Scope::Project);
        let mut cloned = original.clone();
        cloned.name = "beta".to_string();
        assert_eq!(original.name, "alpha");
    }

    #[test]
    fn tags_field_is_accessible_and_defaults_empty() {
        let skill = make_skill("t", Scope::Global);
        assert!(skill.tags.is_empty());
    }

    #[test]
    fn managed_field_is_accessible() {
        let skill = make_skill("t", Scope::Global);
        assert!(!skill.managed);
        let managed = Skill {
            managed: true,
            ..make_skill("m", Scope::Global)
        };
        assert!(managed.managed);
    }

    #[test]
    fn skill_with_manifest_content() {
        let mut skill = make_skill("with-body", Scope::Global);
        skill.manifest_content = Some("# My Skill\nDoes cool things.".to_string());
        assert!(skill.manifest_content.is_some());
    }
}
