# ADR-0002 — Controle manual do versionamento Git

Data: 2026-10-26. Estado: aceito por determinação expressa do usuário na
[instrução complementar](../internal/spec/POLITICA_GIT_GITHUB_BACKUP.md).

## Contexto

O ambiente anterior do CAD-CRAFT sofreu uma falha grave, ocasionando perda do
código-fonte e do histórico Git local. O Prompt 0 reconstruiu o workspace e a
memória. A política complementar foi autorizada após sua conclusão.

## Decisão

O desenvolvimento será automatizado pelo Codex, mas todas as operações de
versionamento e sincronização permanecerão sob controle manual do usuário,
inclusive após a Etapa 15. A regra substitui as autorizações automáticas antigas
de staging, commit e sincronização, preservando as decisões arquiteturais existentes.

O Codex não executa add, commit, push, pull, fetch, merge, rebase, reset, revert,
cherry-pick, checkout, switch, exclusão de branches, clean, stash, tag ou equivalentes.
A vedação vale também para scripts, hooks, subagentes, ferramentas externas,
automações, interfaces gráficas e CI/CD. Não contornar por worktrees ou outras APIs.
Exceção somente mediante autorização explícita para uma operação específica,
sem efeito permanente sobre operações futuras. Consultas sem alteração são permitidas.

Bootstrap: commit/push manuais. Etapas 1–14: commit manual após cada etapa e
push manual nas pares. Etapa 15: commit/push manuais. Homologação posterior:
correções e novos commits manuais, com sincronização sob controle do usuário.

Armazenamento: preferir Git abaixo de 1 GB; 10 GB é referência operacional,
não limite rígido garantido. Avaliar arquivos acima de 50 MB; não adicionar
arquivos acima de 100 MB ao Git convencional. LFS depende de autorização futura.
Excluir do versionamento builds, caches, temporários, VMs, imagens de disco,
credenciais, logs extensos e backups completos; preservar recursos necessários.

## Justificativa

- Evitar alterações não autorizadas no histórico.
- Preservar rastreabilidade e reduzir riscos de sobrescrita.
- Garantir revisão antes de cada commit.
- Proteger o projeto por meio de sincronização remota periódica feita pelo usuário.

## Consequências

O Codex não executará commits nem push. Ao terminar, validará o trabalho,
atualizará documentação/memória, relatará arquivos e pendências e encerrará.
O usuário fará staging, versionamento, sincronização, resolução de conflitos e
manutenção, com orientação externa ao Codex.

O diagnóstico offline não confirma o estado remoto atual. GitHub protege somente
conteúdo efetivamente enviado; alterações não commitadas e commits não enviados
continuam sujeitos a perdas locais. Recomenda-se backup externo independente,
sem implementá-lo automaticamente nesta entrega. A política é processual; não
instala hooks nem configura bloqueios de permissão no GitHub.

Implementação e recuperação: [relatório](../internal/reports/POLITICA_GITHUB_BACKUP.md).

## Adendo — versionamento por etapa (2026-11-03)

Ao aprovar o Plano Mestre (Etapas 13–28), o usuário decidiu: "Eu faço após
seu OK" — o agente faz commit + push de cada etapa concluída somente depois do
OK expresso do usuário para aquela etapa. O OK não se estende a outras etapas.
Continuam proibidos pull, fetch, merge, rebase, reset, revert, cherry-pick,
checkout, switch, exclusão de branches, clean, stash, tag e push forçado;
recusa do remoto interrompe a operação e é informada. A tabela de frequência
por paridade das Etapas 1–15 fica substituída por essa regra. Detalhes em
`AGENTS.md` ("Versionamento por etapa").
