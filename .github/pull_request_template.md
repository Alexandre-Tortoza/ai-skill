## Problema

Descreva o problema ou risco que este PR resolve.

## Solução

Descreva a menor mudança relevante feita.

## Testes

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo audit` quando disponível

## Checklist de Revisão

- [ ] O diff foi lido diretamente, sem confiar apenas nesta descrição.
- [ ] Mudanças em `core` continuam sem I/O.
- [ ] Erros novos são acionáveis.
- [ ] O roadmap/docs foram atualizados quando necessário.
