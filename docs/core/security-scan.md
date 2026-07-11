# Security Scan

The heuristic security scanner analyzes `SKILL.md` content for potentially dangerous patterns before installation. It runs automatically as part of the install wizard in the TUI.

## Public API

```rust
pub fn scan_skill(content: &str) -> Vec<ScanFinding>
```

Returns all findings for the given content. The scan is line-by-line and case-insensitive.

## Finding Model

```rust
pub struct ScanFinding {
    pub severity: Severity,
    pub category: ScanCategory,
    pub detail: String,
    pub line: usize,
}

pub enum Severity { High, Medium }

pub enum ScanCategory {
    DangerousShellPattern,
    EnvVarHarvest,
    HardcodedSecret,
    PromptInjection,
}
```

## Detection Rules

### DangerousShellPattern (High)

| Pattern | Detects |
|---|---|
| `rm -rf` | Recursive forced deletion |
| `curl ... \| ... bash` | Remote pipe-to-shell execution |
| `curl ... \| ... sh` | Remote pipe-to-shell execution |
| `wget ... \| ... bash` | Same via wget |
| `wget ... \| ... sh` | Same via wget |
| `eval` | Dynamic code evaluation |

### EnvVarHarvest (Medium)

Detects references to sensitive environment variables:
- `$aws_` (case-insensitive)
- `$secret_`
- `$token_`
- `$api_key`
- `$private_key`

### HardcodedSecret (High)

Detects inline assignments of secrets:
- `api_key = <non-empty-value>`
- `password = <non-empty-value>`
- `token = <non-empty-value>`
- `secret = <non-empty-value>`

### PromptInjection (High)

Detects phrases that may alter agent behavior:
- `ignore previous instructions`
- `disregard prior directives`
- Prompt override patterns

## Limitations

- **Heuristic only**: regex and keyword matching. False positives and false negatives are possible.
- **Manifest only**: scans `SKILL.md`, not referenced scripts or dependencies.
- **No import chain tracing**: does not build a dependency graph.
- **No community cross-reference**: does not check reputation databases.

The scan is a **safety gate**, not a guarantee. Always review untrusted skills.

---

[← Back to index](../index.md) · Related: [Skill Model](skill-model.md) · [Audit](audit.md)
