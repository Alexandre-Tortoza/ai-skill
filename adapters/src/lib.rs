//! Filesystem and process adapters that implement the core ports.
//!
//! Each adapter shells out to the real filesystem or an external binary (`npx`, `git`).
//! No in-memory fakes — see domain tests in `ai_skill_core` for those.

pub mod cli_installer;
pub mod composite_catalog_gateway;
pub mod fs_bundle_store;
pub mod fs_config_store;
pub mod fs_plugin_discoverer;
pub mod fs_profile_store;
pub mod fs_settings_store;
pub mod fs_skill_content;
pub mod fs_skill_creator;
pub mod fs_skill_repository;
pub mod fs_toggler;
pub mod fs_usage_history;
pub mod fs_watcher;
pub mod git_drift_checker;
pub mod git_skill_diff;
pub mod git_skill_sync;
pub mod import_chain;
pub mod npx_catalog_gateway;
pub mod ssh_connector;

pub use cli_installer::CliInstaller;
/// Adapter that aggregates search results from multiple catalog sources.
pub use composite_catalog_gateway::CompositeCatalogGateway;
/// Adapter that reads predefined skill bundles from YAML files.
pub use fs_bundle_store::FsBundleStore;
/// Adapter that reads/writes `~/.config/ai-skill/config.json`.
pub use fs_config_store::FsConfigStore;
/// Adapter that discovers skills from plugin marketplace manifests.
pub use fs_plugin_discoverer::{FsPluginDiscoverer, PluginDiscovererError};
/// Adapter that persists profiles as YAML files under `~/.claude/ai-skill/profiles/`.
pub use fs_profile_store::FsProfileStore;
/// Adapter that reads/writes `.claude/settings.json` for auto-trigger control.
pub use fs_settings_store::FsSettingsStore;
/// Adapter that reads a skill's on-disk file content (preview, tree, files).
pub use fs_skill_content::FsSkillContentReader;
/// Adapters that create and write SKILL.md files to the filesystem.
pub use fs_skill_creator::{FsSkillCreator, FsSkillWriter};
/// Adapter that scans `~/.claude/skills/` (and optionally the project `.claude/skills/`).
pub use fs_skill_repository::FsSkillRepository;
/// Adapter that enables/disables skills by renaming directories (`.disabled` suffix).
pub use fs_toggler::FsToggler;
/// Adapter that scans local agent history for skill usage events.
pub use fs_usage_history::{FsUsageHistoryReader, UsageHistoryError};
/// Adapter that watches skill directories for changes using `notify`.
pub use fs_watcher::FsWatcher;
/// Adapter that checks Git drift by comparing local HEAD with `@{u}`.
pub use git_drift_checker::GitDriftChecker;
/// Adapter that reads upstream diffs via `git diff HEAD..@{u} -- SKILL.md`.
pub use git_skill_diff::GitSkillDiffReader;
/// Adapter that manages a git repository for skill sync.
pub use git_skill_sync::{GitSkillSync, GitSyncError};
/// Adapter that shells out to `npx skills add|remove|update`.
/// Adapter that traces script import chains for security analysis.
pub use import_chain::{ImportChainResult, trace_import_chain};
/// Adapter that searches the remote catalog via `npx skills find`.
pub use npx_catalog_gateway::NpxCatalogGateway;
/// Adapter that shells out to `ssh` for remote machine management.
pub use ssh_connector::SshCommandConnector;
