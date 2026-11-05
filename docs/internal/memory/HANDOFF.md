# R3 — modelo por assunto validado; publicação autorizada (2026-11-05)

Especificação: docs/internal/spec/R3_MODELO_POR_ASSUNTO.md. Plano: docs/internal/plans/2026-11-05-r3.md. Relatórios: docs/internal/reports/R3_MODELO_POR_ASSUNTO.md e R3_COMPARACOES_F64.md.
62 arquivos movidos por git mv, entidades e Project agrupados por assunto; frame/symbol/macros divididos. Maior arquivo do modelo: 781 linhas. API plana, Serde, dependências e formato 9 preservados. EPS_MM centraliza o mesmo 1e-6 de molduras; comparações exatas de identidade/topologia mantidas.
Validação 2026-11-05: scripts/validate.ps1 exit 0; **694 testes Rust + 6 exemplos executados + 16 PowerShell = 716**, zero falhas/ignorados. fmt, check, Clippy -D warnings e diff checks aprovados; baselines e fixtures v0–v9 preservados. Release compilado (23609856 bytes), sem executar o app.
Revisão independente: nenhum achado material. Não houve teste no app/homologação manual; nenhum R4 iniciado.
Autor autorizou commit/push de R2+R3 no privado e sincronização do público por cópia de arquivos, sem enviar o histórico privado. Privado: C:/Dev/CAD-CRAFT, main, origin Vandecao/CAD-CRAFT1, HEAD pré-versionamento observado 06a6cbf1e3d37d1e7679ca55d98f41f43e7ee001 em 2026-11-05. Público: C:/Dev/CAD-CRAFT-public, main, origin Vandecao/CAD-CRAFT; HEAD inicial 79b1a48d6984d22440e31873eedfd1083f558ac8.
Este registro é anterior ao versionamento da entrega, sem SHA futuro inventado. Resultado/SHAs reais: consultar .local/r3/PUBLICATION.md e git log nos dois checkouts. Regras e memória anteriores que diziam não iniciar R3 foram superadas pela autorização expressa desta sessão. Backup externo não criado/verificado.
Próxima ação após publicação: revisão do autor; R4 somente em nova solicitação.

---

# R2 — documentação de entrada validada; pronta para revisão (2026-11-05)

Escopo aprovado: docs/internal/spec/R2_DOCUMENTACAO_ENTRADA.md, decisões recomendadas.
Plano: docs/internal/plans/2026-11-05-r2.md. Relatório: docs/internal/reports/R2_DOCUMENTACAO_ENTRADA.md.
Quatro guias públicos em inglês (glossário bilíngue), seis exemplos executáveis na fachada,
75 introduções novas e três atualizadas; teste de presença de //! em todos os 158 arquivos de src.
Comportamento, dependências e formato 9 preservados. Exemplos e testes focados passaram.
Validação 2026-11-05: pwsh -NoProfile -File scripts/validate.ps1 terminou com exit 0: **692 testes Rust do workspace + 6 exemplos executados + 16 PowerShell = 714**, zero falhas/ignorados. fmt, check de todos os targets, Clippy com -D warnings e diff checks passaram; saídas de referência idênticas.
Sem teste no app neste bloco; nenhuma homologação manual declarada.
Git: main, HEAD 06a6cbf1e3d37d1e7679ca55d98f41f43e7ee001, consultado em 2026-11-05 (America/Sao_Paulo).
R1 e Etapa 27 já estão nesse HEAD; entradas históricas abaixo com esses blocos pendentes estão superadas.
R2 fora do HEAD. Nenhum staging, commit, push, sincronização ou backup executado.
Próximo: revisão do autor; versionamento/sincronização só com OK expresso. Não iniciar R3.

---

# Handoff — Etapa 27 (assistente de IA) testada no app; pronta para versionar (2026-11-05)

Prompt `docs/internal/spec/ETAPA_27_ASSISTENTE_IA.md`, plano `docs/internal/plans/2026-11-05-etapa-27.md`, relatório `docs/internal/reports/ETAPA_27_ASSISTENTE_IA.md`, ADR-0030.

- Crate `cadcraft-assistant` (sem rede) e painel "✦ Assistente" na GUI (OAuth ChatGPT, Responses API, propostas reversíveis).
- Teste no app com o login do autor: ok; 2 defeitos corrigidos (proposta perdida, negrito com asteriscos). CPU alta parado é anterior (GUI-L01).
- Validação: 689 Rust + 16 PowerShell, exit 0; Release 23.617.536 bytes.
- O R1 já está versionado (`cdca6fc`); a Etapa 27 está fora do HEAD.

Próximo:
1. Teste no app: feito.
2. OK para commit + push no privado (`CAD-CRAFT1`) e sincronização com o público.
3. R2–R6 e Etapa 28: prompts aprovados em 2026-11-05 com as decisões recomendadas (`docs/internal/spec/R2_…R6_…`, `ETAPA_28_FECHAMENTO_HOMOLOGACAO.md`). Implementação pelo Codex, um bloco por vez; teste no app pelo Claude Code ao fim do R5, do R6 e da Etapa 28.

---

# Handoff — Bloco R1 (crates) validado e testado no app; pronto para versionar (2026-11-05)

