//! Colour helpers that respect `NO_COLOR` conventions.

use ai_skill_core::{DriftState, SkillMode, ValidationState};
use ratatui::style::{Color, Style};

/// Returns `true` if the `NO_COLOR` environment variable is set and non-empty.
pub fn no_color_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
}

/// Returns [`Color::Reset`] when `no_color` is true, otherwise `color`.
pub fn color_for_mode(color: Color, no_color: bool) -> Color {
    if no_color { Color::Reset } else { color }
}

/// Returns `color` if `NO_COLOR` is not set, otherwise [`Color::Reset`].
pub fn color(color: Color) -> Color {
    color_for_mode(color, no_color_enabled())
}

/// Returns a style with the given foreground colour (respects `NO_COLOR`).
pub fn fg(color: Color) -> Style {
    Style::default().fg(self::color(color))
}

/// Returns a style with the given background colour (respects `NO_COLOR`).
pub fn bg(color: Color) -> Style {
    if no_color_enabled() {
        Style::default()
    } else {
        Style::default().bg(color)
    }
}

/// Returns a style with both foreground and background colours (respects `NO_COLOR`).
pub fn fg_bg(foreground: Color, background: Color) -> Style {
    if no_color_enabled() {
        Style::default()
    } else {
        Style::default().fg(foreground).bg(background)
    }
}

/// Returns `(badge_text, colour)` for a given validation state.
pub fn badge_for_validation(state: &ValidationState) -> (&'static str, Color) {
    match state {
        ValidationState::Valid => ("", Color::Reset),
        ValidationState::BrokenSymlink => ("[broken-symlink]", Color::Red),
        ValidationState::MissingManifest => ("[no-manifest]", Color::Red),
        ValidationState::InvalidFrontmatter { .. } => ("[bad-frontmatter]", Color::Yellow),
        ValidationState::OrphanLock => ("[orphan-lock]", Color::Magenta),
        ValidationState::Duplicate { .. } => ("[duplicate]", Color::Cyan),
    }
}

/// Returns `(badge_text, colour)` for a given skill mode.
pub fn badge_for_mode(mode: &SkillMode) -> (&'static str, Color) {
    match mode {
        SkillMode::Active => ("", Color::Reset),
        SkillMode::NameOnly => ("[name-only]", Color::Blue),
        SkillMode::Disabled => ("[disabled]", Color::DarkGray),
    }
}

/// Returns a drift badge `("[↑]", yellow)` if an update is available, otherwise `None`.
pub fn drift_badge(state: &DriftState) -> Option<(&'static str, Color)> {
    if matches!(state, DriftState::UpdateAvailable { .. }) {
        Some(("[↑]", Color::Yellow))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn valid_returns_empty_badge_and_reset_color() {
        let (badge, color) = badge_for_validation(&ValidationState::Valid);
        assert!(badge.is_empty());
        assert_eq!(color, Color::Reset);
    }

    #[test]
    fn broken_symlink_returns_red() {
        let (_, color) = badge_for_validation(&ValidationState::BrokenSymlink);
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn missing_manifest_returns_red() {
        let (_, color) = badge_for_validation(&ValidationState::MissingManifest);
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn invalid_frontmatter_returns_yellow() {
        let (_, color) = badge_for_validation(&ValidationState::InvalidFrontmatter {
            reason: "err".into(),
        });
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn orphan_lock_returns_magenta() {
        let (_, color) = badge_for_validation(&ValidationState::OrphanLock);
        assert_eq!(color, Color::Magenta);
    }

    #[test]
    fn duplicate_returns_cyan() {
        let (_, color) = badge_for_validation(&ValidationState::Duplicate {
            conflicts_with: PathBuf::from("/other"),
        });
        assert_eq!(color, Color::Cyan);
    }

    #[test]
    fn drift_badge_update_available_returns_yellow_arrow() {
        use ai_skill_core::DriftState;
        let (badge, color) = drift_badge(&DriftState::UpdateAvailable {
            local_hash: "abc".into(),
            upstream_hash: "def".into(),
        })
        .unwrap();
        assert_eq!(badge, "[↑]");
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn drift_badge_up_to_date_returns_none() {
        use ai_skill_core::DriftState;
        assert!(drift_badge(&DriftState::UpToDate).is_none());
    }

    #[test]
    fn drift_badge_unknown_returns_none() {
        use ai_skill_core::DriftState;
        assert!(drift_badge(&DriftState::Unknown).is_none());
    }

    #[test]
    fn badge_for_mode_active_returns_empty() {
        let (badge, color) = badge_for_mode(&SkillMode::Active);
        assert!(badge.is_empty());
        assert_eq!(color, Color::Reset);
    }

    #[test]
    fn badge_for_mode_name_only_returns_blue() {
        let (badge, color) = badge_for_mode(&SkillMode::NameOnly);
        assert_eq!(badge, "[name-only]");
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn badge_for_mode_disabled_returns_dark_gray() {
        let (badge, color) = badge_for_mode(&SkillMode::Disabled);
        assert_eq!(badge, "[disabled]");
        assert_eq!(color, Color::DarkGray);
    }

    #[test]
    fn color_for_mode_resets_when_no_color_enabled() {
        assert_eq!(color_for_mode(Color::Red, true), Color::Reset);
    }

    #[test]
    fn color_for_mode_keeps_ansi_color_by_default() {
        assert_eq!(color_for_mode(Color::Red, false), Color::Red);
    }
}
