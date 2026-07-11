//! Legacy skill-list renderer (kept for reference; use `installed_panel` instead).

use ai_skill_core::{Scope, Skill};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

#[allow(dead_code)]
/// Renders a simple list of skill names with scope badges.
pub fn render_skills_panel(skills: &[Skill], selected: usize, area: Rect, frame: &mut Frame) {
    let items: Vec<ListItem> = skills
        .iter()
        .map(|s| {
            let badge = match s.scope {
                Scope::Global => "[global]",
                Scope::Project => "[project]",
            };
            ListItem::new(format!("{} {}", s.name, badge))
        })
        .collect();

    let mut state = ListState::default();
    if !skills.is_empty() {
        state.select(Some(selected));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Installed Skills"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::Scope;
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;

    fn make_skill(name: &str, scope: Scope) -> Skill {
        use ai_skill_core::{DriftState, SkillMode, ValidationState};
        Skill {
            name: name.to_string(),
            path: PathBuf::from("/tmp").join(name),
            scope,
            agents: vec![],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: DriftState::default(),
        }
    }

    #[test]
    fn renders_skills_without_panic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let skills = vec![
            make_skill("alpha", Scope::Global),
            make_skill("beta", Scope::Project),
        ];
        terminal
            .draw(|f| render_skills_panel(&skills, 0, f.area(), f))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        // verify skill names appear in the rendered output
        let rendered: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(
            rendered.contains("alpha"),
            "alpha not found in rendered output"
        );
        assert!(
            rendered.contains("beta"),
            "beta not found in rendered output"
        );
        assert!(rendered.contains("Installed Skills"));
    }

    #[test]
    fn renders_empty_list_without_panic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| render_skills_panel(&[], 0, f.area(), f))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        let rendered: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        assert!(rendered.contains("Installed Skills"));
    }

    #[test]
    fn snapshot_skills_panel() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let skills = vec![
            make_skill("omarchy", Scope::Global),
            make_skill("my-skill", Scope::Project),
        ];
        terminal
            .draw(|f| render_skills_panel(&skills, 0, f.area(), f))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        insta::assert_debug_snapshot!(buffer);
    }
}