Prompt `docs/internal/spec/R1_DIVISAO_CRATES.md` (aprovado), plano `docs/internal/plans/2026-11-05-r1.md`, relatório `docs/internal/reports/R1_DIVISAO_CRATES.md`.

- **Core dividido em crates:** `cadcraft-model`, `-format`, `-render`, `-import` e `-export`.
- **Fachada `cad-craft`:** mantém os caminhos de antes.
- **GUI:** passa a `crates/cadcraft-gui` (pacote `cadcraft-gui`, executável `cad-gui.exe`). Compilar com `-p cadcraft-gui`.
- **Validação:** 674 Rust + 16 PowerShell; saídas de referência idênticas.
- **Disco:** `cargo clean` foi necessário (`target` com 112 GB). Vale repetir periodicamente.

Próximo:
1. Teste rápido no app: feito e ok (importação .qet e PDF de 50 páginas).
2. OK para commit + push no privado (`CAD-CRAFT1`).
3. Sincronização com o público (`C:\Dev\CAD-CRAFT-public`), com OK.
4. Etapa 27.

---

# Handoff — CAD-CRAFT publicado como código aberto (2026-11-05)

- **Público:** https://github.com/Vandecao/CAD-CRAFT (GPL-3.0-or-later).
  - Histórico novo: `eb9f805` "Initial public release" e `ea24a73` (correção do `deny.toml`).
  - Publicado a partir da pasta separada `C:\Dev\CAD-CRAFT-public` (cópia limpa, com o próprio `.git`).
- **Privado, com o histórico completo:** renomeado para `Vandecao/CAD-CRAFT1`.
  - O `origin` local aponta para ele: R0 em `7b444a4`, correção do `deny.toml` em `3be39ab`.
  - **Nunca enviar o `main` local para o público:** os commits antigos têm molduras do EPLAN.
- **Primeiro CI (privado):** Windows completo, Core no Linux e licenças passaram. Só os avisos de "sem manutenção" em dependências indiretas do egui falharam, e já foram corrigidos no `deny.toml`. O CI do público estava rodando na hora do registro.
- **Sincronizar os dois repositórios:**
  - publicar no público as mudanças versionadas no privado é um passo por bloco ou etapa, com o OK do autor;
  - copiar os arquivos para `C:\Dev\CAD-CRAFT-public`;
  - fazer o commit com `-s`, em inglês;
  - enviar o push para `Vandecao/CAD-CRAFT`.
- **Próximo:**
  - R1 (crates), com prompt próprio;
  - Etapa 27: o CAD-CRAFT agora é código aberto, então o login "Sign in with ChatGPT" para apps abertos passa a se aplicar. Confirmar na implementação.

---

# Handoff — Bloco R0 (preparação para código aberto) pronto para versionar (2026-11-05)

Prompt `docs/internal/spec/R0_PREPARACAO_PUBLICACAO.md` (aprovado), plano `docs/internal/plans/2026-11-05-r0.md`, ADR-0029, relatório `docs/internal/reports/R0_PREPARACAO_PUBLICACAO.md`. Validação: `scripts/validate.ps1` (2026-11-05) terminou com exit 0: **673 Rust + 16 PowerShell**, zero falhas; saídas de referência idênticas; Release de 20.088.832 bytes (mesmo tamanho de antes do bloco), não executado.

**Atenção na retomada:** as regras do fluxo do autor (aprovações, política Git, encerramento) agora ficam em `AGENTS.local.md`, fora do Git. O `AGENTS.md` versionado é público e técnico. Os documentos mudaram de lugar:
- memória: `docs/internal/memory/`;
- especificações, planos e relatórios: `docs/internal/{spec,plans,reports}/`;
- ADRs: `docs/adr/`.

Amostras reais: `CADCRAFT_SAMPLES` (`docs/TESTES_COM_AMOSTRAS.md`). No computador do autor, a pasta é `<pasta de referências>\samples`, com junções para as pastas existentes e as 19 molduras `.fn1` copiadas.

Próximo:
1. OK do autor para commit + push do R0 em `main`.
2. Publicação: o autor cria o repositório público e autoriza o commit inicial limpo.
3. R1 (crates), com prompt próprio.
4. Etapa 27 (assistente): prompt aprovado, aguardando a publicação ou a aprovação da OpenAI para o login real.

---

# Handoff — teste no app das Etapas 23–26 feito; pronto para versionar (2026-11-05)

Teste no executável Release feito pelo Claude Code, com autorização do usuário. Relatório: `docs/internal/reports/TESTE_APP_ETAPAS_23_26.md`.
- **Roteiro:** importação `.qet` (industrial, 50 folhas), Verificações, relatório Lista de dispositivos, importação `.edz` (4 pacotes, conflitos), inserção de macro com Ctrl+Z, biblioteca de peças e recuperação.
- **Cinco defeitos corrigidos:**
  1. rótulos e textos das ocorrências `.qet`;
  2. setas de relatório de folha soltas e com "-K?";
  3. macros `.edz` não relidas: `serde_json` com `float_roundtrip` (recurso de uma dependência já aprovada);
  4. cópia de recuperação regravada depois de Sair descartando;
  5. tipo de função em formato interno no editor de peças.
- Cada correção tem teste de regressão, quando aplicável.

