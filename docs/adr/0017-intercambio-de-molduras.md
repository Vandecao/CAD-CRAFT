# ADR-0017 — intercâmbio de molduras

Estado: implementado na Etapa 15; validação automática registrada no relatório.
Homologação manual geral permanece na Etapa 28.

## Contexto

O editor da Etapa 14 já trabalha com `FrameTemplate` e `FramePackage`. O
usuário aprovou importar molduras EPLAN `.fn1`, carimbos QElectroTech
`.titleblock` e DXF ASCII, além de exportar DXF. Autorizou `quick-xml` e
fixtures `.fn1`; decidiu por DXF próprio e excluiu exportação `.fn1`.

## Decisão

- `cad-craft::interchange` converte os três formatos para `FramePackage`
  acompanhado de `ImportReport`. Itens ignorados e aproximações são descritos.
- XML é lido por `quick-xml` sem DTD/entidades externas. Entradas têm limite
  de 64 MiB; imagens mantêm os limites da Etapa 14.
- `.fn1` usa o tipo de `A19`, pois os nomes `O<n>` variam. Números de
  propriedades têm mapeamento explícito para `SpecialField`; desconhecidos
  preservam a chave EPLAN. Quando a borda mede a folha, a origem interna do
  EPLAN é deslocada para `(0, 0)`; rótulos acompanham o deslocamento.
- `.titleblock` converte grade, texto, variáveis e imagens PNG/JPEG embutidas
  para elementos de moldura. A folha de destino vem da página escolhida ou
  do A3 paisagem.
- DXF ASCII R12–2018 lê as entidades previstas na especificação. O escritor
  gera R12 em milímetros; área útil e zonas são metadados `999` próprios do
  CAD-CRAFT para ida e volta sem perda. Outros programas podem ignorá-los.
- A GUI mostra prévia, página opcional e relatório antes de confirmar.
  Importação e atribuição à página usam um único `FrameEdit::Batch`, reversível.

## Consequências

- A exportação DXF de uma moldura com imagem omite a imagem e avisa; textos
  traduzidos exportam somente o padrão. O formato `.cadcraft` permanece 3.
- Coordenadas EPLAN só são normalizadas automaticamente quando a borda
  fornece uma âncora confiável. Arquivos sem ela preservam as coordenadas e
  podem exigir ajuste no editor.
- Não há exportação `.fn1`/`.titleblock`, DXF binário, spline ou SVG.
