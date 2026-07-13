//! Binary entry point for `ai-skill`: TUI, `--json` subcommands, `--help`, `--version`.

mod app;
mod event;
mod i18n;
mod terminal;
mod ui;

use i18n::I18n;
use ui::theme::Theme;

use ai_skill_adapters::{
    CliInstaller, CompositeCatalogGateway, FsBundleStore, FsConfigStore, FsPluginDiscoverer,
    FsProfileStore, FsSettingsStore, FsSkillCreator, FsSkillRepository, FsSkillWriter, FsToggler,
    FsUsageHistoryReader, FsWatcher, GitDriftChecker, GitSkillDiffReader, GitSkillSync,
    NpxCatalogGateway, SshCommandConnector,
};
use ai_skill_core::{
    BudgetWarning, ConfigStore, DriftChecker, NoopExternalScanner, NoopSignatureVerifier,
    PluginMarketplaceDiscovery, RemoteHost, Scope, Skill, SkillDiffReader, SkillMode,
    SkillRepository, SkillUsageReader, ValidationState, audit_skills, build_usage_report,
    calculate_budget, classify_budget,
};

use app::{App, View};
use event::next_event;
use ratatui::layout::{Constraint, Layout};
use serde::Serialize;
use std::{path::PathBuf, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match parse_cli_args(std::env::args().skip(1))
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?
    {
        CliMode::Print(output) => {
            print!("{output}");
            return Ok(());
        }
        CliMode::Json(command) => {
            let skills = load_cli_skills()?;
            print_json(command, &skills)?;
            return Ok(());
        }
        CliMode::Markdown(command) => {
            let skills = load_cli_skills()?;
            print_markdown(command, &skills);
            return Ok(());
        }
        CliMode::Tui => {}
    }

    terminal::install_panic_hook();

    let config_store = config_store_from_env();
    let config = config_store.read().unwrap_or_default();
    let stale_after_days = config.stale_after_days;
    let theme = Theme::from_config(&config.theme);
    let i18n = I18n::from_config(config.locale.as_deref());

    let mut repo = FsSkillRepository::from_env()?;
    repo.add_custom_paths(config.custom_agent_paths.clone());
    let drift_checker = GitDriftChecker;
    let mut skills = list_skills_with_drift(&repo)?;
    let plugin_discoverer = FsPluginDiscoverer::from_env().ok();
    if let Some(ref d) = plugin_discoverer
        && let Ok(plugin_skills) = d.discover_skills()
    {
        let plugin_skill_objs: Vec<Skill> = plugin_skills
            .into_iter()
            .map(|ps| Skill {
                name: ps.name,
                path: ps.path,
                scope: Scope::Global,
                agents: vec![format!("Plugin ({})", ps.marketplace_key)],
                tags: vec![],
                managed: false,
                mode: SkillMode::Active,
                validation: ValidationState::Valid,
                manifest_content: ps.manifest_content,
                drift_state: ai_skill_core::DriftState::Unknown,
            })
            .collect();
        skills.extend(plugin_skill_objs);
    }
    let skill_roots = vec![home_dir()?.join(".claude").join("skills")];
    let watcher = FsWatcher::new(&skill_roots).ok();
    let hot_reload_active = watcher
        .as_ref()
        .map(|watcher| watcher.watched_paths() > 0)
        .unwrap_or(false);

    let usage_reader = FsUsageHistoryReader::from_env();
    let diff_reader = GitSkillDiffReader::new();
    let mut usage_report = build_usage_report(
        &usage_reader.read_events().unwrap_or_default(),
        &skill_names(&skills),
        stale_after_days,
    );

    let mut term = terminal::setup()?;
    let settings_store = FsSettingsStore::from_env()
        .ok()
        .map(|s| Box::new(s) as Box<dyn ai_skill_core::SettingsStore>)
        .unwrap_or_else(|| {
            let path = std::path::PathBuf::from(".claude/settings.json");
            Box::new(FsSettingsStore::new(path)) as Box<dyn ai_skill_core::SettingsStore>
        });

    let mut app = App::new(
        skills,
        CompositeCatalogGateway::new(vec![Box::new(NpxCatalogGateway)]),
        CliInstaller,
        FsToggler,
        Box::new(FsProfileStore::from_env()?),
        Box::new(FsSkillCreator::from_env()?),
        Box::new(FsSkillWriter),
        settings_store,
        Box::new(NoopExternalScanner),
        Box::new(NoopSignatureVerifier),
        Box::new(SshCommandConnector),
        Box::new(FsBundleStore::from_env()?),
        Box::new(GitSkillSync::new(
            home_dir()?.join(".claude").join("skills"),
        )),
        config_store,
        config,
    );

    app.ssh_state.hosts = vec![RemoteHost::new("local", "127.0.0.1")];

    loop {
        term.draw(|f| {
            let area = f.area();
            if ui::resize_panel::is_too_small(area) {
                ui::resize_panel::render_resize_panel(area, f);
                return;
            }

            let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

            let main_area = chunks[0];
            let status_area = chunks[1];

            let visible = app.visible_skills();
            match &app.view {
                View::List => {
                    ui::installed_panel::render_installed_panel(
                        &visible,
                        &app.list_state,
                        main_area,
                        f,
                    );
                }
                View::Detail => {
                    if let Some(skill) = app.selected_skill() {
                        let auto_trigger = app.settings.as_ref().map(|s| {
                            s.skill_overrides
                                .iter()
                                .find(|o| o.skill_name == skill.name)
                                .map(|o| o.auto_trigger)
                                .unwrap_or(s.auto_trigger)
                        });
                        ui::detail_panel::render_detail_panel(
                            skill,
                            app.detail_scroll,
                            main_area,
                            f,
                            auto_trigger,
                        );
                    }
                }
                View::Search => {
                    ui::search_panel::render_search_panel(&app.search_state, main_area, f);
                }
                View::Help => {
                    ui::installed_panel::render_installed_panel(
                        &visible,
                        &app.list_state,
                        main_area,
                        f,
                    );
                    ui::help_overlay::render_help_overlay(main_area, f, &i18n);
                }
                View::Confirm => {
                    ui::installed_panel::render_installed_panel(
                        &visible,
                        &app.list_state,
                        main_area,
                        f,
                    );
                    if let Some(action) = &app.pending_action {
                        let preview = app.preview_for_action(action);
                        ui::confirm_panel::render_confirm_panel(&preview, main_area, f);
                    }
                }
                View::InstallWizard => {
                    ui::install_wizard::render_install_wizard(
                        &app.install_wizard_state,
                        main_area,
                        f,
                    );
                }
                View::ScanReport => {
                    ui::scan_report::render_scan_report(
                        &app.scan_findings,
                        &theme,
                        main_area,
                        f,
                        &i18n,
                    );
                }
                View::Profiles => {
                    ui::profiles_panel::render_profiles_panel(
                        &app.profile_state,
                        main_area,
                        f,
                        app.profile_export_message.as_deref(),
                    );
                }
                View::CreateWizard => {
                    ui::create_wizard::render_create_wizard(&app.create_wizard_state, main_area, f);
                }
                View::Editor => {
                    if let Some(state) = &app.editor_state {
                        ui::editor_panel::render_editor_panel(state, main_area, f);
                    }
                }
                View::Audit => {
                    ui::audit_panel::render_audit_panel(
                        &app.all_skills,
                        &usage_report,
                        &theme,
                        main_area,
                        f,
                        &i18n,
                    );
                }
                View::Budget => {
                    ui::budget_panel::render_budget_panel(&app.budget, main_area, f);
                }
                View::Diff => {
                    if let Some(skill) = app.selected_skill() {
                        let diff = diff_reader.read_diff(&skill.path);
                        ui::diff_panel::render_diff_panel(
                            skill,
                            &diff,
                            app.diff_scroll,
                            &theme,
                            main_area,
                            f,
                            &i18n,
                        );
                    }
                }
                View::Settings => {
                    let chunks =
                        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                            .split(main_area);
                    if let Some(ref settings) = app.settings {
                        ui::settings_panel::render_settings_panel(
                            settings,
                            &app.settings_state,
                            chunks[0],
                            f,
                            &i18n,
                        );
                    }
                    ui::settings_panel::render_config_panel(
                        &app.config,
                        &app.config_state,
                        chunks[1],
                        f,
                        &i18n,
                    );
                }
                View::ImportChain => {
                    if let Some(skill) = app.selected_skill() {
                        let auto_trigger = app.settings.as_ref().map(|s| {
                            s.skill_overrides
                                .iter()
                                .find(|o| o.skill_name == skill.name)
                                .map(|o| o.auto_trigger)
                                .unwrap_or(s.auto_trigger)
                        });
                        ui::detail_panel::render_detail_panel(
                            skill,
                            app.detail_scroll,
                            main_area,
                            f,
                            auto_trigger,
                        );
                    }
                    if let Some(ref result) = app.import_chain_result {
                        ui::import_chain_panel::render_import_chain(result, main_area, f);
                    }
                }
                View::SshRemote => {
                    ui::ssh_panel::render_ssh_panel(&app.ssh_state, main_area, f);
                }
                View::Bundles => {
                    ui::bundles_panel::render_bundles_panel(&app.bundle_state, main_area, f);
                }
                View::Sync => {
                    ui::sync_panel::render_sync_panel(&mut app.sync_state, main_area, f);
                }
            }

            let warning = classify_budget(&app.budget);
            ui::status_bar::render_status_bar(
                &app.view,
                status_area,
                f,
                Some(&warning),
                hot_reload_active,
                &i18n,
            );
        })?;

        if watcher
            .as_ref()
            .map(|w| w.rx.try_recv().is_ok())
            .unwrap_or(false)
        {
            app.needs_refresh = true;
        }

        if let Some(event) = next_event(Duration::from_millis(250))? {
            app.handle_event(event);
        }

        if app.needs_refresh {
            if let Ok(mut skills) = repo.list() {
                apply_drift(&mut skills, &drift_checker);
                if let Some(ref d) = plugin_discoverer
                    && let Ok(plugin_skills) = d.discover_skills()
                {
                    let plugin_skill_objs: Vec<Skill> = plugin_skills
                        .into_iter()
                        .map(|ps| Skill {
                            name: ps.name,
                            path: ps.path,
                            scope: Scope::Global,
                            agents: vec![format!("Plugin ({})", ps.marketplace_key)],
                            tags: vec![],
                            managed: false,
                            mode: SkillMode::Active,
                            validation: ValidationState::Valid,
                            manifest_content: ps.manifest_content,
                            drift_state: ai_skill_core::DriftState::Unknown,
                        })
                        .collect();
                    skills.extend(plugin_skill_objs);
                }
                app.budget = calculate_budget(&skills);
                let names = skill_names(&skills);
                usage_report = build_usage_report(
                    &usage_reader.read_events().unwrap_or_default(),
                    &names,
                    stale_after_days,
                );
                app.all_skills = skills;
            }
            app.needs_refresh = false;
        }

        if app.should_quit {
            break;
        }
    }

    terminal::teardown(&mut term)?;
    Ok(())
}

