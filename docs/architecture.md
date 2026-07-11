# Arquitetura

`ai-skill` usa arquitetura hexagonal para manter regras de domínio testáveis sem filesystem, terminal ou rede.

## Crates

### `core`

Domínio puro. Não faz I/O e não depende de `adapters` nem de `tui`.

Responsabilidades:

- Modelar `Skill`, `Profile`, metadados e estados de validação.
- Detectar duplicatas, drift, problemas de audit e achados de segurança.
- Definir ports para repositório, catálogo, installer, toggler, profiles e criação.
- Gerar conteúdo de `SKILL.md` a partir de dados de domínio.

Testes esperados: unitários rápidos, sem tocar filesystem real.

### `adapters`

Implementa ports do `core` usando I/O real.

Responsabilidades:

- Ler skills no filesystem.
- Resolver symlinks e markers de adoção.
- Persistir profiles.
- Chamar `npx skills` para busca, instalação, remoção e update.
- Observar mudanças com `notify`.
- Consultar git para drift detection.

Testes esperados: integração com diretórios temporários e fixtures.

### `tui`

Interface de terminal.

Responsabilidades:

- Renderizar painéis com `ratatui`.
- Receber eventos de teclado/terminal com `crossterm`.
- Coordenar casos de uso chamando `core` com adapters injetados.
- Preservar terminal alternativo e restaurar em panic.

Testes esperados: snapshots de render e testes de estado do app.

## Regra de Dependência

Fluxo permitido:

```text
tui -> core <- adapters
```

`core` não pode importar `adapters`, `tui`, `ratatui`, `crossterm`, `notify` ou qualquer detalhe de I/O. Se uma regra precisa de dados externos, modele um port em `core` e implemente em `adapters`.

## Decisão: Shell Out Para `npx skills`

A integração remota usa shell out para `npx skills` em vez de reimplementar o cliente nativo.

Motivos:

- Reduz acoplamento com APIs remotas instáveis.
- Reaproveita autenticação, cache e comportamento já mantidos pelo tooling upstream.
- Mantém o MVP focado em inventário, audit e UX local.

Consequências:

- Erros de processo precisam virar mensagens acionáveis.
- Testes devem isolar chamadas externas com adapters fake ou comandos controlados.
- Funcionalidades que exigem auth OIDC, como trending, ficam fora do escopo atual.

## Erros

Fronteiras devem retornar `Result`. Erros apresentados ao usuário devem indicar ação concreta quando possível, por exemplo: instalar dependência ausente, corrigir path, remover symlink quebrado ou revisar frontmatter inválido.

`unwrap()` é aceitável em testes. Em código de produção, só use quando a impossibilidade foi provada localmente e a alternativa pioraria a clareza.

## TUI

A TUI deve continuar usável em 80x24. Abaixo disso, deve orientar resize em vez de renderizar uma tela quebrada.

Também deve respeitar `NO_COLOR` e manter modo 16 cores legível. True color pode realçar, mas não pode ser requisito para entender a UI.

---

[← Voltar ao index](index.md) · Relacionadas: [Overview](architecture/overview.md) · [Crates](architecture/crates.md) · [Dependency Rule](architecture/dependency-rule.md) · [Decisions](architecture/decisions.md)
