//! Overlay showing the import chain dependency graph for a skill.

use ai_skill_adapters::ImportChainResult;
use ai_skill_core::{ReferenceType, Severity};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::style_helpers::fg;

/// Renders the import chain overlay for a skill's dependency graph.
pub fn render_import_chain(result: &ImportChainResult, area: Rect, frame: &mut Frame) {
    let popup_width = area.width * 4 / 5;
    let total_height = (result.graph.nodes.len() as u16 * 2 + result.findings.len() as u16 + 10)
        .min(area.height - 4);
    let x = area.x + (area.width - popup_width) / 2;
    let y = area.y + (area.height - total_height) / 2;
    let popup_area = Rect::new(x, y, popup_width, total_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Import Chain ")
        .borders(Borders::ALL)
        .border_style(fg(Color::Cyan));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::vertical([
        Constraint::Length(2), // summary
        Constraint::Min(1),    // dependency graph
        Constraint::Length(3), // findings header + findings (if any)
    ])
    .split(inner);

    // Summary line
    let total_nodes = result.graph.nodes.len();
    let total_edges = result.graph.edges.len();
    let finding_count = result.findings.len();
    let summary = Paragraph::new(format!(
        "Files: {total_nodes}  |  References: {total_edges}  |  Findings: {finding_count}   [Esc to close]"
    ))
    .style(fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
    frame.render_widget(summary, chunks[0]);

    // Dependency graph as a tree list
    let mut items: Vec<ListItem> = Vec::new();
    for node in &result.graph.nodes {
        let label = if node.has_findings {
            format!(" ⚠ {}  ({} finding(s))", node.path, node.finding_count)
        } else {
            format!("   {}", node.path)
        };
        let style = if node.has_findings {
            fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            fg(Color::Reset)
        };
        items.push(ListItem::new(Line::from(Span::styled(label, style))));
    }

    for edge in &result.graph.edges {
        let from = result
            .graph
            .nodes
            .iter()
            .find(|n| n.id == edge.from)
            .map(|n| n.path.as_str())
            .unwrap_or("?");
        let to = result
            .graph
            .nodes
            .iter()
            .find(|n| n.id == edge.to)
            .map(|n| n.path.as_str())
            .unwrap_or("?");
        let ref_type = match edge.reference.reference_type {
            ReferenceType::Source => "source",
            ReferenceType::Import => "import",
            ReferenceType::Require => "require",
            ReferenceType::Exec => "exec",
            ReferenceType::Npx => "npx",
            ReferenceType::Unknown => "ref",
        };
        let label = format!(
            "  └─ [{ref_type}] {from} → {to} (line {})",
            edge.reference.source_line
        );
        items.push(ListItem::new(Line::from(Span::styled(
            label,
            fg(Color::DarkGray),
        ))));
    }

    let graph_list = List::new(items);
    frame.render_widget(graph_list, chunks[1]);

    // Findings section
    if !result.findings.is_empty() {
        let mut finding_lines: Vec<ListItem> = Vec::new();
        for f in &result.findings {
            let severity_label = match f.finding.severity {
                Severity::High => "[HIGH]",
                Severity::Medium => "[MED] ",
            };
            let color = match f.finding.severity {
                Severity::High => Color::Red,
                Severity::Medium => Color::Yellow,
            };
            let text = format!(
                "{severity_label} {:?} — {} (line {}) in {}",
                f.finding.category, f.finding.detail, f.finding.line, f.origin_file,
            );
            finding_lines.push(ListItem::new(Line::from(Span::styled(text, fg(color)))));
        }
        let findings_list = List::new(finding_lines).block(
            Block::default()
                .title(" Findings ")
                .borders(Borders::TOP)
                .border_style(fg(Color::Red)),
        );
        frame.render_widget(findings_list, chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_adapters::ImportChainResult;
    use ai_skill_core::{
        DepEdge, DepGraph, DepNode, ImportChainFinding, ImportRef, ReferenceType, ScanCategory,
        ScanFinding, Severity,
    };
    use ratatui::{Terminal, backend::TestBackend};

    fn make_result() -> ImportChainResult {
        ImportChainResult {
            graph: DepGraph {
                nodes: vec![
                    DepNode {
                        id: 0,
                        path: "SKILL.md".to_string(),
                        has_findings: false,
                        finding_count: 0,
                    },
                    DepNode {
                        id: 1,
                        path: "scripts/setup.sh".to_string(),
                        has_findings: true,
                        finding_count: 1,
                    },
                ],
                edges: vec![DepEdge {
                    from: 0,
                    to: 1,
                    reference: ImportRef {
                        source_line: 3,
                        reference_type: ReferenceType::Source,
                        referenced_path: "./scripts/setup.sh".to_string(),
                    },
                }],
            },
            findings: vec![ImportChainFinding {
                finding: ScanFinding {
                    severity: Severity::High,
                    category: ScanCategory::DangerousShellPattern,
                    detail: "eval detected".to_string(),
                    line: 2,
                },
                origin_file: "scripts/setup.sh".to_string(),
                import_chain: vec![],
            }],
        }
    }

    #[test]
    fn renders_without_panic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let result = make_result();
        terminal
            .draw(|f| render_import_chain(&result, f.area(), f))
            .unwrap();
    }

    #[test]
    fn snapshot_import_chain() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let result = make_result();
        terminal
            .draw(|f| render_import_chain(&result, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }
}
