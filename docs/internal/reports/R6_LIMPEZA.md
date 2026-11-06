# R6 — limpeza final

Especificação aprovada: [R6_LIMPEZA.md](../spec/R6_LIMPEZA.md). Plano: [2026-11-06-r6.md](../plans/2026-11-06-r6.md). Execução em 2026-11-06, no checkout privado C:/Dev/CAD-CRAFT, branch main. Sem mudança de dependências, formato nativo, dados serializados ou comportamento do produto.

## Alterações

Comentários de linha, API e módulo foram reescritos em inglês para explicar responsabilidades, invariantes e contratos, removendo rótulos históricos de etapas. O modelo teve 386 blocos de comentários traduzidos e a API pública antes sem documentação foi documentada, incluindo expansões dos macros de IDs. A fachada teve 171 blocos traduzidos; os outros crates e testes foram auditados integralmente, incluindo comentários inline.

missing_docs está ativo nos roots de cadcraft-model e cadcraft-format, com documentação de tipos, variantes, campos, construtores e operações públicas. O atributo de crate limita seu alcance aos dois crates pedidos, sem dispensar itens públicos por allow. O macro de quantidades ganhou um argumento separado para documentação em inglês: os literais pt-BR de validação e os invariantes numéricos permanecem idênticos. A revisão corrigiu descrições que confundiam designações de posição com coordenadas, traduções de texto com translações geométricas e referências de função com eixos de classificação.

strings.rs é privado da GUI e centraliza 44 frases repetidas em 173 ocorrências: ações comuns, títulos de diálogos e erros repetidos. Capitalização, acentos, pontuação e placeholders foram preservados. Textos únicos continuam locais. Um teste novo de AccessKit fixa os rótulos renderizados independentemente das constantes de produção: Novo documento, Nova página, Cancelar e Confirmar; cancelar deve manter o snapshot exato do projeto.

O workspace agora herda os avisos Clippy unwrap_used, dbg_macro, todo e print_stdout. A validação existente transforma avisos em erros. Seis unwraps de bibliotecas foram substituídos por expect com a razão do invariante: cinco conversões de tamanho previamente verificado do contêiner 7z e o último ponto de uma polilinha fechada com comprimento já verificado. O exemplo de inspeção render_qa teve doze substituições equivalentes, explicando as condições do cenário sintético. Doze diagnósticos dos testes da fachada passaram de stdout para stderr, mantendo o texto; a aplicação e seus arquivos não são afetados.

Foram removidos somente dois reexports sem consumidores externos, criados na divisão do editor de molduras: load_image_file e element_distance. Suas implementações e imports internos permanecem. Os caminhos públicos da fachada, os reexports dos importadores usados por QET/EDZ e os exports usados pelos testes foram preservados. A auditoria não encontrou outros shims temporários descartáveis.

## Conservação e limites

Auditorias de tokens cobriram os 373 fontes Rust originais: Core, GUI, fachada, testes e exemplos. As únicas normalizações permitidas são comentários/documentação, atributos novos de lint, expansão exata das constantes, nomes locais/testes históricos, os dezoito unwrap/expect explicados, os diagnósticos de teste em stderr e a formatação do rustfmt. Nenhuma asserção, entrada de teste, atributo Serde ou literal de formato/protocolo foi enfraquecido. Fixtures e baselines não foram regenerados. O maior arquivo do modelo tem 806 linhas após a documentação (antes 781), próximo da meta de aproximadamente 800; não houve nova divisão de API.

Exceções de idioma/história preservadas deliberadamente:

- O cabeçalho legal obrigatório mantém a atribuição canônica em português ao autor e colaboradores.
- Exemplos de protocolo preservam o marcador pagina:Revisão#2; nomes de fixtures e IDs históricos com etapa permanecem para não alterar os arquivos versionados.
- Literais observáveis da interface, dos relatórios de importação, dos metadados de demonstração e do relatório histórico EDZ não foram traduzidos ou renomeados. A retirada de citações de etapa se aplica a comentários, nomes de testes e diagnósticos de asserção, conforme o escopo aprovado; preservar dados e rótulos tem precedência sobre uma substituição textual indiscriminada.

