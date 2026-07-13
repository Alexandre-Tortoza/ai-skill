//! Panel that renders an audit report broken into health categories.

use ai_skill_core::{Skill, UsageReport, audit_skills};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::style_helpers::fg;
use super::theme::{Theme, ThemeSlot};

/// Renders the audit report grouped by health category, including usage.
pub fn render_audit_panel(
    skills: &[Skill],
    usage: &UsageReport,
    theme: &Theme,
    area: Rect,
    frame: &mut Frame,
) {
    let report = audit_skills(skills);

    let body = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let summary = format!(
        "broken: {}  duplicates: {}  no-agents: {}  updates: {}  dead: {}  stale: {}",
        report.broken.len(),
        report.duplicates.len(),
        report.no_agents.len(),
        report.update_available.len(),
        usage.dead.len(),
        usage.stale.len(),
    );
    let header = Paragraph::new(summary)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Audit Report"));
    frame.render_widget(header, body[0]);

    let show_usage = !usage.dead.is_empty() || !usage.stale.is_empty();
    let (top, bottom) = if show_usage {
        let split = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body[1]);
        (split[0], Some(split[1]))
    } else {
        (body[1], None)
    };

    let panels = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(top);

    render_section(
        "Broken",
        &report.broken,
        theme.color(ThemeSlot::Error),
        panels[0],
        frame,
    );
    render_section(
        "Duplicates",
        &report.duplicates,
        theme.color(ThemeSlot::Accent),
        panels[1],
        frame,
    );
    render_section(
        "No Agents",
        &report.no_agents,
        theme.color(ThemeSlot::Warning),
        panels[2],
        frame,
    );
    render_section(
        "Updates",
        &report.update_available,
        theme.color(ThemeSlot::Success),
        panels[3],
        frame,
    );

    if let Some(bottom) = bottom {
        let usage_panels =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(bottom);
        render_names(
            &format!("Dead (>{}d)", usage.stale_after_days),
            &usage.dead,
            theme.color(ThemeSlot::Dead),
            usage_panels[0],
            frame,
        );
        render_names(
            &format!("Stale (>{}d)", usage.stale_after_days),
            &usage.stale,
            theme.color(ThemeSlot::Stale),
            usage_panels[1],
            frame,
        );
    }
}

fn render_section(title: &str, skills: &[&Skill], color: Color, area: Rect, frame: &mut Frame) {
    let items: Vec<ListItem> = skills
        .iter()
        .map(|s| ListItem::new(Line::from(Span::styled(s.name.clone(), fg(color)))))
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("{title} ({})", skills.len())),
    );
    frame.render_widget(list, area);
}

fn render_names(title: &str, names: &[String], color: Color, area: Rect, frame: &mut Frame) {
    let items: Vec<ListItem> = names
        .iter()
        .map(|n| ListItem::new(Line::from(Span::styled(n.clone(), fg(color)))))
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("{title} ({})", names.len())),
    );
    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::{DriftState, Scope, Skill, SkillMode, UsageReport, ValidationState};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    fn make_skill(name: &str, validation: ValidationState) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation,
            manifest_content: None,
            drift_state: DriftState::default(),
        }
    }

    fn render(skills: &[Skill]) -> String {
        render_with_usage(skills, &UsageReport::default())
    }

    fn render_with_usage(skills: &[Skill], usage: &UsageReport) -> String {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_audit_panel(skills, usage, &Theme::default(), f.area(), f))
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
    fn renders_without_panic_on_empty_skills() {
        let rendered = render(&[]);
        assert!(rendered.contains("Audit Report"));
    }

    #[test]
    fn broken_skill_appears_in_broken_section_summary() {
        let skills = vec![make_skill("dead", ValidationState::BrokenSymlink)];
        let rendered = render(&skills);
        assert!(rendered.contains("broken: 1"));
    }

    #[test]
    fn valid_skill_shows_zero_broken() {
        let skills = vec![make_skill("ok", ValidationState::Valid)];
        let rendered = render(&skills);
        assert!(rendered.contains("broken: 0"));
    }

    #[test]
    fn snapshot_mixed_skills() {
        let mut update_skill = make_skill("drifted", ValidationState::Valid);
        update_skill.drift_state = DriftState::UpdateAvailable {
            local_hash: "abc".into(),
            upstream_hash: "def".into(),
        };
        let skills = vec![
            make_skill("broken", ValidationState::BrokenSymlink),
            update_skill,
        ];
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render_audit_panel(
                    &skills,
                    &UsageReport::default(),
                    &Theme::default(),
                    f.area(),
                    f,
                )
            })
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn summary_includes_dead_and_stale_counts() {
        let skills = vec![make_skill("ok", ValidationState::Valid)];
        let usage = UsageReport {
            dead: vec!["ghost".to_string()],
            stale: vec!["rusty".to_string()],
            stale_after_days: 30,
            ..Default::default()
        };
        let rendered = render_with_usage(&skills, &usage);
        assert!(rendered.contains("dead: 1"));
        assert!(rendered.contains("stale: 1"));
    }

    #[test]
    fn dead_and_stale_sections_render_when_present() {
        let skills = vec![make_skill("ok", ValidationState::Valid)];
        let usage = UsageReport {
            dead: vec!["ghost".to_string()],
            stale: vec!["rusty".to_string()],
            stale_after_days: 30,
            ..Default::default()
        };
        let rendered = render_with_usage(&skills, &usage);
        assert!(rendered.contains("Dead (>30d)"));
        assert!(rendered.contains("Stale (>30d)"));
        assert!(rendered.contains("ghost"));
        assert!(rendered.contains("rusty"));
    }

    #[test]
    fn no_usage_sections_when_empty() {
        let skills = vec![make_skill("ok", ValidationState::Valid)];
        let rendered = render(&skills);
        assert!(!rendered.contains("Dead"));
    }
}