Validação 2026-11-05 (`scripts/validate.ps1`, exit 0): **669 Rust + 15 PowerShell**, zero falhas. Release de 20.088.832 bytes. Amostras: `.qet` 27/27; `.edz` com 45 pacotes convertidos e relidos, 1 recusado (EDZ-L08).

Git: `main`, HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-05. As Etapas 23–26, o teste e esta documentação estão fora do HEAD.

Próximo: com o OK expresso do usuário, um commit + `git push origin main` (sem força) com as Etapas 23 a 26. Incluir:
- código, testes e fixtures;
- `Cargo.toml` e `Cargo.lock`;
- specs, planos, relatórios, ADRs 0025–0028 e memória.

Nunca incluir o `docs.rar` nem as amostras `.qet`/`.edz`. Não iniciar a Etapa 27.

---

# Handoff da Etapa 26 — implementada e validada automaticamente (2026-11-04)

Escopo autorizado: `docs/internal/spec/ETAPA_26_EPLAN_EDZ.md`, decisões 2–5 aprovadas e ajuste ESymLib aprovado na execução. Plano `docs/internal/plans/2026-11-04-etapa-26.md`, ADR-0028, relatórios `docs/internal/reports/ETAPA_26_EPLAN_EDZ.md` e `ETAPA_26_COBERTURA_EDZ.md`.

Core EDZ ZIP/7z em memória → peças e macros inline/fallback. GUI Dados-mestre > Importar .edz, lote/pasta, workers, prévia, conflitos explícitos, confirmação em bibliotecas e relatório. Formato cadcraft continua 9.

ESymLib com definições depende de amostra demonstrativa, conforme ajuste aprovado. Ligações PXF, múltiplas portas sem posições conhecidas, variantes adicionais e estilos sem mapeamento são relatados. A pasta contém 46 arquivos; RIT.2383210.edz é recusado por referência necessária ausente. Ver EDZ-L01–L08.

Validação final `scripts/validate.ps1`, exit 0: **666 Rust + 15 PowerShell**, zero falhas/ignorados; fmt, check, Clippy e diff checks passaram. Release compilado, 19.963.392 bytes, sem executar. Cobertura opcional: 45 convertidos e 1 recusado esperado, sem outros erros/pânicos. Relatório registra tentativas anteriores e limitações. Etapa pronta para revisão técnica; nenhum app executado ou homologação manual declarada.

Git: branch `main`, HEAD observado em 2026-11-04 `0b3b65c1f20951ffef5990ceb38249ba19387c07`. Consulta `git ls-remote origin refs/heads/main` confirmou o mesmo SHA remoto. Etapas 23–26 e esta documentação ainda fora do HEAD. Nenhum Git mutável, backup, staging, commit ou push. Amostras e docs.rar permanecem fora da entrega para versionamento.

Próxima ação: teste no app das Etapas 23–26 pelo Claude Code com autorização; versionamento somente com OK posterior. Não iniciar outra etapa.

---
# Handoff — Etapa 25 implementada; próximo: Etapa 26 pelo Codex (2026-11-04)

Plano `docs/internal/plans/2026-11-04-etapa-25.md` (8/8); ADR-0027; relatórios `docs/internal/reports/ETAPA_25_IMPORTACAO_QET.md` e `ETAPA_25_COBERTURA_QET.md`. Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): **640 Rust + 15 PowerShell**; Release de 19.097.600 bytes, não executado.

Decisão do usuário (2026-11-04): não houve teste no app ao fim da 25. A ordem agora é:
1. O **Codex implementa a Etapa 26**, pelo prompt revisado `docs/internal/spec/ETAPA_26_EPLAN_EDZ.md`, que precisa da aprovação das decisões 2 a 5 registrada no topo. Ele trabalha sobre o working tree com as Etapas 23 a 25 não versionadas e sem nenhum Git mutável.
2. O **Claude Code faz o teste no app das Etapas 23 a 26**, com autorização para usar o computador.
3. Se o teste passar, vem o **commit + push** das Etapas 23 a 26, com o OK do usuário. Nunca incluir o `docs.rar` nem as amostras `.qet`/`.edz`.

---

# Handoff — Etapa 24 pronta para versionar (2026-11-04)

Plano `docs/internal/plans/2026-11-04-etapa-24.md` (8/8); ADR-0026; relatório `docs/internal/reports/ETAPA_24_VERIFICACOES.md`. Validação: 630 Rust + 15 PowerShell; Release ok.

Próximo: OK do usuário para commit + push das **Etapas 23 e 24** juntas, porque compartilham arquivos.
- Incluir os prompts e os planos das duas.
- **Não** incluir `docs/internal/spec/ETAPA_26_EPLAN_EDZ.md` nem o `docs.rar`.

Depois, a Etapa 25 (importação `.qet`), que precisa de prompt aprovado. Ao fim dela vem o teste no app das Etapas 23–25, com autorização.

---

# Handoff — Etapa 23 pronta para versionar (2026-11-04)

Implementada pelo Codex; validação final e documentação pelo Claude Code. Plano `docs/internal/plans/2026-11-04-etapa-23.md` (7/7); ADR-0025; relatório `docs/internal/reports/ETAPA_23_RELATORIOS.md`. Validação: 618 Rust + 15 PowerShell; Release ok.

