# CAD-CRAFT — Bloco R6: limpeza final

**Estado: APROVADO pelo autor em 2026-11-05 ("Aprovo todas"), com todas as decisões recomendadas.**

- **Plano aprovado:** `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md`, passo 9. Último bloco R.
- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE.

## Objetivo

Fechar a reorganização: código sem marcas do processo de desenvolvimento, idioma consistente e lints mais rígidos. **Sem mudança de comportamento.**

## Entregas

1. **Comentários "Etapa N"** (hoje cerca de 230 em `.rs`): reescrever em inglês dizendo *o que* o código faz e *por quê*, sem citar etapas. Nomes de testes e mensagens de `assert` que citam etapas também. A história fica no Git e em `docs/internal/`.
2. **Idioma** (regra do `AGENTS.md`): comentários `//`, `///` e `//!` em inglês. Textos da interface continuam em português.
3. **Textos da interface centralizados**: um módulo `cadcraft-gui/src/strings.rs` (ou `ui_text/`) com as frases repetidas (títulos de diálogos, botões comuns, mensagens de erro), preparando uma futura tradução. Não traduzir nada agora; não mexer em rótulos usados pelos testes de GUI sem atualizar o teste para usar a mesma constante.
4. **Lints** no `Cargo.toml` do workspace (`[workspace.lints]`), começando por:
   - `clippy::unwrap_used` fora de testes (o `AGENTS.md` já pede `expect("motivo")`);
   - `clippy::dbg_macro`, `clippy::todo`, `clippy::print_stdout` fora de binários e exemplos;
   - `missing_docs` só em itens públicos de `cadcraft-model` e `cadcraft-format`.
   Corrigir até zerar; onde a correção mudaria comportamento, usar `#[allow]` com justificativa e listar no relatório.
5. Remover reexportações temporárias que os blocos R1–R5 deixaram e que ninguém usa.
6. `cargo fmt` e um passe final de `cargo doc --no-deps` sem avisos.

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; saídas de referência idênticas; Release compilado.
- Teste rápido no app (texto da interface centralizado): com autorização do autor.
- Relatório `docs/internal/reports/R6_LIMPEZA.md`; plano R0–R6 marcado como concluído; memória atualizada. Commit + push só com o OK; depois, sincronização com o público.

## Decisões pedidas ao autor

1. Remover todas as citações de etapa do código (recomendado) ou manter nos testes.
2. Centralizar só os textos repetidos (recomendado) ou todos os textos da interface.
3. Conjunto de lints acima (recomendado) ou um conjunto menor.