fn load_cli_skills() -> Result<Vec<Skill>, Box<dyn std::error::Error>> {
    let config_store = config_store_from_env();
    let config = config_store.read().unwrap_or_default();
    let mut repo = FsSkillRepository::from_env()?;
    repo.add_custom_paths(config.custom_agent_paths.clone());
    let mut skills = list_skills_with_drift(&repo)?;
    if let Ok(discoverer) = FsPluginDiscoverer::from_env()
        && let Ok(plugin_skills) = discoverer.discover_skills()
    {
        let plugin_skill_objs: Vec<Skill> = plugin_skills
            .into_iter()
            .map(|ps| Skill {
                name: ps.name,
                path: ps.path,
                scope: Scope::Global,
                agents: vec![format!("Plugin ({})", ps.marketplace_key)],
                tags: vec![],
                managed: false,
                mode: SkillMode::Active,
                validation: ValidationState::Valid,
                manifest_content: ps.manifest_content,
                drift_state: ai_skill_core::DriftState::Unknown,
            })
            .collect();
        skills.extend(plugin_skill_objs);
    }
    Ok(skills)
}

/// Loads all skills and runs drift detection for each one.
fn list_skills_with_drift(
    repo: &FsSkillRepository,
) -> Result<Vec<Skill>, Box<dyn std::error::Error>> {
    let mut skills = repo.list()?;
    apply_drift(&mut skills, &GitDriftChecker);
    Ok(skills)
}