Próximo: OK do usuário para commit + push da Etapa 23.
- Incluir o prompt da 23 e o plano.
- **Não** incluir `docs/internal/spec/ETAPA_26_EPLAN_EDZ.md` (rascunho) nem o `docs.rar`.

Depois, a Etapa 24. Não iniciá-la sem prompt aprovado.

---

# Handoff — Etapa 22 pronta para versionar (2026-11-04)

Prompt aprovado `docs/internal/spec/ETAPA_22_DXF_DWG.md`; plano `docs/internal/plans/2026-11-04-etapa-22.md` (9/9); ADR-0024; relatório `docs/internal/reports/ETAPA_22_DXF_DWG.md`.

Arquivos:
- Core: `dxf_export.rs`, `display.rs` (`DisplayLayer`);
- GUI: `dwg.rs`, `pdf_export.rs` (modos DXF/DWG), `preferences.rs` (`dwg_converter`);
- testes: `dxf_export.rs` (6), `cad_export_ui.rs` (4), `output_common/`.

Validação: 597 Rust + 15 PowerShell; Release ok. Teste no app das Etapas 20–22 feito; 3 defeitos corrigidos (`display::wire_number_placement`, `theme::install_symbol_font`).

Próximo: aguardar o OK para commit + push da Etapa 22.

Não iniciar a Etapa 23.

---

# Handoff — Etapa 21 pronta para revisão (2026-11-04)

Prompt aprovado `docs/internal/spec/ETAPA_21_PDF_IMPRESSAO.md`; plano `docs/internal/plans/2026-11-04-etapa-21.md` (9/9); ADR-0023; relatório `docs/internal/reports/ETAPA_21_PDF_IMPRESSAO.md`.

Arquivos:
- Core: `display.rs`, `font.rs`, `pdf.rs`, `domain/project/sheet_references.rs`;
- GUI: `pdf_export.rs`, `frame_view.rs`, `project_file.rs` (`save_pdf`);
- testes: `cad-craft/tests/pdf_export.rs` (6) e `cad-gui/tests/pdf_ui.rs` (2).

Validação: 586 Rust + 15 PowerShell; Release ok, não executado.

Próximo: o usuário pediu commit + push das Etapas 20 e 21 depois desta. Aguardar o OK expresso e nunca incluir `docs.rar`. Não iniciar a Etapa 22. O próximo teste no app é ao fim da 22, com autorização.

---

# Handoff — Etapa 20 pronta para revisão (2026-11-04)

Prompt aprovado `docs/internal/spec/ETAPA_20_NUMERACAO_REFERENCIAS.md`; plano `docs/internal/plans/2026-11-04-etapa-20.md` (9/9, com a seção "Retomada em 2026-11-04"); ADR-0022; relatório `docs/internal/reports/ETAPA_20_NUMERACAO_REFERENCIAS.md`.

Arquivos principais:
- Core: `domain/numbering.rs`, `project/numbering.rs`;
- GUI: `numbering_ui.rs`, `history.rs`, `connection_properties.rs`, `device_properties.rs`, `frame_view.rs`, `viewport.rs`;
- testes: `cad-craft/tests/numbering.rs` (12) e `cad-gui/tests/numbering_ui.rs` (6).

Validação: 577 Rust + 15 PowerShell. Release ok, não executado.

Próximo: OK do usuário para commit + push da Etapa 20 (incluir spec e plano; nunca `docs.rar`). Não iniciar a Etapa 21. O próximo teste no app é ao fim da Etapa 22.

---

# Handoff — Etapa 19 implementada; teste no app feito (2026-11-04)

Prompt aprovado `docs/internal/spec/ETAPA_19_MACROS.md`; plano `docs/internal/plans/2026-11-04-etapa-19.md` (10/12); ADR-0021; relatório `docs/internal/reports/ETAPA_19_MACROS.md`. Validação: 556 Rust + 15 PowerShell; Release ok. Teste no app feito (ver relatório); corrigida a prévia sob o cursor. Próximo: confirmar com o usuário e fazer commit + push das Etapas 18 e 19. Não iniciar a Etapa 20.

---

# Handoff — Etapa 18 pronta para revisão (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_18_DISPOSITIVOS_PECAS_FUNCOES.md`, quatro decisões aprovadas, plano `docs/internal/plans/2026-11-03-etapa-18.md` (10/10), ADR-0020. Etapa 17 commitada e enviada em `0886379`. Código da Etapa 18 inclui Core, GUI, CSV, `.cadpart`, formato 6, testes e fixture. Validação oficial passou: 539 Rust + 15 PowerShell, `fmt`, `check`, Clippy e diff check; build Release passou sem executar o app. Relatório em `docs/internal/reports/ETAPA_18_DISPOSITIVOS_PECAS_FUNCOES.md`. `main` em `0886379`; Etapa 18 fora do HEAD, `docs.rar` é do usuário. Aguardar novo OK para commit + push da Etapa 18. Não iniciar a 19.

---

# Handoff — Etapa 17 pronta para revisão (2026-11-03)

Prompt aprovado: `docs/internal/spec/ETAPA_17_BIBLIOTECA_EDITOR_SIMBOLOS.md`. Plano
(Andamento 11/11): `docs/internal/plans/2026-11-03-etapa-17.md`. Relatório:
`docs/internal/reports/ETAPA_17_BIBLIOTECA_EDITOR_SIMBOLOS.md`. ADR-0019. ai-memory:
`notes/etapa-17-biblioteca-editor-simbolos.md`.

