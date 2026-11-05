# CAD-CRAFT — Bloco R1: divisão do Core em crates

**Estado: APROVADO pelo autor em 2026-11-05 ("Aprovo todas"), com as três decisões recomendadas:** testes de integração na fachada; E/S de arquivos da GUI para `cadcraft-format` só no R5; teste rápido no app ao fim, com autorização.

- **Plano aprovado:** `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md`, passo 3. Vem depois da publicação e antes da Etapa 27.
- **Base:**
  - privado `CAD-CRAFT1`, `main` em `3be39ab`;
  - público `CAD-CRAFT` em `ea24a73`;
  - validação: 673 Rust + 16 PowerShell;
  - saídas de referência gravadas no R0.

## Objetivo

Separar o Core (`crates/cad-craft`, cerca de 28 mil linhas) em cinco crates com dependência num só sentido, para que cada parte possa ser estudada sozinha e o compilador impeça dependências erradas. **Só movimentação de código.** Nada muda no programa:
- formato `.cadcraft`;
- PDF e DXF;
- interface e comportamento.

## Levantamento (feito em 2026-11-05)

As camadas já têm um só sentido: `domain` não usa display, PDF, persistência nem importadores. Só dois pontos atravessam as fronteiras novas:
1. `display.rs` declara `impl Project { fn page_display_list }`. Pela regra do Rust, métodos de `Project` só podem ficar na crate do `Project`, então vira um *trait* de extensão (`PageDisplayList`) com o mesmo nome de método.
2. `Project::report_rows_per_page` (`pub(crate)`, em `domain/project/reports.rs`) é usado por `display.rs` e passa a `pub` (com `#[doc(hidden)]`).

## Crates

```
crates/
├── cadcraft-model/   domain/, error.rs, graph.rs, document_index.rs, physical_index.rs
│                     deps: serde, uuid
├── cadcraft-format/  persistence.rs (.cadcraft, .cadsym, .cadmacro, .cadpart, .cadframe e migrações)
│                     deps: model, serde_json, zip, flate2
├── cadcraft-render/  display.rs, font.rs (lista de exibição, métricas)
│                     deps: model
├── cadcraft-import/  interchange/ (qet, elmt, titleblock, fn1, edz, dxf, csv de peças, xml)
│                     deps: model, quick-xml, csv, sevenz-rust2, image
├── cadcraft-export/  pdf.rs, dxf_export.rs, report_export.rs
│                     deps: model, render, pdf-writer, rust_xlsxwriter, csv, image
├── cad-craft/        FACHADA: reexporta tudo com os caminhos de hoje
│                     (cad_craft::domain, ::persistence, ::interchange, ::display, ::pdf…)
└── cadcraft-gui/     (hoje cad-gui) inalterado por dentro
```

**Grafo:** `gui → cad-craft (fachada) → {format, import, export, render} → model`; e `export → render`.

**Fachada:**
- `crates/cad-craft` continua existindo e só reexporta. Os mais de 60 arquivos de teste e a GUI, que usam `cad_craft::…`, compilam sem mudança de caminho.
- O *trait* `PageDisplayList` é reexportado junto de `cad_craft::domain` e `cad_craft::display`, para que `project.page_display_list(page)` continue funcionando onde já funciona. Onde não der, entra um `use` explícito.

**GUI** (decisão 4 do plano, aprovada):
- o pacote passa a `cadcraft-gui`;
- `[lib] name = "cad_gui"` mantém os caminhos dos testes;
- `[[bin]] name = "cad-gui"` mantém `cad-gui.exe`;
- `-p cad-gui` vira `-p cadcraft-gui` em README, CONTRIBUTING, CI, scripts e documentos.

## Regras de execução

1. **Movimentos com `git mv`** (preservam o histórico de cada arquivo), em passos pequenos: um crate por vez, começando por `cadcraft-model`, cada passo compilando.
2. **Só mudam:**
   - caminhos `use` (`crate::domain` → `cadcraft_model::domain` etc.);
   - visibilidade estritamente necessária;
   - o *trait* de extensão.
   - Nenhuma lógica é alterada.
3. **Testes:**
   - os testes de unidade (`#[cfg(test)]` dentro dos arquivos) vão junto com o arquivo;
   - os testes de integração ficam na fachada (decisão 1).
4. **Dependências:** cada crate declara só as que usa. Nenhuma dependência nova. `Cargo.lock` sem versões novas.
5. **SPDX e licença:** cabeçalho em todos os arquivos novos (o teste do R0 confere) e `license.workspace = true` em cada `Cargo.toml`.

## Critérios de aceitação

1. `scripts/validate.ps1` passa com **673 Rust + 16 PowerShell ou mais**, sem teste removido.
2. **Saídas de referência idênticas:** `baseline_outputs` sem regravar o arquivo de hashes.
3. Fixtures `.cadcraft` v0–v9 abrem como antes (testes de persistência).
4. **Cada crate compila sozinha:** `cargo check -p cadcraft-model` (e as demais).
5. **Sentido das dependências conferido:**
   - `cadcraft-model` não depende de nenhuma outra crate do projeto;
   - `render` não depende de `export`, `format` nem `import`;
   - novo teste ou passo do `validate.ps1` lê `cargo metadata`.
6. Release compilado; `target/release/cad-gui.exe` existe com o mesmo nome.
7. Teste no app: o R1 toca a montagem do executável, então vale um teste rápido no app (abrir, importar um `.qet`, exportar PDF), com autorização para usar o computador.
8. Relatório `docs/internal/reports/R1_DIVISAO_CRATES.md`, atualização do mapa no README e na memória. Commit + push só com o OK; depois, a sincronização com o repositório público, também com o OK.

## Decisões pedidas ao autor

1. **Testes de integração:**
   - **Recomendado:** ficam na fachada `cad-craft` neste bloco, sem mudança de caminho.
   - A distribuição pelas crates fica para o R3 e o R4, quando cada área for reorganizada por dentro.
   - Alternativa: distribuir já, com mais mudanças de `use` neste bloco.
2. **Leitura e gravação de `.cadsym`, `.cadmacro`, `.cadpart` e `.cadframe` que hoje estão na GUI** (`symbol_files.rs`, partes de `macro_library.rs` e `project_file.rs`):
   - **Recomendado:** levar para `cadcraft-format` no R5, junto da reorganização da GUI. Aqui seria refatoração de código da GUI, não só movimentação.
   - Alternativa: fazer já no R1, como sugere o parecer.
3. **Teste no app ao fim do R1 (critério 7):**
   - **Recomendado:** sim, um teste rápido, porque o executável é montado de outro jeito.
   - Alternativa: só a validação automática.

## Fora do escopo

- Dividir arquivos grandes (`qet.rs`, `display.rs`, `pdf.rs`): R4.
- Pastas por assunto dentro do modelo (`electrical/`, `physical/`…): R3.
- Remover os comentários "Etapa N": R6.
- Qualquer mudança de comportamento.

## Instruções para o agente

1. Ler `AGENTS.md`, `AGENTS.local.md`, este prompt, o `HANDOFF` e o plano R0–R6. Git só leitura até a aprovação.
2. Plano `docs/internal/plans/<data>-r1.md` com "Andamento"; página de memória `notes/r1-divisao-crates.md`.
3. Um crate por passo, validando a cada passo. Ao fim, validação completa, Release, relatório. Parar e pedir o OK para versionar.
