# Relatório — Etapa 25 (importação de projetos QElectroTech `.qet`)

Data: 2026-11-04. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_25_IMPORTACAO_QET.md`, aprovado com as cinco decisões recomendadas. Plano: `docs/internal/plans/2026-11-04-etapa-25.md`. ADR-0027. Cobertura das amostras: `docs/internal/reports/ETAPA_25_COBERTURA_QET.md`.

O código-fonte do QElectroTech (GPL) foi consultado, com autorização do usuário, só para entender o formato. A implementação é própria; nenhuma função foi copiada nem traduzida.

## Entregue

| Item | Onde |
|---|---|
| Importador `.qet` → projeto novo | `cad-craft/src/interchange/qet.rs` (`import`, `QetOptions`, `QetImport`, `QetStats`) |
| Definição embutida e carimbo embutido reaproveitando os importadores | `interchange/elmt.rs` (`import_definition`), `interchange/titleblock.rs` (`import_template`) |
| Rota com geometria importada, sem desvio automático | `domain/connection_drawing.rs` e `project/connection_drawing.rs` (`ConnectionDrawingEdit::CreateRouteExact`) |
| Arquivo > Importar > Projeto QElectroTech | `cad-gui/src/qet_import_ui.rs`: Procurar, Analisar em segundo plano, prévia, Criar projeto importado, abrir Verificações |
| Comando "Importar projeto QElectroTech" | `commands.rs` (Início > Sessão), `app.rs` (`PendingAction::ImportQet`), `project_file.rs` (`open_qet`) |

Não houve dependência nova. O formato `.cadcraft` continua 9.

## Decisões aplicadas

1. **Só `.qet`.** `.qetz` fica pendente (QET-L01).
2. **Folha:** tamanho pela grade do QET e pela altura do carimbo, na escala do `.elmt` (0,5 mm/px). Cada combinação de grade e carimbo gera uma moldura com zonas e o carimbo embutido.
3. **Mestre e escravos:** um dispositivo; a bobina é a função principal e os contatos são principais ou auxiliares.
4. **Número do fio:** travado.
5. **Projeto novo:** substitui a sessão, pedindo confirmação se houver alterações pendentes. Fica sem arquivo até o Salvar como.

Ajustes encontrados com as amostras reais e documentados no ADR-0027:
- **Condutores no formato QET 0.9+:** referência por UUID do elemento + UUID do terminal da definição.
- **`CreateRouteExact`:** o percurso original atravessava corpos de símbolos, e a rota do desenho interativo recusava cerca de 27% dos fios de `industrial.qet`.
- **Marcador de relatório de folha:** a porta é deslocada para a origem, regra dos marcadores.
- **Desenhos sem terminais:** viram ocorrência sem dispositivo.
- **Rótulos repetidos:** agrupados no mesmo dispositivo.
- **Carimbos com logotipo:** as imagens entram por `frame_import_edits`.

## Validação automática

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-04) terminou com exit 0:
- `cargo fmt --check`, `cargo check --locked`, Clippy com `-D warnings` e `git diff --check`, sem falhas;
- **640 testes Rust** (630 anteriores + 10 novos), nenhum removido;
- **15 testes PowerShell**.

Build Release: `cargo build --release --locked -p cad-gui` concluído; `target/release/cad-gui.exe` tem 19.097.600 bytes. O app não foi executado.

Testes novos (10):
- 7 em `cad-craft/tests/qet_import.rs`:
  - projeto sintético com 2 folhas, mestre e escravo, fio 12 travado, relatório de folha, texto HTML, forma e desenho sem terminais;
  - coordenadas, rotação e portas;
  - carimbo ausente, elemento fora da coleção e rótulo repetido;
  - entradas malformadas e hostis (XXE, escala 0, terminal inexistente);
  - salvar, reabrir, PDF e DXF;
  - condutor por UUID;
  - amostras reais opcionais.
- 3 egui em `cad-gui/tests/qet_import_ui.rs`:
  - Procurar → Analisar → prévia → Criar → Verificações;
  - arquivo inválido e extensão errada;
  - confirmação com alterações pendentes.

| Critério | Teste |
|---|---|
| Folhas, molduras com zonas, carimbo e metadados | `synthetic_project_converts_pages_devices_wires_reports_and_texts` |
| Mestre e escravos num dispositivo, função principal | idem |
| Fio numerado travado | idem; `conductors_by_element_and_terminal_uuid_are_resolved` |
| Relatórios de folha agrupados | idem |
| Posição, rotação e portas | `coordinates_orientation_and_ports_follow_the_qet_sheet` |
| Relatório do que não converte | `missing_template_label_conflicts_and_unknown_elements_are_reported` |
| Segurança | `malformed_and_hostile_input_is_rejected` |
| Salvar, reabrir, PDF e DXF | `imported_project_saves_reloads_and_exports` |
| GUI | `cad-gui/tests/qet_import_ui.rs` (3) |
| Amostras reais (27) e desempenho | `reference_samples_import_when_available` com `CAD_CRAFT_QET_SAMPLES`: 27/27 sem erro; `industrial.qet` (50 folhas) de 1,6 s a 3,6 s em Release, meta 10 s |

## Limitações

`KNOWN_ISSUES.md`, QET-L01 a QET-L07. Por decisão do usuário (2026-11-04), não houve teste no executável nesta etapa. O teste no app das Etapas 23 a 26 será feito depois de o Codex concluir a Etapa 26. Validação automática não é homologação (Etapa 28).

## Git

`main`, HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-04. As Etapas 23, 24 e 25 estão fora do HEAD. O commit + push vem depois do teste no app, com o OK do usuário. O `docs.rar` e as amostras `.qet`/`.edz` ficam fora.
