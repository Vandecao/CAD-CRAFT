# Relatório — Bloco R1 (divisão do Core em crates)

Data: 2026-11-05. Agente: Claude Code. Prompt: `docs/internal/spec/R1_DIVISAO_CRATES.md`, aprovado com as três decisões recomendadas. Plano: `docs/internal/plans/2026-11-05-r1.md`.

**Nenhum comportamento mudou.** As saídas de referência do R0 (`.cadcraft`, JSON, lista de exibição, PDF e DXF das fixtures v0–v9) continuam idênticas, sem regravar o arquivo de hashes.

## Entregue

| Crate | Conteúdo (movido com `git mv`) | Depende de |
|---|---|---|
| `cadcraft-model` | `domain/`, `error.rs`, `graph.rs`, `document_index.rs`, `physical_index.rs` | serde, uuid, serde_json |
| `cadcraft-format` | `persistence.rs` (raiz da crate) | model, serde, serde_json, zip, flate2 |
| `cadcraft-render` | `display.rs`, `font.rs` | model |
| `cadcraft-import` | `interchange/` (o antigo `mod.rs` vira `lib.rs`) | model, serde, serde_json, zip, flate2, quick-xml, csv, sevenz-rust2 |
| `cadcraft-export` | `pdf.rs`, `dxf_export.rs`, `report_export.rs` | model, render, pdf-writer, image, rust_xlsxwriter, csv, flate2 |
| `cad-craft` | fachada: reexporta com os caminhos de antes; mantém os testes de integração | as cinco acima |
| `cadcraft-gui` | antiga `cad-gui`; biblioteca `cad_gui` e executável `cad-gui.exe` mantidos | `cad-craft` |

Mudanças além de mover arquivos e ajustar `use`:
- `Project::page_display_list` virou o trait `PageDisplayList` (mesmo nome de método), porque métodos de `Project` só podem ficar na crate do modelo. A fachada o reexporta em `cad_craft::domain`. Precisaram de um `use` a mais: `pdf.rs`, `dxf_export.rs` e `tests/baseline_outputs.rs`.
- `report_rows_per_page` passou de `pub(crate)` a `pub` (`#[doc(hidden)]`), porque é usado pela exibição.
- A fachada ganhou como dependências de teste as bibliotecas que os testes usam diretamente (serde_json, uuid, zip, image).
- `Cargo.lock`: só as cinco crates novas e o nome novo da GUI; nenhuma versão externa nova.

Documentos e CI:
- README (inglês e pt-BR) com o mapa das crates;
- `AGENTS.md` com as regras de camadas;
- CI do Linux testando todas as crates do Core (`--workspace --exclude cadcraft-gui`).

## Validação

`scripts/validate.ps1` (2026-11-05) terminou com exit 0:
- **674 Rust** (673 + 1 novo, `crate_layers.rs`, que confere o sentido das dependências pelo `cargo metadata`);
- **16 PowerShell**;
- zero falhas.

Cada crate compila sozinha (`cargo check -p cadcraft-model --all-targets` e as demais).

Release: `target/release/cad-gui.exe` com 20.897.280 bytes. Antes eram 20.088.832; a diferença vem da otimização entre crates. O comportamento é garantido pelas saídas de referência, não pelo tamanho.

## Ocorrência

A primeira validação falhou por **disco cheio**: a pasta `target` acumulava 112 GB de artefatos de compilação. `cargo clean` liberou o espaço (108 GB livres) e a validação seguinte, compilando do zero, passou.

## Teste no app (2026-11-05, com autorização)

Com o executável Release novo:
- o app abriu normalmente;
- `industrial.qet` foi importado com a mesma prévia de antes da divisão (50 folhas, 290 dispositivos, 345 fios, 105 relatórios de folha);
- o projeto foi criado;
- o PDF do projeto inteiro foi exportado (50 páginas, arquivo válido).

O app foi fechado descartando o projeto de teste, sem deixar cópia de recuperação.

## Pendente

- Commit + push no privado e sincronização com o repositório público, com o OK do autor.
