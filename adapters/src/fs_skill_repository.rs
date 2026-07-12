//! Filesystem-based [`SkillRepository`] that scans `~/.claude/skills/` directories.

use ai_skill_core::{
    Agent, DriftState, Scope, Skill, SkillMode, SkillRepository, ValidationState,
    detect_duplicates, extract_body, parse_frontmatter,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when scanning the skill repository.
#[derive(Error, Debug)]
pub enum RepositoryError {
    /// The `HOME` environment variable is not set.
    #[error("HOME is not set; set HOME or run ai-skill from a login shell")]
    MissingHome,
    /// A filesystem operation failed.
    #[error("filesystem error while reading skills: {0}")]
    Io(#[from] std::io::Error),
}

/// Scans known agent skill directories for installed skills.
pub struct FsSkillRepository {
    global_root: PathBuf,
    project_root: Option<PathBuf>,
    extra_roots: Vec<(PathBuf, String)>,
}

impl FsSkillRepository {
    /// Creates a repository with explicit global and optional project roots.
    pub fn new(global_root: PathBuf, project_root: Option<PathBuf>) -> Self {
        Self {
            global_root,
            project_root,
            extra_roots: vec![],
        }
    }

    /// Resolves roots from `$HOME` and the current working directory,
    /// including all known claude-compatible agent directories.
    pub fn from_env() -> Result<Self, RepositoryError> {
        let home = std::env::var("HOME").map_err(|_| RepositoryError::MissingHome)?;
        let home_path = PathBuf::from(&home);
        let global_root = home_path.join(".claude").join("skills");

        let cwd = std::env::current_dir()?;
        let project_candidate = cwd.join(".claude").join("skills");
        let project_root = if project_candidate.is_dir() {
            Some(project_candidate)
        } else {
            None
        };

        let mut extra = Vec::new();
        for agent in Agent::claude_compatible() {
            if agent == Agent::ClaudeCode {
                continue;
            }
            if let Some(dir) = agent.home_skills_dir(&home_path) {
                extra.push((dir, agent.label().to_string()));
            }
        }

        Ok(Self {
            global_root,
            project_root,
            extra_roots: extra,
        })
    }

    fn scan_root(
        &self,
        root: &PathBuf,
        scope: Scope,
        agent_name: &str,
    ) -> Result<Vec<Skill>, RepositoryError> {
        if !root.is_dir() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        for entry in std::fs::read_dir(root)? {
            let entry = entry?;
            let entry_path = entry.path();

            let dir_name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();

            if let Some(base) = dir_name.strip_suffix(".disabled") {
                if entry_path.is_dir() {
                    skills.push(Skill {
                        name: base.to_string(),
                        path: entry_path.clone(),
                        scope: scope.clone(),
                        agents: vec![agent_name.to_string()],
                        tags: vec![],
                        managed: false,
                        mode: SkillMode::Disabled,
                        validation: ValidationState::Valid,
                        manifest_content: None,
                        drift_state: DriftState::default(),
                    });
                }
                continue;
            }

            let canonical = match std::fs::canonicalize(&entry_path) {
                Ok(p) => p,
                Err(_) => {
                    skills.push(Skill {
                        name: dir_name,
                        path: entry_path,
                        scope: scope.clone(),
                        agents: vec![agent_name.to_string()],
                        tags: vec![],
                        managed: false,
                        mode: SkillMode::Active,
                        validation: ValidationState::BrokenSymlink,
                        manifest_content: None,
                        drift_state: DriftState::default(),
                    });
                    continue;
                }
            };

            if !canonical.is_dir() {
                continue;
            }

            let manifest_path = canonical.join("SKILL.md");
            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(_) => {
                    skills.push(Skill {
                        name: dir_name,
                        path: canonical,
                        scope: scope.clone(),
                        agents: vec![agent_name.to_string()],
                        tags: vec![],
                        managed: false,
                        mode: SkillMode::Active,
                        validation: ValidationState::MissingManifest,
                        manifest_content: None,
                        drift_state: DriftState::default(),
                    });
                    continue;
                }
            };

            let metadata = match parse_frontmatter(&content) {
                Ok(m) => m,
                Err(e) => {
                    skills.push(Skill {
                        name: dir_name,
                        path: canonical,
                        scope: scope.clone(),
                        agents: vec![agent_name.to_string()],
                        tags: vec![],
                        managed: false,
                        mode: SkillMode::Active,
                        validation: ValidationState::InvalidFrontmatter {
                            reason: e.to_string(),
                        },
                        manifest_content: None,
                        drift_state: DriftState::default(),
                    });
                    continue;
                }
            };

            let validation = if has_orphan_lock(&canonical) {
                ValidationState::OrphanLock
            } else {
                ValidationState::Valid
            };

            let mode = detect_mode(&canonical);
            let manifest_content = extract_body(&content).map(str::to_owned);

            let mut agents = metadata.agents;
            let agent_label = agent_name.to_string();
            if !agents.contains(&agent_label) {
                agents.push(agent_label);
            }

            skills.push(Skill {
                name: metadata.name,
                path: canonical.clone(),
                scope: scope.clone(),
                agents,
                tags: metadata.tags,
                managed: canonical.join(".ai-skill").exists(),
                mode,
                validation,
                manifest_content,
                drift_state: DriftState::default(),
            });
        }

        Ok(skills)
    }
}

/// Detects the skill's operating mode from filesystem markers.
fn detect_mode(path: &Path) -> SkillMode {
    if path.join(".name-only").exists() {
        SkillMode::NameOnly
    } else {
        SkillMode::Active
    }
}

fn has_orphan_lock(_canonical: &Path) -> bool {
    // TODO(H1.3-orphan-lock): define lockfile format in Wave 2
    false
}

impl SkillRepository for FsSkillRepository {
    type Error = RepositoryError;

    fn list(&self) -> Result<Vec<Skill>, RepositoryError> {
        let mut skills = self.scan_root(&self.global_root, Scope::Global, "Claude Code")?;

        if let Some(ref project_root) = self.project_root {
            skills.extend(self.scan_root(project_root, Scope::Project, "Claude Code")?);
        }

        for (dir, agent_name) in &self.extra_roots {
            skills.extend(self.scan_root(dir, Scope::Global, agent_name)?);
        }

        let dups = detect_duplicates(&skills);
        for (idx, conflicts_with) in dups {
            skills[idx].validation = ValidationState::Duplicate { conflicts_with };
        }

        Ok(skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::SkillRepository;
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    fn write_skill(dir: &Path, name: &str, agents: &[&str]) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        let agents_yaml = if agents.is_empty() {
            String::new()
        } else {
            format!(
                "agents:\n{}\n",
                agents
                    .iter()
                    .map(|a| format!("  - {a}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };
        let content = format!("---\nname: {name}\n{agents_yaml}---\n# {name} body\n");
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn new_stores_roots() {
        let repo = FsSkillRepository::new(PathBuf::from("/global"), Some(PathBuf::from("/proj")));
        assert_eq!(repo.global_root, PathBuf::from("/global"));
        assert_eq!(repo.project_root, Some(PathBuf::from("/proj")));
    }

    #[test]
    fn valid_skill_has_valid_state() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "alpha", &[]);

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].validation, ValidationState::Valid);
        assert_eq!(skills[0].name, "alpha");
    }

    #[test]
    fn valid_skill_has_manifest_content() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "alpha", &[]);

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert!(skills[0].manifest_content.is_some());
        assert!(
            skills[0]
                .manifest_content
                .as_ref()
                .unwrap()
                .contains("alpha body")
        );
    }

    #[test]
    fn broken_symlink_entry_has_broken_symlink_state() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        fs::create_dir_all(&global).unwrap();

        // symlink pointing to non-existent target
        symlink("/nonexistent/target", global.join("dead-link")).unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].validation, ValidationState::BrokenSymlink);
        assert_eq!(skills[0].name, "dead-link");
    }

    #[test]
    fn dir_without_manifest_has_missing_manifest_state() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        let skill_dir = global.join("no-manifest");
        fs::create_dir_all(&skill_dir).unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].validation, ValidationState::MissingManifest);
        assert_eq!(skills[0].name, "no-manifest");
    }

    #[test]
    fn malformed_manifest_has_invalid_frontmatter_state() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        let skill_dir = global.join("bad-fm");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: [unclosed\n---\n").unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert!(matches!(
            &skills[0].validation,
            ValidationState::InvalidFrontmatter { .. }
        ));
    }

    #[test]
    fn duplicate_skills_get_duplicate_state() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        let project = tmp.path().join("project");
        write_skill(&global, "shared", &[]);
        write_skill(&project, "shared", &[]);

        let repo = FsSkillRepository::new(global, Some(project));
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 2);
        let first = skills.iter().find(|s| s.scope == Scope::Global).unwrap();
        let second = skills.iter().find(|s| s.scope == Scope::Project).unwrap();
        assert_eq!(first.validation, ValidationState::Valid);
        assert!(matches!(
            second.validation,
            ValidationState::Duplicate { .. }
        ));
    }

    #[test]
    fn lists_skills_from_both_roots_with_correct_scope() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        let project = tmp.path().join("project");
        write_skill(&global, "alpha", &[]);
        write_skill(&project, "beta", &["claude"]);

        let repo = FsSkillRepository::new(global, Some(project));
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 2);
        let alpha = skills.iter().find(|s| s.name == "alpha").unwrap();
        let beta = skills.iter().find(|s| s.name == "beta").unwrap();
        assert_eq!(alpha.scope, Scope::Global);
        assert_eq!(beta.scope, Scope::Project);
        assert_eq!(beta.agents, vec!["claude", "Claude Code"]);
    }

    #[test]
    fn symlink_resolves_to_canonical_path() {
        let tmp = TempDir::new().unwrap();
        let real_dir = tmp.path().join("real");
        write_skill(&real_dir, "linked", &[]);

        let global = tmp.path().join("global");
        fs::create_dir_all(&global).unwrap();
        symlink(real_dir.join("linked"), global.join("linked")).unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "linked");
        assert_eq!(
            skills[0].path,
            fs::canonicalize(real_dir.join("linked")).unwrap()
        );
    }

    #[test]
    fn no_project_root_returns_only_global() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "only-global", &[]);

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].scope, Scope::Global);
    }

    #[test]
    fn empty_root_returns_empty_vec() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        fs::create_dir_all(&global).unwrap();

        let repo = FsSkillRepository::new(global, None);
        assert!(repo.list().unwrap().is_empty());
    }

    #[test]
    fn nonexistent_root_returns_empty_vec() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("does-not-exist");

        let repo = FsSkillRepository::new(global, None);
        assert!(repo.list().unwrap().is_empty());
    }

    #[test]
    fn disabled_dir_gets_disabled_mode() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        fs::create_dir_all(global.join("alpha.disabled")).unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].mode, SkillMode::Disabled);
        assert_eq!(skills[0].validation, ValidationState::Valid);
        assert_eq!(skills[0].name, "alpha");
    }

    #[test]
    fn name_only_marker_sets_name_only_mode() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "collapsed", &["claude"]);
        std::fs::write(global.join("collapsed").join(".name-only"), "").unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].mode, SkillMode::NameOnly);
        assert_eq!(skills[0].name, "collapsed");
        assert!(skills[0].manifest_content.is_some());
    }

    #[test]
    fn skill_without_name_only_marker_is_active() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "full", &[]);

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].mode, SkillMode::Active);
    }

    #[test]
    fn ai_skill_marker_sets_managed_true() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "managed-skill", &[]);
        fs::write(global.join("managed-skill").join(".ai-skill"), "").unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills.len(), 1);
        assert!(skills[0].managed);
    }

    #[test]
    fn skill_without_marker_is_not_managed() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "unmanaged", &[]);

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert!(!skills[0].managed);
    }

    #[test]
    fn tags_from_frontmatter_are_loaded() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        let skill_dir = global.join("tagged");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: tagged\ntags:\n  - git\n  - productivity\n---\n# body\n",
        )
        .unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();

        assert_eq!(skills[0].tags, vec!["git", "productivity"]);
    }

    #[test]
    fn trait_object_usage_confirms_dip() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        write_skill(&global, "dip-skill", &[]);

        let repo: Box<dyn SkillRepository<Error = RepositoryError>> =
            Box::new(FsSkillRepository::new(global, None));
        assert_eq!(repo.list().unwrap().len(), 1);
    }

    #[test]
    fn from_env_resolves_home_directory() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join(".claude").join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        write_skill(&skills_dir, "my-skill", &["claude"]);

        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", tmp.path());
        }
        let repo = FsSkillRepository::from_env().unwrap();
        if let Some(h) = original_home {
            unsafe {
                std::env::set_var("HOME", h);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }

        let skills = repo.list().unwrap();
        assert!(!skills.is_empty());
        assert_eq!(skills[0].name, "my-skill");
    }

    #[test]
    fn repository_error_io_construction() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = RepositoryError::Io(io_err);
        assert!(
            err.to_string()
                .contains("filesystem error while reading skills")
        );
    }

    #[test]
    fn repository_error_debug() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = RepositoryError::Io(io_err);
        assert!(!format!("{err:?}").is_empty());
    }

    #[test]
    fn list_skips_non_directory_entries() {
        let tmp = TempDir::new().unwrap();
        let global = tmp.path().join("global");
        fs::create_dir_all(&global).unwrap();
        fs::write(global.join("not-a-dir"), "not a skill").unwrap();

        let repo = FsSkillRepository::new(global, None);
        let skills = repo.list().unwrap();
        assert!(skills.is_empty());
    }
}
