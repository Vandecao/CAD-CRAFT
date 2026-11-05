# Etapa 10 — correções de roteamento, bornes, cabos e condutores

**Estado:** implementada e validada automaticamente; pronta para revisão do
usuário. **Não homologada manualmente** (homologação adiada até após a Etapa 15).
Sem commit/push executados pelo agente.

Prompt: `docs/internal/spec/ETAPA_10_BORNES_CABOS_CONDUTORES.md`.
Plano: `docs/internal/plans/2026-11-02-etapa-10.md`. Decisão: ADR-0012.

## Continuidade entre agentes

O Codex executou as unidades 1–4 parcialmente e foi interrompido por limite de
franquia. O Claude Code retomou em 2026-11-02 a partir do código real (as
observações do ai-memory continham apenas metadados de ferramentas). Antes de
qualquer alteração, a suíte herdada passou: **376 testes Rust, 0 falhas**.
Nenhuma correção do Codex foi reimplementada ou revertida.

## Parte I — correções gráficas

### GUI-ROUTE-09A — prolongamentos horizontais (Codex; verificado)

- **Causa-raiz:** `repair_endpoints` reparava a rota mesmo com extremos
  inalterados e inseria um cotovelo colinear ao primeiro/último vértice autoral;
  o resultado era um trecho de ida e volta (ponta residual) no eixo horizontal.
- **Solução:** retorno idêntico quando os extremos coincidem com a base;
  remoção desse vértice redundante apenas na polilinha derivada. A base autoral
  (`routing_base`/`RouteIntent`) permanece intacta, logo voltar à posição
  original restaura a geometria. Junções continuam verificadas pelo Project.
- **Regressões:** `horizontal_endpoint_repair_does_not_draw_a_residual_backtrack`,
  ciclos de redimensionamento em ambos os eixos, rotações seguidas de
  redimensionamento vertical, translação em grupo, junção manual e importação
  de snapshot (`crates/cad-craft/tests/connection_drawing.rs`).

### GUI-ROUTE-09B — condutor atravessando o símbolo (Codex; verificado)

- **Causa-raiz:** a rota era derivada só dos endpoints e da base, sem
  considerar o corpo transformado do símbolo; após 180°/270° a direção de
  saída da porta apontava para o interior.
- **Solução:** `route_clearance.rs` converte primitivas fechadas da
  `SymbolDefinition` transformada em polígonos (não usa AABB como obstáculo),
  aplica afastamento de 2,0 mm documentais, independente do zoom, e desvia
  ortogonalmente de forma determinística a partir da direção real da porta,
  sem mover o endpoint. Traços abertos não são obstáculos (contato no terminal
  e traços internos permitidos). A flag `auto_clearance` é serializada e
  reverificada na importação; prévia e confirmação usam o mesmo algoritmo
  (`Project::preview_route_points`). Quando não existe percurso válido, a
  operação é recusada explicitamente, sem alterar conectividade.
- **Regressões:** rotações 0/90/180/270 e espelhos, símbolo próximo, movimento
  de símbolo sobre a rota com Undo, rota manual contornada; captura egui real
  em três níveis de zoom (`connection_view.rs`).
- **Conectividade:** testes conferem redes, endpoints e condutores físicos
  inalterados; nenhum ConnectionPoint, junção ou rede é criado.

## Parte II — engenharia física

### Camadas (sem segundo sistema de fiação)

1. Conectividade lógica: `Connection`/`ConnectionPoint`, redes derivadas.
2. Ponte: `TerminalBridge`, dona de uma `Connection` explícita entre bornes da
   mesma régua (fixa/inserida).
3. Condutor físico: `Conductor` com terminações em bornes e associação
   opcional explícita a uma conexão.
4. Veia: `CableCore` do cabo, opcionalmente ligada a um condutor.
5. Borne/régua: `Terminal` com numeração documental; `TerminalStrip` com ordem
   explícita de IDs.
6. Representação gráfica: rotas/ocorrências, apenas consultadas.

### Core

- `PhysicalEdit` (upsert/delete de bornes, réguas, cabos, veias, condutores;
  atribuição de veia; associação de condutor; criar/remover ponte) →
  `PhysicalDelta` reversível com preflight, revalidação e rollback total.
- **Novo:** `PhysicalEdit::Batch` atômico (estados intermediários não são
  publicados; um único delta composto). Lote aninhado recusado.
- **Novo:** numeração duplicada de bornes na mesma régua é recusada.
- **Novo:** `Project::physical_lists()` — listas derivadas de réguas, bornes,
  cabos, veias, condutores e associações conexão→condutores, em O(n log n).
- **Novo:** `Project::physical_occurrence(target)` — ocorrência gráfica por
  relações explícitas (objeto, pontos, conexão, dono), nunca por proximidade.
