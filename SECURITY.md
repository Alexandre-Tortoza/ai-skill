# Security Policy

`ai-skill` lida com skills que podem influenciar agentes de IA a ler arquivos, executar comandos ou manipular ambiente local. Reports de segurança são tratados com prioridade.

## Como Reportar

Use GitHub Security Advisories quando estiver habilitado para este repositório.

Se advisories privados ainda não estiverem disponíveis, abra uma issue sem incluir payloads exploráveis, secrets, caminhos sensíveis ou instruções de exploração completas. Informe que você tem um report privado e aguarde um canal seguro.

## Escopo

Estão no escopo:

- Execução inesperada de comandos.
- Bypass do gate de scan antes de instalar.
- Leitura ou exposição indevida de arquivos locais.
- Tratamento inseguro de symlinks, paths ou profiles.
- Vulnerabilidades em parsing de `SKILL.md` ou frontmatter.

Fora de escopo:

- Falhas já corrigidas na branch principal.
- Alertas puramente teóricos sem impacto demonstrável.
- Problemas causados por execução manual deliberada de comandos inseguros fora do `ai-skill`.

## Expectativa

Não publique detalhes até que exista correção ou mitigação documentada.
