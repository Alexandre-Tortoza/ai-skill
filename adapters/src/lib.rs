//! Filesystem and process adapters that implement the core ports.
//!
//! Each adapter shells out to the real filesystem or an external binary (`npx`, `git`).
//! No in-memory fakes — see domain tests in `ai_skill_core` for those.

pub mod cli_installer;
pub mod composite_catalog_gateway;
pub mod fs_bundle_store;
pub mod fs_profile_store;
pub mod fs_settings_store;
pub mod fs_skill_creator;
pub mod fs_skill_repository;
pub mod fs_toggler;
pub mod fs_watcher;
pub mod git_drift_checker;
pub mod import_chain;
pub mod npx_catalog_gateway;
pub mod ssh_connector;

pub use cli_installer::CliInstaller;
/// Adapter that aggregates search results from multiple catalog sources.
pub use composite_catalog_gateway::CompositeCatalogGateway;
/// Adapter that reads predefined skill bundles from YAML files.
pub use fs_bundle_store::FsBundleStore;
/// Adapter that persists profiles as YAML files under `~/.claude/ai-skill/profiles/`.
pub use fs_profile_store::FsProfileStore;
/// Adapter that reads/writes `.claude/settings.json` for auto-trigger control.
pub use fs_settings_store::FsSettingsStore;
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
/// Adapter that shells out to `npx skills add|remove|update`.
/// Adapter that traces script import chains for security analysis.
pub use import_chain::{ImportChainResult, trace_import_chain};
/// Adapter that searches the remote catalog via `npx skills find`.
pub use npx_catalog_gateway::NpxCatalogGateway;
/// Adapter that shells out to `ssh` for remote machine management.
pub use ssh_connector::SshCommandConnector;
