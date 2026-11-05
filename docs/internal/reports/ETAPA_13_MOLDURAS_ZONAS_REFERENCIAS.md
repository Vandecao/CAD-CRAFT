# Etapa 13 (Fase A) — modelo de moldura, zonas e referências

**Estado:** implementada e validada automaticamente; pronta para revisão.
Homologação manual geral na Etapa 28. Sem commit/push até o OK do usuário.

Prompt: `docs/internal/spec/ETAPA_13_MOLDURAS_ZONAS_REFERENCIAS.md` (aprovado em
2026-11-03). Plano: `docs/internal/plans/2026-11-03-etapa-13.md`. Decisão: ADR-0015.
Base: `main`, HEAD `6612e82be8aea4219124a2497b06898498b01b7a` (Etapa 12),
452 Rust + 15 PowerShell.

## Implementado

### Core (`cad-craft`)

- `domain/frame.rs`: `FrameTemplate` (folha, área útil, zonas, elementos,
  versão), `ZoneLayout`/`ZoneAxis` (tamanhos individuais, `Numbers`/`Letters`
  com A…Z, AA…, lados), `FrameElement` (linha, retângulo, círculo, arco,
  texto fixo, texto especial com índice), `FrameTextStyle` (altura, 9
  alinhamentos, rotação), `LocalizedText`, `SpecialField` (16 campos),
  `Zone`, `ReferenceFormat`, `PaperFormat` (A0–A4, `matching`), molduras de
  fábrica `factory_a3()` (10 colunas de 41 mm) e `factory_a4()` (5 de 40 mm).
- `domain/project/frames.rs`: `frame`, `frames`, `default_frame`,
  `page_frame`, `zone_at`, `reference_label`, `resolve_special`,
  `edit_frame`/`restore_frame_edit` (`FrameEdit`/`FrameDelta`), validação.
- `Page.frame` + `Page::with_frame`; `Project.frames/default_frame/
  reference_format` com serialização omitida quando vazia/padrão;
  `DeviceReference.reference`; `check_page` exige moldura existente.
- `persistence`: `FORMAT = 2`, `MIN_READER = 2`, migração 1→2,
  `LoadedProject::is_legacy_snapshot()`.

### GUI (`cad-gui`)

- `frame_view.rs`: desenho de geometria (espessura em escala), textos com
  alinhamento e rotação, textos especiais resolvidos, rótulos de zona nas
  faixas da borda, referências cruzadas na folha.
- Folha: moldura substitui margem/cabeçalho provisórios quando existe.
- Projeto novo: molduras de fábrica, A3 padrão; página nova recebe a moldura
  do mesmo tamanho (A4) quando a padrão não coincide.
- Propriedades — projeto: moldura padrão, formato de referência, metadados
  editáveis (Autor, Cliente, Desenhista, Data, Revisão + existentes);
  página: moldura (padrão ou própria, aviso de tamanho diferente), formato
  A0–A4 paisagem/retrato, metadados (Data, Desenhista, Revisão + existentes).
- Diálogo Nova página: formato A0–A4.
- Navegador, propriedades do dispositivo e marcadores: referência por coluna.
- Histórico: `Transaction::Frame`; edições de moldura mantêm a seleção
  (defeito encontrado no teste de aceitação e corrigido).

## Critérios de aceitação

| # | Critério | Evidência | Resultado |
|---|---|---|---|
| 1 | Projeto novo com A3; página herda; página com outra moldura; Undo/Redo | `frames_ui.rs::new_project_frames_zones_special_texts_and_column_references` (botão e diálogo reais, A4 recebe moldura própria, Ctrl+Z/Y exatos); `properties_panel_assigns_page_frame_and_edits_project_metadata` (seletor do painel com cliques reais); `frames.rs::frame_edits_undo_and_redo_exactly` | Aprovado |
| 2 | Textos especiais refletem nome/número sem editar a moldura | `special_texts_resolve_project_page_and_indexed_properties`; GUI: "Painel CCM", "Comando", "Esquema" na folha; renumerar página → "/20.2"; campo Autor → "Ana" no carimbo | Aprovado |
| 3 | Coluna correta com larguras iguais/diferentes e bordas | `zones_with_different_widths_and_edges_are_exact`, `factory_frames_are_valid_and_divide_the_usable_area`, `reference_labels_follow_frame_zones_and_format` | Aprovado |
| 4 | Trocar moldura recalcula referências | `reference_labels_follow_frame_zones_and_format` ("/2.3" → "/2.5") e fluxo GUI | Aprovado |
| 5 | Salvar/reabrir; v1 migra; fixtures | `frames_round_trip_and_dangling_frame_references_are_refused`, `versioned_fixtures_keep_opening` (v0/v1 da Etapa 12 intactas + `etapa13-v2.cadcraft`), GUI salva/lê formato 2 | Aprovado |
| 6 | Fluxo egui: A3/A4, bobina e contato em páginas diferentes, referência no painel e na folha | `new_project_frames_…`: "/1.4" e "/2.2" no Core, na folha das duas páginas e no painel do dispositivo; rótulos de coluna desenhados | Aprovado |
| 7 | Testes preservados, validação, Release, docs | Abaixo | Aprovado |

## Validação

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-03), exit 0:
**466 testes Rust** (452 anteriores + 14 novos: `frames.rs` 11, unitário
`frame` 1, `frames_ui.rs` 2), 0 falhas/ignorados; **15 PowerShell**; fmt,
check, clippy `-D warnings`, `git diff --check`.
Release final (após as correções de texto): `target/release/cad-gui.exe`, 12.061.696 bytes, SHA-256
`afcbeebcf990bd671dac2675602557d115fa2a5170fe66fb187a45096651f080`.

