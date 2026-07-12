//! Port for integrating with external security scanners (Socket, Snyk, Semgrep).
//!
//! The concrete adapter calls the skills.sh audit API (`/api/v1/skills/audit`)
//! which requires OIDC auth. Until that route is available, use [`NoopExternalScanner`].

use crate::{ScanCategory, ScanFinding, Severity};

/// A single finding reported by an external scanner.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFinding {
    /// Which scanner produced this finding (e.g., "socket", "snyk", "semgrep").
    pub scanner: String,
    /// Severity level.
    pub severity: Severity,
    /// Human-readable description.
    pub detail: String,
    /// Optional package or dependency name.
    pub package: Option<String>,
}

impl From<ExternalFinding> for ScanFinding {
    fn from(f: ExternalFinding) -> Self {
        let detail = match &f.package {
            Some(pkg) => format!("[{}] {} ({})", f.scanner, f.detail, pkg),
            None => format!("[{}] {}", f.scanner, f.detail),
        };
        ScanFinding {
            severity: f.severity,
            category: ScanCategory::ExternalScanner,
            detail,
            line: 0,
        }
    }
}

/// Port for querying external security scanners about a skill.
pub trait ExternalScanner {
    /// Scans a skill by name and returns findings from external scanners.
    fn scan(&self, skill_name: &str) -> Result<Vec<ExternalFinding>, Box<dyn std::error::Error>>;
}

/// A no-op external scanner that always returns empty results.
///
/// Used when the audit API is not available or not configured.
pub struct NoopExternalScanner;

impl ExternalScanner for NoopExternalScanner {
    fn scan(&self, _skill_name: &str) -> Result<Vec<ExternalFinding>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_scanner_returns_empty() {
        let scanner = NoopExternalScanner;
        let results = scanner.scan("any-skill").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn external_finding_converts_to_scan_finding_with_package() {
        let ef = ExternalFinding {
            scanner: "socket".to_string(),
            severity: Severity::High,
            detail: "Malicious dependency detected".to_string(),
            package: Some("evil-pkg".to_string()),
        };
        let sf: ScanFinding = ef.into();
        assert_eq!(sf.category, ScanCategory::ExternalScanner);
        assert_eq!(sf.severity, Severity::High);
        assert!(sf.detail.contains("socket"));
        assert!(sf.detail.contains("evil-pkg"));
    }

    #[test]
    fn external_finding_converts_without_package() {
        let ef = ExternalFinding {
            scanner: "snyk".to_string(),
            severity: Severity::Medium,
            detail: "Vulnerable dependency".to_string(),
            package: None,
        };
        let sf: ScanFinding = ef.into();
        assert_eq!(sf.category, ScanCategory::ExternalScanner);
        assert!(!sf.detail.contains('('));
    }

    #[test]
    fn external_scanner_trait_object_works() {
        let scanner: Box<dyn ExternalScanner> = Box::new(NoopExternalScanner);
        let results = scanner.scan("test").unwrap();
        assert!(results.is_empty());
    }
}
