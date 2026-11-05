# Plano — publicação em código aberto e reorganização (R0 a R6)

**Estado: APROVADO pelo usuário em 2026-11-05 ("Aprovo o plano"), com as decisões 1 a 4 recomendadas:**
- ordem R0 → publicação → R1 → Etapa 27 → R2…R6;
- licença `GPL-3.0-or-later`;
- repositório público novo com commit inicial limpo;
- pacote `cadcraft-gui` com executável `cad-gui.exe`.

A decisão 5 (idioma) fica para o prompt do R0. Cada bloco ainda exige o próprio prompt aprovado antes de qualquer código.

Base:
- parecer `PARECER_TECNICO_CAD-CRAFT_v2.md`, do usuário;
- conferência feita no repositório em `b06be63`.

Os blocos R ("reorganização") não mudam a numeração das etapas.

## 1. Conclusão

**Sim, dá para fazer sem quebrar o que existe.**
- A reorganização é movimentação de código, não reescrita.
- Os 669 testes, as fixtures `.cadcraft` v0–v9 e saídas de referência de PDF e DXF são a garantia de que nada muda.
- Cada bloco termina com a validação completa e só segue com o seu OK.

O parecer está correto nos fatos que conferi:

| Afirmação do parecer | Conferido |
|---|---|
| 19 molduras `.fn1` do EPLAN nos testes e no histórico (23 commits) | Sim: `tests/fixtures/eplan/`, em todos os commits |
| Nomes de empresa e caminhos locais em arquivos versionados | Sim: "<empresa>" em 5 arquivos; caminho `C:\Users\…` em 3 (2 `.fn1` e 1 relatório) |
| E-mail pessoal exposto | **Não:** os commits usam o endereço *noreply* do GitHub |
| Amostras `.qet`/`.edz` no histórico | **Não:** nunca foram versionadas |
| Licenças das dependências compatíveis com GPL-3.0 | Sim: MIT, Apache-2.0, BSD, Zlib, ISC, Unicode, CC0, BSL e as fontes OFL e Ubuntu do egui |
| Recursos binários embutidos (fontes, imagens) no repositório | Nenhum |
| `cmd /C start` ao abrir o PDF | Sim, em `pdf_export.rs:67` |
| ~187 comentários "Etapa N" | Sim, 187 |
| README desatualizado e toolchain 1.99.1 fixa em Windows | Sim |

## 2. Ajustes que proponho ao parecer

1. **Manter a crate `cad-craft` como fachada.**
   - Ela continua existindo e só reexporta as crates novas (`pub use`).
   - Assim, os mais de 60 arquivos de teste e a GUI, que usam `cad_craft::domain::*`, compilam sem mudança no R1.
   - Os caminhos internos migram aos poucos; a fachada é o "crate opcional" do parecer, mas desde o primeiro dia.
2. **Regra do Rust ("orphan rule"):** os métodos de `Project` (`impl Project`) só podem ficar na crate do `Project`.
   - `display.rs` declara `page_display_list` em `Project`. Ao ir para `cadcraft-render`, vira um *trait* de extensão com o mesmo nome; quem chama só acrescenta um `use`, que a fachada já inclui.
   - `sheet_references` fica em `cadcraft-model`, e não em `render` como diz o parecer, porque é um `impl Project` usado pelo modelo.
   - `checks` e `reports` ficam no modelo, como o parecer já propõe.
   - Medido: os itens `pub(crate)` que cruzariam crates são internos aos importadores e vão juntos. Não há outro acoplamento escondido.
3. **Saídas de referência antes de mexer (novo R0.0):** gravar hash ou cópia do PDF, do DXF e do `.cadcraft` de projetos de teste fixos. Depois de cada bloco, comparar byte a byte. Isso transforma "nada muda" em teste.
4. **Git e publicação, segundo o `AGENTS.md`:**
   - o agente não cria repositório, branch nem remoto, e não reescreve histórico;
   - por isso, "um branch por bloco" do parecer vira "um commit por bloco em `main`, com o seu OK";
   - para o histórico limpo, **você cria o repositório público novo** e me autoriza, só para esse caso, a enviar a ele um commit inicial com a árvore já limpa;
   - o repositório privado atual fica como arquivo do histórico completo;
   - não recomendo `git filter-repo` com push forçado.
5. **Variáveis das amostras:** unificar as três de hoje (`CAD_CRAFT_QET_SAMPLES`, `CAD_CRAFT_EDZ_SAMPLES` e a dos `.fn1`) numa só, `CADCRAFT_SAMPLES`, com subpastas `qet/`, `edz/` e `eplan/fn1/`.
6. **Documentos internos:**
   - specs, planos, relatórios e memória vão para `docs/internal/`;
   - antes, revisão de caminhos pessoais e nomes de empresa;
   - as regras do seu fluxo pessoal (OK antes de commit etc.) vão para `AGENTS.local.md`, fora do Git;
   - o `AGENTS.md` público fica só com as regras técnicas.
7. **Idioma (8.5) e i18n (8.6):** são decisões suas e geram muitas mudanças. Proponho decidir no R0 (registrar no `CONTRIBUTING.md`) e aplicar aos poucos no R6, não de uma vez.
8. **Toolchain:**
   - manter a versão fixa para builds reproduzíveis;
   - declarar `rust-version` (a versão mínima medida);
   - tirar a target fixa, para permitir Linux no CI.
