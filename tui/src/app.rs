//! Application state, view transitions, and event handling.

use ai_skill_core::{
    AnyCatalogGateway, CatalogEntry, ContextBudget, Profile, ProfileOp, ProfileStore, ScanFinding,
    Scope, Skill, SkillCreator, SkillInstaller, SkillToggler, SkillWriter, ValidationState,
    calculate_budget, scan_skill,
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::path::PathBuf;

use crate::event::{AppEvent, is_quit};

/// The active screen (or overlay) in the TUI.
#[derive(Debug, Clone, PartialEq)]
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

/// Top-level application state, generic over adapters.
pub struct App<G: AnyCatalogGateway, I: SkillInstaller, T: SkillToggler> {
    pub all_skills: Vec<Skill>,
    pub view: View,
    pub view_before_confirm: View,
    pub list_state: ListUiState,
    pub detail_scroll: u16,
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
    pub should_quit: bool,
}

impl<G: AnyCatalogGateway, I: SkillInstaller, T: SkillToggler> App<G, I, T> {
    pub fn new(
        all_skills: Vec<Skill>,
        catalog: G,
        installer: I,
        toggler: T,
        profile_store: Box<dyn ProfileStore>,
        creator: Box<dyn SkillCreator>,
        writer: Box<dyn SkillWriter>,
    ) -> Self {
        let profiles = profile_store.list().unwrap_or_default();
        let budget = calculate_budget(&all_skills);
        Self {
            all_skills,
            budget,
            view: View::List,
            view_before_confirm: View::List,
            list_state: ListUiState::new(),
            detail_scroll: 0,
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
            should_quit: false,
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
        match event {
            AppEvent::Key(key) if is_quit(&key) => {
                self.should_quit = true;
            }
            AppEvent::Key(key) => match self.view.clone() {
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
            },
            AppEvent::Resize => {}
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
                self.should_quit = true;
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
            KeyCode::Char('d') if key.modifiers == KeyModifiers::NONE => {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Disable { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char('e')
                if key.modifiers == KeyModifiers::NONE
                    && self
                        .selected_skill()
                        .map(|s| s.validation == ValidationState::Disabled)
                        .unwrap_or(false) =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Enable { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::NONE => {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Remove { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char('u') if key.modifiers == KeyModifiers::NONE => {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Update { path });
                    self.view_before_confirm = View::List;
                    self.view = View::Confirm;
                }
            }
            KeyCode::Char('a')
                if key.modifiers == KeyModifiers::NONE
                    && self.selected_skill().map(|s| !s.managed).unwrap_or(false) =>
            {
                if let Some(skill) = self.selected_skill() {
                    let path = skill.path.clone();
                    self.pending_action = Some(AppAction::Adopt { path });
                    self.execute_pending_action();
                }
            }
            KeyCode::Char('s') if key.modifiers == KeyModifiers::NONE => {
                self.search_state = SearchState::default();
                self.view = View::Search;
            }
            KeyCode::Char('p') if key.modifiers == KeyModifiers::NONE => {
                let profiles = self.profile_store.list().unwrap_or_default();
                self.profile_state.profiles = profiles;
                self.profile_state.selected_index = 0;
                self.profile_state.creating = false;
                self.profile_state.new_name_input = String::new();
                self.view = View::Profiles;
            }
            KeyCode::Char('?') if key.modifiers == KeyModifiers::NONE => {
                self.view = View::Help;
            }
            KeyCode::Char('c') if key.modifiers == KeyModifiers::NONE => {
                self.create_wizard_state = CreateWizardState::default();
                self.view = View::CreateWizard;
            }
            KeyCode::Char('e')
                if key.modifiers == KeyModifiers::NONE
                    && self
                        .selected_skill()
                        .map(|s| s.validation != ValidationState::Disabled)
                        .unwrap_or(false) =>
            {
                if let Some(skill) = self.selected_skill() {
                    let name_input = skill.name.clone();
                    let agents_input = skill.agents.join(", ");
                    let tags_input = skill.tags.join(", ");
                    let skill = skill.clone();
                    self.editor_state = Some(EditorState {
                        skill,
                        field: EditField::default(),
                        name_input,
                        agents_input,
                        tags_input,
                    });
                    self.view = View::Editor;
                }
            }
            KeyCode::Char('B') => {
                self.view = View::Budget;
            }
            KeyCode::Char('A') => {
                self.view = View::Audit;
            }
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
            _ => {}
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
                let dest = self.view_before_confirm.clone();
                self.view = dest;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.pending_action = None;
                let dest = self.view_before_confirm.clone();
                self.view = dest;
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
                    let findings = scan_skill(&entry.description);
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
                        .filter(|s| s.validation == ValidationState::Valid)
                        .map(|s| s.name.clone())
                        .collect();
                    let profile = Profile { name, skill_names };
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
                }
                CreateStep::Agents => {
                    self.create_wizard_state.agents_input.pop();
                }
                CreateStep::Tags => {
                    self.create_wizard_state.tags_input.pop();
                }
                CreateStep::Preview => {}
            },
            KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                match self.create_wizard_state.step {
                    CreateStep::Name => self.create_wizard_state.name.push(c),
                    CreateStep::Agents => self.create_wizard_state.agents_input.push(c),
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
            }
            KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                if let Some(state) = &mut self.editor_state {
                    match state.field {
                        EditField::Name => state.name_input.push(c),
                        EditField::Agents => state.agents_input.push(c),
                        EditField::Tags => state.tags_input.push(c),
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_audit_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Esc {
            self.view = View::List;
        }
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
    use std::{cell::RefCell, path::PathBuf};

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

    fn make_app(skills: Vec<Skill>) -> TestApp {
        App::new(
            skills,
            FakeCatalog(vec![]),
            FakeInstaller::default(),
            FakeToggler::default(),
            Box::new(FakeProfileStore::default()),
            Box::new(FakeCreator::default()),
            Box::new(FakeWriter::default()),
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
    fn q_key_sets_should_quit() {
        let mut app = make_app(make_skills(1));
        app.handle_event(key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn ctrl_c_sets_should_quit() {
        let mut app = make_app(make_skills(1));
        app.handle_event(ctrl(KeyCode::Char('c')));
        assert!(app.should_quit);
    }

    #[test]
    fn esc_in_list_view_sets_should_quit() {
        let mut app = make_app(make_skills(1));
        app.handle_event(key(KeyCode::Esc));
        assert!(app.should_quit);
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
            validation: ValidationState::Valid,
            manifest_content: None,
            drift_state: ai_skill_core::DriftState::default(),
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
        let mut app = make_app(vec![Skill {
            validation: ValidationState::Disabled,
            ..make_skill_at_path("alpha", "/skills/alpha.disabled")
        }]);
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
            },
            Profile {
                name: "b".into(),
                skill_names: vec![],
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
        });
        app.handle_event(key(KeyCode::Enter));
        assert!(app.needs_refresh);
        assert_eq!(app.view, View::List);
        assert!(app.editor_state.is_none());
    }

    #[test]
    fn e_key_on_disabled_skill_still_enables_not_opens_editor() {
        let mut app = make_app(vec![Skill {
            validation: ValidationState::Disabled,
            ..make_skill_at_path("alpha", "/skills/alpha.disabled")
        }]);
        app.handle_event(key(KeyCode::Char('e')));
        assert_eq!(app.view, View::Confirm);
        assert!(matches!(app.pending_action, Some(AppAction::Enable { .. })));
        assert!(app.editor_state.is_none());
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
