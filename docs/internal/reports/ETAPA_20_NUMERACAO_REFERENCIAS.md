# Relatório — Etapa 20 (numeração automática e referências cruzadas)

Data: 2026-11-04. Agentes: Codex (início) e Claude Code (conclusão). Prompt:
`docs/internal/spec/ETAPA_20_NUMERACAO_REFERENCIAS.md` (aprovado com as cinco
decisões recomendadas). Plano: `docs/internal/plans/2026-11-04-etapa-20.md`. ADR-0022.

## Entregue

| Item | Onde |
|---|---|
| Regra de numeração no projeto | `domain/numbering.rs` (`NumberingScheme`: formato de dispositivo e de fio, ordem, início/passo, manter travadas, usar na inserção), `project.rs`, `snapshot.rs` |
| Trava de designação | `domain/device.rs` (`designation_locked`), `DeviceEdit::SetDesignationLocked`, caixa "Travar designação" em `device_properties.rs` |
| Numerar dispositivos | `project/numbering.rs` (`preview_device_numbering`, `apply_device_numbering`, `restore_device_numbering`): por prefixo, ordem página → coluna → linha ou linha a linha, página/coluna (-K2.4, -K2.4.2), conflitos na prévia, funções e peças acompanham |
| Numerar conexões | `preview_connection_numbering`/`apply_connection_numbering`: um número por rede, potencial nomeado vence, números travados reservados; número guardado em `Connection` (decisão 3) |
| Número de fio manual | `set_connection_number` (validado) + `Transaction::ConnectionNumber`; campo e trava em Propriedades da conexão |
| Rótulo do fio na folha | `viewport.rs` (meio do trecho mais longo ou posição guardada na rota), `ConnectionDrawingEdit::SetNumberLabelPosition` |
| Numerar páginas | `preview_page_numbering`/`apply_page_numbering`: prefixo e início, ordem preservada, referências `/página.coluna` recalculadas |
| Espelho de contatos | `Project::contact_mirror` (derivado, decisão 4); painel do dispositivo com links que navegam; texto sob a função principal na folha (`frame_view.rs`) |
| Referências de interrupção | `frame_view.rs`: interrupções do mesmo grupo mostram a referência umas das outras |
| Numeração ao inserir | `next_device_designation_at` usado na inserção, na colagem e nas macros quando "Usar a regra ao inserir" está ligado (Configurações) |
| Histórico | `history.rs`: `NumberDevices`, `NumberConnections`, `NumberPages`, `ConnectionNumber` |
| Comandos e diálogo | Dispositivos > Numerar dispositivos, Conexões > Numerar conexões, Início > Página > Numerar páginas; `numbering_ui.rs` (regra, alcance, prévia antes/depois, conflitos, Aplicar) |
| Persistência | `.cadcraft` formato 7 / `min_reader` 7, migração 6→7, fixture `etapa20-v7.cadcraft` |

## Divisão do trabalho

O Codex escreveu o Core, a persistência, os comandos, o diálogo básico e
13 testes, e parou sem marcar o plano. Nesse ponto `project_lifecycle`
falhava. O Claude Code concluiu a etapa:

- corrigiu o ajudante desse teste;
- aplicou a ordem da regra e o "manter travadas";
- eliminou as colisões no modo página/coluna, para dispositivos e para fios;
- reservou os números de potenciais e de fios travados;
- passou a validar e registrar no histórico a edição manual do número do fio;
- completou o diálogo;
- acrescentou 8 testes.

## Validação automática

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-04) terminou com exit 0.
Etapas executadas, todas com sucesso:

- `cargo fmt --check`;
- `cargo check --workspace --all-targets --locked`;
- `cargo test --workspace --locked`: **577 testes Rust**, sendo 556 anteriores
  + 12 em `cad-craft/tests/numbering.rs` + 6 egui em
  `cad-gui/tests/numbering_ui.rs` + 3 nos testes de persistência, desenho de
  conexões e macros;
- Clippy com `-D warnings`;
- `git diff --check`;
- **15 testes PowerShell**.

Testes anteriores alterados:

- `persistence.rs`: as fixtures v6 agora são migradas, porque o formato subiu
  para 7. Os testes de formato futuro passaram a usar 8.
- `parts.rs`: ajuste de construção.
- `project_lifecycle.rs`: no ajudante, a busca de rótulo passa a ignorar
  botões.

Nenhum teste foi removido.

Critérios cobertos por teste:

| Critério | Teste |
|---|---|
| 1. Diálogo egui: -K9/-K5/-K2 em três páginas → -K1/-K2/-K3; função e peça acompanham; Desfazer/Refazer exatos | `three_pages_are_numbered_in_page_order_and_undo_redo_are_exact` |
| 2. Designação travada: conflito mostrado, Aplicar desabilitado, nada muda | `locked_conflict_is_shown_and_apply_does_nothing`, `locked_device_collision_is_previewed_and_rejected` |
| 3. Página/coluna gera -K2.4 (e -K2.4.2 na mesma coluna) | `page_column_rule_names_by_frame_zone_without_collisions` |
| 4. Mesma rede com o mesmo número, redes distintas com números distintos, potencial/travado mantido, Desfazer/Refazer | `network_numbering_keeps_one_label_and_respects_locked_number`, `distinct_networks_get_distinct_numbers_and_skip_reserved_ones`, `connection_and_page_dialogs_apply_from_real_clicks`, `manual_wire_number_is_one_undoable_transaction` |
| 5. Numerar páginas atualiza `/página.coluna` | `page_numbering_keeps_order_and_recomputes_reference` |
| 6. Espelho: tipo, pontos, referência; não representado; mover atualiza; clique navega | `contact_mirror_is_derived_including_unrepresented_contacts`, `contact_mirror_lists_contacts_and_navigates_on_click` |
| 7. Formato 7 conserva números, regra e travas; v0–v6 abrem | `format_seven_preserves_numbering_and_old_fixtures_open`, `versioned_fixtures_keep_opening` |

Build Release: `cargo build --release --locked -p cad-gui` concluído.
`target/release/cad-gui.exe` tem 15.815.168 bytes. O app não foi executado:
não há teste no executável nesta etapa, e o próximo é ao fim da Etapa 22,
com autorização do usuário.

## Limitações

Ver `KNOWN_ISSUES.md` (NUM-001 a NUM-007). Validação automática não é
homologação manual (Etapa 28).

## Git

`main`, HEAD `98ef54ab48cb5e5f6ad24ca0cd170709319e2e1c` (Etapas 18 e 19),
observado em 2026-11-04. A Etapa 20 inteira está fora do HEAD. O `docs.rar`
é do usuário e não entra. Commit + push somente após o OK expresso do
usuário.
