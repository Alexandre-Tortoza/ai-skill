//! Skill usage analytics derived from local agent history.

use std::time::{Duration, SystemTime};

/// A single observed skill invocation in an agent's local history.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillUsageEvent {
    /// Name of the skill that was invoked.
    pub skill_name: String,
    /// When the invocation was recorded (proxy: transcript file mtime).
    pub timestamp: SystemTime,
}

/// Aggregated usage record for a single skill.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SkillUsageRecord {
    /// Skill name this record describes.
    pub skill_name: String,
    /// Number of observed invocations.
    pub count: u64,
    /// Most recent observed invocation time, if any.
    pub last_used: Option<SystemTime>,
}

impl SkillUsageRecord {
    /// Registers one invocation at `timestamp`, keeping the newest time.
    pub fn observe(&mut self, timestamp: SystemTime) {
        self.count += 1;
        self.last_used = Some(match self.last_used {
            Some(prev) if prev >= timestamp => prev,
            _ => timestamp,
        });
    }
}

/// Aggregated usage analytics across all installed skills.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UsageReport {
    /// Per-skill usage records, one per known skill.
    pub records: Vec<SkillUsageRecord>,
    /// Names of skills with zero observed usage ("dead skills").
    pub dead: Vec<String>,
    /// Names of skills unused for longer than the stale threshold.
    pub stale: Vec<String>,
    /// Stale threshold in days used to compute [`UsageReport::stale`].
    pub stale_after_days: u64,
}

impl UsageReport {
    /// Returns the record for `skill_name`, if present.
    pub fn record(&self, skill_name: &str) -> Option<&SkillUsageRecord> {
        self.records.iter().find(|r| r.skill_name == skill_name)
    }
}

/// Port for reading observed skill usage from local agent histories.
pub trait SkillUsageReader {
    /// Returns observed usage events across all known agent histories.
    ///
    /// Implementations should ignore missing history directories and skip
    /// unreadable files; only truly unexpected errors are surfaced.
    fn read_events(&self) -> Result<Vec<SkillUsageEvent>, UsageError>;
}

/// Errors that can occur while reading usage history.
#[derive(Debug, thiserror::Error)]
pub enum UsageError {
    /// A filesystem operation failed unexpectedly.
    #[error("failed to read usage history: {0}")]
    Io(#[from] std::io::Error),
}

/// Number of seconds in one day.
const SECONDS_PER_DAY: u64 = 86_400;

/// Builds an aggregated [`UsageReport`] from raw events.
///
/// Every name in `skill_names` gets a record (with zero usage when absent),
/// so "dead" skills are reported even when no history exists at all.
pub fn build_usage_report(
    events: &[SkillUsageEvent],
    skill_names: &[String],
    stale_after_days: u64,
) -> UsageReport {
    build_usage_report_as_of(events, skill_names, stale_after_days, SystemTime::now())
}

/// Like [`build_usage_report`] but with an injectable `now`, for deterministic tests.
pub fn build_usage_report_as_of(
    events: &[SkillUsageEvent],
    skill_names: &[String],
    stale_after_days: u64,
    now: SystemTime,
) -> UsageReport {
    let mut records: Vec<SkillUsageRecord> = skill_names
        .iter()
        .map(|name| SkillUsageRecord {
            skill_name: name.clone(),
            count: 0,
            last_used: None,
        })
        .collect();

    for event in events {
        if let Some(record) = records
            .iter_mut()
            .find(|r| r.skill_name == event.skill_name)
        {
            record.observe(event.timestamp);
        }
    }

    let threshold = Duration::from_secs(stale_after_days.saturating_mul(SECONDS_PER_DAY));

    let mut dead = Vec::new();
    let mut stale = Vec::new();
    for record in &records {
        if record.count == 0 {
            dead.push(record.skill_name.clone());
        } else if let Some(last_used) = record.last_used
            && now.duration_since(last_used).unwrap_or_default() > threshold
        {
            stale.push(record.skill_name.clone());
        }
    }

    UsageReport {
        records,
        dead,
        stale,
        stale_after_days,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(name: &str, days_ago: u64) -> SkillUsageEvent {
        SkillUsageEvent {
            skill_name: name.to_string(),
            timestamp: SystemTime::now() - Duration::from_secs(days_ago * SECONDS_PER_DAY),
        }
    }

    #[test]
    fn empty_history_marks_all_skills_dead() {
        let report = build_usage_report(&[], &["alpha".into(), "beta".into()], 30);
        assert_eq!(report.dead, vec!["alpha", "beta"]);
        assert!(report.stale.is_empty());
        assert_eq!(report.records.len(), 2);
    }

    #[test]
    fn used_skill_is_not_dead() {
        let report = build_usage_report(&[event("alpha", 1)], &["alpha".into(), "beta".into()], 30);
        assert_eq!(report.dead, vec!["beta"]);
        assert!(report.stale.is_empty());
    }

    #[test]
    fn recent_usage_is_not_stale() {
        let report = build_usage_report(&[event("alpha", 5)], &["alpha".into()], 30);
        assert!(report.stale.is_empty());
    }

    #[test]
    fn old_usage_is_stale() {
        let report = build_usage_report(&[event("alpha", 60)], &["alpha".into()], 30);
        assert_eq!(report.stale, vec!["alpha"]);
    }

    #[test]
    fn threshold_boundary_is_not_stale() {
        let now = SystemTime::now();
        let used = now - Duration::from_secs(30 * SECONDS_PER_DAY);
        let report = build_usage_report_as_of(
            &[SkillUsageEvent {
                skill_name: "alpha".into(),
                timestamp: used,
            }],
            &["alpha".into()],
            30,
            now,
        );
        assert!(report.stale.is_empty());
    }

    #[test]
    fn observe_keeps_newest_timestamp() {
        let older = event("x", 10).timestamp;
        let newer = event("x", 5).timestamp;
        let mut record = SkillUsageRecord::default();
        record.observe(older);
        record.observe(newer);
        assert_eq!(record.count, 2);
        assert_eq!(record.last_used, Some(newer));
    }
}
