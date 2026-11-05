# Etapa 12 (Marco M0.1) — arquivo `.cadcraft` e inserção com dispositivo

**Estado:** implementada e validada automaticamente; pronta para revisão do
usuário. **Não homologada manualmente** (homologação adiada até após a
Etapa 15). Sem staging, commit ou push executados pelo agente.

Prompt: `docs/internal/spec/ETAPA_12_PROJETO_REAL_PERSISTENCIA.md` (seção 7 da
auditoria CAD-CRAFT × QElectroTech, aprovada em 2026-11-03 com autorização das
dependências `zip` e `rfd`). Plano: `docs/internal/plans/2026-11-03-etapa-12.md`.
Decisão: ADR-0014.

## Base

- `main`, HEAD `34785ab03895f8c691838fb76201427831f28349` (Etapa 11
  commitada pelo usuário). `git ls-remote origin refs/heads/main` em
  2026-11-03 11:07 (-03:00): `f1b173e…` — o commit da Etapa 11 ainda não está
  no GitHub (etapa ímpar: só commit).
- Linha de base: 418 testes Rust + 15 PowerShell.
- `docs.rar` não versionado pertence ao usuário e não foi tocado.

## Implementado

### Core (`cad-craft`)

- `persistence` (novo, `src/persistence.rs`): `write_project`,
  `write_project_to_vec`, `read_project`, `read_project_from_slice`,
  `Manifest`, `LoadedProject` (`source_format`, `read_only`, `migrated()`),
  `PersistenceError` com mensagens em português.
  - v1: ZIP com `mimetype` (primeira entrada, Stored), `manifest.json`,
    `project.json`; `FORMAT = 1`, `MIN_READER = 1`.
  - v0: snapshot técnico JSON (BOM/espaços tolerados) migrado por cadeia.
  - Recusas antes de ler o projeto: nomes inseguros, duplicados (também por
    maiúsculas), criptografia, compressão não suportada, >4.096 entradas,
    >1 GiB declarado, bytes reais acima do limite, cabeçalhos inconsistentes.
  - Validação completa do agregado na gravação e na leitura.
- `Project::place_symbol` / `restore_placement` / `next_device_designation`
  (`domain/project/placement.rs`): dispositivo novo ou existente, função
  principal/auxiliar, um ponto por porta, ocorrência e vínculos numa transação.
- `Project::edit_document` / `restore_document_edit` / `remove_document`
  (`domain/project/document_edit.rs`).
- Ajustes: `check_binding` e `page_mut` visíveis no módulo `project`;
  varredura de referências em propriedades generalizada
  (`properties_reference`) sem mudar o comportamento de `remove_page`.

### GUI (`cad-gui`)

- `project_file.rs` (novo): `save`/`load`, `write_atomic`, `FileLock`
  (`.lock`, `acquire`/`take_over`, remoção segura ao soltar), `ProjectFile`,
  `Recovery` (`store`/`entries`/`load`/`discard`), `push_recent`,
  `FilePicker` (`NativePicker` com `rfd`, `ScriptedPicker`), `FileServices`.
- Comandos `OpenProject` (Ctrl+O), `SaveProject` (Ctrl+S, antes planejado) e
  `SaveProjectAs` (Ctrl+Shift+S); barra de acesso rápido com Abrir/Salvar;
  tela inicial com "Abrir projeto .cadcraft"; Backstage: Salvar, Salvar como,
  página Abrir (procurar, recentes, recuperação com Restaurar/Descartar),
  Status com arquivo associado.
- Diálogos: conflito de trava (somente leitura ou assumir), recuperação na
  abertura, aviso de alterações "não salvas".
- Título da janela com nome do arquivo e "somente leitura".
- Recuperação automática (primeira alteração e a cada 120 s) ativada somente
  em `CadApp::new`; recentes persistidos em `cad-craft.recent.v1`.
