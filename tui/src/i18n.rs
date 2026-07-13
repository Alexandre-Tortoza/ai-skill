//! Internationalization for the TUI.
//!
//! The UI currently ships with English (`en`, the default) and Brazilian
//! Portuguese (`pt-BR`). The active locale is resolved from
//! [`ai_skill_core::TuiConfig::locale`] via [`I18n::from_config`].
//!
//! Translations are kept inline (no external catalog) to stay dependency-free
//! and easy to scan. Each [`I18n`] method returns the localized string for the
//! active locale.

use crate::app::View;

/// Supported UI locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    /// English (default).
    #[default]
    En,
    /// Brazilian Portuguese.
    PtBr,
}

impl Locale {
    /// Parses a BCP-47-ish locale code into a [`Locale`].
    ///
    /// Returns `None` for unrecognized codes so callers can fall back to the
    /// default locale.
    pub fn parse(value: &str) -> Option<Locale> {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" | "en-us" | "english" => Some(Locale::En),
            "pt" | "pt-br" | "pt_br" | "ptbr" | "portuguese" => Some(Locale::PtBr),
            _ => None,
        }
    }
}

/// Active translation context for the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct I18n {
    locale: Locale,
}

impl I18n {
    /// Builds an [`I18n`] for the given locale.
    pub fn new(locale: Locale) -> Self {
        I18n { locale }
    }

    /// Resolves the locale from a config value, falling back to English.
    pub fn from_config(value: Option<&str>) -> Self {
        match value.and_then(Locale::parse) {
            Some(locale) => I18n::new(locale),
            None => I18n::default(),
        }
    }

    // --- Help overlay ---------------------------------------------------

