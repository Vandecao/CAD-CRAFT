# CAD-CRAFT — Bloco R3: modelo organizado por assunto

**Estado: APROVADO pelo autor em 2026-11-05 ("Aprovo todas"), com todas as decisões recomendadas.**

- **Plano aprovado:** `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md`, passo 6. Vem depois do R2.
- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE e o relatório do R2.

## Objetivo

Hoje `crates/cadcraft-model/src/domain/` tem cerca de 40 arquivos soltos, e `domain/project/` concentra os `impl Project`. Agrupar por assunto para que cada parte do modelo seja encontrada pelo nome. **Só movimentação de código**: nada muda no formato, no PDF, no DXF ou na interface.

## Organização proposta

```
cadcraft-model/src/domain/
├── electrical/   component, device, function, representation, connection, connection_point,
│                 connection_drawing, network, designation, identification, marker, numbering
├── physical/     part, terminal, terminal_strip, terminal_bridge, cable, cable_core, conductor,
│                 connector, physical_elements, physical_values
├── document/     document, document_structure, page, page_graphic, frame, asset
├── symbols/      symbol, symbol_library, macro_def
├── checks/       check, report
├── geometry/     geometry, transform
├── ids.rs, properties.rs, project.rs
└── project/      impl Project agrupados pelo mesmo assunto (electrical.rs, physical.rs, document.rs…)
```

Ajustar a lista ao que o código mostrar; registrar no relatório cada arquivo e o destino.

## Regras

- `git mv` para preservar o histórico.
- Os caminhos públicos de hoje (`cad_craft::domain::Device` etc.) continuam valendo: reexportar em `domain/mod.rs`. A GUI e os testes não devem precisar mudar; se mudarem, só `use`.
- Arquivos acima de 800 linhas no modelo (`project/macros.rs` 1024, `frame.rs` 871, `symbol.rs` 820) divididos por subassunto.
- **`tolerance.rs`**: criar constantes e funções de comparação de `f64` (`EPS_MM`, `nearly_equal`, `is_zero`). Revisar as comparações exatas de `f64` (cerca de 69 levantadas no R0) **só onde houver teste cobrindo**; as demais ficam listadas no relatório para depois. Nenhuma saída de referência pode mudar.
- Serviços sem estado que hoje são métodos podem virar funções livres só se isso não mudar a API usada pela GUI.

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; `crate_layers.rs` e `baseline_outputs.rs` passando sem regravar hashes; fixtures v0–v9 abrem.
- Release compilado. Relatório `docs/internal/reports/R3_MODELO_POR_ASSUNTO.md` com a tabela de movimentos; memória atualizada.
- Sem teste no app (sem mudança de comportamento; o teste do app fica para o R5). Commit + push só com o OK.

## Decisões pedidas ao autor

1. Pastas em inglês como acima (recomendado) ou em português.
2. Revisão de `f64` só onde há teste (recomendado) ou nenhuma neste bloco.
