# ADR-0013 — conectores industriais e especializações de bornes

Estado: implementado e testado automaticamente na Etapa 11.
Homologação manual abrangente permanece após a Etapa 15.

## Contexto

A Etapa 11 pede conectores (identificação própria, gênero, contatos
numerados, associação explícita a pontos, acoplamento entre metades) e
especializações de bornes (passagem, PE, multinível, distribuição,
seccionável) sobre o modelo físico da Etapa 10 (ADR-0012), sem subsistemas
paralelos e sem continuidade implícita por nome, tipo ou proximidade.

## Decisão

- **Contato = `Terminal` existente.** Identidade `TerminalId`, numeração
  documental (`designation`), posições para `PhysicalTermination` e
  associação via `ConnectionPoint.terminal_id`. Renumerar não muda IDs.
- **`Connector`** (novo, `ConnectorId`): agrupamento ordenado de contatos,
  análogo a `TerminalStrip`; designação própria, `ConnectorGender`
  (`Unspecified/Male/Female/Hermaphroditic`), relação documental opcional com
  `Device`/`Component` (coerente com os donos dos contatos) e propriedades.
  Um borne pertence a no máximo uma régua **ou** um conector; contatos são
  bornes genéricos; numeração única por conector. Designações de conector
  iguais são permitidas e nunca unem redes. `EntityRef::Connector` foi
  acrescentado para erros tipados e `set_properties`.
- **`ConnectorMating`** (novo, `ConnectorMatingId`): acoplamento **documental**
  com mapa explícito `ContactPair(first, second)`. Exige gêneros declarados e
  compatíveis, contatos pertencentes a cada metade, nenhum contato repetido,
  mapa não vazio e no máximo um acoplamento por conector. Excluir conector
  acoplado é recusado (`EntityInUse`), sem cascata.
- **Continuidade por acoplamento NÃO implementada (DEC-11-01).** Unir as redes
  das metades exigiria arestas lógicas pertencentes ao acoplamento (como as
  pontes) — mudança de semântica do grafo que o prompt reserva à decisão do
  usuário. Propostas registradas no relatório da etapa.
- **Especialização no próprio `Terminal`**: `kind: TerminalKind`, `sides`
  (posição → `TerminalSide::A/B`), `levels: Vec<TerminalLevel>` com
  `TerminalLevelId` estável, `disconnect_state` estático. Regras puras em
  `Terminal::check_specialization`:
  - passagem/seccionável: ≥2 posições, todas com lado, lados A e B presentes;
  - PE: ≥1 posição; distribuição: ≥2 posições;
  - multinível: ≥2 níveis, IDs/designações únicos, níveis não vazios,
    posições disjuntas e união igual às posições do borne;
  - níveis só em multinível; estado de seccionamento só (e sempre) em
    seccionável; lados apenas para posições existentes.
- **Nível em `ConnectionPoint`** (`terminal_level`): obrigatório e existente
  para borne multinível, proibido nos demais. Condutor associado por ponto de
  um nível deve terminar numa posição desse nível. Níveis permanecem
  isolados porque não há continuidade sem conexão explícita; ponte explícita
  da régua pode uni-los.
- **PE**: não cria terra, rede ou conexão. Regras verificáveis: condutor com
  função declarada diferente de PE (ou não especificada) não termina em
  borne PE; ponte que toca borne PE exige PE nos dois extremos.
- **Seccionável**: estado `Closed/Open` é documental; não altera grafo, redes
  nem simula (DEC-11-02 registra a alternativa de pontos por lado + ligação
  interna explícita controlada pelo estado).
- **Passagem/distribuição**: continuidade interna somente pela associação
  explícita a ponto funcional (o borne associado a um único ponto é um nó);
  a classificação não cria interligação.
- Edições: `UpsertConnector`, `DeleteConnector`, `UpsertMating`,
  `DeleteMating`, `SetPointTerminal`; `PhysicalDelta` ganhou `connectors`,
  `matings` e `points`, com o mesmo preflight, rollback, `Batch` e Undo/Redo.
- Persistência: listas novas com `#[serde(default)]`; campos novos de
  `Terminal`/`ConnectionPoint` omitidos no valor padrão; snapshots anteriores
  carregam e reserializam idênticos; inconsistências são recusadas na
  importação sem substituir o projeto carregado.
- GUI: aba **Conectores** na janela Engenharia física (comandos
  `ConnectorEditor` e `ConnectorNavigator`, guia Dispositivos/Conectores, que
  substituem os marcadores planejados), editor de especialização do borne
  selecionado na aba Réguas, árvore `Conector → Contatos` e
  `Régua → Borne → Níveis` no navegador Bornes, resumo em Propriedades.
  A janela passa a limitar-se à área da tela (resoluções/escalas menores).

## Consequências

Conectores e especializações reutilizam terminação, associação, numeração,
histórico e listas existentes. Acoplamento não altera redes até decisão do
usuário. A associação ponto↔borne passou a ser editável (antes só na
criação), sempre validada contra pontes, condutores e níveis.
