# Relatório — Etapa 19 (macros)

Data: 2026-11-04. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_19_MACROS.md`
(aprovado com as cinco decisões recomendadas). Plano:
`docs/internal/plans/2026-11-04-etapa-19.md`. ADR-0021.

## Entregue

| Item | Onde |
|---|---|
| Composição de transformações e rota transformada | `domain/transform.rs` (`then`, `swaps_axes`), `domain/connection_drawing.rs` (`ConnectionRoute::transformed`) |
| Modelo de macro | `domain/macro_def.rs` (`MacroId`, `MacroKind`, `MacroDefinition`, `MacroContent`, prévia, limites) |
| Captura, inserção, Undo/Redo, macro de página | `domain/project/macros.rs` (`capture_macro`, `capture_page_macro`, `insert_macro_content`, `insert_macro`, `insert_page_macro`, `restore_macro_insert`, `MacroReport`) |
| `.cadmacro` | `persistence.rs` (`write_macro_package`, `read_macro_package`, 64 MiB) |
| Histórico | `cad-gui/src/history.rs` (`Transaction::Macro`) |
| Inserção na folha | `interaction.rs` (`Tool::InsertMacro`, `MacroDraft`), `editor_view.rs` (prévia com fios, clique, repetição), `app.rs` (Girar/Espelhar no posicionamento) |
| Biblioteca e navegador | `cad-gui/src/macro_library.rs`; pasta em Configurações (`preferences.macro_library`) |
| Comandos | Inserir > Macros (Navegador, Inserir macro); Dados-mestre > Macros (Criar macro da seleção); Início > Página (Macro de página); menu de contexto |
| Colar/duplicar com fios | `state.rs` (`wired_selection`, `paste_wired`) |

Formato do projeto: `.cadcraft` continua no 6 (macros ficam na biblioteca).

## Validação automática

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-04): exit 0 — `cargo fmt
--check`, `cargo check --workspace --all-targets --locked`, `cargo test
--workspace --locked` (**556 testes Rust**: 539 anteriores + 11 Core em
`cad-craft/tests/macros.rs` + 6 egui em `cad-gui/tests/macros_ui.rs`), Clippy
`-D warnings`, `git diff --check` e **15 testes PowerShell**. Nenhum teste
anterior removido ou alterado.

Critérios cobertos por teste: inserir duas vezes (`-Q2/-K2/-F2`,
`-Q3/-K3/-F3`) com a mesma rede (1); Undo/Redo exatos (2); fio com ponta
fora descartado e relatado (3); outro projeto recebe símbolos e peças e
sobrevive a salvar/reabrir (4); giro 90/180/270 e espelho mantêm fios nas
portas (5); macro de página cria página sem moldura duplicada (6);
`.cadmacro` inválido/truncado/de outro tipo/com referência quebrada recusado
(7); colar e duplicar com fios (8); fixtures v0–v6 seguem abrindo (9).

Build Release: `cargo build --release --locked -p cad-gui` concluído;
`target/release/cad-gui.exe` com 15.538.688 bytes (após a correção da prévia).

## Teste no executável (Etapas 18 e 19) — 2026-11-04

Feito pelo agente com autorização do usuário (acesso liberado na terceira
tentativa), no Release, com projeto de teste fora do repositório
(`teste-etapa19.cadcraft`, `pecas/ACME.K1.cadpart`, `macros/`).

| Verificação | Resultado |
|---|---|
| Selecionar circuito por retângulo → menu de contexto "Criar macro da seleção" | Passou (7 representações; miniatura com 4 símbolos, 3 fios, 3 dispositivos) |
| Exportar `.cadmacro`; configurar pasta de macros; recarregar a biblioteca | Passou |
| Inserir na Página 2 por clique, duas vezes | Passou (-Q2/-F2/-K2 e -Q3/-F3/-K3) |
| Ctrl+Z remove a inserção inteira; Ctrl+Y devolve | Passou |
| Girar 90° durante a inserção | Passou (fios e junção presos às portas) |
| Ctrl+C / Ctrl+V de circuito com fios | Passou (fios e junção colados) |
| Recuperação após fechar o app à força | Passou (macro inserida recuperada) |
| Etapa 18: biblioteca de peças carregada da pasta; atribuir ACME.K1 a -K2; funções "não representadas"; lista de peças | Passou |
| Etapa 18: inserir função não representada pelo Centro de inserção | **Não concluído**: os cliques na faixa direita da tela (painel Centro de inserção) deixaram de chegar ao app (janela sobreposta do ambiente, já registrada na memória). Coberto pelo teste egui `existing_coil_is_placed_by_real_canvas_click_and_undo_restores_points` |

**Defeito encontrado e corrigido no teste:** as prévias sob o cursor
(inserir macro, Colar, texto e gráfico livre) eram pintadas antes da folha e
ficavam escondidas por ela. Agora são pintadas numa camada acima da folha,
recortada à área do editor (`editor_view.rs`). Validação repetida após a
correção: exit 0, 556 Rust + 15 PowerShell; Release 15.538.688 bytes.

**Observações:** a prévia de Colar com fios mostra só os símbolos (os fios
aparecem ao colar). Atribuir uma peça cujos rótulos de ponto (A1/A2) não
coincidem com os do símbolo já inserido (1/2) cria funções novas não
representadas em vez de reaproveitar as existentes (comportamento da
Etapa 18).

## Limitações

Ver `KNOWN_ISSUES.md` (MAC-001 a MAC-005): engenharia física fora da macro,
recortar com fios recusado, sem placeholders nem atualização de ocorrências.
Validação automática não é homologação manual (Etapa 28).

## Git

`main`, HEAD `088637983e0933259f230f123a3777c8eadd17de` (Etapa 17) observado
em 2026-11-04. Etapas 18 e 19 fora do HEAD; `docs.rar` é do usuário e não
entra. Commit + push das duas etapas autorizados pelo usuário se o teste no
app passar.
