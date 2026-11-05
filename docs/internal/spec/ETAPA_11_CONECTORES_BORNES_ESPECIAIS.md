# Etapa 11 — Conectores e especializações de bornes

## 1. Prompt autorizado (resumo fiel, 2026-11-02)

Autorização para implementar a Etapa 11 em `C:\Dev\CAD-CRAFT`, `main`,
commit-base `f1b173e` (Etapa 10 homologada e enviada; linha de base 391 testes
Rust + 15 PowerShell). Objetivo: ampliar o modelo elétrico/físico para
conectores industriais e especializações de bornes, preservando arquitetura,
integridade elétrica, persistência e funcionalidades anteriores. Fora do
escopo: ERC, numeração geral de fiação, relatórios avançados, assistente de IA.

Contratos obrigatórios: `Project` é a fonte da verdade; Core sem GUI; IDs
persistentes independentes de nomes; conexões explícitas; redes derivadas do
grafo; condutores não criam conexões lógicas; proximidade, nomes iguais ou
tipos de borne não estabelecem continuidade; operações transacionais e
reversíveis; snapshots anteriores válidos continuam compatíveis; GUI sem
cópias autoritativas. Reutilizar estruturas existentes; nada de subsistemas
paralelos.

- **Conectores (4)**: identificação própria (`-X1`), macho/fêmea, contatos
  numerados ou alfanuméricos, quantidade variável, posições livres/ocupadas,
  associação explícita a pontos existentes, propriedades verificáveis,
  relações documentais com dispositivos/componentes; IDs estáveis ao
  renomear/renumerar. Operações: criar/excluir, adicionar/remover contatos,
  alterar identificação/propriedades, renumerar, associar/desassociar,
  consultar, impedir referências inválidas — no mecanismo transacional.
  Acoplamento: explícito, mapeamento verificável, rejeitar contatos
  inexistentes/incompatíveis e duplicidades, sem continuidade por nome e sem
  arestas automáticas; mudança fundamental no grafo exige decisão do usuário.
- **Bornes (5)**: passagem (lados, continuidade só explícita), PE
  (classificação tipada, sem terra/redes automáticas), multinível (níveis
  identificáveis, posições por nível, conexões independentes, isolados salvo
  continuidade explícita validada, navegação/edição), distribuição (múltiplos
  pontos, sem interligação presumida), seccionável (diferença entre
  identificação, estado estático e continuidade; sem simulação). Não expor
  tipo cuja semântica o Core não valide.
- **Integridade (6)**: validar dependências, rejeitar inválidos, preservar
  integridade referencial, sem cascata não autorizada, rollback integral,
  Undo/Redo, conectividade preservada em edições documentais, serialização
  consistente, snapshots antigos válidos, snapshots inconsistentes recusados
  sem substituir o projeto. Sem registros fictícios.
- **GUI (7)**: criar/editar conectores, contatos, identificação, associação,
  localizar, propriedades, bornes especializados, níveis/posições, mensagens,
  Undo/Redo; navegação `Conector → Contatos/Posições` e
  `Régua → Borne → Níveis/Posições`; preservar tema, Ribbon, seleção,
  janelas estáveis (GUI-PHY-01/02), rolagem e contenção.
- **Testes (8)**, **validação (9)** com `scripts/validate.ps1`, Release e
  `git diff --check`; **documentação e ai-memory (10)**; **Git manual (11)**;
  **critérios (12)**: não declarar parcial como completo, registrar o que
  depende de decisão, encerrar sem iniciar a Etapa 12.

## 2. Especificação técnica executada

Decisão completa no [ADR-0013](../../adr/0013-conectores-e-especializacoes-de-bornes.md).

### Core (`crates/cad-craft`)

| Item | Contrato |
|---|---|
| `ConnectorId`, `ConnectorMatingId`, `TerminalLevelId` | UUID v7 tipados |
| `EntityRef::Connector` | erros tipados e `set_properties` |
| `Connector` | designação, `ConnectorGender`, `device`/`component` opcionais, `contacts: Vec<TerminalId>` ordenados, propriedades |
| `ConnectorMating` / `ContactPair` | acoplamento documental com mapa explícito |
| `Terminal` | `kind: TerminalKind`, `sides`, `levels: Vec<TerminalLevel>`, `disconnect_state`; `check_specialization()` |
| `ConnectionPoint.terminal_level` | nível obrigatório para multinível, proibido nos demais |
| `PhysicalEdit` | `UpsertConnector`, `DeleteConnector`, `UpsertMating`, `DeleteMating`, `SetPointTerminal` |
| Consultas | `connector(s)`, `connector_mating(s)`, `connector_for_terminal`, `mating_for_connector`, `mated_contact`, `points_for_terminal`, `contact_status`, `connector_occurrence` |
| Listas | `PhysicalLists.connectors` (`ConnectorListRow`/`ConnectorContactRow`); `TerminalListRow.kind/levels/connector/connector_designation` |

Regras de integridade: ver ADR-0013 (unicidade de numeração por conector,
borne em uma régua ou um conector, coerência de dono, gêneros, pares,
níveis, PE, seccionável).

### GUI (`crates/cad-gui`)

- `CommandId::ConnectorEditor` ("Conectores") e `ConnectorNavigator`
  ("Navegador de conectores") na guia Dispositivos, grupo Conectores.
- Janela Engenharia física: aba **Conectores** (`physical_editor/connectors.rs`)
  com criação em lote, edição, exclusão atômica, contatos (ordem, renomear,
  remover, adicionar, renumerar), associação explícita a pontos do
  dispositivo, localizar, acoplamento (par manual, preenchimento por
  numeração somente como rascunho, confirmação explícita, exclusão).
- Aba Réguas: editor de especialização do borne selecionado (tipo, posições,
  lados, níveis, estado de seccionamento) e associação de pontos com nível.
- Navegador Bornes: conectores e contatos, níveis sob o borne.
- Propriedades: especialização, lados, níveis, estado, conector e contato
  acoplado.
- Janela limitada ao tamanho da tela (resoluções/escalas menores).

### Não implementado (depende de decisão)

- DEC-11-01: continuidade elétrica pelo acoplamento de conectores.
- DEC-11-02: efeito do estado de seccionamento na continuidade.