/// Mutates each skill's `drift_state` in place.
fn apply_drift(skills: &mut [Skill], drift_checker: &impl DriftChecker) {
    for skill in skills {
        skill.drift_state = drift_checker.check(&skill.path);
    }
}

/// Returns the distinct skill names from a slice of skills.
fn skill_names(skills: &[Skill]) -> Vec<String> {
    skills.iter().map(|s| s.name.clone()).collect()
}

/// Returns the user's home directory from `$HOME`.
fn home_dir() -> Result<PathBuf, std::io::Error> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME is not set; set HOME or run ai-skill from a login shell",
        )
    })
}

fn config_store_from_env() -> Box<dyn ConfigStore> {
    FsConfigStore::from_env()
        .ok()
        .map(|s| Box::new(s) as Box<dyn ConfigStore>)
        .unwrap_or_else(|| {
            let path = home_dir()
                .unwrap_or_default()
                .join(".config")
                .join("ai-skill")
                .join("config.json");
            Box::new(FsConfigStore::new(path)) as Box<dyn ConfigStore>
        })
}

/// Modes the binary can run in based on CLI argument parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliMode {
    /// Interactive TUI.
    Tui,
    /// Print a static string and exit (--help, --version).
    Print(&'static str),
    /// Emit structured JSON and exit.
    Json(JsonCommand),
    /// Emit Markdown and exit.
    Markdown(MarkdownCommand),
}

