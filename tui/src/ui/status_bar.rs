//! Bottom status bar showing keyboard hints per view.

use crate::{app::View, i18n::I18n, ui::style_helpers::fg_bg};
use ai_skill_core::BudgetWarning;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// Renders a single-line status bar with view-specific key hints and optional budget warning.
pub fn render_status_bar(
    view: &View,
    area: Rect,
    frame: &mut Frame,
    budget_warning: Option<&BudgetWarning>,
    hot_reload_active: bool,
    i18n: &I18n,
) {
    let hints = i18n.status_hint(view);

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
            i18n.reload_indicator(),
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
    use crate::i18n::{I18n, Locale};
    use ratatui::{Terminal, backend::TestBackend};

    fn render_bar(view: View) -> String {
        let backend = TestBackend::new(84, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&view, f.area(), f, None, false, &I18n::default()))
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
            .draw(|f| render_status_bar(&View::List, f.area(), f, None, false, &I18n::default()))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_detail_view() {
        let backend = TestBackend::new(84, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&View::Detail, f.area(), f, None, false, &I18n::default()))
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
                    &I18n::default(),
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
                    &I18n::default(),
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
            .draw(|f| render_status_bar(&View::List, f.area(), f, None, true, &I18n::default()))
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = buf.content().iter().map(|c| c.symbol()).collect();
        assert!(text.contains("reload:on"));
    }

    #[test]
    fn pt_br_list_view_shows_portuguese_quit() {
        let rendered = render_bar_locale(View::List, &I18n::new(Locale::PtBr));
        assert!(rendered.contains("sair"));
        assert!(!rendered.contains("quit"));
    }

    fn render_bar_locale(view: View, i18n: &I18n) -> String {
        let backend = TestBackend::new(90, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_status_bar(&view, f.area(), f, None, false, i18n))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    }
}
