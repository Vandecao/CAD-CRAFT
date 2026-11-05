# Etapa 2 — plano de implementação

> Execução inline com as skills writing-plans, executing-plans e test-driven-development;
> revisão independente interna ao final com requesting-code-review e verificação
> com verification-before-completion. Autorização: anexo integral da Etapa 2.

**Objetivo:** conexões lógicas explícitas, consultas e redes derivadas no Core.
**Arquitetura:** registros privados no Project; Connection canônica não direcionada;
índice de adjacência privado, descartável e não serializado; DFS iterativa e resultados
ordenados. Sem cache persistente de redes, bibliotecas adicionais ou GUI.
**Tecnologias:** Rust, std::collections, Serde e uuid já existentes.
**Spec:** ../spec/ETAPA_02_CONECTIVIDADE_REDES.md.

## Restrições e decisões de implementação

- Executar somente Etapa 2 no checkout autorizado C:\Dev\CAD-CRAFT.
  Sem worktree, branch, staging, commit, sincronização ou scripts Git modificadores.
- Anexo pede execução autônoma: escolhas rotineiras ficam registradas aqui,
  sem nova rodada de aprovação de design. Homologação após Etapa 15.
- Preservar entidades, 26 testes Rust e 15 PowerShell da Etapa 1.
- Connection guarda somente ID, ProjectId e dois ConnectionPointId canônicos.
  Sem propriedades nesta etapa: nenhum atributo lógico adicional foi exigido.
  EntityRef/propriedades da Etapa 1 continuam com seus alvos atuais; erros de
  conexão têm variantes específicas com ConnectionId. Isso evita introduzir
  referências removíveis em propriedades sem requisito de negócio.
- Componentes associados a uma rede: por função explicitamente associada ou por
  propriedade de terminal mapeado por ponto. Dispositivo compartilhado, sozinho,
  não associa todos os seus componentes nem conecta funções/terminais.
- Redefinir redes após cada consulta; membros por ID crescente e redes pelo
  menor ponto. Ponto existente está conectado a si; ID ausente retorna erro.
- Consulta entre dispositivos significa existir uma rede contendo pontos de ambos;
  não representa simulação, passagem interna de corrente ou ligação de todos os polos.
- Snapshot sem campo connections (Etapa 1) será aceito como lista vazia. Outros
  campos obrigatórios e rejeição de desconhecidos preservados. Não é formato final.
- Alternativas: recalcular adjacência em cada inserção induziria custo quadrático;
  Union-Find complicaria remoções. Índice derivado + DFS mantém correção e simplicidade.

## Foco da revisão

1. Falha de inserção/remoção não altera registro nem índice.
2. Importação fora de ordem, extremos invertidos e duplicatas não burlam invariantes.
3. Mesmo dispositivo/função/terminal não cria arestas implícitas.
4. Componentes associados não atravessam funções independentes por engano.
5. Grafo vazio, muitos isolados, estrela e cadeia longa não geram recursão ou scans quadráticos.

## Tarefa 1 — conexão, agregado e reconstrução

Arquivos: criar domain/connection.rs, domain/project/connectivity.rs e graph.rs;
alterar domain/{ids,mod,project}.rs, project/{snapshot,integrity}.rs, error.rs, lib.rs;
criar tests/{common/mod,connections,connectivity_serialization}.rs.
Interfaces: Connection::new(ProjectId, ConnectionPointId, ConnectionPointId) -> Result<Self>;
id(), project_id(), endpoints(); Project::insert_connection(Connection) -> Result<ConnectionId>;
connection(ConnectionId) -> Option<&Connection>; connections() -> Iterator<&Connection>;
remove_connection(ConnectionId) -> Result<Connection>.

- [x] Escrever testes de UUID, canonização, duplicação, erro sem mutação, remoção e Serde.
- [x] Executar testes e observar falha por APIs ainda inexistentes.
- [x] Implementar validação local/relacional, índices e snapshot compatível.
- [x] Executar os testes novos e registrar resultado.

## Tarefa 2 — redes e consultas

