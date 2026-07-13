# TUI Views

The TUI has 12 views. Each is rendered by a dedicated panel function.

## List View (default)

**Panel:** `installed_panel::render_installed_panel`

Displays all installed skills as a scrollable list with colored status badges:

| Badge | Color | State |
|---|---|---|
| `[broken-symlink]` | Red | `BrokenSymlink` |
| `[no-manifest]` | Red | `MissingManifest` |
| `[bad-frontmatter]` | Yellow | `InvalidFrontmatter` |
| `[orphan-lock]` | Magenta | `OrphanLock` |
| `[duplicate]` | Cyan | `Duplicate` |
| `[disabled]` | Dark Gray | `Disabled` |
| `[↑]` | Yellow | `DriftState::UpdateAvailable` |

Filters: scope (All/Global/Project via Tab), tag (cycle via `t`).

## Detail View

**Panel:** `detail_panel::render_detail_panel`

Two sections:
- **Metadata**: scope, agents, path, validation state, drift hashes
- **Body**: the `SKILL.md` manifest content with scrolling

## Search View

**Panel:** `search_panel::render_search_panel`

- Query input row at top
- Results list (left 40%) + preview pane (right 60%)
- Incremental search as user types
- Error state displayed in preview pane

## Help Overlay

**Panel:** `help_overlay::render_help_overlay`

Centered popup (60×16) showing all key bindings. Rendered on top of the List panel.

## Confirm Dialog

**Panel:** `confirm_panel::render_confirm_panel`

Centered popup (70% width, 7 lines) showing the action preview and `(y)es / (n)o` prompt. Rendered on top of the List panel.

## Install Wizard

**Panel:** `install_wizard::render_install_wizard`

4 sections:
1. **Skill name** — from search selection
2. **Scope selector** — toggle Global/Project via Tab
3. **Agents list** — toggle individual agents via Space
4. **Security scan** — automatic, gates confirmation

## Scan Report

**Panel:** `scan_report::render_scan_report`

Red-bordered popup listing security findings with severity coloring. Footer: "Enter to proceed | Esc to cancel".

## Profiles Panel

**Panel:** `profiles_panel::render_profiles_panel`

- Left 40%: profile list with skill count
- Right 60%: selected profile detail or creation input
- Actions: create, activate, delete

## Create Wizard

**Panel:** `create_wizard::render_create_wizard`

4-step form (single panel with active step highlighted):
1. **Name** — text input
2. **Agents** — text input (comma-separated)
3. **Tags** — text input (comma-separated)
4. **Preview** — generated SKILL.md scaffold

## Editor Panel

**Panel:** `editor_panel::render_editor_panel`

Split view:
- Left 40%: form with Name, Agents, Tags fields (active field highlighted in yellow)
- Right 60%: live preview of the edited SKILL.md body

## Audit Panel

**Panel:** `audit_panel::render_audit_panel`

Summary line + 4 sections:

| Section | Color | Content |
|---|---|---|
| Broken | Red | BrokenSymlink, MissingManifest, InvalidFrontmatter, OrphanLock |
| Duplicates | Cyan | Duplicate |
| No Agents | Yellow | Valid/Disabled with empty agents |
| Updates | Green | DriftState::UpdateAvailable |

## Diff View

**Panel:** `diff_panel::render_diff_panel`

Color-coded upstream diff of a skill's `SKILL.md` (`git diff HEAD..@{u}`), shown only when an
update is available (opened from the Detail view via `d`). Additions are green, removals red,
headers accented. Requires the skill to be a Git checkout with an upstream tracking branch.

## Key Bindings by View

| Key | List | Detail | Search | Help | Confirm | Wizard | Scan | Profiles | Create | Editor | Audit | Diff |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `↑`/`↓` | Navigate | Scroll | Navigate | — | — | — | Scroll | Navigate | — | — | Scroll | Scroll |
| `j`/`k` | — | — | — | — | — | — | — | — | — | — | — | Scroll |
| `Enter` | Detail | — | Wizard | — | Confirm | Next | Proceed | Activate | Next | Save | — | — |
| `Esc` | Quit | Back | Back | Close | Cancel | Cancel | Cancel | Back | Cancel | Cancel | Back | Back |
| `q` | Quit | Back | — | Close | — | — | — | — | — | — | — | Quit |
| `/` | Search | — | — | — | — | — | — | — | — | — | — | — |
| `Tab` | Filter | — | — | — | — | Scope | — | — | Step | Field | — | — |
| `Space` | Select | — | — | — | — | Agent | — | — | — | — | — | — |
| `t` | Tag | — | — | — | — | — | — | — | — | — | — | — |
| `s` | Scan | — | — | — | — | — | — | — | — | — | — | — |
| `p` | Profiles | — | — | — | — | — | — | — | — | — | — | — |
| `a` | Audit | — | — | — | — | — | — | — | — | — | — | — |
| `c` | Create | — | — | — | — | — | — | Create | — | — | — | — |
| `e` | Edit | — | — | — | — | — | — | — | — | — | — | — |
| `d` | Disable | Diff | — | — | — | — | — | Delete | — | — | — | — |
| `r` | Remove | — | — | — | — | — | — | — | — | — | — | — |
| `u` | Update | — | — | — | — | — | — | — | — | — | — | — |
| `?` | Help | — | — | Close | — | — | — | — | — | — | — | — |
| `y` / `n` | — | — | — | — | Yes/No | — | — | — | — | — | — | — |

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [App State](app-state.md)
