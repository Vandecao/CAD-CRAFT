# CAD-CRAFT — Bloco R0: preparação para publicar em código aberto

**Estado: APROVADO pelo usuário em 2026-11-05 ("Aprovo todas"), com as cinco decisões recomendadas:**
1. código, comentários de API e commits novos em inglês, interface em pt-BR, README em inglês com `README.pt-BR.md`;
2. molduras reais removidas;
3. CI já no repositório privado;
4. `cargo-deny` instalado localmente pelo agente;
5. `docs/internal/` e `docs/adr/`.

- **Plano aprovado:** `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md` (ordem R0 → publicação → R1 → Etapa 27 → R2…R6).
- **Origem:** parecer técnico do usuário (`PARECER_TECNICO_CAD-CRAFT_v2.md`, seções 5, 9 e 10).
- **Base:** `main` em `b06be63`, validação 669 Rust + 15 PowerShell. Fora do HEAD e entrando neste bloco: o prompt da Etapa 27 e o plano R0–R6.

## Objetivo

Deixar o repositório pronto para ser publicado sob **GPL-3.0-or-later**:
- sem arquivos de terceiros nem dados pessoais;
- com licença, README e arquivos para colaboradores;
- com uma rede de segurança que prove que a reorganização (R1 em diante) não muda nada.

**Nenhum comportamento do programa muda.** A publicação em si é um passo separado, depois deste bloco, com autorização específica.

## Escopo

### R0.0 — saídas de referência (rede de segurança)

- Teste `crates/cad-craft/tests/baseline_outputs.rs`, que gera a partir de projetos fixos:
  - os `.cadcraft` das fixtures v0–v9 regravados em v9;
  - o projeto de saída usado nos testes de PDF e DXF.
- Para cada um, compara o SHA-256 com `tests/fixtures/baseline/hashes.txt`:
  - o `.cadcraft` gravado;
  - o PDF, com o campo `/CreationDate` mascarado, porque contém a data do dia;
  - cada DXF por página;
  - o texto da lista de exibição.
- `CAD_CRAFT_WRITE_BASELINE=1` regrava o arquivo de hashes, como as fixtures; só por decisão explícita.
- **Ordem:** este teste entra primeiro, antes de qualquer outra mudança do bloco, e passa com o código atual.

### R0.1 — molduras EPLAN reais fora do repositório

- Criar `tests/fixtures/synthetic/fn1/` com molduras `.fn1` mínimas escritas à mão, só com as estruturas que o importador usa:
  - zonas;
  - linhas, retângulo de borda e textos;
  - campos especiais;
  - a prévia SVG com `matrix(…)`;
  - textos multilíngues.
- Migrar para elas todas as asserções de `authorized_eplan_fixtures_import` (`tests/interchange.rs`): 10 colunas, folha 420 × 297, borda de (0,175; 0,175) a (420,175; 297,175), linha (400, 4)–(420, 4) e as do `FN1_002`.
- Migrar também o uso em `cad-gui/tests/frame_editor_ui.rs`.
- O laço sobre as molduras reais vira teste opcional por `CADCRAFT_SAMPLES` (ver R0.2).
- Remover `crates/cad-craft/tests/fixtures/eplan/` com `git rm`. Os arquivos continuam com você, fora do Git, e podem ir para a pasta de amostras.

### R0.2 — uma variável para as amostras

- `CADCRAFT_SAMPLES` aponta para uma pasta com `eplan/fn1/`, `eplan/edz/`, `qet/` e `elmt/`. Sem a variável, os testes opcionais passam sem rodar.
- Ela substitui `CAD_CRAFT_QET_SAMPLES`, `CAD_CRAFT_EDZ_SAMPLES` e o caminho fixo `<pasta de referências>…` do teste de `.elmt`.
- `.gitignore` ganha `/samples/` e `/docs.rar`.
- Novo `docs/TESTES_COM_AMOSTRAS.md`: como cada um monta a própria pasta.

### R0.3 — dados pessoais e de terceiros

- Revisar os 13 arquivos versionados que citam:
  - caminho do seu usuário do Windows;
  - a empresa "<empresa>";
  - a pasta local de referências.
  - (Lista levantada em 2026-11-05, incluindo `elmt_import.rs`, `interchange.rs`, `HANDOFF.md`, `BOOTSTRAP_AMBIENTE.md` e as specs e relatórios 13, 15, 17, 25 e 26.)
- Trocar por marcadores neutros (`<pasta de amostras>`, `<empresa>`).
- Conferir de novo as fixtures `.cadcraft` v0–v9: só dados sintéticos.
- Varredura final por e-mail, nome de usuário do Windows e chaves/tokens. O resultado vai para o relatório.

### R0.4 — licença

- `LICENSE` com o texto integral da GPL-3.0 (gnu.org).
- `[workspace.package]` com `license = "GPL-3.0-or-later"`, `repository`, `authors` e `license.workspace = true` nas crates.
- Cabeçalho SPDX em todos os `.rs`, inclusive testes:
  ```
  // SPDX-License-Identifier: GPL-3.0-or-later
  // Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
  ```
