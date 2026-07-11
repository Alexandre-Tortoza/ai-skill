//! Port for reading installed skills from the filesystem.

use crate::Skill;

/// Provides the set of installed skills.
pub trait SkillRepository {
    type Error: std::error::Error;

    /// Returns all known skills.
    fn list(&self) -> Result<Vec<Skill>, Self::Error>;
}
