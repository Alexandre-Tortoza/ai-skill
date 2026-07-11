# NPX Catalog (`NpxCatalogGateway`)

Searches the remote skill catalog by shelling out to `npx skills find`.

## Port

```rust
impl AnyCatalogGateway for NpxCatalogGateway {
    fn search(&self, keyword: &str)
        -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>>;
}
```

## How It Works

1. Executes `npx skills find <keyword>` via `std::process::Command`
2. Captures stdout
3. Parses tab-separated output

## Output Parsing

The expected output format from `npx skills find` is tab-separated:

```
name\t[description]\t[url]
```

```rust
fn parse_npx_output(raw: &[u8]) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>>
```

| Column | Required | Maps to |
|---|---|---|
| Name | Yes | `CatalogEntry.name` |
| Description | No | `CatalogEntry.description` |
| URL | No | `CatalogEntry.url` |

Empty input → empty `Vec`.

## `CatalogEntry`

```rust
pub struct CatalogEntry {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
}
```

## Dependencies

Requires `npx` (Node.js) in `PATH`. Tests that call `npx` are marked `#[ignore = "requires npx with skills package in PATH"]`.

## Error Scenarios

- `npx` not installed → `Io(NotFound)` error
- `npx skills` not found → `Io(NotFound)` or `NonZeroExit`
- Malformed output → parse errors propagated as `Box<dyn Error>`

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [CLI Installer](cli-installer.md)
