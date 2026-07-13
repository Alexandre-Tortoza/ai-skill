# Usage

`ai-skill` is a terminal user interface (TUI) for managing AI agent skills. It runs in the terminal at a minimum of **80×24** columns.

## Starting the TUI

```sh
ai-skill
```

Or during development:

```sh
cargo run -p ai-skill
```

## Command-Line Arguments

| Flag | Description |
|---|---|
| (no arguments) | Launches the interactive TUI |
| `-h`, `--help` | Prints usage information and exits |
| `-V`, `--version` | Prints the binary version and exits |
| `--json list` | Prints installed skills as JSON and exits |
| `--json audit` | Prints the aggregated audit report as JSON and exits |
| `--markdown audit` | Prints the aggregated audit report as Markdown and exits |

If the terminal is too small, a resize message is shown instead of a broken layout.

### Exportable Audit Report

For CI jobs or periodic reviews, export the same aggregate health report used by the TUI audit panel:

```sh
ai-skill --json audit > ai-skill-audit.json
ai-skill --markdown audit > ai-skill-audit.md
```

The report includes broken skills, duplicate names, skills without agent assignments, available updates, context budget usage and usage analytics (dead/stale skills).

### Usage Analytics (dead & stale)

`ai-skill` derives usage from local agent history (currently Claude Code transcripts in `~/.claude/projects/**/*.jsonl`). It detects skill invocations heuristically and classifies each installed skill as:

- **Dead** — never observed being used.
- **Stale** — not used for longer than the configured threshold (default **30 days**).

The threshold is set via `stale_after_days` in `~/.config/ai-skill/config.json`:

```json
{ "stale_after_days": 30 }
```

The audit panel (key `a`) shows `dead: N` and `stale: N` in its summary, with dedicated **Dead** and **Stale** sections when any are found. The same data is included in the exported `--json audit` and `--markdown audit` reports as `usage_dead`, `usage_stale` and `stale_after_days`.

## Customization

`ai-skill` reads `~/.config/ai-skill/config.json` for appearance and key bindings.

### Theme (semantic colors)

Colors are resolved from semantic slots, so you can recolor the UI without
touching code. Supported slot keys: `error`, `warning`, `success`, `accent`,
`muted`, `dead`, `stale`. Values are color names (`red`, `blue`, `darkgray`,
…) or `#rrggbb` hex. Unknown keys and invalid colors are ignored.

```json
{
  "theme": {
    "error": "red",
    "warning": "yellow",
    "success": "green",
    "accent": "cyan",
    "muted": "dark_gray",
    "dead": "magenta",
    "stale": "yellow"
  }
}
```

### Key bindings

The main navigation shortcuts are customizable. Action keys:
`quit`, `help`, `audit`, `search`, `create`, `profiles`, `bundles`, `budget`,
`editor`, `sync`, `ssh`, `adopt`, `toggle_name_only`, `disable`, `enable`,
`remove`, `update`. Key syntax: a single character (`a`, `?`, `/`), function
keys (`F1`–`F12`), or `ctrl+<letter>`. Letter bindings are matched
case-insensitively and ignore Shift. `quit` always also accepts `Ctrl-C`.

```json
{
  "keymap": {
    "audit": "A",
    "search": "s",
    "create": "c",
    "help": "?"
  }
}
```

Wizards and contextual keys (navigation, confirm/cancel) remain on their
built-in bindings in this first slice.

### Localization

The TUI ships with English (`en`, the default) and Brazilian Portuguese
(`pt-BR`). Set the `locale` field to switch the interface language for
panel titles, status-bar hints, the help overlay and the security scan
overlay:

```json
{
  "locale": "pt-BR"
}
```

Unrecognized or missing values fall back to English. The `--json` and
`--markdown` CLI reports remain in English regardless of this setting.

## Views (Modes)

The TUI has 11 views, each accessed by a key binding. The status bar at the bottom shows context-sensitive hints for the active view.

### List View (default)

Shows all installed skills grouped by scope (Global, Project).

| Key | Action |
|---|---|
| `↑` / `↓` | Navigate the list |
| `Enter` | Open detail view for the selected skill |
| `/` | Focus search query input |
| `t` | Filter by tag |
| `s` | Open security scan report for the selected skill |
| `p` | Open profiles panel |
| `a` | Open audit report |
| `c` | Open creation wizard |
| `e` | Open frontmatter editor for the selected skill |
| `Space` | Toggle multi-select |
| `d` / `r` / `u` | Disable / remove / update selected skills (with confirmation) |
| `?` | Open help overlay |
| `q` / `Esc` | Quit application |

