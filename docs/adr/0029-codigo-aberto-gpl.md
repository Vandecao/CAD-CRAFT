# ADR-0029 — Código aberto sob GPL-3.0-or-later e saídas de referência

Data: 2026-11-05. Estado: implementado no bloco R0. A publicação é um passo separado, com autorização do autor.

## Contexto

O autor decidiu publicar o CAD-CRAFT como código aberto. O parecer técnico `PARECER_TECNICO_CAD-CRAFT_v2.md` deu origem ao plano R0–R6, aprovado em 2026-11-05. A publicação também torna o CAD-CRAFT elegível ao login "Sign in with ChatGPT" para apps de código aberto (Etapa 27).

## Decisão

- **Licença:** `GPL-3.0-or-later`.
  - `LICENSE` com o texto oficial (SHA-256 `3972dc97…6986`, o mesmo de gnu.org).
  - Campos `license`, `repository` e `authors` no workspace.
  - Cabeçalho SPDX em todos os `.rs`, verificado por `tests/license_headers.rs`.
  - `deny.toml` com as licenças das dependências, todas compatíveis.
- **Contribuições:** DCO (`git commit -s`), Conventional Commits.
- **Idioma:** código, comentários de API e commits novos em inglês; interface em pt-BR; README bilíngue.
- **Amostras de terceiros nunca entram no repositório:**
  - os testes automáticos usam arquivos sintéticos;
  - as amostras reais ficam fora do Git e são lidas por `CADCRAFT_SAMPLES` (`docs/TESTES_COM_AMOSTRAS.md`);
  - as 19 molduras `.fn1` do EPLAN saíram do repositório, substituídas por duas sintéticas escritas à mão.
- **Saídas de referência** (`tests/baseline_outputs.rs` e `tests/fixtures/baseline/hashes.txt`):
  - a impressão digital FNV-1a de 64 bits do que o programa produz a partir das fixtures v0–v9: `.cadcraft`, JSON, lista de exibição, PDF com `/CreationDate` mascarado e DXF;
  - é a garantia de que a reorganização (R1 a R6) não muda o comportamento;
  - uma mudança intencional de saída é regravada com `CAD_CRAFT_WRITE_BASELINE=1` e justificada;
  - roda no Windows, a plataforma de referência: no Linux, funções trigonométricas podem diferir no último bit;
  - o FNV substitui o SHA-256 do prompt para não acrescentar dependência, já que o uso não é criptográfico.
- **Regras de agentes:**
  - `AGENTS.md` público, técnico e em inglês;
  - as regras do fluxo do autor ficam em `AGENTS.local.md`, fora do Git;
  - `scripts/check-repository.ps1` confere as duas estruturas.
- **Documentos:** especificações, planos, relatórios e memória em `docs/internal/`; ADRs em `docs/adr/`.
- **CI** (`.github/workflows/ci.yml`):
  - Windows com o `validate.ps1` completo;
  - Linux só com o Core, porque a GUI exige bibliotecas de janela;
  - `cargo-deny` para licenças, fontes e avisos de segurança.

## Consequências

- O histórico do repositório privado contém as molduras reais. Por isso a publicação será feita num repositório **novo**, com um commit inicial limpo; o privado fica como arquivo.
- O `repository` dos `Cargo.toml` aponta para `Vandecao/CAD-CRAFT` e deve ser corrigido se o repositório público tiver outro nome.