Ajuste de layout: os seletores novos do painel Propriedades usam largura fixa
de 100 pt com truncamento; com outras larguras a área de desenho variava
0,3 pt entre seleções e quebrava testes de inserção existentes.

## Validação funcional na interface (agente, 2026-11-03)

Pedido do usuário: rodar o CAD-CRAFT e testar a etapa. Executável Release
desta etapa operado pelo agente com controle do computador (não é a
homologação manual do usuário, prevista para a Etapa 28).

| Passo | Resultado |
|---|---|
| Criar projeto pela tela inicial | Moldura padrão "CAD-CRAFT A3 paisagem", referência `/página.coluna`, campos Autor/Cliente/Desenhista/Data/Revisão no painel |
| Autor "Ana Souza" no painel | Gravado no histórico; aparece no carimbo das páginas |
| Documento "Esquema"; página 1 "Força" A3 e página 2 "Comando" A4 retrato pelo diálogo (seletor Formato) | Página 1 herda a A3; página 2 recebe "CAD-CRAFT A4 retrato" automaticamente; rótulos de coluna 1…10 / 1…5 |
| Carimbo da página 2 | "Validacao Etapa 13", "Comando", "Ana Souza", "2", "Esquema", folha 2 de 2, nome da moldura |
| Bobina -K1 (novo dispositivo) em X 152 na página 1; contato como função de -K1 em X 64 na página 2 | Folha: contato mostra `/1.4`, bobina mostra `/2.2`; navegador: "-K1 · Main · /1.4" e "-K1 · Auxiliary · /2.2" |
| Renumerar a página 2 para 20 | Bobina passa a `/20.2`; navegador "20 Comando"; carimbo "20" |
| Moldura da página 20 → A3 pelo seletor do painel | Aviso "Moldura com tamanho diferente da folha"; seleção mantida; Ctrl+Z volta à A4, Ctrl+Y refaz |
| Ctrl+S (diálogo nativo `rfd`), fechar, Ctrl+O | Arquivo formato 2/`min_reader` 2 com as duas molduras e a moldura própria da página 20; trava criada e liberada ao fechar; reabertura preserva molduras, autor, referências e carimbo |

Achados (não bloqueantes):
- Texto desatualizado no diálogo "Nova sessão em memória" ("O formato físico
  versionado ainda não está implementado…"): **corrigido** — título "Novo
  projeto" e orientação para salvar como `.cadcraft`.
- Backstage → Informações mostrava "Tipo de sessão: Projeto em memória" mesmo
  com arquivo associado: **corrigido** — mostra "Arquivo .cadcraft (nome)",
  "somente leitura" quando for o caso, ou "Projeto em memória (ainda não salvo)".
- Navegador de dispositivos exibe os papéis em inglês ("Main", "Auxiliary"),
  comportamento anterior à Etapa 13.
- Ambiente: com a janela maximizada, uma faixa à direita da tela não recebia
  cliques (janela de outro programa); a janela foi reposicionada para o teste.
  Seletores do egui exigiram mover o mouse antes do clique na automação.

## Limitações

- Sem editor gráfico de molduras, imagens/logotipos, marca d'água, QR ou
  hiperlink (Etapa 14); sem importação DXF/`.titleblock`/`.fn1` (Etapa 15).
- Textos fixos exibem o valor padrão (pt-BR); variantes por idioma são
  apenas preservadas.
- Moldura com tamanho diferente da folha é desenhada como está, com aviso.
- Referências na folha são recalculadas a cada quadro (custo linear nas
  representações do projeto); a lista de contatos ao lado da bobina é da
  Etapa 20.
- `next_device_designation`, numeração e caminhos (`=`/`+`) nas referências
  continuam simples.
- Arquivos gravados pela Etapa 13 não abrem na versão da Etapa 12
  (`min_reader` 2), por decisão do ADR-0014.

## Arquivos

Criados: `crates/cad-craft/src/domain/frame.rs`,
`crates/cad-craft/src/domain/project/frames.rs`,
`crates/cad-craft/tests/frames.rs`,
`crates/cad-craft/tests/fixtures/etapa13-v2.cadcraft`,
`crates/cad-gui/src/frame_view.rs`, `crates/cad-gui/tests/frames_ui.rs`,
`docs/internal/plans/2026-11-03-etapa-13.md`,
`docs/adr/0015-molduras-zonas-e-referencias.md`, este relatório.
Do planejamento desta sessão (ainda não versionados):
`docs/internal/plans/PLANO_MESTRE_2026-11-03.md`,
`docs/internal/spec/ETAPA_13_MOLDURAS_ZONAS_REFERENCIAS.md`.

Modificados: Core `domain/ids.rs`, `mod.rs`, `page.rs`, `identification.rs`,
`project.rs`, `project/identification.rs`, `project/integrity.rs`,
`project/snapshot.rs`, `persistence.rs`, `tests/persistence.rs`; GUI
`app.rs`, `device_properties.rs`, `dialogs.rs`, `editing.rs`, `history.rs`,
`lib.rs`, `panels/navigator.rs`, `panels/properties.rs`,
`reference_properties.rs`, `state.rs`, `viewport.rs`; `AGENTS.md`,
`docs/internal/memory/*` (estado, handoff, decisões, roadmap, problemas conhecidos,
arquitetura, ADR-0002 adendo).

Excluídos: nenhum.

## Git

`main`, base `6612e82`. Validação final após as correções: `scripts/validate.ps1`
exit 0, 466 Rust + 15 PowerShell. O usuário respondeu "Corrigir e versionar"
(2026-11-03): commit + push da Etapa 13 pelo agente, conforme AGENTS.md. O SHA
do commit não é registrado neste documento (consultar `git log`). Não iniciar
a Etapa 14.