Validation states are shown as colored badges:
- **Valid** — green
- **BrokenSymlink** — red
- **MissingManifest** — yellow
- **InvalidFrontmatter** — yellow
- **OrphanLock** — red
- **Duplicate** — red
- **Disabled** — dim

Drift indicators show whether a skill is up-to-date or has an upstream update available.

If the status bar shows `reload:on`, `ai-skill` is watching an existing skill root and refreshes the inventory when files change. Claude Code 2.1+ reloads changed skills without restarting the agent; the TUI indicator confirms that `ai-skill` is tracking the same live filesystem changes.

### Detail View

Shows full metadata and the rendered `SKILL.md` body for the selected skill.

| Key | Action |
|---|---|
| `↑` / `↓` | Scroll content |
| `Esc` / `q` | Return to list |

### Search View

Searches the remote catalog via `npx skills find`.

| Key | Action |
|---|---|
| Type a query | Incremental search as you type |
| `↑` / `↓` | Navigate results |
| `Enter` | Start install wizard for the selected result |
| `Esc` | Return to list |

### Help Overlay

Shows all key bindings in a centered popup.

| Key | Action |
|---|---|
| `Esc` / `q` / `?` | Close help overlay |

### Confirm Dialog

Centered confirmation prompt for destructive actions (delete, disable, remove).

| Key | Action |
|---|---|
| `y` | Confirm |
| `n` / `Esc` | Cancel |

### Install Wizard

Steps through installing a new skill: name, scope (global/project), agent selection, then security scan gate.

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Cycle through fields |
| `Enter` | Confirm step / proceed past scan |
| `Esc` | Cancel installation |

The security scan runs automatically before installation. If findings are detected, you must explicitly confirm to proceed.

### Scan Report

Shows heuristic security scan findings for a selected skill.

| Key | Action |
|---|---|
| `↑` / `↓` | Scroll findings |
| `Esc` | Close report |

Findings are categorized by severity (High/Medium) and type:
- **DangerousShellPattern** — risky shell commands (`rm -rf`, `curl|bash`, `eval`)
- **EnvVarHarvest** — environment variable collection
- **HardcodedSecret** — potential secrets in the skill content
- **PromptInjection** — patterns that may alter agent behavior

### Profiles Panel

Manages named profiles (sets of skills).

| Key | Action |
|---|---|
| `↑` / `↓` | Select profile |
| `Enter` | Activate selected profile |
| `c` | Create new profile from current state |
| `d` | Delete selected profile |
| `Esc` | Return to list |

When a profile is activated, `ai-skill` computes the diff between the current and desired state and executes the minimal batch of install/remove operations.

### Create Wizard

4-step wizard for scaffolding a new `SKILL.md`:

1. **Name** — enter the skill name
2. **Agents** — select target agents
3. **Tags** — add tags
4. **Preview** — review generated frontmatter and body

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Cycle fields |
| `Enter` | Confirm step / create |
| `Esc` | Cancel |

### Editor Panel

Split view for editing an existing skill's frontmatter.

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Cycle fields (Name, Agents, Tags) |
| `Enter` | Save changes |
| `Esc` | Discard and return |

The left pane shows editable form fields; the right pane shows a live preview of the resulting `SKILL.md`.

### Audit Panel

4-column aggregated report of all skills:

| Column | Content |
|---|---|
| Broken | Skills with broken symlinks, missing manifests, or orphan locks |
| Duplicates | Skills with case-insensitive name collisions across scopes |
| No Agents | Skills that declare no target agents |
| Updates | Skills with upstream drift (update available) |

| Key | Action |
|---|---|
| `↑` / `↓` | Scroll |
| `Esc` | Return to list |

## Multi-Select

In List View, press `Space` to toggle selection on individual skills. Selected skills are highlighted. Bulk actions (`d`, `r`, `u`) apply to all selected items.

## Environment

- `NO_COLOR`: if set, output respects it (16-color palette, no ANSI true color).

---

[← Back to index](index.md) · Related: [Installation](installation.md) · [Views](tui/views.md) · [App State](tui/app-state.md)
