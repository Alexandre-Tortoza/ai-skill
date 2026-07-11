# Security

`ai-skill` handles skills that can influence AI agents to read files, execute commands, or manipulate the local environment. Security is a first-class concern throughout the tool.

## Heuristic Security Scan

Before installing a skill, the built-in heuristic scanner analyzes the `SKILL.md` content for potentially dangerous patterns. The scan runs automatically as part of the install wizard, and findings must be explicitly acknowledged before installation proceeds.

### Scan Categories

| Category | Severity | What it detects |
|---|---|---|
| `DangerousShellPattern` | High | Risky shell commands: `rm -rf` (recursive deletion), `curl|bash` (remote execution), `eval` (dynamic evaluation), and similar patterns |
| `EnvVarHarvest` | Medium | Collection of environment variables that may expose secrets (`AWS_*`, `API_KEY`, `TOKEN`, `SECRET`, `PASSWORD`) |
| `HardcodedSecret` | High | Apparent secrets hardcoded in skill content (API keys, tokens, passwords in plain text) |
| `PromptInjection` | Medium | Patterns that may alter agent behavior in unexpected ways — instructions embedded in user-facing content |

### Limitations

- The scan is **heuristic**, not a deep static analysis. It detects patterns via regex and keyword matching, which means:
  - False positives are possible (benign content matching a dangerous pattern).
  - False negatives are possible (obfuscated or novel attack patterns not in the rule set).
- The scan only analyzes the `SKILL.md` manifest, not referenced scripts or dependencies.
- Import chain tracing (building a dependency graph of scripts) is not yet implemented.
- Cross-reference with community reputation databases is not yet implemented.

The scan is a **safety gate**, not a guarantee. Always review skills from untrusted sources before installation.

## Key Security Properties

| Property | Implementation |
|---|---|
| **Symlink safety** | `FsSkillRepository` resolves symlinks and reports broken ones as `BrokenSymlink` validation state |
| **Path safety** | All filesystem operations use resolved absolute paths; no command injection via skill names |
| **Gate before action** | Install, remove, update, disable, and adopt all require explicit confirmation (key `y`) |
| **Disabled state** | Skills can be disabled via `.disabled` suffix, preventing agents from loading them |
| **Adoption markers** | `adopt` creates a `.ai-skill` marker file to track managed skills |

## Responsible Disclosure

If you discover a security vulnerability, please report it via the process described in [`SECURITY.md`](../SECURITY.md). Do not publish details until a fix or mitigation is available.

In-scope concerns include:
- Unexpected command execution via skill content.
- Bypass of the security scan gate.
- Unauthorized file read or exposure.
- Unsafe symlink, path, or profile handling.
- Vulnerabilities in `SKILL.md` or frontmatter parsing.

---

[← Back to index](index.md) · Related: [Heuristic Scan](core/security-scan.md) · [Installation](installation.md)
