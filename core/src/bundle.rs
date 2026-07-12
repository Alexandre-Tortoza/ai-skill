//! Bundle model and persistence port.
//!
//! A bundle is a predefined set of skills (by name) that can be installed
//! together — e.g. `frontend`, `release-prep`, `ops`.

use serde::{Deserialize, Serialize};

/// A predefined collection of skills installable as a group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bundle {
    /// Display name (e.g. `frontend`, `ops`).
    pub name: String,
    /// Short description of what the bundle provides.
    #[serde(default)]
    pub description: String,
    /// Skill names that make up this bundle.
    pub skills: Vec<String>,
}

/// Persistence port for reading available skill bundles.
pub trait BundleStore {
    /// Returns all available bundles.
    fn list(&self) -> Result<Vec<Bundle>, Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bundle() -> Bundle {
        Bundle {
            name: "frontend".into(),
            description: "Frontend dev skills".into(),
            skills: vec!["react-rules".into(), "typescript-rules".into()],
        }
    }

    #[test]
    fn bundle_fields_are_accessible() {
        let b = sample_bundle();
        assert_eq!(b.name, "frontend");
        assert_eq!(b.description, "Frontend dev skills");
        assert_eq!(b.skills.len(), 2);
    }

    #[test]
    fn bundle_store_is_object_safe() {
        struct FakeStore;
        impl BundleStore for FakeStore {
            fn list(&self) -> Result<Vec<Bundle>, Box<dyn std::error::Error>> {
                Ok(vec![])
            }
        }
        let store: Box<dyn BundleStore> = Box::new(FakeStore);
        assert!(store.list().unwrap().is_empty());
    }
}
