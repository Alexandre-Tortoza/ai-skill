//! Filesystem-based [`PluginMarketplaceDiscovery`] adapter.
//!
//! Scans `.claude-plugin/marketplace.json` and `.cursor-plugin/marketplace.json`
//! in the home directory and the current project directory. For each plugin with
//! a local filesystem source, resolves the plugin directory and discovers skills
//! under `skills/<name>/SKILL.md`.

use ai_skill_core::{
    MarketplaceManifest, PluginDiscoveredSkill, PluginEntry, PluginMarketplaceDiscovery,
    extract_body, parse_frontmatter,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors from [`FsPluginDiscoverer`].
#[derive(Debug, Error)]
pub enum PluginDiscovererError {
    /// Underlying IO error.
    #[error("filesystem error while reading plugin marketplaces: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parsing error.
    #[error("invalid marketplace.json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Adapter that discovers skills from local plugin marketplace manifests.
pub struct FsPluginDiscoverer {
    home: PathBuf,
    project: Option<PathBuf>,
}

impl FsPluginDiscoverer {
    /// Creates a new discoverer with explicit home and optional project roots.
    pub fn new(home: PathBuf, project: Option<PathBuf>) -> Self {
        Self { home, project }
    }

    /// Resolves roots from `$HOME` and the current working directory.
    pub fn from_env() -> Result<Self, PluginDiscovererError> {
        let home = std::env::var("HOME")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME is not set"))?;
        let cwd = std::env::current_dir()?;
        Ok(Self::new(PathBuf::from(home), Some(cwd)))
    }
}

/// Raw marketplace.json structure for serde deserialization.
#[derive(Debug, Deserialize)]
struct RawMarketplace {
    name: Option<String>,
    plugins: Vec<RawPlugin>,
}

/// Raw plugin entry from marketplace.json.
#[derive(Debug, Deserialize)]
struct RawPlugin {
    name: String,
    #[serde(default)]
    source: Option<serde_json::Value>,
    #[serde(default)]
    skills: Option<serde_json::Value>,
}

fn parse_marketplace(path: &Path) -> Result<MarketplaceManifest, PluginDiscovererError> {
    let content = std::fs::read_to_string(path)?;
    let raw: RawMarketplace = serde_json::from_str(&content)?;

    let mut plugins = Vec::new();
    for rp in raw.plugins {
        let source = match rp.source {
            Some(serde_json::Value::String(s)) => Some(s),
            Some(_) => None,
            None => None,
        };
        let skills_path = match rp.skills {
            Some(serde_json::Value::String(s)) => Some(s),
            Some(serde_json::Value::Array(arr)) => {
                arr.first().and_then(|v| v.as_str().map(String::from))
            }
            _ => None,
        };
        plugins.push(PluginEntry {
            name: rp.name,
            source,
            skills_path,
        });
    }

    Ok(MarketplaceManifest {
        name: raw.name.unwrap_or_default(),
        plugins,
    })
}

/// Returns the marketplace directories that could contain marketplace.json.
fn candidate_marketplace_dirs(home: &Path, project: Option<&Path>) -> Vec<(PathBuf, &'static str)> {
    let mut dirs = Vec::new();
    for &key in &["claude-plugin", "cursor-plugin"] {
        dirs.push((home.join(format!(".{key}")), key));
        if let Some(proj) = project {
            dirs.push((proj.join(format!(".{key}")), key));
        }
    }
    dirs
}

/// Discover skills within a plugin directory.
fn discover_plugin_skills(
    plugin_dir: &Path,
    plugin_name: &str,
    skills_override: Option<&str>,
    marketplace_key: &str,
) -> Vec<PluginDiscoveredSkill> {
    let skills_base = match skills_override {
        Some(rel) => plugin_dir.join(rel),
        None => plugin_dir.join("skills"),
    };

    if !skills_base.is_dir() {
        return vec![];
    }

    let mut skills = Vec::new();
    let entries = match std::fs::read_dir(&skills_base) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        // Look for SKILL.md in the skill subdirectory.
        let manifest_path = path.join("SKILL.md");
        let manifest_content = std::fs::read_to_string(&manifest_path).ok();

        // Use the frontmatter name if available, otherwise the directory name.
        let skill_name = manifest_content
            .as_ref()
            .and_then(|c| parse_frontmatter(c).ok())
            .map(|m| m.name)
            .unwrap_or(dir_name);

        skills.push(PluginDiscoveredSkill {
            name: skill_name,
            path,
            manifest_content: manifest_content
                .as_ref()
                .and_then(|c| extract_body(c).map(String::from)),
            plugin_name: plugin_name.to_string(),
            marketplace_key: marketplace_key.to_string(),
        });
    }

    skills
}

impl PluginMarketplaceDiscovery for FsPluginDiscoverer {
    type Error = PluginDiscovererError;

    fn discover_skills(&self) -> Result<Vec<PluginDiscoveredSkill>, Self::Error> {
        let mut all_skills = Vec::new();
        let dirs = candidate_marketplace_dirs(&self.home, self.project.as_deref());

        for (marketplace_dir, marketplace_key) in &dirs {
            let manifest_path = marketplace_dir.join("marketplace.json");
            if !manifest_path.is_file() {
                continue;
            }

            let manifest = match parse_marketplace(&manifest_path) {
                Ok(m) => m,
                Err(PluginDiscovererError::Io(_)) => continue,
                Err(e) => return Err(e),
            };

            // For each plugin with a local source, discover skills.
            for plugin in &manifest.plugins {
                let plugin_dir = match &plugin.source {
                    Some(src) if src.starts_with("./") || src.starts_with('/') => {
                        if src.starts_with('/') {
                            PathBuf::from(src)
                        } else {
                            marketplace_dir.join(src)
                        }
                    }
                    // If no source specified, assume plugin lives next to marketplace.json.
                    None => marketplace_dir.join(&plugin.name),
                    // Non-local sources (github, npm, url) — skip.
                    Some(_) => continue,
                };

                if !plugin_dir.is_dir() {
                    continue;
                }

                // Also check plugin.json inside the plugin directory.
                let plugin_json_skills =
                    discover_skills_from_plugin_json(&plugin_dir, &plugin.name, marketplace_key);
                all_skills.extend(plugin_json_skills);

                // Discover from marketplace entry's skills field or default skills/ dir.
                let marketplace_skills = discover_plugin_skills(
                    &plugin_dir,
                    &plugin.name,
                    plugin.skills_path.as_deref(),
                    marketplace_key,
                );
                all_skills.extend(marketplace_skills);
            }
        }

        Ok(all_skills)
    }
}

/// Parse `plugin.json` inside a plugin directory and discover its skills.
fn discover_skills_from_plugin_json(
    plugin_dir: &Path,
    plugin_name: &str,
    marketplace_key: &str,
) -> Vec<PluginDiscoveredSkill> {
    let manifest_path = plugin_dir.join("plugin.json");
    if !manifest_path.is_file() {
        return vec![];
    }

    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    // plugin.json has the same structure as the entry in marketplace.json.
    #[derive(Deserialize)]
    struct PluginJson {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        skills: Option<serde_json::Value>,
    }

    let parsed: PluginJson = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let pname = parsed.name.unwrap_or_else(|| plugin_name.to_string());

    // Extract skills path from plugin.json.
    let skills_path = match parsed.skills {
        Some(serde_json::Value::String(s)) => Some(s),
        Some(serde_json::Value::Array(arr)) => {
            arr.first().and_then(|v| v.as_str().map(String::from))
        }
        _ => None,
    };

    discover_plugin_skills(plugin_dir, &pname, skills_path.as_deref(), marketplace_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_marketplace(dir: &Path, key: &str, plugins: &[(&str, &str)]) {
        let marketplace_dir = dir.join(format!(".{key}"));
        fs::create_dir_all(&marketplace_dir).unwrap();

        let entries: Vec<serde_json::Value> = plugins
            .iter()
            .map(|(name, source)| {
                serde_json::json!({
                    "name": name,
                    "source": source,
                })
            })
            .collect();

        let manifest = serde_json::json!({
            "name": "test-marketplace",
            "plugins": entries,
        });

        fs::write(
            marketplace_dir.join("marketplace.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }

    fn create_plugin_skill(plugin_dir: &Path, skill_name: &str) {
        let skills_dir = plugin_dir.join("skills").join(skill_name);
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(
            skills_dir.join("SKILL.md"),
            format!("---\nname: {skill_name}\ndescription: A skill\n---\n# {skill_name} body\n"),
        )
        .unwrap();
    }

    #[test]
    fn discovers_skills_from_claude_plugin() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        create_marketplace(
            &home,
            "claude-plugin",
            &[("dev-tools", "./plugins/dev-tools")],
        );

        let plugin_dir = home
            .join(".claude-plugin")
            .join("plugins")
            .join("dev-tools");
        create_plugin_skill(&plugin_dir, "code-review");

        let discoverer = FsPluginDiscoverer::new(home, None);
        let skills = discoverer.discover_skills().unwrap();

        assert!(!skills.is_empty());
        let skill = skills.iter().find(|s| s.name == "code-review").unwrap();
        assert_eq!(skill.plugin_name, "dev-tools");
        assert_eq!(skill.marketplace_key, "claude-plugin");
    }

    #[test]
    fn discovers_skills_from_cursor_plugin() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        create_marketplace(
            &home,
            "cursor-plugin",
            &[("ui-tools", "./plugins/ui-tools")],
        );

        let plugin_dir = home.join(".cursor-plugin").join("plugins").join("ui-tools");
        create_plugin_skill(&plugin_dir, "design-system");

        let discoverer = FsPluginDiscoverer::new(home, None);
        let skills = discoverer.discover_skills().unwrap();

        assert!(!skills.is_empty());
        let skill = skills.iter().find(|s| s.name == "design-system").unwrap();
        assert_eq!(skill.plugin_name, "ui-tools");
        assert_eq!(skill.marketplace_key, "cursor-plugin");
    }

    #[test]
    fn empty_marketplace_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let marketplace_dir = home.join(".claude-plugin");
        fs::create_dir_all(&marketplace_dir).unwrap();
        fs::write(
            marketplace_dir.join("marketplace.json"),
            r#"{"name": "empty", "plugins": []}"#,
        )
        .unwrap();

        let discoverer = FsPluginDiscoverer::new(home, None);
        let skills = discoverer.discover_skills().unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn no_marketplace_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");

        let discoverer = FsPluginDiscoverer::new(home, None);
        let skills = discoverer.discover_skills().unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn skips_remote_source_plugins() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let marketplace_dir = home.join(".claude-plugin");
        fs::create_dir_all(&marketplace_dir).unwrap();

        let manifest = serde_json::json!({
            "name": "test",
            "plugins": [
                {"name": "local-tool", "source": "./plugins/local-tool"},
                {"name": "remote-tool", "source": {"type": "github", "repo": "user/repo"}},
            ]
        });
        fs::write(
            marketplace_dir.join("marketplace.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        // Create local plugin with skill.
        let local_plugin = home
            .join(".claude-plugin")
            .join("plugins")
            .join("local-tool");
        create_plugin_skill(&local_plugin, "local-skill");

        let discoverer = FsPluginDiscoverer::new(home, None);
        let skills = discoverer.discover_skills().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "local-skill");
    }

    #[test]
    fn discovers_from_plugin_json() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let marketplace_dir = home.join(".claude-plugin");
        fs::create_dir_all(&marketplace_dir).unwrap();

        // Marketplace with no explicit skills field.
        let manifest = serde_json::json!({
            "name": "test",
            "plugins": [{"name": "my-plugin", "source": "./plugins/my-plugin"}]
        });
        fs::write(
            marketplace_dir.join("marketplace.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let plugin_dir = home
            .join(".claude-plugin")
            .join("plugins")
            .join("my-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        // Write plugin.json with explicit skills path.
        let plugin_json = serde_json::json!({
            "name": "my-plugin",
            "skills": "./custom-skills"
        });
        fs::write(
            plugin_dir.join("plugin.json"),
            serde_json::to_string_pretty(&plugin_json).unwrap(),
        )
        .unwrap();

        // Create skills under custom-skills dir.
        let custom_skills = plugin_dir.join("custom-skills").join("my-skill");
        fs::create_dir_all(&custom_skills).unwrap();
        fs::write(
            custom_skills.join("SKILL.md"),
            "---\nname: my-skill\ndescription: custom\n---\n# body\n",
        )
        .unwrap();

        let discoverer = FsPluginDiscoverer::new(home, None);
        let skills = discoverer.discover_skills().unwrap();

        assert!(!skills.is_empty());
        assert!(skills.iter().any(|s| s.name == "my-skill"));
    }

    #[test]
    fn from_env_resolves_home() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp
            .path()
            .join(".claude-plugin")
            .join("plugins")
            .join("p1")
            .join("skills")
            .join("s1");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(
            skills_dir.join("SKILL.md"),
            "---\nname: s1\ndescription: test\n---\n# body\n",
        )
        .unwrap();

        let marketplace_dir = tmp.path().join(".claude-plugin");
        let manifest = serde_json::json!({
            "name": "test",
            "plugins": [{"name": "p1", "source": "./plugins/p1"}]
        });
        fs::write(
            marketplace_dir.join("marketplace.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", tmp.path()) }
        let discoverer = FsPluginDiscoverer::from_env().unwrap();
        if let Some(h) = original_home {
            unsafe { std::env::set_var("HOME", h) }
        } else {
            unsafe { std::env::remove_var("HOME") }
        }

        let skills = discoverer.discover_skills().unwrap();
        assert!(!skills.is_empty());
        assert_eq!(skills[0].name, "s1");
    }

    #[test]
    fn discoverer_error_io_debug() {
        let err =
            PluginDiscovererError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(!format!("{err:?}").is_empty());
    }

    #[test]
    fn discoverer_error_io_display() {
        let err =
            PluginDiscovererError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(err.to_string().contains("filesystem error"));
    }

    #[test]
    fn discoverer_error_json_roundtrip() {
        let json_err = serde_json::from_str::<RawMarketplace>("invalid").unwrap_err();
        let err = PluginDiscovererError::Json(json_err);
        assert!(err.to_string().contains("invalid marketplace.json"));
    }
}
