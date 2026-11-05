# ADR-0010 — Rotas documentais e junções elétricas explícitas

Data: 2026-10-30. Estado: implementado, testado automaticamente e revisado na
Etapa 8; sem homologação manual. Escopo autorizado no anexo da Etapa 8.

## Contexto

O Core já possui ConnectionPoint, Connection e redes derivadas. SymbolPort é uma
porta gráfica local, sem equivalência implícita a um ponto elétrico. O editor
precisa representar uma conexão e ajustar sua geometria sem transformar desenho
em fiação física nem inferir conectividade em cruzamentos.

## Decisão

- PortRef combina RepresentationId e identificação estável da porta. Project
  mantém vínculos explícitos, validados contra o alvo elétrico da ocorrência.
  Portas sem alvo compatível não podem ser utilizadas para criar conexões.
- ConnectionRoute complementa a Representation, compartilhando seu ID, página e
  alvo ConnectionId. Guarda portas de origem/destino e polilinha ortogonal em mm;
  identidade e vínculo sobrevivem a mudanças de posição, Undo/Redo e snapshots.
- ConnectionJunction possui ID UUID v7, página, posição, rotas participantes e
  arestas auxiliares exclusivas do mesmo grafo lógico. Não é dispositivo nem ponto
  funcional fictício. Arestas ligam endpoints existentes das rotas participantes.
  Conexões/condutores de terceiros não são apropriados ou removidos implicitamente.
- Cruzamento geométrico sozinho não altera redes. Criar junção exige que a posição
  pertença a todas as rotas indicadas na mesma página e valida topologia resultante.
  O desenho usa ponto cheio ou anel configurável nas preferências visuais.
- ConnectionDrawingEdit produz ConnectionDrawingDelta com somente entidades
  afetadas. A aplicação valida estado e índices antes de publicar o resultado;
  falhas restauram os registros provisórios. Replay confere o estado esperado.
  O histórico da GUI usa esses deltas junto das operações da Etapa 7.
- Transforms atualizam terminais gráficos. Translação comum preserva o percurso.
  Mudanças individuais recompõem trechos terminais; se uma junção não puder
  permanecer consistente, a operação é recusada integralmente.
- Cache de candidatos de snap é invalidado por revisão/página. Redes são consultadas
  no Core ao selecionar propriedades/destaque, nunca recalculadas durante hover.
  Prévia em Interaction não contém entidades persistentes nem transações por frame.
- Snapshot técnico ganha vínculos, rotas e junções com defaults vazios. Importação
  valida referências, geometria, IDs duplicados e posse das arestas. Formato final,
  versionamento físico e migrações continuam fora do escopo.

## Consequências e limites

Há uma única conectividade lógica. Uma Connection criada pelo editor não cria
condutor físico, veia de cabo, propriedades dimensionadas ou certificação elétrica.
Redes continuam derivadas, sem ID persistente próprio. Propriedades mostram IDs,
endpoints, proprietários, rede e vínculos físicos realmente registrados.

Junções desta etapa unem rotas completas existentes. Não implementam todo o
rastreamento de destinos do EPLAN; ramo com extremidade gráfica livre diretamente
em segmento exige futura generalização do endpoint gráfico. Ângulos são dobras de
rota; marcadores de junção têm semântica explícita e não duplicam SymbolDefinition.
Pontes decorativas, interrupções entre páginas e definições de potencial completas
não fazem parte desta implementação. Sugestões por alinhamento são transitórias e
dependem de clique em porta válida para confirmar uma conexão.

Validação das operações confirmadas reconstrói índices; o custo é medido no
relatório. Isso não ocorre por frame e não equivale a desempenho industrial
comprovado. Homologação abrangente permanece após a Etapa 15.
