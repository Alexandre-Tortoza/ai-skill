//! Binary entry point for `ai-skill`: TUI, `--json` subcommands, `--help`, `--version`.

mod app;
mod event;
mod terminal;
mod ui;

use ai_skill_adapters::{
    CliInstaller, FsProfileStore, FsSettingsStore, FsSkillCreator, FsSkillRepository,
    FsSkillWriter, FsToggler, FsWatcher, GitDriftChecker, NpxCatalogGateway,
};
use ai_skill_core::{
    DriftChecker, Skill, SkillRepository, audit_skills, calculate_budget, classify_budget,
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
            let repo = FsSkillRepository::from_env()?;
            let skills = list_skills_with_drift(&repo)?;
            print_json(command, &skills)?;
            return Ok(());
        }
        CliMode::Tui => {}
    }

    terminal::install_panic_hook();

    let repo = FsSkillRepository::from_env()?;
    let drift_checker = GitDriftChecker;
    let skills = list_skills_with_drift(&repo)?;
    let skill_roots = vec![home_dir()?.join(".claude").join("skills")];
    let watcher = FsWatcher::new(&skill_roots).ok();

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
        NpxCatalogGateway,
        CliInstaller,
        FsToggler,
        Box::new(FsProfileStore::from_env()?),
        Box::new(FsSkillCreator::from_env()?),
        Box::new(FsSkillWriter),
        settings_store,
    );

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
                    ui::help_overlay::render_help_overlay(main_area, f);
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
                    ui::scan_report::render_scan_report(&app.scan_findings, main_area, f);
                }
                View::Profiles => {
                    ui::profiles_panel::render_profiles_panel(&app.profile_state, main_area, f);
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
                    ui::audit_panel::render_audit_panel(&app.all_skills, main_area, f);
                }
                View::Budget => {
                    ui::budget_panel::render_budget_panel(&app.budget, main_area, f);
                }
                View::Settings => {
                    if let Some(ref settings) = app.settings {
                        ui::settings_panel::render_settings_panel(
                            settings,
                            &app.settings_state,
                            main_area,
                            f,
                        );
                    }
                }
            }

            let warning = classify_budget(&app.budget);
            ui::status_bar::render_status_bar(&app.view, status_area, f, Some(&warning));
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
                app.budget = calculate_budget(&skills);
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

/// Returns the user's home directory from `$HOME`.
fn home_dir() -> Result<PathBuf, std::io::Error> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME is not set; set HOME or run ai-skill from a login shell",
        )
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
}

/// Subcommands available under `--json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonCommand {
    /// `--json list` — list all skills as JSON.
    List,
    /// `--json audit` — audit report as JSON.
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
        _ => Ok(CliMode::Tui),
    }
}

/// Emits JSON output for the given command to stdout.
fn print_json(command: JsonCommand, skills: &[Skill]) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        JsonCommand::List => serde_json::to_writer_pretty(std::io::stdout(), skills)?,
        JsonCommand::Audit => {
            let report = audit_skills(skills);
            let json = JsonAuditReport {
                broken: report.broken,
                duplicates: report.duplicates,
                no_agents: report.no_agents,
                update_available: report.update_available,
            };
            serde_json::to_writer_pretty(std::io::stdout(), &json)?;
        }
    }
    println!();
    Ok(())
}

#[derive(Serialize)]
struct JsonAuditReport<'a> {
    broken: Vec<&'a Skill>,
    duplicates: Vec<&'a Skill>,
    no_agents: Vec<&'a Skill>,
    update_available: Vec<&'a Skill>,
}

/// Returns the version string (`ai-skill X.Y.Z`).
fn version_text() -> &'static str {
    concat!("ai-skill ", env!("CARGO_PKG_VERSION"), "\n")
}

/// Returns the help usage text.
fn help_text() -> &'static str {
    "ai-skill - manage AI agent skills\n\nUsage:\n  ai-skill\n  ai-skill --help\n  ai-skill --version\n  ai-skill --json list\n  ai-skill --json audit\n"
}

#[cfg(test)]
mod cli_tests {
    use super::{CliMode, JsonCommand, parse_cli_args};

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
    fn json_requires_operation() {
        let err = parse_cli_args(["--json".to_string()].into_iter()).unwrap_err();

        assert!(err.contains("missing --json operation"));
    }
}
