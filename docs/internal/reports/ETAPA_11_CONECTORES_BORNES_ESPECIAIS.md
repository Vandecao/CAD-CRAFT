# Etapa 11 — conectores e especializações de bornes

**Estado:** implementada e validada automaticamente; pronta para revisão do
usuário. **Não homologada manualmente** (homologação adiada até após a
Etapa 15). Sem commit/push executados pelo agente.

Prompt: `docs/internal/spec/ETAPA_11_CONECTORES_BORNES_ESPECIAIS.md`.
Plano: `docs/internal/plans/2026-11-02-etapa-11.md`. Decisão: ADR-0013.

## Base e retomada

- `main`, HEAD `f1b173e9e5a9d0a485163bfd2c13a871a6b46cbd` (Etapa 10 commitada
  e enviada pelo usuário), consulta somente leitura em 2026-11-02.
- Contexto recuperado do handoff do ai-memory, `HANDOFF.md`/`CURRENT_STATE.md`
  e ADR-0012; a sessão anterior (Claude Code) estava encerrada, sem trabalho
  pendente. Árvore inicial: apenas `docs.rar` não versionado (do usuário,
  não tocado).

## Implementado

### Conectores (Core)

- `Connector` tipado (`ConnectorId`): designação própria, gênero
  (`Unspecified/Male/Female/Hermaphroditic`), relação documental opcional com
  `Device`/`Component`, propriedades, contatos ordenados.
- **Contato = `Terminal` existente** (reuso integral de terminação física,
  posições, associação a ponto e numeração). Renumerar/renomear preserva IDs.
- Posições livres/ocupadas derivadas (`contact_status`,
  `ConnectorContactRow::is_free`): ocupado = ponto associado ou condutor.
- Associação explícita ponto↔contato por `PhysicalEdit::SetPointTerminal`,
  validada (ponto do mesmo dispositivo/componente, pontes, condutores,
  níveis). Desassociar com condutor dependente é recusado.
- Integridade: numeração única por conector, borne em uma régua **ou** um
  conector, contato genérico, dono coerente com o dispositivo do conector,
  componente coerente com o dispositivo.
- Acoplamento `ConnectorMating` documental com mapa explícito; gêneros
  declarados e compatíveis; contatos inexistentes, de outro conector ou
  repetidos recusados; um acoplamento por conector; exclusão de conector
  acoplado recusada sem cascata. **Sem continuidade elétrica** (DEC-11-01).

### Especializações de bornes (Core)

`Terminal.kind`, `sides`, `levels`, `disconnect_state` com regras
verificáveis (`Terminal::check_specialization`) — ver ADR-0013:

| Tipo | Regras verificáveis | Continuidade |
|---|---|---|
| Passagem | ≥2 posições, todas com lado, lados A e B | só por ponto associado explicitamente |
| PE | ≥1 posição; condutor declarado não PE recusado; ponte só PE↔PE | nada criado (sem terra/rede) |
| Multinível | ≥2 níveis, IDs/nomes únicos, posições disjuntas = posições do borne; ponto exige nível; condutor no nível do ponto | níveis isolados; ponte explícita pode unir |
| Distribuição | ≥2 posições | só por associação explícita |
| Seccionável | lados A/B + estado `Closed/Open` obrigatório | estado documental, sem efeito (DEC-11-02) |

### Transações e persistência

- Novas edições `UpsertConnector`, `DeleteConnector`, `UpsertMating`,
  `DeleteMating`, `SetPointTerminal`; `PhysicalDelta` com `connectors`,
  `matings`, `points`; mesmo preflight, rollback integral, `Batch` e
  Undo/Redo do histórico existente.
- Snapshots: listas novas com default e omitidas quando vazias; campos novos
  de `Terminal`/`ConnectionPoint` omitidos no padrão — snapshot anterior
  carrega e reserializa **idêntico** (teste). Snapshots inconsistentes
  (acoplamento órfão, contato em dois conectores, ID duplicado, campo
  desconhecido, nível divergente, tipo incoerente) são recusados.

### GUI

- Ribbon, guia Dispositivos, grupo Conectores: **Conectores**
  (`ConnectorEditor`) e **Navegador de conectores** (`ConnectorNavigator`),
  substituindo os três marcadores planejados do grupo.
- Janela Engenharia física, nova aba **Conectores**: criação (dispositivo,
  identificação — padrão = designação do dispositivo —, gênero, quantidade,
  posição de conexão) em uma transação; salvar identificação/gênero;
  "Excluir conector e contatos" atômico; localizar; propriedades derivadas;
  contatos com estado livre/ocupado, ordem ↑/↓, renomear, remover, adicionar,
  renumerar; associação explícita a pontos do dispositivo; acoplamento com
  pares manuais, "Parear por numeração" apenas como rascunho e
  "Confirmar acoplamento" explícito.
