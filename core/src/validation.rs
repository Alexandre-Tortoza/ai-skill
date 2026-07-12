//! Validation state enum describing skill health.

use serde::Serialize;
use std::path::PathBuf;

/// Health state of an installed skill.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationState {
    /// Everything looks good.
    Valid,
    /// The skill's symlink target does not exist.
    BrokenSymlink,
    /// The skill directory exists but has no SKILL.md (or equivalent).
    MissingManifest,
    /// The frontmatter YAML could not be parsed.
    InvalidFrontmatter {
        /// Human-readable parse error message.
        reason: String,
    },
    /// A lock file exists for a skill that is no longer present.
    OrphanLock,
    /// Another skill with the same name (case-insensitive) is already installed.
    Duplicate {
        /// Path of the first installed instance.
        conflicts_with: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_variant_equals_itself() {
        assert_eq!(ValidationState::Valid, ValidationState::Valid);
    }

    #[test]
    fn broken_symlink_variant_constructs() {
        let v = ValidationState::BrokenSymlink;
        assert!(matches!(v, ValidationState::BrokenSymlink));
    }

    #[test]
    fn missing_manifest_variant_constructs() {
        let v = ValidationState::MissingManifest;
        assert!(matches!(v, ValidationState::MissingManifest));
    }

    #[test]
    fn invalid_frontmatter_carries_reason() {
        let v = ValidationState::InvalidFrontmatter {
            reason: "unexpected key".to_string(),
        };
        match v {
            ValidationState::InvalidFrontmatter { reason } => {
                assert_eq!(reason, "unexpected key");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn orphan_lock_variant_constructs() {
        let v = ValidationState::OrphanLock;
        assert!(matches!(v, ValidationState::OrphanLock));
    }

    #[test]
    fn duplicate_carries_conflicting_path() {
        let path = PathBuf::from("/some/path");
        let v = ValidationState::Duplicate {
            conflicts_with: path.clone(),
        };
        match v {
            ValidationState::Duplicate { conflicts_with } => {
                assert_eq!(conflicts_with, path);
            }
            _ => panic!("wrong variant"),
        }
    }
}
