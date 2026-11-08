# Relatório — Etapa 30 (colunas e linhas da moldura)

Data: 2026-11-07. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_30_ZONAS_MOLDURA.md` (aprovado). Decisão: ADR-0033.

## Problemas corrigidos (imagem 1 do autor)

1. Remover uma coluna deixava um espaço sem número: as divisões das molduras de fábrica eram linhas fixas; só os rótulos vinham das zonas.
2. O editor mantinha o tamanho uniforme ao mudar a quantidade: 4 × 40 mm deixavam 40 mm sobrando; 8 linhas × 40 mm = 320 mm eram recusadas com "zonas da moldura inválidas ou fora da área útil".

## Entregue

- **Modelo** (`cadcraft-model/src/domain/document/frame/marks.rs`):
  - `ZoneLayout::generated_marks`;
  - `FrameTemplate::zone_marks`: marca em cada fim de zona (exceto na borda da área útil), atravessando a faixa de rótulos (até 5 mm) de cada lado com rótulo, para colunas e linhas, espessura 0,25 mm;
  - `FrameTemplate::adopt_generated_zone_marks`: troca as linhas fixas que coincidem com as divisões pelas geradas; sem coincidência, não muda nada;
  - erros por eixo: "as colunas/linhas da moldura somam mais que a largura/altura da área útil";
  - molduras de fábrica e em branco sem linhas de divisão fixas, com divisões geradas.
- **Desenho:** a lista de exibição (PDF, DXF) e a tela (`frame_view.rs`) desenham as divisões geradas.
- **Formato:** `.cadcraft` 10 (`min_reader` 10) e `.cadframe` 2. A migração 9 → 10 e a leitura de `.cadframe` 1 convertem as linhas fixas. Fixture nova `etapa30-v10.cadcraft`.
- **Editor de moldura:**
  - mudar a quantidade redistribui a área útil (0,1 mm; a última zona fica com o resto, como tamanhos individuais);
  - botão "Distribuir na área útil";
  - soma de cada eixo ("8 × 31,5 mm = 252,0 mm de 252,0 mm"), em amarelo quando não cabe;
  - ao aplicar zonas que não cabem: "As 8 linhas somam 320,0 mm, mas a área útil tem 252,0 mm de altura. Use "Distribuir na área útil" ou reduza a quantidade.";
  - molduras editadas passam a gerar as divisões.
- Importadores (DXF, titleblock, FN1, QET) mantêm o desenho original (sem divisões geradas).

## Saídas de referência

Regravadas com `CAD_CRAFT_WRITE_BASELINE=1`:
- todos os `.cadcraft` regravados mudaram porque agora são formato 10;
- nas fixtures v2–v5 (Etapas 13–17), que usam molduras com divisões fixas, mudaram também JSON, lista de exibição, PDF e DXF. O teste `converted_fixtures_draw_the_same_items_as_before` prova que cada página desenha exatamente os mesmos itens antes e depois; só a ordem mudou (as divisões passam a vir depois dos elementos da moldura);
- nova linha para a fixture v10.

## Testes

- `crates/cad-craft/tests/frame_marks.rs` (7): mesmo desenho que as linhas fixas (A3 e A4); conversão e preservação de linhas que não coincidem; arquivo antigo abre com divisões geradas e regrava em v10; divisões de colunas e linhas nos dois lados e divisão de fechamento; erros por eixo; fixtures v2–v5 com os mesmos itens; fixture v10.
- `crates/cadcraft-gui/tests/frame_zones_ui.rs` (4), com eventos reais: 5 → 4 colunas redistribui para 50 mm e redesenha 3 divisões; 8 linhas = 31,5 mm aplicam; 3 colunas com o resto na última; zonas que não cabem são recusadas com a mensagem com os números no painel Mensagens.
- `persistence.rs`: o teste de "formato futuro" passou a usar `FORMAT + 1` em vez do número fixo 10.

## Validação

`scripts/validate.ps1` em 2026-11-07: exit 0, **762 Rust + 6 exemplos + 16 PowerShell** (antes 751 + 6 + 16; 11 testes novos), zero falhas; clippy sem avisos. Release: 23.917.568 bytes.

## Pendente

Teste no app ao fim da Etapa 31 (editar a moldura A4 do autor: remover coluna, 8 linhas, rótulos à esquerda e à direita).

## Correção após o teste do autor (2026-11-07)

O autor viu que o editor de moldura não mostrava mais as divisões de colunas e linhas. Desde esta etapa as divisões são geradas pelas zonas, e só a vista da página as desenhava; o editor e a prévia de importação desenhavam só os rótulos. Agora as três vistas usam `frame_view::paint_zones` (divisões e rótulos). Teste: `frame_zones_ui::the_frame_editor_draws_the_generated_divisions`.

Na mesma revisão, o autor pediu que a identificação das linhas fique dentro da moldura. A faixa dos rótulos agora é medida até a borda da moldura (o menor retângulo em volta da área útil; sem ele, a borda da folha), e não até a borda da folha. Quando a área útil já encosta na borda, como nas laterais das molduras de fábrica, a faixa de 5 mm fica do lado de dentro, com uma linha separadora gerada; rótulos, divisões e separadora usam `FrameTemplate::zone_strip`. A faixa das colunas no alto das molduras de fábrica não muda. Saídas de referência regravadas só para `etapa30-v10.cadcraft` (tela, PDF e DXF), a única com rótulos de linha. Teste: `frame_marks::row_labels_stay_inside_the_frame_border`.
