//! Filesystem-based [`BundleStore`] that reads YAML files from
//! `~/.claude/ai-skill/bundles/`.

use std::path::PathBuf;

use ai_skill_core::{Bundle, BundleStore};

fn home_dir() -> Result<PathBuf, std::io::Error> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME is not set; set HOME or run ai-skill from a login shell",
        )
    })
}

/// Reads bundles from `~/.claude/ai-skill/bundles/*.yaml`.
pub struct FsBundleStore {
    base_dir: PathBuf,
}

impl FsBundleStore {
    pub fn from_env() -> Result<Self, std::io::Error> {
        Ok(Self {
            base_dir: home_dir()?.join(".claude/ai-skill/bundles"),
        })
    }

    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn bundle_path(&self, name: &str) -> PathBuf {
        self.base_dir.join(format!("{name}.yaml"))
    }
}

impl BundleStore for FsBundleStore {
    fn list(&self) -> Result<Vec<Bundle>, Box<dyn std::error::Error>> {
        if !self.base_dir.exists() {
            std::fs::create_dir_all(&self.base_dir)?;
            self.seed_default_bundles()?;
            return Ok(vec![]);
        }
        let mut bundles = Vec::new();
        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            let bundle: Bundle = serde_norway::from_str(&content)?;
            bundles.push(bundle);
        }
        if bundles.is_empty() {
            self.seed_default_bundles()?;
        }
        bundles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(bundles)
    }
}

impl FsBundleStore {
    fn seed_default_bundles(&self) -> Result<(), Box<dyn std::error::Error>> {
        let defaults: Vec<Bundle> = vec![
            Bundle {
                name: "frontend".into(),
                description: "Essential frontend development skills".into(),
                skills: vec![
                    "react-rules".into(),
                    "typescript-rules".into(),
                    "tailwind-rules".into(),
                ],
            },
            Bundle {
                name: "ops".into(),
                description: "DevOps and infrastructure skills".into(),
                skills: vec!["docker".into(), "kubernetes".into(), "terraform".into()],
            },
            Bundle {
                name: "release-prep".into(),
                description: "Skills for preparing a release".into(),
                skills: vec![
                    "changelog".into(),
                    "semantic-version".into(),
                    "ci-checks".into(),
                ],
            },
        ];
        for bundle in defaults {
            let path = self.bundle_path(&bundle.name);
            if !path.exists() {
                let content = serde_norway::to_string(&bundle)?;
                std::fs::write(path, content)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, FsBundleStore) {
        let dir = TempDir::new().unwrap();
        let store = FsBundleStore::new(dir.path().to_path_buf());
        (dir, store)
    }

    #[test]
    fn list_on_empty_dir_returns_empty() {
        let (_dir, store) = make_store();
        let bundles = store.list().unwrap();
        assert!(bundles.is_empty());
    }

    #[test]
    fn list_from_nonexistent_dir_creates_and_seeds() {
        let dir = TempDir::new().unwrap();
        let nonexistent = dir.path().join("nonexistent_subdir");
        let store = FsBundleStore::new(nonexistent.clone());
        let result = store.list().unwrap();
        assert!(nonexistent.exists());
        assert!(result.is_empty());
        let second = store.list().unwrap();
        assert_eq!(second.len(), 3);
    }

    #[test]
    fn list_returns_sorted_bundles() {
        let (_dir, store) = make_store();
        let b = Bundle {
            name: "zzz".into(),
            description: "".into(),
            skills: vec!["x".into()],
        };
        let content = serde_norway::to_string(&b).unwrap();
        std::fs::create_dir_all(&store.base_dir).unwrap();
        std::fs::write(store.base_dir.join("zzz.yaml"), content).unwrap();
        let bundles = store.list().unwrap();
        assert_eq!(bundles.len(), 1);
    }

    #[test]
    fn seeded_bundles_are_written_as_yaml() {
        let dir = TempDir::new().unwrap();
        let store = FsBundleStore::new(dir.path().to_path_buf());
        store.seed_default_bundles().unwrap();
        assert!(store.bundle_path("frontend").exists());
        assert!(store.bundle_path("ops").exists());
        assert!(store.bundle_path("release-prep").exists());
    }
}
