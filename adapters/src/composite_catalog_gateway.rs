//! Composite catalog gateway that aggregates results from multiple sources.
//!
//! Sources are searched in order. Duplicate skill names (case-insensitive) are
//! collapsed — the first occurrence wins.

use std::collections::HashSet;

use ai_skill_core::{AnyCatalogGateway, CatalogEntry};

/// Aggregates results from multiple [`AnyCatalogGateway`] sources.
pub struct CompositeCatalogGateway {
    sources: Vec<Box<dyn AnyCatalogGateway>>,
}

impl CompositeCatalogGateway {
    pub fn new(sources: Vec<Box<dyn AnyCatalogGateway>>) -> Self {
        Self { sources }
    }
}

impl AnyCatalogGateway for CompositeCatalogGateway {
    fn search(&self, keyword: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for source in &self.sources {
            match source.search(keyword) {
                Ok(entries) => {
                    for entry in entries {
                        let lower = entry.name.to_lowercase();
                        if seen.insert(lower) {
                            results.push(entry);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[composite] source error: {e}");
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::AnyCatalogGateway;

    struct FakeGateway(Vec<CatalogEntry>);

    impl AnyCatalogGateway for FakeGateway {
        fn search(&self, _kw: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
            Ok(self.0.clone())
        }
    }

    struct ErrorGateway;

    impl AnyCatalogGateway for ErrorGateway {
        fn search(&self, _kw: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
            Err("oops".into())
        }
    }

    #[test]
    fn empty_sources_returns_empty() {
        let gw = CompositeCatalogGateway::new(vec![]);
        assert!(gw.search("test").unwrap().is_empty());
    }

    #[test]
    fn single_source_returns_its_results() {
        let gw = CompositeCatalogGateway::new(vec![Box::new(FakeGateway(vec![CatalogEntry {
            name: "alpha".into(),
            description: "".into(),
            url: None,
        }]))]);
        let results = gw.search("test").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "alpha");
    }

    #[test]
    fn deduplicates_by_name_case_insensitive() {
        let gw = CompositeCatalogGateway::new(vec![
            Box::new(FakeGateway(vec![CatalogEntry {
                name: "Alpha".into(),
                description: "first".into(),
                url: None,
            }])),
            Box::new(FakeGateway(vec![CatalogEntry {
                name: "alpha".into(),
                description: "second".into(),
                url: None,
            }])),
        ]);
        let results = gw.search("test").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "first");
    }

    #[test]
    fn error_source_is_skipped_gracefully() {
        let gw = CompositeCatalogGateway::new(vec![
            Box::new(ErrorGateway),
            Box::new(FakeGateway(vec![CatalogEntry {
                name: "alpha".into(),
                description: "".into(),
                url: None,
            }])),
        ]);
        let results = gw.search("test").unwrap();
        assert_eq!(results.len(), 1);
    }
}
