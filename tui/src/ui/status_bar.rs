//! Bottom status bar showing keyboard hints per view.

use ai_skill_core::BudgetWarning;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{app::View, ui::style_helpers::fg_bg};

/// Renders a single-line status bar with view-specific key hints and optional budget warning.
pub fn render_status_bar(
    view: &View,
    area: Rect,
    frame: &mut Frame,
    budget_warning: Option<&BudgetWarning>,
    hot_reload_active: bool,
) {
    let hints = match view {
        View::List => "j/k  d  e  n  r  u  a  c  A aud  B bud  S set  s srch  F1-F4  ? quit",
        View::Detail => "j/k scroll  Esc back  q quit",
        View::Search => "type search  j/k move  Enter install  Esc back",
        View::Help => "Esc close",
        View::Confirm => "y confirm  n / Esc cancel",
        View::InstallWizard => "Tab scope  Space agent  Enter confirm  Esc back",
        View::ScanReport => "Enter install anyway  Esc cancel",
        View::Profiles => "j/k move  a activate  e export  f from-current  d delete  Esc back",
        View::CreateWizard => "Tab next field  Enter create (on Preview)  Esc cancel",
        View::Editor => "Tab next field  Enter save  Esc cancel",
        View::Audit => "Esc back",
        View::Budget => "Esc back",
        View::Settings => "t toggle  j/k move  o toggle override  d remove  Esc save & back",
        View::ImportChain => "Esc close",
        View::SshRemote => "j/k move  Enter connect  Esc back",
        View::Bundles => "j/k move  Enter install  Esc back",
        View::Sync => "j/k move  Enter init/snap  r rstor  R remote  p push  P pull  Esc back",
    };

    let warning_span = match budget_warning {
        Some(BudgetWarning::None) | None => None,
        Some(BudgetWarning::Approaching { pct }) => Some(Span::styled(
            format!(" ! {pct:.0}%"),
            Style::default().fg(Color::Yellow),
        )),
        Some(BudgetWarning::Critical { pct }) => Some(Span::styled(
            format!(" !! {pct:.0}%"),
            Style::default()
                .fg(Color::Red)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Some(BudgetWarning::OverBudget { pct, .. }) => Some(Span::styled(
            format!(" OVER {pct:.0}%"),
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )),
    };

    let mut spans = vec![Span::raw(hints)];
    if hot_reload_active {
        spans.push(Span::styled(
            "  reload:on",
            Style::default().fg(Color::LightGreen),
        ));
    }
    if let Some(span) = warning_span {
        spans.push(span);
    }
    let content = Line::from(spans);

    let bar = Paragraph::new(content).style(fg_bg(Color::Black, Color::DarkGray));
    frame.render_widget(bar, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_bar(view: View) -> String {
        let backend = TestBackend::new(84, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&view, f.area(), f, None, false))
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
        let backend = TestBackend::new(84, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&View::List, f.area(), f, None, false))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_detail_view() {
        let backend = TestBackend::new(84, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&View::Detail, f.area(), f, None, false))
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

    #[test]
    fn budget_warning_approaching_shows_pct() {
        let backend = TestBackend::new(90, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render_status_bar(
                    &View::Detail,
                    f.area(),
                    f,
                    Some(&BudgetWarning::Approaching { pct: 85.0 }),
                    false,
                )
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("85%"));
    }

    #[test]
    fn budget_warning_over_budget_shows_pct() {
        let backend = TestBackend::new(90, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render_status_bar(
                    &View::Detail,
                    f.area(),
                    f,
                    Some(&BudgetWarning::OverBudget {
                        pct: 120.0,
                        truncated_skills: 2,
                    }),
                    false,
                )
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("120%"));
    }

    #[test]
    fn hot_reload_active_shows_reload_indicator() {
        let backend = TestBackend::new(90, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&View::List, f.area(), f, None, true))
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("reload:on"));
    }
}
