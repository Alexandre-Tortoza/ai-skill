//! Wizard that configures scope and agents before installing from the catalog.

use ai_skill_core::Scope;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::InstallWizardState;

/// Renders the install-from-catalog wizard.
pub fn render_install_wizard(state: &InstallWizardState, area: Rect, frame: &mut Frame) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .title("Install Skill");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([
        Constraint::Length(2), // skill name
        Constraint::Length(2), // scope
        Constraint::Min(3),    // agents
        Constraint::Length(2), // hints
    ])
    .split(inner);

    // Skill name
    let name = state
        .entry
        .as_ref()
        .map(|e| e.name.as_str())
        .unwrap_or("(no skill selected)");
    let name_para = Paragraph::new(Line::from(vec![
        Span::styled("Skill: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(name),
    ]));
    frame.render_widget(name_para, chunks[0]);

    // Scope selector
    let scope_label = match state.scope {
        Scope::Global => "[●] global  [ ] project",
        Scope::Project => "[ ] global  [●] project",
    };
    let scope_para = Paragraph::new(Line::from(vec![
        Span::styled("Scope: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(scope_label),
        Span::raw("  (Tab to switch)"),
    ]));
    frame.render_widget(scope_para, chunks[1]);

    // Agents
    let agent_lines: Vec<Line> = state
        .available_agents
        .iter()
        .map(|a| {
            let selected = state.selected_agents.contains(a);
            let marker = if selected { "[●]" } else { "[ ]" };
            Line::from(vec![Span::raw(format!("{marker} {a}"))])
        })
        .collect();
    let agents_para = Paragraph::new(agent_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Agents (Space to toggle)"),
    );
    frame.render_widget(agents_para, chunks[2]);

    // Hints
    let hints = Paragraph::new("Enter: confirm install  |  Esc: back to search");
    frame.render_widget(hints, chunks[3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::CatalogEntry;
    use ratatui::{Terminal, backend::TestBackend};

    fn render(state: &InstallWizardState) -> String {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_install_wizard(state, f.area(), f))
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
    fn snapshot_empty_wizard() {
        let state = InstallWizardState::default();
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_install_wizard(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_wizard_with_entry() {
        let state = InstallWizardState {
            entry: Some(CatalogEntry {
                name: "omarchy".to_string(),
                description: "WM skill".to_string(),
                url: None,
            }),
            scope: Scope::Global,
            available_agents: vec!["claude".to_string()],
            selected_agents: vec!["claude".to_string()],
        };
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_install_wizard(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn skill_name_appears_in_render() {
        let state = InstallWizardState {
            entry: Some(CatalogEntry {
                name: "omarchy".to_string(),
                description: "".to_string(),
                url: None,
            }),
            scope: Scope::Global,
            available_agents: vec![],
            selected_agents: vec![],
        };
        let rendered = render(&state);
        assert!(rendered.contains("omarchy"));
    }

    #[test]
    fn global_scope_shows_global_selected() {
        let state = InstallWizardState {
            scope: Scope::Global,
            ..InstallWizardState::default()
        };
        let rendered = render(&state);
        assert!(rendered.contains("global"));
    }
}
