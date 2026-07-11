//! Operating mode for a skill — active, name-only, or disabled.

use serde::Serialize;

/// User-chosen operating mode for a skill.
///
/// These three modes mirror the `skillOverrides` flags in Claude Code 2.1:
/// - `Active`         → `{}` (no override — full description in budget)
/// - `NameOnly`       → `{"nameOnly": true}` (only the name counts toward the budget)
/// - `Disabled`       → `{"disabled": true}` (completely off)
///
/// The fourth combination (`{"nameOnly": true, "disabled": true}`) occurs when
/// a name-only skill is later disabled; the `.name-only` marker is preserved
/// inside the `.disabled` directory so the preference survives re-enable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillMode {
    /// Fully enabled — full description counts toward the context budget.
    Active,
    /// Enabled but collapsed — only the name counts toward the budget.
    NameOnly,
    /// Completely disabled — no budget consumption.
    Disabled,
}

impl SkillMode {
    /// Returns `true` if the skill is in any enabled mode.
    pub fn is_enabled(&self) -> bool {
        matches!(self, SkillMode::Active | SkillMode::NameOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_is_enabled() {
        assert!(SkillMode::Active.is_enabled());
    }

    #[test]
    fn name_only_is_enabled() {
        assert!(SkillMode::NameOnly.is_enabled());
    }

    #[test]
    fn disabled_is_not_enabled() {
        assert!(!SkillMode::Disabled.is_enabled());
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(SkillMode::Active, SkillMode::NameOnly);
        assert_ne!(SkillMode::Active, SkillMode::Disabled);
        assert_ne!(SkillMode::NameOnly, SkillMode::Disabled);
    }
}
