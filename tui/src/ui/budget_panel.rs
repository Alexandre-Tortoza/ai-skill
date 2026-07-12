//! Panel showing context budget usage across all skills.

use ai_skill_core::ContextBudget;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

/// Renders the context budget overview panel.
pub fn render_budget_panel(budget: &ContextBudget, area: Rect, frame: &mut Frame) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .title("Context Budget");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(inner);

    // Gauge
    let gauge_color = if budget.usage_ratio >= 1.0 {
        Color::Red
    } else if budget.usage_ratio >= 0.95 {
        Color::Yellow
    } else if budget.usage_ratio >= 0.80 {
        Color::LightYellow
    } else {
        Color::Green
    };
    let gauge_label = format!(
        "{:.0}%  ({} / {} chars)",
        budget.usage_ratio * 100.0,
        budget.used,
        budget.limit
    );
    let gauge = Gauge::default()
        .ratio(budget.usage_ratio.min(1.0))
        .label(gauge_label)
        .gauge_style(Style::default().fg(gauge_color).bg(Color::DarkGray));
    frame.render_widget(gauge, chunks[0]);

    // Summary line
    let warning = ai_skill_core::classify_budget(budget);
    let warning_str = match &warning {
        ai_skill_core::BudgetWarning::None => String::new(),
        ai_skill_core::BudgetWarning::Approaching { pct } => {
            format!("Approaching budget limit ({pct:.0}%)")
        }
        ai_skill_core::BudgetWarning::Critical { pct } => {
            format!("Critical budget usage ({pct:.0}%)")
        }
        ai_skill_core::BudgetWarning::OverBudget {
            pct,
            truncated_skills,
        } => {
            format!("OVER BUDGET ({pct:.0}%) — ~{truncated_skills} skills may be truncated")
        }
    };
    let summary_color = match &warning {
        ai_skill_core::BudgetWarning::None => Color::Green,
        ai_skill_core::BudgetWarning::Approaching { .. } => Color::Yellow,
        ai_skill_core::BudgetWarning::Critical { .. } => Color::Red,
        ai_skill_core::BudgetWarning::OverBudget { .. } => Color::LightRed,
    };
    let summary = Paragraph::new(Line::from(Span::styled(
        warning_str,
        Style::default()
            .fg(summary_color)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(summary, chunks[1]);

    // Per-skill breakdown
    let mut lines: Vec<Line> = budget
        .skill_costs
        .iter()
        .map(|cost| {
            let pct_of_total = if budget.used > 0 {
                cost.char_count as f64 / budget.used as f64 * 100.0
            } else {
                0.0
            };
            Line::from(vec![
                Span::styled(
                    format!("  {:<20}", cost.name),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "  {} chars  ~{} tok  ({pct_of_total:.1}%)",
                    cost.char_count, cost.estimated_tokens
                )),
            ])
        })
        .collect();
    if lines.is_empty() {
        lines.push(Line::from(Span::raw("  (no skills loaded)")));
    }
    let list = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(list, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::SkillCost;
    use ratatui::{Terminal, backend::TestBackend};

    fn budget(used: usize, ratio: f64, costs: Vec<SkillCost>) -> ContextBudget {
        ContextBudget {
            limit: 16_384,
            used,
            available: 16_384usize.saturating_sub(used),
            usage_ratio: ratio,
            skill_costs: costs,
        }
    }

    fn render_budget(budget: &ContextBudget) -> String {
        let backend = TestBackend::new(70, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_budget_panel(budget, f.area(), f))
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
    fn snapshot_empty_budget() {
        let b = budget(0, 0.0, vec![]);
        let backend = TestBackend::new(70, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_budget_panel(&b, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_over_budget() {
        let costs = vec![
            SkillCost {
                name: "big-skill".into(),
                char_count: 12_000,
                estimated_tokens: 3_000,
            },
            SkillCost {
                name: "small".into(),
                char_count: 5_000,
                estimated_tokens: 1_250,
            },
        ];
        let b = budget(17_000, 1.04, costs);
        let rendered = render_budget(&b);
        assert!(rendered.contains("OVER BUDGET"));
        assert!(rendered.contains("big-skill"));
        assert!(rendered.contains("small"));
    }

    #[test]
    fn empty_budget_shows_no_skills_message() {
        let b = budget(0, 0.0, vec![]);
        let rendered = render_budget(&b);
        assert!(rendered.contains("no skills loaded"));
    }

    #[test]
    fn gauge_shows_usage_percentage() {
        let b = budget(8_192, 0.5, vec![]);
        let rendered = render_budget(&b);
        assert!(rendered.contains("50%"));
    }
}
