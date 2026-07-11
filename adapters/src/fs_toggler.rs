//! Adapter that enables/disables skills via filesystem rename (`.disabled` suffix).

use std::path::Path;

use ai_skill_core::SkillToggler;

/// Enables skills by stripping `.disabled`, disables by appending it, and adopts
/// by writing a `.ai-skill` marker file.
pub struct FsToggler;

impl SkillToggler for FsToggler {
    fn enable(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let path_str = path.to_string_lossy();
        if let Some(base) = path_str.strip_suffix(".disabled") {
            std::fs::rename(path, base)?;
        }
        Ok(())
    }

    fn disable(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let disabled = format!("{}.disabled", path.display());
        std::fs::rename(path, &disabled)?;
        Ok(())
    }

    fn adopt(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path.join(".ai-skill"), "")?;
        Ok(())
    }

    fn preview_enable(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy();
        let target = path_str.strip_suffix(".disabled").unwrap_or(&path_str);
        format!("rename {} → {}", path.display(), target)
    }

    fn preview_disable(&self, path: &Path) -> String {
        format!("rename {} → {}.disabled", path.display(), path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_dir(parent: &Path, name: &str) -> std::path::PathBuf {
        let p = parent.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn disable_renames_dir_with_disabled_suffix() {
        let tmp = TempDir::new().unwrap();
        let skill_path = make_dir(tmp.path(), "my-skill");

        FsToggler.disable(&skill_path).unwrap();

        assert!(!skill_path.exists());
        assert!(tmp.path().join("my-skill.disabled").exists());
    }

    #[test]
    fn enable_renames_disabled_dir_back() {
        let tmp = TempDir::new().unwrap();
        let disabled_path = make_dir(tmp.path(), "my-skill.disabled");

        FsToggler.enable(&disabled_path).unwrap();

        assert!(!disabled_path.exists());
        assert!(tmp.path().join("my-skill").exists());
    }

    #[test]
    fn enable_on_non_disabled_path_is_noop() {
        let tmp = TempDir::new().unwrap();
        let skill_path = make_dir(tmp.path(), "my-skill");

        FsToggler.enable(&skill_path).unwrap();

        assert!(skill_path.exists());
    }

    #[test]
    fn preview_disable_contains_disabled_suffix() {
        let path = std::path::PathBuf::from("/skills/my-skill");
        let preview = FsToggler.preview_disable(&path);
        assert!(preview.contains(".disabled"));
        assert!(preview.contains("my-skill"));
    }

    #[test]
    fn preview_enable_shows_path_without_suffix() {
        let path = std::path::PathBuf::from("/skills/my-skill.disabled");
        let preview = FsToggler.preview_enable(&path);
        assert!(preview.contains("my-skill"));
    }

    #[test]
    fn adopt_creates_ai_skill_marker() {
        let tmp = TempDir::new().unwrap();
        let skill_path = make_dir(tmp.path(), "external-skill");

        FsToggler.adopt(&skill_path).unwrap();

        assert!(skill_path.join(".ai-skill").exists());
    }
}
