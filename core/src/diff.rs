//! Skill upstream diff model and reader port.
//!
//! Complements [`crate::drift`] by exposing the actual content changes when a
//! skill has an upstream [`DriftState::UpdateAvailable`].

use serde::Serialize;
use std::path::Path;

/// Classification of a single diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    /// Unchanged context line.
    Context,
    /// Added line (prefixed `+` in the raw diff).
    Add,
    /// Removed line (prefixed `-` in the raw diff).
    Remove,
    /// Hunk/header metadata line (`diff --git`, `index`, `@@`, `---`, `+++`).
    Header,
}

/// A single parsed diff line, kept without its leading marker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffLine {
    /// Semantic kind used for coloring.
    pub kind: DiffLineKind,
    /// Line text (marker stripped).
    pub text: String,
}

/// A parsed upstream diff for a single skill's manifest.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct SkillDiff {
    /// Parsed lines in file order.
    pub lines: Vec<DiffLine>,
}

impl SkillDiff {
    /// Returns true when there is no content to show.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Why an upstream diff could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    /// The skill directory is not inside a Git repository.
    NoGitRepo,
    /// The skill has no configured upstream remote.
    NoUpstream,
    /// The diff command could not run or exited non-zero for another reason.
    CommandFailed,
}

/// Port for reading the upstream diff of a skill's manifest.
pub trait SkillDiffReader {
    /// Returns the upstream diff for the skill at `path`.
    fn read_diff(&self, path: &Path) -> Result<SkillDiff, DiffError>;
}

/// Parses a unified-diff string into a [`SkillDiff`], classifying each line.
///
/// Header lines (`diff --git`, `index`, `---`, `+++`, `@@`) are tagged
/// [`DiffLineKind::Header`]; `+`/`-` prefixed lines become `Add`/`Remove`
/// (marker stripped); everything else is `Context`.
pub fn parse_diff(raw: &str) -> SkillDiff {
    let lines = raw
        .lines()
        .map(|line| {
            if line.starts_with("diff --git")
                || line.starts_with("index ")
                || line.starts_with("@@")
                || line.starts_with("--- ")
                || line.starts_with("+++ ")
            {
                DiffLine {
                    kind: DiffLineKind::Header,
                    text: line.to_string(),
                }
            } else if let Some(rest) = line.strip_prefix('+') {
                DiffLine {
                    kind: DiffLineKind::Add,
                    text: rest.to_string(),
                }
            } else if let Some(rest) = line.strip_prefix('-') {
                DiffLine {
                    kind: DiffLineKind::Remove,
                    text: rest.to_string(),
                }
            } else if let Some(rest) = line.strip_prefix(' ') {
                DiffLine {
                    kind: DiffLineKind::Context,
                    text: rest.to_string(),
                }
            } else {
                DiffLine {
                    kind: DiffLineKind::Context,
                    text: line.to_string(),
                }
            }
        })
        .collect();
    SkillDiff { lines }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
diff --git a/SKILL.md b/SKILL.md
index 1111111..2222222 100644
--- a/SKILL.md
+++ b/SKILL.md
@@ -1,3 +1,3 @@
 name: demo
-description: old
+description: new
 context line
";

    #[test]
    fn parses_headers_adds_and_removes() {
        let diff = parse_diff(SAMPLE);
        assert_eq!(diff.lines.len(), 9);
        assert!(matches!(diff.lines[0].kind, DiffLineKind::Header));
        assert_eq!(diff.lines[0].text, "diff --git a/SKILL.md b/SKILL.md");
        assert!(matches!(diff.lines[2].kind, DiffLineKind::Header));
        assert_eq!(diff.lines[2].text, "--- a/SKILL.md");
        assert!(matches!(diff.lines[3].kind, DiffLineKind::Header));
        assert_eq!(diff.lines[3].text, "+++ b/SKILL.md");
        assert!(matches!(diff.lines[4].kind, DiffLineKind::Header));
        assert_eq!(diff.lines[4].text, "@@ -1,3 +1,3 @@");
        let remove = &diff.lines[6];
        assert_eq!(remove.kind, DiffLineKind::Remove);
        assert_eq!(remove.text, "description: old");
        let add = &diff.lines[7];
        assert_eq!(add.kind, DiffLineKind::Add);
        assert_eq!(add.text, "description: new");
        assert_eq!(diff.lines[8].kind, DiffLineKind::Context);
        assert_eq!(diff.lines[8].text, "context line");
    }

    #[test]
    fn empty_input_yields_empty_diff() {
        assert!(parse_diff("").is_empty());
    }

    #[test]
    fn diff_error_variants_are_distinct() {
        assert_ne!(DiffError::NoGitRepo, DiffError::NoUpstream);
        assert_ne!(DiffError::NoUpstream, DiffError::CommandFailed);
    }

    #[test]
    fn header_prefixes_are_classified() {
        let diff = parse_diff("index abc..def 100644\n+++ b/SKILL.md\n");
        assert_eq!(diff.lines[0].kind, DiffLineKind::Header);
        assert_eq!(diff.lines[1].kind, DiffLineKind::Header);
    }
}
