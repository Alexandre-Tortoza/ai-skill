//! Ports and functions for creating and editing skill manifests.

use std::path::{Path, PathBuf};

use crate::frontmatter::extract_body;

/// Port for creating a new skill directory with a scaffolded manifest.
pub trait SkillCreator {
    /// Creates a skill directory and returns its path.
    fn create(
        &self,
        name: &str,
        agents: &[String],
        tags: &[String],
    ) -> Result<PathBuf, Box<dyn std::error::Error>>;
}

/// Port for writing content to a skill manifest file.
pub trait SkillWriter {
    /// Writes content to the given path.
    fn write(&self, path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// Generates a SKILL.md scaffold with frontmatter metadata and a body template.
pub fn scaffold_skill(name: &str, agents: &[String], tags: &[String]) -> String {
    let agents_yaml = if agents.is_empty() {
        String::new()
    } else {
        let list = agents
            .iter()
            .map(|a| format!("  - {a}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("agents:\n{list}\n")
    };

    let tags_yaml = if tags.is_empty() {
        String::new()
    } else {
        let list = tags
            .iter()
            .map(|t| format!("  - {t}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("tags:\n{list}\n")
    };

    format!(
        "---\nname: {name}\n{agents_yaml}{tags_yaml}---\n\n# {name}\n\nDescribe what this skill does.\n"
    )
}

/// Re-serialises a SKILL.md with updated frontmatter, preserving the original body.
///
/// If frontmatter cannot be parsed, a new frontmatter-only document is returned.
pub fn apply_edit(
    original_content: &str,
    name: &str,
    agents: &[String],
    tags: &[String],
) -> String {
    let body = extract_body(original_content).unwrap_or("");
    let agents_yaml = if agents.is_empty() {
        String::new()
    } else {
        let list = agents
            .iter()
            .map(|a| format!("  - {a}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("agents:\n{list}\n")
    };

    let tags_yaml = if tags.is_empty() {
        String::new()
    } else {
        let list = tags
            .iter()
            .map(|t| format!("  - {t}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("tags:\n{list}\n")
    };

    if body.is_empty() {
        format!("---\nname: {name}\n{agents_yaml}{tags_yaml}---\n")
    } else {
        format!("---\nname: {name}\n{agents_yaml}{tags_yaml}---\n\n{body}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_contains_name_in_frontmatter() {
        let out = scaffold_skill("my-skill", &[], &[]);
        assert!(out.contains("name: my-skill"));
        assert!(out.starts_with("---\n"));
    }

    #[test]
    fn scaffold_with_agents_includes_them() {
        let agents = vec!["claude".to_string(), "codex".to_string()];
        let out = scaffold_skill("s", &agents, &[]);
        assert!(out.contains("agents:"));
        assert!(out.contains("  - claude"));
        assert!(out.contains("  - codex"));
    }

    #[test]
    fn scaffold_with_tags_includes_them() {
        let tags = vec!["git".to_string(), "productivity".to_string()];
        let out = scaffold_skill("s", &[], &tags);
        assert!(out.contains("tags:"));
        assert!(out.contains("  - git"));
    }

    #[test]
    fn scaffold_body_template_not_empty() {
        let out = scaffold_skill("my-skill", &[], &[]);
        assert!(out.contains("# my-skill"));
    }

    #[test]
    fn scaffold_without_agents_omits_agents_key() {
        let out = scaffold_skill("s", &[], &[]);
        assert!(!out.contains("agents:"));
    }

    #[test]
    fn apply_edit_updates_name() {
        let original = "---\nname: old\n---\n\nBody here.\n";
        let result = apply_edit(original, "new-name", &[], &[]);
        assert!(result.contains("name: new-name"));
        assert!(!result.contains("name: old"));
    }

    #[test]
    fn apply_edit_preserves_body() {
        let original = "---\nname: old\n---\n\nOriginal body content.\n";
        let result = apply_edit(original, "new", &[], &[]);
        assert!(result.contains("Original body content."));
    }

    #[test]
    fn apply_edit_updates_agents() {
        let original = "---\nname: s\nagents:\n  - claude\n---\n\nBody.\n";
        let new_agents = vec!["codex".to_string()];
        let result = apply_edit(original, "s", &new_agents, &[]);
        assert!(result.contains("  - codex"));
        assert!(!result.contains("  - claude"));
    }

    #[test]
    fn apply_edit_updates_tags() {
        let original = "---\nname: s\n---\n\nBody.\n";
        let new_tags = vec!["rust".to_string()];
        let result = apply_edit(original, "s", &[], &new_tags);
        assert!(result.contains("tags:"));
        assert!(result.contains("  - rust"));
    }

    #[test]
    fn apply_edit_no_body_produces_valid_frontmatter_only() {
        let original = "---\nname: s\n---\n";
        let result = apply_edit(original, "new", &[], &[]);
        assert!(result.contains("name: new"));
        assert!(result.ends_with("---\n"));
    }

    #[test]
    fn apply_edit_malformed_returns_original() {
        let original = "no frontmatter here";
        let result = apply_edit(original, "new", &[], &[]);
        assert_eq!(result, "---\nname: new\n---\n");
        // extract_body returns None, so body is empty, so we get a new frontmatter-only doc
    }

    #[test]
    fn skill_creator_trait_object_compiles() {
        struct FakeCreator;
        impl SkillCreator for FakeCreator {
            fn create(
                &self,
                name: &str,
                _a: &[String],
                _t: &[String],
            ) -> Result<PathBuf, Box<dyn std::error::Error>> {
                Ok(PathBuf::from(name))
            }
        }
        let c: Box<dyn SkillCreator> = Box::new(FakeCreator);
        assert!(c.create("test", &[], &[]).is_ok());
    }

    #[test]
    fn skill_writer_trait_object_compiles() {
        struct FakeWriter;
        impl SkillWriter for FakeWriter {
            fn write(&self, _p: &Path, _c: &str) -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }
        let w: Box<dyn SkillWriter> = Box::new(FakeWriter);
        assert!(w.write(Path::new("/tmp/test"), "content").is_ok());
    }
}
