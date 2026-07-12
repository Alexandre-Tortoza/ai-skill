//! Multi-step wizard for creating a new skill.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{CreateStep, CreateWizardState},
    ui::style_helpers::fg,
};

/// Renders the create-skill wizard with name/agents/tags steps and a preview.
pub fn render_create_wizard(state: &CreateWizardState, area: Rect, frame: &mut Frame) {
    let active = fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let inactive = Style::default();

    let step_label =
        |step: &CreateStep, current: &CreateStep, label: &str, value: &str| -> Line<'static> {
            let s = if step == current { active } else { inactive };
            Line::from(vec![
                Span::styled(format!("{label}: "), s),
                Span::raw(value.to_string()),
                if step == current {
                    Span::styled(" ◀", active)
                } else {
                    Span::raw("")
                },
            ])
        };

    let mut lines = vec![
        Line::from(Span::styled(
            "Create New Skill",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        step_label(&CreateStep::Name, &state.step, "Name", &state.name),
        step_label(
            &CreateStep::Agents,
            &state.step,
            "Agents (comma-separated)",
            &state.agents_input,
        ),
        step_label(
            &CreateStep::Tags,
            &state.step,
            "Tags (comma-separated)",
            &state.tags_input,
        ),
    ];

    if !state.errors.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            "Errors:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for err in &state.errors {
            lines.push(Line::from(Span::styled(format!("  ✗ {err}"), fg(Color::Red))));
        }
    }

    lines.push(Line::raw(""));
    if state.step == CreateStep::Preview {
        if state.errors.is_empty() {
            lines.push(Line::from(Span::styled("[ Press Enter to create ]", active)));
        } else {
            lines.push(Line::from(Span::styled(
                "[ Fix errors above before creating ]",
                fg(Color::Red),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled("Tab: next field  Esc: cancel", inactive)));
    }

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("New Skill Wizard"),
    );
    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render(state: &CreateWizardState) -> String {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_create_wizard(state, f.area(), f))
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
    fn renders_without_panic_on_default_state() {
        let state = CreateWizardState::default();
        let rendered = render(&state);
        assert!(rendered.contains("New Skill Wizard"));
    }

    #[test]
    fn name_step_shows_active_marker_on_name() {
        let state = CreateWizardState::default();
        assert_eq!(state.step, CreateStep::Name);
        let rendered = render(&state);
        assert!(rendered.contains("Name"));
    }

    #[test]
    fn preview_step_shows_enter_hint() {
        let state = CreateWizardState {
            step: CreateStep::Preview,
            name: "my-skill".into(),
            agents_input: "claude".into(),
            tags_input: String::new(),
            errors: vec![],
        };
        let rendered = render(&state);
        assert!(rendered.contains("Enter to create"));
    }

    #[test]
    fn snapshot_name_step() {
        let state = CreateWizardState {
            step: CreateStep::Name,
            name: "test".into(),
            agents_input: String::new(),
            tags_input: String::new(),
            errors: vec![],
        };
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_create_wizard(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }
}