- Aba Réguas: editor de especialização do borne selecionado (tipo, posições,
  lados, níveis com adicionar/remover/renomear, estado de seccionamento) e
  associação de pontos com nível; tipo exibido na lista de bornes da régua.
- Navegador Bornes: `Régua → Borne [tipo] → Níveis` e
  `Conector → Contatos` (contatos não aparecem como bornes soltos).
- Propriedades: especialização, lados, níveis, estado, conector e contato
  acoplado (documental).
- GUI-PHY-01/02 preservados (testes estendidos à nova aba). **GUI-PHY-03**
  corrigido: em telas pequenas/escalas altas a janela excedia a tela; agora
  `default_size`/`max_size` limitam-se à área disponível.

## Testes

| Arquivo | Novos | Cobertura |
|---|---|---|
| `crates/cad-craft/tests/connectors.rs` | 12 | 4 posições, inclusão/remoção, renumeração com IDs, associação explícita e incompatível, condutor dependente, nomes iguais sem continuidade, acoplamento válido/inválido, exclusão atômica com dependência, persistência e snapshots antigos |
| `crates/cad-craft/tests/terminal_kinds.rs` | 8 | regras por tipo, PE sem redes fictícias, PE×condutores/pontes, multinível (validação, independência, ponte explícita, remoção de nível em uso), Undo/Redo, snapshots, renumeração |
| `crates/cad-gui/tests/connector_ui.rs` | 5 | cliques egui reais: criar/editar/associar/localizar conector, renumerar e acoplar, especialização PE e multinível com nível de ponto, Undo; estabilidade em 1280×720@1, 1366×768@1,25, 1920×1080@1,5, 2560×1440@2, 1600×900@1 |
| `crates/cad-gui/src/physical_editor.rs` (unit) | 2 | árvore do navegador com níveis/contatos e resumo de Propriedades; rascunho de especialização |
| `crates/cad-gui/tests/physical_ui.rs` | 0 (ampliado) | GUI-PHY-01/02 incluem a aba Conectores |

## Validação (2026-11-02)

| Comando | Resultado |
|---|---|
| `pwsh -NoProfile -File scripts/validate.ps1` | exit 0: fmt, check, clippy `-D warnings`, testes e diff check aprovados |
| Testes Rust | **418 aprovados, 0 falhas, 0 ignorados** (57 suítes; 391 anteriores preservados + 27 novos) |
| Testes PowerShell operacionais | **15 aprovados, 0 falharam** (pwsh disponível nesta sessão; PHY-013 encerrado) |
| `cargo build --release -p cad-gui --bin cad-gui --locked` | exit 0; `target/release/cad-gui.exe` 10.358.784 bytes, SHA-256 `B043E93E48499251C4055B1A69D1107E2D129ED009754F5C10581DB447D542D1` |
| `git diff --check` | sem ocorrências |

Logs locais (não versionados): `.local/etapa-11/validate-final.log`,
`.local/etapa-11/release.log`.

## Revisão independente

Releitura do diff com foco em integridade elétrica, persistência,
Undo/Redo e GUI. Conclusões: dependências são revalidadas no estado final
(`restore_physical`/`Batch` reconstroem grafo e índice); nenhuma operação
cria conexões, exceto `CreateBridge` pré-existente; exclusões sem cascata no
Core; snapshot reconstrói o mesmo índice. Ajustes aplicados na revisão:
rótulo de contato usa o conector (`X1:2`, não `-K1:2`), botão renomeado para
"Excluir conector e contatos", tipo do borne visível na lista da régua,
janela limitada à tela. Nenhum P1 aberto.

## Limitações e pendências

- **DEC-11-01 (decisão do usuário):** continuidade pelo acoplamento.
  - Proposta A: o acoplamento passa a possuir uma `Connection` por par cujos
    contatos tenham ponto associado (como as pontes), criada/removida na
    mesma transação, sem condutor externo.
  - Proposta B: manter o acoplamento documental e exigir conexões lógicas
    desenhadas/explicitas entre os pontos (situação atual).
- **DEC-11-02 (decisão do usuário):** efeito do estado seccionável na
  continuidade (pontos por lado + ligação interna controlada pelo estado).
- Designações de conector não são únicas no projeto (CON-001).
- Criação de pontos funcionais continua fora do editor físico (PHY-011).
- Homologação manual após a Etapa 15; nenhuma homologação declarada.

## Git (somente consulta)

Branch `main`; HEAD local `f1b173e9e5a9d0a485163bfd2c13a871a6b46cbd`
(2026-11-02). Todas as alterações da Etapa 11 estão **fora do HEAD**
(não commitadas). Etapa ímpar: **commit manual** do usuário (push opcional
conforme política). `docs.rar` não versionado pertence ao usuário e não foi
alterado; avaliar antes de qualquer `git add`. Backup externo independente
recomendado.
