//! Core domain model: [`Scope`], [`Agent`] and [`Skill`].

use std::path::{Path, PathBuf};

use crate::{DriftState, SkillMode, ValidationState};
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

/// Supported AI agents that can host skills.
#[derive(Debug, Clone, PartialEq)]
pub enum Agent {
    ClaudeCode,
    Cursor,
    Windsurf,
    GitHubCopilot,
    Codex,
    GeminiCli,
    OpenCode,
}

impl Agent {
    /// Human-readable label for this agent.
    pub fn label(&self) -> &'static str {
        match self {
            Agent::ClaudeCode => "Claude Code",
            Agent::Cursor => "Cursor",
            Agent::Windsurf => "Windsurf",
            Agent::GitHubCopilot => "GitHub Copilot",
            Agent::Codex => "Codex",
            Agent::GeminiCli => "Gemini CLI",
            Agent::OpenCode => "OpenCode",
        }
    }

    /// Subdirectory name under `$HOME` where this agent keeps its skills.
    fn home_subdir(&self) -> Option<&'static str> {
        match self {
            Agent::ClaudeCode => Some(".claude/skills"),
            Agent::Cursor => Some(".cursor/skills"),
            Agent::Windsurf => Some(".windsurf/skills"),
            Agent::Codex => Some(".codex/skills"),
            Agent::GeminiCli => Some(".gemini-cli/skills"),
            // Agents without a known file-based skill directory.
            Agent::GitHubCopilot => None,
            Agent::OpenCode => None,
        }
    }

    /// Returns the skill directory path for this agent under the given home dir.
    pub fn home_skills_dir(&self, home: &Path) -> Option<PathBuf> {
        self.home_subdir().map(|sub| home.join(sub))
    }

    /// Returns all known agents that use Claude-compatible skill format.
    pub fn claude_compatible() -> Vec<Agent> {
        vec![
            Agent::ClaudeCode,
            Agent::Cursor,
            Agent::Windsurf,
            Agent::Codex,
            Agent::GeminiCli,
        ]
    }
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
    /// User-chosen operating mode (active / name-only / disabled).
    pub mode: SkillMode,
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

    #[test]
    fn agent_variants_have_labels() {
        assert_eq!(Agent::ClaudeCode.label(), "Claude Code");
        assert_eq!(Agent::Cursor.label(), "Cursor");
        assert_eq!(Agent::Windsurf.label(), "Windsurf");
    }

    #[test]
    fn claude_code_has_skills_dir() {
        let home = Path::new("/home/user");
        assert_eq!(
            Agent::ClaudeCode.home_skills_dir(home),
            Some(PathBuf::from("/home/user/.claude/skills"))
        );
    }

    #[test]
    fn copilot_has_no_skills_dir() {
        let home = Path::new("/home/user");
        assert_eq!(Agent::GitHubCopilot.home_skills_dir(home), None);
    }

    #[test]
    fn claude_compatible_includes_cursor() {
        let agents = Agent::claude_compatible();
        assert!(agents.contains(&Agent::Cursor));
        assert!(!agents.contains(&Agent::GitHubCopilot));
    }

    #[test]
    fn agents_are_distinct() {
        assert_ne!(Agent::ClaudeCode, Agent::Cursor);
        assert_ne!(Agent::Cursor, Agent::Windsurf);
    }

    fn make_skill(name: &str, scope: Scope) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from("/tmp").join(name),
            scope,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
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
