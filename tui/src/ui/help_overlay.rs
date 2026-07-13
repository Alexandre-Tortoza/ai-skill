//! Keymap help overlay rendered on top of the installed-panel.

use crate::i18n::I18n;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Renders a centred overlay showing keyboard shortcuts.
pub fn render_help_overlay(area: Rect, frame: &mut Frame, i18n: &I18n) {
    let popup = centered_rect(60, 16, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(i18n.help_title())
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let text = Paragraph::new(i18n.help_text());
    frame.render_widget(text, inner);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
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
    fn snapshot_help_overlay() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_help_overlay(f.area(), f, &I18n::default()))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn help_overlay_contains_key_bindings() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_help_overlay(f.area(), f, &I18n::default()))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(rendered.contains("quit"));
        assert!(rendered.contains("search"));
        assert!(rendered.contains("Tab"));
    }

    #[test]
    fn help_overlay_pt_br_shows_portuguese() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_help_overlay(f.area(), f, &I18n::new(Locale::PtBr)))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(rendered.contains("sair"));
        assert!(rendered.contains("Ajuda"));
    }
}
