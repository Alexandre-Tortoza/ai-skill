use ai_skill_core::ProjectSettings;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::style_helpers::{bg, fg};

#[derive(Default)]
pub struct SettingsState {
    pub project_path: Option<String>,
    pub dirty: bool,
    pub selected_override_index: usize,
    pub editing_global: bool,
}

pub fn render_settings_panel(
    settings: &ProjectSettings,
    state: &SettingsState,
    area: Rect,
    frame: &mut Frame,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(area);

    let path_str = state.project_path.as_deref().unwrap_or("(no project)");
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "Project settings: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(path_str),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Project Settings "),
    );
    frame.render_widget(header, chunks[0]);

    let auto_trigger_str = if settings.auto_trigger { "ON" } else { "OFF" };
    let auto_color = if settings.auto_trigger {
        Color::Green
    } else {
        Color::Red
    };
    let global_line = Line::from(vec![
        Span::styled(
            "Global auto-trigger: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            auto_trigger_str,
            fg(auto_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("    [t] toggle"),
    ]);
    let attrs = if state.editing_global {
        bg(Color::Blue).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let global_widget = Paragraph::new(global_line).style(attrs).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Auto-Trigger "),
    );
    frame.render_widget(global_widget, chunks[1]);

    if settings.skill_overrides.is_empty() {
        let msg = Paragraph::new(" No skill overrides. Press [a] to add selected skill.").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Skill Overrides "),
        );
        frame.render_widget(msg, chunks[2]);
    } else {
        let items: Vec<ListItem> = settings
            .skill_overrides
            .iter()
            .enumerate()
            .map(|(i, o)| {
                let trigger_str = if o.auto_trigger { "ON" } else { "OFF" };
                let trigger_color = if o.auto_trigger {
                    Color::Green
                } else {
                    Color::Red
                };
                let prefix = if i == state.selected_override_index {
                    "> "
                } else {
                    "  "
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, fg(Color::Yellow)),
                    Span::raw(format!("{}  ", o.skill_name)),
                    Span::styled(trigger_str, fg(trigger_color).add_modifier(Modifier::BOLD)),
                    Span::raw("  [o] toggle  [d] remove"),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Skill Overrides "),
            )
            .highlight_style(bg(Color::Blue).add_modifier(Modifier::BOLD));
        let mut list_state = ListState::default();
        if !settings.skill_overrides.is_empty() {
            list_state.select(Some(state.selected_override_index));
        }
        frame.render_stateful_widget(list, chunks[2], &mut list_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::SkillOverride;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_settings(settings: &ProjectSettings, state: &SettingsState) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_settings_panel(settings, state, f.area(), f))
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
    fn renders_without_panic_empty_settings() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let settings = ProjectSettings::default();
        let state = SettingsState::default();
        terminal
            .draw(|f| render_settings_panel(&settings, &state, f.area(), f))
            .unwrap();
    }

    #[test]
    fn renders_without_panic_with_overrides() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let settings = ProjectSettings {
            auto_trigger: false,
            skill_overrides: vec![SkillOverride {
                skill_name: "alpha".into(),
                auto_trigger: false,
            }],
        };
        let state = SettingsState::default();
        terminal
            .draw(|f| render_settings_panel(&settings, &state, f.area(), f))
            .unwrap();
    }

    #[test]
    fn shows_auto_trigger_status() {
        let settings = ProjectSettings {
            auto_trigger: true,
            skill_overrides: vec![],
        };
        let state = SettingsState::default();
        let rendered = render_settings(&settings, &state);
        assert!(rendered.contains("ON"));
    }

    #[test]
    fn shows_auto_trigger_off() {
        let settings = ProjectSettings {
            auto_trigger: false,
            skill_overrides: vec![],
        };
        let state = SettingsState::default();
        let rendered = render_settings(&settings, &state);
        assert!(rendered.contains("OFF"));
    }

    #[test]
    fn shows_override_skill_name() {
        let settings = ProjectSettings {
            auto_trigger: true,
            skill_overrides: vec![SkillOverride {
                skill_name: "my-skill".into(),
                auto_trigger: false,
            }],
        };
        let state = SettingsState::default();
        let rendered = render_settings(&settings, &state);
        assert!(rendered.contains("my-skill"));
    }

    #[test]
    fn shows_project_path_if_available() {
        let settings = ProjectSettings::default();
        let state = SettingsState {
            project_path: Some("my-project/.claude/settings.json".into()),
            ..SettingsState::default()
        };
        let rendered = render_settings(&settings, &state);
        eprintln!("RENDERED OUTPUT for path test:\n---\n{rendered}\n---");
        assert!(rendered.contains("my-project"));
    }
}
