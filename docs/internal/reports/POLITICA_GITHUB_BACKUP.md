# CAD-CRAFT — Política de Git, GitHub, armazenamento e backup

Data: 2026-10-26, America/Sao_Paulo. Escopo: instrução complementar após o
Prompt 0. Etapa 1 não iniciada. Nenhuma homologação manual declarada.

## Política permanente

**O Codex desenvolve, testa, documenta e registra a memória. O usuário controla
exclusivamente o versionamento e a sincronização Git**, inclusive após a Etapa 15.
Política integral preservada em [docs/spec](../spec/POLITICA_GIT_GITHUB_BACKUP.md),
normas de retomada em [AGENTS](../../../AGENTS.md), decisão em
[ADR-0002](../../adr/0002-controle-manual-git.md).

O Codex não realiza staging, commit, push, pull, fetch, merge, rebase, reset,
revert, cherry-pick, checkout, switch, exclusão de branches, clean, stash, tag
ou equivalentes por scripts, hooks, subagentes, interfaces, automações ou CI/CD.
Exceção somente por autorização explícita para uma operação específica, sem
permissão permanente. As consultas permitidas não alteram arquivos/refs/histórico.
Não foram instalados hooks, mecanismos de bloqueio ou automações de Git.

| Evento | Procedimento exclusivo do usuário |
|---|---|
| Bootstrap e política complementar | Commit + push manuais |
| Etapas 1, 3, 5, 7, 9, 11, 13 | Commit manual |
| Etapas 2, 4, 6, 8, 10, 12, 14 | Commit + push manuais |
| Etapa 15 | Commit + push manuais |
| Homologação após Etapa 15 | Correções e novos commits manuais; sincronização manual |

Ao concluir uma entrega, o Codex testa, corrige, atualiza documentação/memória,
relata arquivos criados/modificados/excluídos, informa necessidade de commit/push
e encerra. O usuário revisa e executa as operações com orientação externa ao Codex.

## Armazenamento e ignore

Manter Git preferencialmente abaixo de 1 GB. 10 GB é uma referência operacional,
não limite rígido garantido: a documentação oficial distingue tamanho recomendado
e limites efetivamente impostos. Ver [limites de repositórios do GitHub](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits).

Avaliar arquivos acima de 50 MB antes de commit; acima de 100 MB não adicionar
ao Git convencional. O script usa MB decimais (50.000.000 / 100.000.000 bytes),
limiares conservadores em relação aos 50/100 MiB documentados pelo GitHub.
Git LFS só poderá ser adotado com autorização expressa futura.
Ver [arquivos grandes no GitHub](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github).

Não versionar builds, caches, temporários, VMs, imagens de disco, credenciais,
tokens, chaves privadas, logs extensos ou backups completos no próprio projeto.
O `.gitignore` foi ampliado para targets aninhados, artefatos de compilação,
temporários, IDEs, caches Python e imagens de VM/disco. Preservadas as regras
existentes e a exceção `.env.example`; nenhum executável/biblioteca foi ignorado
globalmente. Nenhum arquivo foi excluído.

Os arquivos rastreados foram confrontados com as regras efetivas. Cargo.toml,
Cargo.lock, AGENTS, README, docs/spec, docs/memory, docs/reports, código Rust,
testes, scripts, configurações e exemplos de recursos continuam versionáveis.
Ignore não remove arquivos que já tenham sido rastreados; qualquer correção
futura do índice/histórico cabe ao usuário.

## Diagnóstico offline

Executar `pwsh -NoProfile -File scripts/check-repository.ps1`; aceita
`-RepositoryPath` para uma árvore de trabalho existente. Requer PowerShell 7 e Git.

O script identifica branch, HEAD, tracking, ahead/behind local, tamanhos do índice
e da árvore de trabalho, temporários rastreados e nomes potencialmente sensíveis.
Verifica seções obrigatórias de AGENTS, Cargo.lock, docs/memory e comportamento
do ignore. Exibe armazenamento de objetos por `count-objects`; esse número não
é uma medição completa de todos os metadados da pasta `.git`.

Sem acesso à rede, exibe sempre **ESTADO REMOTO NÃO VERIFICADO**. Não confundir
0 commits à frente com confirmação atual do GitHub. Não há exclusão, correção,
limpeza nem alteração de referências. Locks opcionais e busca tardia de objetos
ausentes ficam desativados no subprocesso Git.

