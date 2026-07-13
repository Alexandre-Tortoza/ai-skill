//! Application state, view transitions, and event handling.

use crate::ui::keymap::{Action, KeyBindings};
use ai_skill_adapters::ImportChainResult;
use ai_skill_core::{
    AnyCatalogGateway, Bundle, BundleStore, CatalogEntry, ConfigStore, ConnectionStatus,
    ContextBudget, DriftState, ExternalScanner, LintWarning, Phase, Profile, ProfileOp,
    ProfileStore, ProjectSettings, RemoteHost, RemoteSkill, ScanFinding, Scope, SettingsStore,
    SignatureVerifier, Skill, SkillCreator, SkillInstaller, SkillMode, SkillSync, SkillToggler,
    SkillWriter, Snapshot, SshConnector, SyncStatus, TuiConfig, calculate_budget, cross_reference,
    scan_skill,
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::event::AppEvent;
use crate::ui::settings_panel::{ConfigState, SettingsState};

/// The active screen (or overlay) in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
    Search,
    Help,
    Confirm,
    InstallWizard,
    ScanReport,
    Profiles,
    CreateWizard,
    Editor,
    Audit,
    Budget,
    #[allow(dead_code)]
    Settings,
    ImportChain,
    SshRemote,
    Bundles,
    Sync,
    /// Upstream diff of a skill with an available update.
    Diff,
}

/// Steps in the create-skill wizard.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CreateStep {
    #[default]
    /// Entering the skill name.
    Name,
    /// Entering agent names (comma-separated).
    Agents,
    /// Entering tags (comma-separated).
    Tags,
    /// Preview before creating.
    Preview,
}

/// State for the create-skill wizard workflow.
#[derive(Debug, Default)]
pub struct CreateWizardState {
    /// Current wizard step.
    pub step: CreateStep,
    /// Skill name being entered.
    pub name: String,
    /// Raw agents input string.
    pub agents_input: String,
    /// Raw tags input string.
    pub tags_input: String,
    /// Validation errors for the current input.
    pub errors: Vec<String>,
}

/// Field being edited in the skill editor.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum EditField {
    #[default]
    Name,
    Agents,
    Tags,
}

/// State for the in-app skill manifest editor.
#[derive(Debug)]
pub struct EditorState {
    /// The skill being edited (cloned from the main list).
    pub skill: Skill,
    /// Currently focused field.
    pub field: EditField,
    /// Current name input.
    pub name_input: String,
    /// Current agents input.
    pub agents_input: String,
    /// Current tags input.
    pub tags_input: String,
    /// Linter warnings for the current skill content.
    pub warnings: Vec<LintWarning>,
}

/// An action that can be confirmed or rejected by the user.
#[derive(Debug, Clone, PartialEq)]
pub enum AppAction {
    /// Install a skill from the catalog.
    Install {
        name: String,
        agents: Vec<String>,
        scope: Scope,
    },
    /// Remove an installed skill.
    Remove { path: PathBuf },
    /// Update an installed skill.
    Update { path: PathBuf },
    /// Enable a disabled skill.
    Enable { path: PathBuf },
    /// Disable a skill.
    Disable { path: PathBuf },
    /// Adopt an unmanaged skill.
    Adopt { path: PathBuf },
    /// Activate a named profile (install/remove ops).
    ActivateProfile { name: String, ops: Vec<ProfileOp> },
    /// Toggle a skill between name-only and full mode.
    ToggleNameOnly { path: PathBuf },
}

/// A command selectable from the floating command palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCommand {
    /// Open the catalog search panel.
    Search,
    /// Open the create-skill wizard.
    Create,
    /// Open the audit panel.
    Audit,
    /// Open the budget panel.
    Budget,
    /// Open the profiles panel.
    Profiles,
    /// Open the bundles panel.
    Bundles,
    /// Open the sync panel.
    Sync,
    /// Open the settings panel.
    Settings,
    /// Open the help overlay.
    Help,
    /// Open the detail view for the selected skill.
    OpenDetail,
    /// Open the editor for the selected skill.
    Edit,
    /// Disable the selected skill.
    Disable,
    /// Remove the selected skill.
    Remove,
    /// Update the selected skill.
    Update,
    /// Show the upstream diff for the selected skill.
    Diff,
}

/// State for the profiles panel.
#[derive(Debug, Default)]
pub struct ProfileState {
    /// Loaded profiles.
    pub profiles: Vec<Profile>,
    /// Index of the selected profile in the list.
    pub selected_index: usize,
    /// Raw input for creating a new profile.
    pub new_name_input: String,
    /// Whether the user is creating a new profile.
    pub creating: bool,
}

/// State for the install-from-catalog wizard.
#[derive(Debug, Default)]
pub struct InstallWizardState {
    /// The catalog entry being installed.
    pub entry: Option<CatalogEntry>,
    /// Installation scope selected by the user.
    pub scope: Scope,
    /// Agent options available for the entry.
    pub available_agents: Vec<String>,
    /// Agents that the user has selected.
    pub selected_agents: Vec<String>,
}

/// Filter for skills by installation scope.
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeFilter {
    /// Show both global and project skills.
    All,
    /// Show only global skills.
    Global,
    /// Show only project-scoped skills.
    Project,
}

impl ScopeFilter {
    pub fn next(&self) -> Self {
        match self {
            ScopeFilter::All => ScopeFilter::Global,
            ScopeFilter::Global => ScopeFilter::Project,
            ScopeFilter::Project => ScopeFilter::All,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ScopeFilter::All => "all",
            ScopeFilter::Global => "global",
            ScopeFilter::Project => "project",
        }
    }
}

/// State for the main skill-list panel (filtering, selection, multi-select).
#[derive(Debug)]
pub struct ListUiState {
    pub scope_filter: ScopeFilter,
    pub agent_filter: Option<String>,
    pub tag_filter: Option<String>,
    pub selected_index: usize,
    pub selected_items: Vec<usize>,
}

impl ListUiState {
    fn new() -> Self {
        Self {
            scope_filter: ScopeFilter::All,
            agent_filter: None,
            tag_filter: None,
            selected_index: 0,
            selected_items: vec![],
        }
    }
}

/// State for the catalog search panel.
#[derive(Debug, Default)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<CatalogEntry>,
    pub selected_index: usize,
    pub error: Option<String>,
}

/// State for the SSH remote management panel.
#[derive(Debug, Default)]
pub struct SshState {
    /// Configured remote hosts the user can connect to.
    pub hosts: Vec<RemoteHost>,
    /// Index of the selected host.
    pub selected_index: usize,
    /// Skills listed from the selected host.
    pub skills: Vec<RemoteSkill>,
    /// Result of the last connection check.
    pub connection_status: Option<ConnectionStatus>,
    /// Error message from the last operation.
    pub error: Option<String>,
}

/// State for the bundles panel.
#[derive(Debug, Default)]
pub struct BundleState {
    /// Available bundles from the store.
    pub bundles: Vec<Bundle>,
    /// Index of the selected bundle.
    pub selected_index: usize,
    /// Result message after attempting installation.
    pub result_message: Option<String>,
}

/// State for the sync panel.
#[derive(Debug, Default)]
pub struct SyncState {
    /// Available snapshots.
    pub snapshots: Vec<Snapshot>,
    /// Current sync status.
    pub status: Option<SyncStatus>,
    /// Index of the selected snapshot.
    pub selected_index: usize,
    /// Message for snapshot creation.
    pub snapshot_message: String,
    /// Remote URL input.
    pub remote_input: String,
    /// Branch name input.
    pub branch: String,
    /// Error or success message.
    pub message: Option<String>,
    /// Whether we're prompting for snapshot message.
    pub creating_snapshot: bool,
    /// Whether we're prompting for remote.
    pub configuring_remote: bool,
}

/// Top-level application state, generic over adapters.
pub struct App<G: AnyCatalogGateway, I: SkillInstaller, T: SkillToggler> {
    pub all_skills: Vec<Skill>,
    pub view: View,
    pub view_before_confirm: View,
    pub list_state: ListUiState,
    pub detail_scroll: u16,
    pub diff_scroll: u16,
    pub search_state: SearchState,
    pub install_wizard_state: InstallWizardState,
    pub pending_action: Option<AppAction>,
    pub needs_refresh: bool,
    pub last_error: Option<String>,
    pub catalog: G,
    pub installer: I,
    pub toggler: T,
    pub profile_store: Box<dyn ProfileStore>,
    pub profile_state: ProfileState,
    pub scan_findings: Vec<ScanFinding>,
    pub creator: Box<dyn SkillCreator>,
    pub writer: Box<dyn SkillWriter>,
    pub create_wizard_state: CreateWizardState,
    pub editor_state: Option<EditorState>,
    pub budget: ContextBudget,
    pub settings_store: Box<dyn SettingsStore>,
    pub settings: Option<ProjectSettings>,
    pub settings_state: SettingsState,
    #[allow(dead_code)]
    pub config_store: Box<dyn ConfigStore>,
    pub config: TuiConfig,
    pub config_state: ConfigState,
    /// Resolved key bindings, customizable via `config.keymap`.
    pub key_bindings: KeyBindings,
    pub external_scanner: Box<dyn ExternalScanner>,
    #[allow(dead_code)]
    pub signature_verifier: Box<dyn SignatureVerifier>,
    pub ssh_connector: Box<dyn SshConnector>,
    pub ssh_state: SshState,
    pub bundle_store: Box<dyn BundleStore>,
    pub bundle_state: BundleState,
    pub sync_store: Box<dyn SkillSync>,
    pub sync_state: SyncState,
    pub should_quit: bool,
    pub import_chain_result: Option<ImportChainResult>,
    pub profile_export_message: Option<String>,
    /// When `Some`, the first `Ctrl-C` was received and a second within 3s quits.
    pub quit_armed_at: Option<Instant>,
    /// Whether the floating command palette is open.
    pub command_palette_open: bool,
    /// Selected index in the command palette.
    pub palette_index: usize,
    /// Commands available in the palette (rebuilt when opened).
    pub palette_commands: Vec<PaletteCommand>,
}

