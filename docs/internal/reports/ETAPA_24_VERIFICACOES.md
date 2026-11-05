# Relatório — Etapa 24 (verificações de consistência; DEC-11-01/02 e CON-001)

Data: 2026-11-04. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_24_VERIFICACOES.md`, aprovado com as cinco decisões recomendadas. Plano: `docs/internal/plans/2026-11-04-etapa-24.md`. ADR-0026.

## Entregue

| Item | Onde |
|---|---|
| Tipos do motor (regras, gravidade, alvos, esquema, ignorados) | `domain/check.rs` (`CHECK_RULES`, `CheckScheme`, `CheckMessage`, `CheckReport`) |
| Regras, alcance, localização, delta do esquema, linhas de exportação | `domain/project/checks.rs` (`check`, `check_target_location`, `set_check_scheme`, `check_export_rows`) |
| DEC-11-01: acoplamento com continuidade | `domain/connector.rs` (`owned`), `project/connectors.rs` (pares, coerência, planejamento), `project/physical_edit.rs` (`UpsertMating`/`DeleteMating`), `project/snapshot.rs` (acoplamentos antigos) |
| Formato 9 | `persistence.rs` (migração 8→9), fixture `etapa24-v9.cadcraft` |
| CSV/XLSX genéricos | `report_export.rs` (`export_rows_csv`, `export_rows_xlsx`) |
| Painel Verificações | `cad-gui/src/checks_ui.rs`: verificar projeto/página em segundo plano, filtros, contadores, duplo clique, F8/Shift+F8, Ignorar com justificativa, exportar, reavaliação automática |
| Comandos e configurações | Ferramentas > Verificação: Verificar projeto, Verificar página; Configurações > Verificações do projeto; `Transaction::CheckScheme` |

Não houve dependência nova.

## Decisões aplicadas

- **Regras:** 15 regras com código estável (tabela no ADR-0026). Cinco situações do prompt já são impedidas pela integridade do modelo: duas funções principais, veia em duas conexões, veias além da capacidade, ponte entre réguas e peça sem código. Elas não ganharam regras que nunca disparariam e ficaram documentadas no ADR.
- **DEC-11-01 (proposta A):** acoplar une as redes pelo mapa explícito de contatos. Desacoplar separa. O número do fio sobrevive a reeditar o acoplamento. Mudar o ponto de um contato acoplado é recusado.
- **DEC-11-02:** o seccionável continua documental; a VER-016 informa os bornes abertos.
- **CON-001:** a VER-002 avisa as designações repetidas; nomes continuam sem unir redes.
- **Reavaliação:** depois de cada edição, o painel recalcula o projeto em segundo plano e mantém só as mensagens já listadas. Na prática, as corrigidas somem. O prompt falava em reavaliar só os alvos tocados; o recálculo completo dá o mesmo resultado visível e custa 0,24 s para 20.040 ocorrências no Release.

## Validação automática

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-04) terminou com exit 0. Etapas executadas, todas com sucesso:
- `cargo fmt --check`;
- `cargo check --locked`;
- `cargo test --locked`: **630 testes Rust**, sendo 618 anteriores mais 12 novos:
  - 9 em `cad-craft/tests/checks.rs`;
  - 3 egui em `cad-gui/tests/checks_ui.rs`;
- Clippy com `-D warnings`;
- `git diff --check`;
- **15 testes PowerShell**.

Testes anteriores ajustados:
- `connectors.rs`: o teste `mating_is_explicit_documental_and_validated`, que afirmava que o acoplamento **não** une redes, passou a `mating_is_explicit_validated_and_gives_continuity`, com as asserções da proposta A aprovada na decisão 2.
- `persistence.rs`: os testes de formato futuro usam 10.

Nenhum teste foi removido.

Critérios cobertos:

| Critério | Teste |
|---|---|
| 1. Cada regra dispara no projeto que a viola e some quando ele é corrigido | `designation_page_and_connector_duplicates_are_reported`, `function_and_part_rules`, `connection_rules`, `marker_rules`, `physical_rules`, `stale_report_is_reported` |
| 2. CON-001: dois `-X1` avisam e não unem redes | `designation_page_and_connector_duplicates_are_reported` |
| 3. DEC-11-01: acoplar une, desacoplar separa, Desfazer exato, migração de projeto antigo | `mating_gives_continuity_and_unmating_separates` |
| 4. DEC-11-02: seccionável aberto informa, rede inalterada | `physical_rules` |
| 5. Desativar, mudar gravidade, ignorar com justificativa, Desfazer, salvar/reabrir | `scheme_disables_changes_severity_and_ignores_with_undo_and_persistence`, `ignore_with_reason_is_undoable_and_list_exports_csv` |
| 6. Painel: verificar, contadores, duplo clique navega, F8/Shift+F8, correção remove a mensagem, CSV | `check_project_lists_messages_and_double_click_navigates`, `fixing_the_problem_removes_the_message_automatically`, `ignore_with_reason_is_undoable_and_list_exports_csv` |
| 7. 10.000 ocorrências | `ten_thousand_occurrences_are_checked_within_budget` |
| 8. Formato 9 com fixture v9; v8 abre | `scheme_disables_…` |

Desempenho no Release: 20.040 ocorrências (334 páginas × 30 bobinas e fios) e 10.050 mensagens verificadas em **0,24 s**, contra a meta de 2 s. Em Debug, depois de indexar funções e peças, o tempo ficou abaixo do limite de 60 s.

Build Release: `cargo build --release --locked -p cad-gui` concluído; `target/release/cad-gui.exe` tem 18.887.680 bytes. O app não foi executado (sem teste no executável nesta etapa; o próximo é ao fim da 25).

## Limitações

Ver `KNOWN_ISSUES.md` (VER-L01 a VER-L05). Validação automática não é homologação manual (Etapa 28).

## Git

`main`, HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-04. As Etapas 23 e 24 estão fora do HEAD; o commit + push aguarda o OK expresso. O rascunho da Etapa 26 e o `docs.rar` ficam fora.
