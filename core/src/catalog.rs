//! Remote skill catalog entry model and query port.

/// A single skill listed in a remote catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogEntry {
    /// Unique skill name.
    pub name: String,
    /// One-line description.
    pub description: String,
    /// Optional URL for more information or the source repository.
    pub url: Option<String>,
}

/// Port for searching a remote skill catalog.
pub trait AnyCatalogGateway {
    /// Searches for skills matching `keyword` and returns matching entries.
    fn search(&self, keyword: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeGateway(Vec<CatalogEntry>);

    impl AnyCatalogGateway for FakeGateway {
        fn search(&self, _keyword: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn catalog_entry_fields_accessible() {
        let entry = CatalogEntry {
            name: "my-skill".to_string(),
            description: "Does things".to_string(),
            url: Some("https://example.com".to_string()),
        };
        assert_eq!(entry.name, "my-skill");
        assert_eq!(entry.description, "Does things");
    }

    #[test]
    fn any_catalog_gateway_trait_object_works() {
        let gw: Box<dyn AnyCatalogGateway> = Box::new(FakeGateway(vec![CatalogEntry {
            name: "alpha".to_string(),
            description: "Alpha skill".to_string(),
            url: None,
        }]));
        let results = gw.search("alpha").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "alpha");
    }
}