    pub fn help_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Help — Key Bindings",
            Locale::PtBr => "Ajuda — Atalhos",
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self.locale {
            Locale::En => HELP_EN,
            Locale::PtBr => HELP_PT_BR,
        }
    }

    // --- Status bar -----------------------------------------------------

    /// Returns the per-view key hint line.
    pub fn status_hint(&self, view: &View) -> &'static str {
        match (self.locale, *view) {
            (Locale::En, View::List) => {
                "j/k  d  e  n  r  u  a  c  A aud  B bud  S set  s srch  F1-F4  ? quit"
            }
            (Locale::En, View::Detail) => "j/k scroll  Esc back  q quit",
            (Locale::En, View::Search) => "type search  j/k move  Enter install  Esc back",
            (Locale::En, View::Help) => "Esc close",
            (Locale::En, View::Confirm) => "y confirm  n / Esc cancel",
            (Locale::En, View::InstallWizard) => "Tab scope  Space agent  Enter confirm  Esc back",
            (Locale::En, View::ScanReport) => "Enter install anyway  Esc cancel",
            (Locale::En, View::Profiles) => {
                "j/k move  a activate  e export  f from-current  d delete  Esc back"
            }
            (Locale::En, View::CreateWizard) => {
                "Tab next field  Enter create (on Preview)  Esc cancel"
            }
            (Locale::En, View::Editor) => "Tab next field  Enter save  Esc cancel",
            (Locale::En, View::Audit) => "Esc back",
            (Locale::En, View::Budget) => "Esc back",
            (Locale::En, View::Settings) => {
                "t toggle  j/k move  o toggle override  d remove  Esc save & back"
            }
            (Locale::En, View::ImportChain) => "Esc close",
            (Locale::En, View::SshRemote) => "j/k move  Enter connect  Esc back",
            (Locale::En, View::Bundles) => "j/k move  Enter install  Esc back",
            (Locale::En, View::Sync) => {
                "j/k move  Enter init/snap  r rstor  R remote  p push  P pull  Esc back"
            }
            (Locale::PtBr, View::List) => {
                "j/k  d  e  n  r  u  a  c  A aud  B bud  S cfg  s busca  F1-F4  ? sair"
            }
            (Locale::PtBr, View::Detail) => "j/k rolar  Esc voltar  q sair",
            (Locale::PtBr, View::Search) => "digite busca  j/k mover  Enter instalar  Esc voltar",
            (Locale::PtBr, View::Help) => "Esc fechar",
            (Locale::PtBr, View::Confirm) => "s confirmar  n / Esc cancelar",
            (Locale::PtBr, View::InstallWizard) => {
                "Tab escopo  Esp agente  Enter confirmar  Esc voltar"
            }
            (Locale::PtBr, View::ScanReport) => "Enter instalar mesmo assim  Esc cancelar",
            (Locale::PtBr, View::Profiles) => {
                "j/k mover  a ativar  e exportar  f do-atual  d apagar  Esc voltar"
            }
            (Locale::PtBr, View::CreateWizard) => {
                "Tab prox campo  Enter criar (em Preview)  Esc cancelar"
            }
            (Locale::PtBr, View::Editor) => "Tab prox campo  Enter salvar  Esc cancelar",
            (Locale::PtBr, View::Audit) => "Esc voltar",
            (Locale::PtBr, View::Budget) => "Esc voltar",
            (Locale::PtBr, View::Settings) => {
                "t alternar  j/k mover  o alternar override  d remover  Esc salvar & voltar"
            }
            (Locale::PtBr, View::ImportChain) => "Esc fechar",
            (Locale::PtBr, View::SshRemote) => "j/k mover  Enter conectar  Esc voltar",
            (Locale::PtBr, View::Bundles) => "j/k mover  Enter instalar  Esc voltar",
            (Locale::PtBr, View::Sync) => {
                "j/k mover  Enter init/snap  r restaurar  R remoto  p push  P pull  Esc voltar"
            }
        }
    }

    /// Indicator shown when the hot-reload watcher is active.
    pub fn reload_indicator(&self) -> &'static str {
        match self.locale {
            Locale::En => "  reload:on",
            Locale::PtBr => "  recarregar:on",
        }
    }

    // --- Audit panel ----------------------------------------------------

    pub fn audit_title(&self) -> &'static str {
        match self.locale {
            Locale::En => "Audit Report",
            Locale::PtBr => "Relatório de Auditoria",
        }
    }

    pub fn audit_summary(
        &self,
        broken: usize,
        duplicates: usize,
        no_agents: usize,
        updates: usize,
        dead: usize,
        stale: usize,
    ) -> String {
        match self.locale {
            Locale::En => format!(
                "broken: {}  duplicates: {}  no-agents: {}  updates: {}  dead: {}  stale: {}",
                broken, duplicates, no_agents, updates, dead, stale
            ),
            Locale::PtBr => format!(
                "quebrados: {}  duplicados: {}  sem-agentes: {}  atualizações: {}  mortos: {}  obsoletos: {}",
                broken, duplicates, no_agents, updates, dead, stale
            ),
        }
    }

    pub fn cat_broken(&self) -> &'static str {
        match self.locale {
            Locale::En => "Broken",
            Locale::PtBr => "Quebrados",
        }
    }

    pub fn cat_duplicates(&self) -> &'static str {
        match self.locale {
            Locale::En => "Duplicates",
            Locale::PtBr => "Duplicados",
        }
    }

    pub fn cat_no_agents(&self) -> &'static str {
        match self.locale {
            Locale::En => "No Agents",
            Locale::PtBr => "Sem Agentes",
        }
    }

    pub fn cat_updates(&self) -> &'static str {
        match self.locale {
            Locale::En => "Updates",
            Locale::PtBr => "Atualizações",
        }
    }

    pub fn cat_dead(&self) -> &'static str {
        match self.locale {
            Locale::En => "Dead",
            Locale::PtBr => "Mortos",
        }
    }

    pub fn cat_stale(&self) -> &'static str {
        match self.locale {
            Locale::En => "Stale",
            Locale::PtBr => "Obsoletos",
        }
    }

    pub fn usage_dead_title(&self, days: u64) -> String {
        format!("{} (>{days}d)", self.cat_dead())
    }

    pub fn usage_stale_title(&self, days: u64) -> String {
        format!("{} (>{days}d)", self.cat_stale())
    }

    // --- Scan report ----------------------------------------------------

    pub fn scan_title(&self) -> &'static str {
        match self.locale {
            Locale::En => " Security Findings ",
            Locale::PtBr => " Achados de Segurança ",
        }
    }

    pub fn scan_footer(&self) -> &'static str {
        match self.locale {
            Locale::En => "Enter to install anyway  |  Esc to cancel",
            Locale::PtBr => "Enter para instalar mesmo assim  |  Esc para cancelar",
        }
    }

    pub fn severity_high(&self) -> &'static str {
        match self.locale {
            Locale::En => "[HIGH]",
            Locale::PtBr => "[ALTO]",
        }
    }

    pub fn severity_medium(&self) -> &'static str {
        match self.locale {
            Locale::En => "[MED ]",
            Locale::PtBr => "[MÉD ]",
        }
    }

    // --- Settings: config panel ----------------------------------------

    pub fn config_header(&self) -> &'static str {
        match self.locale {
            Locale::En => " TUI Configuration ",
            Locale::PtBr => " Configuração da TUI ",
        }
    }

    pub fn config_proxy_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Proxy: ",
            Locale::PtBr => "Proxy: ",
        }
    }

    pub fn config_proxy_unset(&self) -> &'static str {
        match self.locale {
            Locale::En => "(not set)",
            Locale::PtBr => "(não definido)",
        }
    }

    pub fn config_custom_paths_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Custom agent paths:",
            Locale::PtBr => "Caminhos de agentes:",
        }
    }

    pub fn config_custom_paths_none(&self) -> &'static str {
        match self.locale {
            Locale::En => "Custom agent paths: (none)",
            Locale::PtBr => "Caminhos de agentes: (nenhum)",
        }
    }

    pub fn config_theme_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Theme overrides:",
            Locale::PtBr => "Sobrescritas de tema:",
        }
    }

    pub fn config_keymap_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Keymap overrides:",
            Locale::PtBr => "Sobrescritas de atalhos:",
        }
    }

    pub fn config_keymap_none(&self) -> &'static str {
        match self.locale {
            Locale::En => "Keymap overrides: (none)",
            Locale::PtBr => "Sobrescritas de atalhos: (nenhum)",
        }
    }

    pub fn config_current_title(&self) -> &'static str {
        match self.locale {
            Locale::En => " Current Config ",
            Locale::PtBr => " Configuração Atual ",
        }
    }

    pub fn config_message_title(&self) -> &'static str {
        match self.locale {
            Locale::En => " Message ",
            Locale::PtBr => " Mensagem ",
        }
    }

    pub fn config_edit_hint(&self) -> &'static str {
        match self.locale {
            Locale::En => " Edit ~/.config/ai-skill/config.json to change settings ",
            Locale::PtBr => " Edite ~/.config/ai-skill/config.json para mudar as configurações ",
        }
    }

    pub fn config_path_arrow(&self) -> &'static str {
        match self.locale {
            Locale::En => " -> ",
            Locale::PtBr => " -> ",
        }
    }

    // --- Settings: settings panel ---------------------------------------

    pub fn settings_project_label(&self) -> &'static str {
        match self.locale {
            Locale::En => "Project settings: ",
            Locale::PtBr => "Configurações do projeto: ",
        }
    }

    pub fn settings_project_none(&self) -> &'static str {
        match self.locale {
            Locale::En => "(no project)",
            Locale::PtBr => "(sem projeto)",
        }
    }

    pub fn settings_project_title(&self) -> &'static str {
        match self.locale {
            Locale::En => " Project Settings ",
            Locale::PtBr => " Configurações do Projeto ",
        }
    }

    pub fn settings_global_auto_trigger(&self) -> &'static str {
        match self.locale {
            Locale::En => "Global auto-trigger: ",
            Locale::PtBr => "Auto-disparo global: ",
        }
    }

    pub fn settings_toggle_hint(&self) -> &'static str {
        match self.locale {
            Locale::En => "    [t] toggle",
            Locale::PtBr => "    [t] alternar",
        }
    }

    pub fn settings_auto_trigger_title(&self) -> &'static str {
        match self.locale {
            Locale::En => " Auto-Trigger ",
            Locale::PtBr => " Auto-Disparo ",
        }
    }

    pub fn settings_no_overrides(&self) -> &'static str {
        match self.locale {
            Locale::En => " No skill overrides. Press [a] to add selected skill.",
            Locale::PtBr => " Sem sobrescritas de skill. Pressione [a] para adicionar skill.",
        }
    }

    pub fn settings_overrides_title(&self) -> &'static str {
        match self.locale {
            Locale::En => " Skill Overrides ",
            Locale::PtBr => " Sobrescritas de Skill ",
        }
    }

    pub fn settings_override_toggle(&self) -> &'static str {
        match self.locale {
            Locale::En => "  [o] toggle  [d] remove",
            Locale::PtBr => "  [o] alternar  [d] remover",
        }
    }
}

