# Etapa 14 (Fase A) — editor de molduras, imagens e `.cadframe`

**Estado:** implementada e validada automaticamente; pronta para revisão.
Homologação manual geral na Etapa 28. Sem commit/push até o OK do usuário.

Prompt: `docs/internal/spec/ETAPA_14_EDITOR_DE_MOLDURAS.md` (aprovado em 2026-11-03,
com a dependência `image` autorizada, limites 10 MB e 4096×4096 px, QR e
hiperlink adiados). Plano: `docs/internal/plans/2026-11-03-etapa-14.md`. Decisão:
ADR-0016. Base: `main`, HEAD `98d57af21efbfda30bddabce23ca194aefb07e99`
(Etapa 13), 466 Rust + 15 PowerShell.

## Implementado

### Core (`cad-craft`)

- `domain/asset.rs` (novo): `AssetId` (UUID v7), `ImageAsset` (PNG/JPEG,
  largura/altura em px, tamanho e bytes), `inspect_image` lê só o cabeçalho
  (assinatura, IHDR do PNG, SOF do JPEG) e aplica os limites
  `MAX_IMAGE_BYTES = 10 MB` e `MAX_IMAGE_SIDE = 4096`. Sem decodificador no Core.
- `domain/frame.rs`: `FrameElement::Image { rect, asset }`,
  `FrameTextStyle.watermark`, `translated`, `anchor`, `kind_label`;
  `FrameDraft` com `FrameTemplate::draft()`/`edited()` (mesmo ID,
  versão + 1), `duplicate()` (novo ID, versão 1), `blank()` (borda sem
  zonas), `assets()` e remapeamento de imagens.
- `Project.assets`, `FrameEdit::AddAsset/RemoveAsset/Batch` e os deltas
  correspondentes; o `Batch` é aplicado em sequência e revertido por inteiro
  quando um passo falha. Molduras só referenciam imagens existentes; imagem
  em uso não pode ser removida.
- `FramePackage`, `Project::frame_package` e `Project::frame_import_edits`:
  importação reaproveita imagens de mesmo conteúdo e gera IDs novos quando o
  ID da moldura ou da imagem já existe com outro conteúdo.
- `persistence`: `.cadcraft` **formato 3** / `min_reader` 3 (migração 2→3
  identidade), imagens em `assets/<id>.png|jpg` armazenadas sem compressão;
  gravação recusa projeto com imagem sem bytes; leitura confere bytes ×
  metadados. `.cadframe` (`application/x-cad-craft-frame+zip`, formato 1):
  `mimetype`, `manifest.json`, `frame.json` e as imagens, com o mesmo leitor
  defensivo do `.cadcraft`.

### GUI (`cad-gui`)

- `frame_editor.rs` (novo): modo editor ocupando a área de documentos, com
  barra de ferramentas (selecionar, linha, retângulo, círculo, arco, texto,
  texto especial, imagem), "Duplicar seleção", "Excluir seleção", "Alinhar à
  grade", "Ajustar à tela", "Fechar editor"; tela com zoom, deslocamento com
  o botão do meio, grade, área útil, seleção por clique e arraste para mover;
  Delete, Ctrl+D e Esc.
- Painel lateral do editor: propriedades do elemento (coordenadas, tamanhos,
  espessura, texto e traduções `idioma=texto; …`, campo especial com chave e
  índice, altura, alinhamento, rotação, marca d'água) e da moldura (nome,
  descrição, formato A0–A4/personalizado, área útil, zonas uniformes ou
  individuais, numeração 1/A, início, lados).
- Comandos na guia Dados-mestre, grupo "Molduras": Editor, Nova, Duplicar,
  Excluir, Exportar `.cadframe`, Importar `.cadframe`. Pasta da biblioteca de
  molduras em Configurações (diálogos abrem nela).
- `frame_view.rs`: imagens desenhadas como textura (cache por `AssetId`;
  marcador com X quando o conteúdo falta), marcas d'água antes do conteúdo,
  textos especiais mostrados como `<campo>` no editor.
- Cada edição confirmada vira `UpsertFrame` (versão + 1), com a inserção de
  imagem junto num `Batch`, no histórico comum (`Transaction::Frame`).
- Dependência nova autorizada: `image` 0.25 (somente PNG e JPEG).

## Critérios de aceitação