/// Subcommands available under `--json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonCommand {
    /// `--json list` — list all skills as JSON.
    List,
    /// `--json audit` — audit report as JSON.
    Audit,
}

/// Subcommands available under `--markdown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownCommand {
    /// `--markdown audit` — audit report as Markdown.
    Audit,
}

/// Parses CLI arguments into a [`CliMode`] action.
fn parse_cli_args(mut args: impl Iterator<Item = String>) -> Result<CliMode, String> {
    let Some(first) = args.next() else {
        return Ok(CliMode::Tui);
    };

    match first.as_str() {
        "-h" | "--help" => Ok(CliMode::Print(help_text())),
        "--version" | "-V" => Ok(CliMode::Print(version_text())),
        "--json" => match args.next().as_deref() {
            Some("list") => Ok(CliMode::Json(JsonCommand::List)),
            Some("audit") => Ok(CliMode::Json(JsonCommand::Audit)),
            Some(other) => Err(format!(
                "unknown --json operation '{other}'; expected 'list' or 'audit'"
            )),
            None => Err("missing --json operation; expected 'list' or 'audit'".to_string()),
        },
        "--markdown" => match args.next().as_deref() {
            Some("audit") => Ok(CliMode::Markdown(MarkdownCommand::Audit)),
            Some(other) => Err(format!(
                "unknown --markdown operation '{other}'; expected 'audit'"
            )),
            None => Err("missing --markdown operation; expected 'audit'".to_string()),
        },
        _ => Ok(CliMode::Tui),
    }
}

/// Emits JSON output for the given command to stdout.
fn print_json(command: JsonCommand, skills: &[Skill]) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        JsonCommand::List => serde_json::to_writer_pretty(std::io::stdout(), skills)?,
        JsonCommand::Audit => {
            let report = audit_skills(skills);
            let budget_warning = classify_budget(&report.budget);
            let usage = build_usage_report(
                &FsUsageHistoryReader::from_env()
                    .read_events()
                    .unwrap_or_default(),
                &skill_names(skills),
                stale_after_days_from_env(),
            );
            let json = JsonAuditReport {
                broken: report.broken,
                duplicates: report.duplicates,
                no_agents: report.no_agents,
                update_available: report.update_available,
                budget: report.budget,
                budget_warning,
                usage_dead: usage.dead,
                usage_stale: usage.stale,
                stale_after_days: usage.stale_after_days,
            };
            serde_json::to_writer_pretty(std::io::stdout(), &json)?;
        }
    }
    println!();
    Ok(())
}

/// Emits Markdown output for the given command to stdout.
fn print_markdown(command: MarkdownCommand, skills: &[Skill]) {
    match command {
        MarkdownCommand::Audit => print!("{}", render_audit_markdown(skills)),
    }
}

