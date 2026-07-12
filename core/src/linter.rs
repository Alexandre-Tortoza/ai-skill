//! Linter for skill descriptions and wizard input validation.

use crate::Skill;

/// Severity level of a lint warning.
#[derive(Debug, Clone, PartialEq)]
pub enum LintLevel {
    Warning,
    Error,
}

/// A single lint warning attached to a field.
#[derive(Debug, Clone, PartialEq)]
pub struct LintWarning {
    pub level: LintLevel,
    pub field: &'static str,
    pub message: String,
}

/// Runs linter checks on the description (body) and name of a skill manifest.
///
/// `body` is the markdown content after the frontmatter block.
/// `current_name` is the skill name being edited.
/// `all_skills` is the full list of installed skills (used for collision detection).
/// `original_name` is the skill's name before editing (`None` for new skills).
pub fn lint_description(
    body: &str,
    current_name: &str,
    all_skills: &[Skill],
    original_name: Option<&str>,
) -> Vec<LintWarning> {
    let mut warnings: Vec<LintWarning> = Vec::new();

    // Check #1: multiple "and" conjunctions (more than 2 suggests poor writing)
    let and_count = body
        .to_lowercase()
        .split_whitespace()
        .filter(|&w| w == "and")
        .count();
    if and_count > 2 {
        warnings.push(LintWarning {
            level: LintLevel::Warning,
            field: "body",
            message: format!(
                "Description uses 'and' {and_count} times; consider rewriting for clarity (triggers false positives)."
            ),
        });
    }

    // Check #2: missing "Use when [context]. [what it does]." pattern
    let lower = body.to_lowercase();
    if !lower.contains("use when") {
        warnings.push(LintWarning {
            level: LintLevel::Warning,
            field: "body",
            message: "Description should start with 'Use when [context]. [what it does].' to help the agent decide when to trigger this skill."
                .to_string(),
        });
    }

    // Check #3: name collision with another skill (case-insensitive).
    // Exclude skills whose name matches original_name (the skill being edited).
    let collides = all_skills
        .iter()
        .filter(|s| original_name.is_none_or(|orig| s.name.to_lowercase() != orig.to_lowercase()))
        .any(|s| s.name.to_lowercase() == current_name.to_lowercase());
    if collides {
        let conflicting: Vec<&str> = all_skills
            .iter()
            .filter(|s| {
                s.name.to_lowercase() == current_name.to_lowercase()
                    && original_name.is_none_or(|orig| s.name.to_lowercase() != orig.to_lowercase())
            })
            .map(|s| s.name.as_str())
            .collect();
        let detail = if conflicting.is_empty() {
            "another installed skill".to_string()
        } else {
            format!("'{}'", conflicting.join("', '"))
        };
        warnings.push(LintWarning {
            level: LintLevel::Error,
            field: "name",
            message: format!("Name collides with {detail} (case-insensitive match)."),
        });
    }

    warnings
}

/// Runs lint checks on the full manifest content (frontmatter + body).
///
/// `original_name` is the skill's name before editing (`None` for new skills).
pub fn lint_content(
    content: &str,
    all_skills: &[Skill],
    current_name: &str,
    original_name: Option<&str>,
) -> Vec<LintWarning> {
    let body = crate::extract_body(content).unwrap_or("");
    lint_description(body, current_name, all_skills, original_name)
}

/// Validates wizard input before creating a new skill.
///
/// Returns a list of error messages. Empty list means input is valid.
pub fn validate_wizard_input(name: &str, agents: &[String], all_skills: &[Skill]) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    let name = name.trim();
    if name.is_empty() {
        errors.push("Skill name cannot be empty.".to_string());
    }

    if name.contains('/') || name.contains('\0') || name.contains('\\') {
        errors.push(
            "Skill name contains invalid characters ('/', '\\\\', or null byte).".to_string(),
        );
    }

    if agents.is_empty() {
        errors.push("At least one agent must be specified.".to_string());
    }

    // Check name collision with any existing skill
    let collides: Vec<&str> = all_skills
        .iter()
        .filter(|s| s.name.to_lowercase() == name.to_lowercase())
        .map(|s| s.name.as_str())
        .collect();
    if !collides.is_empty() {
        errors.push(format!(
            "A skill named '{}' already exists (case-insensitive match).",
            collides[0]
        ));
    }

    errors
}

