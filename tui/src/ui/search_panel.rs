//! Catalog search panel with query input and results list.

use ai_skill_core::CatalogEntry;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::SearchState;

/// Renders the catalog search panel with query input and results list.
pub fn render_search_panel(state: &SearchState, area: Rect, frame: &mut Frame) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .title("Search Catalog");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(inner);

    // Input row
    let cursor = format!("{}│", state.query);
    let input = Paragraph::new(cursor).block(Block::default().borders(Borders::ALL).title("Query"));
    frame.render_widget(input, chunks[0]);

    // Results + preview
    let halves = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    let result_items: Vec<ListItem> = state
        .results
        .iter()
        .map(|e| ListItem::new(e.name.as_str()))
        .collect();
    let mut list_state = ListState::default();
    if !state.results.is_empty() {
        list_state.select(Some(state.selected_index));
    }
    let results_list = List::new(result_items)
        .block(Block::default().borders(Borders::ALL).title("Results"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(results_list, halves[0], &mut list_state);

    let preview_text = if let Some(err) = &state.error {
        format!("Error: {err}")
    } else {
        state
            .results
            .get(state.selected_index)
            .map(format_preview)
            .unwrap_or_else(|| "No results".to_string())
    };
    let preview = Paragraph::new(preview_text)
        .block(Block::default().borders(Borders::ALL).title("Preview"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(preview, halves[1]);
}

fn format_preview(entry: &CatalogEntry) -> String {
    let mut s = format!("name: {}\n\n{}", entry.name, entry.description);
    if let Some(url) = &entry.url {
        s.push_str(&format!("\n\nurl: {url}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::CatalogEntry;
    use ratatui::{Terminal, backend::TestBackend};

    fn entry(name: &str, desc: &str) -> CatalogEntry {
        CatalogEntry {
            name: name.to_string(),
            description: desc.to_string(),
            url: None,
        }
    }

    fn render_to_string(state: &SearchState) -> String {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_search_panel(state, f.area(), f))
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
    fn snapshot_empty_query() {
        let state = SearchState::default();
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_search_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_with_results() {
        let state = SearchState {
            query: "om".to_string(),
            results: vec![entry("omarchy", "WM skill"), entry("other", "Desc")],
            selected_index: 0,
            error: None,
        };
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_search_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn error_state_shows_error_message() {
        let state = SearchState {
            query: "x".to_string(),
            results: vec![],
            selected_index: 0,
            error: Some("npx not found".to_string()),
        };
        let rendered = render_to_string(&state);
        assert!(rendered.contains("npx not found"));
    }

    #[test]
    fn query_cursor_appears_in_input() {
        let state = SearchState {
            query: "abc".to_string(),
            ..Default::default()
        };
        let rendered = render_to_string(&state);
        assert!(rendered.contains("abc"));
    }
}
