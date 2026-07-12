# ai-skill — Features Futuras (Icebox)

> **Quando atacar:** somente **depois** de concluir as Ondas 0–4 do `backlog-e-saude-repo.md`.
> Nada aqui é MVP. Ordem interna é sugestão, não compromisso.
> Cada item vira épico/fatias TDD no momento em que for puxado — aqui fica só o "o quê" e o "por quê".

Legenda: **[MUST]** ataca dor top da pesquisa · **[DEP]** tem dependência bloqueante · **[R]** risco/esforço alto.

---

## Leva A — Contexto & controle (extensão direta do MVP)

- [x] **Medidor de budget de contexto** **[MUST]**
      Estima o custo do conjunto ativo de skills contra o teto de discovery (~2% / ~16k chars) e alerta quando skills serão truncadas silenciosamente. Ataca a dor nº1 (truncamento invisível).
- [x] **Estado `name-only` (4 estados)** **[MUST]**
      Além de on/off, permitir colapsar a skill só ao nome (esconde a description do budget sem desligar). Espelha `skillOverrides` do Claude Code 2.1. Complementa o medidor acima.
- [x] **Controle de auto-trigger por projeto**
      Editar `.claude/settings.json` (`autoTrigger: false`, whitelist/blacklist de skills) pela TUI, sem abrir o arquivo na mão.
- [x] **Presets por fase**
      Refinar profiles em presets init/dev/test/release, aplicáveis com uma tecla.

## Leva B — Qualidade autoral

- [x] **Linter de description**
      No editor, avisar sobre descriptions que causam falso disparo: múltiplos "and", ausência do padrão "Use when [contexto]. [o que faz].", colisão de nome com outra skill. Ataca a causa do falso-trigger, não o sintoma.
- [x] **Validação de estrutura no wizard**
      Checar layout (SKILL.md presente, frontmatter com `name`+`description`, sem arquivos proibidos) antes de salvar.

## Leva C — Segurança avançada (evolução do scan heurístico)

- [x] **Cross-reference com registry**
      No scan, cruzar a skill contra a base community do skills.sh (reputação, installs verificados) e sinalizar typosquat/impersonation.
- [x] **Import chain tracing** **[R]**
      Lê SKILL.md e scripts referenciados, constrói o grafo de dependências, scaneia cada arquivo com o scanner heurístico e mostra a cadeia de imports. Acessível pela tecla `i` no detail view.
- [x] **Integração com scanners externos** **[DEP]**
      Port `ExternalScanner` definido em `core`, adapter `NoopExternalScanner` (retorna vazio até a API estar disponível), integrado ao fluxo de scan no wizard de instalação. _Dep: a rota `/api/v1/skills/audit` exige auth OIDC._
- [x] **Verificação de assinatura (ed25519)** **[R]**
      Port `SignatureVerifier` definido em `core`, adapter `NoopSignatureVerifier` (retorna `NotSupported` até o ecossistema adotar assinatura). Integrado à estrutura do `App` com `#[allow(dead_code)]` — apto para uso futuro (OWASP AST01).

## Leva D — Colaboração & fleet

- [x] **Gerenciamento remoto via SSH** **[R]**
      Port `SshConnector` definido em `core`, adapter `SshCommandConnector` (shell-out para `ssh`), `NoopSshConnector`. TUI: `View::SshRemote`, tecla `R` no list view, `ssh_panel.rs` com lista de hosts + detalhes + skills remotas. Host padrão `local` (127.0.0.1) pré-configurado.
- [x] **Biblioteca git-backed + sync multi-device**
      Versionar a biblioteca de skills num repo git, com snapshots restauráveis e sync entre máquinas.
- [x] **Export/share de profile**
      Exportar profile como YAML pra commitar em dotfiles. Tecla `e` no profiles view salva `<name>.skill-profile.yaml` no diretório atual. Método `export` adicionado ao trait `ProfileStore`.
- [x] **Bundles**
      Conjuntos pré-definidos instaláveis de uma vez. Modelo + port `BundleStore` em `core`, adapter `FsBundleStore` (com 3 bundles seed: frontend/ops/release-prep). TUI: `View::Bundles`, tecla `b`, painel de listagem + detalhes + instalação em lote.

## Leva E — Descoberta ampliada

- [ ] **Trending** **[DEP]**
      Aba de trending/hot do leaderboard skills.sh. _Dep bloqueante: rota `/api/v1` exige token Vercel OIDC. Opções: self-host mirror (mastra-ai/skills-api) ou pedir API key oficial. Reavaliar quando decidido._
- [x] **Multi-source marketplace**
      `CompositeCatalogGateway` em `adapters` que agrega múltiplos `Box<dyn AnyCatalogGateway>` com dedup case-insensitive. Fontes falham graciosamente sem bloquear as demais. Atualmente configurado com `NpxCatalogGateway`; novas fontes só precisam implementar o trait `AnyCatalogGateway`.
- [x] **Suporte a mais agentes**
      Estender além de Claude Code: Cursor, Windsurf, Copilot, Codex, Gemini CLI, OpenCode, etc. — cada um com seu diretório/formato, detectados e adotados.
- [x] **Compat com plugin marketplace**
      Descobrir skills declaradas em `.claude-plugin/marketplace.json` / `plugin.json`.

## Leva F — UX & personalização

- [x] **Config persistida da TUI**
      Paths custom de agentes, tema, keymap, proxy — em `~/.config/ai-skill/`. (Hoje só temos cores/responsividade como débito transversal.)
- [ ] **Temas (base16) + keymap customizável**
      Definir cores por slot semântico; permitir rebind de teclas estilo Atuin.
- [ ] **Diff viewer visual** **[R]**
      Ver diff upstream com stage de hunks estilo lazygit, não só link externo.
- [ ] **i18n da TUI**
      Começar por pt-BR + en.

## Leva G — Analytics & manutenção

- [ ] **Uso & stale detection**
      Ler históricos locais dos agentes pra mostrar frequência de uso, skills nunca chamadas ("dead skills") e stale (sem uso há N dias). Inspirado no `skilled`.
- [ ] **Relatório de saúde exportável**
      Exportar o audit agregado (broken refs, duplicados, dead skills, budget) como markdown/JSON pra CI ou revisão periódica.
- [ ] **Hot-reload awareness**
      Integrar com o hot-reload nativo do Claude Code 2.1 (skills recarregam sem restart) e refletir isso no watch reativo.

---

[← Voltar ao index](index.md) · Relacionadas: [Roadmap](roadmap.md) · [Usage](usage.md)
