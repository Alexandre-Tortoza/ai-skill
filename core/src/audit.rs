//! Audit report that groups skills by health category.

use crate::{DriftState, Skill, ValidationState};

/// A snapshot of skill health, grouping them into actionable categories.
pub struct AuditReport<'a> {
    /// Skills with broken symlinks, missing manifests, invalid frontmatter, or orphan locks.
    pub broken: Vec<&'a Skill>,
    /// Skills whose name conflicts with another installed skill.
    pub duplicates: Vec<&'a Skill>,
    /// Valid or disabled skills that have no agent assignments.
    pub no_agents: Vec<&'a Skill>,
    /// Skills where an upstream update is available.
    pub update_available: Vec<&'a Skill>,
}

/// Produces an [`AuditReport`] from a slice of skills.
pub fn audit_skills(skills: &[Skill]) -> AuditReport<'_> {
    let mut broken = Vec::new();
    let mut duplicates = Vec::new();
    let mut no_agents = Vec::new();
    let mut update_available = Vec::new();

    for skill in skills {
        match skill.validation {
            ValidationState::BrokenSymlink
            | ValidationState::MissingManifest
            | ValidationState::InvalidFrontmatter { .. }
            | ValidationState::OrphanLock => {
                broken.push(skill);
            }
            ValidationState::Duplicate { .. } => {
                duplicates.push(skill);
            }
            ValidationState::Valid | ValidationState::Disabled => {
                if skill.agents.is_empty() {
                    no_agents.push(skill);
                }
            }
        }

        if matches!(skill.drift_state, DriftState::UpdateAvailable { .. }) {
            update_available.push(skill);
        }
    }

    AuditReport {
        broken,
        duplicates,
        no_agents,
        update_available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DriftState, Scope, ValidationState};
    use std::path::PathBuf;

    fn valid_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: DriftState::default(),
        }
    }

    #[test]
    fn valid_skill_with_agents_absent_from_all_lists() {
        let skills = vec![valid_skill("alpha")];
        let report = audit_skills(&skills);
        assert!(report.broken.is_empty());
        assert!(report.duplicates.is_empty());
        assert!(report.no_agents.is_empty());
        assert!(report.update_available.is_empty());
    }

    #[test]
    fn broken_symlink_appears_in_broken() {
        let skills = vec![Skill {
            validation: ValidationState::BrokenSymlink,
            ..valid_skill("broken")
        }];
        let report = audit_skills(&skills);
        assert_eq!(report.broken.len(), 1);
        assert!(report.duplicates.is_empty());
    }

    #[test]
    fn duplicate_appears_in_duplicates_not_broken() {
        let skills = vec![Skill {
            validation: ValidationState::Duplicate {
                conflicts_with: PathBuf::from("/tmp/other"),
            },
            ..valid_skill("dup")
        }];
        let report = audit_skills(&skills);
        assert_eq!(report.duplicates.len(), 1);
        assert!(report.broken.is_empty());
    }

    #[test]
    fn valid_skill_without_agents_appears_in_no_agents() {
        let skills = vec![Skill {
            agents: vec![],
            ..valid_skill("lonely")
        }];
        let report = audit_skills(&skills);
        assert_eq!(report.no_agents.len(), 1);
        assert!(report.broken.is_empty());
    }

    #[test]
    fn update_available_appears_in_update_available() {
        let skills = vec![Skill {
            drift_state: DriftState::UpdateAvailable {
                local_hash: "abc".into(),
                upstream_hash: "def".into(),
            },
            ..valid_skill("stale")
        }];
        let report = audit_skills(&skills);
        assert_eq!(report.update_available.len(), 1);
    }

    #[test]
    fn mixed_skills_each_in_correct_list() {
        let skills = vec![
            valid_skill("good"),
            Skill {
                validation: ValidationState::BrokenSymlink,
                ..valid_skill("b1")
            },
            Skill {
                validation: ValidationState::Duplicate {
                    conflicts_with: PathBuf::from("/tmp/x"),
                },
                ..valid_skill("d1")
            },
            Skill {
                agents: vec![],
                ..valid_skill("n1")
            },
            Skill {
                drift_state: DriftState::UpdateAvailable {
                    local_hash: "a".into(),
                    upstream_hash: "b".into(),
                },
                ..valid_skill("u1")
            },
        ];
        let report = audit_skills(&skills);
        assert_eq!(report.broken.len(), 1);
        assert_eq!(report.duplicates.len(), 1);
        assert_eq!(report.no_agents.len(), 1);
        assert_eq!(report.update_available.len(), 1);
    }
}
