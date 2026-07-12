//! Case-insensitive duplicate name detection.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::Skill;

/// Returns `(index, path_of_first_occurrence)` for each entry whose name
/// (case-insensitive) already appeared earlier in the slice.
///
/// The first occurrence is NOT included in the result.
pub fn detect_duplicates(skills: &[Skill]) -> Vec<(usize, PathBuf)> {
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    let mut duplicates = Vec::new();

    for (idx, skill) in skills.iter().enumerate() {
        let key = skill.name.to_lowercase();
        match seen.get(&key) {
            Some(first_path) => duplicates.push((idx, first_path.clone())),
            None => {
                seen.insert(key, skill.path.clone());
            }
        }
    }

    duplicates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Scope, SkillMode, ValidationState};

    fn skill(name: &str, path: &str) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(path),
            scope: Scope::Global,
            agents: vec![],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: crate::DriftState::default(),
        }
    }

    #[test]
    fn no_duplicates_returns_empty() {
        let skills = vec![skill("alpha", "/a"), skill("beta", "/b")];
        assert!(detect_duplicates(&skills).is_empty());
    }

    #[test]
    fn two_skills_same_name_returns_second_index() {
        let skills = vec![skill("alpha", "/a"), skill("alpha", "/b")];
        let dups = detect_duplicates(&skills);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].0, 1);
        assert_eq!(dups[0].1, PathBuf::from("/a"));
    }

    #[test]
    fn detection_is_case_insensitive() {
        let skills = vec![skill("Alpha", "/a"), skill("alpha", "/b")];
        let dups = detect_duplicates(&skills);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].0, 1);
    }

    #[test]
    fn cross_scope_duplicates_are_detected() {
        let skills = vec![
            Skill {
                name: "shared".to_string(),
                path: PathBuf::from("/global/shared"),
                scope: Scope::Global,
                agents: vec![],
                tags: vec![],
                managed: false,
                mode: SkillMode::Active,
                validation: ValidationState::Valid,
                manifest_content: None,
                drift_state: crate::DriftState::default(),
            },
            Skill {
                name: "shared".to_string(),
                path: PathBuf::from("/project/shared"),
                scope: Scope::Project,
                agents: vec![],
                tags: vec![],
                managed: false,
                mode: SkillMode::Active,
                validation: ValidationState::Valid,
                manifest_content: None,
                drift_state: crate::DriftState::default(),
            },
        ];
        let dups = detect_duplicates(&skills);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].0, 1);
    }

    #[test]
    fn first_occurrence_is_not_flagged() {
        let skills = vec![skill("alpha", "/a"), skill("alpha", "/b")];
        let dups = detect_duplicates(&skills);
        assert!(dups.iter().all(|(idx, _)| *idx != 0));
    }

    #[test]
    fn three_of_same_name_flags_second_and_third() {
        let skills = vec![skill("x", "/1"), skill("x", "/2"), skill("x", "/3")];
        let dups = detect_duplicates(&skills);
        assert_eq!(dups.len(), 2);
        let flagged: Vec<usize> = dups.iter().map(|(i, _)| *i).collect();
        assert!(flagged.contains(&1));
        assert!(flagged.contains(&2));
    }
}
