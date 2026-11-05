# Relatório — Bloco R0 (preparação para publicar em código aberto)

Data: 2026-11-05. Agente: Claude Code. Prompt: `docs/internal/spec/R0_PREPARACAO_PUBLICACAO.md`, aprovado com as cinco decisões recomendadas. Plano: `docs/internal/plans/2026-11-05-r0.md`. ADR-0029.

**Nenhum comportamento do programa mudou.** As saídas de referência foram gravadas antes de qualquer outra mudança e continuam iguais no fim do bloco.

## Entregue

| Item | Onde |
|---|---|
| R0.0 Saídas de referência: 49 impressões digitais (`.cadcraft`, JSON, lista de exibição, PDF, DXF) das fixtures v0–v9 | `crates/cad-craft/tests/baseline_outputs.rs`, `tests/fixtures/baseline/hashes.txt` |
| R0.1 Molduras EPLAN sintéticas no lugar das 19 reais | `tests/fixtures/synthetic/fn1/` (2 arquivos escritos à mão); `tests/interchange.rs`; `cad-gui/tests/frame_editor_ui.rs`; `tests/fixtures/eplan/` removido com `git rm` |
| R0.2 Uma variável de amostras | `CADCRAFT_SAMPLES` em `interchange`, `elmt_import`, `qet_import` e `edz_import`; `.gitignore`; `docs/TESTES_COM_AMOSTRAS.md` |
| R0.3 Dados pessoais e de terceiros | Caminhos do usuário, nome de empresa e pasta local trocados por marcadores em 9 documentos; varredura final limpa |
| R0.4 Licença | `LICENSE` (GPL-3.0, SHA-256 `3972dc97…6986`); `license`, `repository` e `authors` no workspace; SPDX em 239 `.rs` + teste `license_headers.rs`; `deny.toml` |
| R0.5 Entrada pública | `README.md` (inglês), `README.pt-BR.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md` |
| R0.6 CI e toolchain | `.github/workflows/ci.yml` (Windows completo, Linux só o Core, `cargo-deny`); `rust-toolchain.toml` sem target fixa |
| R0.7 Regras e documentos | `AGENTS.md` público e técnico; `AGENTS.local.md` (fora do Git) com o fluxo do autor; `docs/internal/` e `docs/adr/` com `git mv`; 108 arquivos com caminhos atualizados, sem link relativo quebrado; `check-repository.ps1` confere as duas estruturas de regras (+1 teste PowerShell) |

## Verificações feitas

- **Varredura do que vai ser publicado:**
  - nenhum e-mail além do endereço *noreply* do GitHub;
  - nenhum segredo (padrões de chaves OpenAI, GitHub, AWS e chaves privadas);
  - nenhum caminho pessoal ou nome de empresa.
- **Fixtures `.cadcraft` v0–v9:** extraídas e conferidas; só nomes sintéticos ("Fixture Etapa 12", "Comando", "Bobina"…).
- **Licenças das dependências:** todas compatíveis com GPL-3.0 (MIT, Apache-2.0, BSD, Zlib, ISC, Unicode, CC0, BSL, 0BSD, Unlicense; fontes OFL e Ubuntu do egui).
- **Testes opcionais com `CADCRAFT_SAMPLES`**, montada com junções para as pastas locais existentes:
  - 19 molduras `.fn1`;
  - 46 pacotes `.edz` (45 convertidos e relidos, 1 recusado como esperado);
  - 27 projetos `.qet`;
  - amostra de 200 `.elmt`;
  - 13 `.titleblock`.
  - Todos passaram.

## Validação

`scripts/validate.ps1` (2026-11-05) terminou com exit 0: **673 Rust + 16 PowerShell**, zero falhas; saídas de referência idênticas; Release de 20.088.832 bytes (mesmo tamanho de antes do bloco), não executado.

Testes:
- **Rust:** 669 anteriores, menos 2 que dependiam das molduras reais (`authorized_eplan_fixtures_import`, `available_eplan_series_frames_import`), mais 6 novos:
  - `outputs_match_the_recorded_baseline`;
  - `synthetic_eplan_frame_with_offset_origin_imports`;
  - `synthetic_eplan_frame_covers_groups_polylines_fields_and_reports`;
  - `sample_eplan_frames_import_when_available`;
  - `every_rust_file_has_the_spdx_header`;
  - `gpl_license_text_is_present`.
- **PowerShell:** 16, um a mais (`AGENTS.local exige as seções do fluxo do autor`).

## Desvios do prompt

- **Impressão digital:** FNV-1a de 64 bits em vez de SHA-256, para não acrescentar dependência; o uso não é criptográfico (ADR-0029).
- **`cargo-deny` local:** a instalação falhou porque o computador perdeu a resolução de DNS durante a sessão (crates.io e gnu.org inacessíveis). Pela alternativa da decisão 4, a verificação fica no CI (`EmbarkStudios/cargo-deny-action`). O `deny.toml` não foi executado localmente.
- **Texto da GPL:** obtido da cópia local idêntica à oficial (pacote do mintty no Git for Windows), conferida pelo SHA-256 publicado. O site gnu.org estava inacessível.
- **Código de conduta:** adota o Contributor Covenant 2.1 por referência, com link e atribuição, sem copiar o texto integral (R0-L05).
- **Operações de índice:** `git rm` (molduras reais) e `git mv` (documentos) foram feitas porque o prompt aprovado as pedia expressamente. Nenhum commit foi feito.
- **Baseline no Linux:** o teste de saídas de referência fica ignorado fora do Windows (R0-L01).

## Limitações

Ver `docs/internal/memory/KNOWN_ISSUES.md`, R0-L01 a R0-L05.

## Git

`main`, HEAD `b06be631c16a8104ca7b502f26d3c517784701c0` (Etapas 23–26), observado em 2026-11-05. O bloco R0 está fora do HEAD.

Próximos passos:
1. Com o seu OK: commit + push do R0. O primeiro push também roda o CI pela primeira vez.
2. Você cria o repositório público e autoriza, só para isso, o envio do commit inicial limpo.

O `docs.rar` e as amostras ficam fora.
