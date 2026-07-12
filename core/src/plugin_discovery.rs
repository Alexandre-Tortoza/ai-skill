//! Plugin marketplace discovery: models and port for finding skills
//! declared in `.claude-plugin/marketplace.json` and `.cursor-plugin/marketplace.json`.

use std::path::PathBuf;

/// A parsed entry from a plugin marketplace manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginEntry {
    /// Plugin name (kebab-case).
    pub name: String,
    /// Source path or URL.
    pub source: Option<String>,
    /// Override for the skills directory (default is `skills/` under the plugin dir).
    pub skills_path: Option<String>,
}

/// A parsed marketplace manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct MarketplaceManifest {
    /// Marketplace name.
    pub name: String,
    /// Plugin entries.
    pub plugins: Vec<PluginEntry>,
}

/// A skill discovered inside a plugin directory.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginDiscoveredSkill {
    /// Skill name (directory name).
    pub name: String,
    /// Absolute path to the skill directory.
    pub path: PathBuf,
    /// Raw SKILL.md content (if readable).
    pub manifest_content: Option<String>,
    /// Plugin name this skill belongs to.
    pub plugin_name: String,
    /// The marketplace file key (e.g. "claude-plugin" or "cursor-plugin").
    pub marketplace_key: String,
}

/// Port for discovering skills from plugin marketplace manifests.
pub trait PluginMarketplaceDiscovery {
    type Error: std::error::Error;

    /// Returns all skills found in plugin marketplace directories.
    fn discover_skills(&self) -> Result<Vec<PluginDiscoveredSkill>, Self::Error>;
}

/// In-memory no-op implementation for tests and fallback.
pub struct NoopPluginDiscoverer;

impl PluginMarketplaceDiscovery for NoopPluginDiscoverer {
    type Error = std::convert::Infallible;

    fn discover_skills(&self) -> Result<Vec<PluginDiscoveredSkill>, Self::Error> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_discoverer_returns_empty() {
        let d = NoopPluginDiscoverer;
        assert!(d.discover_skills().unwrap().is_empty());
    }

    #[test]
    fn plugin_entry_fields_accessible() {
        let entry = PluginEntry {
            name: "my-plugin".into(),
            source: Some("./plugins/my-plugin".into()),
            skills_path: Some("./skills".into()),
        };
        assert_eq!(entry.name, "my-plugin");
        assert_eq!(entry.source.unwrap(), "./plugins/my-plugin");
    }

    #[test]
    fn marketplace_manifest_fields_accessible() {
        let manifest = MarketplaceManifest {
            name: "test-marketplace".into(),
            plugins: vec![PluginEntry {
                name: "p1".into(),
                source: None,
                skills_path: None,
            }],
        };
        assert_eq!(manifest.name, "test-marketplace");
        assert_eq!(manifest.plugins.len(), 1);
    }

    #[test]
    fn plugin_discovered_skill_fields_accessible() {
        let s = PluginDiscoveredSkill {
            name: "code-review".into(),
            path: PathBuf::from("/tmp/skills/code-review"),
            manifest_content: Some("---\nname: code-review\n---\n# Body".into()),
            plugin_name: "dev-tools".into(),
            marketplace_key: "claude-plugin".into(),
        };
        assert_eq!(s.name, "code-review");
        assert!(s.manifest_content.is_some());
        assert_eq!(s.plugin_name, "dev-tools");
        assert_eq!(s.marketplace_key, "claude-plugin");
    }
}
