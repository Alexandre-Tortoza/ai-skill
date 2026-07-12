//! SSH remote management panel for inspecting skills on remote machines.

use crate::app::SshState;
use ai_skill_core::ConnectionStatus;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Renders the SSH remote management panel.
pub fn render_ssh_panel(state: &SshState, area: Rect, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title = Block::default()
        .borders(Borders::ALL)
        .title(" SSH Remote Management ")
        .title_style(Style::default().add_modifier(Modifier::BOLD));

    let inner = title.inner(chunks[1]);
    frame.render_widget(title, chunks[1]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    render_host_list(state, main_chunks[0], frame);
    render_host_detail(state, main_chunks[1], frame);
}

fn render_host_list(state: &SshState, area: Rect, frame: &mut Frame) {
    let items: Vec<ListItem> = state
        .hosts
        .iter()
        .enumerate()
        .map(|(i, host)| {
            let prefix = if i == state.selected_index {
                "▸ "
            } else {
                "  "
            };
            let label = format!("{}{}", prefix, host.label);
            ListItem::new(Line::from(Span::raw(label)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Hosts"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(list, area);
}

fn render_host_detail(state: &SshState, area: Rect, frame: &mut Frame) {
    let mut lines = vec![];

    if let Some(host) = state.hosts.get(state.selected_index) {
        lines.push(Line::from(Span::styled(
            format!("Host: {}", host.label),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::raw(format!("SSH: {}", host.host))));
        if let Some(port) = host.port {
            lines.push(Line::from(Span::raw(format!("Port: {port}"))));
        }
        lines.push(Line::from(Span::raw("")));

        if let Some(ref status) = state.connection_status {
            let (label, color) = match status {
                ConnectionStatus::Connected => ("Connected", Color::Green),
                ConnectionStatus::Refused => ("Refused", Color::Red),
                ConnectionStatus::Timeout => ("Timeout", Color::Yellow),
                ConnectionStatus::Unreachable => ("Unreachable", Color::DarkGray),
            };
            lines.push(Line::from(vec![
                Span::raw("Status: "),
                Span::styled(label, Style::default().fg(color)),
            ]));
            lines.push(Line::from(Span::raw("")));
        }

        if let Some(ref error) = state.error {
            lines.push(Line::from(Span::styled(
                format!("Error: {error}"),
                Style::default().fg(Color::Red),
            )));
            lines.push(Line::from(Span::raw("")));
        }

        if !state.skills.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("Skills ({}):", state.skills.len()),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for skill in &state.skills {
                let prefix = if skill.managed { " " } else { " ⚠" };
                lines.push(Line::from(Span::raw(format!("  {prefix} {}", skill.name))));
            }
        } else if state.connection_status == Some(ConnectionStatus::Connected) {
            lines.push(Line::from(Span::raw("No skills found.")));
        }
    } else {
        lines.push(Line::from(Span::raw("No hosts configured.")));
    }

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Details"));
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::{RemoteHost, RemoteSkill};
    use ratatui::{Terminal, backend::TestBackend};

    fn sample_hosts() -> Vec<RemoteHost> {
        vec![
            RemoteHost::new("dev-box", "dev.example.com"),
            RemoteHost::new("prod", "prod.example.com"),
        ]
    }

    #[test]
    fn renders_without_panic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = SshState {
            hosts: sample_hosts(),
            ..SshState::default()
        };
        terminal
            .draw(|f| render_ssh_panel(&state, f.area(), f))
            .unwrap();
    }

    #[test]
    fn snapshot_ssh_panel_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = SshState::default();
        terminal
            .draw(|f| render_ssh_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_ssh_panel_connected() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = SshState {
            hosts: sample_hosts(),
            connection_status: Some(ConnectionStatus::Connected),
            skills: vec![
                RemoteSkill {
                    name: "git-workflow".into(),
                    path: "~/.claude/skills/git-workflow".into(),
                    managed: true,
                },
                RemoteSkill {
                    name: "review".into(),
                    path: "~/.claude/skills/review.disabled".into(),
                    managed: false,
                },
            ],
            selected_index: 0,
            ..SshState::default()
        };
        terminal
            .draw(|f| render_ssh_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }
}
