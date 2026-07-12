//! Adapter that shells out to `npx skills` for install/remove/update operations.

use std::path::Path;
use std::process::Command;

use ai_skill_core::{Scope, SkillInstaller};
use thiserror::Error;

/// Errors that can occur when running the `npx skills` CLI.
#[derive(Error, Debug)]
pub enum CliInstallerError {
    /// The `npx` binary could not be found or started.
    #[error("failed to run npx; install Node.js/npm and ensure npx is on PATH: {0}")]
    Io(#[from] std::io::Error),
    /// The `npx skills` subcommand returned a non-zero exit code.
    #[error("npx skills exited with status {0}; rerun the command above manually for details")]
    NonZeroExit(i32),
}

/// Delegates install, remove, and update to the `npx skills` CLI.
pub struct CliInstaller;

fn scope_flag(scope: &Scope) -> &'static str {
    match scope {
        Scope::Global => "--global",
        Scope::Project => "--project",
    }
}

fn agents_arg(agents: &[String]) -> Option<String> {
    if agents.is_empty() {
        None
    } else {
        Some(agents.join(","))
    }
}

impl CliInstaller {
    fn install_with_npx(
        &self,
        npx: &Path,
        name: &str,
        agents: &[String],
        scope: Scope,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::new(npx);
        cmd.args(["skills", "add", name, scope_flag(&scope)]);
        if let Some(a) = agents_arg(agents) {
            cmd.args(["--agents", &a]);
        }
        let status = cmd.status().map_err(CliInstallerError::Io)?;
        if !status.success() {
            return Err(CliInstallerError::NonZeroExit(status.code().unwrap_or(-1)).into());
        }
        Ok(())
    }

    fn remove_with_npx(&self, npx: &Path, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new(npx)
            .args(["skills", "remove", &path.to_string_lossy()])
            .status()
            .map_err(CliInstallerError::Io)?;
        if !status.success() {
            return Err(CliInstallerError::NonZeroExit(status.code().unwrap_or(-1)).into());
        }
        Ok(())
    }

    fn update_with_npx(&self, npx: &Path, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new(npx)
            .args(["skills", "update", &path.to_string_lossy()])
            .status()
            .map_err(CliInstallerError::Io)?;
        if !status.success() {
            return Err(CliInstallerError::NonZeroExit(status.code().unwrap_or(-1)).into());
        }
        Ok(())
    }
}

impl SkillInstaller for CliInstaller {
    fn install(
        &self,
        name: &str,
        agents: &[String],
        scope: Scope,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.install_with_npx(Path::new("npx"), name, agents, scope)
    }

    fn remove(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.remove_with_npx(Path::new("npx"), path)
    }

    fn update(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.update_with_npx(Path::new("npx"), path)
    }

    fn preview_install(&self, name: &str, agents: &[String], scope: Scope) -> String {
        let mut parts = vec![
            "npx".to_string(),
            "skills".to_string(),
            "add".to_string(),
            name.to_string(),
            scope_flag(&scope).to_string(),
        ];
        if let Some(a) = agents_arg(agents) {
            parts.push("--agents".to_string());
            parts.push(a);
        }
        parts.join(" ")
    }

    fn preview_remove(&self, path: &Path) -> String {
        format!("npx skills remove {}", path.display())
    }

    fn preview_update(&self, path: &Path) -> String {
        format!("npx skills update {}", path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn write_executable(path: &Path, content: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        drop(file);
        fs::set_permissions(path, PermissionsExt::from_mode(0o755)).unwrap();
    }

    #[test]
    fn cli_installer_error_io_construction() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = CliInstallerError::Io(io_err);
        assert!(err.to_string().contains("failed to run npx"));
    }

    #[test]
    fn cli_installer_error_io_debug() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = CliInstallerError::Io(io_err);
        assert!(!format!("{err:?}").is_empty());
    }

    #[test]
    fn cli_installer_error_non_zero_exit_construction() {
        let err = CliInstallerError::NonZeroExit(1);
        assert!(err.to_string().contains("status 1"));
    }

    #[test]
    fn cli_installer_error_non_zero_exit_debug() {
        let err = CliInstallerError::NonZeroExit(42);
        assert!(!format!("{err:?}").is_empty());
    }

    #[test]
    fn install_without_npx_returns_io_error() {
        let dir = TempDir::new().unwrap();
        let missing_npx = dir.path().join("npx");

        let installer = CliInstaller;
        let result = installer.install_with_npx(&missing_npx, "test", &[], Scope::Global);

        assert!(result.is_err());
    }

    #[test]
    fn remove_without_npx_returns_io_error() {
        let dir = TempDir::new().unwrap();
        let missing_npx = dir.path().join("npx");

        let installer = CliInstaller;
        let result = installer.remove_with_npx(&missing_npx, Path::new("/tmp/test"));

        assert!(result.is_err());
    }

    #[test]
    fn update_without_npx_returns_io_error() {
        let dir = TempDir::new().unwrap();
        let missing_npx = dir.path().join("npx");

        let installer = CliInstaller;
        let result = installer.update_with_npx(&missing_npx, Path::new("/tmp/test"));

        assert!(result.is_err());
    }

    #[test]
    fn install_with_mock_npx_succeeds() {
        let dir = TempDir::new().unwrap();
        let mock_path = dir.path().join("npx");
        write_executable(&mock_path, "#!/bin/sh\nexit 0\n");

        let installer = CliInstaller;
        let result = installer.install_with_npx(&mock_path, "test-skill", &[], Scope::Global);

        assert!(result.is_ok());
    }

    #[test]
    fn install_with_mock_npx_exit_1_returns_error() {
        let dir = TempDir::new().unwrap();
        let mock_path = dir.path().join("npx");
        write_executable(&mock_path, "#!/bin/sh\nexit 1\n");

        let installer = CliInstaller;
        let result = installer.install_with_npx(&mock_path, "test-skill", &[], Scope::Global);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("status 1"));
    }

    #[test]
    fn preview_install_contains_npx_skills_add_and_name() {
        let installer = CliInstaller;
        let preview = installer.preview_install("omarchy", &[], Scope::Global);
        assert!(preview.contains("npx skills add"));
        assert!(preview.contains("omarchy"));
        assert!(preview.contains("--global"));
    }

    #[test]
    fn preview_install_includes_agents_when_provided() {
        let installer = CliInstaller;
        let agents = vec!["claude".to_string(), "codex".to_string()];
        let preview = installer.preview_install("omarchy", &agents, Scope::Project);
        assert!(preview.contains("--agents"));
        assert!(preview.contains("claude,codex"));
        assert!(preview.contains("--project"));
    }

    #[test]
    fn preview_remove_contains_npx_skills_remove() {
        let installer = CliInstaller;
        let path = PathBuf::from("/home/user/.claude/skills/omarchy");
        let preview = installer.preview_remove(&path);
        assert!(preview.contains("npx skills remove"));
        assert!(preview.contains("omarchy"));
    }

    #[test]
    fn preview_update_contains_npx_skills_update() {
        let installer = CliInstaller;
        let path = PathBuf::from("/home/user/.claude/skills/omarchy");
        let preview = installer.preview_update(&path);
        assert!(preview.contains("npx skills update"));
        assert!(preview.contains("omarchy"));
    }

    #[test]
    #[ignore = "requires npx"]
    fn live_install_and_remove() {
        let installer = CliInstaller;
        installer.install("test-skill", &[], Scope::Global).unwrap();
    }
}
