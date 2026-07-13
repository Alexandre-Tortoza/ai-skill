//! Floating command palette overlay.

use crate::app::PaletteCommand;
use crate::i18n::I18n;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

/// Renders the floating command palette over the current view.
pub fn render_command_palette(
    commands: &[PaletteCommand],
    selected: usize,
    area: Rect,
    frame: &mut Frame,
    i18n: &I18n,
) {
    let height = (commands.len() as u16).min(12) + 4;
    let popup = centered_rect(50, height, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(i18n.palette_title())
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let [list_area, footer_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);

    let items: Vec<ListItem> = commands
        .iter()
        .map(|c| ListItem::new(i18n.palette_command_label(*c)))
        .collect();
    let mut state = ListState::default();
    state.select(Some(selected.min(commands.len().saturating_sub(1))));
    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, list_area, &mut state);

    let footer = Paragraph::new(i18n.palette_hint()).style(Style::default());
    frame.render_widget(footer, footer_area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let [vertical] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [horizontal] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(vertical);
    horizontal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{I18n, Locale};
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn palette_renders_command_labels() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let commands = vec![PaletteCommand::Search, PaletteCommand::Audit];
        terminal
            .draw(|f| render_command_palette(&commands, 0, f.area(), f, &I18n::default()))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(rendered.contains("Search catalog"));
        assert!(rendered.contains("Audit report"));
        assert!(rendered.contains("Commands"));
    }

    #[test]
    fn palette_renders_portuguese_labels() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let commands = vec![PaletteCommand::Search];
        terminal
            .draw(|f| render_command_palette(&commands, 0, f.area(), f, &I18n::new(Locale::PtBr)))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(rendered.contains("Buscar no catálogo"));
    }
}
