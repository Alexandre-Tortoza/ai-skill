//! Filesystem-based [`ProfileStore`] that reads/writes YAML files.

use std::path::PathBuf;

use ai_skill_core::{Phase, Profile, ProfileStore};

fn home_dir() -> Result<PathBuf, std::io::Error> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME is not set; set HOME or run ai-skill from a login shell",
        )
    })
}

/// Persists profiles as YAML files under `~/.claude/ai-skill/profiles/`.
pub struct FsProfileStore {
    base_dir: PathBuf,
}

impl FsProfileStore {
    /// Resolves the base directory from `$HOME`.
    pub fn from_env() -> Result<Self, std::io::Error> {
        Ok(Self {
            base_dir: home_dir()?.join(".claude/ai-skill/profiles"),
        })
    }

    /// Creates a store with an explicit base directory.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn profile_path(&self, name: &str) -> PathBuf {
        self.base_dir.join(format!("{name}.yaml"))
    }
}

impl ProfileStore for FsProfileStore {
    fn list(&self) -> Result<Vec<Profile>, Box<dyn std::error::Error>> {
        if !self.base_dir.exists() {
            std::fs::create_dir_all(&self.base_dir)?;
            self.seed_default_presets()?;
            return Ok(vec![]);
        }
        let mut profiles = Vec::new();
        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            let profile: Profile = serde_norway::from_str(&content)?;
            profiles.push(profile);
        }
        if profiles.is_empty() {
            self.seed_default_presets()?;
        }
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profiles)
    }

    fn save(&self, profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.base_dir)?;
        let content = serde_norway::to_string(profile)?;
        std::fs::write(self.profile_path(&profile.name), content)?;
        Ok(())
    }

    fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.profile_path(name);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

impl FsProfileStore {
    fn seed_default_presets(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (name, phase) in [
            ("init", Phase::Init),
            ("dev", Phase::Dev),
            ("test", Phase::Test),
            ("release", Phase::Release),
        ] {
            let path = self.profile_path(name);
            if !path.exists() {
                let profile = Profile {
                    name: name.to_string(),
                    skill_names: vec![],
                    phase: Some(phase),
                };
                let content = serde_norway::to_string(&profile)?;
                std::fs::write(path, content)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, FsProfileStore) {
        let dir = TempDir::new().unwrap();
        let store = FsProfileStore::new(dir.path().to_path_buf());
        (dir, store)
    }

    fn profile(name: &str, skills: &[&str]) -> Profile {
        Profile {
            name: name.to_string(),
            skill_names: skills.iter().map(|s| s.to_string()).collect(),
            phase: None,
        }
    }

    #[test]
    fn list_on_empty_dir_returns_empty() {
        let (_dir, store) = make_store();
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn list_on_nonexistent_dir_creates_and_seeds() {
        let dir = TempDir::new().unwrap();
        let nonexistent = dir.path().join("nonexistent_subdir");
        let store = FsProfileStore::new(nonexistent.clone());
        let result = store.list().unwrap();
        // Creates the directory and returns empty (seeding happens on next list)
        assert!(nonexistent.exists());
        assert!(result.is_empty());
        // Second call should return the seeded presets
        let second = store.list().unwrap();
        assert_eq!(second.len(), 4);
    }

    #[test]
    fn save_creates_yaml_file() {
        let (_dir, store) = make_store();
        let p = profile("dev", &["alpha", "beta"]);
        store.save(&p).unwrap();
        let path = store.profile_path("dev");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("dev"));
        assert!(content.contains("alpha"));
        assert!(content.contains("beta"));
    }

    #[test]
    fn list_after_save_returns_profile() {
        let (_dir, store) = make_store();
        let p = profile("dev", &["alpha", "beta"]);
        store.save(&p).unwrap();
        let profiles = store.list().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "dev");
        assert_eq!(profiles[0].skill_names, vec!["alpha", "beta"]);
    }

    #[test]
    fn delete_removes_file() {
        let (_dir, store) = make_store();
        let p = profile("dev", &["alpha"]);
        store.save(&p).unwrap();
        store.delete("dev").unwrap();
        let profiles = store.list().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn delete_nonexistent_does_not_error() {
        let (_dir, store) = make_store();
        assert!(store.delete("ghost").is_ok());
    }

    #[test]
    fn list_ignores_non_yaml_files() {
        let (dir, store) = make_store();
        std::fs::create_dir_all(&store.base_dir).unwrap();
        std::fs::write(store.base_dir.join("notes.txt"), "not yaml").unwrap();
        let _ = dir; // keep alive
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn multiple_profiles_are_sorted_by_name() {
        let (_dir, store) = make_store();
        store.save(&profile("zzz", &["x"])).unwrap();
        store.save(&profile("aaa", &["y"])).unwrap();
        let profiles = store.list().unwrap();
        assert_eq!(profiles[0].name, "aaa");
        assert_eq!(profiles[1].name, "zzz");
    }

    #[test]
    fn from_env_resolves_home_directory() {
        let tmp = TempDir::new().unwrap();
        let profiles_dir = tmp.path().join(".claude/ai-skill/profiles");
        fs::create_dir_all(&profiles_dir).unwrap();
        let p = profile("dev", &["alpha"]);
        let store = FsProfileStore::new(profiles_dir);
        store.save(&p).unwrap();

        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", tmp.path());
        }
        let from_env_store = FsProfileStore::from_env().unwrap();
        if let Some(h) = original_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }

        let profiles = from_env_store.list().unwrap();
        assert!(!profiles.is_empty());
        assert_eq!(profiles[0].name, "dev");
    }

    #[test]
    fn list_with_malformed_yaml_returns_error() {
        let (dir, store) = make_store();
        std::fs::create_dir_all(&store.base_dir).unwrap();
        std::fs::write(store.base_dir.join("bad.yaml"), "not: valid: yaml: [").unwrap();
        let _ = dir;
        assert!(store.list().is_err());
    }
}
