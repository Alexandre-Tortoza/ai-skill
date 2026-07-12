//! Panel showing full details of a single skill.

use ai_skill_core::{DriftState, Scope, Skill, estimate_skill_cost};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::style_helpers::{badge_for_mode, badge_for_validation, fg};

/// Renders the full detail view for a single skill (scrollable).
/// `auto_trigger` is `None` (unknown), `Some(true)` (on), or `Some(false)` (off).
pub fn render_detail_panel(
    skill: &Skill,
    scroll: u16,
    area: Rect,
    frame: &mut Frame,
    auto_trigger: Option<bool>,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .title(skill.name.as_str());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([Constraint::Length(8), Constraint::Min(0)]).split(inner);

    // Metadata section
    let scope_str = match skill.scope {
        Scope::Global => "global",
        Scope::Project => "project",
    };
    let agents_str = if skill.agents.is_empty() {
        "(none)".to_string()
    } else {
        skill.agents.join(", ")
    };
    let (val_badge, val_color) = badge_for_validation(&skill.validation);
    let val_str = if val_badge.is_empty() {
        "valid"
    } else {
        val_badge
    };

    let (mode_badge, mode_color) = badge_for_mode(&skill.mode);
    let mode_str = if mode_badge.is_empty() {
        "active"
    } else {
        mode_badge
    };

    let cost = estimate_skill_cost(skill);

    let auto_trigger_str = match auto_trigger {
        Some(true) => "on",
        Some(false) => "off",
        None => "(unknown)",
    };

    let drift_line = match &skill.drift_state {
        DriftState::UpdateAvailable {
            local_hash,
            upstream_hash,
        } => Line::from(vec![
            Span::styled(
                "drift:      ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("local={local_hash}  upstream={upstream_hash}")),
        ]),
        _ => Line::raw(""),
    };

    let meta = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "scope:      ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(scope_str),
        ]),
        Line::from(vec![
            Span::styled(
                "agents:     ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(agents_str),
        ]),
        Line::from(vec![
            Span::styled(
                "path:       ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(skill.path.to_string_lossy().to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "validation: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(val_str, fg(val_color)),
        ]),
        Line::from(vec![
            Span::styled(
                "mode:       ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(mode_str, fg(mode_color)),
        ]),
        Line::from(vec![
            Span::styled(
                "budget:     ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{} chars  ~{} tok",
                cost.char_count, cost.estimated_tokens
            )),
        ]),
        Line::from(vec![
            Span::styled(
                "auto-trig:  ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                auto_trigger_str,
                match auto_trigger {
                    Some(true) => fg(Color::Green),
                    Some(false) => fg(Color::Red),
                    None => fg(Color::DarkGray),
                },
            ),
            Span::raw("  [o] toggle"),
        ]),
        drift_line,
    ]);
    frame.render_widget(meta, chunks[0]);

    // Body section
    let body_text = skill
        .manifest_content
        .as_deref()
        .unwrap_or("(no manifest available)");
    let body = Paragraph::new(body_text)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(body, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::SkillMode;
    use ai_skill_core::ValidationState;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    fn make_skill(name: &str, validation: ValidationState, content: Option<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation,
            manifest_content: content.map(str::to_string),
            drift_state: ai_skill_core::DriftState::default(),
        }
    }

    fn render_to_string(skill: &Skill, scroll: u16) -> String {
        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_detail_panel(skill, scroll, f.area(), f, None))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    }

    #[test]
    fn snapshot_valid_skill_with_content() {
        let skill = make_skill(
            "my-skill",
            ValidationState::Valid,
            Some("# My Skill\n\nDoes something useful."),
        );
        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_detail_panel(&skill, 0, f.area(), f, None))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn broken_skill_shows_no_manifest_message() {
        let skill = make_skill("broken", ValidationState::BrokenSymlink, None);
        let rendered = render_to_string(&skill, 0);
        assert!(rendered.contains("no manifest available"));
    }

    #[test]
    fn skill_name_appears_in_title() {
        let skill = make_skill("omarchy", ValidationState::Valid, Some("body"));
        let rendered = render_to_string(&skill, 0);
        assert!(rendered.contains("omarchy"));
    }

    #[test]
    fn update_available_drift_state_shows_hashes() {
        use ai_skill_core::DriftState;
        let mut skill = make_skill("s", ValidationState::Valid, Some("body"));
        skill.drift_state = DriftState::UpdateAvailable {
            local_hash: "abc1234".into(),
            upstream_hash: "def5678".into(),
        };
        let rendered = render_to_string(&skill, 0);
        assert!(rendered.contains("abc1234"));
        assert!(rendered.contains("def5678"));
    }

    #[test]
    fn up_to_date_drift_state_shows_no_hashes() {
        use ai_skill_core::DriftState;
        let mut skill = make_skill("s", ValidationState::Valid, Some("body"));
        skill.drift_state = DriftState::UpToDate;
        let rendered = render_to_string(&skill, 0);
        assert!(!rendered.contains("upstream="));
    }

    #[test]
    fn scope_and_agents_appear_in_metadata() {
        let skill = make_skill("s", ValidationState::Valid, None);
        let rendered = render_to_string(&skill, 0);
        assert!(rendered.contains("global"));
        assert!(rendered.contains("claude"));
    }
}
