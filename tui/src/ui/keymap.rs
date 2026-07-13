//! Customizable key bindings for the TUI.
//!
//! Bindings are read from `~/.config/ai-skill/config.json` (`keymap` map of
//! action name -> key string). Unknown actions and unparseable keys fall back
//! to built-in defaults. This first slice covers the main navigation/panel
//! shortcuts; wizards and contextual keys remain on their built-in bindings.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// A user-facing action that can be bound to a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Quit the application.
    Quit,
    /// Open the help overlay.
    Help,
    /// Open the audit panel.
    Audit,
    /// Open the search panel.
    Search,
    /// Open the create-skill wizard.
    Create,
    /// Open the profiles panel.
    Profiles,
    /// Open the bundles panel.
    Bundles,
    /// Open the budget panel.
    Budget,
    /// Open the skill editor.
    Editor,
    /// Open the sync panel.
    Sync,
    /// Open the SSH remote panel.
    SshRemote,
    /// Adopt an unmanaged skill.
    Adopt,
    /// Toggle a skill between name-only and full mode.
    ToggleNameOnly,
    /// Disable the selected skill.
    Disable,
    /// Enable a disabled skill.
    Enable,
    /// Remove the selected skill.
    Remove,
    /// Update the selected skill.
    Update,
}

impl Action {
    /// Stable config key for this action.
    pub fn key(&self) -> &'static str {
        match self {
            Action::Quit => "quit",
            Action::Help => "help",
            Action::Audit => "audit",
            Action::Search => "search",
            Action::Create => "create",
            Action::Profiles => "profiles",
            Action::Bundles => "bundles",
            Action::Budget => "budget",
            Action::Editor => "editor",
            Action::Sync => "sync",
            Action::SshRemote => "ssh",
            Action::Adopt => "adopt",
            Action::ToggleNameOnly => "toggle_name_only",
            Action::Disable => "disable",
            Action::Enable => "enable",
            Action::Remove => "remove",
            Action::Update => "update",
        }
    }
}

/// Resolved key bindings for all customizable actions.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyBindings {
    bindings: HashMap<Action, (KeyCode, KeyModifiers)>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(Action::Quit, (KeyCode::Char('q'), KeyModifiers::NONE));
        bindings.insert(Action::Help, (KeyCode::Char('?'), KeyModifiers::NONE));
        bindings.insert(Action::Audit, (KeyCode::Char('A'), KeyModifiers::NONE));
        bindings.insert(Action::Search, (KeyCode::Char('s'), KeyModifiers::NONE));
        bindings.insert(Action::Create, (KeyCode::Char('c'), KeyModifiers::NONE));
        bindings.insert(Action::Profiles, (KeyCode::Char('p'), KeyModifiers::NONE));
        bindings.insert(Action::Bundles, (KeyCode::Char('b'), KeyModifiers::NONE));
        bindings.insert(Action::Budget, (KeyCode::Char('B'), KeyModifiers::NONE));
        bindings.insert(Action::Editor, (KeyCode::Char('e'), KeyModifiers::NONE));
        bindings.insert(Action::Sync, (KeyCode::Char('S'), KeyModifiers::NONE));
        bindings.insert(Action::SshRemote, (KeyCode::Char('R'), KeyModifiers::NONE));
        bindings.insert(Action::Adopt, (KeyCode::Char('a'), KeyModifiers::NONE));
        bindings.insert(
            Action::ToggleNameOnly,
            (KeyCode::Char('n'), KeyModifiers::NONE),
        );
        bindings.insert(Action::Disable, (KeyCode::Char('d'), KeyModifiers::NONE));
        bindings.insert(Action::Enable, (KeyCode::Char('e'), KeyModifiers::NONE));
        bindings.insert(Action::Remove, (KeyCode::Char('r'), KeyModifiers::NONE));
        bindings.insert(Action::Update, (KeyCode::Char('u'), KeyModifiers::NONE));
        KeyBindings { bindings }
    }
}

impl KeyBindings {
    /// Builds bindings from a config map, falling back to defaults for any
    /// action not present or with an unparseable key.
    pub fn from_config(map: &HashMap<String, String>) -> Self {
        let mut bindings = KeyBindings::default().bindings;
        for action in all_actions() {
            if let Some(value) = map.get(action.key())
                && let Some(parsed) = parse_key(value)
            {
                bindings.insert(*action, parsed);
            }
        }
        KeyBindings { bindings }
    }

    /// Returns true if `key` triggers `action`.
    ///
    /// `Quit` additionally accepts `Ctrl-C` regardless of configuration.
    /// Letter bindings are matched case-insensitively and ignore the `SHIFT`
    /// modifier, so physical keys work regardless of caps-lock state.
    pub fn matches(&self, key: &KeyEvent, action: Action) -> bool {
        if action == Action::Quit
            && key.code == KeyCode::Char('c')
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return true;
        }
        match self.bindings.get(&action) {
            Some((code, mods)) => {
                codes_match(key.code, *code) && modifiers_match(key.modifiers, *mods)
            }
            None => false,
        }
    }
}

