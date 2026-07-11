//! Ports for installing, toggling, and adopting skills.

use std::path::Path;

use crate::Scope;

/// Port for adding, removing, and updating skills via external tooling (e.g. `npx skills`).
pub trait SkillInstaller {
    /// Installs a skill by name with the given agent configuration and scope.
    fn install(
        &self,
        name: &str,
        agents: &[String],
        scope: Scope,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Removes an installed skill at the given path.
    fn remove(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Updates an installed skill at the given path to the latest version.
    fn update(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Returns a human-readable command preview for installation.
    fn preview_install(&self, name: &str, agents: &[String], scope: Scope) -> String;
    /// Returns a human-readable command preview for removal.
    fn preview_remove(&self, path: &Path) -> String;
    /// Returns a human-readable command preview for updating.
    fn preview_update(&self, path: &Path) -> String;
}

/// Port for enabling, disabling, and adopting skills.
pub trait SkillToggler {
    /// Enables a previously disabled skill.
    fn enable(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    /// Disables a skill without removing it.
    fn disable(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    /// Adopts an unmanaged skill, registering it in the lock file.
    fn adopt(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    /// Returns a human-readable command preview for enabling.
    fn preview_enable(&self, path: &Path) -> String;
    /// Returns a human-readable command preview for disabling.
    fn preview_disable(&self, path: &Path) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct FakeInstaller {
        pub installed: std::cell::Cell<bool>,
    }

    impl FakeInstaller {
        fn new() -> Self {
            Self {
                installed: std::cell::Cell::new(false),
            }
        }
    }

    impl SkillInstaller for FakeInstaller {
        fn install(
            &self,
            _name: &str,
            _agents: &[String],
            _scope: Scope,
        ) -> Result<(), Box<dyn std::error::Error>> {
            self.installed.set(true);
            Ok(())
        }
        fn remove(&self, _path: &Path) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn update(&self, _path: &Path) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn preview_install(&self, name: &str, _agents: &[String], _scope: Scope) -> String {
            format!("install {name}")
        }
        fn preview_remove(&self, path: &Path) -> String {
            format!("remove {}", path.display())
        }
        fn preview_update(&self, path: &Path) -> String {
            format!("update {}", path.display())
        }
    }

    struct FakeToggler;

    impl SkillToggler for FakeToggler {
        fn enable(&self, _path: &Path) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn disable(&self, _path: &Path) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn adopt(&self, _path: &Path) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn preview_enable(&self, path: &Path) -> String {
            format!("enable {}", path.display())
        }
        fn preview_disable(&self, path: &Path) -> String {
            format!("disable {}", path.display())
        }
    }

    #[test]
    fn fake_installer_install_sets_flag() {
        let installer = FakeInstaller::new();
        installer.install("omarchy", &[], Scope::Global).unwrap();
        assert!(installer.installed.get());
    }

    #[test]
    fn fake_toggler_enable_and_disable_return_ok() {
        let toggler = FakeToggler;
        let path = PathBuf::from("/tmp/skill");
        assert!(toggler.enable(&path).is_ok());
        assert!(toggler.disable(&path).is_ok());
    }

    #[test]
    fn skill_installer_is_object_safe() {
        let installer: Box<dyn SkillInstaller> = Box::new(FakeInstaller::new());
        assert!(installer.install("x", &[], Scope::Global).is_ok());
    }

    #[test]
    fn skill_toggler_is_object_safe() {
        let toggler: Box<dyn SkillToggler> = Box::new(FakeToggler);
        let path = PathBuf::from("/tmp/s");
        assert!(toggler.enable(&path).is_ok());
    }

    #[test]
    fn preview_install_contains_skill_name() {
        let installer = FakeInstaller::new();
        let preview = installer.preview_install("omarchy", &[], Scope::Global);
        assert!(preview.contains("omarchy"));
    }
}
