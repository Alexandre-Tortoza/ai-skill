//! Domain types, ports, and pure functions for ai-skill.
//!
//! This crate defines the hexagonal-core boundary: skill model, repository/catalog/installer
//! ports, frontmatter parsing, security scanning, profile diffing, drift detection, audit,
//! duplicate detection, and skill scaffolding.

pub mod audit;
pub mod budget;
pub mod bundle;
pub mod catalog;
pub mod config;
pub mod creator;
pub mod drift;
pub mod duplicate_detector;
pub mod external_scanner;
pub mod frontmatter;
pub mod installer;
pub mod linter;
pub mod mode;
pub mod plugin_discovery;
pub mod profile;
pub mod repository;
pub mod security_scan;
pub mod settings;
pub mod signature;
pub mod skill;
pub mod ssh;
pub mod sync;
pub mod validation;

/// Report grouping skills by their health category.
pub use audit::{AuditReport, audit_skills};
/// Context budget estimation types and functions.
pub use budget::{
    BudgetWarning, ContextBudget, SkillCost, calculate_budget, classify_budget, estimate_skill_cost,
};
/// Bundle model and persistence port for predefined skill collections.
pub use bundle::{Bundle, BundleStore};
/// Port for querying a remote skill catalog.
pub use catalog::{AnyCatalogGateway, CatalogEntry};
/// TUI configuration model and persistence port.
pub use config::{CONFIG_DIR, CONFIG_FILE, ConfigStore, TuiConfig};
/// Ports and helpers for creating and editing skill manifests.
pub use creator::{SkillCreator, SkillWriter, apply_edit, scaffold_skill};
/// Port and state enum for detecting upstream drift.
pub use drift::{DriftChecker, DriftState};
/// Case-insensitive duplicate name detection across a skill slice.
pub use duplicate_detector::detect_duplicates;
/// Port for integrating with external security scanners.
pub use external_scanner::{ExternalFinding, ExternalScanner, NoopExternalScanner};
/// Frontmatter (`---` delimited YAML) parsing and body extraction.
pub use frontmatter::{ParseError, SkillMetadata, extract_body, parse_frontmatter};
/// Ports for installing, removing, and updating skills via external tooling.
pub use installer::{SkillInstaller, SkillToggler};
/// Linter for skill descriptions and wizard input validation.
pub use linter::{
    LintLevel, LintWarning, lint_content, lint_description, validate_name, validate_wizard_input,
};
/// Skill operating mode.
pub use mode::SkillMode;
/// Plugin marketplace discovery models and port.
pub use plugin_discovery::{
    MarketplaceManifest, NoopPluginDiscoverer, PluginDiscoveredSkill, PluginEntry,
    PluginMarketplaceDiscovery,
};
/// Profile model, diff algorithm, and persistence port.
pub use profile::{Phase, Profile, ProfileOp, ProfileStore, diff_profile};
/// Port for listing installed skills.
pub use repository::SkillRepository;
pub use security_scan::{
    DepEdge, DepGraph, DepNode, ImportChainFinding, ImportRef, ReferenceType, ScanCategory,
    ScanFinding, Severity, cross_reference, parse_import_refs, scan_skill,
};
/// Settings model and persistence port for `.claude/settings.json`.
pub use settings::{ProjectSettings, SettingsStore, SkillOverride};
/// Heuristic content scanner for dangerous patterns.
/// Port for ed25519 signature verification.
pub use signature::{NoopSignatureVerifier, SignatureVerifier, VerificationStatus};
/// Core skill model, agent enum, and scope.
pub use skill::{Agent, Scope, Skill};
/// Port for SSH-based remote machine management.
pub use ssh::{ConnectionStatus, NoopSshConnector, RemoteHost, RemoteSkill, SshConnector};
/// Port and types for git-backed skill sync.
pub use sync::{NoopSkillSync, SkillSync, Snapshot, SyncStatus};
/// Validation state enum describing a skill's health.
pub use validation::ValidationState;
