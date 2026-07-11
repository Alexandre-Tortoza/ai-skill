//! Main skill-list panel with scope/agent/tag filtering and multi-select.

use ai_skill_core::{Scope, Skill};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::ListUiState;

use super::style_helpers::{badge_for_validation, drift_badge, fg};

/// Renders the main scrollable skill list with badges and a filter header.
pub fn render_installed_panel(
    skills: &[&Skill],
    state: &ListUiState,
    area: Rect,
    frame: &mut Frame,
) {
    let title = match &state.tag_filter {
        None => format!("Installed Skills [{}]", state.scope_filter.label()),
        Some(tag) => format!("Installed Skills [{}] #{tag}", state.scope_filter.label()),
    };

    let items: Vec<ListItem> = skills
        .iter()
        .map(|s| {
            let scope_badge = match s.scope {
                Scope::Global => "[global]",
                Scope::Project => "[project]",
            };
            let (val_badge, val_color) = badge_for_validation(&s.validation);
            let db = drift_badge(&s.drift_state);
            let text = match (val_badge.is_empty(), db) {
                (true, None) => format!("{} {}", s.name, scope_badge),
                (false, None) => format!("{} {} {}", s.name, scope_badge, val_badge),
                (true, Some((d, _))) => format!("{} {} {}", s.name, scope_badge, d),
                (false, Some((d, _))) => format!("{} {} {} {}", s.name, scope_badge, val_badge, d),
            };
            ListItem::new(text).style(fg(val_color))
        })
        .collect();

    let mut list_state = ListState::default();
    if !skills.is_empty() {
        list_state.select(Some(state.selected_index));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut list_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{ListUiState, ScopeFilter};
    use ai_skill_core::ValidationState;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    fn make_skill(name: &str, scope: Scope, validation: ValidationState) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope,
            agents: vec![],
            tags: vec![],
            managed: false,
            validation,
            manifest_content: None,
            drift_state: ai_skill_core::DriftState::default(),
        }
    }

    fn render(skills: &[&Skill], state: &ListUiState) -> String {
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_installed_panel(skills, state, f.area(), f))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    }

    fn default_state() -> ListUiState {
        ListUiState {
            scope_filter: ScopeFilter::All,
            agent_filter: None,
            tag_filter: None,
            selected_index: 0,
            selected_items: vec![],
        }
    }

    #[test]
    fn snapshot_all_filter_with_mixed_skills() {
        let skills = [
            make_skill("alpha", Scope::Global, ValidationState::Valid),
            make_skill("beta", Scope::Project, ValidationState::BrokenSymlink),
        ];
        let refs: Vec<&Skill> = skills.iter().collect();
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_installed_panel(&refs, &default_state(), f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn valid_skill_has_no_badge_text() {
        let skill = make_skill("clean", Scope::Global, ValidationState::Valid);
        let state = default_state();
        let rendered = render(&[&skill], &state);
        assert!(rendered.contains("clean"));
        assert!(!rendered.contains("[broken"));
    }

    #[test]
    fn broken_symlink_badge_appears_in_render() {
        let skill = make_skill("dead", Scope::Project, ValidationState::BrokenSymlink);
        let state = default_state();
        let rendered = render(&[&skill], &state);
        assert!(rendered.contains("broken-symlink"));
    }

    #[test]
    fn update_available_skill_shows_drift_badge() {
        use ai_skill_core::DriftState;
        let mut skill = make_skill("drifted", Scope::Global, ValidationState::Valid);
        skill.drift_state = DriftState::UpdateAvailable {
            local_hash: "abc1234".into(),
            upstream_hash: "def5678".into(),
        };
        let state = default_state();
        let rendered = render(&[&skill], &state);
        assert!(rendered.contains('↑'));
    }

    #[test]
    fn up_to_date_skill_has_no_drift_badge() {
        use ai_skill_core::DriftState;
        let mut skill = make_skill("fresh", Scope::Global, ValidationState::Valid);
        skill.drift_state = DriftState::UpToDate;
        let state = default_state();
        let rendered = render(&[&skill], &state);
        assert!(!rendered.contains('↑'));
    }

    #[test]
    fn title_shows_scope_filter_label() {
        let state = ListUiState {
            scope_filter: ScopeFilter::Global,
            agent_filter: None,
            tag_filter: None,
            selected_index: 0,
            selected_items: vec![],
        };
        let rendered = render(&[], &state);
        assert!(rendered.contains("global"));
    }
}
