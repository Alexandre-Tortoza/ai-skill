# TUI Views

The TUI has 13 views. Each is rendered by a dedicated panel function.

## List View (default)

**Panel:** `split_preview_panel::render_split_preview`

Split layout:

- **Left (40%):** `installed_panel::render_installed_panel` — all installed skills as a
  scrollable list with colored status badges.
- **Right (60%):** live preview of the selected skill's `README.md`/`SKILL.md`.

Displays all installed skills with colored status badges:

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

## Explorer View

**Panel:** `skill_explorer_panel::render_skill_explorer`

Opened with `Enter` from the List view. Split layout:

- **Left (40%):** a depth-first directory tree of the skill. Directories use `▾`, nested
  sub-skills (directories containing `SKILL.md`) use `▾◈`, and files are marked by kind
  (`▸` markdown, `$` script, `#` config, `·` other).
- **Right (60%):** content of the selected file, or a `README.md`/`SKILL.md` preview when a
  directory is selected.

## Command Palette

**Overlay:** `command_palette::render_command_palette`

Opened with `Ctrl+P` from any view. Lists every available action as a searchable list:

- Search catalog, New skill, Audit report, Context budget, Profiles & presets, Bundles, Git sync, Settings, Help
- When a skill is selected in the list: Open detail, Edit skill, Disable, Remove, Update, and Upstream diff (only when an update is available)

Navigation: `↑`/`↓` (or `j`/`k`) to move, `Enter` to run the selected command, `Esc` to close. This removes the need to expose every shortcut on the status bar.

## Key Bindings by View

| Key | List | Detail | Search | Help | Confirm | Wizard | Scan | Profiles | Create | Editor | Audit | Diff | Explorer |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `↑`/`↓` | Navigate | Scroll | Navigate | — | — | — | Scroll | Navigate | — | — | Scroll | Scroll | Navigate |
| `j`/`k` | — | — | — | — | — | — | — | — | — | — | Scroll | Scroll | Navigate |
| `←`/`→` | — | — | — | — | — | — | — | — | — | — | — | — | Parent/Child |
| `PgUp`/`PgDn` | — | — | — | — | — | — | — | — | — | — | — | — | Scroll |
| `Enter` | Explorer | — | Wizard | — | Confirm | Next | Proceed | Activate | Next | Save | — | — | Open |
| `Esc` | — | Back | Back | Close | Cancel | Cancel | Cancel | Back | Cancel | Cancel | Back | Back | Back |
| `Ctrl+P` | Palette | Palette | Palette | Palette | Palette | Palette | Palette | Palette | Palette | Palette | Palette | Palette | Palette |
| `Ctrl-C` | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 | Quit×2 |
| `/` | Search | — | — | — | — | — | — | — | — | — | — | — | — |
| `Tab` | Filter | — | — | — | — | Scope | — | — | Step | Field | — | — | — |
| `Space` | Select | — | — | — | — | Agent | — | — | — | — | — | — | — |
| `t` | Tag | — | — | — | — | — | — | — | — | — | — | — | — |
| `s` | Scan | — | — | — | — | — | — | — | — | — | — | — | — |
| `p` | Profiles | — | — | — | — | — | — | — | — | — | — | — | — |
| `a` | Audit | — | — | — | — | — | — | — | — | — | — | — | — |
| `c` | Create | — | — | — | — | — | — | Create | — | — | — | — | — |
| `e` | Edit | — | — | — | — | — | — | — | — | — | — | — | — |
| `d` | Disable | Diff | — | — | — | — | — | Delete | — | — | — | — | — |
| `r` | Remove | — | — | — | — | — | — | — | — | — | — | — | — |
| `u` | Update | — | — | — | — | — | — | — | — | — | — | — | — |
| `?` | Help | — | — | Close | — | — | — | — | — | — | — | — | — |
| `y` / `n` | — | — | — | — | Yes/No | — | — | — | — | — | — | — | — |

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [App State](app-state.md)
