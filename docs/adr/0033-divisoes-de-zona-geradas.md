# ADR-0033 — Divisões de zona geradas pelas zonas (formato 10)

Data: 2026-11-07. Estado: aceito na Etapa 30, conforme o plano `docs/internal/plans/PLANO_ETAPAS_29_31.md` aprovado pelo autor.

## Contexto

As molduras dividem a área útil em colunas e linhas (zonas). Os rótulos das zonas sempre foram gerados a partir delas, mas as divisões entre colunas das molduras de fábrica eram linhas fixas da moldura. Ao remover uma coluna no editor, o número sumia e a divisão ficava, com um espaço sem número; as linhas (fileiras) não tinham divisões. O editor também mantinha o tamanho uniforme ao mudar a quantidade, e o núcleo recusava a soma maior que a área útil com uma mensagem genérica. O QElectroTech, usado como referência pelo autor, redesenha as divisões quando a quantidade de colunas muda.

## Decisão

- `ZoneLayout` ganha `generated_marks`. Ligado, a moldura desenha uma marca em cada fim de zona (exceto na borda da área útil), atravessando a faixa de rótulos de cada lado com rótulo habilitado, até 5 mm, com espessura 0,25 mm (`FrameTemplate::zone_marks`). Tela, PDF e DXF usam a mesma geometria.
- Molduras de fábrica, novas e editadas no editor geram as divisões. Molduras importadas (DXF, titleblock, FN1, QET) mantêm o desenho original.
- **Formato `.cadcraft` 10** (`min_reader` 10) e **`.cadframe` 2**. Ao ler versões anteriores, as linhas fixas que coincidem exatamente com divisões de zona são removidas e a moldura passa a gerar as divisões (`FrameTemplate::adopt_generated_zone_marks`); linhas que não coincidem ficam. Sem nenhuma coincidência, a moldura não muda.
- O núcleo diz qual eixo não cabe ("as colunas/linhas da moldura somam mais que a largura/altura da área útil"); o editor mostra a soma de cada eixo e, ao aplicar, a mensagem com os números. Mudar a quantidade redistribui a área útil (0,1 mm, a última zona fica com o resto); o botão "Distribuir na área útil" refaz a divisão.

## Consequências

- O desenho das molduras convertidas é o mesmo: o teste `converted_fixtures_draw_the_same_items_as_before` compara os itens desenhados das fixtures v2–v5 lidas sem e com a conversão. Mudou só a ordem dos itens (as divisões vêm depois dos elementos da moldura), o que altera os bytes de PDF/DXF/lista de exibição dessas fixtures; as saídas de referência foram regravadas com essa explicação.
- Arquivos gravados no formato 10 não abrem em versões anteriores do CAD-CRAFT (`min_reader` 10).
- Fixture nova: `etapa30-v10.cadcraft` (A4 com 4 colunas e 8 linhas, rótulos dos dois lados).
