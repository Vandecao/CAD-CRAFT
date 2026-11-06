# CAD-CRAFT — Bloco R5: organização da GUI

**Estado: APROVADO pelo autor em 2026-11-05 ("Aprovo todas"), com todas as decisões recomendadas.**

- **Plano aprovado:** `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md`, passo 8. Vem depois do R4.
- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE.

## Objetivo

Deixar `cadcraft-gui` legível: arquivos menores, estado do app dividido e erros tipados. **Comportamento igual**, com uma correção pontual (abrir arquivos com `opener`).

## Entregas

1. Dividir os arquivos acima de 1.000 linhas, por assunto:
   - `physical_editor.rs` (2048), `commands.rs` (1939), `frame_editor.rs` (1562), `state.rs` (1296), `app.rs` (1286), `symbol_editor.rs` (1091), `edz_import_ui.rs` (1068), `panels/navigator.rs` (1051), `part_editor.rs` (1033).
   - Editores no padrão `editor/state.rs` (dados), `editor/input.rs` (mouse e teclado), `editor/draw.rs` (desenho).
   - `commands.rs`: catálogo de comandos separado por aba da faixa.
   - Meta: nenhum arquivo acima de 800 linhas.
2. `CadApp` com subestruturas por assunto (por exemplo `editors`, `dialogs`, `exports`, `imports`, `assistant`, `panels`) em vez de dezenas de campos soltos.
3. Menos `pub mod` em `lib.rs` (hoje 28): deixar público só o que os testes de GUI usam; o resto `pub(crate)`.
4. Erros tipados na GUI (`enum GuiError` com `thiserror`, se aprovado), no lugar de `String` nas operações de arquivo, importação e exportação; mensagens ao usuário iguais.
5. Leitura e gravação de `.cadsym`, `.cadmacro`, `.cadpart` e `.cadframe` que ainda estiverem na GUI passam para `cadcraft-format` (decisão adiada do R1).
6. **Correção:** `pdf_export.rs:69` usa `cmd /C start` para abrir o PDF. Trocar pela crate `opener` (funciona em Windows, Linux e macOS e não passa o caminho por um shell).

## Regras

- `git mv` antes de dividir.
- Os testes de GUI (eventos egui reais e nós AccessKit) não mudam, salvo `use`. Rótulos da interface iguais.
- Dependências novas (`opener`, `thiserror`) só com aprovação e licença compatível (`deny.toml`).

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; saídas de referência idênticas; Release compilado.
- **Teste no app** ao fim (toca a GUI): feito pelo Claude Code com a autorização do autor para usar o computador. Roteiro mínimo: abrir projeto, editar página, editores de símbolo, moldura, peça e físico, importar QET, exportar PDF (abre no visualizador) e DXF, assistente.
- Relatório `docs/internal/reports/R5_GUI.md`; memória atualizada. Commit + push só com o OK.

## Decisões pedidas ao autor

1. Aprovar as dependências `opener` (recomendado) e `thiserror` (recomendado).
2. Mover a E/S de `.cadsym`/`.cadmacro`/`.cadpart`/`.cadframe` para `cadcraft-format` neste bloco (recomendado).