- Histórico: `Transaction::Placement` e `Transaction::Document`; criar
  documento, criar/excluir/reordenar/mover página deixaram de limpar o
  histórico; abas de páginas removidas por Desfazer fecham.
- Centro de inserção: associação padrão "Novo dispositivo" (prefixo editável,
  sugerido por símbolo), "Função em dispositivo existente" e "Entidade
  existente ou sem associação" (comportamento anterior). Resumo compacto
  "Nome · cria -K3" mantém a lista de símbolos visível.

### Dependências (autorizadas)

| Crate | Versão | Licença | Uso |
|---|---|---|---|
| `zip` | 9.3.0 (sem features padrão, `deflate-flate2`) | MIT | contêiner `.cadcraft` (Core; dev-dep na GUI para testes) |
| `flate2` | 1.1.10 (`rust_backend`) | MIT/Apache-2.0 | apenas seleciona o backend Rust exigido pelo `zip` |
| `serde_json` | 1 | MIT/Apache-2.0 | passou de dev para normal no Core |
| `rfd` | 0.17.2 | MIT | diálogos nativos Abrir/Salvar |

`Cargo.lock` atualizado. Nenhum código do QElectroTech (GPL) foi copiado.

## Critérios de aceitação

| # | Critério | Evidência | Resultado |
|---|---|---|---|
| 1 | Fluxo real ponta a ponta | `cad-gui/tests/project_lifecycle.rs::real_project_is_created_saved_reopened_edited_and_saved_again`: botão "Criar sessão em memória", digitação e "Confirmar" nos diálogos de projeto, documento e 2 páginas; 4 símbolos por cliques na folha (2 dispositivos K1/K2 + 2 contatos como funções); 3 conexões por cliques nas portas (3 redes de 2 pontos); Ctrl+S → Salvar como; Fechar; Ctrl+O; igualdade do agregado; mover símbolo e renomear dispositivo; Ctrl+S no mesmo arquivo; releitura igual | Aprovado. Escolha do plano de inserção, mover (após seleção por clique) e renomear usam a API da sessão (mesmo caminho do painel/propriedades) |
| 2 | Demonstração ida e volta | Core: `persistence.rs::round_trip_preserves_the_complete_aggregate` (fixture física completa das Etapas 3–11 + amostra da Etapa 12); GUI: `demonstration_projects_round_trip_through_cadcraft` (duas demonstrações) | Aprovado |
| 3 | v0/v1 e versões | `tests/fixtures/etapa12-v0-snapshot.json` e `etapa12-v1.cadcraft` (`versioned_fixtures_keep_opening`); `newer_formats_open_read_only_or_are_refused`; GUI `legacy_snapshot_migrates…` | Aprovado |
| 4 | ZIP malicioso sem tocar o disco | `unsafe_entry_names_are_refused`, `duplicate_names_exact_or_by_case_are_refused`, `declared_sizes_and_entry_counts_are_bounded`, `encrypted_entries_are_refused`; GUI confere a pasta inalterada e o projeto atual preservado | Aprovado |
| 5 | Falha de gravação preserva o anterior | `project_file::tests::failed_save_preserves_previous_project_file` (+ teste existente de `snapshots.rs`) | Aprovado |
| 6 | Trava e recuperação | `second_session_gets_lock_warning_and_read_only_copy`, `reopening_the_same_file_keeps_the_session_lock`, `recovery_copy_is_offered_after_abrupt_termination`, `lock_is_exclusive…`, `recovery_entries_round_trip_and_discard` | Aprovado |
| 7 | Undo/Redo exatos | Core `placement.rs` (6 testes de colocação/Undo/recusas, documentos em ordem reversa); GUI `undo_redo_restore_placements_and_page_operations_exactly` (Ctrl+Z/Ctrl+Y) | Aprovado |
| 8 | Acima de 8 MiB, tempo medido | `persistence_scale.rs`: JSON 16,2 MiB → `.cadcraft` 1,2 MiB; gravação 2,88 s, leitura 2,21 s (Debug, inclui validação) | Aprovado |
| 9 | Validação, Release, docs, sem Git | Abaixo | Aprovado |

