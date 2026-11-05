# Etapa 17 (Fase B) — biblioteca externa e editor de símbolos

**Estado:** implementada e validada automaticamente; pronta para revisão.
Sem teste no executável nesta etapa (cadência: fim da Etapa 19, com
autorização do usuário para usar o computador). Homologação geral na Etapa
28. Sem commit/push até o OK do usuário.

Prompt: `docs/internal/spec/ETAPA_17_BIBLIOTECA_EDITOR_SIMBOLOS.md` (aprovado em
2026-11-03 com as decisões: `.cadsym` por símbolo; `.elmt` 0,5 mm/px
ajustável; "Atualizar símbolos do projeto" incluído; estilos/elipse fora das
molduras). Plano e retomada: `docs/internal/plans/2026-11-03-etapa-17.md`. Decisão:
ADR-0019. ai-memory: `notes/etapa-17-biblioteca-editor-simbolos.md`.
Base: `4a32b54`, 513 Rust + 15 PowerShell.

## Implementado

### Core (`cad-craft`)

- `domain/symbol.rs`: `SymbolPrimitive::Ellipse` e `DynamicText`;
  `SymbolTextField` (designação, função, rótulo de porta, referência,
  propriedade); `LineDash` e `PrimitiveStyle` (traço, espessura,
  preenchimento só em formas fechadas); `SymbolDefinition.styles` por
  índice (omitido quando vazio), `with_styles`, `edited` (versão + 1),
  `with_identity`; `translated`, `is_closed`, `kind_label`.
- `domain/page_graphic.rs`: estilo no conteúdo livre; `GraphicKind::Ellipse`.
- `project/symbol_texts.rs`: `Project::symbol_text` resolve os textos
  dinâmicos na ocorrência.
- `project/symbols.rs`: `update_symbol_definition` (mesmos IDs e vínculos;
  recusa porta em uso ausente, orientação não permitida ou projeto inválido),
  `symbol_update_problems`, `restore_symbol_update`, `SymbolUpdateDelta`,
  `add_symbol_definition`, `symbol_occurrences`.
- `persistence`: `.cadcraft` formato 5 / `min_reader` 5 (migração 4→5
  identidade; fixture `etapa17-v5.cadcraft`; v0–v4 intactas);
  `.cadsym` (`write_symbol_package`/`read_symbol_package`, formato 1).
- `interchange/elmt.rs`: importação `.elmt` (linha, retângulo, elipse,
  círculo, arco, polígono, texto, texto dinâmico `label` → designação e
  `ElementInfo` → propriedade, terminal → porta n/e/s/o, estilos, nomes
  multilíngues, escala ajustável, relatório).

### GUI (`cad-gui`)

- `symbol_view.rs`: desenho com estilos (tracejado em tela, espessura,
  preenchimento), elipse, texto dinâmico com alinhamento e rotação; hit
  test da elipse e do texto dinâmico.
- Folha: textos dinâmicos resolvidos por ocorrência (ex.: `-K1`); conteúdo
  livre com estilo; comando Inserir > Gráfico > Elipse; painel do conteúdo
  livre com traço e preenchimento.
- `symbol_editor.rs`: editor (Selecionar, Linha, Polilinha, Polígono,
  Retângulo, Círculo, Elipse, Arco, Texto, Texto dinâmico, Porta), mover
  arrastando, Delete, Desfazer/Refazer próprios, propriedades da primitiva
  (traço, espessura, preenchimento, texto, campo, altura, rotação) e da
  porta (ID, identificação, orientação), "Salvar na biblioteca",
  "Atualizar no projeto", "Fechar editor".
- `symbol_files.rs`: pasta da biblioteca (Configurações, recarga ao mudar),
  importar/exportar `.cadsym`, importar `.elmt` (arquivos ou pasta, escala
  em Configurações), "Atualizar símbolos do projeto" (compatíveis num passo;
  lista os recusados).
- Centro de inserção: catálogo projeto + biblioteca + embutidos (cache por
  revisão); inserção e prévia usam o símbolo de qualquer origem.
- Comandos em Dados-mestre > Símbolos: Novo símbolo, Editar símbolo,
  Exportar símbolo, Importar símbolo, Importar .elmt…, Importar pasta
  .elmt…, Recarregar biblioteca, Atualizar símbolos do projeto.
- Histórico: `Transaction::Symbols` (várias atualizações num passo).

## Critérios de aceitação

