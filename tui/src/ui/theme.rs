//! Semantic theming for the TUI.
//!
//! Colors are resolved from semantic slots so users can customize the palette
//! in `~/.config/ai-skill/config.json` without touching code. The default theme
//! matches the colours previously hardcoded across the panels.

use ratatui::style::Color;
use std::collections::HashMap;

/// A semantic colour slot used across the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeSlot {
    /// Broken skills and high-severity findings.
    Error,
    /// Warnings, medium-severity findings, no-agent skills.
    Warning,
    /// Success states, available updates.
    Success,
    /// Informational accents (duplicates, connections).
    Accent,
    /// Muted text (disabled, hints).
    Muted,
    /// Dead skills (never used).
    Dead,
    /// Stale skills (unused beyond threshold).
    Stale,
}

impl ThemeSlot {
    /// All slots, used when resolving a config map.
    pub fn all() -> &'static [ThemeSlot] {
        &[
            ThemeSlot::Error,
            ThemeSlot::Warning,
            ThemeSlot::Success,
            ThemeSlot::Accent,
            ThemeSlot::Muted,
            ThemeSlot::Dead,
            ThemeSlot::Stale,
        ]
    }

    /// Stable config key for this slot.
    pub fn key(&self) -> &'static str {
        match self {
            ThemeSlot::Error => "error",
            ThemeSlot::Warning => "warning",
            ThemeSlot::Success => "success",
            ThemeSlot::Accent => "accent",
            ThemeSlot::Muted => "muted",
            ThemeSlot::Dead => "dead",
            ThemeSlot::Stale => "stale",
        }
    }
}

/// A resolved semantic colour theme.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    slots: HashMap<ThemeSlot, Color>,
}

impl Default for Theme {
    fn default() -> Self {
        let mut slots = HashMap::new();
        slots.insert(ThemeSlot::Error, Color::Red);
        slots.insert(ThemeSlot::Warning, Color::Yellow);
        slots.insert(ThemeSlot::Success, Color::Green);
        slots.insert(ThemeSlot::Accent, Color::Cyan);
        slots.insert(ThemeSlot::Muted, Color::DarkGray);
        slots.insert(ThemeSlot::Dead, Color::Magenta);
        slots.insert(ThemeSlot::Stale, Color::Yellow);
        Theme {
            slots: slots.clone(),
        }
    }
}

impl Theme {
    /// Resolves a theme from a config color map (slot key -> colour string).
    ///
    /// Unknown keys and unparseable colours are ignored; missing slots fall
    /// back to the default colour.
    pub fn from_config(map: &Option<HashMap<String, String>>) -> Self {
        let mut theme = Theme::default();
        if let Some(map) = map {
            for slot in ThemeSlot::all() {
                if let Some(value) = map.get(slot.key())
                    && let Some(color) = parse_color(value)
                {
                    theme.slots.insert(*slot, color);
                }
            }
        }
        theme
    }

    /// Returns the resolved colour for a slot.
    pub fn color(&self, slot: ThemeSlot) -> Color {
        *self.slots.get(&slot).unwrap_or(&Color::Reset)
    }
}

/// Parses a colour from a name or `#rrggbb` hex string.
pub fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "lightgray" | "lightgrey" => Some(Color::Gray),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_all_slots() {
        let theme = Theme::default();
        for slot in ThemeSlot::all() {
            assert_ne!(theme.color(*slot), Color::Reset);
        }
    }

    #[test]
    fn missing_config_uses_defaults() {
        let theme = Theme::from_config(&None);
        assert_eq!(theme.color(ThemeSlot::Error), Color::Red);
    }

    #[test]
    fn unknown_slot_key_is_ignored() {
        let mut map = HashMap::new();
        map.insert("not_a_slot".to_string(), "blue".to_string());
        let theme = Theme::from_config(&Some(map));
        assert_eq!(theme.color(ThemeSlot::Error), Color::Red);
    }

    #[test]
    fn overrides_resolve_named_and_hex() {
        let mut map = HashMap::new();
        map.insert("error".to_string(), "blue".to_string());
        map.insert("warning".to_string(), "#00ff00".to_string());
        let theme = Theme::from_config(&Some(map));
        assert_eq!(theme.color(ThemeSlot::Error), Color::Blue);
        assert_eq!(theme.color(ThemeSlot::Warning), Color::Rgb(0, 255, 0));
    }

    #[test]
    fn invalid_color_falls_back_to_default() {
        let mut map = HashMap::new();
        map.insert("error".to_string(), "notacolor".to_string());
        let theme = Theme::from_config(&Some(map));
        assert_eq!(theme.color(ThemeSlot::Error), Color::Red);
    }

    #[test]
    fn parse_hex_requires_six_digits() {
        assert_eq!(parse_color("#abc"), None);
        assert_eq!(parse_color("#112233"), Some(Color::Rgb(17, 34, 51)));
    }

    #[test]
    fn parse_named_is_case_insensitive() {
        assert_eq!(parse_color("RED"), Some(Color::Red));
        assert_eq!(parse_color("DarkGray"), Some(Color::DarkGray));
    }
}