fn render_audit_markdown(skills: &[Skill]) -> String {
    let report = audit_skills(skills);
    let warning = classify_budget(&report.budget);
    let usage = build_usage_report(
        &FsUsageHistoryReader::from_env()
            .read_events()
            .unwrap_or_default(),
        &skill_names(skills),
        stale_after_days_from_env(),
    );
    let mut out = String::new();

    out.push_str("# ai-skill Health Report\n\n");
    out.push_str("## Summary\n\n");
    out.push_str("| Category | Count |\n");
    out.push_str("|---|---:|\n");
    out.push_str(&format!("| Total skills | {} |\n", skills.len()));
    out.push_str(&format!("| Broken | {} |\n", report.broken.len()));
    out.push_str(&format!("| Duplicates | {} |\n", report.duplicates.len()));
    out.push_str(&format!("| No agents | {} |\n", report.no_agents.len()));
    out.push_str(&format!(
        "| Updates available | {} |\n",
        report.update_available.len()
    ));
    out.push_str(&format!("| Dead (never used) | {} |\n", usage.dead.len()));
    out.push_str(&format!(
        "| Stale (> {}d) | {} |\n",
        usage.stale_after_days,
        usage.stale.len()
    ));

    out.push_str("\n## Context Budget\n\n");
    out.push_str(&format!("- Limit: {} chars\n", report.budget.limit));
    out.push_str(&format!("- Used: {} chars\n", report.budget.used));
    out.push_str(&format!("- Available: {} chars\n", report.budget.available));
    out.push_str(&format!(
        "- Usage: {:.1}%\n",
        report.budget.usage_ratio * 100.0
    ));
    out.push_str(&format!("- Warning: {}\n", budget_warning_label(&warning)));

    append_skill_section(&mut out, "Broken", &report.broken);
    append_skill_section(&mut out, "Duplicates", &report.duplicates);
    append_skill_section(&mut out, "No Agents", &report.no_agents);
    append_skill_section(&mut out, "Updates Available", &report.update_available);
    append_name_section(&mut out, "Dead (never used)", &usage.dead);
    append_name_section(
        &mut out,
        &format!("Stale (unused > {}d)", usage.stale_after_days),
        &usage.stale,
    );

    out
}

fn append_skill_section(out: &mut String, title: &str, skills: &[&Skill]) {
    out.push_str(&format!("\n## {title}\n\n"));
    if skills.is_empty() {
        out.push_str("No findings.\n");
        return;
    }

    out.push_str("| Skill | Scope | Agents | Path |\n");
    out.push_str("|---|---|---|---|\n");
    for skill in skills {
        let agents = if skill.agents.is_empty() {
            "-".to_string()
        } else {
            skill.agents.join(", ")
        };
        out.push_str(&format!(
            "| {} | {:?} | {} | `{}` |\n",
            escape_markdown_cell(&skill.name),
            skill.scope,
            escape_markdown_cell(&agents),
            skill.path.display()
        ));
    }
}

fn budget_warning_label(warning: &BudgetWarning) -> String {
    match warning {
        BudgetWarning::None => "none".to_string(),
        BudgetWarning::Approaching { pct } => format!("approaching ({pct:.1}%)"),
        BudgetWarning::Critical { pct } => format!("critical ({pct:.1}%)"),
        BudgetWarning::OverBudget {
            pct,
            truncated_skills,
        } => format!("over budget ({pct:.1}%, ~{truncated_skills} truncated skills)"),
    }
}

fn escape_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

/// Appends a simple bullet list of skill names under a heading.
fn append_name_section(out: &mut String, title: &str, names: &[String]) {
    out.push_str(&format!("\n## {title}\n\n"));
    if names.is_empty() {
        out.push_str("No findings.\n");
        return;
    }
    for name in names {
        out.push_str(&format!("- {}\n", escape_markdown_cell(name)));
    }
}

/// Reads the stale threshold from the user config, defaulting to 30 days.
fn stale_after_days_from_env() -> u64 {
    config_store_from_env()
        .read()
        .map(|c| c.stale_after_days)
        .unwrap_or_else(|_| ai_skill_core::config::default_stale_after_days())
}

