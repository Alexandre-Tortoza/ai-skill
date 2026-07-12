//! Panel for browsing and installing predefined skill bundles.

use crate::app::BundleState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Renders the bundles panel with a list of bundles on the left and details on the right.
pub fn render_bundles_panel(state: &BundleState, area: Rect, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main = chunks[0];
    let msg_area = chunks[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main);

    render_bundle_list(state, inner[0], frame);
    render_bundle_detail(state, inner[1], frame);

    if let Some(ref msg) = state.result_message {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                msg,
                Style::default().fg(Color::Green),
            ))),
            msg_area,
        );
    }
}

fn render_bundle_list(state: &BundleState, area: Rect, frame: &mut Frame) {
    let items: Vec<ListItem> = state
        .bundles
        .iter()
        .map(|b| ListItem::new(b.name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Bundles ").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    frame.render_widget(list, area);
}

fn render_bundle_detail(state: &BundleState, area: Rect, frame: &mut Frame) {
    let block = Block::default()
        .title(" Bundle Details ")
        .borders(Borders::ALL);

    if let Some(bundle) = state.bundles.get(state.selected_index) {
        let mut lines = vec![
            Line::from(Span::styled(
                &bundle.name,
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::raw("")),
        ];
        if !bundle.description.is_empty() {
            lines.push(Line::from(Span::raw(&bundle.description)));
            lines.push(Line::from(Span::raw("")));
        }
        lines.push(Line::from(Span::styled(
            format!("Skills ({}):", bundle.skills.len()),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for skill in &bundle.skills {
            lines.push(Line::from(Span::raw(format!("  • {skill}"))));
        }
        frame.render_widget(Paragraph::new(lines).block(block), area);
    } else {
        frame.render_widget(Paragraph::new("No bundles available.").block(block), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::Bundle;
    use ratatui::{Terminal, backend::TestBackend};

    fn sample_bundles() -> Vec<Bundle> {
        vec![
            Bundle {
                name: "frontend".into(),
                description: "Frontend dev skills".into(),
                skills: vec!["react-rules".into(), "typescript-rules".into()],
            },
            Bundle {
                name: "ops".into(),
                description: "DevOps skills".into(),
                skills: vec!["docker".into()],
            },
        ]
    }

    #[test]
    fn renders_without_panic_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = BundleState::default();
        terminal
            .draw(|f| render_bundles_panel(&state, f.area(), f))
            .unwrap();
    }

    #[test]
    fn renders_without_panic_with_bundles() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = BundleState {
            bundles: sample_bundles(),
            ..BundleState::default()
        };
        terminal
            .draw(|f| render_bundles_panel(&state, f.area(), f))
            .unwrap();
    }

    #[test]
    fn snapshot_bundles_panel() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = BundleState {
            bundles: sample_bundles(),
            ..BundleState::default()
        };
        terminal
            .draw(|f| render_bundles_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn bundle_name_appears_in_detail() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = BundleState {
            bundles: sample_bundles(),
            selected_index: 0,
            ..BundleState::default()
        };
        terminal
            .draw(|f| render_bundles_panel(&state, f.area(), f))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(rendered.contains("frontend"));
    }
}
