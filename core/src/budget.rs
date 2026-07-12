//! Context budget estimation for active skills.
//!
//! Claude Code reserves ~2 % of the context window for skill discovery
//! (~16 384 characters at an 8 k token window). This module calculates
//! how much of that budget each skill consumes and warns when the total
//! approaches or exceeds the limit.

use crate::Skill;
use serde::Serialize;

/// Default budget in characters (~2 % of 8 k tokens).
pub const DEFAULT_BUDGET_CHARS: usize = 16_384;

/// Warning threshold as a fraction of the budget.
pub const WARN_THRESHOLD: f64 = 0.80;

/// Critical threshold as a fraction of the budget.
pub const CRIT_THRESHOLD: f64 = 0.95;

/// Estimated character cost of a single skill.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkillCost {
    /// Skill name.
    pub name: String,
    /// Raw character count of the manifest content.
    pub char_count: usize,
    /// Estimated tokens (chars / 4, a rough heuristic).
    pub estimated_tokens: usize,
}

/// Aggregated budget state across all active skills.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContextBudget {
    /// Maximum allowed characters.
    pub limit: usize,
    /// Total characters consumed by all skills.
    pub used: usize,
    /// Remaining characters.
    pub available: usize,
    /// Usage as a fraction (0.0 – 1.0+).
    pub usage_ratio: f64,
    /// Per-skill breakdown.
    pub skill_costs: Vec<SkillCost>,
}

/// Warning level about budget exhaustion.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BudgetWarning {
    /// Budget usage is below the warning threshold.
    None,
    /// Usage is above 80 %.
    Approaching {
        /// Current usage percentage.
        pct: f64,
    },
    /// Usage is above 95 %.
    Critical {
        /// Current usage percentage.
        pct: f64,
    },
    /// Usage exceeds 100 % — skills will be silently truncated.
    OverBudget {
        /// Current usage percentage.
        pct: f64,
        /// Estimated number of skills that may be truncated.
        truncated_skills: usize,
    },
}

/// Estimates the character cost of a single skill.
///
/// Cost depends on the skill's operating mode:
/// - `Active`    → length of manifest content (if present) + length of agent names.
/// - `NameOnly`  → only the skill's name (plus a small formatting overhead).
/// - `Disabled`  → zero (the skill is off and consumes no budget).
pub fn estimate_skill_cost(skill: &Skill) -> SkillCost {
    let char_count = match skill.mode {
        crate::SkillMode::Active => {
            let content_len = skill.manifest_content.as_deref().map(str::len).unwrap_or(0);
            let agents_len: usize = skill.agents.iter().map(|a| a.len()).sum();
            content_len + agents_len
        }
        crate::SkillMode::NameOnly => {
            skill.name.len() + 20 // formatting overhead for the name-only entry
        }
        crate::SkillMode::Disabled => 0,
    };
    SkillCost {
        name: skill.name.clone(),
        char_count,
        estimated_tokens: char_count / 4,
    }
}

/// Calculates the aggregate budget across all skills.
pub fn calculate_budget(skills: &[Skill]) -> ContextBudget {
    let skill_costs: Vec<SkillCost> = skills.iter().map(estimate_skill_cost).collect();
    let used: usize = skill_costs.iter().map(|c| c.char_count).sum();
    let limit = DEFAULT_BUDGET_CHARS;
    let available = limit.saturating_sub(used);
    let usage_ratio = if limit > 0 {
        used as f64 / limit as f64
    } else {
        0.0
    };
    ContextBudget {
        limit,
        used,
        available,
        usage_ratio,
        skill_costs,
    }
}

