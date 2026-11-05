# Bloco R2 — documentação de entrada

Data: 2026-11-05, America/Sao_Paulo. Prompt aprovado: ../spec/R2_DOCUMENTACAO_ENTRADA.md.
Plano com andamento e evidências: ../plans/2026-11-05-r2.md.

## Entrega

- docs/ARCHITECTURE.md: oito crates, dependências, modelo, deltas, undo/redo, display list, fronteira de rede e ciclo de arquivos.
- docs/TOUR.md: percurso model → format → render → export → import → GUI, com arquivos-chave e notas da fachada/assistant.
- docs/GLOSSARY.md: termos de domínio com português ao lado; distingue entidade lógica, física e ocorrência.
- docs/FORMATS.md: versões 0–9 e fixtures, migrações, quatro bibliotecas nativas, leitores/escritores e perdas/reports próprios.
- Links para os quatro guias nos dois READMEs. CONTRIBUTING descreve a execução dos exemplos na validação.
- Seis exemplos em crates/cad-craft/examples, 29–63 linhas cada após fmt, todos com SPDX, main testado e entradas sintéticas: criar/salvar/reler projeto, inserir símbolo/display list/undo, conectar pontos/rede, numerar/desfazer, PDF temporário, importar QET/relatório.
- Fixture tests/fixtures/synthetic/entry.qet reutilizada por um teste de importação. Nenhuma amostra real ou rede.
- 75 introduções //! novas, três introduções existentes atualizadas; teste em tests/module_docs.rs cobre todos os arquivos de src (158 no checkout atual).
- validate.ps1 executa cargo test -p cad-craft --examples --locked; CI Windows usa o script e Linux executa esse comando explicitamente. Compilação continua coberta por all-targets.

## Verificação

Validação 2026-11-05: pwsh -NoProfile -File scripts/validate.ps1 terminou com exit 0: **692 testes Rust do workspace + 6 exemplos executados + 16 PowerShell = 714**, zero falhas/ignorados. fmt, check de todos os targets, Clippy com -D warnings e diff checks passaram; saídas de referência idênticas.

| Verificação | Resultado |
|---|---|
| cargo fmt --check | Aprovado |
| cargo check --workspace --all-targets --locked | Aprovado |
| cargo test --workspace --locked | 692 aprovados; 0 falhas; 0 ignorados |
| cargo test -p cad-craft --examples --locked | 6 exemplos executados; 0 falhas |
| cargo clippy --workspace --all-targets --locked -- -D warnings | Aprovado |
| scripts/tests/check-repository.Tests.ps1 | 16 aprovados; 0 falhas |
| git diff --check e git diff --cached --check | Aprovados |

Baseline outputs_match_the_recorded_baseline aprovado; hashes/fixtures de saída não foram regenerados. A suíte cresceu de 689 para 692 testes do workspace (2 de introduções e 1 da fixture QET), mais os 6 exemplos. Log completo: .local/r2-validate.log. Nenhum teste anterior removido ou enfraquecido.


Evidências focadas em .local/ (ignoradas, não entregues ao Git):

- r2-module-red.log: novo scanner falhou por ausência das introduções (1 falha esperada, teste auxiliar passou).
- r2-module-green.log: 2 testes aprovados após introduções.
- r2-examples.log: 6 exemplos executados e aprovados. A primeira execução encontrou uma suposição errada no tutorial sobre rótulo automático da bobina; corrigida para verificar geometria na camada Symbols, conforme o renderer.
- r2-fixture.log: fixture QET válida, com texto importado e perda de shape relatada.
- Todos os links relativos dos quatro guias e READMEs foram conferidos por existência do destino.
- Comparação com HEAD removendo somente linhas //! e normalizando CRLF/LF confirmou que os 78 fontes de produção alterados diferem exclusivamente em comentários de módulo. Nenhum item movido/renomeado, dependência ou formato alterado.

Revisão independente: review_r2, somente leitura. Um P2 corrigido: glossário descrevia acoplamento como documental sem continuidade; ConnectorMating e plan_mating_connections demonstram criação de conexões de posse para pares com pontos. O teste existente mating_is_explicit_validated_and_gives_continuity integra a validação completa. Sem outros achados materiais ou menores pendentes. Revisor deixou validação completa e homologação GUI ao executor/autor; R2 exige validação automática, dispensa teste no app e não declara homologação.

## Limites e Git

Build/check de desenvolvimento e testes previstos pelo script; nenhum build Release ou execução do aplicativo solicitado neste bloco documental. A validação local não equivale a CI remoto ou homologação manual. Limites de formatos/GUI preexistentes mantidos; nenhum novo problema funcional identificado. Nenhuma alteração intencional de saída ou regeneração de baseline.

Branch main. Último HEAD observado por git rev-parse HEAD em 2026-11-05: 06a6cbf1e3d37d1e7679ca55d98f41f43e7ee001.
Origin existente: https://github.com/Vandecao/CAD-CRAFT1.git. Estado remoto não consultado nesta entrega; tracking local não é prova de sincronização. R2 permanece fora do HEAD, sem staging, commit, push, outro Git mutável, sincronização com público ou backup criado/verificado. Bloco validado e pronto para revisão/versionamento; versionamento depende do OK expresso do autor. Não iniciar R3.

## Inventário

