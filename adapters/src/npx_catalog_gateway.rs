//! Adapter that queries the remote skill catalog via `npx skills find`.

use ai_skill_core::{AnyCatalogGateway, CatalogEntry};
use std::path::Path;

/// Searches the remote catalog by shelling out to `npx skills find <keyword>`.
pub struct NpxCatalogGateway;

impl NpxCatalogGateway {
    fn search_with_npx(
        &self,
        npx: &Path,
        keyword: &str,
    ) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
        let output = std::process::Command::new(npx)
            .args(["skills", "find", keyword])
            .output()?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            return Err(format!("npx skills exited with status {code}").into());
        }

        parse_npx_output(&output.stdout)
    }
}

impl AnyCatalogGateway for NpxCatalogGateway {
    fn search(&self, keyword: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
        self.search_with_npx(Path::new("npx"), keyword)
    }
}

/// Parses tab-separated lines of `name\tdescription[\turl]` from `npx skills find`.
fn parse_npx_output(raw: &[u8]) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
    let text = std::str::from_utf8(raw)?;
    let mut entries = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((name, rest)) = line.split_once('\t') {
            let (description, url) = if let Some((desc, url)) = rest.split_once('\t') {
                (desc.to_string(), Some(url.to_string()))
            } else {
                (rest.to_string(), None)
            };
            entries.push(CatalogEntry {
                name: name.to_string(),
                description,
                url,
            });
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::AnyCatalogGateway;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn write_executable(path: &Path, content: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        drop(file);
        fs::set_permissions(path, PermissionsExt::from_mode(0o755)).unwrap();
    }

    #[test]
    fn search_without_npx_returns_error() {
        let dir = TempDir::new().unwrap();
        let missing_npx = dir.path().join("npx");

        let gw = NpxCatalogGateway;
        let result = gw.search_with_npx(&missing_npx, "test");

        assert!(result.is_err());
    }

    #[test]
    fn search_with_mock_npx_returns_parsed_entries() {
        let dir = TempDir::new().unwrap();
        let mock_path = dir.path().join("npx");
        let output = "my-skill\tDoes things\thttps://example.com\nother\tOther skill\n";
        write_executable(&mock_path, &format!("#!/bin/sh\nprintf '%s' '{output}'\n"));

        let gw = NpxCatalogGateway;
        let results = gw.search_with_npx(&mock_path, "test").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "my-skill");
        assert_eq!(results[0].url, Some("https://example.com".to_string()));
        assert_eq!(results[1].name, "other");
    }

    #[test]
    fn parse_output_with_name_and_description() {
        let raw = b"omarchy\tOmarchy WM skill\n";
        let entries = parse_npx_output(raw).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "omarchy");
        assert_eq!(entries[0].description, "Omarchy WM skill");
        assert!(entries[0].url.is_none());
    }

    #[test]
    fn parse_output_with_name_description_and_url() {
        let raw = b"my-skill\tDoes things\thttps://example.com\n";
        let entries = parse_npx_output(raw).unwrap();
        assert_eq!(entries[0].url, Some("https://example.com".to_string()));
    }

    #[test]
    fn parse_empty_output_returns_empty_vec() {
        let entries = parse_npx_output(b"").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_multiple_entries() {
        let raw = b"alpha\tAlpha skill\nbeta\tBeta skill\n";
        let entries = parse_npx_output(raw).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].name, "beta");
    }

    #[test]
    fn trait_object_compiles() {
        let _gw: Box<dyn AnyCatalogGateway> = Box::new(NpxCatalogGateway);
    }

    #[test]
    #[ignore = "requires npx with skills package in PATH"]
    fn live_search_returns_results() {
        let gw = NpxCatalogGateway;
        let results = gw.search("omarchy").unwrap();
        assert!(!results.is_empty());
    }
}
