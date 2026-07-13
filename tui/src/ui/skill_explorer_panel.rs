//! Skill directory explorer: file tree on the left, selected file on the right.

use ai_skill_core::{SkillFileKind, SkillTreeNode};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// Renders the explorer screen: a directory tree (left) and file content (right).
pub fn render_skill_explorer(
    nodes: &[SkillTreeNode],
    selected_index: usize,
    file_content: Option<&str>,
    title: &str,
    scroll: u16,
    area: Rect,
    frame: &mut Frame,
) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    render_tree(nodes, selected_index, title, chunks[0], frame);
    render_file(file_content, title, scroll, chunks[1], frame);
}

fn render_tree(
    nodes: &[SkillTreeNode],
    selected_index: usize,
    title: &str,
    area: Rect,
    frame: &mut Frame,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Files — {title}"));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = nodes
        .iter()
        .map(|node| {
            let indent = "  ".repeat(node.depth);
            let icon = if node.is_dir {
                if node.is_subskill { "▾◈" } else { "▾" }
            } else {
                match node.kind {
                    SkillFileKind::Markdown => "▸",
                    SkillFileKind::Script => "$",
                    SkillFileKind::Config => "#",
                    SkillFileKind::Other => "·",
                }
            };
            let label = format!("{indent}{icon} {}", node.name);
            ListItem::new(label)
        })
        .collect();

    let mut list_state = ListState::default();
    if !nodes.is_empty() {
        list_state.select(Some(selected_index));
    }

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_file(content: Option<&str>, title: &str, scroll: u16, area: Rect, frame: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Content — {title}"));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let body = content.unwrap_or("(no selection)");
    let para = Paragraph::new(body)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(para, inner);
}
