# CAD-CRAFT — Etapa 35: bornes, coordenadas digitadas e conexão unifilar

**Estado: APROVADO pelo autor em 2026-11-07** (plano `docs/internal/plans/PLANO_ETAPAS_32_35.md`, itens 9, 11 e 12; decisão 3).

- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE; código de réguas e bornes (`cadcraft-model/src/domain/.../terminal*`, editor de réguas na GUI), `panels/insertion.rs`, `commands/edit.rs` (grupo "Opções"), ferramentas de conexão.

## Problemas

- O símbolo "Borne" cria dispositivos X1, X2…; a lista de bornes, que usa bornes de régua, sai vazia.
- "Inserção de coordenadas" existe só como item planejado.
- Não há conexão unifilar: ligar representações no unifilar criaria conexões elétricas no multifilar; no teste foram usadas linhas gráficas.

## Requisitos

1. **Borne de régua pelo símbolo:** ao inserir "Borne", o centro de inserção pede a régua (existente ou nova, ex.: X1) e numera o próximo borne (X1:1, X1:2…). A lista de bornes, as pontes e o editor de réguas passam a vê-los.
2. **Inserção de coordenadas:** campos X/Y (absolutos e relativos ao último ponto) para posicionar símbolos, cantos de conexão e pontos de linhas gráficas; Enter confirma; funciona junto com o mouse.
3. **Conexão unifilar** (decisão 3): ferramenta própria para páginas unifilares que liga representações sem criar conexões no multifilar, com marca do número de fases (traços oblíquos); aparece no PDF e no DXF; não entra na lista de conexões nem na numeração de fios.

## Testes

- Três bornes inseridos pelo símbolo formam X1:1–X1:3 e aparecem na lista de bornes.
- Coordenadas digitadas posicionam símbolo e rota exatamente nos valores; Ctrl+Z desfaz.
- Conexão unifilar entre Q1 e K1 no unifilar não altera redes, lista de conexões nem numeração; aparece no PDF/DXF.
- Formato com migração e fixture se a conexão unifilar for persistida em estrutura nova.

## Entrega

- `validate.ps1`, Release, relatório `docs/internal/reports/ETAPA_35_BORNES_COORDENADAS_UNIFILAR.md`, ADR e `docs/FORMATS.md` se o formato mudar, HANDOFF, CURRENT_STATE, ROADMAP. Commit + push só com o OK do autor.
- Teste no app ao fim da 35: refazer a partida estrela-triângulo usando as macros da biblioteca (junções em T, tags, nomes de pontos, lista de dispositivos sem duplicatas, lista de bornes).