- Exclusões com dependências (cabo com veias, condutor atribuído, borne com
  ponto, régua com ponte, remoção de borne-membro de ponte) são recusadas sem
  mutação. Pontes não recebem condutor externo.

### GUI

- Janela **Engenharia física** (dispatcher central):
  - *Réguas e bornes:* criar/renomear/excluir régua; adicionar, remover e
    reordenar bornes; linha por borne com dono, lados de conexão, conexões,
    pontes e condutores; renomear borne; **renumeração sequencial em uma
    transação**; criar/remover pontes fixas/inseridas; localizar.
  - *Cabos e veias:* criar cabo, editar identificação, descrição, fabricante,
    código, tipo e quantidade de veias; cadastrar/editar/excluir veias com
    seção opcional; atribuir/liberar condutor (incompatibilidade rejeitada).
  - *Condutores (novo):* lista virtualizada de conexões lógicas com estado;
    extremos e bornes; condutores associados e remoção da associação; associar
    condutor compatível livre; cadastrar condutor (seção obrigatória, sem
    valores inventados); **criar e associar em uma transação**; editar
    designação/seção, excluir, localizar.
  - *Listas técnicas (novo):* réguas, bornes, cabos, veias, condutores e
    associações, virtualizadas, derivadas por revisão; seleção e localização.
- Ribbon: Editor de réguas (Dispositivos), Atribuir fiação (Conexões), Editor
  de cabos e Listas técnicas (nova guia Cabos). Sem comandos decorativos.
- Navegadores **Cabos** (cabo → veias; fios individuais) e **Bornes** (régua →
  bornes em ordem e pontes; bornes sem régua; página da ocorrência),
  hierárquicos e cacheados por revisão; clique seleciona, duplo clique abre a
  página e centraliza a ocorrência.
- Propriedades: resumo físico tipado por entidade. Inspetor de conexão: lista
  condutores com extremos, seção e cabo/veia, distingue ponte, e oferece
  "Atribuir fiação física" pré-selecionando a conexão.
- Histórico: `Transaction::Physical` no histórico existente (Undo/Redo).

### Persistência

`terminal_bridges` e `Cable.description` com defaults; snapshots anteriores
carregam. Ponte com régua inexistente é rejeitada e não substitui o projeto.

## Testes

Executados em 2026-11-02 (America/Sao_Paulo), passo a passo do `validate.ps1`:

| Comando | Resultado |
|---|---|
| `cargo fmt --all --check` | aprovado |
| `cargo check --workspace --all-targets --locked` | aprovado |
| `cargo test --workspace --locked` | **389 aprovados, 0 falhas, 0 ignorados** |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | aprovado, 0 avisos |
| `git diff --check` | aprovado |
| `cargo build --release -p cad-gui --bin cad-gui --locked` | aprovado; 9.896.448 bytes; SHA-256 `ee25828be12e299bae49ed8523a649cff95bd41ea15786ff91910feb6d554992` |
| `pwsh … check-repository.Tests.ps1` (15 testes) | **não executado**: PowerShell 7 ausente; PowerShell 5.1 recusa o `#requires` |

Contagem Rust: 364 (Etapa 9, preservados) + 12 do Codex (rotas, edição física,
histórico, captura egui) + 13 desta retomada = 389.

Novos testes desta retomada:
- `crates/cad-craft/tests/physical_engineering.rs` (7): lote atômico
  criar+associar e rollback; numeração única e troca atômica; exclusões com
  dependências; listas derivadas; compatibilidade/validação de snapshot;
  4.002 registros; `UpsertBridge` restrito.
- `crates/cad-gui/tests/physical_ui.rs` (3): comandos nas guias da Ribbon;
  **cliques egui reais** (localizados por accesskit) criando régua com
  Undo/Redo e verificando abas Condutores/Listas sem placeholder; associação de
  condutor como um passo do histórico preservando redes.
- `physical_editor.rs` (3 unitários): árvore hierárquica dos navegadores,
  renumeração sequencial reversível, resumo de propriedades sem valores
  inventados.

## Revisão independente

Revisão independente focada (subagente, somente leitura) sobre transação
física, lote, integridade de pontes, snapshot, listas e GUI. Sem P1: lote,
rollback, composição do delta, Undo/Redo e validação de pontes no snapshot
confirmados corretos. Achados corrigidos:

- **P2** "Salvar veia" descartava a cor da marcação → cor preservada.
- **P2** um único estado de borne servia à lista da régua, ao combo de
  anexação e à exclusão (duplicata/exclusão do borne errado) → estados
  separados; exclusão só do borne selecionado existente, rotulado.