/// Returns every customizable action.
fn all_actions() -> &'static [Action] {
    &[
        Action::Quit,
        Action::Help,
        Action::Audit,
        Action::Search,
        Action::Create,
        Action::Profiles,
        Action::Bundles,
        Action::Budget,
        Action::Editor,
        Action::Sync,
        Action::SshRemote,
        Action::Adopt,
        Action::ToggleNameOnly,
        Action::Disable,
        Action::Enable,
        Action::Remove,
        Action::Update,
    ]
}

fn codes_match(a: KeyCode, b: KeyCode) -> bool {
    match (a, b) {
        (KeyCode::Char(x), KeyCode::Char(y)) => x.eq_ignore_ascii_case(&y),
        _ => a == b,
    }
}

fn modifiers_match(a: KeyModifiers, b: KeyModifiers) -> bool {
    // Ignore SHIFT for letter bindings; it only affects case.
    let strip = |m: KeyModifiers| m & !KeyModifiers::SHIFT;
    strip(a) == strip(b)
}

/// Parses a key string into a `(KeyCode, KeyModifiers)` pair.
///
/// Supported forms: `ctrl+c`, `F1`..`F12`, a single character (`a`, `A`, `?`, `/`).
pub fn parse_key(value: &str) -> Option<(KeyCode, KeyModifiers)> {
    let value = value.trim();
    if let Some(rest) = value.to_ascii_lowercase().strip_prefix("ctrl+") {
        let mut chars = rest.chars();
        let c = chars.next()?;
        if chars.next().is_some() {
            return None;
        }
        if !c.is_ascii_alphabetic() {
            return None;
        }
        return Some((KeyCode::Char(c), KeyModifiers::CONTROL));
    }
    if let Some(num) = value.to_ascii_lowercase().strip_prefix('f')
        && let Ok(n) = num.parse::<u8>()
        && (1..=12).contains(&n)
    {
        return Some((KeyCode::F(n), KeyModifiers::NONE));
    }
    let mut chars = value.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some((KeyCode::Char(c), KeyModifiers::NONE))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    fn key_with(c: char, mods: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: mods,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    #[test]
    fn default_quit_matches_q() {
        let bindings = KeyBindings::default();
        assert!(bindings.matches(&key('q'), Action::Quit));
    }

    #[test]
    fn default_quit_matches_ctrl_c() {
        let bindings = KeyBindings::default();
        assert!(bindings.matches(&key_with('c', KeyModifiers::CONTROL), Action::Quit));
    }

    #[test]
    fn default_audit_matches_uppercase_a() {
        let bindings = KeyBindings::default();
        assert!(bindings.matches(&key('A'), Action::Audit));
    }

    #[test]
    fn letter_match_is_case_insensitive() {
        let bindings = KeyBindings::default();
        // 's' is Search; SHIFT+s should also match.
        assert!(bindings.matches(&key_with('s', KeyModifiers::SHIFT), Action::Search));
    }

    #[test]
    fn unmapped_action_does_not_match() {
        let bindings = KeyBindings::default();
        assert!(!bindings.matches(&key('z'), Action::Create));
    }

    #[test]
    fn config_override_changes_binding() {
        let mut map = HashMap::new();
        map.insert("search".to_string(), "f".to_string());
        let bindings = KeyBindings::from_config(&map);
        assert!(bindings.matches(&key('f'), Action::Search));
        assert!(!bindings.matches(&key('s'), Action::Search));
    }

    #[test]
    fn config_unknown_action_keeps_default() {
        let mut map = HashMap::new();
        map.insert("not_an_action".to_string(), "x".to_string());
        let bindings = KeyBindings::from_config(&map);
        assert!(bindings.matches(&key('q'), Action::Quit));
    }

    #[test]
    fn config_invalid_key_keeps_default() {
        let mut map = HashMap::new();
        map.insert("search".to_string(), "shift+??".to_string());
        let bindings = KeyBindings::from_config(&map);
        assert!(bindings.matches(&key('s'), Action::Search));
    }

    #[test]
    fn parse_ctrl_key() {
        assert_eq!(
            parse_key("ctrl+c"),
            Some((KeyCode::Char('c'), KeyModifiers::CONTROL))
        );
    }

    #[test]
    fn parse_function_key() {
        assert_eq!(parse_key("F5"), Some((KeyCode::F(5), KeyModifiers::NONE)));
    }

    #[test]
    fn parse_symbol_key() {
        assert_eq!(
            parse_key("?"),
            Some((KeyCode::Char('?'), KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key("/"),
            Some((KeyCode::Char('/'), KeyModifiers::NONE))
        );
    }

    #[test]
    fn parse_rejects_multi_char() {
        assert_eq!(parse_key("ab"), None);
        assert_eq!(parse_key("ctrl+"), None);
    }
}