Criados: quatro guias acima; examples/01_new_project.rs, 02_place_symbol.rs, 03_connect.rs, 04_number.rs, 05_export_pdf.rs, 06_import_qet.rs (sob crates/cad-craft); crates/cad-craft/tests/module_docs.rs; crates/cad-craft/tests/fixtures/synthetic/entry.qet; docs/internal/plans/2026-11-05-r2.md e este relatório.

Modificados: .github/workflows/ci.yml, scripts/validate.ps1, README.md, README.pt-BR.md, CONTRIBUTING.md, crates/cad-craft/tests/qet_import.rs; memória HANDOFF.md, CURRENT_STATE.md, ROADMAP.md, KNOWN_ISSUES.md e DECISIONS.md. Nenhuma exclusão.

Fontes de produção (apenas //!):

~~~text
crates/cadcraft-format/src/lib.rs
crates/cadcraft-gui/src/app.rs
crates/cadcraft-gui/src/backstage.rs
crates/cadcraft-gui/src/commands.rs
crates/cadcraft-gui/src/device_properties.rs
crates/cadcraft-gui/src/dialogs.rs
crates/cadcraft-gui/src/editing.rs
crates/cadcraft-gui/src/panels/insertion.rs
crates/cadcraft-gui/src/panels/messages.rs
crates/cadcraft-gui/src/panels/mod.rs
crates/cadcraft-gui/src/panels/navigator.rs
crates/cadcraft-gui/src/panels/preview.rs
crates/cadcraft-gui/src/panels/properties.rs
crates/cadcraft-gui/src/preferences.rs
crates/cadcraft-gui/src/reference_properties.rs
crates/cadcraft-gui/src/ribbon.rs
crates/cadcraft-gui/src/snapshots.rs
crates/cadcraft-gui/src/state.rs
crates/cadcraft-gui/src/theme.rs
crates/cadcraft-gui/src/viewport.rs
crates/cadcraft-import/src/edz/parts.rs
crates/cadcraft-import/src/lib.rs
crates/cadcraft-import/src/parts_csv.rs
crates/cadcraft-model/src/document_index.rs
crates/cadcraft-model/src/domain/cable.rs
crates/cadcraft-model/src/domain/cable_core.rs
crates/cadcraft-model/src/domain/component.rs
crates/cadcraft-model/src/domain/conductor.rs
crates/cadcraft-model/src/domain/connection.rs
crates/cadcraft-model/src/domain/connection_point.rs
crates/cadcraft-model/src/domain/connector.rs
crates/cadcraft-model/src/domain/designation.rs
crates/cadcraft-model/src/domain/device.rs
crates/cadcraft-model/src/domain/document.rs
crates/cadcraft-model/src/domain/document_structure.rs
crates/cadcraft-model/src/domain/function.rs
crates/cadcraft-model/src/domain/geometry.rs
crates/cadcraft-model/src/domain/ids.rs
crates/cadcraft-model/src/domain/network.rs
crates/cadcraft-model/src/domain/numbering.rs
crates/cadcraft-model/src/domain/page.rs
crates/cadcraft-model/src/domain/page_graphic.rs
crates/cadcraft-model/src/domain/part.rs
crates/cadcraft-model/src/domain/physical_elements.rs
crates/cadcraft-model/src/domain/physical_values.rs
crates/cadcraft-model/src/domain/project.rs
crates/cadcraft-model/src/domain/project/connection_drawing.rs
crates/cadcraft-model/src/domain/project/connectivity.rs
crates/cadcraft-model/src/domain/project/connectors.rs
crates/cadcraft-model/src/domain/project/document_queries.rs
crates/cadcraft-model/src/domain/project/documentation.rs
crates/cadcraft-model/src/domain/project/function_edit.rs
crates/cadcraft-model/src/domain/project/identification.rs
crates/cadcraft-model/src/domain/project/integrity.rs
crates/cadcraft-model/src/domain/project/markers.rs
crates/cadcraft-model/src/domain/project/numbering.rs
crates/cadcraft-model/src/domain/project/parts.rs
crates/cadcraft-model/src/domain/project/physical_edit.rs
crates/cadcraft-model/src/domain/project/physical_integrity.rs
crates/cadcraft-model/src/domain/project/physical_lists.rs
crates/cadcraft-model/src/domain/project/physical_queries.rs
crates/cadcraft-model/src/domain/project/physical_wiring.rs
crates/cadcraft-model/src/domain/project/reports.rs
crates/cadcraft-model/src/domain/project/representations.rs
crates/cadcraft-model/src/domain/project/snapshot.rs
crates/cadcraft-model/src/domain/project/symbols.rs
crates/cadcraft-model/src/domain/properties.rs
crates/cadcraft-model/src/domain/report.rs
crates/cadcraft-model/src/domain/representation.rs
crates/cadcraft-model/src/domain/symbol.rs
crates/cadcraft-model/src/domain/symbol_library.rs
crates/cadcraft-model/src/domain/terminal.rs
crates/cadcraft-model/src/domain/terminal_bridge.rs
crates/cadcraft-model/src/domain/terminal_strip.rs
crates/cadcraft-model/src/domain/transform.rs
crates/cadcraft-model/src/error.rs
crates/cadcraft-model/src/physical_index.rs
crates/cadcraft-render/src/display.rs
~~~

Preservados os ajustes preexistentes de HANDOFF.md e os prompts não rastreados R2–R6 e Etapa 28; não são código de etapas implementadas nesta sessão. O prompt R2 autoriza esta entrega; nenhum dos blocos seguintes foi iniciado.