/// Determines the warning level from a [`ContextBudget`].
pub fn classify_budget(budget: &ContextBudget) -> BudgetWarning {
    if budget.usage_ratio >= 1.0 {
        // Count skills that would need to be removed to get back under budget.
        let mut sorted: Vec<&SkillCost> = budget.skill_costs.iter().collect();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.char_count));
        let mut cum = 0usize;
        let mut cut = 0usize;
        // Count from largest to smallest until we are under budget (very rough).
        let over = budget.used.saturating_sub(budget.limit);
        for cost in &sorted {
            if cum >= over {
                break;
            }
            cum += cost.char_count;
            cut += 1;
        }
        BudgetWarning::OverBudget {
            pct: budget.usage_ratio * 100.0,
            truncated_skills: cut,
        }
    } else if budget.usage_ratio >= CRIT_THRESHOLD {
        BudgetWarning::Critical {
            pct: budget.usage_ratio * 100.0,
        }
    } else if budget.usage_ratio >= WARN_THRESHOLD {
        BudgetWarning::Approaching {
            pct: budget.usage_ratio * 100.0,
        }
    } else {
        BudgetWarning::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DriftState, Scope, SkillMode, ValidationState};
    use std::path::PathBuf;

    fn skill(name: &str, content: Option<&str>, agents: Vec<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: agents.into_iter().map(str::to_string).collect(),
            tags: vec![],
            managed: true,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: content.map(str::to_string),
            drift_state: DriftState::default(),
        }
    }

    #[test]
    fn empty_skill_cost_is_zero() {
        let s = skill("empty", None, vec![]);
        let cost = estimate_skill_cost(&s);
        assert_eq!(cost.char_count, 0);
        assert_eq!(cost.estimated_tokens, 0);
    }

    #[test]
    fn skill_cost_counts_content_and_agents() {
        let s = skill("test", Some("hello world"), vec!["claude"]);
        let cost = estimate_skill_cost(&s);
        assert_eq!(cost.char_count, 17); // "hello world" (11) + "claude" (6)
    }

    #[test]
    fn skill_cost_rounds_down_tokens() {
        let s = skill("t", Some("abcd"), vec![]);
        let cost = estimate_skill_cost(&s);
        assert_eq!(cost.estimated_tokens, 1);
    }

    #[test]
    fn calculate_budget_on_empty_list() {
        let budget = calculate_budget(&[]);
        assert_eq!(budget.used, 0);
        assert_eq!(budget.limit, DEFAULT_BUDGET_CHARS);
        assert_eq!(budget.available, DEFAULT_BUDGET_CHARS);
    }

    #[test]
    fn calculate_budget_sums_multiple_skills() {
        let skills = vec![
            skill("a", Some("ten chars!"), vec![]),
            skill("b", Some("eleven chars"), vec![]),
        ];
        let budget = calculate_budget(&skills);
        assert_eq!(budget.used, 22); // "ten chars!"(10) + "eleven chars"(12)
        assert_eq!(budget.skill_costs.len(), 2);
    }

    #[test]
    fn classify_none_when_under_warn_threshold() {
        let budget = ContextBudget {
            limit: 100,
            used: 50,
            available: 50,
            usage_ratio: 0.5,
            skill_costs: vec![],
        };
        assert_eq!(classify_budget(&budget), BudgetWarning::None);
    }

    #[test]
    fn classify_approaching_at_80_pct() {
        let budget = ContextBudget {
            limit: 100,
            used: 80,
            available: 20,
            usage_ratio: 0.8,
            skill_costs: vec![],
        };
        assert_eq!(
            classify_budget(&budget),
            BudgetWarning::Approaching { pct: 80.0 }
        );
    }

    #[test]
    fn classify_critical_at_95_pct() {
        let budget = ContextBudget {
            limit: 100,
            used: 95,
            available: 5,
            usage_ratio: 0.95,
            skill_costs: vec![],
        };
        assert_eq!(
            classify_budget(&budget),
            BudgetWarning::Critical { pct: 95.0 }
        );
    }

    #[test]
    fn classify_over_budget_when_exceeded() {
        let costs = vec![
            SkillCost {
                name: "big".into(),
                char_count: 80,
                estimated_tokens: 20,
            },
            SkillCost {
                name: "small".into(),
                char_count: 40,
                estimated_tokens: 10,
            },
        ];
        let budget = ContextBudget {
            limit: 100,
            used: 120,
            available: 0,
            usage_ratio: 1.2,
            skill_costs: costs,
        };
        let warning = classify_budget(&budget);
        assert!(matches!(warning, BudgetWarning::OverBudget { .. }));
        if let BudgetWarning::OverBudget {
            truncated_skills, ..
        } = warning
        {
            // The big skill (80 chars) alone covers the 20-char overage.
            assert!(truncated_skills >= 1);
        }
    }

    #[test]
    fn estimate_skill_cost_returns_skill_cost() {
        let s = skill("name", Some("content"), vec!["agent1", "agent2"]);
        let cost = estimate_skill_cost(&s);
        assert_eq!(cost.name, "name");
        assert_eq!(cost.char_count, 19); // "content"(7) + "agent1"(6) + "agent2"(6)
        assert_eq!(cost.estimated_tokens, 4);
    }

    #[test]
    fn name_only_skill_only_counts_name() {
        let mut s = skill(
            "my-skill",
            Some("very long description that would normally cost a lot"),
            vec!["claude", "codex"],
        );
        s.mode = SkillMode::NameOnly;
        let cost = estimate_skill_cost(&s);
        assert_eq!(cost.name, "my-skill");
        assert_eq!(cost.char_count, 28); // "my-skill"(8) + overhead(20)
    }

    #[test]
    fn disabled_skill_costs_zero() {
        let mut s = skill("off", Some("content"), vec!["claude"]);
        s.mode = SkillMode::Disabled;
        let cost = estimate_skill_cost(&s);
        assert_eq!(cost.char_count, 0);
        assert_eq!(cost.estimated_tokens, 0);
    }
}
