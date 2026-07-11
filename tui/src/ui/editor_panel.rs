//! In-app editor for skill manifest frontmatter fields.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app::{EditField, EditorState},
    ui::style_helpers::fg,
};

/// Renders the editor panel with name/agents/tags input fields.
pub fn render_editor_panel(state: &EditorState, area: Rect, frame: &mut Frame) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    render_form(state, chunks[0], frame);
    render_preview(state, chunks[1], frame);
}

fn render_form(state: &EditorState, area: Rect, frame: &mut Frame) {
    let active = fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let inactive = Style::default();

    let field_line = |field: EditField, label: &'static str, value: &str| -> Line<'static> {
        let s = if state.field == field {
            active
        } else {
            inactive
        };
        Line::from(vec![
            Span::styled(format!("{label}: "), s),
            Span::raw(value.to_string()),
            if state.field == field {
                Span::styled(" ◀", active)
            } else {
                Span::raw("")
            },
        ])
    };

    let lines = vec![
        Line::from(Span::styled(
            "Edit Frontmatter",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        field_line(EditField::Name, "Name", &state.name_input),
        field_line(EditField::Agents, "Agents", &state.agents_input),
        field_line(EditField::Tags, "Tags", &state.tags_input),
        Line::raw(""),
        Line::from(Span::styled(
            "Tab: next field  Enter: save  Esc: cancel",
            inactive,
        )),
    ];

    let widget =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Editor"));
    frame.render_widget(widget, area);
}

fn render_preview(state: &EditorState, area: Rect, frame: &mut Frame) {
    let body = state
        .skill
        .manifest_content
        .as_deref()
        .unwrap_or("(no body)");
    let widget = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title("Body Preview"))
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::{Scope, Skill, ValidationState};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    fn make_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            validation: ValidationState::Valid,
            manifest_content: Some("# body text\n".into()),
            drift_state: ai_skill_core::DriftState::default(),
        }
    }

    fn make_state(skill: Skill) -> EditorState {
        EditorState {
            name_input: skill.name.clone(),
            agents_input: skill.agents.join(", "),
            tags_input: skill.tags.join(", "),
            skill,
            field: EditField::default(),
        }
    }

    fn render(state: &EditorState) -> String {
        let backend = TestBackend::new(80, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_editor_panel(state, f.area(), f))
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
    fn renders_without_panic() {
        let state = make_state(make_skill("my-skill"));
        let rendered = render(&state);
        assert!(rendered.contains("Editor"));
    }

    #[test]
    fn skill_name_appears_in_form() {
        let state = make_state(make_skill("my-skill"));
        let rendered = render(&state);
        assert!(rendered.contains("my-skill"));
    }

    #[test]
    fn body_appears_in_preview_pane() {
        let state = make_state(make_skill("my-skill"));
        let rendered = render(&state);
        assert!(rendered.contains("body text"));
    }

    #[test]
    fn snapshot_editor_name_field_active() {
        let state = make_state(make_skill("alpha"));
        let backend = TestBackend::new(80, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_editor_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }
}
