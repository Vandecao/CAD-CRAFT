# ADR-0016 — editor de molduras, imagens e arquivo `.cadframe`

Estado: implementado e testado automaticamente na Etapa 14.
Homologação manual geral na Etapa 28.

## Contexto

A Etapa 13 trouxe o modelo de moldura sem editor. O Plano Mestre (Fase A)
pede criar e editar molduras como no EPLAN, com logotipos, marca d'água e
intercâmbio de molduras entre projetos. A aprovação autorizou a dependência
`image`, limites de 10 MB e 4096×4096 px e adiou QR e hiperlink.

## Decisão

- **Imagens são entidades do projeto** (`Project.assets`, `AssetId` v7,
  `ImageAsset`). O Core não decodifica: lê o cabeçalho PNG/JPEG para formato,
  dimensões e limites. A GUI decodifica com `image` (PNG/JPEG) antes de
  aceitar o arquivo e para criar a textura.
- **Bytes fora do JSON**: `project.json` guarda só os metadados; o
  `.cadcraft` grava `assets/<id>.<ext>` sem compressão e a leitura confere os
  bytes com os metadados. Gravar com imagem sem conteúdo é recusado.
- **Moldura editada é substituída inteira**: `FrameDraft` →
  `FrameTemplate::edited` (mesmo ID, versão + 1) via `UpsertFrame`.
  Inserir imagem e moldura juntos usa `FrameEdit::Batch`, atômico e
  reversível como um passo do histórico.
- **Integridade**: elemento `Image` exige imagem existente; imagem em uso não
  é removida; imagem igual (mesmos bytes) é reaproveitada.
- **`.cadframe`**: ZIP próprio (`application/x-cad-craft-frame+zip`, formato
  1) com `frame.json` e as imagens, lido pelo mesmo leitor defensivo do
  `.cadcraft`. Importar gera edições comuns (`frame_import_edits`) e troca
  IDs que colidem com conteúdo diferente.
- **`.cadcraft` formato 3 / `min_reader` 3**, porque `project.json` ganhou
  `assets`, `Image` e `watermark` (regra do ADR-0014). Migração 2→3 identidade.
- **Editor na GUI** como modo que ocupa a área de documentos, com câmera
  própria e painel de propriedades; não é um documento do projeto.

## Consequências

- Versões da Etapa 13 recusam arquivos da 14.
- Snapshot técnico JSON com imagens perde o conteúdo; a GUI desenha um
  marcador e o projeto só volta a ser salvo como `.cadcraft` após reinserir.
- A versão da moldura cresce a cada edição confirmada.
- QR code e hiperlink continuam futuros. O intercâmbio DXF/`.titleblock`/
  `.fn1` foi implementado posteriormente na Etapa 15 (ADR-0017).