- `deny.toml` com a lista de licenças das dependências atuais (todas compatíveis, conferido em 2026-11-05) e o bloqueio de licenças incompatíveis.

### R0.5 — README e documentos de entrada

- **README novo** (bilíngue conforme a decisão 1):
  - o que é;
  - **tabela de formatos** de importação e exportação, com o que é parcial ou pendente (projetos EPLAN completos, `.qetz`);
  - aviso de marcas EPLAN e QElectroTech;
  - funcionalidades;
  - como compilar e rodar;
  - estado alfa;
  - licença e como contribuir.
  - **Captura de tela** do programa: o Claude Code tira no teste do app, com autorização.
- `CONTRIBUTING.md`:
  - compilar e testar (`scripts/validate.ps1` e os comandos do CI);
  - regra de idioma (decisão 1);
  - DCO com `Signed-off-by`;
  - Conventional Commits.
- `CODE_OF_CONDUCT.md`: Contributor Covenant 2.1, com a atribuição exigida.
- `SECURITY.md`: relato privado pelo GitHub (*private vulnerability reporting*), sem e-mail pessoal.

### R0.6 — CI e toolchain

- `.github/workflows/ci.yml`:
  - **Windows:** `fmt`, `clippy -D warnings`, `test --workspace --locked` e `cargo deny check`.
  - **Linux:** só o Core (`-p cad-craft`), porque a GUI exige bibliotecas de janela no Linux.
  - Sem segredos e sem deploy.
- `rust-toolchain.toml`: manter a versão fixa e tirar a linha `targets`, para não forçar Windows. A versão mínima (`rust-version`) fica para o R2.

### R0.7 — regras dos agentes e documentos internos

- **`AGENTS.md` público:** só regras técnicas (arquitetura, unidades, testes, validação, estilo).
- **`AGENTS.local.md`:** as regras do seu fluxo pessoal (OK antes de commit, política Git, cadência do teste no app). Fica **fora do Git** (`.gitignore`), e o `AGENTS.md` público avisa que ele existe localmente.
- **Movimentação com `git mv`:**
  - `docs/spec`, `docs/plans`, `docs/reports` e `docs/memory` vão para `docs/internal/`;
  - os ADRs vão para `docs/adr/`;
  - links internos atualizados.
- Ao fim do bloco, `HANDOFF` e `CURRENT_STATE` apontam os caminhos novos.

## Critérios de aceitação

1. O teste de saídas de referência (R0.0) passa **antes e depois** do bloco, com os mesmos hashes.
2. Todos os testes anteriores passam (669 ou mais).
   - As asserções das molduras reais passam nas sintéticas.
   - Com `CADCRAFT_SAMPLES` apontando para a pasta local, os testes opcionais rodam `.fn1`, `.qet`, `.edz` e `.elmt`.
3. `git ls-files` não tem `.fn1` reais, e a varredura de dados pessoais não encontra nada.
4. `LICENSE`, campos de licença e SPDX em 100% dos `.rs` (teste ou script que confere).
5. `cargo deny check licenses` passa localmente (o agente instala o `cargo-deny` só com autorização; senão, só no CI).
6. `scripts/validate.ps1` passa; o Release compila; o app não é executado.
7. Relatório `docs/internal/reports/R0_PREPARACAO_PUBLICACAO.md` e memória atualizada. Commit + push só com o seu OK.

## Decisões pedidas ao usuário

1. **Idioma** (decisão 5 do plano):
   - **Recomendado:**
     - código, comentários de API (`///`, `//!`) e commits **novos** em inglês, a partir da publicação;
     - a interface continua em pt-BR;
     - README em inglês com `README.pt-BR.md`;
     - os comentários antigos em português são convertidos aos poucos no R6.
   - Alternativa: tudo em português, que é o padrão de hoje.
2. **Molduras reais:** remover dos testes, mantendo os arquivos com você fora do Git (recomendado, é obrigatório para publicar).
3. **CI no GitHub Actions** já no repositório privado. Recomendado: sim; consome minutos do plano gratuito do GitHub, sem custo esperado neste volume.
4. **`cargo-deny`:**
   - Recomendado: o agente instala localmente (`cargo install cargo-deny`, ferramenta de código aberto bem conhecida) para validar antes do CI.
   - Alternativa: só no CI.
5. **Mover os documentos internos para `docs/internal/` e os ADRs para `docs/adr/`.** Recomendado: sim, como no parecer; muda os caminhos que você e os agentes usam.

## Fora do escopo

- Criar o repositório público e o commit inicial: passo seguinte, com autorização específica.
- Mover código entre crates (R1).
- Comentários "Etapa N", lints e i18n (R6).
- Exemplos e documentação de arquitetura (R2).
- Mudanças de comportamento.

## Instruções para o agente

1. Ler `AGENTS.md`, este prompt, o plano R0–R6, `HANDOFF.md` e `CURRENT_STATE.md`. Git só leitura até o OK.
2. Fazer R0.0 primeiro e confirmar que passa com o código atual.
3. Plano `docs/internal/plans/<data>-r0.md` com "Andamento"; página de memória `notes/r0-preparacao-publicacao.md`.
4. Validar, gerar o relatório e parar. Não iniciar a publicação nem o R1.
