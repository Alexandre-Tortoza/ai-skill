//! Adapter that creates skill directories and writes manifest files.

use std::path::PathBuf;

use ai_skill_core::{SkillCreator, SkillWriter, scaffold_skill};

fn home_dir() -> Result<PathBuf, std::io::Error> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME is not set; set HOME or run ai-skill from a login shell",
        )
    })
}

/// Creates a skill directory with a scaffolded SKILL.md in `~/.claude/skills/` (or custom root).
pub struct FsSkillCreator {
    /// Base directory where skill subdirectories are created.
    pub base_dir: PathBuf,
}

impl FsSkillCreator {
    /// Resolves the base directory from `$HOME`.
    pub fn from_env() -> Result<Self, std::io::Error> {
        Ok(Self {
            base_dir: home_dir()?.join(".claude").join("skills"),
        })
    }

    /// Creates a creator with an explicit base directory.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl SkillCreator for FsSkillCreator {
    fn create(
        &self,
        name: &str,
        agents: &[String],
        tags: &[String],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let skill_dir = self.base_dir.join(name);
        std::fs::create_dir_all(&skill_dir)?;
        let manifest_path = skill_dir.join("SKILL.md");
        let content = scaffold_skill(name, agents, tags);
        std::fs::write(&manifest_path, content)?;
        Ok(skill_dir)
    }
}

/// Writes raw content to a file path (implements [`SkillWriter`]).
pub struct FsSkillWriter;

impl SkillWriter for FsSkillWriter {
    fn write(
        &self,
        path: &std::path::Path,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::parse_frontmatter;
    use tempfile::TempDir;

    #[test]
    fn create_writes_skill_md_in_base_dir() {
        let tmp = TempDir::new().unwrap();
        let creator = FsSkillCreator::new(tmp.path().to_path_buf());
        let path = creator.create("my-skill", &[], &[]).unwrap();
        assert!(path.join("SKILL.md").exists());
    }

    #[test]
    fn created_skill_has_correct_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let creator = FsSkillCreator::new(tmp.path().to_path_buf());
        let agents = vec!["claude".to_string()];
        creator.create("test-skill", &agents, &[]).unwrap();

        let content =
            std::fs::read_to_string(tmp.path().join("test-skill").join("SKILL.md")).unwrap();
        let meta = parse_frontmatter(&content).unwrap();
        assert_eq!(meta.name, "test-skill");
        assert_eq!(meta.agents, vec!["claude"]);
    }

    #[test]
    fn create_returns_path_to_skill_directory() {
        let tmp = TempDir::new().unwrap();
        let creator = FsSkillCreator::new(tmp.path().to_path_buf());
        let path = creator.create("new-skill", &[], &[]).unwrap();
        assert_eq!(path, tmp.path().join("new-skill"));
    }

    #[test]
    #[ignore = "requires ~/.claude/skills directory to exist"]
    fn from_env_creates_in_home_dir() {
        let creator = FsSkillCreator::from_env().unwrap();
        let path = creator.create("__test-wave4__", &[], &[]).unwrap();
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn fs_skill_writer_writes_content() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("SKILL.md");
        let writer = FsSkillWriter;
        writer.write(&path, "content").unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "content");
    }
}
