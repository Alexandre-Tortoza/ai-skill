use crate::i18n::I18n;
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

pub fn render_config_panel(
    config: &TuiConfig,
    state: &ConfigState,
    area: Rect,
    frame: &mut Frame,
    i18n: &I18n,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);

    let header = Paragraph::new(i18n.config_header())
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            i18n.config_proxy_label(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(config.proxy.as_deref().unwrap_or(i18n.config_proxy_unset())),
    ]));

    if config.custom_agent_paths.is_empty() {
        lines.push(Line::from(Span::styled(
            i18n.config_custom_paths_none(),
            Style::default(),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            i18n.config_custom_paths_label(),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (agent, path) in &config.custom_agent_paths {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(agent.clone(), fg(Color::Cyan)),
                Span::raw(i18n.config_path_arrow()),
                Span::raw(path.display().to_string()),
            ]));
        }
    }

    if let Some(ref theme) = config.theme {
        lines.push(Line::from(Span::styled(
            i18n.config_theme_label(),
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
        lines.push(Line::from(i18n.config_keymap_none()));
    } else {
        lines.push(Line::from(Span::styled(
            i18n.config_keymap_label(),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (action, key) in &config.keymap {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(action.clone(), fg(Color::Yellow)),
                Span::raw(i18n.config_path_arrow()),
                Span::raw(key.clone()),
            ]));
        }
    }

    let body = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(i18n.config_current_title()),
    );
    frame.render_widget(body, chunks[1]);

    if let Some(ref msg) = state.message {
        let msg_widget = Paragraph::new(msg.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(i18n.config_message_title()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(msg_widget, chunks[2]);
    } else {
        let hint = Paragraph::new(i18n.config_edit_hint());
        frame.render_widget(hint, chunks[2]);
    }
}

pub fn render_settings_panel(
    settings: &ProjectSettings,
    state: &SettingsState,
    area: Rect,
    frame: &mut Frame,
    i18n: &I18n,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(area);

    let path_str = state
        .project_path
        .as_deref()
        .unwrap_or(i18n.settings_project_none());
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            i18n.settings_project_label(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(path_str),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(i18n.settings_project_title()),
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
            i18n.settings_global_auto_trigger(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            auto_trigger_str,
            fg(auto_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(i18n.settings_toggle_hint()),
    ]);
    let attrs = if state.editing_global {
        bg(Color::Blue).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let global_widget = Paragraph::new(global_line).style(attrs).block(
        Block::default()
            .borders(Borders::ALL)
            .title(i18n.settings_auto_trigger_title()),
    );
    frame.render_widget(global_widget, chunks[1]);

    if settings.skill_overrides.is_empty() {
        let msg = Paragraph::new(i18n.settings_no_overrides()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(i18n.settings_overrides_title()),
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
                    Span::raw(i18n.settings_override_toggle()),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(i18n.settings_overrides_title()),
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
    use crate::i18n::{I18n, Locale};
    use ai_skill_core::SkillOverride;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_settings(settings: &ProjectSettings, state: &SettingsState) -> String {
        render_settings_i18n(settings, state, &I18n::default())
    }

    fn render_settings_i18n(
        settings: &ProjectSettings,
        state: &SettingsState,
        i18n: &I18n,
    ) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_settings_panel(settings, state, f.area(), f, i18n))
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
            .draw(|f| render_config_panel(config, state, f.area(), f, &I18n::default()))
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
            .draw(|f| render_settings_panel(&settings, &state, f.area(), f, &I18n::default()))
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
            .draw(|f| render_settings_panel(&settings, &state, f.area(), f, &I18n::default()))
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
            locale: None,
        };
        let rendered = render_config(&config, &ConfigState::default());
        assert!(rendered.contains("cursor"));
        assert!(rendered.contains("quit"));
        assert!(rendered.contains("proxy"));
    }

    #[test]
    fn pt_br_config_panel_localizes_labels() {
        let config = TuiConfig {
            custom_agent_paths: [("cursor".into(), "/tmp/cursor-skills".into())]
                .into_iter()
                .collect(),
            theme: Some([("primary".into(), "blue".into())].into_iter().collect()),
            keymap: [("quit".into(), "q".into())].into_iter().collect(),
            proxy: Some("http://proxy:8080".into()),
            stale_after_days: 30,
            locale: Some("pt-BR".into()),
        };
        let rendered = {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    render_config_panel(
                        &config,
                        &ConfigState::default(),
                        f.area(),
                        f,
                        &I18n::new(Locale::PtBr),
                    )
                })
                .unwrap();
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|c| c.symbol().to_string())
                .collect::<String>()
        };
        assert!(rendered.contains("Configuração da TUI"));
        assert!(rendered.contains("Caminhos de agentes:"));
        assert!(rendered.contains("Sobrescritas de tema:"));
        assert!(rendered.contains("Sobrescritas de atalhos:"));
        assert!(!rendered.contains("TUI Configuration"));
    }

    #[test]
    fn pt_br_settings_panel_localizes_labels() {
        let settings = ProjectSettings {
            auto_trigger: true,
            skill_overrides: vec![SkillOverride {
                skill_name: "my-skill".into(),
                auto_trigger: false,
            }],
        };
        let state = SettingsState {
            project_path: Some("meu-projeto/.claude/settings.json".into()),
            ..SettingsState::default()
        };
        let rendered = render_settings_i18n(&settings, &state, &I18n::new(Locale::PtBr));
        assert!(rendered.contains("Configurações do Projeto"));
        assert!(rendered.contains("Auto-Disparo"));
        assert!(rendered.contains("Sobrescritas de Skill"));
        assert!(rendered.contains("meu-projeto"));
        assert!(!rendered.contains("Project Settings"));
    }
}
