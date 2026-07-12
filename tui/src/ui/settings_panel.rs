use ai_skill_core::{ProjectSettings, TuiConfig};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::style_helpers::{bg, fg};

#[derive(Default)]
pub struct SettingsState {
    pub project_path: Option<String>,
    pub dirty: bool,
    pub selected_override_index: usize,
    pub editing_global: bool,
}

#[derive(Default)]
pub struct ConfigState {
    pub message: Option<String>,
}

pub fn render_config_panel(config: &TuiConfig, state: &ConfigState, area: Rect, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);

    let header = Paragraph::new(" TUI Configuration ")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Proxy: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(config.proxy.as_deref().unwrap_or("(not set)")),
    ]));

    if config.custom_agent_paths.is_empty() {
        lines.push(Line::from(Span::styled(
            "Custom agent paths: (none)",
            Style::default(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Custom agent paths:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (agent, path) in &config.custom_agent_paths {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(agent.clone(), fg(Color::Cyan)),
                Span::raw(" -> "),
                Span::raw(path.display().to_string()),
            ]));
        }
    }

    if let Some(ref theme) = config.theme {
        lines.push(Line::from(Span::styled(
            "Theme overrides:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (key, val) in theme {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(key.clone(), fg(Color::Magenta)),
                Span::raw(": "),
                Span::raw(val.clone()),
            ]));
        }
    }

    if config.keymap.is_empty() {
        lines.push(Line::from("Keymap overrides: (none)"));
    } else {
        lines.push(Line::from(Span::styled(
            "Keymap overrides:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (action, key) in &config.keymap {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(action.clone(), fg(Color::Yellow)),
                Span::raw(" -> "),
                Span::raw(key.clone()),
            ]));
        }
    }

    let body = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Current Config "),
    );
    frame.render_widget(body, chunks[1]);

    if let Some(ref msg) = state.message {
        let msg_widget = Paragraph::new(msg.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Message "))
            .wrap(Wrap { trim: false });
        frame.render_widget(msg_widget, chunks[2]);
    } else {
        let hint = Paragraph::new(" Edit ~/.config/ai-skill/config.json to change settings ");
        frame.render_widget(hint, chunks[2]);
    }
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

    fn render_config(config: &TuiConfig, state: &ConfigState) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_config_panel(config, state, f.area(), f))
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

    #[test]
    fn config_panel_shows_custom_path_and_keymap() {
        let config = TuiConfig {
            custom_agent_paths: [("cursor".into(), "/tmp/cursor-skills".into())]
                .into_iter()
                .collect(),
            theme: Some([("primary".into(), "blue".into())].into_iter().collect()),
            keymap: [("quit".into(), "q".into())].into_iter().collect(),
            proxy: Some("http://proxy:8080".into()),
            stale_after_days: 30,
        };
        let rendered = render_config(&config, &ConfigState::default());
        assert!(rendered.contains("cursor"));
        assert!(rendered.contains("quit"));
        assert!(rendered.contains("proxy"));
    }
}