- Core: `domain/symbol.rs` (Ellipse, DynamicText, PrimitiveStyle, styles),
  `project/symbol_texts.rs`, `project/symbols.rs` (update_symbol_definition),
  `persistence` formato 5 + `.cadsym`, `interchange/elmt.rs`; fixture v5.
- GUI: `symbol_editor.rs`, `symbol_files.rs`, `symbol_view.rs` (estilos),
  catálogo no Centro de inserção, comandos Dados-mestre > Símbolos.
- Validação: 526 Rust + 15 PowerShell; Release ok, não executado.
- Prompt da Etapa 18 (rascunho para o Codex, aguardando aprovação):
  `docs/internal/spec/ETAPA_18_DISPOSITIVOS_PECAS_FUNCOES.md`; ai-memory
  `notes/etapa-18-dispositivos-pecas-funcoes.md`.
- Pendente: OK do usuário para commit + push. Próximo teste no app ao fim
  da Etapa 19 (com autorização). Não iniciar a Etapa 18 sem prompt aprovado.

---

# Handoff — Etapa 16 pronta para revisão (2026-11-03)

Supera as entradas "Etapa 16 em preparação" abaixo (código já existe).
Prompt revisado: `docs/internal/spec/ETAPA_16_CONTEUDO_LIVRE_CLIPBOARD.md`. ADR-0018.
Relatório: `docs/internal/reports/ETAPA_16_CONTEUDO_LIVRE_CLIPBOARD.md`.

- Core: `domain/page_graphic.rs`, `project/clipboard.rs` (`ClipItem`,
  `cut_representations`, `paste_items`, `SheetEditDelta`), formato 4,
  fixture `etapa16-v4.cadcraft`.
- GUI: `state.rs` (área de transferência, `duplicate_selection`),
  `history.rs` (`Transaction::Sheet`), comandos Colar no mesmo dispositivo e
  Duplicar, diálogo Editar texto; testes `free_*.rs`.
- Validação: 512 Rust + 15 PowerShell; Release ok, não executado.
- Teste no app (14–16) feito em 2026-11-03; defeito Ctrl+C/X/V corrigido
  (513 Rust). Pendente: commit + push das Etapas 15 e 16 só após OK expresso.

---

# Handoff — Etapa 16 em preparação (2026-11-03)

O usuário autorizou iniciar a Etapa 16. O escopo geral está no Plano Mestre;
o prompt detalhado `docs/internal/spec/ETAPA_16_CONTEUDO_LIVRE_CLIPBOARD.md` e o plano
preliminar `docs/internal/plans/2026-11-03-etapa-16-rascunho.md` foram escritos para
revisão. A aprovação expressa do prompt ainda é necessária antes de alterar
código. Nenhum código da Etapa 16 foi iniciado. A Etapa 15 segue completa,
validada e não versionada por instrução do usuário. Não fazer commit/push.

---

# Handoff — Etapa 15 pronta para revisão (2026-11-03)

Prompt aprovado: `docs/internal/spec/ETAPA_15_MOLDURAS_OUTROS_SISTEMAS.md`.
Plano: `docs/internal/plans/2026-11-03-etapa-15.md`. Relatório:
`docs/internal/reports/ETAPA_15_MOLDURAS_OUTROS_SISTEMAS.md`. ADR-0017.
Core `.fn1`, `.titleblock` e DXF; GUI com prévia/relatório e exportação DXF;
importação reversível. Validação oficial: 495 Rust + 15 PowerShell, sem falhas.
Release e inventário final constam no relatório. Não houve commit/push por
instrução expressa do usuário naquela retomada. A preparação da Etapa 16 foi
autorizada posteriormente; o código ainda aguarda aprovação do prompt.

---

# Handoff — Etapa 14 pronta para revisão (2026-11-03)

Prompt: docs/internal/spec/ETAPA_14_EDITOR_DE_MOLDURAS.md (aprovado; `image`
autorizada; QR/hiperlink adiados). Plano: docs/internal/plans/2026-11-03-etapa-14.md.
Relatório: docs/internal/reports/ETAPA_14_EDITOR_DE_MOLDURAS.md. Decisão: ADR-0016.

- Core: `domain/asset.rs`, `FrameElement::Image`, `watermark`, `FrameDraft`,
  `FrameEdit::Batch`, `FramePackage`/`frame_import_edits`; `persistence`
  formato 3 + `.cadframe` (fixture `tests/fixtures/etapa14-v3.cadcraft`;
  v0/v1/v2 intactas; `tests/frames_common` gera PNG de teste).
- GUI: `frame_editor.rs` (modo editor; testes em `tests/frame_editor_ui.rs`
  localizam ferramentas na barra do editor e campos à direita da tela do
  editor), texturas em cache em `frame_view.rs` (só cacheia imagem com bytes).
- Validação oficial: 480 Rust + 15 PowerShell; Release ok.
- Teste no app a cada 3 etapas (decisão do usuário; próximo ao fim da
  Etapa 16). Commit + push autorizados ("Pode versionar"). Próxima: Etapa 15 (DXF,
  `.titleblock`, `.fn1`), sem iniciar.