#[allow(clippy::too_many_arguments)]
impl<G: AnyCatalogGateway, I: SkillInstaller, T: SkillToggler> App<G, I, T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        all_skills: Vec<Skill>,
        catalog: G,
        installer: I,
        toggler: T,
        profile_store: Box<dyn ProfileStore>,
        creator: Box<dyn SkillCreator>,
        writer: Box<dyn SkillWriter>,
        settings_store: Box<dyn SettingsStore>,
        external_scanner: Box<dyn ExternalScanner>,
        signature_verifier: Box<dyn SignatureVerifier>,
        ssh_connector: Box<dyn SshConnector>,
        bundle_store: Box<dyn BundleStore>,
        sync_store: Box<dyn SkillSync>,
        config_store: Box<dyn ConfigStore>,
        config: TuiConfig,
    ) -> Self {
        let profiles = profile_store.list().unwrap_or_default();
        let budget = calculate_budget(&all_skills);
        let key_bindings = KeyBindings::from_config(&config.keymap);
        Self {
            all_skills,
            budget,
            view: View::List,
            view_before_confirm: View::List,
            list_state: ListUiState::new(),
            detail_scroll: 0,
            diff_scroll: 0,
            search_state: SearchState::default(),
            install_wizard_state: InstallWizardState::default(),
            pending_action: None,
            needs_refresh: false,
            last_error: None,
            catalog,
            installer,
            toggler,
            profile_store,
            profile_state: ProfileState {
                profiles,
                selected_index: 0,
                new_name_input: String::new(),
                creating: false,
            },
            scan_findings: vec![],
            creator,
            writer,
            create_wizard_state: CreateWizardState::default(),
            editor_state: None,
            settings_store,
            settings: None,
            config_store,
            config,
            config_state: super::ui::settings_panel::ConfigState::default(),
            key_bindings,
            settings_state: SettingsState::default(),
            external_scanner,
            signature_verifier,
            ssh_connector,
            ssh_state: SshState::default(),
            bundle_store,
            bundle_state: BundleState::default(),
            sync_store,
            sync_state: SyncState::default(),
            should_quit: false,
            import_chain_result: None,
            profile_export_message: None,
            quit_armed_at: None,
            command_palette_open: false,
            palette_index: 0,
            palette_commands: vec![],
        }
    }

    pub fn visible_skills(&self) -> Vec<&Skill> {
        self.all_skills
            .iter()
            .filter(|s| match self.list_state.scope_filter {
                ScopeFilter::All => true,
                ScopeFilter::Global => s.scope == Scope::Global,
                ScopeFilter::Project => s.scope == Scope::Project,
            })
            .filter(|s| match &self.list_state.agent_filter {
                None => true,
                Some(agent) => s.agents.contains(agent),
            })
            .filter(|s| match &self.list_state.tag_filter {
                None => true,
                Some(tag) => s.tags.contains(tag),
            })
            .collect()
    }

    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .all_skills
            .iter()
            .flat_map(|s| s.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    pub fn cycle_tag_filter(&mut self) {
        let tags = self.all_tags();
        self.list_state.tag_filter = match &self.list_state.tag_filter {
            None => tags.into_iter().next(),
            Some(current) => {
                let idx = tags.iter().position(|t| t == current);
                match idx {
                    Some(i) if i + 1 < tags.len() => Some(tags[i + 1].clone()),
                    _ => None,
                }
            }
        };
        self.list_state.selected_index = 0;
    }

    pub fn selected_skill(&self) -> Option<&Skill> {
        let visible = self.visible_skills();
        visible.get(self.list_state.selected_index).copied()
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        self.disarm_quit_if_expired();
        match event {
            AppEvent::Key(key) if self.key_bindings.matches(&key, Action::Quit) => {
                self.request_quit();
            }
            AppEvent::Key(key) if self.key_bindings.matches(&key, Action::CommandPalette) => {
                self.toggle_command_palette();
            }
            AppEvent::Key(key) if self.command_palette_open => {
                self.handle_palette_key(key);
            }
            AppEvent::Key(key) => match self.view {
                View::List => self.handle_list_key(key),
                View::Detail => self.handle_detail_key(key),
                View::Search => self.handle_search_key(key),
                View::Help => self.handle_help_key(key),
                View::Confirm => self.handle_confirm_key(key),
                View::InstallWizard => self.handle_install_wizard_key(key),
                View::ScanReport => self.handle_scan_report_key(key),
                View::Profiles => self.handle_profiles_key(key),
                View::CreateWizard => self.handle_create_wizard_key(key),
                View::Editor => self.handle_editor_key(key),
                View::Audit => self.handle_audit_key(key),
                View::Budget => self.handle_audit_key(key),
                View::Settings => self.handle_settings_key(key),
                View::ImportChain => self.handle_import_chain_key(key),
                View::SshRemote => self.handle_ssh_key(key),
                View::Bundles => self.handle_bundles_key(key),
                View::Sync => self.handle_sync_key(key),
                View::Diff => self.handle_diff_key(key),
            },
            AppEvent::Resize => {}
        }
    }

    /// Clears the armed-quit timer once the 3s window has elapsed.
    fn disarm_quit_if_expired(&mut self) {
        if self
            .quit_armed_at
            .is_some_and(|at| at.elapsed() >= Duration::from_secs(3))
        {
            self.quit_armed_at = None;
        }
    }

    /// First `Ctrl-C` arms a 3s quit window; a second `Ctrl-C` within it quits.
    fn request_quit(&mut self) {
        match self.quit_armed_at {
            Some(at) if at.elapsed() < Duration::from_secs(3) => {
                self.should_quit = true;
            }
            _ => {
                self.quit_armed_at = Some(Instant::now());
            }
        }
    }

    /// True while the "press Ctrl-C again to quit" warning should show.
    pub fn quit_warning_active(&self) -> bool {
        self.quit_armed_at
            .map(|at| at.elapsed() < Duration::from_secs(3))
            .unwrap_or(false)
    }

    /// Opens (or closes) the floating command palette.
    fn toggle_command_palette(&mut self) {
        if self.command_palette_open {
            self.command_palette_open = false;
        } else {
            self.palette_index = 0;
            self.palette_commands = self.build_palette_commands();
            self.command_palette_open = true;
        }
    }

    /// Builds the palette command list for the current context.
    fn build_palette_commands(&self) -> Vec<PaletteCommand> {
        let mut cmds = vec![
            PaletteCommand::Search,
            PaletteCommand::Create,
            PaletteCommand::Audit,
            PaletteCommand::Budget,
            PaletteCommand::Profiles,
            PaletteCommand::Bundles,
            PaletteCommand::Sync,
            PaletteCommand::Settings,
            PaletteCommand::Help,
        ];
        if self.selected_skill().is_some() {
            cmds.push(PaletteCommand::OpenDetail);
            cmds.push(PaletteCommand::Edit);
            cmds.push(PaletteCommand::Disable);
            cmds.push(PaletteCommand::Remove);
            cmds.push(PaletteCommand::Update);
            if self
                .selected_skill()
                .is_some_and(|s| matches!(s.drift_state, DriftState::UpdateAvailable { .. }))
            {
                cmds.push(PaletteCommand::Diff);
            }
        }
        cmds
    }

    /// Handles keys while the command palette is open.
    fn handle_palette_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.command_palette_open = false;
            }
            KeyCode::Up | KeyCode::Char('k')
                if key.modifiers == KeyModifiers::NONE && !self.palette_commands.is_empty() =>
            {
                self.palette_index = self.palette_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j')
                if key.modifiers == KeyModifiers::NONE && !self.palette_commands.is_empty() =>
            {
                let max = self.palette_commands.len() - 1;
                if self.palette_index < max {
                    self.palette_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(cmd) = self.palette_commands.get(self.palette_index).copied() {
                    self.execute_palette_command(cmd);
                }
            }
            _ => {}
        }
    }

    /// Executes the selected palette command and closes the palette.
    fn execute_palette_command(&mut self, cmd: PaletteCommand) {
        self.command_palette_open = false;
        match cmd {
            PaletteCommand::Search => {
                self.search_state = SearchState::default();
                self.view = View::Search;
            }
            PaletteCommand::Create => {
                self.create_wizard_state = CreateWizardState::default();
                self.view = View::CreateWizard;
            }
            PaletteCommand::Audit => {
                self.view = View::Audit;
            }
            PaletteCommand::Budget => {
                self.view = View::Budget;
            }
            PaletteCommand::Profiles => {
                self.profile_state.profiles = self.profile_store.list().unwrap_or_default();
                self.profile_state.selected_index = 0;
                self.profile_state.creating = false;
                self.profile_state.new_name_input = String::new();
                self.view = View::Profiles;
            }
            PaletteCommand::Bundles => {
                self.bundle_state = BundleState {
                    bundles: self.bundle_store.list().unwrap_or_default(),
                    ..BundleState::default()
                };
                self.view = View::Bundles;
            }
            PaletteCommand::Sync => {
                self.refresh_sync_state();
                self.view = View::Sync;
            }
            PaletteCommand::Settings => {
                self.view = View::Settings;
            }
            PaletteCommand::Help => {
                self.view = View::Help;
            }
            PaletteCommand::OpenDetail => {
                if self.selected_skill().is_some() {
                    self.detail_scroll = 0;
                    self.view = View::Detail;
                }
            }
            PaletteCommand::Edit => {
                if let Some(skill) = self.selected_skill() {
                    let name_input = skill.name.clone();
                    let agents_input = skill.agents.join(", ");
                    let tags_input = skill.tags.join(", ");
                    let skill = skill.clone();
                    let warnings = ai_skill_core::lint_content(
                        skill.manifest_content.as_deref().unwrap_or(""),
                        &self.all_skills,
                        &name_input,
                        Some(&skill.name),
                    );
                    self.editor_state = Some(EditorState {
                        skill,
                        field: EditField::default(),
                        name_input,
                        agents_input,
                        tags_input,
                        warnings,
                    });
                    self.view = View::Editor;
                }
            }
            PaletteCommand::Disable => {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Disable { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            PaletteCommand::Remove => {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Remove { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            PaletteCommand::Update => {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Update { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            PaletteCommand::Diff => {
                if self
                    .selected_skill()
                    .is_some_and(|s| matches!(s.drift_state, DriftState::UpdateAvailable { .. }))
                {
                    self.diff_scroll = 0;
                    self.view = View::Diff;
                }
            }
        }
    }

    pub fn preview_for_action(&self, action: &AppAction) -> String {
        match action {
            AppAction::Install {
                name,
                agents,
                scope,
            } => self.installer.preview_install(name, agents, scope.clone()),
            AppAction::Remove { path } => self.installer.preview_remove(path),
            AppAction::Update { path } => self.installer.preview_update(path),
            AppAction::Enable { path } => self.toggler.preview_enable(path),
            AppAction::Disable { path } => self.toggler.preview_disable(path),
            AppAction::Adopt { path } => format!("adopt {}", path.display()),
            AppAction::ToggleNameOnly { path } => {
                let skill = self.all_skills.iter().find(|s| s.path == *path);
                match skill.map(|s| s.mode) {
                    Some(SkillMode::NameOnly) => self.toggler.preview_expand(path),
                    _ => self.toggler.preview_collapse(path),
                }
            }
            AppAction::ActivateProfile { name, ops } => {
                let installs = ops
                    .iter()
                    .filter(|o| matches!(o, ProfileOp::Install { .. }))
                    .count();
                let removes = ops
                    .iter()
                    .filter(|o| matches!(o, ProfileOp::Remove { .. }))
                    .count();
                format!("activate profile \"{name}\": install {installs}, remove {removes}")
            }
        }
    }

    fn execute_pending_action(&mut self) {
        let action = match self.pending_action.take() {
            Some(a) => a,
            None => return,
        };
        match &action {
            AppAction::ActivateProfile { ops, .. } => {
                for op in ops.clone() {
                    let result = match op {
                        ProfileOp::Install { name } => {
                            self.installer.install(&name, &[], Scope::Global)
                        }
                        ProfileOp::Remove { name } => {
                            let path = PathBuf::from(&name);
                            self.installer.remove(&path)
                        }
                    };
                    if let Err(e) = result {
                        self.last_error = Some(e.to_string());
                        return;
                    }
                }
                self.needs_refresh = true;
                self.last_error = None;
            }
            _ => {
                let result = match &action {
                    AppAction::Install {
                        name,
                        agents,
                        scope,
                    } => self.installer.install(name, agents, scope.clone()),
                    AppAction::Remove { path } => self.installer.remove(path),
                    AppAction::Update { path } => self.installer.update(path),
                    AppAction::Enable { path } => self.toggler.enable(path),
                    AppAction::Disable { path } => self.toggler.disable(path),
                    AppAction::ToggleNameOnly { path } => {
                        let skill = self.all_skills.iter().find(|s| s.path == *path);
                        match skill.map(|s| s.mode) {
                            Some(SkillMode::NameOnly) => self.toggler.expand(path),
                            _ => self.toggler.collapse(path),
                        }
                    }
                    AppAction::Adopt { path } => self.toggler.adopt(path),
                    AppAction::ActivateProfile { .. } => unreachable!(),
                };
                match result {
                    Ok(()) => {
                        self.needs_refresh = true;
                        self.last_error = None;
                    }
                    Err(e) => {
                        self.last_error = Some(e.to_string());
                    }
                }
            }
        }
    }

    fn handle_list_key(&mut self, key: crossterm::event::KeyEvent) {
        let visible_len = self.visible_skills().len();
        match key.code {
            KeyCode::Esc => {
                // No parent view to return to from the list; do nothing.
            }
            KeyCode::Down | KeyCode::Char('j')
                if key.modifiers == KeyModifiers::NONE
                    && visible_len > 0
                    && self.list_state.selected_index + 1 < visible_len =>
            {
                self.list_state.selected_index += 1;
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.list_state.selected_index = self.list_state.selected_index.saturating_sub(1);
            }
            KeyCode::Tab => {
                self.list_state.scope_filter = self.list_state.scope_filter.next();
                self.list_state.selected_index = 0;
            }
            KeyCode::Enter if self.selected_skill().is_some() => {
                self.detail_scroll = 0;
                self.view = View::Detail;
            }
            KeyCode::Char('t') if key.modifiers == KeyModifiers::NONE => {
                self.cycle_tag_filter();
            }
            KeyCode::Char(' ') if key.modifiers == KeyModifiers::NONE => {
                let idx = self.list_state.selected_index;
                if let Some(pos) = self
                    .list_state
                    .selected_items
                    .iter()
                    .position(|&i| i == idx)
                {
                    self.list_state.selected_items.remove(pos);
                } else {
                    self.list_state.selected_items.push(idx);
                }
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Disable)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Disable { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Enable)
                    && key.modifiers == KeyModifiers::NONE
                    && self
                        .selected_skill()
                        .map(|s| s.mode == SkillMode::Disabled)
                        .unwrap_or(false) =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Enable { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Remove)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Remove { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Update)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Update { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::ToggleNameOnly)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::ToggleNameOnly { path });
                    self.execute_pending_action();
                }
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Adopt)
                    && key.modifiers == KeyModifiers::NONE
                    && self.selected_skill().map(|s| !s.managed).unwrap_or(false) =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Adopt { path });
                    self.execute_pending_action();
                }
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Search)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                self.search_state = SearchState::default();
                self.view = View::Search;
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Profiles)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                let profiles = self.profile_store.list().unwrap_or_default();
                self.profile_state.profiles = profiles;
                self.profile_state.selected_index = 0;
                self.profile_state.creating = false;
                self.profile_state.new_name_input = String::new();
                self.view = View::Profiles;
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Help)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                self.view = View::Help;
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Bundles)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                let bundles = self.bundle_store.list().unwrap_or_default();
                self.bundle_state = BundleState {
                    bundles,
                    ..BundleState::default()
                };
                self.view = View::Bundles;
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Create)
                    && key.modifiers == KeyModifiers::NONE =>
            {
                self.create_wizard_state = CreateWizardState::default();
                self.view = View::CreateWizard;
            }
            KeyCode::Char(_)
                if self.key_bindings.matches(&key, Action::Editor)
                    && key.modifiers == KeyModifiers::NONE
                    && self
                        .selected_skill()
                        .map(|s| s.mode != SkillMode::Disabled)
                        .unwrap_or(false) =>
            {
                if let Some(skill) = self.selected_skill() {
                    let name_input = skill.name.clone();
                    let agents_input = skill.agents.join(", ");
                    let tags_input = skill.tags.join(", ");
                    let skill = skill.clone();
                    let warnings = ai_skill_core::lint_content(
                        skill.manifest_content.as_deref().unwrap_or(""),
                        &self.all_skills,
                        &name_input,
                        Some(&skill.name),
                    );
                    self.editor_state = Some(EditorState {
                        skill,
                        field: EditField::default(),
                        name_input,
                        agents_input,
                        tags_input,
                        warnings,
                    });
                    self.view = View::Editor;
                }
            }
            KeyCode::Char(_) if self.key_bindings.matches(&key, Action::Budget) => {
                self.view = View::Budget;
            }
            KeyCode::Char(_) if self.key_bindings.matches(&key, Action::Audit) => {
                self.view = View::Audit;
            }
            KeyCode::Char(_) if self.key_bindings.matches(&key, Action::Sync) => {
                self.refresh_sync_state();
                self.view = View::Sync;
            }
            KeyCode::Char(_) if self.key_bindings.matches(&key, Action::SshRemote) => {
                self.ssh_state = SshState::default();
                self.view = View::SshRemote;
            }
            KeyCode::F(1) => self.activate_phase_preset(Phase::Init),
            KeyCode::F(2) => self.activate_phase_preset(Phase::Dev),
            KeyCode::F(3) => self.activate_phase_preset(Phase::Test),
            KeyCode::F(4) => self.activate_phase_preset(Phase::Release),
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => self.view = View::List,
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                self.detail_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            KeyCode::Char('o') if key.modifiers == KeyModifiers::NONE => {
                if let Some(skill) = self.selected_skill().cloned() {
                    self.toggle_skill_auto_trigger(&skill.name);
                }
            }
            KeyCode::Char('i') if key.modifiers == KeyModifiers::NONE => {
                if let Some(skill) = self.selected_skill() {
                    use ai_skill_adapters::trace_import_chain;
                    self.import_chain_result = trace_import_chain(&skill.path).ok();
                    self.view = View::ImportChain;
                }
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::NONE => {
                if let Some(skill) = self.selected_skill()
                    && matches!(
                        skill.drift_state,
                        ai_skill_core::DriftState::UpdateAvailable { .. }
                    )
                {
                    self.diff_scroll = 0;
                    self.view = View::Diff;
                }
            }
            _ => {}
        }
    }

    fn handle_diff_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.view = View::Detail;
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                self.diff_scroll += 1;
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.diff_scroll = self.diff_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn toggle_skill_auto_trigger(&mut self, skill_name: &str) {
        if self.settings.is_none() {
            self.settings = self.settings_store.read().ok();
        }
        if let Some(ref mut settings) = self.settings {
            let existing = settings
                .skill_overrides
                .iter_mut()
                .find(|o| o.skill_name == skill_name);
            match existing {
                Some(override_) => {
                    override_.auto_trigger = !override_.auto_trigger;
                }
                None => {
                    settings.skill_overrides.push(ai_skill_core::SkillOverride {
                        skill_name: skill_name.to_string(),
                        auto_trigger: false,
                    });
                }
            }
            let _ = self.settings_store.write(settings);
        }
    }

    fn handle_search_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_state = SearchState::default();
                self.view = View::List;
            }
            KeyCode::Enter if !self.search_state.results.is_empty() => {
                let entry = self.search_state.results[self.search_state.selected_index].clone();
                self.install_wizard_state = InstallWizardState {
                    entry: Some(entry),
                    scope: Scope::Global,
                    available_agents: vec!["claude".to_string()],
                    selected_agents: vec!["claude".to_string()],
                };
                self.view = View::InstallWizard;
            }
            KeyCode::Backspace => {
                self.search_state.query.pop();
                self.do_search();
            }
            KeyCode::Down | KeyCode::Char('j')
                if key.modifiers == KeyModifiers::NONE
                    && self.search_state.selected_index + 1 < self.search_state.results.len() =>
            {
                self.search_state.selected_index += 1;
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.search_state.selected_index =
                    self.search_state.selected_index.saturating_sub(1);
            }
            KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                self.search_state.query.push(c);
                self.do_search();
            }
            _ => {}
        }
    }

    fn handle_help_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Esc {
            self.view = View::List;
        }
    }

    fn handle_confirm_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.execute_pending_action();
                self.view = self.view_before_confirm;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.pending_action = None;
                self.view = self.view_before_confirm;
            }
            _ => {}
        }
    }

    fn handle_install_wizard_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.view = View::Search;
            }
            KeyCode::Tab => {
                self.install_wizard_state.scope = match self.install_wizard_state.scope {
                    Scope::Global => Scope::Project,
                    Scope::Project => Scope::Global,
                };
            }
            KeyCode::Char(' ') if key.modifiers == KeyModifiers::NONE => {
                let agents = &self.install_wizard_state.available_agents;
                let selected = &mut self.install_wizard_state.selected_agents;
                if let Some(agent) = agents.first().cloned() {
                    if let Some(pos) = selected.iter().position(|a| *a == agent) {
                        selected.remove(pos);
                    } else {
                        selected.push(agent);
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(entry) = self.install_wizard_state.entry.clone() {
                    let scope = self.install_wizard_state.scope.clone();
                    let agents = self.install_wizard_state.selected_agents.clone();
                    let mut findings = scan_skill(&entry.description);
                    findings.extend(cross_reference(&entry, &self.catalog));
                    if let Ok(ext_findings) = self.external_scanner.scan(&entry.name) {
                        findings.extend(ext_findings.into_iter().map(Into::into));
                    }
                    let action = AppAction::Install {
                        name: entry.name,
                        agents,
                        scope,
                    };
                    self.view_before_confirm = View::List;
                    if findings.is_empty() {
                        self.pending_action = Some(action);
                        self.view = View::Confirm;
                    } else {
                        self.scan_findings = findings;
                        self.pending_action = Some(action);
                        self.view = View::ScanReport;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_scan_report_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.scan_findings.clear();
                self.pending_action = None;
                self.view = View::InstallWizard;
            }
            KeyCode::Enter => {
                self.scan_findings.clear();
                self.view = View::Confirm;
            }
            _ => {}
        }
    }

    fn handle_profiles_key(&mut self, key: crossterm::event::KeyEvent) {
        let len = self.profile_state.profiles.len();
        match key.code {
            KeyCode::Esc => {
                self.profile_state.creating = false;
                self.profile_export_message = None;
                self.view = View::List;
            }
            KeyCode::Down | KeyCode::Char('j')
                if key.modifiers == KeyModifiers::NONE
                    && len > 0
                    && self.profile_state.selected_index + 1 < len =>
            {
                self.profile_state.selected_index += 1;
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.profile_state.selected_index =
                    self.profile_state.selected_index.saturating_sub(1);
            }
            KeyCode::Char('a') if key.modifiers == KeyModifiers::NONE && len > 0 => {
                let profile =
                    self.profile_state.profiles[self.profile_state.selected_index].clone();
                let ops = ai_skill_core::diff_profile(&self.all_skills, &profile);
                let name = profile.name.clone();
                self.pending_action = Some(AppAction::ActivateProfile { name, ops });
                self.view_before_confirm = View::Profiles;
                self.view = View::Confirm;
            }
            KeyCode::Char('f') if key.modifiers == KeyModifiers::NONE => {
                self.profile_state.creating = true;
                self.profile_state.new_name_input = String::new();
            }
            KeyCode::Enter if self.profile_state.creating => {
                let name = self.profile_state.new_name_input.trim().to_string();
                if !name.is_empty() {
                    let skill_names = self
                        .all_skills
                        .iter()
                        .filter(|s| s.mode.is_enabled())
                        .map(|s| s.name.clone())
                        .collect();
                    let profile = Profile {
                        name,
                        skill_names,
                        phase: None,
                    };
                    let _ = self.profile_store.save(&profile);
                    let profiles = self.profile_store.list().unwrap_or_default();
                    self.profile_state.profiles = profiles;
                }
                self.profile_state.creating = false;
                self.profile_state.new_name_input = String::new();
            }
            KeyCode::Backspace if self.profile_state.creating => {
                self.profile_state.new_name_input.pop();
            }
            KeyCode::Char(c)
                if key.modifiers == KeyModifiers::NONE && self.profile_state.creating =>
            {
                self.profile_state.new_name_input.push(c);
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::NONE && len > 0 => {
                let name = self.profile_state.profiles[self.profile_state.selected_index]
                    .name
                    .clone();
                let _ = self.profile_store.delete(&name);
                let profiles = self.profile_store.list().unwrap_or_default();
                self.profile_state.profiles = profiles;
                self.profile_state.selected_index =
                    self.profile_state.selected_index.saturating_sub(1);
            }
            KeyCode::Char('e') if key.modifiers == KeyModifiers::NONE && len > 0 => {
                let name = self.profile_state.profiles[self.profile_state.selected_index]
                    .name
                    .clone();
                if let Ok(cwd) = std::env::current_dir() {
                    let dest = cwd.join(format!("{name}.skill-profile.yaml"));
                    match self.profile_store.export(&name, &dest) {
                        Ok(()) => {
                            self.profile_export_message =
                                Some(format!("Exported to {}", dest.display()));
                        }
                        Err(e) => {
                            self.last_error = Some(e.to_string());
                        }
                    }
                } else {
                    self.last_error = Some("Cannot determine current directory".into());
                }
            }
            _ => {}
        }
    }

    fn handle_create_wizard_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.create_wizard_state = CreateWizardState::default();
                self.view = View::List;
            }
            KeyCode::Tab => {
                self.create_wizard_state.step = match self.create_wizard_state.step {
                    CreateStep::Name => CreateStep::Agents,
                    CreateStep::Agents => CreateStep::Tags,
                    CreateStep::Tags => CreateStep::Preview,
                    CreateStep::Preview => CreateStep::Name,
                };
            }
            KeyCode::Enter if self.create_wizard_state.step == CreateStep::Preview => {
                self.recalc_wizard_errors();
                if !self.create_wizard_state.errors.is_empty() {
                    return;
                }
                let name = self.create_wizard_state.name.trim().to_string();
                if !name.is_empty() {
                    let agents: Vec<String> = self
                        .create_wizard_state
                        .agents_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let tags: Vec<String> = self
                        .create_wizard_state
                        .tags_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    match self.creator.create(&name, &agents, &tags) {
                        Ok(_) => {
                            self.needs_refresh = true;
                            self.create_wizard_state = CreateWizardState::default();
                            self.view = View::List;
                        }
                        Err(e) => {
                            self.last_error = Some(e.to_string());
                        }
                    }
                }
            }
            KeyCode::Backspace => match self.create_wizard_state.step {
                CreateStep::Name => {
                    self.create_wizard_state.name.pop();
                    self.recalc_wizard_errors();
                }
                CreateStep::Agents => {
                    self.create_wizard_state.agents_input.pop();
                    self.recalc_wizard_errors();
                }
                CreateStep::Tags => {
                    self.create_wizard_state.tags_input.pop();
                }
                CreateStep::Preview => {}
            },
            KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                match self.create_wizard_state.step {
                    CreateStep::Name => {
                        self.create_wizard_state.name.push(c);
                        self.recalc_wizard_errors();
                    }
                    CreateStep::Agents => {
                        self.create_wizard_state.agents_input.push(c);
                        self.recalc_wizard_errors();
                    }
                    CreateStep::Tags => self.create_wizard_state.tags_input.push(c),
                    CreateStep::Preview => {}
                }
            }
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.editor_state = None;
                self.view = View::List;
            }
            KeyCode::Tab => {
                if let Some(state) = &mut self.editor_state {
                    state.field = match state.field {
                        EditField::Name => EditField::Agents,
                        EditField::Agents => EditField::Tags,
                        EditField::Tags => EditField::Name,
                    };
                }
            }
            KeyCode::Enter => {
                if let Some(state) = self.editor_state.take() {
                    let agents: Vec<String> = state
                        .agents_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let tags: Vec<String> = state
                        .tags_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let original = state.skill.manifest_content.as_deref().unwrap_or("");
                    let new_content =
                        ai_skill_core::apply_edit(original, &state.name_input, &agents, &tags);
                    let manifest_path = state.skill.path.join("SKILL.md");
                    match self.writer.write(&manifest_path, &new_content) {
                        Ok(()) => {
                            self.needs_refresh = true;
                            self.view = View::List;
                        }
                        Err(e) => {
                            self.last_error = Some(e.to_string());
                            self.view = View::List;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(state) = &mut self.editor_state {
                    match state.field {
                        EditField::Name => {
                            state.name_input.pop();
                        }
                        EditField::Agents => {
                            state.agents_input.pop();
                        }
                        EditField::Tags => {
                            state.tags_input.pop();
                        }
                    }
                }
                self.recalc_editor_warnings();
            }
            KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                if let Some(state) = &mut self.editor_state {
                    match state.field {
                        EditField::Name => {
                            state.name_input.push(c);
                        }
                        EditField::Agents => state.agents_input.push(c),
                        EditField::Tags => state.tags_input.push(c),
                    }
                }
                self.recalc_editor_warnings();
            }
            _ => {}
        }
    }

    fn handle_audit_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Esc {
            self.view = View::List;
        }
    }

    fn handle_import_chain_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Esc {
            self.import_chain_result = None;
            self.view = View::Detail;
        }
    }

    fn handle_bundles_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.bundle_state.result_message = None;
                self.view = View::List;
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                let len = self.bundle_state.bundles.len();
                if len > 0 && self.bundle_state.selected_index + 1 < len {
                    self.bundle_state.selected_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.bundle_state.selected_index =
                    self.bundle_state.selected_index.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(bundle) = self
                    .bundle_state
                    .bundles
                    .get(self.bundle_state.selected_index)
                {
                    let mut installed = 0usize;
                    let mut errors = Vec::new();
                    for skill_name in &bundle.skills {
                        match self.installer.install(skill_name, &[], Scope::Global) {
                            Ok(()) => installed += 1,
                            Err(e) => errors.push(format!("{skill_name}: {e}")),
                        }
                    }
                    if errors.is_empty() {
                        self.bundle_state.result_message = Some(format!(
                            "Installed {installed} skill(s) from \"{}\"",
                            bundle.name
                        ));
                    } else {
                        self.bundle_state.result_message = Some(format!(
                            "Installed {installed}/{} skill(s). Errors: {}",
                            bundle.skills.len(),
                            errors.join("; "),
                        ));
                    }
                    self.needs_refresh = true;
                }
            }
            _ => {}
        }
    }

    fn refresh_sync_state(&mut self) {
        let status = self.sync_store.status().ok();
        let snapshots = self.sync_store.list_snapshots().ok().unwrap_or_default();
        self.sync_state = SyncState {
            status,
            snapshots,
            ..SyncState::default()
        };
    }

    fn handle_sync_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if self.sync_state.creating_snapshot {
                    self.sync_state.creating_snapshot = false;
                    self.sync_state.snapshot_message.clear();
                } else if self.sync_state.configuring_remote {
                    self.sync_state.configuring_remote = false;
                    self.sync_state.remote_input.clear();
                } else {
                    self.view = View::List;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                let len = self.sync_state.snapshots.len();
                if len > 0 && self.sync_state.selected_index + 1 < len {
                    self.sync_state.selected_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.sync_state.selected_index = self.sync_state.selected_index.saturating_sub(1);
            }
            KeyCode::Enter if self.sync_state.creating_snapshot => {
                let msg = self.sync_state.snapshot_message.clone();
                match self.sync_store.snapshot(&msg) {
                    Ok(hash) => {
                        self.sync_state.message = Some(format!("Snapshot created: {hash}"));
                        self.sync_state.snapshot_message.clear();
                        self.sync_state.creating_snapshot = false;
                        self.refresh_sync_state();
                    }
                    Err(e) => {
                        self.sync_state.message = Some(format!("Error: {e}"));
                    }
                }
            }
            KeyCode::Enter if self.sync_state.configuring_remote => {
                let input = self.sync_state.remote_input.clone();
                let parts: Vec<&str> = input.splitn(2, ' ').collect();
                match parts.as_slice() {
                    [name, url] => match self.sync_store.add_remote(name, url) {
                        Ok(()) => {
                            self.sync_state.message = Some(format!("Remote '{name}' configured"));
                            self.sync_state.remote_input.clear();
                            self.sync_state.configuring_remote = false;
                        }
                        Err(e) => {
                            self.sync_state.message = Some(format!("Error: {e}"));
                        }
                    },
                    _ => {
                        self.sync_state.message = Some("Usage: <name> <url>".into());
                    }
                }
            }
            KeyCode::Enter => {
                // No snapshot selected: init or snapshot creation, depending on status.
                match &self.sync_state.status {
                    Some(SyncStatus::Uninitialized) => match self.sync_store.init() {
                        Ok(()) => {
                            self.sync_state.message = Some("Repository initialized".into());
                            self.refresh_sync_state();
                        }
                        Err(e) => {
                            self.sync_state.message = Some(format!("Error: {e}"));
                        }
                    },
                    _ => {
                        self.sync_state.creating_snapshot = true;
                    }
                }
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::NONE => {
                // Restore selected snapshot.
                if let Some(snapshot) = self
                    .sync_state
                    .snapshots
                    .get(self.sync_state.selected_index)
                {
                    match self.sync_store.restore(&snapshot.id) {
                        Ok(()) => {
                            self.sync_state.message = Some(format!(
                                "Restored to {}",
                                &snapshot.id[..7.min(snapshot.id.len())]
                            ));
                            self.needs_refresh = true;
                            self.refresh_sync_state();
                        }
                        Err(e) => {
                            self.sync_state.message = Some(format!("Error: {e}"));
                        }
                    }
                }
            }
            KeyCode::Char('R') => {
                self.sync_state.configuring_remote = true;
                self.sync_state.remote_input.clear();
            }
            KeyCode::Char('p') if key.modifiers == KeyModifiers::NONE => {
                // Push to remote.
                let branch = if self.sync_state.branch.is_empty() {
                    "main"
                } else {
                    &self.sync_state.branch
                };
                match self.sync_store.push("origin", branch) {
                    Ok(()) => {
                        self.sync_state.message = Some("Pushed to origin".into());
                        self.refresh_sync_state();
                    }
                    Err(e) => {
                        self.sync_state.message = Some(format!("Push failed: {e}"));
                    }
                }
            }
            KeyCode::Char('P') => {
                // Pull from remote.
                let branch = if self.sync_state.branch.is_empty() {
                    "main"
                } else {
                    &self.sync_state.branch
                };
                match self.sync_store.pull("origin", branch) {
                    Ok(()) => {
                        self.sync_state.message = Some("Pulled from origin".into());
                        self.needs_refresh = true;
                        self.refresh_sync_state();
                    }
                    Err(e) => {
                        self.sync_state.message = Some(format!("Pull failed: {e}"));
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_ssh_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                let len = self.ssh_state.hosts.len();
                if len > 0 && self.ssh_state.selected_index + 1 < len {
                    self.ssh_state.selected_index += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.ssh_state.selected_index = self.ssh_state.selected_index.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(host) = self.ssh_state.hosts.get(self.ssh_state.selected_index) {
                    self.ssh_state.connection_status =
                        Some(self.ssh_connector.check_connection(host));
                    match self.ssh_connector.list_skills(host) {
                        Ok(skills) => {
                            self.ssh_state.skills = skills;
                            self.ssh_state.error = None;
                        }
                        Err(e) => {
                            self.ssh_state.skills.clear();
                            self.ssh_state.error = Some(e);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn activate_phase_preset(&mut self, phase: Phase) {
        let profiles = self.profile_store.list().unwrap_or_default();
        if let Some(profile) = profiles
            .into_iter()
            .find(|p| p.phase == Some(phase.clone()))
        {
            let ops = ai_skill_core::diff_profile(&self.all_skills, &profile);
            if !ops.is_empty() {
                let name = profile.name.clone();
                self.pending_action = Some(AppAction::ActivateProfile { name, ops });
                self.view_before_confirm = View::List;
                self.view = View::Confirm;
            }
        }
    }

    fn handle_settings_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                if self.settings_state.dirty
                    && let Some(ref settings) = self.settings
                {
                    let _ = self.settings_store.write(settings);
                }
                self.settings = None;
                self.view = View::List;
            }
            KeyCode::Char('t') if key.modifiers == KeyModifiers::NONE => {
                if let Some(ref mut settings) = self.settings {
                    settings.auto_trigger = !settings.auto_trigger;
                    self.settings_state.dirty = true;
                }
            }
            KeyCode::Char('j') | KeyCode::Down if key.modifiers == KeyModifiers::NONE => {
                if let Some(ref settings) = self.settings
                    && self.settings_state.selected_override_index + 1
                        < settings.skill_overrides.len()
                {
                    self.settings_state.selected_override_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up if key.modifiers == KeyModifiers::NONE => {
                self.settings_state.selected_override_index = self
                    .settings_state
                    .selected_override_index
                    .saturating_sub(1);
            }
            KeyCode::Char('o') if key.modifiers == KeyModifiers::NONE => {
                if let Some(ref mut settings) = self.settings {
                    let idx = self.settings_state.selected_override_index;
                    if idx < settings.skill_overrides.len() {
                        settings.skill_overrides[idx].auto_trigger =
                            !settings.skill_overrides[idx].auto_trigger;
                        self.settings_state.dirty = true;
                    }
                }
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::NONE => {
                if let Some(ref mut settings) = self.settings {
                    let idx = self.settings_state.selected_override_index;
                    if idx < settings.skill_overrides.len() {
                        settings.skill_overrides.remove(idx);
                        self.settings_state.selected_override_index = self
                            .settings_state
                            .selected_override_index
                            .saturating_sub(1);
                        self.settings_state.dirty = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn recalc_editor_warnings(&mut self) {
        let Some(state) = &mut self.editor_state else {
            return;
        };
        let body = state
            .skill
            .manifest_content
            .as_deref()
            .and_then(ai_skill_core::extract_body)
            .unwrap_or("");
        state.warnings = ai_skill_core::lint_description(
            body,
            &state.name_input,
            &self.all_skills,
            Some(&state.skill.name),
        );
    }

    fn recalc_wizard_errors(&mut self) {
        let name = self.create_wizard_state.name.trim().to_string();
        let agents: Vec<String> = self
            .create_wizard_state
            .agents_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.create_wizard_state.errors =
            ai_skill_core::validate_wizard_input(&name, &agents, &self.all_skills);
    }

    fn do_search(&mut self) {
        if self.search_state.query.is_empty() {
            self.search_state.results.clear();
            self.search_state.error = None;
            self.search_state.selected_index = 0;
            return;
        }
        match self.catalog.search(&self.search_state.query) {
            Ok(results) => {
                self.search_state.results = results;
                self.search_state.error = None;
                self.search_state.selected_index = 0;
            }
            Err(e) => {
                self.search_state.results.clear();
                self.search_state.error = Some(e.to_string());
                self.search_state.selected_index = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_skill_core::{CatalogEntry, ValidationState};
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};
    use std::{cell::RefCell, path::Path, path::PathBuf};

    struct FakeCatalog(Vec<CatalogEntry>);

    impl AnyCatalogGateway for FakeCatalog {
        fn search(&self, _kw: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
            Ok(self.0.clone())
        }
    }

    struct ErrorCatalog;

    impl AnyCatalogGateway for ErrorCatalog {
        fn search(&self, _kw: &str) -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>> {
            Err("npx not found".into())
        }
    }

    #[derive(Default)]
    struct FakeInstaller {
        calls: std::cell::RefCell<Vec<String>>,
    }

    impl SkillInstaller for FakeInstaller {
        fn install(
            &self,
            name: &str,
            _agents: &[String],
            _scope: Scope,
        ) -> Result<(), Box<dyn std::error::Error>> {
            self.calls.borrow_mut().push(format!("install:{name}"));
            Ok(())
        }
        fn remove(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("remove:{}", path.display()));
            Ok(())
        }
        fn update(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("update:{}", path.display()));
            Ok(())
        }
        fn preview_install(&self, name: &str, _a: &[String], _s: Scope) -> String {
            format!("npx skills add {name}")
        }
        fn preview_remove(&self, path: &std::path::Path) -> String {
            format!("npx skills remove {}", path.display())
        }
        fn preview_update(&self, path: &std::path::Path) -> String {
            format!("npx skills update {}", path.display())
        }
    }

    #[derive(Default)]
    struct FakeToggler {
        calls: std::cell::RefCell<Vec<String>>,
    }

    impl SkillToggler for FakeToggler {
        fn enable(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("enable:{}", path.display()));
            Ok(())
        }
        fn disable(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("disable:{}", path.display()));
            Ok(())
        }
        fn collapse(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("collapse:{}", path.display()));
            Ok(())
        }
        fn expand(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("expand:{}", path.display()));
            Ok(())
        }
        fn adopt(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("adopt:{}", path.display()));
            Ok(())
        }
        fn preview_enable(&self, path: &std::path::Path) -> String {
            format!("enable {}", path.display())
        }
        fn preview_disable(&self, path: &std::path::Path) -> String {
            format!("disable {}", path.display())
        }
        fn preview_collapse(&self, path: &std::path::Path) -> String {
            format!("collapse {}", path.display())
        }
        fn preview_expand(&self, path: &std::path::Path) -> String {
            format!("expand {}", path.display())
        }
    }

    fn key(code: KeyCode) -> AppEvent {
        AppEvent::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn ctrl(code: KeyCode) -> AppEvent {
        AppEvent::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn make_skill(name: &str, scope: Scope) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope,
            agents: vec!["claude".to_string()],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: Some(format!("# {name}\nBody.").to_string()),
            drift_state: ai_skill_core::DriftState::default(),
        }
    }

    #[derive(Default)]
    struct FakeProfileStore {
        profiles: RefCell<Vec<Profile>>,
    }

    impl ProfileStore for FakeProfileStore {
        fn list(&self) -> Result<Vec<Profile>, Box<dyn std::error::Error>> {
            Ok(self.profiles.borrow().clone())
        }
        fn save(&self, p: &Profile) -> Result<(), Box<dyn std::error::Error>> {
            let mut profiles = self.profiles.borrow_mut();
            profiles.retain(|x| x.name != p.name);
            profiles.push(p.clone());
            Ok(())
        }
        fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
            self.profiles.borrow_mut().retain(|p| p.name != name);
            Ok(())
        }
        fn export(&self, name: &str, _dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
            let profiles = self.profiles.borrow();
            if profiles.iter().any(|p| p.name == name) {
                Ok(())
            } else {
                Err(format!("profile '{name}' not found").into())
            }
        }
    }

    #[derive(Default)]
    struct FakeSettingsStore;

    impl SettingsStore for FakeSettingsStore {
        fn read(&self) -> Result<ProjectSettings, Box<dyn std::error::Error>> {
            Ok(ProjectSettings::default())
        }
        fn write(&self, _settings: &ProjectSettings) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeCreator {
        calls: RefCell<Vec<String>>,
    }

    impl SkillCreator for FakeCreator {
        fn create(
            &self,
            name: &str,
            _agents: &[String],
            _tags: &[String],
        ) -> Result<PathBuf, Box<dyn std::error::Error>> {
            self.calls.borrow_mut().push(name.to_string());
            Ok(PathBuf::from(format!("/tmp/{name}")))
        }
    }

    #[derive(Default)]
    struct FakeWriter {
        calls: RefCell<Vec<String>>,
    }

    impl SkillWriter for FakeWriter {
        fn write(
            &self,
            path: &std::path::Path,
            content: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            self.calls
                .borrow_mut()
                .push(format!("{}:{}", path.display(), content.len()));
            Ok(())
        }
    }

    type TestApp = App<FakeCatalog, FakeInstaller, FakeToggler>;

    struct FakeBundleStore;
    impl BundleStore for FakeBundleStore {
        fn list(&self) -> Result<Vec<Bundle>, Box<dyn std::error::Error>> {
            Ok(vec![Bundle {
                name: "test-bundle".into(),
                description: "Test".into(),
                skills: vec!["alpha".into()],
            }])
        }
    }

    struct FakeSkillSync;
    impl ai_skill_core::SkillSync for FakeSkillSync {
        fn is_initialized(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(false)
        }
        fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn snapshot(&self, _: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok("0000000".into())
        }
        fn list_snapshots(
            &self,
        ) -> Result<Vec<ai_skill_core::Snapshot>, Box<dyn std::error::Error>> {
            Ok(vec![])
        }
        fn restore(&self, _: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn status(&self) -> Result<ai_skill_core::SyncStatus, Box<dyn std::error::Error>> {
            Ok(ai_skill_core::SyncStatus::Uninitialized)
        }
        fn push(&self, _: &str, _: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn pull(&self, _: &str, _: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        fn add_remote(&self, _: &str, _: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    struct FakeConfigStore;
    impl ConfigStore for FakeConfigStore {
        fn read(&self) -> Result<TuiConfig, Box<dyn std::error::Error>> {
            Ok(TuiConfig::default())
        }

        fn write(&self, _: &TuiConfig) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
    }

    fn make_app(skills: Vec<Skill>) -> TestApp {
        App::new(
            skills,
            FakeCatalog(vec![]),
            FakeInstaller::default(),
            FakeToggler::default(),
            Box::new(FakeProfileStore::default()),
            Box::new(FakeCreator::default()),
            Box::new(FakeWriter::default()),
            Box::new(FakeSettingsStore),
            Box::new(ai_skill_core::NoopExternalScanner),
            Box::new(ai_skill_core::NoopSignatureVerifier),
            Box::new(ai_skill_core::NoopSshConnector),
            Box::new(FakeBundleStore),
            Box::new(FakeSkillSync),
            Box::new(FakeConfigStore),
            TuiConfig::default(),
        )
    }

    fn make_skills(n: usize) -> Vec<Skill> {
        (0..n)
            .map(|i| make_skill(&format!("skill-{i}"), Scope::Global))
            .collect()
    }

    // ── initial state ─────────────────────────────────────────────────────────

    #[test]
    fn initial_state_is_list_view_index_zero_not_quitting() {
        let app = make_app(make_skills(3));
        assert_eq!(app.view, View::List);
        assert_eq!(app.list_state.selected_index, 0);
        assert!(!app.should_quit);
    }

    // ── quit ──────────────────────────────────────────────────────────────────

    #[test]
    fn q_key_no_longer_quits() {
        let mut app = make_app(make_skills(1));
        app.handle_event(key(KeyCode::Char('q')));
        assert!(!app.should_quit);
    }

    #[test]
    fn first_ctrl_c_arms_quit_without_quitting() {
        let mut app = make_app(make_skills(1));
        app.handle_event(ctrl(KeyCode::Char('c')));
        assert!(!app.should_quit);
        assert!(app.quit_warning_active());
    }

    #[test]
    fn second_ctrl_c_within_window_quits() {
        let mut app = make_app(make_skills(1));
        app.handle_event(ctrl(KeyCode::Char('c')));
        app.handle_event(ctrl(KeyCode::Char('c')));
        assert!(app.should_quit);
    }

    #[test]
    fn esc_in_list_view_does_not_quit() {
        let mut app = make_app(make_skills(1));
        app.handle_event(key(KeyCode::Esc));
        assert!(!app.should_quit);
    }

    // ── command palette ───────────────────────────────────────────────────────

    #[test]
    fn ctrl_p_opens_command_palette() {
        let mut app = make_app(make_skills(3));
        app.handle_event(ctrl(KeyCode::Char('p')));
        assert!(app.command_palette_open);
        assert!(!app.palette_commands.is_empty());
    }

    #[test]
    fn ctrl_p_toggles_command_palette_closed() {
        let mut app = make_app(make_skills(3));
        app.handle_event(ctrl(KeyCode::Char('p')));
        app.handle_event(ctrl(KeyCode::Char('p')));
        assert!(!app.command_palette_open);
    }

    #[test]
    fn palette_esc_closes() {
        let mut app = make_app(make_skills(3));
        app.handle_event(ctrl(KeyCode::Char('p')));
        app.handle_event(key(KeyCode::Esc));
        assert!(!app.command_palette_open);
    }

    #[test]
    fn palette_includes_selected_skill_commands() {
        let mut app = make_app(make_skills(3));
        app.handle_event(ctrl(KeyCode::Char('p')));
        assert!(app.palette_commands.contains(&PaletteCommand::OpenDetail));
        assert!(app.palette_commands.contains(&PaletteCommand::Remove));
    }

    #[test]
    fn palette_enter_executes_command() {
        let mut app = make_app(make_skills(3));
        app.handle_event(ctrl(KeyCode::Char('p')));
        let idx = app
            .palette_commands
            .iter()
            .position(|c| *c == PaletteCommand::Audit)
            .unwrap();
        app.palette_index = idx;
        app.handle_event(key(KeyCode::Enter));
        assert!(!app.command_palette_open);
        assert_eq!(app.view, View::Audit);
    }

    // ── list navigation ───────────────────────────────────────────────────────

    #[test]
    fn down_arrow_increments_selected_index() {
        let mut app = make_app(make_skills(3));
        app.handle_event(key(KeyCode::Down));
        assert_eq!(app.list_state.selected_index, 1);
    }

    #[test]
    fn down_at_last_item_does_not_overflow() {
        let mut app = make_app(make_skills(2));
        app.handle_event(key(KeyCode::Down));
        app.handle_event(key(KeyCode::Down));
        assert_eq!(app.list_state.selected_index, 1);
    }

    #[test]
    fn up_at_first_item_stays_at_zero() {
        let mut app = make_app(make_skills(3));
        app.handle_event(key(KeyCode::Up));
        assert_eq!(app.list_state.selected_index, 0);
    }

    // ── scope filter ──────────────────────────────────────────────────────────

    #[test]
    fn tab_cycles_scope_filter() {
        let mut app = make_app(vec![]);
        assert_eq!(app.list_state.scope_filter, ScopeFilter::All);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.list_state.scope_filter, ScopeFilter::Global);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.list_state.scope_filter, ScopeFilter::Project);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.list_state.scope_filter, ScopeFilter::All);
    }

    #[test]
    fn visible_skills_all_filter_returns_all() {
        let app = make_app(vec![
            make_skill("a", Scope::Global),
            make_skill("b", Scope::Project),
        ]);
        assert_eq!(app.visible_skills().len(), 2);
    }

    #[test]
    fn visible_skills_global_filter_excludes_project() {
        let mut app = make_app(vec![
            make_skill("a", Scope::Global),
            make_skill("b", Scope::Project),
        ]);
        app.list_state.scope_filter = ScopeFilter::Global;
        let visible = app.visible_skills();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].scope, Scope::Global);
    }

    #[test]
    fn visible_skills_project_filter_excludes_global() {
        let mut app = make_app(vec![
            make_skill("a", Scope::Global),
            make_skill("b", Scope::Project),
        ]);
        app.list_state.scope_filter = ScopeFilter::Project;
        let visible = app.visible_skills();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].scope, Scope::Project);
    }

    #[test]
    fn tab_resets_selected_index_to_zero() {
        let mut app = make_app(make_skills(3));
        app.list_state.selected_index = 2;
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.list_state.selected_index, 0);
    }

    // ── view transitions ──────────────────────────────────────────────────────

    #[test]
    fn enter_transitions_to_detail_view() {
        let mut app = make_app(make_skills(2));
        app.handle_event(key(KeyCode::Enter));
        assert_eq!(app.view, View::Detail);
    }

    #[test]
    fn esc_in_detail_returns_to_list() {
        let mut app = make_app(make_skills(1));
        app.view = View::Detail;
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::List);
    }

    #[test]
    fn s_key_transitions_to_search_view() {
        let mut app = make_app(make_skills(1));
        app.handle_event(key(KeyCode::Char('s')));
        assert_eq!(app.view, View::Search);
    }

    #[test]
    fn esc_in_search_returns_to_list() {
        let mut app = make_app(make_skills(1));
        app.view = View::Search;
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::List);
    }

    #[test]
    fn question_mark_opens_help() {
        let mut app = make_app(make_skills(1));
        app.handle_event(key(KeyCode::Char('?')));
        assert_eq!(app.view, View::Help);
    }

    #[test]
    fn esc_in_help_returns_to_list() {
        let mut app = make_app(make_skills(1));
        app.view = View::Help;
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::List);
    }

    // ── detail scroll ─────────────────────────────────────────────────────────

    #[test]
    fn j_in_detail_increments_scroll() {
        let mut app = make_app(make_skills(1));
        app.view = View::Detail;
        app.handle_event(key(KeyCode::Char('j')));
        assert_eq!(app.detail_scroll, 1);
    }

    #[test]
    fn k_in_detail_does_not_go_below_zero() {
        let mut app = make_app(make_skills(1));
        app.view = View::Detail;
        app.handle_event(key(KeyCode::Char('k')));
        assert_eq!(app.detail_scroll, 0);
    }

    // ── diff view ─────────────────────────────────────────────────────────────

    #[test]
    fn d_in_detail_opens_diff_when_update_available() {
        let mut app = make_app(make_skills(1));
        app.view = View::Detail;
        app.all_skills[0].drift_state = ai_skill_core::DriftState::UpdateAvailable {
            local_hash: "abc".into(),
            upstream_hash: "def".into(),
        };
        app.handle_event(key(KeyCode::Char('d')));
        assert_eq!(app.view, View::Diff);
    }

    #[test]
    fn d_in_detail_does_not_open_diff_without_update() {
        let mut app = make_app(make_skills(1));
        app.view = View::Detail;
        app.handle_event(key(KeyCode::Char('d')));
        assert_eq!(app.view, View::Detail);
    }

    #[test]
    fn esc_in_diff_returns_to_detail() {
        let mut app = make_app(make_skills(1));
        app.view = View::Diff;
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::Detail);
    }

    #[test]
    fn j_in_diff_increments_scroll() {
        let mut app = make_app(make_skills(1));
        app.view = View::Diff;
        app.handle_event(key(KeyCode::Char('j')));
        assert_eq!(app.diff_scroll, 1);
    }

    // ── search ────────────────────────────────────────────────────────────────

    #[test]
    fn typing_char_in_search_updates_query_and_calls_catalog() {
        let mut app = App::new(
            vec![],
            FakeCatalog(vec![CatalogEntry {
                name: "omarchy".to_string(),
                description: "WM skill".to_string(),
                url: None,
            }]),
            FakeInstaller::default(),
            FakeToggler::default(),
            Box::new(FakeProfileStore::default()),
            Box::new(FakeCreator::default()),
            Box::new(FakeWriter::default()),
            Box::new(FakeSettingsStore),
            Box::new(ai_skill_core::NoopExternalScanner),
            Box::new(ai_skill_core::NoopSignatureVerifier),
            Box::new(ai_skill_core::NoopSshConnector),
            Box::new(FakeBundleStore),
            Box::new(FakeSkillSync),
            Box::new(FakeConfigStore),
            TuiConfig::default(),
        );
        app.view = View::Search;
        app.handle_event(key(KeyCode::Char('o')));
        assert_eq!(app.search_state.query, "o");
        assert_eq!(app.search_state.results.len(), 1);
    }

    #[test]
    fn backspace_removes_last_char_from_query() {
        let mut app = App::new(
            vec![],
            FakeCatalog(vec![]),
            FakeInstaller::default(),
            FakeToggler::default(),
            Box::new(FakeProfileStore::default()),
            Box::new(FakeCreator::default()),
            Box::new(FakeWriter::default()),
            Box::new(FakeSettingsStore),
            Box::new(ai_skill_core::NoopExternalScanner),
            Box::new(ai_skill_core::NoopSignatureVerifier),
            Box::new(ai_skill_core::NoopSshConnector),
            Box::new(FakeBundleStore),
            Box::new(FakeSkillSync),
            Box::new(FakeConfigStore),
            TuiConfig::default(),
        );
        app.view = View::Search;
        app.search_state.query = "om".to_string();
        app.handle_event(key(KeyCode::Backspace));
        assert_eq!(app.search_state.query, "o");
    }

    #[test]
    fn search_error_is_stored_in_state() {
        let mut app = App::new(
            vec![],
            ErrorCatalog,
            FakeInstaller::default(),
            FakeToggler::default(),
            Box::new(FakeProfileStore::default()),
            Box::new(FakeCreator::default()),
            Box::new(FakeWriter::default()),
            Box::new(FakeSettingsStore),
            Box::new(ai_skill_core::NoopExternalScanner),
            Box::new(ai_skill_core::NoopSignatureVerifier),
            Box::new(ai_skill_core::NoopSshConnector),
            Box::new(FakeBundleStore),
            Box::new(FakeSkillSync),
            Box::new(FakeConfigStore),
            TuiConfig::default(),
        );
        app.view = View::Search;
        app.handle_event(key(KeyCode::Char('x')));
        assert!(app.search_state.error.is_some());
        assert!(app.search_state.results.is_empty());
    }

    #[test]
    fn search_j_k_navigates_results() {
        let mut app = App::new(
            vec![],
            FakeCatalog(vec![
                CatalogEntry {
                    name: "a".into(),
                    description: "".into(),
                    url: None,
                },
                CatalogEntry {
                    name: "b".into(),
                    description: "".into(),
                    url: None,
                },
            ]),
            FakeInstaller::default(),
            FakeToggler::default(),
            Box::new(FakeProfileStore::default()),
            Box::new(FakeCreator::default()),
            Box::new(FakeWriter::default()),
            Box::new(FakeSettingsStore),
            Box::new(ai_skill_core::NoopExternalScanner),
            Box::new(ai_skill_core::NoopSignatureVerifier),
            Box::new(ai_skill_core::NoopSshConnector),
            Box::new(FakeBundleStore),
            Box::new(FakeSkillSync),
            Box::new(FakeConfigStore),
            TuiConfig::default(),
        );
        app.view = View::Search;
        app.handle_event(key(KeyCode::Char('x')));
        app.handle_event(key(KeyCode::Down));
        assert_eq!(app.search_state.selected_index, 1);
        app.handle_event(key(KeyCode::Up));
        assert_eq!(app.search_state.selected_index, 0);
    }

    // ── tag filter ───────────────────────────────────────────────────────────

    fn make_skill_with_tags(name: &str, tags: Vec<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: vec![],
            tags: tags.into_iter().map(str::to_string).collect(),
            managed: false,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: ai_skill_core::DriftState::default(),
        }
    }

    #[test]
    fn all_tags_returns_unique_sorted_tags() {
        let app = make_app(vec![
            make_skill_with_tags("a", vec!["git", "productivity"]),
            make_skill_with_tags("b", vec!["git", "rust"]),
        ]);
        assert_eq!(app.all_tags(), vec!["git", "productivity", "rust"]);
    }

    #[test]
    fn all_tags_empty_when_no_tags() {
        let app = make_app(make_skills(3));
        assert!(app.all_tags().is_empty());
    }

    #[test]
    fn cycle_tag_filter_advances_through_tags() {
        let mut app = make_app(vec![
            make_skill_with_tags("a", vec!["git"]),
            make_skill_with_tags("b", vec!["rust"]),
        ]);
        assert_eq!(app.list_state.tag_filter, None);
        app.cycle_tag_filter();
        assert_eq!(app.list_state.tag_filter, Some("git".into()));
        app.cycle_tag_filter();
        assert_eq!(app.list_state.tag_filter, Some("rust".into()));
        app.cycle_tag_filter();
        assert_eq!(app.list_state.tag_filter, None);
    }

    #[test]
    fn visible_skills_applies_tag_filter() {
        let mut app = make_app(vec![
            make_skill_with_tags("a", vec!["git"]),
            make_skill_with_tags("b", vec!["rust"]),
        ]);
        app.list_state.tag_filter = Some("git".into());
        let visible = app.visible_skills();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "a");
    }

    #[test]
    fn t_key_cycles_tag_filter() {
        let mut app = make_app(vec![make_skill_with_tags("a", vec!["git"])]);
        app.handle_event(key(KeyCode::Char('t')));
        assert_eq!(app.list_state.tag_filter, Some("git".into()));
    }

    // ── multi-select ─────────────────────────────────────────────────────────

    #[test]
    fn space_adds_index_to_selected_items() {
        let mut app = make_app(make_skills(3));
        app.handle_event(key(KeyCode::Char(' ')));
        assert!(app.list_state.selected_items.contains(&0));
    }

    #[test]
    fn space_again_removes_index_from_selected_items() {
        let mut app = make_app(make_skills(3));
        app.handle_event(key(KeyCode::Char(' ')));
        app.handle_event(key(KeyCode::Char(' ')));
        assert!(!app.list_state.selected_items.contains(&0));
    }

    // ── lifecycle actions ─────────────────────────────────────────────────────

    fn make_skill_at_path(name: &str, path: &str) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(path),
            scope: Scope::Global,
            agents: vec![],
            tags: vec![],
            managed: false,
            mode: SkillMode::Active,
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: ai_skill_core::DriftState::default(),
        }
    }

    fn make_disabled_skill_at_path(name: &str, path: &str) -> Skill {
        Skill {
            mode: SkillMode::Disabled,
            ..make_skill_at_path(name, path)
        }
    }

    #[test]
    fn d_key_on_skill_sets_disable_action_and_confirm_view() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        app.handle_event(key(KeyCode::Char('d')));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(
            app.pending_action,
            Some(AppAction::Disable { .. })
        ));
    }

    #[test]
    fn r_key_on_skill_sets_remove_action_and_confirm_view() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        app.handle_event(key(KeyCode::Char('r')));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(app.pending_action, Some(AppAction::Remove { .. })));
    }

    #[test]
    fn u_key_on_skill_sets_update_action_and_confirm_view() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        app.handle_event(key(KeyCode::Char('u')));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(app.pending_action, Some(AppAction::Update { .. })));
    }

    #[test]
    fn e_key_on_disabled_skill_sets_enable_action_and_confirm_view() {
        let mut app = make_app(vec![make_disabled_skill_at_path(
            "alpha",
            "/skills/alpha.disabled",
        )]);
        app.handle_event(key(KeyCode::Char('e')));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(app.pending_action, Some(AppAction::Enable { .. })));
    }

    #[test]
    fn e_key_on_valid_skill_opens_editor() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        app.handle_event(key(KeyCode::Char('e')));
        assert_eq!(app.view, View::Editor);
        assert!(app.editor_state.is_some());
    }

    #[test]
    fn y_in_confirm_executes_action_and_sets_needs_refresh() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        app.handle_event(key(KeyCode::Char('d')));
        assert_eq!(app.view, View::Confirm);
        app.handle_event(key(KeyCode::Char('y')));
        assert!(app.needs_refresh);
        assert_eq!(app.view, View::List);
        assert!(
            app.toggler
                .calls
                .borrow()
                .iter()
                .any(|c| c.starts_with("disable:"))
        );
    }

    #[test]
    fn n_in_confirm_cancels_without_refresh() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        app.handle_event(key(KeyCode::Char('d')));
        app.handle_event(key(KeyCode::Char('n')));
        assert!(!app.needs_refresh);
        assert_eq!(app.view, View::List);
        assert!(app.pending_action.is_none());
    }

    #[test]
    fn a_key_on_unmanaged_skill_calls_adopt_and_refreshes() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        assert!(!app.all_skills[0].managed);
        app.handle_event(key(KeyCode::Char('a')));
        assert!(app.needs_refresh);
        assert!(
            app.toggler
                .calls
                .borrow()
                .iter()
                .any(|c| c.starts_with("adopt:"))
        );
    }

    // ── install wizard ────────────────────────────────────────────────────────

    #[test]
    fn enter_in_search_with_result_opens_install_wizard() {
        let mut app = App::new(
            vec![],
            FakeCatalog(vec![CatalogEntry {
                name: "omarchy".into(),
                description: "".into(),
                url: None,
            }]),
            FakeInstaller::default(),
            FakeToggler::default(),
            Box::new(FakeProfileStore::default()),
            Box::new(FakeCreator::default()),
            Box::new(FakeWriter::default()),
            Box::new(FakeSettingsStore),
            Box::new(ai_skill_core::NoopExternalScanner),
            Box::new(ai_skill_core::NoopSignatureVerifier),
            Box::new(ai_skill_core::NoopSshConnector),
            Box::new(FakeBundleStore),
            Box::new(FakeSkillSync),
            Box::new(FakeConfigStore),
            TuiConfig::default(),
        );
        app.view = View::Search;
        app.handle_event(key(KeyCode::Char('o'))); // search to get results
        app.handle_event(key(KeyCode::Enter));
        assert_eq!(app.view, View::InstallWizard);
        assert_eq!(
            app.install_wizard_state.entry.as_ref().unwrap().name,
            "omarchy"
        );
    }

    #[test]
    fn tab_in_install_wizard_cycles_scope() {
        let mut app = make_app(vec![]);
        app.view = View::InstallWizard;
        app.install_wizard_state.scope = Scope::Global;
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.install_wizard_state.scope, Scope::Project);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.install_wizard_state.scope, Scope::Global);
    }

    #[test]
    fn enter_in_install_wizard_sets_install_action_and_confirm_view() {
        let mut app = make_app(vec![]);
        app.view = View::InstallWizard;
        app.install_wizard_state.entry = Some(CatalogEntry {
            name: "omarchy".into(),
            description: "".into(),
            url: None,
        });
        app.handle_event(key(KeyCode::Enter));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(
            app.pending_action,
            Some(AppAction::Install { .. })
        ));
    }

    #[test]
    fn confirm_install_calls_installer_and_refreshes() {
        let mut app = make_app(vec![]);
        app.view = View::InstallWizard;
        app.install_wizard_state.entry = Some(CatalogEntry {
            name: "omarchy".into(),
            description: "".into(),
            url: None,
        });
        app.handle_event(key(KeyCode::Enter)); // → Confirm
        app.handle_event(key(KeyCode::Char('y'))); // execute
        assert!(app.needs_refresh);
        assert!(
            app.installer
                .calls
                .borrow()
                .iter()
                .any(|c| c.starts_with("install:"))
        );
    }

    // ── resize ────────────────────────────────────────────────────────────────

    #[test]
    fn resize_event_does_not_panic_or_change_state() {
        let mut app = make_app(make_skills(2));
        app.handle_event(AppEvent::Resize);
        assert_eq!(app.list_state.selected_index, 0);
        assert!(!app.should_quit);
    }

    // ── scan gate (H3.5) ─────────────────────────────────────────────────────

    #[test]
    fn wizard_enter_with_clean_description_goes_to_confirm() {
        let mut app = make_app(vec![]);
        app.view = View::InstallWizard;
        app.install_wizard_state.entry = Some(CatalogEntry {
            name: "safe-skill".into(),
            description: "A perfectly safe skill.".into(),
            url: None,
        });
        app.handle_event(key(KeyCode::Enter));
        assert_eq!(app.view, View::Confirm);
        assert!(app.scan_findings.is_empty());
    }

    #[test]
    fn wizard_enter_with_dangerous_description_goes_to_scan_report() {
        let mut app = make_app(vec![]);
        app.view = View::InstallWizard;
        app.install_wizard_state.entry = Some(CatalogEntry {
            name: "bad-skill".into(),
            description: "Run: rm -rf /tmp/cache".into(),
            url: None,
        });
        app.handle_event(key(KeyCode::Enter));
        assert_eq!(app.view, View::ScanReport);
        assert!(!app.scan_findings.is_empty());
        assert!(app.pending_action.is_some());
    }

    #[test]
    fn esc_in_scan_report_returns_to_install_wizard_and_clears_findings() {
        let mut app = make_app(vec![]);
        app.view = View::ScanReport;
        app.scan_findings = vec![ai_skill_core::ScanFinding {
            severity: ai_skill_core::Severity::High,
            category: ai_skill_core::ScanCategory::DangerousShellPattern,
            detail: "rm -rf detected".into(),
            line: 1,
        }];
        app.pending_action = Some(AppAction::Install {
            name: "bad".into(),
            agents: vec![],
            scope: Scope::Global,
        });
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::InstallWizard);
        assert!(app.scan_findings.is_empty());
        assert!(app.pending_action.is_none());
    }

    #[test]
    fn enter_in_scan_report_proceeds_to_confirm() {
        let mut app = make_app(vec![]);
        app.view = View::ScanReport;
        app.scan_findings = vec![ai_skill_core::ScanFinding {
            severity: ai_skill_core::Severity::High,
            category: ai_skill_core::ScanCategory::DangerousShellPattern,
            detail: "rm -rf detected".into(),
            line: 1,
        }];
        app.pending_action = Some(AppAction::Install {
            name: "bad".into(),
            agents: vec![],
            scope: Scope::Global,
        });
        app.handle_event(key(KeyCode::Enter));
        assert_eq!(app.view, View::Confirm);
        assert!(app.scan_findings.is_empty());
        assert!(app.pending_action.is_some());
    }

    // ── profiles panel (H3.3) ─────────────────────────────────────────────────

    #[test]
    fn p_key_in_list_switches_to_profiles_view() {
        let mut app = make_app(vec![]);
        app.handle_event(key(KeyCode::Char('p')));
        assert_eq!(app.view, View::Profiles);
    }

    #[test]
    fn esc_in_profiles_returns_to_list() {
        let mut app = make_app(vec![]);
        app.view = View::Profiles;
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::List);
    }

    #[test]
    fn j_k_in_profiles_navigates_selected_index() {
        let mut app = make_app(vec![]);
        app.view = View::Profiles;
        app.profile_state.profiles = vec![
            Profile {
                name: "a".into(),
                skill_names: vec![],
                phase: None,
            },
            Profile {
                name: "b".into(),
                skill_names: vec![],
                phase: None,
            },
        ];
        app.handle_event(key(KeyCode::Char('j')));
        assert_eq!(app.profile_state.selected_index, 1);
        app.handle_event(key(KeyCode::Char('k')));
        assert_eq!(app.profile_state.selected_index, 0);
    }

    #[test]
    fn a_in_profiles_sets_activate_profile_action_and_confirm_view() {
        let mut app = make_app(vec![make_skill("alpha", Scope::Global)]);
        app.view = View::Profiles;
        app.profile_state.profiles = vec![Profile {
            name: "dev".into(),
            skill_names: vec!["alpha".into(), "new".into()],
            phase: None,
        }];
        app.handle_event(key(KeyCode::Char('a')));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(
            app.pending_action,
            Some(AppAction::ActivateProfile { .. })
        ));
    }

    #[test]
    fn f_in_profiles_enables_creating_mode() {
        let mut app = make_app(vec![]);
        app.view = View::Profiles;
        app.handle_event(key(KeyCode::Char('f')));
        assert!(app.profile_state.creating);
    }

    #[test]
    fn enter_in_creating_mode_saves_profile_from_current_skills() {
        let mut app = make_app(vec![make_skill("alpha", Scope::Global)]);
        app.view = View::Profiles;
        app.profile_state.creating = true;
        app.profile_state.new_name_input = "my-profile".into();
        app.handle_event(key(KeyCode::Enter));
        assert!(!app.profile_state.creating);
        assert!(
            app.profile_state
                .profiles
                .iter()
                .any(|p| p.name == "my-profile")
        );
        assert!(
            app.profile_state.profiles[0]
                .skill_names
                .contains(&"alpha".to_string())
        );
    }

    #[test]
    fn d_in_profiles_deletes_selected_profile() {
        let mut app = make_app(vec![]);
        let p = Profile {
            name: "dev".into(),
            skill_names: vec![],
            phase: None,
        };
        app.profile_store.save(&p).unwrap();
        app.view = View::Profiles;
        app.profile_state.profiles = vec![p];
        app.handle_event(key(KeyCode::Char('d')));
        assert!(app.profile_state.profiles.is_empty());
    }

    // ── create wizard (H4.5) ─────────────────────────────────────────────────────

    #[test]
    fn c_key_in_list_opens_create_wizard() {
        let mut app = make_app(vec![]);
        app.handle_event(key(KeyCode::Char('c')));
        assert_eq!(app.view, View::CreateWizard);
    }

    #[test]
    fn esc_in_create_wizard_returns_to_list() {
        let mut app = make_app(vec![]);
        app.view = View::CreateWizard;
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::List);
    }

    #[test]
    fn tab_in_create_wizard_advances_step() {
        let mut app = make_app(vec![]);
        app.view = View::CreateWizard;
        assert_eq!(app.create_wizard_state.step, CreateStep::Name);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.create_wizard_state.step, CreateStep::Agents);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.create_wizard_state.step, CreateStep::Tags);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.create_wizard_state.step, CreateStep::Preview);
    }

    #[test]
    fn enter_in_preview_step_calls_creator_and_refreshes() {
        let mut app = make_app(vec![]);
        app.view = View::CreateWizard;
        app.create_wizard_state.step = CreateStep::Preview;
        app.create_wizard_state.name = "my-skill".into();
        app.create_wizard_state.agents_input = "claude".into();
        app.handle_event(key(KeyCode::Enter));
        assert!(app.needs_refresh);
        assert_eq!(app.view, View::List);
    }

    #[test]
    fn typing_in_name_step_builds_name() {
        let mut app = make_app(vec![]);
        app.view = View::CreateWizard;
        app.handle_event(key(KeyCode::Char('f')));
        app.handle_event(key(KeyCode::Char('o')));
        app.handle_event(key(KeyCode::Char('o')));
        assert_eq!(app.create_wizard_state.name, "foo");
    }

    #[test]
    fn backspace_in_name_step_removes_char() {
        let mut app = make_app(vec![]);
        app.view = View::CreateWizard;
        app.create_wizard_state.name = "foo".into();
        app.handle_event(key(KeyCode::Backspace));
        assert_eq!(app.create_wizard_state.name, "fo");
    }

    // ── editor (H4.4) ────────────────────────────────────────────────────────────

    #[test]
    fn esc_in_editor_returns_to_list() {
        let mut app = make_app(vec![]);
        app.view = View::Editor;
        app.editor_state = Some(EditorState {
            skill: make_skill("x", Scope::Global),
            field: EditField::default(),
            name_input: "x".into(),
            agents_input: String::new(),
            tags_input: String::new(),
            warnings: vec![],
        });
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::List);
        assert!(app.editor_state.is_none());
    }

    #[test]
    fn tab_in_editor_cycles_edit_field() {
        let mut app = make_app(vec![]);
        app.view = View::Editor;
        app.editor_state = Some(EditorState {
            skill: make_skill("x", Scope::Global),
            field: EditField::default(),
            name_input: "x".into(),
            agents_input: String::new(),
            tags_input: String::new(),
            warnings: vec![],
        });
        assert_eq!(app.editor_state.as_ref().unwrap().field, EditField::Name);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.editor_state.as_ref().unwrap().field, EditField::Agents);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.editor_state.as_ref().unwrap().field, EditField::Tags);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.editor_state.as_ref().unwrap().field, EditField::Name);
    }

    #[test]
    fn enter_in_editor_calls_writer_and_refreshes() {
        let mut app = make_app(vec![make_skill("alpha", Scope::Global)]);
        app.view = View::Editor;
        app.editor_state = Some(EditorState {
            skill: make_skill("alpha", Scope::Global),
            field: EditField::default(),
            name_input: "alpha".into(),
            agents_input: "claude".into(),
            tags_input: String::new(),
            warnings: vec![],
        });
        app.handle_event(key(KeyCode::Enter));
        assert!(app.needs_refresh);
        assert_eq!(app.view, View::List);
        assert!(app.editor_state.is_none());
    }

    #[test]
    fn e_key_on_disabled_skill_still_enables_not_opens_editor() {
        let mut app = make_app(vec![make_disabled_skill_at_path(
            "alpha",
            "/skills/alpha.disabled",
        )]);
        app.handle_event(key(KeyCode::Char('e')));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(app.pending_action, Some(AppAction::Enable { .. })));
        assert!(app.editor_state.is_none());
    }

    // ── name-only toggle ────────────────────────────────────────────────────────────

    #[test]
    fn n_key_on_active_skill_calls_collapse_and_refreshes() {
        let mut app = make_app(vec![make_skill_at_path("alpha", "/skills/alpha")]);
        app.handle_event(key(KeyCode::Char('n')));
        assert!(app.needs_refresh);
        assert!(
            app.toggler
                .calls
                .borrow()
                .iter()
                .any(|c| c.starts_with("collapse:"))
        );
    }

    #[test]
    fn n_key_on_name_only_skill_calls_expand_and_refreshes() {
        let mut app = make_app(vec![Skill {
            mode: SkillMode::NameOnly,
            ..make_skill_at_path("alpha", "/skills/alpha")
        }]);
        app.handle_event(key(KeyCode::Char('n')));
        assert!(app.needs_refresh);
        assert!(
            app.toggler
                .calls
                .borrow()
                .iter()
                .any(|c| c.starts_with("expand:"))
        );
    }

    #[test]
    fn n_key_on_disabled_skill_calls_collapse_and_preserves_pref() {
        let mut app = make_app(vec![make_disabled_skill_at_path(
            "alpha",
            "/skills/alpha.disabled",
        )]);
        app.handle_event(key(KeyCode::Char('n')));
        assert!(app.needs_refresh);
        assert!(
            app.toggler
                .calls
                .borrow()
                .iter()
                .any(|c| c.starts_with("collapse:"))
        );
    }

    #[test]
    fn preview_for_action_toggle_name_only_active_shows_collapse() {
        let app = make_app(vec![make_skill_at_path("alpha", "/tmp/alpha")]);
        let action = AppAction::ToggleNameOnly {
            path: PathBuf::from("/tmp/alpha"),
        };
        let preview = app.preview_for_action(&action);
        assert!(preview.contains("collapse"));
    }

    // ── audit view (H4.6) ─────────────────────────────────────────────────────────

    #[test]
    fn shift_a_in_list_opens_audit_view() {
        let mut app = make_app(vec![]);
        app.handle_event(AppEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char('A'),
            modifiers: KeyModifiers::SHIFT,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));
        assert_eq!(app.view, View::Audit);
    }

    #[test]
    fn esc_in_audit_returns_to_list() {
        let mut app = make_app(vec![]);
        app.view = View::Audit;
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.view, View::List);
    }

    // ── drift badge state (H4.2) ─────────────────────────────────────────────────

    #[test]
    fn update_available_drift_state_is_carried_on_skill() {
        use ai_skill_core::DriftState;
        let skill = Skill {
            drift_state: DriftState::UpdateAvailable {
                local_hash: "abc1234".into(),
                upstream_hash: "def5678".into(),
            },
            ..make_skill("drifted", Scope::Global)
        };
        let app = make_app(vec![skill]);
        assert!(matches!(
            app.all_skills[0].drift_state,
            DriftState::UpdateAvailable { .. }
        ));
    }

    #[test]
    fn preview_for_action_activate_profile_shows_diff_counts() {
        let app = make_app(vec![]);
        let ops = vec![
            ProfileOp::Install { name: "a".into() },
            ProfileOp::Install { name: "b".into() },
            ProfileOp::Remove { name: "c".into() },
        ];
        let action = AppAction::ActivateProfile {
            name: "dev".into(),
            ops,
        };
        let preview = app.preview_for_action(&action);
        assert!(preview.contains("install 2"));
        assert!(preview.contains("remove 1"));
    }

    #[test]
    fn preview_for_action_install_shows_command() {
        let app: TestApp = make_app(vec![]);
        let action = AppAction::Install {
            name: "my-skill".into(),
            agents: vec!["claude".into()],
            scope: Scope::Global,
        };
        let preview = app.preview_for_action(&action);
        assert!(preview.contains("npx skills add my-skill"));
    }

    #[test]
    fn preview_for_action_remove_shows_command() {
        let app = make_app(vec![]);
        let action = AppAction::Remove {
            path: PathBuf::from("/tmp/my-skill"),
        };
        let preview = app.preview_for_action(&action);
        assert!(preview.contains("npx skills remove"));
        assert!(preview.contains("/tmp/my-skill"));
    }

    #[test]
    fn preview_for_action_update_shows_command() {
        let app = make_app(vec![]);
        let action = AppAction::Update {
            path: PathBuf::from("/tmp/my-skill"),
        };
        let preview = app.preview_for_action(&action);
        assert!(preview.contains("npx skills update"));
    }

    #[test]
    fn preview_for_action_enable_shows_path() {
        let app = make_app(vec![]);
        let action = AppAction::Enable {
            path: PathBuf::from("/tmp/my-skill.disabled"),
        };
        let preview = app.preview_for_action(&action);
        assert!(preview.contains("enable"));
        assert!(preview.contains("my-skill.disabled"));
    }

    #[test]
    fn preview_for_action_disable_shows_path() {
        let app = make_app(vec![]);
        let action = AppAction::Disable {
            path: PathBuf::from("/tmp/my-skill"),
        };
        let preview = app.preview_for_action(&action);
        assert!(preview.contains("disable"));
        assert!(preview.contains("my-skill"));
    }

    #[test]
    fn preview_for_action_adopt_shows_adopt_prefix() {
        let app = make_app(vec![]);
        let action = AppAction::Adopt {
            path: PathBuf::from("/tmp/my-skill"),
        };
        let preview = app.preview_for_action(&action);
        assert_eq!(preview, "adopt /tmp/my-skill");
    }
}
