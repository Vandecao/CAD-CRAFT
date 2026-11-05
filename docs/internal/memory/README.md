# Memória persistente

Fonte de continuidade versionada no próprio Git, sem APIs externas.

| Documento | Responsabilidade |
|---|---|
| PROJECT_CONTEXT | Visão, público, tecnologias e escopo |
| ARCHITECTURE | Limites, contratos e dependências |
| CURRENT_STATE | Estado verificável da execução atual |
| ROADMAP | Etapas e seus estados, sem requisitos inventados |
| KNOWN_ISSUES | Problemas, evidência e critérios de resolução |
| DECISIONS | Índice de ADRs |
| HANDOFF | Retomada curta e próxima ação autorizável |
| adr/ | Decisões significativas e consequências |

Atualizar estado e handoff em toda entrega; problemas só se encerram após
verificação. Ler seletivamente conforme AGENTS.md.

Política permanente: ler a seção POLÍTICA DE GIT, GITHUB E BACKUP de AGENTS
antes de comandos Git. Instrução integral em `docs/internal/spec/POLITICA_GIT_GITHUB_BACKUP.md`;
decisão operacional no ADR-0002. Memória só está protegida no GitHub após
commit e push manuais efetivamente realizados pelo usuário.