---

# Handoff — Etapa 13 pronta para revisão (2026-11-03)

Prompt: docs/internal/spec/ETAPA_13_MOLDURAS_ZONAS_REFERENCIAS.md (aprovado). Plano:
docs/internal/plans/2026-11-03-etapa-13.md. Relatório:
docs/internal/reports/ETAPA_13_MOLDURAS_ZONAS_REFERENCIAS.md. Decisão: ADR-0015.

- Core: `domain/frame.rs` (modelo, zonas, fábrica A3/A4, `PaperFormat`),
  `project/frames.rs` (`FrameEdit`, `reference_label`, `resolve_special`),
  `Page.frame`, `DeviceReference.reference`; `.cadcraft` formato 2
  (fixture `tests/fixtures/etapa13-v2.cadcraft`; v0/v1 intactas).
- GUI: `frame_view.rs`, Propriedades (seletores com largura fixa 100 pt —
  outras larguras alteram a área de desenho e quebram testes de inserção),
  diálogo Nova página com formato, molduras de fábrica em projeto novo.
- Validação oficial: 466 Rust + 15 PowerShell; Release ok.
- Validação funcional na interface pelo agente; textos de "Novo projeto" e
  "Tipo de sessão" corrigidos. Commit + push da Etapa 13 autorizados pelo
  usuário ("Corrigir e versionar"); SHA em `git log`.
- Próximas: Etapa 14 (editor de molduras, imagens) e 15 (DXF, `.titleblock`,
  `.fn1`; amostras em `<pasta de molduras do EPLAN>`).

---

# Handoff — Plano Mestre aprovado; Etapa 13 em rascunho (2026-11-03)

- Etapa 12 versionada pelo agente com autorização: commit `6612e82` em
  `origin/main` (levou também a Etapa 11). Teste manual da 12 informado como
  aprovado pelo usuário.
- Plano Mestre aprovado: Etapas 13–28 (`docs/internal/plans/PLANO_MESTRE_2026-11-03.md`,
  ROADMAP). Meta: ECAD utilizável + EPLAN 2027 + IA. Homologação geral só na
  Etapa 28. Antiga "Etapa 17" (IA) é a Etapa 27.
- Git: AGENTS.md e ADR-0002 (adendo) — agente faz commit + push de cada etapa
  somente após OK expresso do usuário para aquela etapa.
- Etapa 13 (molduras, zonas, referências por coluna): rascunho em
  `docs/internal/spec/ETAPA_13_MOLDURAS_ZONAS_REFERENCIAS.md`, **aguardando aprovação**.
- `.fn1` do EPLAN é XML (EplanPxfRoot, PxfVersion 1.23) com propriedades
  codificadas; amostra recebida vazia. Pedir molduras com conteúdo para a 15.
- Elementos do QElectroTech: não usar para construir modelos de IA; o
  assistente (Etapa 27) apenas ajuda o usuário a operar o programa.

---

# Handoff — Etapa 12 (M0.1) pronta para revisão (2026-11-03)

Prompt: docs/internal/spec/ETAPA_12_PROJETO_REAL_PERSISTENCIA.md (auditoria CAD-CRAFT ×
QElectroTech, seção 7, aprovada; `zip` + `rfd` autorizados). Plano:
docs/internal/plans/2026-11-03-etapa-12.md. Relatório:
docs/internal/reports/ETAPA_12_PROJETO_REAL_PERSISTENCIA.md. Decisão: ADR-0014.

- Core: `persistence.rs` (formato v1/v0, limites, migração),
  `project/placement.rs` (`place_symbol`, `restore_placement`,
  `next_device_designation`), `project/document_edit.rs`.
- GUI: `project_file.rs` (trava, recuperação, recentes, gravação atômica,
  `FilePicker`), comandos Abrir/Salvar/Salvar como, Backstage Abrir,
  diálogos de trava/recuperação, histórico `Placement`/`Document`, centro de
  inserção com "Novo dispositivo".
- Fixtures versionadas em `crates/cad-craft/tests/fixtures/` (regerar só com
  `CAD_CRAFT_WRITE_FIXTURES=1`).
- Validação oficial: 452 Rust + 15 PowerShell, 0 falhas; Release ok.
- HEAD `34785ab`, `main`; remoto em `f1b173e`. Etapa 12 fora do HEAD. Usuário
  autorizou expressamente o agente a fazer commit + push da Etapa 12 em
  2026-11-03 (exceção pontual do ADR-0002); o push levou também a Etapa 11.
  Teste manual do roteiro informado como aprovado pelo usuário.
- Plano Mestre revisado (M0 = Etapas 12–15, 16 pós-M0, 17 IA, frente
  I-EPLAN) registrado no ROADMAP a pedido do usuário em 2026-11-03, como
  planejamento — não autoriza implementação. Quantidade/numeração das
  próximas etapas em aberto: o usuário definirá um novo plano. Não iniciar
  nenhuma etapa sem prompt.

---

# Handoff — Etapa 11 pronta para revisão (2026-11-02)

Prompt: docs/internal/spec/ETAPA_11_CONECTORES_BORNES_ESPECIAIS.md. Plano:
docs/internal/plans/2026-11-02-etapa-11.md. Relatório:
docs/internal/reports/ETAPA_11_CONECTORES_BORNES_ESPECIAIS.md. Decisão: ADR-0013.

