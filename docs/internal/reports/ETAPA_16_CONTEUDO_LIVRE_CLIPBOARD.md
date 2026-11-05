# Etapa 16 (Fase B) — conteúdo livre e copiar/recortar/colar

**Estado:** implementada e validada automaticamente; pronta para revisão.
Teste funcional no executável previsto para o fim desta etapa (cadência de 3
etapas), **somente com autorização expressa do usuário para usar o
computador**; ainda não feito. Homologação geral na Etapa 28. Sem commit/push.

Prompt: `docs/internal/spec/ETAPA_16_CONTEUDO_LIVRE_CLIPBOARD.md` (aprovado; revisão
das regras de colar/recortar aceita em 2026-11-03). Decisão: ADR-0018. Base:
`main`, HEAD `7bacd28` (Etapa 14) com a Etapa 15 não versionada na árvore.

## Origem do código

Iniciada pelo Codex (sessão interrompida sem encerramento): conteúdo livre no
Core (`domain/page_graphic.rs`, `Representation::graphic`,
`edit_representations`), formato 4, ferramentas de inserção, comandos
Copiar/Recortar/Colar/Excluir e 9 testes. Concluída pelo Claude Code, que
reaproveitou o modelo, a inserção, a prévia e o histórico e refez as regras de
colar/recortar conforme a revisão aprovada.

## Implementado

### Core (`cad-craft`)

- Conteúdo livre: `PageGraphic` (texto, linha, retângulo, círculo, arco) em
  representações sem alvo elétrico; validação de limites; lote reversível
  `edit_representations`/`restore_representations` (Codex).
- `domain/project/clipboard.rs` (novo): `ClipItem` (instantâneo),
  `clipboard_items`, `cut_representations`, `paste_items`,
  `restore_sheet_edit`, `SheetEditDelta`, `MAX_CLIPBOARD_ITEMS = 10 000`.
  - Colar símbolo de função: `place_symbol` com dispositivo novo
    (`next_device_designation` do mesmo prefixo, mesma classificação e tipo
    de função) ou função nova do mesmo dispositivo; aparência copiada.
  - Recortar: remove ocorrências e vínculos porta→ponto na hora; cadastro
    elétrico permanece; símbolo ligado a fios é recusado.
  - Colar recorte: mesmos IDs e vínculos. Tudo atômico, com reversão em caso
    de erro; desfazer/refazer na ordem inversa/direta dos passos.
- `.cadcraft` formato 4 / `min_reader` 4 (migração 3→4 identidade);
  fixture nova `tests/fixtures/etapa16-v4.cadcraft` (v0–v3 intactas).

### GUI (`cad-gui`)

- Área de transferência da sessão com os instantâneos do Core; prévia,
  encaixe na grade, colagem repetida até Esc; Recortar com mensagem de
  Ctrl+Z; comando novo "Colar no mesmo dispositivo"; Duplicar (Ctrl+D,
  deslocamento 10 mm / −10 mm); histórico `Transaction::Sheet`.
- Duplo clique em texto livre abre "Editar texto" (altura, traço e
  alinhamento preservados).
- `CadApp::has_dialog()` (consulta usada pelos testes).

## Critérios de aceitação

| # | Critério | Evidência | Resultado |
|---|---|---|---|
| 1 | Cinco variantes, editar, mover, selecionar, excluir por eventos reais; render e hit test | `free_graphic_ui.rs`: linha e texto por cliques; retângulo, círculo e arco por cliques; seleção pela borda (centro vazio não seleciona); arraste de 12 mm; Delete e Ctrl+Z; duplo clique edita texto. `free_graphic_geometry.rs`: hit em duas escalas | Aprovado |
| 2 | Copiar/colar múltiplo; símbolo cria dispositivo novo; um passo de Desfazer | `free_clipboard.rs`: `copy_multiple…`, `symbol_paste_creates_new_device_with_next_designation` (designação diferente, mesma classificação, ligações existentes inalteradas, portas ligadas, Undo exato, segunda colagem com outra designação), `paste_same_device_adds_function_and_reference_on_other_page`; Ctrl+C/Ctrl+V e Ctrl+D por teclado | Aprovado |
| 3 | Recortar remove na hora; mover entre páginas preserva IDs e vínculos; recusa atômica | `cut_removes_now_and_first_paste_restores_same_id`, `later_pastes_of_a_cut_are_copies`, `cut_refuses_wired_symbol_and_moves_bindings_of_unwired_one`, `mixed_selection_rejection_preserves_clipboard_and_history`; Core `clipboard_cut_paste_round_trip_and_refusals_leave_project_unchanged` | Aprovado |
| 4 | Salvar/reabrir pela interface; fixtures v0–v4 | `free_content_survives_save_and_reopen_through_the_interface` (Salvar como → outra instância → Abrir; representações idênticas); `versioned_fixtures_keep_opening` | Aprovado |
| 5 | Validação, Release, documentação; teste no app | Abaixo; teste no executável das Etapas 14–16 feito com autorização (seção final) | Aprovado |

## Validação

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-03), exit 0:
**512 testes Rust** (495 da Etapa 15 + 17 da Etapa 16), 0 falhas/ignorados;
**15 PowerShell**; fmt, check, clippy `-D warnings`, `git diff --check`.
Dos testes do Codex, 2 foram reescritos porque descreviam as regras
substituídas (cópia do mesmo alvo elétrico e recorte adiado).

Release: `cargo build --release --locked -p cad-gui`,
`target/release/cad-gui.exe`, 13.283.840 bytes, SHA-256
`2e55b6a77cdf0b1d8357adfee5b3a2f5217800a0993f5faba938d4ab9887503e`.
O executável não foi aberto.

