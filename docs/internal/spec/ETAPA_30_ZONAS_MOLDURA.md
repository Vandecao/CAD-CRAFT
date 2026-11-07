# CAD-CRAFT — Etapa 30: colunas e linhas da moldura

**Estado: APROVADO pelo autor em 2026-11-07** (plano `docs/internal/plans/PLANO_ETAPAS_29_31.md`, decisão 3 recomendada).

- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE, `docs/FORMATS.md`; código em `cadcraft-model/src/domain/document/frame/`, `cadcraft-render/src/display/frame.rs`, `cadcraft-gui/src/frame_editor/`.
- **Referência de comportamento:** QElectroTech (vídeo do autor de 2026-11-07): ao mudar a quantidade de colunas, as divisões e os rótulos são redesenhados; colunas numeradas e linhas com letras, divisões nos dois lados.

## Problemas (imagem 1 do autor)

1. O editor mantém o "tamanho uniforme" ao mudar a quantidade. 4 colunas × 40 mm deixam 40 mm sem coluna; 8 linhas × 40 mm = 320 mm não cabem na área útil de 252 mm e o núcleo recusa com "zonas da moldura inválidas ou fora da área útil", sem dizer o porquê.
2. Nas molduras de fábrica, as divisões entre colunas são linhas fixas (`FrameElement::Line`) criadas em `factory.rs`; só os rótulos vêm das zonas (`frame_zone_labels`). Removida uma coluna, a divisão fica e o rótulo some. Linhas (fileiras) não têm divisões desenhadas.

## Requisitos

1. **Editor:**
   - mudar a quantidade recalcula o tamanho uniforme para preencher a área útil (`área útil / quantidade`, arredondado a 0,1 mm, com a última zona absorvendo o resto);
   - botão "Distribuir na área útil" para refazer a divisão depois de ajustes manuais ou de tamanhos individuais;
   - ao lado de cada eixo, a soma: "8 × 31,5 mm = 252,0 mm de 252,0 mm", em laranja quando não fecha.
2. **Mensagem de erro clara** no núcleo e no editor, por eixo: "As 8 linhas somam 320,0 mm, mas a área útil tem 252,0 mm." No lugar da mensagem genérica.
3. **Divisões geradas pelas zonas** na lista de exibição (`cadcraft-render`), como os rótulos já são:
   - marcas de divisão de cada coluna e de cada linha, nos lados com rótulo habilitado (em cima/embaixo, esquerda/direita);
   - a faixa de rótulos delimitada (linha entre a faixa e a área útil);
   - mesma espessura e camada das linhas fixas de hoje; aparecem na tela, no PDF e no DXF.
4. **Formato `.cadcraft` 10 e `.cadframe` correspondente**, com migração da versão anterior e fixture nova (regra do `AGENTS.md`):
   - campo novo em `ZoneLayout` indicando que as divisões são geradas;
   - migração: em cada moldura, as linhas fixas que coincidem exatamente (tolerância de `tolerance.rs`) com divisões de zona são removidas e a moldura passa a gerar as divisões. Linhas que não coincidem ficam como estão.
   - molduras de fábrica (`factory.rs`) deixam de criar as linhas de divisão.
5. **Saídas de referência:** abrir as fixtures v0–v9 deve produzir o mesmo desenho. Se algum byte mudar (ordem de primitivas, por exemplo), explicar no relatório e regravar com `CAD_CRAFT_WRITE_BASELINE=1`, conferindo visualmente que o desenho é igual.
6. Undo/Redo do editor continuam exatos.

## Testes

- Editor com eventos reais: mudar a quantidade de colunas de 5 para 4 recalcula para 50 mm e a moldura mostra 4 divisões e 4 rótulos; 8 linhas numa área de 252 mm aplicam sem erro; tamanhos individuais que não fecham mostram a soma em laranja e a mensagem clara ao aplicar.
- Núcleo: mensagem por eixo; validação aceita a soma exata e recusa acima da área útil.
- Render: divisões geradas para colunas e linhas, nos dois lados quando habilitados; nenhuma divisão onde não há rótulo.
- Migração: `.cadcraft` 9 e `.cadframe` antigo com linhas fixas → v10 sem as linhas coincidentes e com divisões geradas; linhas que não coincidem preservadas; fixture v10 nova.
- Exportação PDF/DXF de uma moldura com 4 colunas e 8 linhas: divisões presentes.

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; Release compilado.
- Teste no app fica para o fim da Etapa 31.
- Relatório `docs/internal/reports/ETAPA_30_ZONAS_MOLDURA.md`; `docs/FORMATS.md` (versão 10), DECISIONS, HANDOFF, CURRENT_STATE e ROADMAP. Commit + push só com o OK do autor.