- Core: `Connector`/`ConnectorMating` (domain/connector.rs,
  project/connectors.rs), especializações em `Terminal` (terminal.rs),
  `ConnectionPoint.terminal_level`, edições novas em `PhysicalEdit`
  (physical_edit.rs), regras PE/níveis (integrity.rs, physical_integrity.rs),
  índice (physical_index.rs), listas (physical_lists.rs).
- GUI: aba Conectores e editor de especialização
  (cad-gui/src/physical_editor/connectors.rs), comandos
  `ConnectorEditor`/`ConnectorNavigator`, navegador Bornes, Propriedades,
  janela limitada à tela.
- Validação oficial: 418 Rust + 15 PowerShell, 0 falhas; Release ok.
- HEAD `f1b173e9e5a9d0a485163bfd2c13a871a6b46cbd`, `main`; Etapa 11 fora do
  HEAD. Usuário faz commit manual (etapa ímpar). Nenhum Git mutável executado.
  `docs.rar` não versionado é do usuário.
- Pendentes de decisão: DEC-11-01 (continuidade por acoplamento) e DEC-11-02
  (efeito do estado seccionável). Não implementar sem decisão.
- Não iniciar a Etapa 12; não declarar homologação.

---

# Handoff — Etapa 10 pronta para revisão (2026-11-02)

Prompt: docs/internal/spec/ETAPA_10_BORNES_CABOS_CONDUTORES.md. Plano com andamento:
docs/internal/plans/2026-11-02-etapa-10.md. Relatório final:
docs/internal/reports/ETAPA_10_BORNES_CABOS_CONDUTORES.md. Decisão: ADR-0012.

- Codex fez correções de rota 09A/09B e base física; foi interrompido. Claude
  Code concluiu: Batch atômico, unicidade de numeração, listas derivadas,
  aba Condutores, listas técnicas, renumeração, localizar ocorrência,
  navegadores hierárquicos, resumo físico em Propriedades, inspetor de conexão,
  correções da revisão independente (sem P1).
- Validação: 389 Rust aprovados; fmt/check/clippy/diff check/Release ok.
  Repetir `pwsh -NoProfile -File scripts/validate.ps1` onde houver PowerShell 7
  (15 testes PowerShell não executados nesta sessão).
- HEAD `a4f8233799292741c5e93bb65aec28983f1c9d41`, `main`; Etapa 10 fora do HEAD.
  Usuário faz commit + push manuais. Nenhum Git mutável executado.
- Não iniciar Etapa 11; não declarar homologação. Pendências registradas em
  KNOWN_ISSUES (PHY-010…014).
- Homologação do usuário encontrou GUI-PHY-01 (janela Engenharia física
  crescendo sem parar nas Listas técnicas). Corrigido em `physical_editor.rs`
  (painéis internos + rolagem) com 2 regressões em `tests/physical_ui.rs`;
  391 testes Rust aprovados. Aguarda reteste manual; Release não reconstruído.
- GUI-PHY-02 (regressão: controles pintados fora da janela) corrigido sem
  `CentralPanel/TopBottomPanel::show_inside`; ver relatório. Não reintroduzir
  painéis dentro de `egui::Window` sem alocar o espaço no `Ui` da janela.

---

# Handoff — Etapa 9 pronta para revisão (2026-11-02)

Prompt em docs/internal/spec/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md. Correção
geométrica validada antes dos novos recursos: 350 testes oficiais. Core/GUI da
Etapa 9 implementados; revisão independente leu o conjunto e apontou três
lacunas P2 de interface, corrigidas no painel/navegador. Validação oficial
passou: 379 testes (364 Rust e 15 PowerShell), fmt/check/clippy/diff check;
Release aprovado. Relatório final contém inventário, limites e hash do binário.
Não iniciar Etapa 10, não declarar homologação manual e não executar Git mutável.
Em 2026-11-02, HEAD local e remoto consultado `5377559ff3b5bd5ce1cd9d19265833ef3714796e`
em `main`; toda a Etapa 9 e memória estão fora desse HEAD. Usuário faz commit
manual; backup externo independente segue recomendável. O ai-memory respondeu.

## Histórico de retomada de 2026-10-31 (superado pelo estado acima)

Em 2026-11-02 o usuário autorizou instalar/ativar ai-memory, como tarefa
ambiental separada. CLI Codex testada com captura real; reiniciar o Codex
desktop para carregar MCP, hooks e variável de autenticação. Detalhes em
`docs/internal/reports/AI_MEMORY_INSTALACAO_2026-11-02.md`. A implementação da Etapa 9
permanece em andamento; a validação geral executada nesta tarefa ambiental
falhou em fmt/check/test/clippy do código parcial da etapa, sem correções aqui.
Retomar a etapa sem pular validações. HEAD observado
nessa data: `5377559ff3b5bd5ce1cd9d19265833ef3714796e`, `main`; todas as
alterações da Etapa 9 e desta tarefa estão fora desse HEAD. Nenhum commit/push
do CAD-CRAFT executado pelo Codex.

