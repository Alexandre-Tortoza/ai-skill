//! Git-backed skill sync: snapshots, restore, push/pull for multi-device sync.

/// A single snapshot (git commit) in the skill repository.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    /// Git commit hash (short or full).
    pub id: String,
    /// Commit message.
    pub message: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Author name.
    pub author: String,
}

/// Current sync state of the skill repository.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    /// No git repository found.
    Uninitialized,
    /// Repository is clean (no uncommitted changes).
    Clean,
    /// Repository has uncommitted changes.
    Dirty,
    /// Local is ahead of remote.
    Ahead { commits: usize },
    /// Local is behind remote.
    Behind { commits: usize },
    /// Branches have diverged.
    Diverged,
}

/// Port for git-based skill repository sync.
pub trait SkillSync {
    /// Check whether a git repository exists at the skills root.
    fn is_initialized(&self) -> Result<bool, Box<dyn std::error::Error>>;

    /// Initialize a new git repository at the skills root, with an initial commit.
    fn init(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Create a snapshot (commit) of all current skills.
    fn snapshot(&self, message: &str) -> Result<String, Box<dyn std::error::Error>>;

    /// List all snapshots (git log), most recent first.
    fn list_snapshots(&self) -> Result<Vec<Snapshot>, Box<dyn std::error::Error>>;

    /// Restore skills to a given snapshot.
    fn restore(&self, snapshot_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Show the current sync status.
    fn status(&self) -> Result<SyncStatus, Box<dyn std::error::Error>>;

    /// Push to a remote.
    fn push(&self, remote: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Pull from a remote.
    fn pull(&self, remote: &str, branch: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Add a git remote.
    fn add_remote(&self, name: &str, url: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// In-memory no-op implementation for tests.
pub struct NoopSkillSync;

impl SkillSync for NoopSkillSync {
    fn is_initialized(&self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(false)
    }

    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn snapshot(&self, _message: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("0000000".into())
    }

    fn list_snapshots(&self) -> Result<Vec<Snapshot>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }

    fn restore(&self, _snapshot_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn status(&self) -> Result<SyncStatus, Box<dyn std::error::Error>> {
        Ok(SyncStatus::Uninitialized)
    }

    fn push(&self, _remote: &str, _branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn pull(&self, _remote: &str, _branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn add_remote(&self, _name: &str, _url: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_is_initialized_returns_false() {
        let sync = NoopSkillSync;
        assert!(!sync.is_initialized().unwrap());
    }

    #[test]
    fn noop_snapshot_returns_hash() {
        let sync = NoopSkillSync;
        assert_eq!(sync.snapshot("msg").unwrap(), "0000000");
    }

    #[test]
    fn noop_list_snapshots_returns_empty() {
        let sync = NoopSkillSync;
        assert!(sync.list_snapshots().unwrap().is_empty());
    }

    #[test]
    fn noop_restore_does_not_error() {
        let sync = NoopSkillSync;
        assert!(sync.restore("abc123").is_ok());
    }

    #[test]
    fn noop_status_returns_uninitialized() {
        let sync = NoopSkillSync;
        assert_eq!(sync.status().unwrap(), SyncStatus::Uninitialized);
    }

    #[test]
    fn noop_push_pull_add_remote_do_not_error() {
        let sync = NoopSkillSync;
        assert!(sync.push("origin", "main").is_ok());
        assert!(sync.pull("origin", "main").is_ok());
        assert!(
            sync.add_remote("origin", "https://example.com/repo.git")
                .is_ok()
        );
    }

    #[test]
    fn snapshot_fields_accessible() {
        let s = Snapshot {
            id: "abc123".into(),
            message: "fix: update code-review".into(),
            timestamp: "2026-07-12T10:00:00Z".into(),
            author: "User".into(),
        };
        assert_eq!(s.id, "abc123");
        assert_eq!(s.message, "fix: update code-review");
    }

    #[test]
    fn sync_status_variants_are_distinct() {
        assert_ne!(SyncStatus::Uninitialized, SyncStatus::Clean);
        assert_ne!(SyncStatus::Dirty, SyncStatus::Clean);
        assert_ne!(
            SyncStatus::Ahead { commits: 1 },
            SyncStatus::Behind { commits: 1 }
        );
        assert_eq!(
            SyncStatus::Ahead { commits: 2 },
            SyncStatus::Ahead { commits: 2 }
        );
    }
}