/// Validates that a skill name fits basic filesystem-safe conventions.
pub fn validate_name(name: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let name = name.trim();
    if name.is_empty() {
        errors.push("Skill name cannot be empty.".to_string());
    }
    if name.contains('/') || name.contains('\0') || name.contains('\\') {
        errors.push(
            "Skill name contains invalid characters ('/', '\\\\', or null byte).".to_string(),
        );
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DriftState, Scope, SkillMode, ValidationState};
    use std::path::PathBuf;

    fn skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from("/tmp").join(name),
            scope: Scope::Global,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: true,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: DriftState::default(),
        }
    }

    // --- lint_description ---

    #[test]
    fn no_warnings_for_good_description() {
        let body = "Use when setting up a new Rust project. Scaffolds the project structure and configures the toolchain.";
        let warnings = lint_description(body, "my-skill", &[], None);
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn warns_on_multiple_and() {
        let body = "Do this and that and the other and also something and another thing.";
        let warnings = lint_description(body, "s", &[], None);
        assert!(warnings.iter().any(|w| w.message.contains("'and'")));
    }

    #[test]
    fn warns_on_missing_use_when() {
        let body = "This skill does something useful.";
        let warnings = lint_description(body, "s", &[], None);
        assert!(warnings.iter().any(|w| w.message.contains("Use when")));
    }

    #[test]
    fn accepts_use_when_without_exact_case() {
        let body = "use when in a Rust project. Sets up tooling.";
        let warnings = lint_description(body, "s", &[], None);
        assert!(!warnings.iter().any(|w| w.message.contains("Use when")));
    }

    #[test]
    fn warns_on_name_collision() {
        let all = vec![skill("my-skill")];
        let warnings = lint_description("Use when testing. Runs tests.", "My-Skill", &all, None);
        assert!(warnings.iter().any(|w| w.field == "name"));
    }

    #[test]
    fn warns_on_name_collision_with_original() {
        let all = vec![skill("my-skill"), skill("other")];
        // Editing "other", changing name to "my-skill" — collision with existing "my-skill"
        let warnings = lint_description(
            "Use when testing. Runs tests.",
            "my-skill",
            &all,
            Some("other"),
        );
        assert!(warnings.iter().any(|w| w.field == "name"));
    }

    #[test]
    fn no_collision_warning_for_same_name_with_original() {
        let all = vec![skill("my-skill")];
        // Editing "my-skill", keeping same name — no collision
        let warnings = lint_description(
            "Use when testing. Runs tests.",
            "my-skill",
            &all,
            Some("my-skill"),
        );
        assert!(!warnings.iter().any(|w| w.field == "name"));
    }

    #[test]
    fn no_collision_warning_for_case_change() {
        let all = vec![skill("my-skill")];
        // Editing "my-skill", changing case to "My-Skill" — same skill, no collision
        let warnings = lint_description(
            "Use when testing. Runs tests.",
            "My-Skill",
            &all,
            Some("my-skill"),
        );
        assert!(!warnings.iter().any(|w| w.field == "name"));
    }

    #[test]
    fn empty_body_produces_warnings() {
        let warnings = lint_description("", "s", &[], None);
        assert!(warnings.iter().any(|w| w.message.contains("Use when")));
    }

    #[test]
    fn lint_content_parses_frontmatter_and_lints_body() {
        let content = "---\nname: my-skill\n---\n\nUse when building. Builds stuff.";
        let warnings = lint_content(content, &[], "my-skill", None);
        assert!(warnings.is_empty());
    }

    #[test]
    fn lint_content_no_body_still_runs_checks() {
        let content = "---\nname: s\n---\n";
        let warnings = lint_content(content, &[], "s", None);
        assert!(warnings.iter().any(|w| w.message.contains("Use when")));
    }

    // --- validate_wizard_input ---

    #[test]
    fn valid_wizard_input_passes() {
        let errors = validate_wizard_input("my-skill", &["claude".to_string()], &[]);
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn empty_name_is_rejected() {
        let errors = validate_wizard_input("", &["claude".to_string()], &[]);
        assert!(errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn whitespace_name_is_rejected() {
        let errors = validate_wizard_input("   ", &["claude".to_string()], &[]);
        assert!(errors.iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn name_with_slash_is_rejected() {
        let errors = validate_wizard_input("my/skill", &["claude".to_string()], &[]);
        assert!(errors.iter().any(|e| e.contains("invalid")));
    }

    #[test]
    fn no_agents_is_rejected() {
        let errors = validate_wizard_input("my-skill", &[], &[]);
        assert!(errors.iter().any(|e| e.contains("agent")));
    }

    #[test]
    fn name_collision_is_rejected() {
        let all = vec![skill("existing")];
        let errors = validate_wizard_input("Existing", &["claude".to_string()], &all);
        assert!(errors.iter().any(|e| e.contains("already exists")));
    }

    #[test]
    fn multiple_errors_at_once() {
        let errors = validate_wizard_input("", &[], &[]);
        assert!(errors.len() >= 2);
    }

    // --- validate_name ---

    #[test]
    fn validate_name_empty() {
        let errors = validate_name("");
        assert!(!errors.is_empty());
    }

    #[test]
    fn validate_name_valid() {
        let errors = validate_name("valid-name");
        assert!(errors.is_empty());
    }
}