- **P2** formulários semeados só no clique podiam reaplicar valor desfeito
  após Undo; criação e edição de condutor compartilhavam campos → formulários
  re-semeados do Core a cada revisão; campos de edição de condutor próprios.
- **P3** pontos de ponte obsoletos ao trocar de régua → limpos.
- **P3** varredura redundante O(pontes × condutores) em cada validação →
  removida (a regra já é garantida por `check_conductor_connection`).
- **Plausível, fechado:** `UpsertBridge` podia adotar conexão arbitrária →
  agora edita somente ponte existente na mesma régua (teste dedicado).

Não corrigidos (registrados): custo de formatação das listas por frame com
linhas virtualizadas; `physical_occurrence` de régua percorre pontos (só no
clique); sem unicidade de pontes paralelas; snapshot antigo com numeração
duplicada na mesma régua passa a ser recusado (sem migração).

## Correção durante homologação manual — GUI-PHY-01 (2026-11-02)

Defeito relatado pelo usuário: a janela "Engenharia física", aba Listas
técnicas, crescia verticalmente sem parar, ultrapassava a tela e voltava a
crescer após redimensionamento manual.

- **Causa-raiz:** `egui::Resize` (usado pela janela redimensionável) faz
  `desired_size = max(desired_size, last_content_size)` a cada frame em que não
  há arraste. A lista técnica usava `ScrollArea` com `auto_shrink([false,
  false])`, que ocupa toda a altura disponível; o separador e a linha de
  retorno (`feedback`) eram desenhados *depois* dela. O conteúdo medido ficava
  sempre ~28 px maior que a janela, que crescia esses 28 px por frame. Soltar
  o mouse reativava o mesmo ciclo. Reproduzido em teste: 872 → 900 → 928 … 939
  px numa tela de 900 px.
- **Correção (`physical_editor.rs`, somente layout):** a linha de retorno
  passou a um `TopBottomPanel::bottom` interno; o corpo da aba fica num
  `CentralPanel` interno que ocupa exatamente o espaço restante. Abas Réguas,
  Cabos e Condutores rolam num `ScrollArea::both` próprio; as listas técnicas
  (já virtualizadas) rolam nos dois eixos, e o cabeçalho monoespaçado é
  truncado, para que linhas longas não imponham largura mínima. Linhas de abas
  usam `horizontal_wrapped`. Tamanho mínimo da janela 420×240; o limite à área
  disponível continua sendo o `constrain` padrão do egui. Nenhuma regra,
  comando ou dado foi alterado.
- **Outras janelas:** as quatro abas da Engenharia física compartilham a mesma
  janela e foram cobertas. "Resultados de comandos" (`ScrollArea` com
  `max_height` 320) e Backstage (não redimensionável) não têm área de altura
  total seguida de conteúdo; sem o defeito.
- **Regressões** em `crates/cad-gui/tests/physical_ui.rs`:
  `physical_window_size_is_stable_across_frames_on_every_tab` (40 frames por
  aba com retorno visível, janela dentro da tela) e
  `manual_resize_of_physical_window_is_preserved` (arraste real do canto
  para reduzir e ampliar; tamanho mantido após soltar, por 40 frames e na
  troca de todas as abas).
- **Validação:** fmt, check, clippy `-D warnings`, `git diff --check`
  aprovados; **391 testes Rust, 0 falhas/ignorados** (389 + 2). PowerShell
  segue não executado (PHY-013). Release não reconstruído após esta correção.

## Regressão da correção — GUI-PHY-02 (2026-11-02)

Relatado pelo usuário (captura e vídeo): após GUI-PHY-01 os controles eram
desenhados fora da janela, sobre a área de desenho; o fundo cobria apenas a
linha de abas.

- **Causa-raiz:** `CentralPanel::show_inside` (e `TopBottomPanel::show_inside`)
  desenham num `ui.new_child` com retângulo próprio, sem alocar esse espaço no
  `Ui` pai. A janela redimensionável mede o conteúdo pelo `min_rect` do seu
  `Ui`, que passou a conter só a linha de abas; no primeiro frame
  (passo de dimensionamento do `Area`) a janela assumiu ~81 px de altura e a
  moldura/fundo foram pintados nesse tamanho, enquanto o painel central pintava
  os controles no retângulo disponível com clip próprio, além da moldura. Com a
  linha de retorno visível o painel inferior expandia o `Ui` até a base e
  mascarava o defeito — por isso os testes de GUI-PHY-01 não o detectaram.
- **Correção (`physical_editor.rs`, somente layout):** painéis removidos.
  `Layout::bottom_up` posiciona a linha de retorno (rótulo + separador, mesma
  aparência e posição de antes) e um `Layout::top_down` aninhado ocupa
  exatamente o restante com o `ScrollArea` da aba. Ambos alocam no `Ui` da
  janela, então o conteúdo medido é igual à área da janela: sem crescimento,
  sem colapso, fundo cobrindo tudo. Mantidos: listas técnicas virtualizadas com
  rolagem nos dois eixos, cabeçalho truncado, abas com quebra e mínimo 420×240.