const HELP_EN: &str = "\
j / ↓       move down
k / ↑       move up
Tab         cycle scope filter (all → global → project)
Enter       open skill detail
s           open catalog search
S           git sync (snapshots / push / pull)
R           SSH remote management
b           bundles (predefined skill sets)
p           profiles / presets
F1-F4       apply phase preset (init/dev/test/release)
?           show this help
Esc         go back / close
q           quit
Ctrl-C      quit

--- in detail view ---
o           toggle skill auto-trigger

--- in settings view ---
t           toggle global auto-trigger
j/k         move in overrides list
o           toggle override auto-trigger
d           remove override
Esc         save & back";

const HELP_PT_BR: &str = "\
j / ↓       mover para baixo
k / ↑       mover para cima
Tab         alternar filtro de escopo (todos → global → projeto)
Enter       abrir detalhe da skill
s           abrir busca no catálogo
S           sincronizar git (snapshots / push / pull)
R           gerenciar remoto SSH
b           bundles (conjuntos predefinidos de skills)
p           perfis / predefinições
F1-F4       aplicar predefinição de fase (init/dev/test/release)
?           mostrar esta ajuda
Esc         voltar / fechar
q           sair
Ctrl-C      sair

--- na view de detalhe ---
o           alternar auto-disparo da skill

