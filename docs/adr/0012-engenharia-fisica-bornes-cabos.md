# ADR-0012 — engenharia física: bornes, pontes, cabos e condutores

Estado: implementado e testado automaticamente na Etapa 10.
Homologação manual abrangente permanece após a Etapa 15.

## Contexto

A Etapa 3 criou Terminal, TerminalStrip, Cable, CableCore e Conductor com
validação estrutural, mas sem operações de edição reversíveis nem interface.
A Etapa 10 exige editor de réguas, pontes, cabos/veias, atribuição de fiação,
navegadores e listas técnicas sem criar um segundo sistema de fiação e sem
confundir rota gráfica, conexão lógica, condutor e cabo. A mesma etapa exigiu
corrigir antes os defeitos GUI-ROUTE-09A e GUI-ROUTE-09B.

## Decisão

- Reutilizar os registros da Etapa 3. Toda mutação física passa por
  `Project::edit_physical(PhysicalEdit) -> PhysicalDelta`; o delta guarda
  `before/after` somente das entidades tocadas e `restore_physical` faz
  preflight do estado esperado, grava, reconstrói grafo/índice físico e valida
  o projeto inteiro; em erro reverte tudo (sem mutação parcial).
- `PhysicalEdit::Batch` aplica várias edições sem publicar estados
  intermediários, valida apenas o estado final e compõe um único delta
  (primeiro `before`, último `after` por chave). Usado em "criar e associar
  condutor" e na renumeração sequencial (permite trocar 1↔2 sem violar a
  unicidade intermediária). Lote aninhado é recusado.
- Ponte (`TerminalBridge`) tem como identidade o `ConnectionId` da conexão
  lógica que possui; une dois pontos funcionais de bornes da mesma régua, é
  `Fixed` ou `Inserted`, não pode ser removida por `remove_connection`, não
  recebe condutor externo e não pode pertencer a rota/marcador. Atualiza redes
  porque é uma aresta explícita do grafo, nunca por proximidade gráfica.
- Numeração de borne é designação documental, não identidade. A régua mantém
  ordem explícita de `TerminalId`; designações duplicadas na mesma régua são
  recusadas. Tipos especiais (PE, multinível, distribuição, seccionável) não
  são declarados: não há regras validáveis nesta etapa.
- Associação física: `Conductor.connection` com orientação pelo ponto cujo
  borne coincide com a terminação de origem. Condutor exige seção nominal em
  mm² informada pelo usuário; veia aceita seção ausente. Cor/seção não são
  inventadas; veia PE não é presumida.
- Listas técnicas (`Project::physical_lists`) e a árvore dos navegadores são
  derivadas do Core em O(n log n) e cacheadas na GUI por (projeto, revisão);
  não há cópias persistidas. `physical_occurrence` localiza ocorrência
  gráfica somente por relações explícitas (objeto, pontos, conexão, dono).
- GUI: janela "Engenharia física" (réguas/bornes/pontes, cabos/veias,
  condutores, listas) aberta pelos comandos do dispatcher nas guias
  Dispositivos, Conexões e na nova guia Cabos; navegadores Cabos/Bornes do
  painel esquerdo passam a ser hierárquicos; Propriedades e inspetor de
  conexão mostram camadas físicas. Histórico compartilhado
  (`Transaction::Physical`).
- Snapshot: `terminal_bridges` e `Cable.description` têm defaults; pontes são
  validadas na importação (régua, posse, bornes membros).

## Consequências

Undo/Redo físico preserva IDs e não captura snapshots integrais. Cada edição
física revalida o projeto inteiro (custo linear medido com milhares de
registros, sem garantia industrial). Bornes sem posição de conexão não podem
receber condutor; pontos funcionais sem borne físico não aceitam associação —
a interface informa em vez de criar entidades fictícias. Relatórios completos
ficam para a Etapa 13; formato `.cadcraft` final permanece futuro.
