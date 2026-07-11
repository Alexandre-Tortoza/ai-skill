//! Domain types, ports, and pure functions for ai-skill.
//!
//! This crate defines the hexagonal-core boundary: skill model, repository/catalog/installer
//! ports, frontmatter parsing, security scanning, profile diffing, drift detection, audit,
//! duplicate detection, and skill scaffolding.

pub mod audit;
pub mod catalog;
pub mod creator;
pub mod drift;
pub mod duplicate_detector;
pub mod frontmatter;
pub mod installer;
pub mod profile;
pub mod repository;
pub mod security_scan;
pub mod skill;
pub mod validation;

/// Report grouping skills by their health category.
pub use audit::{AuditReport, audit_skills};
/// Port for querying a remote skill catalog.
pub use catalog::{AnyCatalogGateway, CatalogEntry};
/// Ports and helpers for creating and editing skill manifests.
pub use creator::{SkillCreator, SkillWriter, apply_edit, scaffold_skill};
/// Port and state enum for detecting upstream drift.
pub use drift::{DriftChecker, DriftState};
/// Case-insensitive duplicate name detection across a skill slice.
pub use duplicate_detector::detect_duplicates;
/// Frontmatter (`---` delimited YAML) parsing and body extraction.
pub use frontmatter::{ParseError, SkillMetadata, extract_body, parse_frontmatter};
/// Ports for installing, removing, and updating skills via external tooling.
pub use installer::{SkillInstaller, SkillToggler};
/// Profile model, diff algorithm, and persistence port.
pub use profile::{Profile, ProfileOp, ProfileStore, diff_profile};
/// Port for listing installed skills.
pub use repository::SkillRepository;
/// Heuristic content scanner for dangerous patterns.
pub use security_scan::{ScanCategory, ScanFinding, Severity, scan_skill};
/// Core skill model and scope enum.
pub use skill::{Scope, Skill};
/// Validation state enum describing a skill's health.
pub use validation::ValidationState;