--- na view de configurações ---
t           alternar auto-disparo global
j/k         mover na lista de sobrescritas
o           alternar auto-disparo da sobrescrita
d           remover sobrescrita
Esc         salvar & voltar";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_locale_is_english() {
        assert_eq!(I18n::default().locale, Locale::En);
    }

    #[test]
    fn locale_parse_accepts_en_and_pt_br() {
        assert_eq!(Locale::parse("en"), Some(Locale::En));
        assert_eq!(Locale::parse("pt-BR"), Some(Locale::PtBr));
        assert_eq!(Locale::parse("PT-br"), Some(Locale::PtBr));
        assert_eq!(Locale::parse("portuguese"), Some(Locale::PtBr));
    }

    #[test]
    fn locale_parse_rejects_unknown() {
        assert_eq!(Locale::parse("fr"), None);
        assert_eq!(Locale::parse(""), None);
    }

    #[test]
    fn from_config_falls_back_to_english() {
        assert_eq!(I18n::from_config(None).locale, Locale::En);
        assert_eq!(I18n::from_config(Some("xx")).locale, Locale::default());
    }

    #[test]
    fn from_config_resolves_pt_br() {
        assert_eq!(I18n::from_config(Some("pt-BR")).locale, Locale::PtBr);
    }

    #[test]
    fn help_title_changes_with_locale() {
        assert_eq!(I18n::default().help_title(), "Help — Key Bindings");
        assert_eq!(I18n::new(Locale::PtBr).help_title(), "Ajuda — Atalhos");
    }

    #[test]
    fn audit_summary_is_localized() {
        let pt = I18n::new(Locale::PtBr);
        let summary = pt.audit_summary(1, 2, 3, 4, 5, 6);
        assert!(summary.contains("quebrados: 1"));
        assert!(summary.contains("obsoletos: 6"));
        let en = I18n::default();
        assert!(en.audit_summary(1, 0, 0, 0, 0, 0).contains("broken: 1"));
    }

    #[test]
    fn status_hint_is_localized_per_view() {
        let pt = I18n::new(Locale::PtBr);
        assert!(pt.status_hint(&View::List).contains("sair"));
        assert!(pt.status_hint(&View::Detail).contains("voltar"));
        let en = I18n::default();
        assert!(en.status_hint(&View::List).contains("quit"));
        assert!(en.status_hint(&View::Detail).contains("back"));
    }
}