Retornos: 0 = nenhum achado nas regras auditadas; 1 = revisão necessária;
2 = diagnóstico incompleto/falha. Nomes suspeitos são reportados sem conteúdo;
mensagens brutas de erro Git não são reproduzidas. Caminhos exibidos são escapados.

Limitações: triagem por nomes, não detector completo de segredos; não varre arquivos
ignorados nem todo o histórico. Não percorre destinos de links/submódulos.
AGENTS recebe verificação estrutural, não prova semântica de todas as instruções.
Os alertas não aplicam modificações nem bloqueiam operações manuais do usuário.

## Recuperação após perda de máquina ou VM

GitHub protege **somente conteúdo efetivamente enviado**. Arquivos não versionados,
alterações sem commit e commits sem push continuam sujeitos a perdas locais.
Se estiverem ausentes tanto do remoto quanto de uma cópia externa sobrevivente,
GitHub não poderá recuperá-los.

1. Preservar discos, clones e backups sobreviventes sem sobrescrevê-los. O usuário
   consulta o remoto oficial e escolhe uma pasta nova para recuperação.
2. Instalar, por fontes oficiais, Git, PowerShell 7, Rustup e Microsoft C++ Build
   Tools com Desktop C++ e Windows SDK; VS Code é opcional. Autenticar no GitHub
   pelo fluxo oficial se necessário; não inserir/copiar tokens em arquivos do projeto.
3. **Somente o usuário** executa clone do repositório oficial numa pasta nova:

   ```powershell
   git clone https://github.com/Vandecao/CAD-CRAFT.git C:\Dev\CAD-CRAFT-restaurado
   ```

   Este comando é documentação de recuperação, não autorização de execução pelo
   Codex. Não foi executado nesta entrega. Não criar outro repositório remoto.
4. Ler AGENTS, HANDOFF e CURRENT_STATE do clone recuperado. Confirmar origin,
   branch, histórico e último SHA recebido antes de retomar qualquer trabalho.
   A memória recuperada é a versão efetivamente enviada ao remoto.
5. Na pasta restaurada, reconstruir o ambiente conforme a toolchain versionada:

   ```powershell
   rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy
   pwsh -NoProfile -File scripts/validate.ps1
   pwsh -NoProfile -File scripts/check-repository.ps1
   ```

   Cargo resolve/obtém dependências pelo `Cargo.lock` com `--locked`. Builds
   requerem a toolchain MSVC/SDK compatível; a primeira obtenção de ferramentas
   e dependências pode precisar de rede. O diagnóstico, isoladamente, não precisa.
6. Verificar integridade com `git fsck --full` (sem `--lost-found`), `git log`,
   `git --no-optional-locks status -sb` e os testes. Comparar SHAs com os registros
   disponíveis; somente confirmar recuperação após observar os resultados reais.
7. Para recuperar uma versão anterior, o usuário escolhe um commit verificado,
   preserva a cópia atual e restaura em outra pasta. Qualquer checkout, revert,
   integração, resolução de conflito ou sincronização é manual, sob seu controle.
   Validar novamente antes de continuar. Não usar reset destrutivo ou force push.
8. Incorporar arquivos de backup não versionados somente após revisão pelo usuário,
   repetir validações, atualizar memória e confirmar a etapa expressamente autorizada.

Recomenda-se backup externo independente, em outro dispositivo/serviço sob controle
do usuário, com versões datadas e teste de restauração. Deve cobrir as mudanças
locais necessárias e memória ainda não enviada. Credenciais exigem proteção
separada. Um bundle Git inclui commits/refs, mas não mudanças não commitadas nem
arquivos não rastreados. Nenhum backup, clone ou restauração foi executado aqui.

## Arquivos desta entrega

Criados (6):

- `docs/internal/spec/POLITICA_GIT_GITHUB_BACKUP.md` — instrução integral.
- `docs/internal/plans/POLITICA_GIT_GITHUB_BACKUP.md` — plano e registro de execução.
- `docs/adr/0002-controle-manual-git.md` — decisão operacional.
- `docs/internal/reports/POLITICA_GITHUB_BACKUP.md` — política, recuperação e evidências.
- `scripts/check-repository.ps1` — diagnóstico somente de leitura.
- `scripts/tests/check-repository.Tests.ps1` — testes operacionais.

