//! Renders the git-backed skill sync panel: status, snapshots, init/snapshot/restore.

use ai_skill_core::SyncStatus;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::SyncState;

/// Renders the sync panel in the given area.
pub fn render_sync_panel(state: &mut SyncState, area: Rect, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(6),
        ])
        .split(area);

    render_status(state, chunks[0], f);
    render_snapshot_list(state, chunks[1], f);
    render_actions(state, chunks[2], f);
}

fn render_status(state: &SyncState, area: Rect, f: &mut Frame) {
    let status_text = match &state.status {
        Some(SyncStatus::Uninitialized) => Line::from(vec![
            Span::raw("Sync: "),
            Span::styled("Not initialized", Style::default().fg(Color::Yellow)),
        ]),
        Some(SyncStatus::Clean) => Line::from(vec![
            Span::raw("Sync: "),
            Span::styled("Clean", Style::default().fg(Color::Green)),
        ]),
        Some(SyncStatus::Dirty) => Line::from(vec![
            Span::raw("Sync: "),
            Span::styled(
                "Dirty (uncommitted changes)",
                Style::default().fg(Color::Red),
            ),
        ]),
        Some(SyncStatus::Ahead { commits }) => Line::from(vec![
            Span::raw("Sync: "),
            Span::styled(
                format!("Ahead by {commits} commit(s)"),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Some(SyncStatus::Behind { commits }) => Line::from(vec![
            Span::raw("Sync: "),
            Span::styled(
                format!("Behind by {commits} commit(s)"),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Some(SyncStatus::Diverged) => Line::from(vec![
            Span::raw("Sync: "),
            Span::styled("Diverged from remote", Style::default().fg(Color::Red)),
        ]),
        None => Line::from(Span::raw("Sync: Unknown")),
    };

    let block = Block::default().borders(Borders::ALL).title(" Status ");
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(status_text).wrap(Wrap { trim: false }),
        inner,
    );
}

fn render_snapshot_list(state: &mut SyncState, area: Rect, f: &mut Frame) {
    let items: Vec<ListItem> = if state.snapshots.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  No snapshots yet",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        state
            .snapshots
            .iter()
            .map(|s| {
                let short_id = &s.id[..7.min(s.id.len())];
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {short_id} "), Style::default().fg(Color::Cyan)),
                    Span::raw(&s.message),
                    Span::styled(
                        format!("  [{}]", s.author),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Snapshots "))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Yellow),
        )
        .highlight_symbol("> ");

    let mut list_state =
        ratatui::widgets::ListState::default().with_selected(Some(state.selected_index));
    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_actions(state: &SyncState, area: Rect, f: &mut Frame) {
    let lines: Vec<Line> = if state.creating_snapshot {
        let input = if state.snapshot_message.is_empty() {
            "  <type message, then Enter>"
        } else {
            &state.snapshot_message
        };
        vec![
            Line::from(Span::styled(
                "  Enter snapshot message:",
                Style::default().fg(Color::Green),
            )),
            Line::from(Span::raw(format!("  > {input}"))),
        ]
    } else if state.configuring_remote {
        let input = if state.remote_input.is_empty() {
            "  <type name url, then Enter>"
        } else {
            &state.remote_input
        };
        vec![
            Line::from(Span::styled(
                "  Enter remote <name> <url>:",
                Style::default().fg(Color::Green),
            )),
            Line::from(Span::raw(format!("  > {input}"))),
        ]
    } else if let Some(ref msg) = state.message {
        vec![
            Line::from(Span::styled(
                format!("  {msg}"),
                Style::default().fg(Color::Green),
            )),
            Line::from(Span::styled(
                "  Press any key to continue",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        let is_init = matches!(state.status, Some(SyncStatus::Uninitialized));
        let mut items = Vec::new();
        if is_init {
            items.push(Line::from(Span::styled(
                "  Enter  — Initialize git repository",
                Style::default().fg(Color::Blue),
            )));
        } else {
            items.push(Line::from(Span::styled(
                "  Enter  — Create snapshot",
                Style::default().fg(Color::Blue),
            )));
        }
        items.push(Line::from(Span::styled(
            "  r      — Restore selected snapshot",
            Style::default().fg(Color::Blue),
        )));
        items.push(Line::from(Span::styled(
            "  R      — Configure remote",
            Style::default().fg(Color::Blue),
        )));
        items.push(Line::from(Span::styled(
            "  p      — Push to origin",
            Style::default().fg(Color::Blue),
        )));
        items.push(Line::from(Span::styled(
            "  P      — Pull from origin",
            Style::default().fg(Color::Blue),
        )));
        items
    };

    let block = Block::default().borders(Borders::ALL).title(" Actions ");
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::Snapshot;
    use ratatui::backend::TestBackend;

    #[test]
    fn renders_without_panic() {
        let mut state = SyncState::default();
        let backend = TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                render_sync_panel(&mut state, area, f);
            })
            .unwrap();
    }

    #[test]
    fn renders_with_snapshots() {
        let mut state = SyncState {
            status: Some(SyncStatus::Clean),
            snapshots: vec![Snapshot {
                id: "abc123def456".into(),
                message: "Add code-review skill".into(),
                timestamp: "2026-07-12T10:00:00Z".into(),
                author: "User".into(),
            }],
            selected_index: 0,
            ..SyncState::default()
        };
        let backend = TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                render_sync_panel(&mut state, area, f);
            })
            .unwrap();
    }
}
