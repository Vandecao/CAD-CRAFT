# Instalação e ativação do ai-memory — 2026-11-02

## Escopo e fonte

Pedido do usuário: instalar e ativar o [ai-memory](https://github.com/akitaonrails/ai-memory).
Foi usada a [instalação nativa do Windows e configuração do Codex](https://github.com/akitaonrails/ai-memory/blob/main/INSTALL_FOR_AGENTS.md),
somente para o usuário local. Nenhum arquivo de domínio da Etapa 9 foi alterado
por esta intervenção. A etapa permanece em andamento, sem homologação.

## Instalação

- O instalador `install.ps1` foi baixado da release oficial e conferido contra
  `install.ps1.sha256`: SHA-256
  `6B105E4C8E6BCB3A2561A5316615A5C577B4074DF24A1414EA45D306C1C7B89A`.
  Seu conteúdo foi inspecionado antes da execução; ele também verificou a soma
  SHA-256 do pacote Windows.
- Instalado `ai-memory 3.3.1` em
  `%LOCALAPPDATA%\Programs\ai-memory`; diretório incluído no PATH do usuário.
- `ai-memory setup --yes --agent codex` criou os dados em
  `%LOCALAPPDATA%\ai-memory`, iniciou o serviço local na porta 49374 e criou
  a entrada de inicialização no login em `HKCU\...\Run\ai-memory`.
- O setup configurou MCP em `%USERPROFILE%\.codex\config.toml`, hooks em
  `%USERPROFILE%\.codex\hooks.json` e seis Agent Skills em
  `%USERPROFILE%\.agents\skills`. O backup da configuração Codex foi criado
  pelo instalador. Os sete hooks foram confiados no diálogo oficial do Codex.
- A variável de usuário `AI_MEMORY_AUTH_TOKEN` foi configurada a partir do
  arquivo local protegido `auth-tokens/default`, sem registrar seu valor neste
  relatório ou no repositório. Reiniciar o Codex desktop para ele herdar a
  variável, o MCP e os hooks novos.

## Verificação

- `ai-memory setup --plan --agent codex`: plano de dados, serviço e integração
  confirmado antes de aplicar.
- `ai-memory setup --yes --agent codex`: exit 0; `/healthz` e `memory_status`
  responderam `ok`.
- `codex mcp list`: `ai-memory` habilitado, com autenticação por variável.
- Teste `codex exec --ephemeral -s read-only` com resposta `OK`, sem ações em
  arquivos: `SessionStart`, `UserPromptSubmit` e `Stop` executados com sucesso.
- `ai-memory status`: servidor e cliente 3.3.1; 1 sessão, 4 observações,
  1 página; 0 itens pendentes no spool; LLM e embeddings desabilitados.
- `ai-memory doctor`: 1 sessão Codex capturada para `default/CAD-CRAFT`,
  cobertura satisfatória e nenhum hook sem confiança. O diagnóstico ainda
  informa marcador de projeto ausente e usa o escopo derivado do checkout.

O wiki Git do ai-memory fica exclusivamente no diretório de dados externo ao
CAD-CRAFT. O ai-memory não cria arquivos no checkout por padrão. Não foi
configurado remoto nem sincronização para esse wiki. Isso não constitui backup
do projeto, nem altera a política de commit/push manuais do CAD-CRAFT.

## Validação do checkout

`scripts/validate.ps1` foi executado e parou em `cargo fmt --check` (exit 1),
com diferenças de formatação nos arquivos Rust da Etapa 9 em andamento. As
demais verificações foram executadas separadamente, sem modificar o código:

| Comando | Resultado |
|---|---|
| `cargo check --workspace --all-targets --locked` | exit 101; código GUI parcial da Etapa 9, inclusive referência a `egui` não resolvida |
| `cargo test --workspace --locked` | exit 101; compilação GUI incompleta, sem contagem de testes Rust desta execução |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 101; `large_enum_variant` em `MarkerEdit` da Etapa 9 |
| `pwsh -NoProfile -File scripts/tests/check-repository.Tests.ps1` | 15 testes operacionais aprovados, 0 falhas |
| `git diff --check` | exit 0 |

Logs em `.local/ai-memory-validation/` (ignorados pelo Git). Essas falhas
pertencem à implementação parcial da Etapa 9 que já constava no status antes
da instalação; não foram corrigidas nesta tarefa ambiental.

## Git e inventário desta intervenção

Consulta de 2026-11-02, America/Sao_Paulo: branch `main`, HEAD local
`5377559ff3b5bd5ce1cd9d19265833ef3714796e`; origin existente
`https://github.com/Vandecao/CAD-CRAFT.git`. A consulta read-only
`git ls-remote origin refs/heads/main` falhou por DNS (`Could not resolve
host: github.com`); portanto o estado atual remoto não foi verificado.
A árvore já continha alterações da Etapa 9 antes desta
intervenção e continua não commitada. Nenhum Git mutável foi executado no
CAD-CRAFT.

Arquivos criados no checkout por esta tarefa: `docs/internal/plans/2026-11-02-ai-memory.md`
e este relatório. Modificados: `docs/internal/memory/CURRENT_STATE.md`,
`HANDOFF.md`, `KNOWN_ISSUES.md` e `DECISIONS.md`. Excluídos: nenhum. A
instalação também criou arquivos externos sob `%LOCALAPPDATA%`, `.codex` e
`.agents`, conforme descrito acima.

## Limites e continuidade

A sessão Codex desktop já aberta precisa ser reiniciada para carregar a nova
configuração. A captura foi comprovada na CLI nativa; não se declarou teste da
integração dentro desta sessão desktop já aberta. A memória contém apenas o
teste curto; não houve importação retroativa do histórico do projeto.

Consultar `ai-memory status` e `ai-memory doctor` em sessão nova se houver
dúvida de operação. Não iniciar a Etapa 10. A Etapa 9 continua pendente das
próprias validações e revisão. Commit/push/backup do CAD-CRAFT são exclusivos
do usuário.
