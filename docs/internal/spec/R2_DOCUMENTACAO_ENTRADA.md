# CAD-CRAFT — Bloco R2: documentação de entrada

**Estado: APROVADO pelo autor em 2026-11-05 ("Aprovo todas"), com todas as decisões recomendadas.**

- **Plano aprovado:** `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md`, passo 5.
- **Base:** privado `CAD-CRAFT1` em `06a6cbf` (Etapa 27); público `CAD-CRAFT` em `79b1a48`; validação 689 Rust + 16 PowerShell.
- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, `docs/internal/memory/HANDOFF.md` e `CURRENT_STATE.md`.

## Objetivo

Uma pessoa nova deve entender o projeto em uma tarde. **Nenhuma mudança de comportamento**: só documentos, comentários `//!` e exemplos.

## Entregas

1. `docs/ARCHITECTURE.md` (inglês): as 8 crates, a direção das dependências, o fluxo editar → delta → undo, a lista de exibição, onde fica a rede (só `cadcraft-gui/src/assistant/`), com um diagrama em texto.
2. `docs/TOUR.md` (inglês): "por onde começar a ler", na ordem `cadcraft-model` → `cadcraft-format` → `cadcraft-render` → `cadcraft-export` → `cadcraft-import` → `cadcraft-gui`, com 2 ou 3 arquivos-chave por crate.
3. `docs/GLOSSARY.md` (inglês, com o termo em português ao lado): componente, dispositivo, representação, função, conexão lógica, condutor, rede, borne, régua, ponte, cabo, veia, moldura, macro, designação, referência cruzada, potencial.
4. `docs/FORMATS.md` (inglês): `.cadcraft` (versões 0–9 e migrações), `.cadsym`, `.cadmacro`, `.cadpart`, `.cadframe`; e o que cada importador e exportador lê ou grava (QET, ELMT, titleblock, FN1, EDZ, DXF, CSV, PDF, XLSX), com as perdas conhecidas (`ImportReport`).
5. `examples/` na fachada `cad-craft` (`crates/cad-craft/examples/`), compilados no CI e executados por `cargo test --examples` ou por um teste:
   - `01_new_project` (criar projeto, documento e página, salvar `.cadcraft` num diretório temporário);
   - `02_place_symbol` (inserir símbolo e ler a lista de exibição);
   - `03_connect` (ligar dois pontos e ver a rede derivada);
   - `04_number` (numerar dispositivos e conexões, desfazer);
   - `05_export_pdf` (exportar PDF para um temporário);
   - `06_import_qet` (importar um `.qet` **sintético** dos testes e imprimir o `ImportReport`).
   Cada exemplo com o cabeçalho SPDX e menos de 100 linhas. Nenhum usa rede ou arquivo real de terceiros.
6. `//!` em todo módulo que ainda não tem (hoje cerca de 75 arquivos em `crates/*/src`): o que faz e por onde começar a ler, em inglês.
7. Teste novo que falha se algum `.rs` em `crates/*/src` não começar (depois do cabeçalho SPDX) com `//!`, no mesmo estilo de `license_headers.rs`.
8. `README.md` e `README.pt-BR.md`: links para os quatro documentos.

## Fora do escopo

Mover código, renomear itens, mudar comentários que não sejam `//!` (isso é R6).

## Validação e entrega

- `pwsh -NoProfile -File scripts/validate.ps1` com contagem de testes igual ou maior; saídas de referência idênticas.
- Relatório `docs/internal/reports/R2_DOCUMENTACAO_ENTRADA.md`; HANDOFF, CURRENT_STATE e ROADMAP atualizados.
- Sem teste no app (não toca a GUI). Commit + push só com o OK do autor.

## Decisões pedidas ao autor

1. Documentos de entrada em inglês, com o glossário bilíngue (recomendado), ou em português.
2. Exemplos na fachada `cad-craft` (recomendado) ou numa crate `examples` separada.