#[derive(Serialize)]
struct JsonAuditReport<'a> {
    broken: Vec<&'a Skill>,
    duplicates: Vec<&'a Skill>,
    no_agents: Vec<&'a Skill>,
    update_available: Vec<&'a Skill>,
    budget: ai_skill_core::ContextBudget,
    budget_warning: BudgetWarning,
    usage_dead: Vec<String>,
    usage_stale: Vec<String>,
    stale_after_days: u64,
}

/// Returns the version string (`ai-skill X.Y.Z`).
fn version_text() -> &'static str {
    concat!("ai-skill ", env!("CARGO_PKG_VERSION"), "\n")
}

/// Returns the help usage text.
fn help_text() -> &'static str {
    "ai-skill - manage AI agent skills\n\nUsage:\n  ai-skill\n  ai-skill --help\n  ai-skill --version\n  ai-skill --json list\n  ai-skill --json audit\n  ai-skill --markdown audit\n"
}

#[cfg(test)]
mod cli_tests {
    use super::{CliMode, JsonCommand, MarkdownCommand, parse_cli_args, render_audit_markdown};
    use ai_skill_core::{DriftState, Scope, Skill, SkillMode, ValidationState};
    use std::path::PathBuf;

    #[test]
    fn version_flag_prints_binary_version() {
        let mode = parse_cli_args(["--version".to_string()].into_iter()).unwrap();

        assert!(matches!(mode, CliMode::Print(output) if output.starts_with("ai-skill ")));
    }

    #[test]
    fn help_flag_prints_usage() {
        let mode = parse_cli_args(["--help".to_string()].into_iter()).unwrap();

        assert!(matches!(mode, CliMode::Print(output) if output.contains("Usage:")));
    }

    #[test]
    fn unknown_flag_falls_through_to_tui() {
        assert_eq!(
            parse_cli_args(["--unknown".to_string()].into_iter()).unwrap(),
            CliMode::Tui
        );
    }

    #[test]
    fn json_list_mode_is_supported() {
        assert_eq!(
            parse_cli_args(["--json".to_string(), "list".to_string()].into_iter()).unwrap(),
            CliMode::Json(JsonCommand::List)
        );
    }

    #[test]
    fn json_audit_mode_is_supported() {
        assert_eq!(
            parse_cli_args(["--json".to_string(), "audit".to_string()].into_iter()).unwrap(),
            CliMode::Json(JsonCommand::Audit)
        );
    }

    #[test]
    fn markdown_audit_mode_is_supported() {
        assert_eq!(
            parse_cli_args(["--markdown".to_string(), "audit".to_string()].into_iter()).unwrap(),
            CliMode::Markdown(MarkdownCommand::Audit)
        );
    }

    #[test]
    fn json_requires_operation() {
        let err = parse_cli_args(["--json".to_string()].into_iter()).unwrap_err();

        assert!(err.contains("missing --json operation"));
    }

    #[test]
    fn markdown_requires_operation() {
        let err = parse_cli_args(["--markdown".to_string()].into_iter()).unwrap_err();

        assert!(err.contains("missing --markdown operation"));
    }

    #[test]
    fn markdown_audit_contains_summary_budget_and_findings() {
        let skills = vec![
            skill("broken", ValidationState::BrokenSymlink, vec!["claude"]),
            skill("lonely", ValidationState::Valid, vec![]),
        ];

        let report = render_audit_markdown(&skills);

        assert!(report.contains("# ai-skill Health Report"));
        assert!(report.contains("| Total skills | 2 |"));
        assert!(report.contains("## Context Budget"));
        assert!(report.contains("## Broken"));
        assert!(report.contains("broken"));
        assert!(report.contains("## No Agents"));
        assert!(report.contains("lonely"));
        assert!(report.contains("## Dead (never used)"));
        assert!(report.contains("## Stale (unused > "));
    }

    fn skill(name: &str, validation: ValidationState, agents: Vec<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            scope: Scope::Global,
            agents: agents.into_iter().map(str::to_string).collect(),
            tags: vec![],
            managed: true,
            mode: SkillMode::Active,
            validation,
            manifest_content: Some(format!("# {name}")),
            drift_state: DriftState::Unknown,
        }
    }
}
