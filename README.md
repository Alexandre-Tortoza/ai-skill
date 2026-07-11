# ai-skill

[![CI](https://github.com/alexmrtr/ai-skill/actions/workflows/ci.yml/badge.svg)](https://github.com/alexmrtr/ai-skill/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ai-skill.svg)](https://crates.io/crates/ai-skill)
[![License: AGPL-3.0-only](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](LICENSE)

Skills para agentes de IA tendem a se espalhar entre diretórios globais, projetos locais, symlinks e catálogos remotos. Depois de algumas instalações, fica difícil responder perguntas simples: o que está instalado, o que está quebrado, o que está duplicado, qual skill precisa de atualização e qual conteúdo merece revisão antes de ser executado por um agente.

`ai-skill` é uma TUI para inventariar, auditar e gerenciar skills de agentes como Claude Code e ferramentas compatíveis. O foco é dar visibilidade e controle antes de instalar, atualizar, desabilitar ou adotar skills.

## Estado

Projeto em estágio inicial. As ondas funcionais do roadmap já possuem implementação local, mas a superfície de distribuição ainda está em construção.

Releases por tag publicam arquivos compactados no GitHub Releases, pacotes DEB/RPM, crates no crates.io, Homebrew tap e pacotes AUR quando os secrets externos estiverem configurados.

## Instalação

### Script Com Curl

O instalador baixa o artefato da release `latest`, valida `SHA256SUMS` e copia o binário para `${BIN_DIR:-$HOME/.local/bin}`.

```sh
curl -fsSL https://raw.githubusercontent.com/alexmrtr/ai-skill/main/install.sh -o install.sh
sh install.sh
```

Para instalar em outro diretório:

```sh
BIN_DIR=/usr/local/bin sh install.sh
```

Para conferir o que seria feito sem baixar arquivos:

```sh
sh install.sh --dry-run
```

### GitHub Release

Baixe o arquivo da sua plataforma em `https://github.com/alexmrtr/ai-skill/releases/latest`:

- Linux x86_64: `ai-skill-x86_64-unknown-linux-gnu.tar.gz`
- macOS Intel: `ai-skill-x86_64-apple-darwin.tar.gz`
- macOS Apple Silicon: `ai-skill-aarch64-apple-darwin.tar.gz`
- Windows x86_64: `ai-skill-x86_64-pc-windows-msvc.zip`

Verifique o arquivo com `SHA256SUMS`, extraia e coloque o binário `ai-skill` em um diretório no seu `PATH`.

No macOS, os binários ainda não são assinados ou notarizados. Se o Gatekeeper bloquear a primeira execução, remova a quarentena do arquivo baixado depois de conferir a origem e o checksum:

```sh
xattr -d com.apple.quarantine ./ai-skill
```

No Windows, o binário ainda não é assinado. O SmartScreen pode alertar na primeira execução.

### Cargo

Depois que a versão estiver publicada no crates.io:

```sh
cargo install ai-skill
```

O pacote instala o binário `ai-skill`.

### Homebrew

Depois que o tap `alexmrtr/homebrew-tap` estiver criado e atualizado pela primeira release:

```sh
brew tap alexmrtr/tap
brew install ai-skill
```

### AUR

Depois que a primeira release publicar os pacotes AUR:

```sh
paru -S ai-skill-bin
```

O pacote source também fica disponível como `ai-skill`.

### DEB/RPM

Releases publicam pacotes Linux gerados via `nfpm` a partir do binário já compilado:

```sh
sudo dpkg -i ai-skill_*_amd64.deb
sudo rpm -i ai-skill-*.x86_64.rpm
```

### Mise

Com releases nomeadas no formato `vX.Y.Z`, o GitHub backend do `mise` consegue instalar direto do repositório:

```sh
mise use -g github:alexmrtr/ai-skill
```

Esse caminho depende dos assets publicados pelo workflow de release.

## Uso Local

```sh
cargo run -p ai-skill
```

Comandos úteis durante desenvolvimento:

```sh
bin/check
```

Se `cargo-audit` estiver instalado:

```sh
cargo audit
```

## O Que A TUI Faz

- Lista skills instaladas em escopos globais e de projeto.
- Mostra estado de validação, duplicatas, symlinks quebrados e manifests inválidos.
- Busca skills remotas via `npx skills find <termo>`.
- Exibe detalhes do `SKILL.md` e metadados.
- Instala, remove, atualiza, habilita, desabilita e adota skills.
- Filtra por escopo e tags.
- Gerencia profiles/presets.
- Executa scan heurístico de segurança antes de instalar.
- Detecta drift por hash local/upstream.
- Oferece editor de frontmatter, wizard de criação e relatório agregado de audit.

## Documentation

| Document | Description |
|---|---|
| [`docs/installation.md`](docs/installation.md) | Installation via pre-built binaries or from source |
| [`docs/usage.md`](docs/usage.md) | TUI user guide — views, key bindings, workflows |
| [`docs/architecture.md`](docs/architecture.md) | Hexagonal architecture, crate boundaries, design decisions |
| [`docs/development.md`](docs/development.md) | Developer setup, commands, methodology, CI/CD |
| [`docs/security.md`](docs/security.md) | Security model, heuristic scan, responsible disclosure |
| [`docs/api.md`](docs/api.md) | Public API surface of each crate |
| [`docs/roadmap.md`](docs/roadmap.md) | Product backlog (Waves 0–4) and repository health checklist |
| [`docs/features.md`](docs/features.md) | Future feature ideas (icebox), ordered by priority |
| [`docs/review-prompts.md`](docs/review-prompts.md) | LLM review prompts for PRs |

## Architecture

The project follows a hexagonal architecture in a Cargo workspace:

- `core`: pure domain, no I/O.
- `adapters`: filesystem, shell-out for catalog/installer, and watcher.
- `tui`: terminal interface with `ratatui` and `crossterm`.

See `docs/architecture.md` for details and decisions.

## Roadmap

The backlog and repository health checklist live in `docs/roadmap.md`. Ideas outside the current scope are in `docs/features.md`.

## Contributing

See `CONTRIBUTING.md` for setup, expected commands, and contribution workflow.

## Security

The tool handles content that can guide agents to execute commands. Security reports should follow `SECURITY.md`.

## License

AGPL-3.0-only. See `LICENSE`.
