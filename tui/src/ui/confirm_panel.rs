//! Confirmation overlay that shows a command preview and asks y/n.

use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::style_helpers::{bg, fg};

/// Computes a centred rectangle within `area` at `percent_x` width and given height.
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_width = area.width * percent_x / 100;
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: height.min(area.height),
    }
}

/// Renders a centred confirmation dialog with the action preview.
pub fn render_confirm_panel(preview: &str, area: Rect, frame: &mut Frame) {
    let popup = centered_rect(70, 7, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Confirm")
        .style(bg(Color::DarkGray));

    let text = vec![
        Line::raw(""),
        Line::from(vec![Span::raw(preview)]),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  Press "),
            Span::styled("y", fg(Color::Green)),
            Span::raw(" to confirm, "),
            Span::styled("n", fg(Color::Red)),
            Span::raw(" / Esc to cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, popup);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render(preview: &str) -> String {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_confirm_panel(preview, f.area(), f))
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
    fn snapshot_confirm_panel() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_confirm_panel("npx skills disable /tmp/my-skill", f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn confirm_text_appears_in_render() {
        let rendered = render("npx skills remove /skills/omarchy");
        assert!(rendered.contains("Confirm"));
        assert!(rendered.contains("npx skills remove"));
    }

    #[test]
    fn y_and_n_hints_appear() {
        let rendered = render("some action");
        assert!(rendered.contains('y'));
        assert!(rendered.contains('n'));
    }
}
