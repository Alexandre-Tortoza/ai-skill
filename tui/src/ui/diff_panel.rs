//! Panel that renders an upstream diff for a skill.

use crate::i18n::I18n;
use ai_skill_core::{DiffError, DiffLineKind, Skill, SkillDiff};
use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::style_helpers::fg;
use super::theme::{Theme, ThemeSlot};

/// Renders the upstream diff of a skill's manifest, or a status/error message.
pub fn render_diff_panel(
    skill: &Skill,
    diff_result: &Result<SkillDiff, DiffError>,
    scroll: u16,
    theme: &Theme,
    area: Rect,
    frame: &mut Frame,
    i18n: &I18n,
) {
    let title = format!(" Upstream Diff — {} ", skill.name);
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content: Vec<Line> = match diff_result {
        Ok(diff) if diff.is_empty() => vec![Line::from(Span::styled(
            i18n.diff_no_changes(),
            fg(theme.color(ThemeSlot::Muted)),
        ))],
        Ok(diff) => diff
            .lines
            .iter()
            .map(|line| {
                let (color, prefix) = match line.kind {
                    DiffLineKind::Add => (theme.color(ThemeSlot::Success), "+ "),
                    DiffLineKind::Remove => (theme.color(ThemeSlot::Error), "- "),
                    DiffLineKind::Header => (theme.color(ThemeSlot::Accent), "  "),
                    DiffLineKind::Context => (Color::Reset, "  "),
                };
                Line::from(Span::styled(format!("{prefix}{}", line.text), fg(color)))
            })
            .collect(),
        Err(err) => vec![Line::from(Span::styled(
            i18n.diff_error(err),
            fg(theme.color(ThemeSlot::Warning)),
        ))],
    };

    let body = Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(body, inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Locale;
    use ai_skill_core::{DiffLineKind, DriftState, Scope, Skill, SkillMode, ValidationState};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    fn make_skill(name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: DriftState::UpdateAvailable {
                local_hash: "abc".into(),
                upstream_hash: "def".into(),
            },
        }
    }

    fn render(diff: &Result<SkillDiff, DiffError>, i18n: &I18n) -> String {
        let skill = make_skill("demo");
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_diff_panel(&skill, diff, 0, &Theme::default(), f.area(), f, i18n))
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
    fn shows_added_and_removed_lines() {
        let diff = Ok(SkillDiff {
            lines: vec![
                ai_skill_core::DiffLine {
                    kind: DiffLineKind::Header,
                    text: "@@ -1,2 +1,2 @@".into(),
                },
                ai_skill_core::DiffLine {
                    kind: DiffLineKind::Remove,
                    text: "old".into(),
                },
                ai_skill_core::DiffLine {
                    kind: DiffLineKind::Add,
                    text: "new".into(),
                },
            ],
        });
        let rendered = render(&diff, &I18n::default());
        assert!(rendered.contains("demo"));
        assert!(rendered.contains("old"));
        assert!(rendered.contains("new"));
        assert!(rendered.contains("Upstream Diff"));
    }

    #[test]
    fn empty_diff_shows_no_changes_message() {
        let rendered = render(&Ok(SkillDiff::default()), &I18n::default());
        assert!(rendered.contains("no upstream changes"));
    }

    #[test]
    fn error_shows_message() {
        let rendered = render(&Err(DiffError::NoUpstream), &I18n::default());
        assert!(rendered.contains("No upstream"));
    }

    #[test]
    fn pt_br_localizes_messages() {
        let empty = render(&Ok(SkillDiff::default()), &I18n::new(Locale::PtBr));
        assert!(empty.contains("sem alterações"));
        let err = render(&Err(DiffError::NoGitRepo), &I18n::new(Locale::PtBr));
        assert!(err.contains("Git"));
    }
}
