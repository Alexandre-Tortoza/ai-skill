//! Overlay that shows security scan findings before confirming an install.

use crate::i18n::I18n;
use ai_skill_core::{ScanFinding, Severity};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::style_helpers::fg;
use super::theme::{Theme, ThemeSlot};

/// Renders an overlay listing security scan findings before install confirmation.
pub fn render_scan_report(
    findings: &[ScanFinding],
    theme: &Theme,
    area: Rect,
    frame: &mut Frame,
    i18n: &I18n,
) {
    let popup_width = area.width * 3 / 4;
    let popup_height = (findings.len() as u16 + 6).min(area.height - 4);
    let x = area.x + (area.width - popup_width) / 2;
    let y = area.y + (area.height - popup_height) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(i18n.scan_title())
        .borders(Borders::ALL)
        .border_style(fg(theme.color(ThemeSlot::Error)));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);

    let items: Vec<ListItem> = findings
        .iter()
        .map(|f| {
            let severity_label = match f.severity {
                Severity::High => i18n.severity_high(),
                Severity::Medium => i18n.severity_medium(),
            };
            let color = match f.severity {
                Severity::High => theme.color(ThemeSlot::Error),
                Severity::Medium => theme.color(ThemeSlot::Warning),
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

    let footer = Paragraph::new(i18n.scan_footer())
        .style(fg(theme.color(ThemeSlot::Muted)).add_modifier(Modifier::ITALIC));
    frame.render_widget(footer, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{I18n, Locale};
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

    fn render(findings: &[ScanFinding], i18n: &I18n) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_scan_report(findings, &Theme::default(), f.area(), f, i18n))
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
    fn renders_without_panic_with_findings() {
        let findings = vec![make_finding(Severity::High)];
        render(&findings, &I18n::default());
    }

    #[test]
    fn renders_without_panic_empty() {
        render(&[], &I18n::default());
    }

    #[test]
    fn snapshot_single_high_finding() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let findings = vec![make_finding(Severity::High)];
        terminal
            .draw(|f| {
                render_scan_report(&findings, &Theme::default(), f.area(), f, &I18n::default())
            })
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn pt_br_localizes_title_and_footer() {
        let findings = vec![make_finding(Severity::High)];
        let rendered = render(&findings, &I18n::new(Locale::PtBr));
        assert!(rendered.contains("Achados de Segurança"));
        assert!(rendered.contains("instalar mesmo assim"));
        assert!(!rendered.contains("Security Findings"));
    }
}
