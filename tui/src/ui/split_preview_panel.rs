//! Split view for the list screen: skill list on the left, README preview on the right.

use ai_skill_core::{Skill, SkillDoc};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::ListUiState;

use super::installed_panel::render_installed_panel;

/// Renders the list screen as two panes: skills (left) and a doc preview (right).
///
/// The preview shows the selected skill's README/SKILL.md, or a placeholder
/// when nothing is selected.
pub fn render_split_preview(
    skills: &[&Skill],
    state: &ListUiState,
    preview: Option<&SkillDoc>,
    preview_scroll: u16,
    area: Rect,
    frame: &mut Frame,
) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    render_installed_panel(skills, state, chunks[0], frame);

    let (title, body) = match preview {
        Some(doc) => (doc.title.as_str(), doc.content.as_str()),
        None => ("Preview", "(select a skill to preview its README)"),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Preview — {title}"));
    let inner = block.inner(chunks[1]);
    frame.render_widget(block, chunks[1]);

    let para = Paragraph::new(body)
        .wrap(Wrap { trim: false })
        .scroll((preview_scroll, 0))
        .style(Style::default().add_modifier(Modifier::empty()));
    frame.render_widget(para, inner);
}
