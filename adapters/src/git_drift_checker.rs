//! Adapter that checks drift by comparing `git rev-parse HEAD` with `@{u}`.

use std::path::Path;
use std::process::Command;

use ai_skill_core::{DriftChecker, DriftState};

/// Compares local HEAD with the upstream tracking branch to detect drift.
pub struct GitDriftChecker;

impl DriftChecker for GitDriftChecker {
    fn check(&self, path: &Path) -> DriftState {
        let local_hash = match run_git(path, &["rev-parse", "HEAD"]) {
            Some(h) => h,
            None => return DriftState::NoGitRepo,
        };

        let upstream_hash = match run_git(path, &["rev-parse", "@{u}"]) {
            Some(h) => h,
            None => return DriftState::NoUpstream,
        };

        if local_hash == upstream_hash {
            DriftState::UpToDate
        } else {
            DriftState::UpdateAvailable {
                local_hash,
                upstream_hash,
            }
        }
    }
}

/// Runs a Git command in the given working directory and returns stdout on success.
fn run_git(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;

    if output.status.success() {
        let hash = String::from_utf8(output.stdout).ok()?.trim().to_string();
        if hash.is_empty() { None } else { Some(hash) }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn path_without_git_returns_no_git_repo() {
        let dir = TempDir::new().unwrap();
        let checker = GitDriftChecker;
        assert_eq!(checker.check(dir.path()), DriftState::NoGitRepo);
    }

    #[test]
    fn git_repo_without_upstream_returns_no_upstream() {
        let dir = TempDir::new().unwrap();
        // init a bare repo with no remote
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "t@t.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "t@t.com")
            .current_dir(dir.path())
            .output()
            .unwrap();

        let checker = GitDriftChecker;
        assert_eq!(checker.check(dir.path()), DriftState::NoUpstream);
    }

    #[test]
    #[ignore = "requires a git repo with a configured upstream remote"]
    fn live_repo_with_upstream_returns_up_to_date_or_update_available() {
        let checker = GitDriftChecker;
        let result = checker.check(std::path::Path::new("."));
        assert!(matches!(
            result,
            DriftState::UpToDate | DriftState::UpdateAvailable { .. }
        ));
    }
}
