# CAD-CRAFT — Bloco R4: importadores e exportadores

**Estado: APROVADO pelo autor em 2026-11-05 ("Aprovo todas"), com todas as decisões recomendadas.**

- **Plano aprovado:** `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md`, passo 7. Vem depois do R3.
- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE e `docs/FORMATS.md` (R2).

## Objetivo

Cada formato externo segue o mesmo caminho, fácil de estudar e de testar: **ler** (arquivo → estrutura crua do formato) → **converter** (estrutura crua → `Project` neutro) → **relatar** (`ImportReport`). **Sem mudança de comportamento.**

## Entregas

1. `cadcraft-import`: uma pasta por formato, começando pelo `qet` (hoje `qet.rs` com 1372 linhas):
   ```
   qet/ mod.rs (API pública igual), read.rs (XML → tipos crus), convert.rs (→ Project), report.rs
   ```
   Depois, na mesma forma: `elmt`, `titleblock`, `fn1`, `edz`, `dxf`, CSV de peças. Formatos pequenos podem ficar num arquivo só se já seguirem as três fases.
2. DXF: separar o que é comum (leitura de entidades, unidades) do importar (`cadcraft-import`) e do exportar (`cadcraft-export/dxf_export.rs`), sem duplicar código; o comum fica onde as duas crates possam usar sem quebrar a direção das dependências (se precisar, um módulo pequeno em `cadcraft-model` ou uma crate `cadcraft-dxf`; registrar a escolha num ADR).
3. `cadcraft-render/src/display.rs` (955 linhas) e `cadcraft-export/src/pdf.rs` divididos por assunto (página, moldura, símbolos, conexões, texto).
4. Garantia contra pânico em entrada externa: um teste por importador com arquivos sintéticos truncados ou corrompidos que deve retornar erro, nunca entrar em pânico.
5. Os testes com amostras reais continuam opcionais por `CADCRAFT_SAMPLES` (ver `docs/TESTES_COM_AMOSTRAS.md`); se o autor tiver a pasta, rodar e registrar no relatório que os resultados (convertidos, recusados, perdas) não mudaram.

## Regras

- `git mv` primeiro, depois dividir, para manter o histórico.
- APIs públicas usadas pela GUI e pela fachada sem mudança (reexportar se preciso).
- Nenhum importador pode panicar em entrada externa (`AGENTS.md`).

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; saídas de referência idênticas (PDF e DXF inclusive).
- Release compilado. Relatório `docs/internal/reports/R4_IMPORTADORES_EXPORTADORES.md`; memória atualizada.
- Sem teste no app. Commit + push só com o OK.

## Decisões pedidas ao autor

1. Código comum de DXF: módulo na crate que já existe (recomendado se couber sem quebrar camadas) ou crate nova `cadcraft-dxf`.
2. Rodar as amostras reais ao fim (recomendado, se a pasta estiver disponível).
