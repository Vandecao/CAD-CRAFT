# Política Git, GitHub e backup — plano de implementação

**Objetivo:** aplicar a instrução complementar após o Prompt 0, com Git manual
e diagnóstico offline, sem iniciar a Etapa 1.

**Arquitetura:** documentação normativa em AGENTS, memória e ADR-0002;
um script PowerShell 7 somente de leitura, sem dependências adicionais.
Testes operacionais separados dos futuros testes de domínio Rust.

**Especificação:** [instrução integral](../spec/POLITICA_GIT_GITHUB_BACKUP.md).

## Restrições globais

- Trabalhar no diretório existente e na branch atual, `main`.
- Não executar staging, commits, sincronização, troca/criação de branches,
  alteração de refs, limpeza ou qualquer equivalente por outras ferramentas.
- Preservar origin, decisões arquiteturais, código e arquivos necessários.
- GitHub protege apenas conteúdo enviado. Backup externo somente recomendado.
- Homologação manual não realizada; Etapa 1 não autorizada.

## Decisões de implementação

- Executar diretamente o escopo detalhado autorizado; prevalece sobre passos
  genéricos de skills que pedem worktree, commits ou nova aprovação de execução.
- Usar limites decimais conservadores: 50 MB = 50.000.000 bytes e
  100 MB = 100.000.000 bytes. Não confundir com MiB do GitHub.
- Verificar tamanhos do índice e do arquivo de trabalho, sem ler conteúdos
  sensíveis. Nomes suspeitos geram alerta, não uma garantia de ausência de segredos.
- A consulta pontual `ls-remote` é somente leitura e foi feita na auditoria;
  o script não acessará rede e distinguirá tracking local de estado remoto.
- Testes usam respostas Git simuladas para cenários de histórico/índice;
  não criar repositórios ou commits nem para fixtures. Integração real somente
  de leitura, com comparação dos hashes dos arquivos Git antes/depois.

## Tarefas

### 1. Registrar e harmonizar a política

- [x] Auditar arquivos rastreados e `.gitignore`; preservar recursos legítimos.
- [x] Atualizar AGENTS, memória, referência de recuperação e índices.
- [x] Criar ADR-0002 e preservar a instrução integral em docs/spec.

### 2. Implementar diagnóstico e testes

Arquivos: `scripts/check-repository.ps1`,
`scripts/tests/check-repository.Tests.ps1`, `scripts/validate.ps1`.
Interface: `pwsh -File scripts/check-repository.ps1 [-RepositoryPath <pasta>]`.
Saída: relatório legível; códigos 0 (sem achados), 1 (revisão necessária),
2 (diagnóstico incompleto/falha). Sempre informar estado remoto não verificado.

- [x] Escrever e observar falha dos testes antes do script existir.
- [x] Implementar consultas de branch, HEAD, upstream, ahead/behind,
  índice, tamanhos, temporários, sensíveis, estrutura e ignore efetivo.
- [x] Testar limites 50/100 MB, índice maior que arquivo local, caminhos com
  espaços/Unicode, upstream ausente, HEAD sem commit/destacado, falha Git,
  regras ausentes, regra que oculta fonte e AGENTS incompleto.
- [x] Integrar testes operacionais à validação existente e executar diagnóstico.

### 3. Verificar e entregar

- [x] Executar validações Cargo, testes PowerShell e diff sem staging.
- [x] Verificar arquivos novos, ausência de exclusões e preservação do estado Git.
- [x] Revisar a implementação, registrar resultados e limitações no relatório.
- [x] Atualizar CURRENT_STATE, HANDOFF e KNOWN_ISSUES com evidências.
- [x] Entregar lista de alterações e necessidade de commit/push manuais.

## Foco da revisão

Ausência de upstream não significa sincronizado; arquivos grandes já no índice
continuam alertados mesmo reduzidos ou ausentes no disco; nomes não podem ser
quebrados por espaços/Unicode; ignore não pode ocultar fonte/lockfile; diagnóstico
não pode escrever no Git ou buscar objetos ausentes pela rede.

## Registro de execução

- Leitura inicial: árvore limpa, main, bootstrap concluído.
- HEAD e consulta ao remoto oficial: 99b4161f155f9e14b4f20d320e6a706a15df3dba.
- A política nova substitui apenas as regras operacionais antigas incompatíveis;
  Prompt 0 e relatório histórico permanecem identificados como registros históricos.
- Teste inicial: falhou porque o script ainda não existia; implementação seguida
  de 15 testes aprovados. Integração compara SHA-256 de todos os arquivos de .git.
- Validação completa: fmt/check/test/clippy, 15 testes operacionais e diff
  aprovados. Rust continua com 0 testes de domínio. Seis novos arquivos conferidos
  sem staging; 23 links locais válidos; nenhuma exclusão ou alteração de código Rust.
- Revisão independente somente de leitura: nenhum achado crítico, importante ou
  menor. Limitações aceitas por corresponderem ao escopo: triagem de sensíveis
  por nome, diagnóstico offline, restauração/homologação não executadas e Etapa 1
  não iniciada. Consequências práticas registradas no relatório.
- Entrega: 6 arquivos novos, 14 modificados, nenhum excluído; tudo sem staging,
  commit ou push, para revisão e versionamento manuais do usuário.