Prompt autorizado preservado em docs/internal/spec/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md.
Plano: docs/internal/plans/2026-10-31-etapa-09.md. Relatório ainda parcial:
docs/internal/reports/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md.
HEAD observado: 5377559ff3b5bd5ce1cd9d19265833ef3714796e, main; árvore inicial limpa.
O anexo confirma commit/push manuais da Etapa 8; informações antigas abaixo
sobre a Etapa 8 fora do HEAD são históricas, superadas por esta consulta.

Prioridade atual: concluir correção geométrica e executar scripts/validate.ps1
ANTES de implementar recursos novos. RED de acúmulo de dobras reproduzido;
base persistente parcial passou 28 testes Core; RED egui de L automático ainda
motivou RouteIntent separado. Ver logs .local/etapa-09/geometry-*.log.
Não declarar correção concluída até existir validação completa registrada.
Contratos Core/GUI novos planejados no plano; ainda sem implementação.
Nenhum Git mutável, commit/push/worktree; nenhuma homologação declarada.
Retomar estado real dos arquivos/logs, sem refazer trabalho já concluído.

---
# Handoff

**Etapa 8 implementada e validada automaticamente: 339 testes aprovados.** Ver estado final e
contagem em `docs/internal/reports/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md`.
Homologação geral após Etapa 15; não solicitar nesta entrega.

## Retomada seletiva

1. Ler AGENTS, este HANDOFF e CURRENT_STATE; Git exclusivamente de leitura.
2. Consultar HEAD/status e o prompt autorizado, sem depender de chats.
3. Para conexões: plano Etapa 8, ADR-0010 e módulos/testes connection_* pertinentes.
4. Core anterior: ADRs 0003–0006; UI/editor/símbolos: ADRs 0007–0009.
5. Não iniciar a próxima etapa ou sincronizar Git automaticamente.

## Base observada

`C:\Dev\CAD-CRAFT`, main, origin `https://github.com/Vandecao/CAD-CRAFT.git`.
HEAD consultado em 2026-10-30: `6bfa4c24e3b3a39642cea1e387a3a7f502835828`,
Etapa 7 commitada pelo usuário. Árvore limpa antes de copiar o anexo/plano da
Etapa 8. Toda esta implementação e memória seguem fora do HEAD observado.

## Continuidade técnica

- PortRef = occurrence ID + identificação local de SymbolPort. Associação explícita
  a ponto funcional compatível; Unassociated não produz conexão. Inspetor permite
  associar/desassociar portas livres; pontos usados por rotas não mudam silenciosamente.
- ConnectionRoute compartilha ID com Representation target Connection. Pontos
  ortogonais finitos em mm, endpoints conferidos com portas transformadas.
- ConnectionJunction identifica união explícita de rotas incidentes numa posição.
  Usa arestas auxiliares exclusivas entre endpoints existentes, sem entidades
  físicas fictícias. Remoções recusam dependências e preservam redes consistentes.
- Project::edit_connection_drawing / restore_connection_drawing validam deltas
  compactos com rollback antes da publicação dos índices. Histórico GUI captura
  esses deltas, conserva IDs e restaura geometria/topologia em uma transação.
- points_with_endpoints é o algoritmo puro compartilhado da prévia/commit.
  Movimento em grupo ignora transform direto de rota quando ambos símbolos
  extremos estão no lote. O Core continua recusando transform arbitrário de rota.
- Interaction guarda rascunho/drag/snap/seleção de junção. ConnectionScene guarda
  candidatos por página/revisão. Rede não é recalculada por hover; destaque é
  descartado após mutação. Propriedades da conexão são derivadas por revisão.
- Connection tool: clique porta, waypoints, porta; Espaço alterna L, Esc cancela.
  Junções são operações explícitas sobre cruzamentos de rotas existentes. Seleção
  do marcador tem hit próprio, separado da prioridade elétrica de snap.
- Camada e hit concordam: na mesma layer, símbolos acima das rotas. Ctrl+arraste
  não altera rota; gestos de símbolo e de conexão não competem silenciosamente.
- Preferência junction_outline alterna ponto/anel; cruzamento sem junção não tem
  marca. Exemplo multifilar separado preserva demonstração e testes anteriores.
- Dados novos são serializados; listas/defaults preservam compatibilidade e
  duplicatas/referências inválidas são rejeitadas antes de substituir Session.
- Limites de rota→junção sem porta, automação avançada, engenharia/ERC, persistência
  final e performance permanecem explícitos no ADR/KNOWN_ISSUES/relatório.

## Evidências e encerramento

Validação oficial: 324 Rust + 15 PowerShell, zero falhas/ignorados; 67 novos.
Revisão independente final sem P1/P2 funcional aberto; achados GUI corrigidos.
Release aprovado, PE Subsystem 2; hash/tamanho no relatório. Capturas internas
1600×900/100% e 1280×720/125% inspecionadas, sem homologação manual. Ensaio
denso GUI mediu cinco frames CPU com tesselação em ~1,992 s; otimização futura
depende de projetos reais e escopo aprovado.
Logs, testes auxiliares independentes e capturas ficam em `.local/etapa-08/`;
artefatos de compilação em `target/`. São ignorados e não constituem backup.
Inventário versionável, comandos, contagem final, build e limitações no relatório.
Commit + push exclusivos do usuário; backup externo independente recomendado.
Etapas 9–15 somente propostas no roadmap, sem autorização de implementação.
