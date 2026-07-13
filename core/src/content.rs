//! Skill file-content models and the reader port.
//!
//! These types back the TUI split preview (a skill's README/SKILL.md) and the
//! skill directory explorer (which can descend into nested sub-skills, i.e.
//! directories that themselves contain a `SKILL.md`).

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Classification of a node in a skill's directory tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillFileKind {
    /// Markdown documentation (`*.md`).
    Markdown,
    /// Executable script (`*.py`, `*.js`, `*.ts`, `*.sh`, ...).
    Script,
    /// Structured configuration (`*.json`, `*.yaml`, `*.yml`, `*.toml`).
    Config,
    /// Any other file type.
    Other,
}

/// A single node in a skill's directory tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillTreeNode {
    /// File or directory name (not the full path).
    pub name: String,
    /// Absolute path to this node.
    pub path: PathBuf,
    /// True when this node is a directory.
    pub is_dir: bool,
    /// Best-effort classification of the node's content.
    pub kind: SkillFileKind,
    /// True when this node is a nested skill (a directory containing `SKILL.md`).
    pub is_subskill: bool,
    /// Nesting depth; the skill root's direct children are at depth 0.
    pub depth: usize,
}

/// A rendered preview document for a skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillDoc {
    /// Source file used for the preview (e.g. `README.md`).
    pub title: String,
    /// Document text, truncated if larger than the adapter's size cap.
    pub content: String,
}

/// Why skill content could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    /// The skill directory (or file) does not exist.
    NotFound,
    /// A filesystem operation failed.
    Io,
}

/// Port for reading a skill's on-disk file content.
pub trait SkillContentReader {
    /// Reads the most relevant preview doc for the skill directory.
    ///
    /// Resolution order: `README.md` → `readme.md` → `Readme.md` → `SKILL.md`.
    /// For `SKILL.md` the YAML frontmatter is stripped so only the body is shown.
    fn read_preview(&self, skill_dir: &Path) -> Result<SkillDoc, ContentError>;

    /// Reads a depth-first, directory-and-file listing of the skill tree.
    ///
    /// Directories are recursed into; entries such as `.git`, `target` and
    /// `.ai-skill` are skipped as noise.
    fn read_tree(&self, skill_dir: &Path) -> Result<Vec<SkillTreeNode>, ContentError>;

    /// Reads the raw text of a single file within the skill directory.
    fn read_file(&self, file_path: &Path) -> Result<String, ContentError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_kind_classifies_by_extension() {
        let md = SkillFileKind::Markdown;
        assert_eq!(md, SkillFileKind::Markdown);
        assert_ne!(md, SkillFileKind::Script);
    }

    #[test]
    fn node_fields_round_trip_construction() {
        let node = SkillTreeNode {
            name: "scripts".into(),
            path: PathBuf::from("/s/scripts"),
            is_dir: true,
            kind: SkillFileKind::Other,
            is_subskill: false,
            depth: 1,
        };
        assert!(node.is_dir);
        assert_eq!(node.depth, 1);
        assert!(!node.is_subskill);
    }

    #[test]
    fn doc_fields_round_trip() {
        let doc = SkillDoc {
            title: "README.md".into(),
            content: "hello".into(),
        };
        assert_eq!(doc.title, "README.md");
        assert_eq!(doc.content, "hello");
    }

    #[test]
    fn content_error_variants_are_distinct() {
        assert_ne!(ContentError::NotFound, ContentError::Io);
    }
}