Modificados (14): `.gitignore`, `AGENTS.md`, `README.md`,
`docs/internal/memory/CURRENT_STATE.md`, `docs/internal/memory/DECISIONS.md`,
`docs/internal/memory/HANDOFF.md`, `docs/internal/memory/KNOWN_ISSUES.md`,
`docs/internal/memory/PROJECT_CONTEXT.md`, `docs/internal/memory/README.md`,
`docs/internal/references/BACKUP_RECUPERACAO.md`, `docs/internal/reports/BOOTSTRAP_AMBIENTE.md`,
`docs/internal/spec/README.md`, `scripts/validate.ps1`, `tests/README.md`.

Excluídos: nenhum. Código Rust, manifests, lockfile, ADR-0001 e contratos
arquiteturais preservados. Registros históricos do bootstrap não foram apagados.
Todos os 20 arquivos criados/modificados aguardam revisão e versionamento manual.

## Evidências de validação

| Verificação executada | Resultado observado |
|---|---|
| `cargo fmt --check` | Aprovado |
| `cargo check --workspace --all-targets --locked` | Aprovado |
| `cargo test --workspace --locked` | Aprovado; 0 testes Rust, 0 falhas |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Aprovado |
| Testes operacionais PowerShell | 15 aprovados; 0 falhas |
| Parser PowerShell nos três scripts | Sem erros sintáticos |
| Diagnóstico no repositório real | Exit 0, nenhum achado; 27 arquivos rastreados |
| Snapshot SHA-256 dos arquivos de metadados Git antes/depois do diagnóstico | Idêntico |
| `git diff --check` e `git diff --cached --check` | Aprovados; índice sem alterações |
| Arquivos novos, verificados separadamente sem staging | 6 arquivos, sem espaços finais |
| Links Markdown locais e cópia integral da instrução | 23 links válidos; conteúdo igual ao anexo após normalização de finais de linha |
| `git fsck --full` | Exit 0; informou árvore não referenciada, sem erros de integridade |
| CLI com caminho inválido | Retorno 2 e diagnóstico incompleto, conforme contrato |

A árvore não referenciada informada pelo fsck foi
`4b825dc642cb6eb9a060e54bf8d69288fbee4904`; nenhum objeto foi removido.
Avisos iniciais de finais de linha foram resolvidos adequando os arquivos às
regras existentes; a checagem de diff subsequente terminou sem avisos.

Os testes incluem limites exatos e superiores a 50/100 MB, divergência entre
índice e arquivo local, temporários, sensíveis, AGENTS incompleto, ausência de
upstream/commits, HEAD destacado, divergência local, espaços/Unicode, falha de
leitura Git e ignore incorreto. A execução inicial falhou pela ausência do script;
após implementação, os 15 testes passaram. Cenários de histórico/índice usam
respostas simuladas, sem criar repositórios ou commits; integração real é de leitura.

Revisão independente somente de leitura concluída: nenhum achado crítico,
importante ou menor. O revisor confirmou 15 testes operacionais aprovados,
diff válido, HEAD preservado e alinhamento da documentação com o escopo.

Limites considerados na revisão e mantidos conforme a instrução: varredura
completa de segredos não implementada (triagem por nome não garante ausência);
estado remoto atual não inferido pelo diagnóstico (consulta pontual registrada
separadamente); restauração real de backup e homologação manual não executadas
(recuperação documentada ainda depende de teste pelo usuário); funções elétricas
e gráficas permanecem fora desta entrega. Nenhum desses limites autoriza avanço
para Etapa 1 ou operações Git automáticas.

## Estado Git e pendências

Branch `main`; último commit local observado:
`99b4161f155f9e14b4f20d320e6a706a15df3dba`.
Consulta de leitura `git ls-remote --exit-code origin refs/heads/main` confirmou
o mesmo SHA remoto nesta auditoria. Tracking local também aponta para ele;
0 commits à frente/atrás na consulta. Origin permanece
`https://github.com/Vandecao/CAD-CRAFT.git`.

Árvore inicialmente limpa; esta entrega permanece sem staging/commit/push.
As alterações atuais ainda não estão protegidas no remoto. Nenhuma operação
Git modificadora, sincronização, limpeza ou exposição de conteúdo de credenciais
foi executada nesta entrega. Sem homologação manual e sem Etapa 1.

Pendências: revisão e commit/push manuais do usuário; backup externo independente
não implementado/verificado; observação de reinício ENV-003 permanece sem confirmação.

**IMPLEMENTAÇÃO CONCLUÍDA — AGUARDANDO COMMIT E PUSH MANUAIS DO USUÁRIO.**
