//! Adapter that reads skill file content from the filesystem.

use ai_skill_core::{
    ContentError, SkillContentReader, SkillDoc, SkillFileKind, SkillTreeNode, extract_body,
};
use std::path::Path;

/// Maximum bytes rendered for any single preview/file before truncation.
const PREVIEW_CAP: usize = 64 * 1024;

/// Reads skill directory content directly from disk.
pub struct FsSkillContentReader;

impl FsSkillContentReader {
    /// Creates a new reader.
    pub fn new() -> Self {
        FsSkillContentReader
    }
}

impl Default for FsSkillContentReader {
    fn default() -> Self {
        Self::new()
    }
}

fn classify(path: &Path) -> SkillFileKind {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("md") | Some("markdown") => SkillFileKind::Markdown,
        Some("py") | Some("js") | Some("ts") | Some("jsx") | Some("tsx") | Some("sh")
        | Some("rb") | Some("rs") | Some("go") => SkillFileKind::Script,
        Some("json") | Some("yaml") | Some("yml") | Some("toml") => SkillFileKind::Config,
        _ => SkillFileKind::Other,
    }
}

/// Returns true for directory entries that should not appear in the explorer.
fn is_noise(name: &str) -> bool {
    matches!(name, ".git" | "target" | "node_modules") || name.starts_with(".ai-skill")
}

fn read_string_capped(path: &Path) -> Result<String, ContentError> {
    let bytes = std::fs::read(path).map_err(|_| ContentError::Io)?;
    let capped: Vec<u8> = if bytes.len() > PREVIEW_CAP {
        let mut v = bytes[..PREVIEW_CAP].to_vec();
        v.extend_from_slice(b"\n\n[... preview truncated ...]");
        v
    } else {
        bytes
    };
    Ok(String::from_utf8_lossy(&capped).into_owned())
}

fn dir_entries(dir: &Path) -> Result<Vec<std::fs::DirEntry>, ContentError> {
    let mut entries = std::fs::read_dir(dir)
        .map_err(|_| ContentError::Io)?
        .collect::<Result<Vec<_>, std::io::Error>>()
        .map_err(|_| ContentError::Io)?;
    entries.sort_by_key(|e| e.file_name());
    Ok(entries)
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<SkillTreeNode>) -> Result<(), ContentError> {
    for entry in dir_entries(dir)? {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if is_noise(&name) {
            continue;
        }
        let is_dir = path.is_dir();
        let is_subskill = is_dir && path.join("SKILL.md").is_file();
        let node = SkillTreeNode {
            name: name.clone(),
            path: path.clone(),
            is_dir,
            kind: classify(&path),
            is_subskill,
            depth,
        };
        out.push(node);
        if is_dir {
            walk(&path, depth + 1, out)?;
        }
    }
    Ok(())
}

impl SkillContentReader for FsSkillContentReader {
    fn read_preview(&self, dir: &Path) -> Result<SkillDoc, ContentError> {
        if !dir.is_dir() {
            return Err(ContentError::NotFound);
        }
        // README variants take precedence over the skill manifest.
        for name in ["README.md", "readme.md", "Readme.md", "SKILL.md"] {
            let candidate = dir.join(name);
            if candidate.is_file() {
                let mut content = read_string_capped(&candidate)?;
                if name == "SKILL.md"
                    && let Some(body) = extract_body(&content)
                {
                    content = body.to_string();
                }
                return Ok(SkillDoc {
                    title: name.to_string(),
                    content,
                });
            }
        }
        Err(ContentError::NotFound)
    }

    fn read_tree(&self, dir: &Path) -> Result<Vec<SkillTreeNode>, ContentError> {
        if !dir.is_dir() {
            return Err(ContentError::NotFound);
        }
        let mut out = Vec::new();
        walk(dir, 0, &mut out)?;
        Ok(out)
    }

    fn read_file(&self, path: &Path) -> Result<String, ContentError> {
        if !path.is_file() {
            return Err(ContentError::NotFound);
        }
        read_string_capped(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &Path, name: &str, body: &str) {
        std::fs::write(dir.join(name), body).unwrap();
    }

    #[test]
    fn readme_preview_takes_precedence_over_skill_md() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "SKILL.md", "---\nname: x\n---\nbody only");
        write(tmp.path(), "README.md", "# Read me\nwelcome");
        let reader = FsSkillContentReader::new();
        let doc = reader.read_preview(tmp.path()).unwrap();
        assert_eq!(doc.title, "README.md");
        assert!(doc.content.contains("welcome"));
    }

    #[test]
    fn skill_md_preview_strips_frontmatter() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "SKILL.md", "---\nname: x\n---\nbody only");
        let reader = FsSkillContentReader::new();
        let doc = reader.read_preview(tmp.path()).unwrap();
        assert_eq!(doc.title, "SKILL.md");
        assert!(doc.content.contains("body only"));
        assert!(!doc.content.contains("name: x"));
    }

    #[test]
    fn missing_preview_returns_not_found() {
        let tmp = TempDir::new().unwrap();
        let reader = FsSkillContentReader::new();
        assert_eq!(reader.read_preview(tmp.path()), Err(ContentError::NotFound));
    }

    #[test]
    fn read_tree_lists_files_and_nested_subskill() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "SKILL.md", "x");
        write(tmp.path(), "README.md", "r");
        let sub = tmp.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        write(&sub, "SKILL.md", "nested");
        write(&sub, "helper.py", "print(1)");
        let reader = FsSkillContentReader::new();
        let tree = reader.read_tree(tmp.path()).unwrap();
        let names: Vec<&str> = tree.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"SKILL.md"));
        assert!(names.contains(&"README.md"));
        assert!(names.contains(&"sub"));
        let sub_node = tree.iter().find(|n| n.name == "sub").unwrap();
        assert!(sub_node.is_dir);
        assert!(sub_node.is_subskill);
        let helper = tree.iter().find(|n| n.name == "helper.py").unwrap();
        assert_eq!(helper.kind, SkillFileKind::Script);
        assert_eq!(helper.depth, 1);
    }

    #[test]
    fn read_tree_skips_noise_entries() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "SKILL.md", "x");
        std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
        std::fs::create_dir_all(tmp.path().join("target")).unwrap();
        let reader = FsSkillContentReader::new();
        let tree = reader.read_tree(tmp.path()).unwrap();
        let names: Vec<&str> = tree.iter().map(|n| n.name.as_str()).collect();
        assert!(!names.contains(&".git"));
        assert!(!names.contains(&"target"));
    }

    #[test]
    fn read_file_returns_content() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "a.txt", "hello world");
        let reader = FsSkillContentReader::new();
        assert_eq!(
            reader.read_file(&tmp.path().join("a.txt")).unwrap(),
            "hello world"
        );
    }

    #[test]
    fn read_file_missing_returns_not_found() {
        let tmp = TempDir::new().unwrap();
        let reader = FsSkillContentReader::new();
        assert_eq!(
            reader.read_file(&tmp.path().join("nope.txt")),
            Err(ContentError::NotFound)
        );
    }
}
