//! Frontmatter (`---`-delimited YAML) parsing and markdown body extraction.

use serde::Deserialize;
use thiserror::Error;

/// Errors that can occur when parsing skill frontmatter.
#[derive(Error, Debug)]
pub enum ParseError {
    /// The content does not contain valid `---\n...\n---` delimiters.
    #[error("missing frontmatter delimiters")]
    MissingDelimiters,
    /// The YAML block inside the delimiters could not be parsed.
    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_norway::Error),
}

/// Metadata extracted from a skill's frontmatter block.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillMetadata {
    /// Skill name.
    pub name: String,
    /// Agent identifiers this skill targets.
    pub agents: Vec<String>,
    /// Free-form tags.
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
struct SkillFrontmatter {
    name: String,
    #[serde(default)]
    agents: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// Returns the markdown body after the closing `---` delimiter, trimmed, or `None`.
pub fn extract_body(content: &str) -> Option<&str> {
    let after_open = content.strip_prefix("---\n")?;
    let end = after_open.find("\n---")?;
    let after_close = &after_open[end + 4..]; // skip `\n---`
    let body = after_close.trim_start_matches('\n').trim_end();
    if body.is_empty() { None } else { Some(body) }
}

/// Parses the frontmatter block and returns the metadata, or a [`ParseError`].
pub fn parse_frontmatter(content: &str) -> Result<SkillMetadata, ParseError> {
    let after_open = content
        .strip_prefix("---\n")
        .ok_or(ParseError::MissingDelimiters)?;

    let end = after_open
        .find("\n---")
        .ok_or(ParseError::MissingDelimiters)?;

    let yaml = &after_open[..end];
    let fm = serde_norway::from_str::<SkillFrontmatter>(yaml)?;
    Ok(SkillMetadata {
        name: fm.name,
        agents: fm.agents,
        tags: fm.tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn minimal_frontmatter_name_only() {
        let content = indoc! {"
            ---
            name: my-skill
            ---
        "};
        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, "my-skill");
        assert!(fm.agents.is_empty());
    }

    #[test]
    fn frontmatter_with_agents_list() {
        let content = indoc! {"
            ---
            name: omarchy
            agents:
              - claude
              - codex
            ---
        "};
        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, "omarchy");
        assert_eq!(fm.agents, vec!["claude", "codex"]);
    }

    #[test]
    fn missing_delimiters_returns_error() {
        let content = "name: my-skill\n";
        let err = parse_frontmatter(content).unwrap_err();
        assert!(matches!(err, ParseError::MissingDelimiters));
    }

    #[test]
    fn malformed_yaml_returns_yaml_error() {
        let content = indoc! {"
            ---
            name: [unclosed
            ---
        "};
        let err = parse_frontmatter(content).unwrap_err();
        assert!(matches!(err, ParseError::Yaml(_)));
    }

    #[test]
    fn body_after_closing_delimiter_is_ignored() {
        let content = indoc! {"
            ---
            name: my-skill
            ---
            # This is the body

            Some markdown content here.
        "};
        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, "my-skill");
    }

    #[test]
    fn frontmatter_with_tags_list() {
        let content = indoc! {"
            ---
            name: my-skill
            tags:
              - git
              - productivity
            ---
        "};
        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.tags, vec!["git", "productivity"]);
    }

    #[test]
    fn frontmatter_without_tags_defaults_to_empty() {
        let content = indoc! {"
            ---
            name: my-skill
            ---
        "};
        let fm = parse_frontmatter(content).unwrap();
        assert!(fm.tags.is_empty());
    }

    #[test]
    fn extract_body_returns_none_for_frontmatter_only() {
        let content = "---\nname: my-skill\n---\n";
        assert!(extract_body(content).is_none());
    }

    #[test]
    fn extract_body_returns_text_after_delimiter() {
        let content = indoc! {"
            ---
            name: my-skill
            ---
            # Heading

            Body text here.
        "};
        let body = extract_body(content).unwrap();
        assert!(body.contains("# Heading"));
        assert!(body.contains("Body text here."));
    }

    #[test]
    fn parse_frontmatter_empty_content_returns_missing_delimiters() {
        let err = parse_frontmatter("").unwrap_err();
        assert!(matches!(err, ParseError::MissingDelimiters));
    }

    #[test]
    fn extract_body_empty_content_returns_none() {
        assert!(extract_body("").is_none());
    }

    #[test]
    fn extract_body_only_opening_delimiter_returns_none() {
        assert!(extract_body("---\n").is_none());
    }

    #[test]
    fn parse_frontmatter_without_newline_after_opening_returns_missing() {
        let fm = parse_frontmatter("---\nname: x\n---").unwrap();
        assert_eq!(fm.name, "x");
    }

    #[test]
    fn parse_error_display_missing_delimiters() {
        let err = ParseError::MissingDelimiters;
        assert_eq!(err.to_string(), "missing frontmatter delimiters");
    }

    #[test]
    fn parse_error_debug() {
        let err = ParseError::MissingDelimiters;
        assert!(!format!("{err:?}").is_empty());
    }
}