A configuração de testes de Clippy segue a [documentação oficial](https://doc.rust-lang.org/stable/clippy/lint_configuration.html#allow-unwrap-in-tests). As permissões de unwrap ficam em contextos de teste; stdout é permitido somente nos exemplos executáveis. Não foi introduzida permissão geral em bibliotecas.

Os três exemplos compile_fail do modelo usam a fachada nos trechos visíveis. Foi adicionado um alias oculto de cadcraft-model para a fachada no contexto de doctest, evitando que um import não resolvido faça o teste passar pelo motivo errado. O toolchain estável foi mantido; a checagem de códigos de erro nas fences é recurso [nightly do rustdoc](https://doc.rust-lang.org/nightly/rustdoc/unstable-features.html#error-numbers-for-compile-fail-doctests), portanto não foi ativada.

## Verificação

O teste novo de rótulos passou isoladamente (1 aprovado, 8 filtrados), e Clippy de todos os alvos passou com -D warnings. A rodada inicial de lint demonstrou a detecção de stdout em testes e unwraps de helpers/exemplo; esses casos foram resolvidos com as fronteiras descritas acima. Revisão independente: nenhum achado material remanescente após corrigir as descrições de API e os comentários dos testes do assistente.

Em 2026-11-06, `pwsh -NoProfile -File scripts/validate.ps1` passou integralmente: **707 testes Rust do workspace + 6 exemplos + 16 testes PowerShell = 729**, zero falhas/ignorados. fmt, check de todos os alvos, Clippy com -D warnings, baselines e verificações de whitespace passaram. Após o ajuste exclusivamente documental dos aliases, os três doctests do modelo passaram novamente; a inspeção separada pelo rustc confirmou E0308 (tipos incompatíveis) em cada exemplo, sem import não resolvido.

`cargo doc --workspace --no-deps --locked` passou com `RUSTDOCFLAGS=-D warnings`. O primeiro passe encontrou `<uuid>` sem formatação de código em um comentário da GUI; o comentário foi corrigido e o passe completo repetido sem avisos. `cargo build -p cadcraft-gui --release --locked` passou. Executável: 23.736.832 bytes; SHA-256 `35C21049B2FC841EAC0BA3868EC8796D53E5C15E31FC7C5FD550CDB57AE05855`.

Teste rápido nativo no novo Release concluído pelo Codex em 2026-11-06 por teclado e AccessKit: tela inicial em pt-BR; recuperação preexistente cancelada sem restaurar/apagar sua cópia; Nova sessão abriu Novo projeto com Nome do projeto, Confirmar e Cancelar; Escape cancelou o diálogo e a tela retornou a Nenhum projeto aberto. Nenhum projeto do usuário foi alterado. A verificação confirma os rótulos e o cancelamento observáveis pela acessibilidade; não é homologação visual geral. A captura de tela do helper permanece limitada, e as observações de acessibilidade às vezes chegam atrasadas, exigindo atualização antes da próxima ação. O app ficou na tela inicial.

Evidências locais: .local/r6/{core.md,model.md,gui.md,facade-review.md}, scripts e resultados de conservação, gui-label-test.log, clippy-final.log, validate.log, model-doctests-final.log, doctest-type-errors.json, rustdoc-final.log e release.log. Amostras reais opcionais não foram repetidas no R6: os importadores conservam tokens executáveis após a normalização restrita dos expect; a equivalência de amostras reais foi registrada no R4.

Inventário de caminhos: [R6_ARQUIVOS.md](R6_ARQUIVOS.md).

## Git e continuidade

HEAD privado observado no início do R6 em 2026-11-06: 6b38bac26a9728917ba26572c7a1248a8c231b13 (R4/R5 publicados). Público: c92d81bbfb887e6f41c89186ee9a662cc8a93c6f. Alterações de R6 estão fora desses HEADs. Nenhum staging, commit, push, cópia pública, worktree ou alteração de refs foi executado no bloco R6. A autorização de publicação anterior se limitava a R4/R5; o versionamento deste bloco depende do OK por etapa exigido em AGENTS.local.md. Etapa 28 não iniciada; seu prompt permanece preservado fora da entrega.

## Exceções novas de lint

Cada atributo abaixo está limitado a um arquivo de teste ou exemplo. unwrap_used em testes preserva a preparação sintética com falha imediata; print_stdout nos seis exemplos permite apresentar o resultado do executável. Não há allow novo em biblioteca de produção, nem exceção a missing_docs, dbg_macro ou todo. A configuração allow-unwrap-in-tests=true cobre funções anotadas e módulos cfg(test); os atributos locais cobrem helpers de fixtures. allow-unwrap-in-consts=false mantém a proibição em constantes de produção.

- `crates/cad-craft/examples/01_new_project.rs`: `#![allow(clippy::print_stdout)]`
- `crates/cad-craft/examples/02_place_symbol.rs`: `#![allow(clippy::print_stdout)]`
- `crates/cad-craft/examples/03_connect.rs`: `#![allow(clippy::print_stdout)]`
- `crates/cad-craft/examples/04_number.rs`: `#![allow(clippy::print_stdout)]`
- `crates/cad-craft/examples/05_export_pdf.rs`: `#![allow(clippy::print_stdout)]`
- `crates/cad-craft/examples/06_import_qet.rs`: `#![allow(clippy::print_stdout)]`
- `crates/cad-craft/tests/baseline_outputs.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/checks.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/common/mod.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/connections.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/connection_drawing.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/connection_drawing_scale.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/connectivity.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/connectivity_scale.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/connectivity_serialization.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/connectors.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/crate_layers.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/document_common/mod.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/document_integrity.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/document_pages.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/document_representations.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/document_serialization.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/document_transactions.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/document_values.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/dxf_export.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/dxf_protocol.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/edz_import.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/elmt_import.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/frames.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/frame_editing.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/identification.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/interchange.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/license_headers.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/macros.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/malformed_imports.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/markers.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/module_docs.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/numbering.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/output_common/mod.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/page_graphics.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/parts.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/pdf_export.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/persistence.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/persistence_scale.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_common/mod.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_edit.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_engineering.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_insertion.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_scale.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_serialization.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_values.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/physical_wiring.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/placement.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/project_model.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/qet_import.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/reports.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/serialization.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/symbol_editing.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/symbol_geometry.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/symbol_project.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/terminal_kinds.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cad-craft/tests/value_types.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-assistant/tests/assistant.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/assistant_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/cad_export_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/camera_preferences.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/checks_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/connection_editor.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/connection_geometry.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/connection_history.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/connection_scale.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/connection_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/connector_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/editor_history.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/editor_interaction.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/editor_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/edz_import_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/frames_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/frame_editor_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/free_clipboard.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/free_graphic_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/interface.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/macros_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/numbering_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/pdf_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/physical_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/project_lifecycle.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/qet_import_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/references.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/reference_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/reports_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/session.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/symbol_editor_ui.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/symbol_geometry.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/symbol_history.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/symbol_render.rs`: `#![allow(clippy::unwrap_used)]`
- `crates/cadcraft-gui/tests/symbol_ui.rs`: `#![allow(clippy::unwrap_used)]`


**Versionamento autorizado em 2026-11-06:** o autor deu OK expresso para commit + push de R6 nos dois repositórios. Este documento registra o estado anterior à publicação; os HEADs acima são os pais observados, não os commits futuros. Resultado e SHAs reais serão registrados em `.local/r6/PUBLICATION.md` nos dois checkouts e podem ser confirmados com Git. Etapa 28 continua fora da entrega.