| # | Critério | Evidência | Resultado |
|---|---|---|---|
| 1 | Criar A3 do zero com borda, linhas, texto fixo, 3 especiais e logotipo PNG por eventos egui; atribuir e ver os valores | `frame_editor_ui.rs::frame_is_created_from_scratch_with_texts_specials_and_logo`: comando Nova moldura, ferramentas por clique na barra, cliques na tela, texto digitado no painel, campo especial escolhido no seletor (Número da página, Propriedade da página "Data"), imagem pelo seletor de arquivo; folha mostra "ACME LTDA", "Painel CCM", "1", "03/11/2026" e a textura do logotipo | Aprovado |
| 2 | Editar moldura em uso atualiza páginas e referências | `editing_a_frame_in_use_updates_pages_and_references`: excluir elemento e arrastar texto pela tela (versão + 1), mudar as zonas para 1.3 ← 1.4; troca de campo especial pelo seletor coberta no critério 1. A mudança de zonas usa `commit_frame_draft` (o mesmo caminho do botão "Aplicar moldura"), não o formulário | Aprovado |
| 3 | Desfazer/Refazer exatos | Ctrl+Z nos dois testes GUI (moldura igual à anterior, imagem removida junto com a inserção); `frame_editing.rs::batch_undo_and_redo_restore_exactly` | Aprovado |
| 4 | `.cadframe` exportar/importar idêntico, imagens byte a byte | `cadframe_export_and_import_between_projects` (GUI); `cadframe_round_trip_and_import_reuses_images_and_ids` (Core) | Aprovado |
| 5 | Inválidos recusados sem alteração | `invalid_images_and_frame_files_are_refused_without_changes` (PNG corrompido, imagem grande demais, `.cadframe` inválido: projeto igual e nenhuma imagem); `invalid_cadframe_files_are_refused`, `missing_or_mismatched_image_entries_are_refused`, testes unitários de `asset` | Aprovado |
| 6 | Salvar/reabrir, migração da Etapa 13, fixtures | `watermark_and_images_survive_cadcraft_round_trip`; `versioned_fixtures_keep_opening` (v0, v1, v2 intactas e migradas; nova `etapa14-v3.cadcraft` com imagem e marca d'água) | Aprovado |
| 7 | Testes preservados, validação, Release, documentação, validação na interface | Validação abaixo. Validação no executável adiada: o usuário definiu (2026-11-03) teste do app a cada 3 etapas; próximo ao fim da Etapa 16 | Aprovado (teste no app adiado) |

## Validação

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-03), exit 0:
**480 testes Rust** (466 anteriores + 14 novos: `frame_editing.rs` 8,
unitários `asset` 2, `frame_editor_ui.rs` 4), 0 falhas/ignorados; **15
PowerShell**; fmt, check, clippy `-D warnings`, `git diff --check`.
Dois testes anteriores foram ajustados por mudança legítima de versão de
formato (`persistence.rs`, `frames_ui.rs` passam a usar `FORMAT`).

Release: `cargo build --release --locked -p cad-gui`,
`target/release/cad-gui.exe`, 12.774.912 bytes, SHA-256
`4B192C9B0682A1D6ABBDC20315241DC05AEFC4B3E1926CBD4F94C927EC3C2A3F`.

Defeitos encontrados pelos testes e corrigidos:
- A textura de uma imagem ficava "ausente" para sempre quando a pré-visualização
  era desenhada antes da inserção (cache guardava o resultado vazio). Agora só
  o resultado da decodificação de bytes existentes entra no cache.
- O seletor de campo especial cortava as opções do fim da lista; a lista
  ganhou altura maior.

## Limitações

- Snapshot técnico JSON leva só os metadados das imagens: não pode ser salvo
  como `.cadcraft` enquanto houver imagem sem conteúdo (recusa explícita).
- QR code e hiperlink adiados (decisão da aprovação).
- Textos fixos mostram só o idioma padrão; as traduções são editadas como
  `idioma=texto; …`.
- Cada edição confirmada aumenta a versão da moldura (inclusive pequenos
  ajustes); não há versão por sessão de edição.
- Sem seleção por retângulo, alças de redimensionamento ou rotação gráfica
  no editor; tamanhos e ângulos são editados no painel.
- Arquivos da Etapa 14 (formato 3) não abrem na versão da Etapa 13.
- Diálogos nativos (`rfd`) não são exercitados nos testes (substituídos por
  `ScriptedPicker`).

## Arquivos

Criados: `crates/cad-craft/src/domain/asset.rs`,
`crates/cad-craft/tests/frame_editing.rs`,
`crates/cad-craft/tests/frames_common/mod.rs`,
`crates/cad-craft/tests/fixtures/etapa14-v3.cadcraft`,
`crates/cad-gui/src/frame_editor.rs`, `crates/cad-gui/tests/frame_editor_ui.rs`,
`docs/internal/spec/ETAPA_14_EDITOR_DE_MOLDURAS.md`, `docs/internal/plans/2026-11-03-etapa-14.md`,
`docs/adr/0016-editor-de-molduras-imagens-cadframe.md`, este relatório.

Modificados: `Cargo.lock`; Core `domain/frame.rs`, `ids.rs`, `mod.rs`,
`project.rs`, `project/frames.rs`, `project/snapshot.rs`, `persistence.rs`,
`tests/persistence.rs`; GUI `Cargo.toml`, `app.rs`, `backstage.rs`,
`commands.rs`, `frame_view.rs`, `lib.rs`, `preferences.rs`,
`project_file.rs`, `tests/frames_ui.rs`, `tests/project_lifecycle.rs`;
`docs/internal/memory/*` (estado, handoff, decisões, roadmap, problemas conhecidos,
arquitetura).

Excluídos: nenhum. `docs.rar` (do usuário) não faz parte da etapa.

## Git

`main`, HEAD `98d57af21efbfda30bddabce23ca194aefb07e99` (consulta de
2026-11-03); toda a Etapa 14 está fora do HEAD. Commit + push pelo agente
somente após o OK expresso do usuário para esta etapa: OK dado em 2026-11-03
("Pode versionar"); SHA em `git log`. Não iniciar a Etapa 15.
