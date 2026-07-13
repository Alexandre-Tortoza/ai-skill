//! Adapter that reads upstream diffs via `git diff HEAD..@{u} -- SKILL.md`.

use std::path::Path;
use std::process::Command;

use ai_skill_core::{DiffError, SkillDiff, SkillDiffReader, parse_diff};

/// Reads the upstream diff of a skill's manifest by shelling out to Git.
pub struct GitSkillDiffReader;

impl GitSkillDiffReader {
    /// Creates a new reader.
    pub fn new() -> Self {
        GitSkillDiffReader
    }
}

impl Default for GitSkillDiffReader {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillDiffReader for GitSkillDiffReader {
    fn read_diff(&self, path: &Path) -> Result<SkillDiff, DiffError> {
        // Probe the repo/upstream first so we can return a precise error
        // instead of a generic command failure.
        let is_repo = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["rev-parse", "HEAD"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !is_repo {
            return Err(DiffError::NoGitRepo);
        }
        let has_upstream = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["rev-parse", "@{u}"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !has_upstream {
            return Err(DiffError::NoUpstream);
        }

        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["diff", "HEAD..@{u}", "--", "SKILL.md"])
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let raw = String::from_utf8_lossy(&out.stdout);
                Ok(parse_diff(&raw))
            }
            _ => Err(DiffError::CommandFailed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::DiffLineKind;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_repo(dir: &TempDir) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "name: base\n").unwrap();
        Command::new("git")
            .args(["add", "SKILL.md"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
    }

    #[test]
    fn non_git_dir_returns_no_git_repo() {
        let dir = TempDir::new().unwrap();
        let reader = GitSkillDiffReader::new();
        assert_eq!(reader.read_diff(dir.path()), Err(DiffError::NoGitRepo));
    }

    #[test]
    fn repo_without_upstream_returns_no_upstream() {
        let dir = TempDir::new().unwrap();
        init_repo(&dir);
        let reader = GitSkillDiffReader::new();
        assert_eq!(reader.read_diff(dir.path()), Err(DiffError::NoUpstream));
    }

    /// Initializes a repo with a bare remote and a tracking upstream branch.
    fn init_repo_with_upstream(dir: &TempDir) -> PathBuf {
        let work = dir.path();
        let remote = work.join("remote.git");
        Command::new("git")
            .args(["init", "--bare", &remote.to_string_lossy()])
            .output()
            .unwrap();
        init_repo(dir);
        Command::new("git")
            .args(["remote", "add", "origin", &remote.to_string_lossy()])
            .current_dir(work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["push", "-u", "origin", "HEAD:main"])
            .current_dir(work)
            .output()
            .unwrap();
        work.to_path_buf()
    }

    #[test]
    fn local_diverged_from_upstream_produces_parsed_diff() {
        let dir = TempDir::new().unwrap();
        let work = init_repo_with_upstream(&dir);
        // Amend the local commit so HEAD diverges from the upstream branch.
        std::fs::write(work.join("SKILL.md"), "name: base\naltered line\n").unwrap();
        Command::new("git")
            .args(["add", "SKILL.md"])
            .current_dir(&work)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "--amend", "-m", "changed"])
            .current_dir(&work)
            .output()
            .unwrap();

        let reader = GitSkillDiffReader::new();
        let diff = reader.read_diff(&work).unwrap();
        let has_content = diff
            .lines
            .iter()
            .any(|l| l.kind == DiffLineKind::Add || l.kind == DiffLineKind::Remove);
        assert!(has_content, "expected added/removed lines in diff");
    }

    #[test]
    fn unchanged_manifest_produces_empty_diff() {
        let dir = TempDir::new().unwrap();
        let work = init_repo_with_upstream(&dir);
        let reader = GitSkillDiffReader::new();
        let diff = reader.read_diff(&work).unwrap();
        assert!(diff.is_empty());
    }
}