Arquivos: criar domain/network.rs e tests/connectivity.rs; ampliar graph.rs e
domain/project/connectivity.rs. Network: listas imutáveis ordenadas de pontos,
conexões, funções, dispositivos e componentes, has_cycle().
Project: networks() -> Vec<Network>; network_for_point(id) -> Result<Network>;
are_connected(a,b) -> Result<bool>; directly_connected_points(id) -> Result<Vec<ConnectionPointId>>;
incident_connections(id) -> Result<Vec<ConnectionId>>;
are_devices_connected(a,b) -> Result<bool>.

- [x] Escrever cenários A–J, relações entre funções/componentes e consultas inválidas.
- [x] Observar falhas; implementar DFS iterativa e enriquecimento dos resultados.
- [x] Testar inserções/remoções e comparação com oráculo independente em grafos pequenos.

## Tarefa 3 — escala, revisão e fechamento

Arquivos: tests/connectivity_scale.rs; docs/internal/reports/ETAPA_02_CONECTIVIDADE_REDES.md;
ADR-0004; memória, README e índice de specs.

- [x] Testar cadeia, estrela e isolados com 100, 1.000 e 10.000 pontos; medir operações
  separadamente com Instant, sem limites de tempo arbitrários.
- [x] Executar cargo fmt e scripts/validate.ps1 (fmt/check/test/clippy, 15 PowerShell,
  diff normal/cached) e diagnóstico offline, guardar logs ignorados em .local/etapa-02.
- [x] Revisão independente do código atual incluindo arquivos novos; corrigir achados.
- [x] Atualizar documentação, memória e inventário; conferir whitespace inclusive novos.
- [x] Consultar branch/HEAD/origin/remoto; entregar sem iniciar Etapa 3.

## Registro de execução

- 2026-10-27: contexto e política lidos; árvore inicial limpa; main em
  dfbbbc17b3d4cec42362b288e2e74f281f193c08; origin/main remoto em
  69f8d6c2877ba617f1c41f8c28d333b67f51e9a8. Etapa 1 commitada pelo usuário.
- Interfaces entre tarefas: T1 fornece conexões e índice a T2; T2 fornece redes
  para T3. Nenhum conflito de assinatura identificado.
- Ruling: preservar EntityRef sem novos alvos removíveis — ausência de requisito
  de propriedades em Connection — extensão futura exige política referencial própria.
- Ruling: execução e ledger neste plano, sem scripts de worktree/commit da skill —
  política expressa do usuário — histórico só existirá após commit manual.
- Baseline: cargo test --workspace --locked, exit 0, 26 Rust preservados.
- Task 1: complete. Testes novos falharam por Connection/ConnectionId/APIs ausentes
  (task1-red.log); depois 12/12 aprovados (task1-green.log), exit 0.
- Task 2: complete. Falha por networks/consultas ausentes (task2-red.log), seguida
  de 13/13 aprovados (task2-green.log), exit 0. Inclui todos os 64 grafos simples
  de quatro pontos e remoções sucessivas, comparados com fecho transitivo independente.
- Task 3: complete. scripts/validate.ps1 exit 0, 55 Rust + 15 PowerShell = 70
  aprovados, zero falhas/ignorados; fmt/check/Clippy/diff aprovados. Ensaios release
  com três amostras por tamanho/topologia, 3/3 aprovados, sem metas de tempo.
- Diagnóstico offline, rustdoc e smoke cad-gui: exit 0. Logs locais ignorados em
  .local/etapa-02. Sem alteração nos testes antigos/manifests/lockfile/scripts.
- Final review: revisor independente interno por inspeção, sem executar testes;
  nenhum achado Critical/Important/Minor, nenhuma deliberação deixada sem julgamento.
  Documentação final conferida pelo principal, sem nova auditoria independente.
- Final: nenhum achado menor adiado. Memória/ADR/relatório atualizados; 13 novos,
  15 modificados, zero excluídos. Branch e SHAs observados mantidos. Commit/push e
  backup externo continuam pendentes do usuário; Etapa 3 não iniciada.
