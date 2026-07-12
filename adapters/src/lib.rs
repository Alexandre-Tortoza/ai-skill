//! Filesystem and process adapters that implement the core ports.
//!
//! Each adapter shells out to the real filesystem or an external binary (`npx`, `git`).
//! No in-memory fakes — see domain tests in `ai_skill_core` for those.

pub mod cli_installer;
pub mod fs_profile_store;
pub mod fs_settings_store;
pub mod fs_skill_creator;
pub mod fs_skill_repository;
pub mod fs_toggler;
pub mod fs_watcher;
pub mod git_drift_checker;
pub mod npx_catalog_gateway;

/// Adapter that shells out to `npx skills add|remove|update`.
pub use cli_installer::CliInstaller;
/// Adapter that reads/writes `.claude/settings.json` for auto-trigger control.
pub use fs_settings_store::FsSettingsStore;
/// Adapter that persists profiles as YAML files under `~/.claude/ai-skill/profiles/`.
pub use fs_profile_store::FsProfileStore;
/// Adapters that create and write SKILL.md files to the filesystem.
pub use fs_skill_creator::{FsSkillCreator, FsSkillWriter};
/// Adapter that scans `~/.claude/skills/` (and optionally the project `.claude/skills/`).
pub use fs_skill_repository::FsSkillRepository;
/// Adapter that enables/disables skills by renaming directories (`.disabled` suffix).
pub use fs_toggler::FsToggler;
/// Adapter that watches skill directories for changes using `notify`.
pub use fs_watcher::FsWatcher;
/// Adapter that checks Git drift by comparing local HEAD with `@{u}`.
pub use git_drift_checker::GitDriftChecker;
/// Adapter that searches the remote catalog via `npx skills find`.
pub use npx_catalog_gateway::NpxCatalogGateway;
