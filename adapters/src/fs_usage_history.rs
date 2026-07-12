//! Filesystem [`SkillUsageReader`] that scans local agent history.
//!
//! The first supported source is Claude Code's transcript history, stored as
//! newline-delimited JSON (`.jsonl`) under `~/.claude/projects/`. Each skill
//! invocation is emitted in prose as `Skill(\`name\`)`; we extract those names
//! and use the transcript file's modification time as the observed usage time.
//!
//! The reader is intentionally heuristic: unreadable files are skipped and
//! missing history directories yield no events rather than an error.

use ai_skill_core::{SkillUsageEvent, SkillUsageReader, UsageError};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Errors surfaced by [`FsUsageHistoryReader`].
#[derive(Debug, thiserror::Error)]
pub enum UsageHistoryError {
    /// A filesystem operation failed unexpectedly.
    #[error("usage history error: {0}")]
    Io(#[from] std::io::Error),
}

/// Scans local agent history directories for skill usage events.
pub struct FsUsageHistoryReader {
    roots: Vec<PathBuf>,
}

impl FsUsageHistoryReader {
    /// Creates a reader that scans the given history roots.
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }

    /// Resolves the default history roots from `$HOME` (Claude Code projects).
    pub fn from_env() -> Self {
        let roots = std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| vec![home.join(".claude").join("projects")])
            .unwrap_or_default();
        Self::new(roots)
    }

    /// Recursively collects `.jsonl` files under `root`.
    fn collect_jsonl(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if !root.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_jsonl(&path, out)?;
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("jsonl"))
                .unwrap_or(false)
            {
                out.push(path);
            }
        }
        Ok(())
    }
}

impl SkillUsageReader for FsUsageHistoryReader {
    fn read_events(&self) -> Result<Vec<SkillUsageEvent>, UsageError> {
        let mut files = Vec::new();
        for root in &self.roots {
            Self::collect_jsonl(root, &mut files)?;
        }

        let mut events = Vec::new();
        for file in files {
            // Use the transcript's modification time as the usage timestamp.
            let mtime = std::fs::metadata(&file)?
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH);

            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut seen = std::collections::HashSet::new();
            for line in content.lines() {
                for name in extract_skill_names(line) {
                    seen.insert(name);
                }
            }
            for name in seen {
                events.push(SkillUsageEvent {
                    skill_name: name,
                    timestamp: mtime,
                });
            }
        }

        Ok(events)
    }
}

/// Extracts skill names referenced as `Skill(\`name\`)` or `Skill(name)` in a
/// single transcript line.
fn extract_skill_names(line: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = line;
    while let Some(pos) = rest.find("Skill(") {
        let after = &rest[pos + "Skill(".len()..];
        let after = after.strip_prefix('`').unwrap_or(after);
        let end = after.find(['`', ')']).unwrap_or(after.len());
        let name = after[..end].trim();
        if !name.is_empty() {
            names.push(name.to_string());
        }
        rest = &after[end..];
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_transcript(dir: &Path, name: &str, body: &str) -> PathBuf {
        std::fs::write(dir.join(name), body).unwrap();
        dir.join(name)
    }

    #[test]
    fn missing_root_yields_no_events() {
        let reader = FsUsageHistoryReader::new(vec![PathBuf::from("/no/such/dir")]);
        assert!(reader.read_events().unwrap().is_empty());
    }

    #[test]
    fn extracts_backticked_skill_names() {
        let tmp = TempDir::new().unwrap();
        write_transcript(
            tmp.path(),
            "session.jsonl",
            "{\"type\":\"user\",\"message\":\"Skill(`my-skill`) was used\"}\n",
        );
        let reader = FsUsageHistoryReader::new(vec![tmp.path().to_path_buf()]);
        let events = reader.read_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].skill_name, "my-skill");
    }

    #[test]
    fn extracts_unquoted_skill_names() {
        let tmp = TempDir::new().unwrap();
        write_transcript(tmp.path(), "session.jsonl", "Skill(my-skill) invoked\n");
        let reader = FsUsageHistoryReader::new(vec![tmp.path().to_path_buf()]);
        let events = reader.read_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].skill_name, "my-skill");
    }

    #[test]
    fn ignores_lines_without_skill_markers() {
        let tmp = TempDir::new().unwrap();
        write_transcript(
            tmp.path(),
            "session.jsonl",
            "{\"type\":\"user\",\"message\":\"hello world\"}\n",
        );
        let reader = FsUsageHistoryReader::new(vec![tmp.path().to_path_buf()]);
        assert!(reader.read_events().unwrap().is_empty());
    }

    #[test]
    fn dedupes_names_per_file() {
        let tmp = TempDir::new().unwrap();
        write_transcript(tmp.path(), "session.jsonl", "Skill(`dup`)\nSkill(`dup`)\n");
        let reader = FsUsageHistoryReader::new(vec![tmp.path().to_path_buf()]);
        let events = reader.read_events().unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn scans_nested_directories() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("sub").join("deep");
        std::fs::create_dir_all(&nested).unwrap();
        write_transcript(&nested, "x.jsonl", "Skill(`nested-skill`)\n");
        let reader = FsUsageHistoryReader::new(vec![tmp.path().to_path_buf()]);
        let events = reader.read_events().unwrap();
        assert_eq!(events[0].skill_name, "nested-skill");
    }

    #[test]
    fn from_env_resolves_claude_projects() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join(".claude").join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        write_transcript(&projects, "s.jsonl", "Skill(`env-skill`)\n");

        let original_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", tmp.path()) };
        let reader = FsUsageHistoryReader::from_env();
        match original_home {
            Some(home) => unsafe { std::env::set_var("HOME", home) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        let events = reader.read_events().unwrap();
        assert_eq!(events[0].skill_name, "env-skill");
    }

    #[test]
    fn extract_helper_handles_empty_name() {
        assert!(extract_skill_names("Skill()").is_empty());
        assert!(extract_skill_names("no marker").is_empty());
        assert_eq!(
            extract_skill_names("Skill(`a`) and Skill(`b`)"),
            vec!["a", "b"]
        );
    }
}
