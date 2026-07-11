//! Drift-detection state enum and checker port.

use serde::Serialize;
use std::path::Path;

/// Describes whether a skill is out of sync with its upstream source.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftState {
    /// Drift has not been checked yet.
    #[default]
    Unknown,
    /// Local copy matches upstream.
    UpToDate,
    /// Local and upstream versions differ.
    UpdateAvailable {
        /// Local commit hash or content fingerprint.
        local_hash: String,
        /// Upstream commit hash or content fingerprint.
        upstream_hash: String,
    },
    /// The skill is not inside a Git repository.
    NoGitRepo,
    /// The skill has no configured upstream remote.
    NoUpstream,
}

/// Port for checking whether a skill is up-to-date with its upstream.
pub trait DriftChecker {
    /// Checks drift state for the skill at `path`.
    fn check(&self, path: &Path) -> DriftState;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_is_unknown() {
        assert_eq!(DriftState::default(), DriftState::Unknown);
    }

    #[test]
    fn up_to_date_not_equal_unknown() {
        assert_ne!(DriftState::UpToDate, DriftState::Unknown);
    }

    #[test]
    fn update_available_exposes_hashes() {
        let state = DriftState::UpdateAvailable {
            local_hash: "abc1234".into(),
            upstream_hash: "def5678".into(),
        };
        if let DriftState::UpdateAvailable {
            local_hash,
            upstream_hash,
        } = &state
        {
            assert_eq!(local_hash, "abc1234");
            assert_eq!(upstream_hash, "def5678");
        } else {
            panic!("expected UpdateAvailable");
        }
    }

    #[test]
    fn drift_checker_trait_object_compiles() {
        struct AlwaysUpToDate;
        impl DriftChecker for AlwaysUpToDate {
            fn check(&self, _path: &Path) -> DriftState {
                DriftState::UpToDate
            }
        }
        let checker: Box<dyn DriftChecker> = Box::new(AlwaysUpToDate);
        assert_eq!(checker.check(&PathBuf::from("/tmp")), DriftState::UpToDate);
    }

    #[test]
    fn no_git_repo_and_no_upstream_are_distinct() {
        assert_ne!(DriftState::NoGitRepo, DriftState::NoUpstream);
    }
}
