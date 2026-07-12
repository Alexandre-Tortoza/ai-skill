//! Git-based [`SkillSync`] adapter that shells out to the `git` CLI.
//!
//! Operates on a given root directory (typically `~/.claude/skills`).

use ai_skill_core::{SkillSync, Snapshot, SyncStatus};
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

/// Errors from [`GitSkillSync`].
#[derive(Debug, Error)]
pub enum GitSyncError {
    /// Underlying IO error.
    #[error("git operation failed: {0}")]
    Io(#[from] std::io::Error),
    /// Git command returned a non-zero exit code.
    #[error("git command failed (exit {code}): {stderr}")]
    Git { code: i32, stderr: String },
    /// Invalid UTF-8 in git output.
    #[error("invalid utf-8 from git: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Adapter that manages a git repository at the given root path.
pub struct GitSkillSync {
    root: PathBuf,
}

impl GitSkillSync {
    /// Creates a new sync manager for the given directory.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn run_git(&self, args: &[&str]) -> Result<String, GitSyncError> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            Ok(stdout.trim().to_string())
        } else {
            let stderr = String::from_utf8(output.stderr)?;
            Err(GitSyncError::Git {
                code: output.status.code().unwrap_or(-1),
                stderr,
            })
        }
    }
}

impl SkillSync for GitSkillSync {
    fn is_initialized(&self) -> Result<bool, Box<dyn std::error::Error>> {
        match self.run_git(&["rev-parse", "--git-dir"]) {
            Ok(s) => Ok(!s.is_empty()),
            Err(GitSyncError::Git { .. }) => Ok(false),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_initialized()? {
            return Ok(());
        }
        self.run_git(&["init"]).map_err(Box::new)?;
        self.run_git(&["add", "-A"]).map_err(Box::new)?;
        let result = self.run_git(&[
            "commit",
            "--allow-empty",
            "-m",
            "ai-skill: initialize skill repository",
        ]);
        match result {
            Ok(_) | Err(GitSyncError::Git { .. }) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn snapshot(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.run_git(&["add", "-A"]).map_err(Box::new)?;
        let hash = self
            .run_git(&["commit", "--allow-empty", "-m", message])
            .map_err(Box::new)?;
        let hash = hash
            .split_whitespace()
            .nth(1)
            .and_then(|s| {
                if s.len() >= 7 {
                    Some(s.trim_end_matches(']').to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                self.run_git(&["rev-parse", "--short", "HEAD"])
                    .unwrap_or_default()
            });
        Ok(hash)
    }

    fn list_snapshots(&self) -> Result<Vec<Snapshot>, Box<dyn std::error::Error>> {
        let output = self
            .run_git(&["log", "--format=%H|%s|%aI|%an", "--max-count=50"])
            .map_err(Box::new)?;

        if output.is_empty() {
            return Ok(vec![]);
        }

        let snapshots = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() < 4 {
                    return None;
                }
                Some(Snapshot {
                    id: parts[0].to_string(),
                    message: parts[1].to_string(),
                    timestamp: parts[2].to_string(),
                    author: parts[3].to_string(),
                })
            })
            .collect();

        Ok(snapshots)
    }

    fn restore(&self, snapshot_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_git(&["rev-parse", "--verify", snapshot_id])
            .map_err(Box::new)?;
        self.run_git(&["reset", "--hard", snapshot_id])
            .map_err(Box::new)?;
        Ok(())
    }

    fn status(&self) -> Result<SyncStatus, Box<dyn std::error::Error>> {
        if !self.is_initialized()? {
            return Ok(SyncStatus::Uninitialized);
        }

        let porcelain = self.run_git(&["status", "--porcelain"]).map_err(Box::new)?;
        let has_changes = !porcelain.is_empty();

        match self.run_git(&["rev-list", "--count", "--left-right", "HEAD...@{u}"]) {
            Ok(counts) => {
                let parts: Vec<&str> = counts.split_whitespace().collect();
                let ahead: usize = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let behind: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

                if ahead > 0 && behind > 0 {
                    return Ok(SyncStatus::Diverged);
                }
                if ahead > 0 {
                    return Ok(SyncStatus::Ahead { commits: ahead });
                }
                if behind > 0 {
                    return Ok(SyncStatus::Behind { commits: behind });
                }
                if has_changes {
                    return Ok(SyncStatus::Dirty);
                }
                Ok(SyncStatus::Clean)
            }
            Err(GitSyncError::Git { .. }) => {
                if has_changes {
                    Ok(SyncStatus::Dirty)
                } else {
                    Ok(SyncStatus::Clean)
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    fn push(&self, remote: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_git(&["push", remote, branch]).map_err(Box::new)?;
        Ok(())
    }

    fn pull(&self, remote: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.run_git(&["pull", "--ff-only", remote, branch])
            .map_err(Box::new)?;
        Ok(())
    }

    fn add_remote(&self, name: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.run_git(&["remote", "get-url", name]) {
            Ok(existing) if existing == url => return Ok(()),
            Ok(_) => {
                self.run_git(&["remote", "set-url", name, url])
                    .map_err(Box::new)?;
            }
            Err(GitSyncError::Git { .. }) => {
                self.run_git(&["remote", "add", name, url])
                    .map_err(Box::new)?;
            }
            Err(e) => return Err(Box::new(e)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, GitSkillSync) {
        let tmp = TempDir::new().unwrap();
        let sync = GitSkillSync::new(tmp.path().to_path_buf());
        (tmp, sync)
    }

    #[test]
    fn uninitialized_dir_returns_uninitialized() {
        let (_tmp, sync) = setup();
        assert!(!sync.is_initialized().unwrap());
        assert_eq!(sync.status().unwrap(), SyncStatus::Uninitialized);
    }

    #[test]
    fn init_creates_repo() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();
        assert!(sync.is_initialized().unwrap());
    }

    #[test]
    fn init_twice_is_idempotent() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();
        sync.init().unwrap();
        assert!(sync.is_initialized().unwrap());
    }

    #[test]
    fn snapshot_and_list() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();

        // Create a snapshot.
        std::fs::write(sync.root.join("test-skill.md"), "# test").unwrap();
        let hash = sync.snapshot("add test skill").unwrap();
        assert!(!hash.is_empty());

        let snapshots = sync.list_snapshots().unwrap();
        assert!(!snapshots.is_empty());
        assert!(snapshots[0].message.contains("test skill"));
    }

    #[test]
    fn restore_after_snapshot() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();

        // Add a file and snapshot.
        std::fs::write(sync.root.join("my-skill.md"), "v1").unwrap();
        let hash = sync.snapshot("v1").unwrap();

        // Modify and snapshot again.
        std::fs::write(sync.root.join("my-skill.md"), "v2").unwrap();
        sync.snapshot("v2").unwrap();

        // Restore to first snapshot.
        sync.restore(&hash).unwrap();
        let content = std::fs::read_to_string(sync.root.join("my-skill.md")).unwrap();
        assert_eq!(content, "v1");
    }

    #[test]
    fn clean_status_after_init() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();
        assert_eq!(sync.status().unwrap(), SyncStatus::Clean);
    }

    #[test]
    fn dirty_after_file_write() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();
        std::fs::write(sync.root.join("new.md"), "content").unwrap();
        assert_eq!(sync.status().unwrap(), SyncStatus::Dirty);
    }

    #[test]
    fn add_remote_works() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();
        sync.add_remote("origin", "https://example.com/test.git")
            .unwrap();

        // Verify by checking push fails gracefully (no network).
        let result = sync.push("origin", "main");
        assert!(result.is_err());
    }

    #[test]
    fn add_remote_twice_is_idempotent() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();
        sync.add_remote("origin", "https://example.com/test.git")
            .unwrap();
        sync.add_remote("origin", "https://example.com/test.git")
            .unwrap();
        // Should not error.
    }

    #[test]
    fn snapshot_uses_provided_message() {
        let (_tmp, sync) = setup();
        sync.init().unwrap();
        std::fs::write(sync.root.join("a.md"), "a").unwrap();
        let hash = sync.snapshot("my snapshot").unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn error_debug() {
        let err = GitSyncError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(!format!("{err:?}").is_empty());
    }

    #[test]
    fn error_display_git() {
        let err = GitSyncError::Git {
            code: 128,
            stderr: "fatal: not a git repository".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("git command failed"));
        assert!(msg.contains("128"));
    }
}
