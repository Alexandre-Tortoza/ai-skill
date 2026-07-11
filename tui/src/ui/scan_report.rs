//! Overlay that shows security scan findings before confirming an install.

use ai_skill_core::{ScanFinding, Severity};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::style_helpers::fg;

/// Renders an overlay listing security scan findings before install confirmation.
pub fn render_scan_report(findings: &[ScanFinding], area: Rect, frame: &mut Frame) {
    let popup_width = area.width * 3 / 4;
    let popup_height = (findings.len() as u16 + 6).min(area.height - 4);
    let x = area.x + (area.width - popup_width) / 2;
    let y = area.y + (area.height - popup_height) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Security Findings ")
        .borders(Borders::ALL)
        .border_style(fg(Color::Red));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);

    let items: Vec<ListItem> = findings
        .iter()
        .map(|f| {
            let severity_label = match f.severity {
                Severity::High => "[HIGH]",
                Severity::Medium => "[MED] ",
            };
            let color = match f.severity {
                Severity::High => Color::Red,
                Severity::Medium => Color::Yellow,
            };
            let category = format!("{:?}", f.category);
            let text = format!(
                "{severity_label} {category} — {} (line {})",
                f.detail, f.line
            );
            ListItem::new(Line::from(Span::styled(text, fg(color))))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[0]);

    let footer = Paragraph::new("Enter to install anyway  |  Esc to cancel")
        .style(fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
    frame.render_widget(footer, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::{ScanCategory, ScanFinding, Severity};
    use ratatui::{Terminal, backend::TestBackend};

    fn make_finding(severity: Severity) -> ScanFinding {
        ScanFinding {
            severity,
            category: ScanCategory::DangerousShellPattern,
            detail: "rm -rf detected".to_string(),
            line: 1,
        }
    }

    #[test]
    fn renders_without_panic_with_findings() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let findings = vec![make_finding(Severity::High)];
        terminal
            .draw(|f| render_scan_report(&findings, f.area(), f))
            .unwrap();
    }

    #[test]
    fn renders_without_panic_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_scan_report(&[], f.area(), f))
            .unwrap();
    }

    #[test]
    fn snapshot_single_high_finding() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let findings = vec![make_finding(Severity::High)];
        terminal
            .draw(|f| render_scan_report(&findings, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }
}