## Limitações

- Fios, junções e marcadores não entram na área de transferência; copiar um
  circuito com os fios fica para as macros (Etapa 19).
- Dispositivo novo recebe designação sem caminho funcional/local (`=`/`+`),
  como `next_device_designation` da Etapa 12.
- Recortar um símbolo deixa dispositivo, função e pontos no cadastro até a
  colagem (aparecem sem ocorrência no navegador nesse intervalo).
- Área de transferência só da sessão: não vai para o sistema nem entre
  projetos.
- Seleção por retângulo e estilos de linha seguem como antes (Etapa 17).

## Arquivos

Criados: `crates/cad-craft/src/domain/page_graphic.rs` (Codex),
`crates/cad-craft/src/domain/project/clipboard.rs`,
`crates/cad-craft/tests/page_graphics.rs`,
`crates/cad-craft/tests/fixtures/etapa16-v4.cadcraft`,
`crates/cad-gui/tests/free_clipboard.rs`, `free_graphic_geometry.rs`,
`free_graphic_ui.rs`, `docs/internal/plans/2026-11-03-etapa-16-rascunho.md`,
`docs/adr/0018-conteudo-livre-e-area-de-transferencia.md`, este
relatório. Modificados: Core `domain/mod.rs`, `project.rs`,
`project/representations.rs`, `representation.rs`, `persistence.rs`,
`tests/persistence.rs`; GUI `app.rs`, `commands.rs`, `dialogs.rs`,
`editor_view.rs`, `history.rs`, `hit_test.rs`, `interaction.rs`,
`panels/insertion.rs`, `panels/properties.rs`, `state.rs`, `symbol_view.rs`,
`viewport.rs`; spec da Etapa 16; `docs/internal/memory/*`. Excluídos: nenhum.
Arquivos da Etapa 15 continuam na árvore, também não versionados.

## Teste funcional no executável (Etapas 14–16, 2026-11-03)

Autorizado pelo usuário ("Autorizo o teste"); Release desta etapa operado
pelo agente com controle do computador. Não é a homologação geral (Etapa 28).
Arquivos gravados só em `%TEMP%\cadcraft-teste-app`; a pasta EPLAN do
usuário foi lida e não teve arquivos alterados.

| Etapa | Passo | Resultado |
|---|---|---|
| 14 | Nova moldura; 10 colunas de 41 mm com rótulos em cima; linha, texto "ACME LTDA", texto especial e logotipo (EPLAN.jpg da pasta de imagens do usuário) | Aprovado; logotipo desenhado no editor e na folha |
| 14 | Moldura padrão do projeto = Moldura 1; página 3 | Texto especial resolvido com o nome do projeto; logotipo preservado pela cópia de recuperação |
| 15 | Importar de outro sistema → `FN1_011.fn1` (prévia, 73 elementos, relatório) e atribuir à página 3 | Aprovado; 10 colunas de 41 mm, área útil 0/16, 410 × 266 mm |
| 15 | Exportar DXF (R12, 73 entidades) e reimportar | Aprovado; prévia idêntica (420 × 297 mm, 73 elementos) |
| 16 | Retângulo e texto livre; duplo clique edita o texto ("Bomba principal") | Aprovado |
| 16 | Bobina da biblioteca (Novo dispositivo, -K2); Ctrl+C, Ctrl+V | Aprovado após correção: -K3 criado, referência /3.8 pela moldura EPLAN |
| 16 | Ctrl+X em -K3; colar na página 5; Ctrl+Z duas vezes | Aprovado: some na hora; volta com o mesmo ID e vínculos (/5.4); desfazer devolve /3.8 |
| 16 | Ctrl+D em -K2 | Aprovado: -K4 ao lado |
| 16 | Salvar como `.cadcraft` | Formato 4 / `min_reader` 4, imagem em `assets/`, conteúdo livre e FN1_011 no `project.json` |

Defeito encontrado e corrigido: no Windows o winit entrega Ctrl+C/X/V como
eventos de área de transferência (`Copy`/`Cut`/`Paste`), não como teclas, e o
`Paste` só existe se a área de transferência do sistema tiver texto. Os
atalhos não funcionavam no executável (os testes enviavam teclas). Agora o
app trata esses eventos e Copiar/Recortar põe um texto curto
("CAD-CRAFT: n item(ns)…") na área de transferência do sistema. Teste novo:
`clipboard_events_from_the_os_drive_copy_cut_and_paste`.

Achados não corrigidos (registrados em KNOWN_ISSUES):
- Atalhos são ignorados enquanto um elemento de painel tem o foco do teclado
  (ex.: após duplo clique na árvore de páginas); clicar na folha resolve.
- Descrição multilinha do EPLAN aparece com "□" no campo de linha única.
- Símbolos antigos ligados diretamente ao dispositivo (alvo `Device`, como na
  demonstração) são colados como cópia do mesmo dispositivo; a regra de
  dispositivo novo vale para símbolos com função (inserção da Etapa 12).
- A cópia de recuperação continua oferecida após "descartar" ao fechar.
- Ambiente: uma janela sempre visível do aplicativo Claude cobre o canto
  superior direito da tela e bloqueia cliques; a janela do CAD-CRAFT foi
  estreitada para o teste (não é defeito do CAD-CRAFT).

Validação após a correção: `scripts/validate.ps1` exit 0, **513 Rust** e 15
PowerShell. Release após a correção: 13.283.328 bytes, SHA-256
`886102ec2c20d8ead9aa70606d652da5063ad4d7a6dd4240da562ff3b42682eb`.
