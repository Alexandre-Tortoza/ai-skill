# Frontmatter Parsing

Skills declare metadata in a YAML frontmatter block at the top of `SKILL.md`.

## Format

```markdown
---
name: my-skill
agents:
  - claude
tags:
  - coding
  - rust
---

# Skill body here

Markdown content after the closing `---`.
```

## Public API

### `parse_frontmatter`

```rust
pub fn parse_frontmatter(content: &str) -> Result<SkillMetadata, ParseError>
```

Extracts and parses the YAML frontmatter between `---` delimiters. Returns:

```rust
pub struct SkillMetadata {
    pub name: String,
    pub agents: Vec<String>,  // #[serde(default)]
    pub tags: Vec<String>,    // #[serde(default)]
}
```

### `extract_body`

```rust
pub fn extract_body(content: &str) -> Option<&str>
```

Returns everything after the closing `---` delimiter, trimmed. Returns `None` if no body exists.

### `ParseError`

```rust
pub enum ParseError {
    MissingDelimiters,  // No valid `---` frontmatter block
    Yaml(serde_norway::Error),  // YAML parse failure
}
```

## Internal Parsing

The parser uses `serde_norway` (a YAML serializer matching serde conventions). The internal deserialization struct is:

```rust
#[derive(Deserialize)]
struct SkillFrontmatter {
    name: String,
    #[serde(default)]
    agents: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}
```

Fields default to empty vectors when absent. The `name` field is required — if missing, YAML parsing fails.

## Edge Cases

| Input | Result |
|---|---|
| No `---` delimiters | `Err(MissingDelimiters)` |
| Only `---` with no closing `---` | `Err(MissingDelimiters)` |
| Malformed YAML between delimiters | `Err(Yaml(...))` |
| Valid frontmatter, no body | `name` and `agents` parsed, `extract_body` returns `None` |
| Valid frontmatter, body present | Both parsed, body trimmed |

---

[← Back to index](../index.md) · Related: [Skill Model](skill-model.md) · [Audit](audit.md)