9. **Fuzzing** (`cargo-fuzz`) exige o Rust *nightly*. Fica opcional, fora da validação obrigatória.

## 3. Ordem recomendada

| Ordem | Bloco | Conteúdo | Muda comportamento? |
|---|---|---|---|
| 1 | **R0** — preparar a publicação | Saídas de referência; fixtures `.fn1` sintéticos no lugar dos reais; `CADCRAFT_SAMPLES`; remoção de nomes e caminhos pessoais; `LICENSE` GPL-3.0-or-later, `license` nos `Cargo.toml` e SPDX nos `.rs`; README novo com a tabela de formatos e aviso de marcas; `CONTRIBUTING`, `CODE_OF_CONDUCT`, `SECURITY`; CI (Windows + Linux); `deny.toml`; `AGENTS.md` público e local; `docs/internal/` | Não |
| 2 | **Publicação** | Você cria o repositório público; eu preparo a árvore limpa e envio o commit inicial com a sua autorização. A partir daí o CAD-CRAFT é código aberto, e o login do ChatGPT passa a valer para ele (ver seção 5). | — |
| 3 | **R1** — crates | `cadcraft-model`, `-format`, `-render`, `-import`, `-export` e a fachada `cad-craft`; testes distribuídos; leitura e gravação de `.cadsym`, `.cadmacro`, `.cadpart` e `.cadframe` saem da GUI. Movimentos com `git mv`. | Não |
| 4 | **Etapa 27** — assistente | O prompt aprovado, em `cadcraft-assistant` (crate própria, como sugere o parecer), com login ChatGPT real | Sim (etapa nova) |
| 5 | **R2** — documentação de entrada | `ARQUITETURA`, `TOUR`, `GLOSSARIO`, `FORMATOS`, `examples/01–06` compilados no CI, `//!` em todos os módulos | Não |
| 6 | **R3** — modelo por assunto | Pastas `electrical/`, `physical/`, `document/`…; `impl Project` agrupados; serviços como funções; `tolerance.rs` e revisão das 69 comparações de `f64` | Não (a revisão de `f64` só onde houver teste) |
| 7 | **R4** — importadores e exportadores | `read` → `convert` → `ImportReport` por formato (começando pelo `qet`); DXF dividido entre importar e exportar; `pdf.rs` e `display.rs` divididos | Não |
| 8 | **R5** — GUI | Arquivos acima de 1.000 linhas divididos; `CadApp` com sub-estruturas; editores em `state`/`input`/`draw`; menos módulos `pub`; erros tipados; `opener` no lugar de `cmd /C start` | Correção pontual (`opener`) |
| 9 | **R6** — limpeza | Fim dos comentários "Etapa N", regra de idioma, lints do parecer zerados, textos da interface centralizados | Não |

Por que essa ordem:
- **R0 primeiro:** é pequeno e é o que permite publicar, e a publicação destrava o login real da Etapa 27.
- **R1 antes da Etapa 27:** evita mover o código do assistente duas vezes.
- **R2 a R6:** podem vir depois, inclusive junto da Etapa 28.

## 4. Regras de todos os blocos

- **Antes:** prompt do bloco em `docs/internal/spec/`, plano em `docs/internal/plans/` e o seu OK.
- **Durante:**
  - só movimentação e ajustes de `use`, nos blocos sem mudança de comportamento;
  - `git mv` para preservar o histórico dos arquivos;
  - reexportações temporárias, removidas ao fim do bloco.
- **Depois:**
  - `scripts/validate.ps1`, com contagem de testes igual ou maior;
  - fixtures v0–v9 abrem;
  - saídas de referência iguais byte a byte;
  - Release compilado;
  - relatório e memória atualizados;
  - commit + push só com o OK.
- **Teste no app:** a cada 3 blocos, ou quando o bloco tocar a GUI (R1, R5), com a sua autorização para usar o computador.

## 5. Login do ChatGPT para você testar (Etapa 27)

- **Hoje, com o repositório privado e sem licença:** o CAD-CRAFT não se enquadra. Pela OpenAI, o uso do plano por OAuth vale para "all open-source partners and selected private clients". Seu formulário está na fila (resposta prevista para o início do 4º trimestre).
- **Assim que o CAD-CRAFT for publicado com GPL-3.0 (passo 2 acima):** ele passa a ser um cliente de código aberto, que se registra sozinho pelo fluxo oficial ("Register an open-source client"). Aí você entra com a sua conta ChatGPT e testa.
- **Se a OpenAI aprovar o formulário antes:** o login vale mesmo sem publicar.
- **Em qualquer caso**, a implementação do assistente pode começar antes. O provedor real fica desligado até a elegibilidade estar confirmada, e os testes usam um provedor simulado.

## 6. Decisões pedidas ao usuário

1. **Ordem:** R0 → publicação → R1 → Etapa 27 → R2…R6 (recomendado), ou outra.
2. **Licença:** `GPL-3.0-or-later` (recomendado pelo parecer) ou `GPL-3.0-only`.
3. **Publicação:** repositório público novo com commit inicial limpo, mantendo o privado como arquivo (recomendado), ou reescrever o histórico do atual.
4. **Nome do pacote da GUI no R1:** renomear `cad-gui` para `cadcraft-gui` mantendo o executável `cad-gui.exe` (recomendado), ou renomear também o executável.
5. **Idioma do código e dos commits:** inglês (parecer) ou português (hoje). Decidir no R0 e aplicar aos poucos no R6.
