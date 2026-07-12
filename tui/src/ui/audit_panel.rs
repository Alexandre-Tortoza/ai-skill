//! Panel that renders an audit report broken into health categories.

use ai_skill_core::{Skill, audit_skills};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::style_helpers::fg;

/// Renders the audit report grouped by broken, duplicate, no-agents, and update-available.
pub fn render_audit_panel(skills: &[Skill], area: Rect, frame: &mut Frame) {
    let report = audit_skills(skills);

    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    let summary = format!(
        "broken: {}  duplicates: {}  no-agents: {}  updates: {}",
        report.broken.len(),
        report.duplicates.len(),
        report.no_agents.len(),
        report.update_available.len(),
    );
    let header = Paragraph::new(summary)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Audit Report"));
    frame.render_widget(header, chunks[0]);

    let panels = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(chunks[1]);

    render_section("Broken", &report.broken, Color::Red, panels[0], frame);
    render_section(
        "Duplicates",
        &report.duplicates,
        Color::Cyan,
        panels[1],
        frame,
    );
    render_section(
        "No Agents",
        &report.no_agents,
        Color::Yellow,
        panels[2],
        frame,
    );
    render_section(
        "Updates",
        &report.update_available,
        Color::Green,
        panels[3],
        frame,
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::{DriftState, Scope, Skill, SkillMode, ValidationState};
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
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_audit_panel(skills, f.area(), f))
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
            .draw(|f| render_audit_panel(&skills, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }
}