## Validação

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-03), exit 0:
`cargo fmt --check`, `cargo check --workspace --all-targets --locked`,
`cargo test --workspace --locked`, `cargo clippy … -D warnings`, testes
operacionais PowerShell, `git diff --check`, `git diff --cached --check`.

- **452 testes Rust aprovados, 0 falhas, 0 ignorados** (418 anteriores
  preservados + 34 novos) e **15 testes PowerShell** aprovados.
- Novos: Core `placement.rs` 9, `persistence.rs` 11, `persistence_scale.rs` 1,
  unitário `persistence` 1; GUI `project_file` unitários 5,
  `project_lifecycle.rs` 7.
- Release: `cargo build --release --locked -p cad-gui` aprovado;
  `target/release/cad-gui.exe` (tamanho e SHA-256 na seção final).

A contagem de 452 inclui a regressão de reabertura do mesmo arquivo,
acrescentada após a primeira execução do script (451); a validação final foi
repetida com o conjunto completo.

## Limitações

Registradas em `KNOWN_ISSUES.md` (FILE-001…005, PLC-001…003, M0-001):
trava cooperativa sem detecção de processo; `project.json` estrito exige
elevar `min_reader` em mudanças de conteúdo; histórico não persistido e
`Edit::apply` ainda limpa o histórico; diálogo `rfd` sem teste automatizado;
Desfazer de inserção revalida o agregado; numeração simples por prefixo;
renomear/excluir documento inexistentes; nenhuma homologação manual.

## Arquivos

Criados:
`crates/cad-craft/src/persistence.rs`,
`crates/cad-craft/src/domain/project/placement.rs`,
`crates/cad-craft/src/domain/project/document_edit.rs`,
`crates/cad-craft/tests/placement.rs`, `persistence.rs`,
`persistence_scale.rs`, `tests/fixtures/etapa12-v0-snapshot.json`,
`tests/fixtures/etapa12-v1.cadcraft`,
`crates/cad-gui/src/project_file.rs`,
`crates/cad-gui/tests/project_lifecycle.rs`,
`docs/internal/spec/ETAPA_12_PROJETO_REAL_PERSISTENCIA.md`,
`docs/internal/plans/2026-11-03-etapa-12.md`,
`docs/adr/0014-arquivo-cadcraft-e-insercao-com-dispositivo.md`,
este relatório.

Modificados: `Cargo.lock`, `crates/cad-craft/Cargo.toml`, `src/lib.rs`,
`src/domain/project.rs`, `project/connection_drawing.rs`,
`project/documentation.rs`; `crates/cad-gui/Cargo.toml`, `src/app.rs`,
`backstage.rs`, `commands.rs`, `dialogs.rs`, `editing.rs`, `history.rs`,
`interaction.rs`, `lib.rs`, `panels/insertion.rs`, `ribbon.rs`, `state.rs`,
`viewport.rs`; `docs/internal/memory/ARCHITECTURE.md`, `CURRENT_STATE.md`,
`DECISIONS.md`, `HANDOFF.md`, `KNOWN_ISSUES.md`, `ROADMAP.md`.

Excluídos: nenhum.

## Build Release

`cargo build --release --locked -p cad-gui` com o código final:
`target/release/cad-gui.exe`, 11.686.400 bytes, SHA-256
`bc30d5fdc7680622e828bac3f3d7d958c99631894439f89af740a709ca98aeed`.
O executável não foi aberto manualmente (sem homologação).

## Git e próxima ação

Somente consultas (`status`, `log`, `rev-parse`, `ls-remote`). Etapa 12 é
**par**: o usuário faz **commit + push manuais**. O push também enviará o
commit da Etapa 11 (`34785ab`), ainda ausente do remoto. Backup externo
independente recomendado. Não iniciar a Etapa 13.
