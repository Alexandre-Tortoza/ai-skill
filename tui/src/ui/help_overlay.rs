//! Keymap help overlay rendered on top of the installed-panel.

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

const KEYMAP: &str = "\
j / ↓       move down
k / ↑       move up
Tab         cycle scope filter (all → global → project)
Enter       open skill detail
s           open catalog search
S           project settings (auto-trigger)
p           profiles / presets
F1-F4       apply phase preset (init/dev/test/release)
?           show this help
Esc         go back / close
q           quit
Ctrl-C      quit

--- in detail view ---
o           toggle skill auto-trigger

--- in settings view ---
t           toggle global auto-trigger
j/k         move in overrides list
o           toggle override auto-trigger
d           remove override
Esc         save & back";

/// Renders a centred overlay showing keyboard shortcuts.
pub fn render_help_overlay(area: Rect, frame: &mut Frame) {
    let popup = centered_rect(60, 16, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Help — Key Bindings")
        .title_style(Style::default().add_modifier(Modifier::BOLD));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let text = Paragraph::new(KEYMAP);
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
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn snapshot_help_overlay() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_help_overlay(f.area(), f)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn help_overlay_contains_key_bindings() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render_help_overlay(f.area(), f)).unwrap();
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
}
