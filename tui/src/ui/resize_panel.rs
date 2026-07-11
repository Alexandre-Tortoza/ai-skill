//! Resize guard that shows a "terminal too small" message.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Minimum terminal width required by the TUI.
pub const MIN_WIDTH: u16 = 80;
/// Minimum terminal height required by the TUI.
pub const MIN_HEIGHT: u16 = 24;

/// Returns `true` if the terminal is too small to render the TUI.
pub fn is_too_small(area: Rect) -> bool {
    area.width < MIN_WIDTH || area.height < MIN_HEIGHT
}

/// Renders a centred "terminal too small" message.
pub fn render_resize_panel(area: Rect, frame: &mut Frame) {
    let message = format!(
        "Terminal too small\n\nMinimum: {MIN_WIDTH}x{MIN_HEIGHT}\nCurrent: {}x{}\n\nResize the terminal to continue.",
        area.width, area.height
    );

    let widget = Paragraph::new(message)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Resize"));

    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render(width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_resize_panel(f.area(), f)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    }

    #[test]
    fn width_below_minimum_is_too_small() {
        assert!(is_too_small(Rect::new(0, 0, 79, 24)));
    }

    #[test]
    fn height_below_minimum_is_too_small() {
        assert!(is_too_small(Rect::new(0, 0, 80, 23)));
    }

    #[test]
    fn exact_minimum_is_not_too_small() {
        assert!(!is_too_small(Rect::new(0, 0, 80, 24)));
    }

    #[test]
    fn resize_message_includes_current_size() {
        let rendered = render(60, 12);

        assert!(rendered.contains("Terminal too small"));
        assert!(rendered.contains("Minimum: 80x24"));
        assert!(rendered.contains("Current: 60x12"));
    }
}
