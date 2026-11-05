# ADR-0015 — molduras, zonas e referências por coluna

Estado: implementado e testado automaticamente na Etapa 13.
Homologação manual geral na Etapa 28.

## Contexto

O Plano Mestre (Fase A) pede folhas com moldura e carimbo e referências
cruzadas por coluna/linha, como no EPLAN. A análise da moldura `FN1_00A1.fn1`
do usuário (XML PXF do EPLAN 2027) mostrou borda, linhas, textos fixos
multilíngues, textos especiais por número de propriedade (alguns indexados,
como o quadro de revisões) e rótulos de coluna posicionados pela moldura.

## Decisão

- **`FrameTemplate` no Core** (`domain/frame.rs`): folha, área útil,
  `ZoneLayout` (colunas e linhas com tamanhos individuais, rótulos numéricos
  ou alfabéticos, lados), elementos `Line/Rectangle/Circle/Arc/Text/Special`
  com espessura e `FrameTextStyle` (altura, 9 alinhamentos, rotação),
  `LocalizedText` (padrão pt-BR + idiomas) e `SpecialField` (projeto, página,
  documento, contagens, página anterior/seguinte, propriedades por chave com
  índice `chave.n`). Validação completa na criação e na leitura.
- **Zonas**: colunas da esquerda para a direita e linhas de cima para baixo,
  a partir da área útil; intervalos `[início, fim)`, a última inclui o fim.
  Rótulos de zona são gerados pela configuração (não são elementos).
- **Atribuição**: `Project.default_frame` e `Page.frame` (substituição por
  página). Página nova recebe a moldura do mesmo tamanho quando a padrão não
  coincide. Projeto novo na GUI recebe as molduras de fábrica (A3 paisagem,
  padrão, e A4 retrato), desenhos originais.
- **`FrameEdit`/`FrameDelta`**: moldura (inserir/substituir/remover sem uso),
  padrão, moldura da página, formato de referência, metadados de projeto e de
  página (valor vazio remove). Reversíveis no histórico comum
  (`Transaction::Frame`); edições pelo painel mantêm a seleção.
- **Referências**: `Project::reference_label` → `/página.coluna` (ou
  `/página.colunaLinha`); fora da divisão ou sem moldura, página e
  coordenadas. `DeviceReference.reference` leva o rótulo; navegador,
  Propriedades e marcadores usam-no. Na folha, ocorrências de dispositivos
  com mais de uma ocorrência mostram as referências (principal lista as
  demais; as outras apontam a principal).
- **Persistência**: `.cadcraft` formato 2 e `min_reader` 2, porque o
  `project.json` ganhou campos (regra do ADR-0014). Migração 1→2 identidade;
  arquivos v1 continuam associados ao caminho (só o v0 é tratado como
  snapshot). Campos novos omitidos quando vazios: snapshots antigos
  reserializam iguais.

## Consequências

- Uma versão da Etapa 12 recusa arquivos gravados pela 13 (`min_reader` 2).
- Referências desenhadas na folha são recalculadas a cada quadro a partir do
  agregado (custo linear nas representações; medir com projetos reais).
- O editor gráfico de molduras, imagens/logotipos e marca d'água ficam para a
  Etapa 14; DXF, `.titleblock` e `.fn1` para a Etapa 15.
- O mapeamento dos números de propriedade do EPLAN (`A1559`) para
  `SpecialField` será definido e verificado na Etapa 15.
