//! Bottom status bar showing keyboard hints per view.

use ratatui::{Frame, layout::Rect, style::Color, widgets::Paragraph};

use crate::{app::View, ui::style_helpers::fg_bg};

/// Renders a single-line status bar with view-specific key hints.
pub fn render_status_bar(view: &View, area: Rect, frame: &mut Frame) {
    let hints = match view {
        View::List => "j/k  d dis  e edit  r rm  u up  a adopt  c new  A audit  s search  ? quit",
        View::Detail => "j/k scroll  Esc back  q quit",
        View::Search => "type search  j/k move  Enter install  Esc back",
        View::Help => "Esc close",
        View::Confirm => "y confirm  n / Esc cancel",
        View::InstallWizard => "Tab scope  Space agent  Enter confirm  Esc back",
        View::ScanReport => "Enter install anyway  Esc cancel",
        View::Profiles => "j/k move  a activate  f from-current  d delete  Esc back",
        View::CreateWizard => "Tab next field  Enter create (on Preview)  Esc cancel",
        View::Editor => "Tab next field  Enter save  Esc cancel",
        View::Audit => "Esc back",
    };

    let bar = Paragraph::new(hints).style(fg_bg(Color::Black, Color::DarkGray));
    frame.render_widget(bar, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_bar(view: View) -> String {
        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&view, f.area(), f))
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
    fn snapshot_list_view() {
        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&View::List, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_detail_view() {
        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&View::Detail, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn list_view_shows_quit_hint() {
        assert!(render_bar(View::List).contains("quit"));
    }

    #[test]
    fn detail_view_shows_back_hint() {
        assert!(render_bar(View::Detail).contains("back"));
    }

    #[test]
    fn search_view_shows_search_hint() {
        assert!(render_bar(View::Search).contains("search"));
    }

    #[test]
    fn help_view_shows_close_hint() {
        assert!(render_bar(View::Help).contains("close"));
    }
}
