//! Panel for viewing, creating, activating, and deleting profiles.

use ai_skill_core::Phase;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::ProfileState,
    ui::style_helpers::{bg, fg},
};

/// Renders the profiles panel with list, activate, and create controls.
pub fn render_profiles_panel(state: &ProfileState, area: Rect, frame: &mut Frame) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    render_profile_list(state, chunks[0], frame);
    render_profile_detail(state, chunks[1], frame);
}

fn render_profile_list(state: &ProfileState, area: Rect, frame: &mut Frame) {
    let items: Vec<ListItem> = state
        .profiles
        .iter()
        .map(|p| {
            let count = p.skill_names.len();
            let phase_badge = match &p.phase {
                Some(Phase::Init) => Some(("[init]", Color::Cyan)),
                Some(Phase::Dev) => Some(("[dev]", Color::Green)),
                Some(Phase::Test) => Some(("[test]", Color::Yellow)),
                Some(Phase::Release) => Some(("[release]", Color::Red)),
                None => None,
            };
            let mut spans = vec![Span::raw(p.name.clone())];
            if let Some((badge, color)) = phase_badge {
                spans.push(Span::styled(format!(" {badge}"), fg(color).add_modifier(Modifier::BOLD)));
            }
            spans.push(Span::styled(format!("  ({count} skills)"), fg(Color::DarkGray)));
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Profiles ").borders(Borders::ALL))
        .highlight_style(bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if !state.profiles.is_empty() {
        list_state.select(Some(state.selected_index));
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_profile_detail(state: &ProfileState, area: Rect, frame: &mut Frame) {
    if state.creating {
        let input_text = format!("Name: {}_", state.new_name_input);
        let widget = Paragraph::new(input_text).block(
            Block::default()
                .title(" New Profile ")
                .borders(Borders::ALL),
        );
        frame.render_widget(widget, area);
        return;
    }

    let selected = state.profiles.get(state.selected_index);
    let block = Block::default()
        .title(" Skills in Profile ")
        .borders(Borders::ALL);

    if let Some(profile) = selected {
        let items: Vec<ListItem> = profile
            .skill_names
            .iter()
            .map(|s| ListItem::new(s.as_str()))
            .collect();
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    } else {
        let msg =
            Paragraph::new("No profiles yet.\nPress f to create from current skills.").block(block);
        frame.render_widget(msg, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::Profile;
    use ratatui::{Terminal, backend::TestBackend};

    fn make_state(profiles: Vec<Profile>) -> ProfileState {
        ProfileState {
            profiles,
            selected_index: 0,
            new_name_input: String::new(),
            creating: false,
        }
    }

    #[test]
    fn renders_without_panic_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = make_state(vec![]);
        terminal
            .draw(|f| render_profiles_panel(&state, f.area(), f))
            .unwrap();
    }

    #[test]
    fn renders_without_panic_with_profiles() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = make_state(vec![
            Profile {
                name: "dev".into(),
                skill_names: vec!["alpha".into(), "beta".into()],
                phase: None,
            },
            Profile {
                name: "ops".into(),
                skill_names: vec!["deploy".into()],
                phase: None,
            },
        ]);
        terminal
            .draw(|f| render_profiles_panel(&state, f.area(), f))
            .unwrap();
    }

    #[test]
    fn snapshot_two_profiles() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = make_state(vec![
            Profile {
                name: "dev".into(),
                skill_names: vec!["alpha".into(), "beta".into()],
                phase: None,
            },
            Profile {
                name: "ops".into(),
                skill_names: vec!["deploy".into()],
                phase: None,
            },
        ]);
        terminal
            .draw(|f| render_profiles_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }

    #[test]
    fn snapshot_creating_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = make_state(vec![]);
        state.creating = true;
        state.new_name_input = "my-prf".into();
        terminal
            .draw(|f| render_profiles_panel(&state, f.area(), f))
            .unwrap();
        insta::assert_debug_snapshot!(terminal.backend().buffer().clone());
    }
}
