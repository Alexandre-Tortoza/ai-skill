# App State Machine

The `App<G, I, T>` struct is the central state machine. It holds all application state, receives events, and dispatches to per-view handlers.

## Generic Parameters

```rust
pub struct App<G: AnyCatalogGateway, I: SkillInstaller, T: SkillToggler> {
    // ... state fields ...
    pub catalog: G,
    pub installer: I,
    pub toggler: T,
    pub profile_store: Box<dyn ProfileStore>,
    pub creator: Box<dyn SkillCreator>,
    pub writer: Box<dyn SkillWriter>,
}
```

Three ports are generic parameters (monomorphized at compile time); the rest are `Box<dyn Trait>` (object-safe dispatch).

## State Fields

| Field | Type | Purpose |
|---|---|---|
| `all_skills` | `Vec<Skill>` | The full inventory, refreshed on change |
| `view` | `View` | Current active view |
| `view_before_confirm` | `View` | View to return to after confirmation |
| `list_state` | `ListUiState` | List filter, selection, multi-select |
| `detail_scroll` | `u16` | Scroll position in detail view |
| `diff_scroll` | `u16` | Scroll position in diff view |
| `preview_scroll` | `u16` | Scroll position in list split-preview pane |
| `explorer_nodes` | `Vec<SkillTreeNode>` | Directory tree of the explored skill |
| `explorer_selected_index` | `usize` | Selected node in the explorer tree |
| `explorer_scroll` | `u16` | Scroll position in explorer file pane |
| `explorer_file_content` | `Option<String>` | Right-pane content for the selected node |
| `explorer_title` | `String` | Title (skill name) of the active explorer |
| `content_reader` | `Box<dyn SkillContentReader>` | Reads skill file content from disk |
| `search_state` | `SearchState` | Query, results, selection |
| `install_wizard_state` | `InstallWizardState` | Install wizard fields |
| `pending_action` | `Option<AppAction>` | Action awaiting confirmation |
| `needs_refresh` | `bool` | Flag for TUI render refresh |
| `last_error` | `Option<String>` | Error message to display |
| `profile_state` | `ProfileState` | Profile list and creation |
| `scan_findings` | `Vec<ScanFinding>` | Security scan results |
| `create_wizard_state` | `CreateWizardState` | Creation wizard fields |
| `editor_state` | `Option<EditorState>` | Editor state (None when not editing) |
| `should_quit` | `bool` | Exit flag |

## View Enum

```rust
pub enum View {
    List,           // Split list + README preview (default)
    Detail,         // Skill detail + manifest body
    Search,         // Remote catalog search
    Help,           // Key binding overlay
    Confirm,        // Confirmation dialog
    InstallWizard,  // Install steps
    ScanReport,     // Security findings
    Profiles,       // Profile management
    CreateWizard,   // Creation steps
    Editor,         // Frontmatter editor
    Audit,          // Aggregated audit report
    Diff,           // Upstream diff of a skill's manifest
    Explorer,       // Directory explorer for a single skill
}
```

## Key Dispatch

```rust
pub fn handle_event(&mut self, event: AppEvent) {
    match event {
        AppEvent::Key(key) => match self.view {
            View::List         => self.handle_list_key(key),
            View::Detail       => self.handle_detail_key(key),
            View::Search       => self.handle_search_key(key),
            View::Help         => self.handle_help_key(key),
            View::Confirm      => self.handle_confirm_key(key),
            View::InstallWizard => self.handle_install_wizard_key(key),
            View::ScanReport   => self.handle_scan_report_key(key),
            View::Profiles     => self.handle_profiles_key(key),
            View::CreateWizard => self.handle_create_wizard_key(key),
            View::Editor       => self.handle_editor_key(key),
            View::Audit        => self.handle_audit_key(key),
            View::Diff         => self.handle_diff_key(key),
            View::Explorer     => self.handle_explorer_key(key),
        },
        AppEvent::Resize => { /* no state change, TUI redraws */ },
    }
}
```

## AppAction

```rust
pub enum AppAction {
    Install { name: String, agents: Vec<String>, scope: Scope },
    Remove { path: PathBuf },
    Update { path: PathBuf },
    Enable { path: PathBuf },
    Disable { path: PathBuf },
    Adopt { path: PathBuf },
    ActivateProfile { name: String, ops: Vec<ProfileOp> },
}
```

Actions are created by view handlers, stored as `pending_action`, and executed after user confirmation.

## Constructor

```rust
pub fn new(
    all_skills: Vec<Skill>,
    catalog: G, installer: I, toggler: T,
    profile_store: Box<dyn ProfileStore>,
    creator: Box<dyn SkillCreator>,
    writer: Box<dyn SkillWriter>,
) -> Self
```

Initial state: `View::List`, selected index 0, no pending action, all filters at defaults.

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [Views](views.md)
