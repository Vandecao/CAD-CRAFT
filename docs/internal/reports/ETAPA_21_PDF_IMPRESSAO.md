# Relatório — Etapa 21 (PDF multipágina com links e impressão)

Data: 2026-11-04. Agente: Claude Code. Prompt:
`docs/internal/spec/ETAPA_21_PDF_IMPRESSAO.md`, aprovado com as cinco decisões
recomendadas. Plano: `docs/internal/plans/2026-11-04-etapa-21.md`. ADR-0023.

## Entregue

| Item | Onde |
|---|---|
| Lista de exibição da página (mm): caminhos, textos com matriz, imagens, links, cores lógicas | `cad-craft/src/display.rs` (`Project::page_display_list`, `frame_zone_labels`) |
| Referências da folha com destino: cruzadas, espelho, interrupções | `domain/project/sheet_references.rs` (`sheet_references`, `short_reference`); `frame_view.rs` passa a usá-las |
| Métricas Helvetica WinAnsi | `cad-craft/src/font.rs` |
| Escritor de PDF | `cad-craft/src/pdf.rs` (`export_pdf`, `PdfOptions`, `PdfScope`, `PaperFit`, `PdfReport`) |
| Links e marcadores | anotações `/Link` `GoTo` com destino XYZ; árvore documento → páginas |
| Diálogo Exportar PDF / Imprimir | `cad-gui/src/pdf_export.rs`: alcance, preto e branco, marcadores, abrir ao terminar, papel A4/A3, ajustar/1:1, progresso, cancelamento, relatório |
| Sistema substituível | `SystemShell` (`NativeShell`: `cmd /c start` e `Start-Process -Verb Print`) |
| Comandos | Início > Sessão: Exportar PDF e Imprimir; Arquivo (backstage): Imprimir e Exportar PDF |
| Seletor de arquivo | `FilePicker::save_pdf` (nativo e `ScriptedPicker.pdf_save`) |

O formato do projeto não mudou: o `.cadcraft` continua no formato 7.

Dependências novas, aprovadas na decisão 1:
- `pdf-writer = "=0.15.0"` no Core;
- `lopdf = "=0.47.0"` só nos testes;
- `image` 0.25, com o recurso `png`, passou a ser usado também no Core. O crate já tinha sido aprovado na Etapa 14.

## Validação automática

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-04) terminou com exit 0. Etapas executadas, todas com sucesso:
- `cargo fmt --check`;
- `cargo check --workspace --all-targets --locked`;
- `cargo test --workspace --locked`: **586 testes Rust**, sendo 577 anteriores mais 9 novos:
  - 6 em `cad-craft/tests/pdf_export.rs`;
  - 1 unitário em `font.rs`;
  - 2 egui em `cad-gui/tests/pdf_ui.rs`;
- Clippy com `-D warnings`;
- `git diff --check`;
- **15 testes PowerShell**.

Nenhum teste anterior foi alterado ou removido. Os testes de folha que já existiam continuam passando com os textos de referência vindos do Core (critério 9).

Critérios cobertos por teste:

| Critério | Teste |
|---|---|
| 1. 3 páginas A3; a lista de exibição tem textos especiais, designações, número do fio, `/página.coluna`, espelho e logotipo | `display_list_reproduces_the_sheet`, `pdf_has_three_real_size_pages_fonts_images_and_bookmarks` |
| 2. PDF lido de volta com `lopdf`; Helvetica e imagem declaradas | `pdf_has_three_real_size_pages_fonts_images_and_bookmarks` |
| 3. Links: bobina ↔ contato e interrupções entre si; destino fora do alcance relatado | `links_follow_cross_references_and_are_omitted_outside_scope` |
| 4. Marcadores "Esquema elétrico" → "1 — Força", "2 — Comando", "3 — Sinalização"; título e data nos metadados | idem ao critério 2 |
| 5. ç/ã/é em WinAnsi; Ω substituído e relatado | `accents_are_kept_unsupported_characters_reported_and_monochrome_has_no_rgb`, `font::tests` |
| 6. Preto e branco sem operadores RGB/CMYK | idem ao critério 5 |
| 7. Diálogo com cliques reais: página ativa, depois projeto, relatório e "abrir"; Imprimir em A3 entrega o PDF temporário ao sistema (gravador) | `export_dialog_writes_active_page_then_project_and_reports`, `print_dialog_builds_temporary_pdf_and_hands_it_to_the_system` |
| 8. 200 páginas (moldura, 30 símbolos e 30 fios por página) | `two_hundred_pages_export_within_the_time_limit` |
| Papel, alcance vazio, cancelamento | `paper_fit_places_the_sheet_on_the_chosen_paper` |

Desempenho medido em 2026-11-04 (200 páginas, PDF de 0,6 MiB):

| Build | Tempo | Limite do teste |
|---|---|---|
| Release | **1,07 s** | meta de 10 s |
| Debug | 9,4 s | 120 s |

O teste usa 30 fios por página, não 60. A montagem das 200 páginas copia a primeira página no JSON com IDs novos e valida o projeto uma vez ao carregar. Inserir 199 macros de página levaria mais de 200 s em Debug.

Build Release: `cargo build --release --locked -p cad-gui` concluído; `target/release/cad-gui.exe` tem 16.448.512 bytes. O app não foi executado: não há teste no executável nesta etapa, e o próximo é ao fim da Etapa 22, com autorização do usuário.

## Limitações

Ver `KNOWN_ISSUES.md` (PDF-001 a PDF-007). Os principais:
- a fonte não é embutida;
- a tipografia da tela (fonte do egui) difere da do PDF (Helvetica);
- a impressão depende do leitor de PDF padrão do Windows.

Validação automática não é homologação manual, que fica para a Etapa 28.

## Git

`main`, HEAD `98ef54ab48cb5e5f6ad24ca0cd170709319e2e1c` (Etapas 18–19), observado em 2026-11-04. As Etapas 20 e 21 estão fora do HEAD; o commit + push das duas fica para depois do OK expresso do usuário. O `docs.rar` é do usuário e não entra.
