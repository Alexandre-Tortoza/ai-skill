# Contributing

Obrigado por contribuir com o `ai-skill`.

## Setup

Instale Rust pelo `rustup` ou pelo gerenciador de pacotes do sistema. O toolchain esperado está fixado em `rust-toolchain.toml`.

```sh
cargo build --workspace
```

## Checks Locais

Antes de abrir PR, rode:

```sh
bin/check
```

O script executa `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` e `cargo audit` quando `cargo-audit` estiver instalado.

Também é possível rodar os comandos manualmente:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
```

## Release

Pré-requisitos:

- `cargo-set-version` instalado para atualizar versões do workspace.
- Secret `CARGO_REGISTRY_TOKEN` configurado no GitHub para publicar no crates.io.
- Secret `HOMEBREW_TAP_TOKEN` configurado com permissão de push para `alexmrtr/homebrew-tap`.
- Repositório `alexmrtr/homebrew-tap` criado antes da primeira release com Homebrew.
- Seção da versão em `CHANGELOG.md` no formato `## [X.Y.Z] - YYYY-MM-DD`.

Fluxo esperado:

```sh
cargo set-version --workspace X.Y.Z
bin/release-prep vX.Y.Z
git add .
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
git push origin main vX.Y.Z
```

O workflow de release builda artefatos, publica os crates de forma idempotente, cria o GitHub Release com checksums e atualiza a fórmula `Formula/ai-skill.rb` no Homebrew tap.

## Método de Trabalho

- Use TDD estrito para mudanças funcionais: Red, Green, Refactor.
- Prefira fatias verticais pequenas e testáveis.
- Mantenha `core` livre de I/O.
- Não adicione abstrações para casos hipotéticos.
- Erros em fronteiras devem ser acionáveis para o usuário.

## Commits e PRs

- Faça commits pequenos e focados.
- Descreva o problema resolvido, não só os arquivos alterados.
- Inclua testes ou explique por que não cabem.
- Atualize `docs/roadmap.md` quando concluir uma história ou item de saúde.

## Revisão

Revisões devem priorizar bugs, regressões, riscos de segurança, falhas de arquitetura e testes ausentes. Não confie apenas na descrição do autor do PR; leia o diff.
