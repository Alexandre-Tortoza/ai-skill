# Prompts de Revisão

Use estes prompts para revisão assistida por LLM. Eles não substituem leitura humana do diff.

## Revisão Geral

```text
Revise este diff como reviewer senior. Priorize bugs, regressões, riscos de segurança, violações da arquitetura hexagonal e testes ausentes. Não resuma primeiro. Liste achados por severidade com arquivo e linha. Se não houver achados, diga isso e indique riscos residuais.
```

## Arquitetura

```text
Verifique se este diff preserva a regra de dependência: tui -> core <- adapters. O core deve continuar puro, sem filesystem, terminal, rede, processo externo ou dependências de UI. Aponte qualquer vazamento de primitivos ou acoplamento indevido.
```

## Segurança

```text
Revise este diff procurando execução inesperada de comandos, tratamento inseguro de paths/symlinks, exposição de secrets, parsing permissivo demais de frontmatter e bypass de gates de confirmação ou scan. Foque em impacto explorável.
```

## TUI

```text
Revise este diff de TUI procurando regressões em navegação por teclado, render em 80x24, restauração de terminal, snapshots frágeis, estados impossíveis e perda de feedback contextual ao usuário.

---

[← Voltar ao index](index.md) · Relacionadas: [Code of Conduct](../CODE_OF_CONDUCT.md) · [Contributing](../CONTRIBUTING.md)
```