- **Testes ampliados** (`crates/cad-gui/tests/physical_ui.rs`), substituindo os
  dois de GUI-PHY-01:
  - `physical_window_size_is_stable_and_bounded_on_every_tab`: sem e com linha
    de retorno, em todas as abas: janela não colapsa (tamanho padrão), não cresce
    em 40 frames, fica na tela e passa por `assert_contained`.
  - `manual_resize_of_physical_window_is_preserved_and_bounded`: arrastes reais
    do canto (reduzir duas vezes e ampliar), tamanho preservado após soltar e
    na troca de abas, com `assert_contained` em cada tamanho.
  - `assert_contained` localiza o fundo da janela nas formas pintadas
    (`FullOutput.shapes`), exige que ele cubra a área da janela e que a parte
    visível (limites ∩ clip) de toda forma posterior esteja dentro dela.
  - Contraprova: reintroduzindo o `CentralPanel`, os dois testes falham
    ("window collapsed to … 266.0"); sem a verificação de tamanho,
    `assert_contained` falha sozinho ("content painted at … outside window").
- **Validação:** fmt, check, clippy `-D warnings`, `git diff --check`
  aprovados; **391 testes Rust, 0 falhas/ignorados**. Build Release
  **não concluído**: "Acesso negado (os error 5)" ao substituir
  `target/release/cad-gui.exe`, provavelmente em uso pelo aplicativo aberto;
  repetir com o aplicativo fechado.

## Validação manual necessária (não executada)

- Ciclos de movimento horizontal/vertical e rotações 180°/270° no editor real.
- Fluxos completos dos editores físicos, navegadores e listas com mouse/teclado.
- Leitura das listas em resoluções/DPI diferentes.

## Limitações e pendências

- Tipos especiais de borne não declarados (PHY-010); associação exige bornes
  explícitos nos pontos (PHY-011); custo linear de revalidação (PHY-012).
- `pwsh` ausente nesta sessão: os 15 testes PowerShell do `validate.ps1` não
  foram executados (PHY-013); os demais passos foram executados um a um.
- Relatórios completos (Etapa 13), formato `.cadcraft`, autosave e backups
  avançados permanecem futuros. Etapa 17 (plugin CAD-CRAFT AI, OpenAI)
  preservada. Melhoria futura da densidade visual da grade preservada.

## Git

Consulta somente leitura em 2026-11-02T19:50-03:00: branch `main`, HEAD
`a4f8233799292741c5e93bb65aec28983f1c9d41` (Etapa 9). Toda a Etapa 10 está
fora desse HEAD. Nenhum add/commit/push/fetch executado. Etapa 10 é par:
**commit + push manuais do usuário**; backup externo independente recomendado.

Criados: `crates/cad-craft/src/domain/terminal_bridge.rs`,
`crates/cad-craft/src/domain/project/{physical_edit,physical_lists,route_clearance}.rs`,
`crates/cad-craft/tests/{physical_edit,physical_engineering}.rs`,
`crates/cad-gui/src/physical_editor.rs`,
`crates/cad-gui/tests/{physical_history,physical_ui}.rs`,
`docs/adr/0012-engenharia-fisica-bornes-cabos.md`,
`docs/internal/spec/ETAPA_10_BORNES_CABOS_CONDUTORES.md`, `docs/internal/plans/2026-11-02-etapa-10.md`,
este relatório. (Também não rastreados, anteriores à etapa:
`docs/internal/plans/2026-11-02-ai-memory.md`, `docs/internal/reports/AI_MEMORY_INSTALACAO_2026-11-02.md`.)

Modificados — Core: `cable.rs`, `cable_core.rs`, `conductor.rs`,
`connection_drawing.rs`, `mod.rs`, `terminal.rs`, `terminal_strip.rs`,
`project.rs`, `project/{connection_drawing,connectivity,integrity,physical_integrity,physical_wiring,snapshot}.rs`,
`tests/connection_drawing.rs`. GUI: `app.rs`, `commands.rs`,
`connection_properties.rs`, `connection_view.rs`, `history.rs`,
`interaction.rs`, `lib.rs`, `panels/{mod,navigator,properties}.rs`,
`preferences.rs`, `state.rs`. Docs: `CURRENT_STATE.md`, `HANDOFF.md`,
`KNOWN_ISSUES.md`, `DECISIONS.md`, `ROADMAP.md`, `ARCHITECTURE.md`.

Excluídos: nenhum. Sem nova dependência; `Cargo.lock` inalterado.