| # | Critério | Evidência | Resultado |
|---|---|---|---|
| 1 | Criar símbolo do zero por eventos reais, salvar, inserir com Novo dispositivo, `-K1` no texto dinâmico | `symbol_editor_ui.rs::symbol_created_from_scratch_is_saved_inserted_and_shows_designation` (retângulo, Ctrl+Z/Ctrl+Y, 2 portas, elipse, texto dinâmico, arrastar, nome digitado, "Salvar na biblioteca", catálogo, inserção, `-K1` desenhado, 2 portas ligadas) | Aprovado |
| 2 | Elipse, polígono preenchido, tracejado; hit test em dois zooms | `symbol_render.rs` (texto `-H1`, preenchimento, tracejado desenha menos comprimento; hit da elipse e do texto); `free_graphic_ui.rs::ellipse_tool_and_style_survive_edits` | Aprovado |
| 3 | Atualizar com portas compatíveis; recusa com porta removida; Desfazer/Refazer | `editing_a_used_symbol_updates_project_or_refuses_missing_ports`; Core `symbol_update_keeps_occurrences_and_refuses_missing_ports` | Aprovado |
| 4 | `.cadsym` idêntico; inválidos recusados | `cadsym_round_trip_and_invalid_files_are_refused`; `library_folder_imports_exports_and_updates_project_symbols` | Aprovado |
| 5 | `.elmt`: ≥ 50 elementos sem erro, portas e textos | `elmt_import.rs`: amostra de 200 elementos da coleção de referência sem erro (maioria com portas); elemento sintético com todas as primitivas; inválidos/XXE recusados | Aprovado |
| 6 | Salvar/reabrir; v0–v4 migram; v5 versionada | `versioned_fixtures_keep_opening` | Aprovado |
| 7 | Testes preservados, validação, Release, docs | Abaixo; sem teste no app (cadência) | Aprovado |

## Validação

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-03), exit 0:
**526 testes Rust** (513 anteriores + 13 novos: Core `symbol_editing.rs` 4,
`elmt_import.rs` 3; GUI `symbol_render.rs` 2, `symbol_editor_ui.rs` 3,
`free_graphic_ui.rs` 1), 0 falhas/ignorados; **15 PowerShell**; fmt, check,
clippy `-D warnings`, `git diff --check`. Testes anteriores ajustados por
mudança legítima: versão de formato (`page_graphics.rs`, `persistence.rs`)
e catálogo do Centro de inserção (teste unitário de `insertion.rs`).

Release: `target/release/cad-gui.exe`, 13.836.800 bytes, SHA-256
`2297839492ff1b97c17421be59443f0917c4d273d599780b2b60640aaec6f8e4`;
não executado.

Defeitos achados pelos testes e corrigidos: dois cliques rápidos no editor
eram tratados como duplo clique (encerrar polilinha) e perdiam o ponto;
editar conteúdo livre (painel ou duplo clique) recriava o gráfico sem o
estilo.

## Limitações

- Preenchimento de polígono não convexo é aproximado (desenho convexo).
- `.elmt`: setas de linha, cantos arredondados, cores e arcos elípticos são
  aproximados; textos fixos girados ficam na horizontal (relatório avisa).
- Sem pasta configurada, a biblioteca existe só na sessão.
- Texto dinâmico de propriedade lê propriedades personalizadas do
  dispositivo (o cadastro de peças é da Etapa 18).
- Um símbolo editado que ainda não está no projeto vai para a biblioteca;
  entra no projeto ao ser inserido.

## Arquivos

Criados: `crates/cad-craft/src/domain/project/symbol_texts.rs`,
`crates/cad-craft/src/interchange/elmt.rs`,
`crates/cad-craft/tests/{symbol_editing,elmt_import}.rs`,
`crates/cad-craft/tests/fixtures/etapa17-v5.cadcraft`,
`crates/cad-gui/src/{symbol_editor,symbol_files}.rs`,
`crates/cad-gui/tests/{symbol_editor_ui,symbol_render}.rs`, prompt, plano,
ADR-0019, este relatório. Modificados: Core `domain/{symbol,page_graphic}.rs`,
`domain/project.rs`, `project/symbols.rs`, `interchange/mod.rs`,
`persistence.rs`, `tests/{persistence,page_graphics}.rs`; GUI `app.rs`,
`backstage.rs`, `commands.rs`, `editor_view.rs`, `frame_editor.rs`,
`history.rs`, `lib.rs`, `panels/{insertion,properties}.rs`,
`preferences.rs`, `project_file.rs`, `state.rs`, `symbol_view.rs`,
`viewport.rs`, `tests/free_graphic_ui.rs`; `docs/internal/memory/*`.
Excluídos: nenhum.

## Git

`main`, HEAD `4a32b54c9af59c4a1048518c3e379791d48c02f0` (consulta de
2026-11-03); toda a Etapa 17 fora do HEAD. Commit + push somente após o OK
expresso do usuário. Não iniciar a Etapa 18.
