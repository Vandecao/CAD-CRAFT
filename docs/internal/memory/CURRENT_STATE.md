# R4/R5 concluídos; publicação autorizada (2026-11-06)

R4: importadores/render/export organizados; protocolo DXF compartilhado conforme ADR-0031. R5: GUI por assunto, estado agrupado, GuiError e opener conforme ADR-0032. Formato 9, rótulos e baselines preservados. Relatórios: docs/internal/reports/R4_IMPORTADORES_EXPORTADORES.md e R5_GUI.md; inventário R4_R5_ARQUIVOS.md.

Validação completa em 2026-11-05, repetida com exit 0 em 2026-11-06 antes da publicação: 706 testes Rust + 6 exemplos + 16 PowerShell, zero falhas/ignorados; Release 23.736.832 bytes compilado e executado. Revisão independente sem achados materiais. Amostras reais equivalentes a R3: 27 QET idênticos; EDZ 45 convertidos, uma recusa esperada e zero erros inesperados; FN1/ELMT/titleblock passaram. Nenhum dado real versionado.

Roteiro nativo de R5 concluído pelo Codex em 2026-11-05/06: abrir/editar/salvar projeto, quatro editores, PDF aberto no visualizador padrão (nome especial), DXF e importação QET sintética. Assistente conferido sem autenticação: contexto/provedor corretos e envio bloqueado até login; OAuth/conversa real não repetidos. Captura de tela do helper indisponível; confirmação por acessibilidade, arquivos gerados e inventário de janelas. Inspeção adicional do Edge bloqueada por verificação de URL; não houve contorno. Não declarar homologação geral da Etapa 28.

Autor autorizou commit -s e push origin main nos dois repositórios. Privado C:/Dev/CAD-CRAFT, origin Vandecao/CAD-CRAFT1; público C:/Dev/CAD-CRAFT-public, origin Vandecao/CAD-CRAFT, por cópia dos arquivos aprovados e commit independente. HEADs pré-publicação consultados em 2026-11-06: 7f5463a956aaa731c77105a4142719db76ba264a e 35d7074b7bc4a5ccae781359a02ba2cbd2d5f4d5. R4/R5 ainda fora desses HEADs na escrita deste registro. SHAs efetivos e confirmação remota serão registrados em .local/r4-r5/PUBLICATION.md e podem ser consultados em git log/ls-remote. Nenhum SHA futuro inventado.

Próximo: executar publicação autorizada após conferência final e encerrar. R6 e Etapa 28 não iniciados; prompts correspondentes preservados fora do commit. Sem backup externo criado/verificado. App deixado na sessão QET sintética; projeto anterior salvo e recuperação de outra sessão preservada.

---

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

# Estado — Etapa 27 (assistente de IA e ajuda) testada no app; pronta para versionar (2026-11-05)

- Assistente no painel lateral (aba "✦ Assistente" e botão na barra de status), só ChatGPT por OAuth; nada da conta salvo.
- Ajuda local de comandos e tarefas; consultas só de leitura; propostas validadas e aplicadas por operações reversíveis.
- Rede só em `cadcraft-gui/src/assistant/`; núcleo em `cadcraft-assistant`.
- Formato `.cadcraft` inalterado (9).

Validação 2026-11-05: `scripts/validate.ps1` exit 0, **689 Rust + 16 PowerShell**; Release de 23.617.536 bytes. Teste no app com o login do autor feito (IA-L01 resolvido).

Git: `main`, HEAD `cdca6fc` (R1). A Etapa 27 está fora do HEAD; commit + push aguardam o OK.

---

# Estado — Bloco R0 concluído; código pronto para publicação GPL-3.0 (2026-11-05)

- Licença GPL-3.0-or-later, SPDX em todos os `.rs`, `deny.toml`, README bilíngue, `CONTRIBUTING`, `CODE_OF_CONDUCT`, `SECURITY` e CI.
- Molduras EPLAN reais fora do repositório (testes com sintéticas); amostras por `CADCRAFT_SAMPLES`.
- Saídas de referência (`baseline_outputs.rs`) garantem que a reorganização não muda nada.
- `AGENTS.md` público; fluxo do autor em `AGENTS.local.md`. Documentos em `docs/internal/` e ADRs em `docs/adr/`.
- Comportamento do programa inalterado; formato `.cadcraft` 9.

Validação 2026-11-05: `scripts/validate.ps1` (2026-11-05) terminou com exit 0: **673 Rust + 16 PowerShell**, zero falhas; saídas de referência idênticas; Release de 20.088.832 bytes (mesmo tamanho de antes do bloco), não executado.

Git: `main`, HEAD `b06be631c16a8104ca7b502f26d3c517784701c0` (Etapas 23–26), observado em 2026-11-05. O R0 está fora do HEAD; commit + push aguardam o OK. A publicação vem depois, com autorização específica.

---

# Estado — Etapas 23 a 26 testadas no app; prontas para versionar (2026-11-05)

As Etapas 23 (relatórios), 24 (verificações), 25 (importação `.qet`) e 26 (importação `.edz`) estão implementadas, validadas e testadas no executável. Relatório: `docs/internal/reports/TESTE_APP_ETAPAS_23_26.md`, com cinco defeitos corrigidos.

Validação 2026-11-05 (`scripts/validate.ps1`, exit 0): 669 Rust + 15 PowerShell. Release de 20.088.832 bytes. Formato `.cadcraft`: 9. Recurso `float_roundtrip` do `serde_json` ligado no Core (ADR-0028, adendo).

Git: `main`, HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-05. Etapas 23–26 fora do HEAD; commit + push aguardam o OK do usuário. O `docs.rar` e as amostras ficam fora. Homologação geral: Etapa 28.

---

# Estado da Etapa 26 — implementada e validada automaticamente (2026-11-04)

Escopo autorizado: `docs/internal/spec/ETAPA_26_EPLAN_EDZ.md`, decisões 2–5 aprovadas e ajuste ESymLib aprovado na execução. Plano `docs/internal/plans/2026-11-04-etapa-26.md`, ADR-0028, relatórios `docs/internal/reports/ETAPA_26_EPLAN_EDZ.md` e `ETAPA_26_COBERTURA_EDZ.md`.

Core EDZ ZIP/7z em memória → peças e macros inline/fallback. GUI Dados-mestre > Importar .edz, lote/pasta, workers, prévia, conflitos explícitos, confirmação em bibliotecas e relatório. Formato cadcraft continua 9.

ESymLib com definições depende de amostra demonstrativa, conforme ajuste aprovado. Ligações PXF, múltiplas portas sem posições conhecidas, variantes adicionais e estilos sem mapeamento são relatados. A pasta contém 46 arquivos; RIT.2383210.edz é recusado por referência necessária ausente. Ver EDZ-L01–L08.

Validação final `scripts/validate.ps1`, exit 0: **666 Rust + 15 PowerShell**, zero falhas/ignorados; fmt, check, Clippy e diff checks passaram. Release compilado, 19.963.392 bytes, sem executar. Cobertura opcional: 45 convertidos e 1 recusado esperado, sem outros erros/pânicos. Relatório registra tentativas anteriores e limitações. Etapa pronta para revisão técnica; nenhum app executado ou homologação manual declarada.

Git: branch `main`, HEAD observado em 2026-11-04 `0b3b65c1f20951ffef5990ceb38249ba19387c07`. Consulta `git ls-remote origin refs/heads/main` confirmou o mesmo SHA remoto. Etapas 23–26 e esta documentação ainda fora do HEAD. Nenhum Git mutável, backup, staging, commit ou push. Amostras e docs.rar permanecem fora da entrega para versionamento.

Próxima ação: teste no app das Etapas 23–26 pelo Claude Code com autorização; versionamento somente com OK posterior. Não iniciar outra etapa.

---
# Estado da Etapa 25 — implementada e validada (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_25_IMPORTACAO_QET.md` (cinco decisões recomendadas). Documentos:
- plano `docs/internal/plans/2026-11-04-etapa-25.md` (8/8);
- ADR-0027;
- relatórios `docs/internal/reports/ETAPA_25_IMPORTACAO_QET.md` e `docs/internal/reports/ETAPA_25_COBERTURA_QET.md`.

- **Core:**
  - `interchange/qet.rs` importa `.qet` como projeto novo:
    - folhas, molduras com zonas e carimbo embutido;
    - símbolos embutidos;
    - mestre e escravos num dispositivo;
    - fios com número travado;
    - relatórios de folha como interrupções agrupadas;
    - textos e desenhos sem terminais;
  - nova edição `ConnectionDrawingEdit::CreateRouteExact`;
  - formato `.cadcraft` inalterado (9).
- **GUI:** Arquivo > Importar > Projeto QElectroTech (`qet_import_ui.rs`), com análise em segundo plano, prévia, criação do projeto e opção de abrir as Verificações.
- **Amostras:** 27 de 27 exemplos do QElectroTech importam. `industrial.qet` (50 folhas) leva de 1,6 s a 3,6 s em Release.

Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): 640 Rust + 15 PowerShell; Release de 19.097.600 bytes, não executado. Por decisão do usuário, não houve teste no app nesta etapa. O Codex implementa a Etapa 26 e, em seguida, o Claude Code faz o teste no app das Etapas 23 a 26.

Git: `main`, HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-04.
- Fora do HEAD: as Etapas 23, 24 e 25. O commit + push vem depois do teste no app, com o OK do usuário.
- O `docs.rar` e as amostras ficam fora.

---

# Estado da Etapa 24 — pronta para versionar (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_24_VERIFICACOES.md` (cinco decisões recomendadas). Plano `docs/internal/plans/2026-11-04-etapa-24.md` (8/8), ADR-0026, relatório `docs/internal/reports/ETAPA_24_VERIFICACOES.md`.

- **Core:**
  - motor de verificação com 15 regras de código estável (`domain/check.rs`, `project/checks.rs`);
  - esquema e mensagens ignoradas no projeto (`.cadcraft` formato 9);
  - DEC-11-01, proposta A: o acoplamento de conectores dá continuidade por conexões de posse;
  - DEC-11-02 documental; CON-001 como aviso.
- **GUI:**
  - painel Verificações (`checks_ui.rs`) com filtros, contadores, duplo clique, F8/Shift+F8, Ignorar com justificativa, CSV/XLSX e reavaliação automática;
  - Ferramentas > Verificação: Verificar projeto e Verificar página;
  - Configurações > Verificações do projeto.

Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): 630 Rust + 15 PowerShell. Release de 18.887.680 bytes, não executado. Verificação de 20.040 ocorrências em 0,24 s no Release. Próximo teste no app: fim da Etapa 25.

Git: `main`, HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-04.
- Fora do HEAD: as Etapas 23 e 24, que compartilham arquivos (persistência, histórico, app) e por isso vão num commit só, depois do OK.
- O rascunho da Etapa 26 e o `docs.rar` ficam fora.

---

# Estado da Etapa 23 — pronta para versionar (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_23_RELATORIOS.md` (cinco decisões recomendadas). Plano `docs/internal/plans/2026-11-04-etapa-23.md` (7/7), ADR-0025, relatório `docs/internal/reports/ETAPA_23_RELATORIOS.md`. Implementada pelo Codex; validação final e documentação pelo Claude Code.

- **Core:** seis relatórios derivados do cadastro (`domain/report.rs`, `project/reports.rs`): índice de páginas, dispositivos, materiais, conexões, bornes e cabos. Também:
  - paginação na área útil da moldura;
  - tabela na lista de exibição, que sai no PDF e no DXF;
  - CSV com `;` e BOM, e XLSX (`report_export.rs`, `rust_xlsxwriter` 0.102.0);
  - `.cadcraft` formato 8.
- **GUI:** Ferramentas > Relatórios > Gerar/Atualizar, aviso de desatualizado, Exportar lista (`reports_ui.rs`) e `Transaction::Report`.

Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): 618 Rust + 15 PowerShell. Release de 18.607.104 bytes, não executado. Os testes de 5.000 linhas passaram no Release (limite de 5 s). Próximo teste no app: fim da Etapa 25.

Git: `main`, HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-04.
- Fora do HEAD: a Etapa 23, incluindo o prompt e o plano.
- O prompt da Etapa 26 é rascunho e fica fora do commit da 23.
- O `docs.rar` do usuário fica fora.

---

# Estado da Etapa 22 — pronta para versionar (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_22_DXF_DWG.md` (cinco decisões recomendadas). Plano `docs/internal/plans/2026-11-04-etapa-22.md`, ADR-0024, relatório `docs/internal/reports/ETAPA_22_DXF_DWG.md`.

- **Core:** `DisplayLayer` na lista de exibição; DXF R12 de página (`dxf_export.rs`):
  - camadas por categoria e espessura, ANSI 1252, `\U+XXXX`;
  - `SOLID` para preenchimento, tipos de linha para tracejado;
  - um arquivo por página, com nomes seguros.
- **GUI:** diálogo com os modos DXF e DWG; conversor externo (`dwg.rs`, ODA File Converter configurado em Configurações, com tempo limite e limpeza dos temporários).

Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): 597 Rust + 15 PowerShell. Release de 16.526.848 bytes. 200 páginas em DXF: 1,45 s.

Teste no executável das Etapas 20–22 feito em 2026-11-04, com autorização. Passaram numeração, Desfazer/Refazer, espelho, número manual do fio, PDF e DXF. Três defeitos foram encontrados e corrigidos:
- número do fio sobre o traço;
- número do fio com 11 px fixos;
- setas "→" como quadrados (fonte reserva Segoe UI Symbol).

Próximo teste no app: fim da Etapa 25.

Git: `main`, HEAD `1fe365998aada547d48bc84a01a4b770e86f0930` (Etapas 20–21), observado em 2026-11-04. A Etapa 22 está fora do HEAD; o `docs.rar` do usuário fica fora. Commit + push somente após o OK expresso.

---

# Estado da Etapa 21 — pronta para revisão (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_21_PDF_IMPRESSAO.md` (cinco decisões recomendadas). Plano `docs/internal/plans/2026-11-04-etapa-21.md` (9/9), ADR-0023, relatório `docs/internal/reports/ETAPA_21_PDF_IMPRESSAO.md`.

Core:
- lista de exibição da página (`display.rs`);
- referências da folha com destino (`sheet_references.rs`), usadas também pela tela;
- métricas Helvetica WinAnsi (`font.rs`);
- PDF com `pdf-writer` (`pdf.rs`): páginas em tamanho real ou no papel, links de referência, marcadores, imagens, preto e branco, relatório.

GUI:
- diálogo Exportar PDF / Imprimir (`pdf_export.rs`), com thread, progresso e cancelamento;
- impressão pelo sistema (`SystemShell`).

O formato do `.cadcraft` não mudou (continua no 7).

Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): 586 Rust + 15 PowerShell.

Release: `cad-gui.exe` com 16.448.512 bytes, não executado. Exportação de 200 páginas: 1,07 s.

Git: `main`, HEAD `98ef54ab48cb5e5f6ad24ca0cd170709319e2e1c`, observado em 2026-11-04.
- Fora do HEAD: Etapas 20 e 21, incluindo specs e planos.
- Commit + push das duas somente após o OK expresso.
- O `docs.rar` do usuário fica fora.
- Próximo teste no executável: fim da Etapa 22, com autorização.

---

# Estado da Etapa 20 — pronta para revisão (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_20_NUMERACAO_REFERENCIAS.md` (cinco decisões recomendadas). Plano `docs/internal/plans/2026-11-04-etapa-20.md` (9/9), ADR-0022, relatório `docs/internal/reports/ETAPA_20_NUMERACAO_REFERENCIAS.md`. Iniciada pelo Codex, concluída pelo Claude Code.

Core:
- `NumberingScheme` guardado no projeto;
- trava de designação;
- prévia + aplicação reversível para dispositivos (sequencial ou página/coluna, ordem da regra), fios (um número por rede, potencial e travados vencem, número na `Connection`) e páginas;
- espelho de contatos derivado;
- regra usada na inserção, na colagem e nas macros (opcional);
- `.cadcraft` formato 7 com migração e fixture v7.

GUI:
- diálogo de numeração com regra, alcance, prévia e conflitos;
- quatro transações novas no histórico;
- número e trava do fio em Propriedades da conexão;
- rótulo do fio na folha;
- espelho com links no painel do dispositivo;
- referências de interrupção.

Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): 577 Rust + 15 PowerShell. Release 15.815.168 bytes, não executado. O próximo teste no executável é ao fim da Etapa 22, com autorização.

Git: `main`, HEAD `98ef54ab48cb5e5f6ad24ca0cd170709319e2e1c` (Etapas 18–19), observado em 2026-11-04. A Etapa 20 inteira não está commitada; o prompt e o plano também não. O `docs.rar` do usuário fica fora. Commit + push só após o OK expresso.

---

# Estado da Etapa 19 — implementada, teste no app pendente (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_19_MACROS.md` (cinco decisões recomendadas). Plano `docs/internal/plans/2026-11-04-etapa-19.md`, ADR-0021, relatório `docs/internal/reports/ETAPA_19_MACROS.md`.

Core: `MacroDefinition`/`MacroContent`, captura da seleção ou da página, inserção com IDs UUID v7 novos, próxima designação por prefixo, fios e junções transformados, símbolos/peças ausentes acrescentados (versão diferente: mantém a do projeto e avisa), Undo/Redo exatos, `.cadmacro`. GUI: janela Macros (navegador, miniatura, criar, inserir, página nova, importar/exportar/excluir), ferramenta de inserção com Girar/Espelhar, colar/duplicar com fios. `.cadcraft` continua no formato 6.

Validação 2026-11-04 (`scripts/validate.ps1`, exit 0): 556 Rust + 15 PowerShell. Release 15.538.688 bytes. Teste no executável das Etapas 18–19 feito em 2026-11-04: macros, colar com fios e peças passaram; corrigida a prévia escondida pela folha; inserir função não representada pelo Centro de inserção não concluído (cliques bloqueados no painel direito pelo ambiente).

Git: `main`, HEAD `088637983e0933259f230f123a3777c8eadd17de` (Etapa 17), observado em 2026-11-04. Etapas 18 e 19 não commitadas; `docs.rar` do usuário fica fora. O usuário autorizou commit + push das duas etapas se o teste no app passar.

---

# Estado da Etapa 18 — pronta para revisão (2026-11-04)

Prompt aprovado: `docs/internal/spec/ETAPA_18_DISPOSITIVOS_PECAS_FUNCOES.md`; decisões: `.cadpart` individual, atribuição via `Component`, CSV e inserção de função não representada. Plano: `docs/internal/plans/2026-11-03-etapa-18.md`. ADR-0020. Etapa 17 versionada e enviada em `0886379`.

Core: `Part` e modelo de funções, atribuição/atualização reversível, edições de funções/pontos, inserção de função existente, CSV, BOM e `.cadcraft` formato 6 com migração. GUI: biblioteca configurável, editor de peças/dispositivos/funções/pontos, Centro de inserção e histórico comum. Validação integral: 539 testes Rust e 15 PowerShell, 0 falhas; `fmt`, `check`, Clippy e `diff --check` passaram. Release passou (`cad-gui.exe`, 14.716.928 bytes). Aplicativo não executado. Homologação manual pendente.

Git: `main`, HEAD `088637983e0933259f230f123a3777c8eadd17de`, observado em 2026-11-04; `origin/main` coincide na consulta `ls-remote` sem fetch. Alterações da Etapa 18 ainda não commitadas; `docs.rar` é arquivo do usuário, não rastreado e excluído da etapa. Commit + push da Etapa 18 exigem novo OK expresso.

---

# Estado da Etapa 17 — pronta para revisão (2026-11-03)

Biblioteca externa `.cadsym`, editor de símbolos, elipse/estilos/
preenchimento, textos dinâmicos (designação, função, porta, referência,
propriedade), versões e "Atualizar símbolos do projeto", importação `.elmt`
(0,5 mm/px ajustável). `.cadcraft` formato 5 com fixture v5. ADR-0019.
Relatório: `docs/internal/reports/ETAPA_17_BIBLIOTECA_EDITOR_SIMBOLOS.md`.

Validação 2026-11-03 (`scripts/validate.ps1`, exit 0): **526 Rust** e **15
PowerShell**; Release 13.836.800 bytes (não executado). Git: `main`, HEAD
`4a32b54` (Etapas 15 e 16); Etapa 17 fora do HEAD. Próximo: OK para
versionar; teste no app ao fim da Etapa 19.

---

# Estado da Etapa 16 — pronta para revisão (2026-11-03)

Conteúdo livre (texto, linha, retângulo, círculo, arco) e área de
transferência. Iniciada pelo Codex, concluída pelo Claude Code com a revisão
aprovada: colar símbolo cria dispositivo novo (ou função do mesmo
dispositivo), recortar remove na hora, Ctrl+D, colar repetido, duplo clique
edita texto; `.cadcraft` formato 4 com fixture v4. ADR-0018. Relatório:
`docs/internal/reports/ETAPA_16_CONTEUDO_LIVRE_CLIPBOARD.md`.

Teste no app (14–16) feito em 2026-11-03 com autorização; Ctrl+C/X/V corrigidos.
Validação 2026-11-03 (`scripts/validate.ps1`, exit 0): **513 Rust** e **15
PowerShell**; Release 13.283.328 bytes. Achados abertos: CLP-007/008,
GUI-016, FILE-005b. Próximo teste no app ao fim da Etapa 19. Git: `main`, HEAD `7bacd28` (Etapa 14); Etapas 15 e 16 fora do
HEAD, sem commit/push (as duas compartilham arquivos).

---

# Estado da Etapa 16 — preparação iniciada (2026-11-03)

Usuário autorizou iniciar a Etapa 16. Criados prompt detalhado em rascunho
(`docs/internal/spec/ETAPA_16_CONTEUDO_LIVRE_CLIPBOARD.md`) e plano preliminar
(`docs/internal/plans/2026-11-03-etapa-16-rascunho.md`), derivados do Plano Mestre.
A aprovação expressa do prompt permanece pendente conforme `AGENTS.md` e o
Plano Mestre. Nenhum código, build ou teste da Etapa 16 foi iniciado. Etapa 15
segue implementada, validada e fora do HEAD. Não houve commit/push.

---

# Estado da Etapa 15 — pronta para revisão (2026-11-03)

Etapa 15 implementada: intercâmbio de molduras EPLAN `.fn1`, QElectroTech
`.titleblock` e DXF ASCII; exportação DXF. O Core mantém relatório de itens
ignorados/aproximados e segurança de entrada; a GUI oferece prévia, opções DXF,
atribuição à página e confirmação reversível em um passo. A origem interna
EPLAN é ajustada quando a borda da folha dá âncora confiável.

Validação oficial `scripts/validate.ps1` em 2026-11-03: 495 testes Rust e
15 PowerShell, 0 falhas; fmt, check, clippy `-D warnings` e diff check passaram.
Release aprovado (`cad-gui.exe`, 13.112.832 bytes; hash no relatório).
Sem teste manual do app nesta etapa, conforme decisão
do usuário; previsto ao fim da Etapa 16. Homologação geral na Etapa 28.

Git: `main`, HEAD observado `7bacd28fd5f2e86af6a1091f0e13ab52e2659b5a`
(Etapa 14), consulta em 2026-11-03. Toda a Etapa 15 está fora do HEAD.
`origin/main` consultado sem fetch em 2026-11-03 coincide com esse HEAD.
O usuário proibiu commit e push nesta retomada. `docs.rar` já estava não
rastreado na árvore e não foi alterado nem incluído na etapa.

---

# Estado da Etapa 14 — pronta para revisão (2026-11-03)

Etapa 14 (Fase A: editor de molduras) implementada e validada
automaticamente. Core: imagens do projeto (`domain/asset.rs`, PNG/JPEG,
10 MB / 4096 px, bytes em `assets/` do `.cadcraft`), elemento `Image`, marca
d'água, `FrameDraft`/versão + 1, `FrameEdit::AddAsset/RemoveAsset/Batch`,
`.cadframe` e importação com remapeamento; `.cadcraft` formato 3. GUI:
editor de molduras (`frame_editor.rs`) com ferramentas, mover/duplicar/
excluir/alinhar, propriedades do elemento e da moldura, comandos em
Dados-mestre > Molduras, pasta da biblioteca nas Configurações. ADR-0016.

Validação 2026-11-03 (`scripts/validate.ps1`, exit 0): **480 testes Rust**
(466 anteriores + 14 novos) e **15 PowerShell**; Release 12.774.912 bytes.
Relatório: `docs/internal/reports/ETAPA_14_EDITOR_DE_MOLDURAS.md`.

Git: `main`, HEAD `98d57af21efbfda30bddabce23ca194aefb07e99` (Etapa 13);
toda a Etapa 14 fora do HEAD. Validação funcional no executável pelo agente
adiada: teste do app a cada 3 etapas (próximo ao fim da Etapa 16). Usuário
autorizou commit + push ("Pode versionar"); SHA em `git log`. Não iniciar a
Etapa 15 sem prompt aprovado.

---

# Estado da Etapa 13 — pronta para revisão (2026-11-03)

Etapa 13 (Fase A: modelo de moldura, zonas e referências) implementada e
validada automaticamente. Molduras no Core (`domain/frame.rs`,
`project/frames.rs`): zonas de colunas/linhas, textos fixos e especiais com
índice, moldura padrão do projeto e por página, `/página.coluna`;
`.cadcraft` formato 2. GUI: moldura e rótulos na folha, referências cruzadas
na folha, Propriedades (moldura, formato A0–A4, metadados, formato de
referência), molduras de fábrica em projeto novo. ADR-0015.

Validação 2026-11-03 (`scripts/validate.ps1`, exit 0): **466 testes Rust**
(452 anteriores + 14 novos) e **15 PowerShell**; Release 12.061.184 bytes.
Relatório: `docs/internal/reports/ETAPA_13_MOLDURAS_ZONAS_REFERENCIAS.md`.

Git: `main`, HEAD `6612e82be8aea4219124a2497b06898498b01b7a`; Etapa 13 e o
registro do Plano Mestre (AGENTS.md, ADR-0002 adendo, ROADMAP) fora do HEAD.
Validação funcional na interface feita pelo agente (roteiro no relatório);
dois textos desatualizados corrigidos. O usuário autorizou ("Corrigir e
versionar"): commit + push da Etapa 13 pelo agente; SHA em `git log`.
Próxima ação: prompt da Etapa 14 (editor de molduras). Não iniciá-la sem OK.

---

# Estado — Plano Mestre aprovado (2026-11-03)

Etapas 1–12 concluídas; `origin/main` = `6612e82` (Etapa 12). Plano Mestre
com Etapas 13–28 aprovado (ROADMAP e `docs/internal/plans/PLANO_MESTRE_2026-11-03.md`).
Regra de Git atualizada (AGENTS.md, ADR-0002 adendo): commit + push pelo agente
após OK expresso por etapa. Homologação geral na Etapa 28. Próxima ação:
aprovação do rascunho `docs/internal/spec/ETAPA_13_MOLDURAS_ZONAS_REFERENCIAS.md`.
Alterações de documentação desta sessão ainda não versionadas.

---

# Estado da Etapa 12 (Marco M0.1) — pronta para revisão (2026-11-03)

Etapa 12 implementada e validada automaticamente. Em 2026-11-03 o usuário
executou o roteiro manual da etapa e informou "tudo funcionando"; a
homologação geral fica para o momento definido no novo plano do usuário.
Origem: auditoria CAD-CRAFT × QElectroTech aprovada pelo usuário (seção 7 =
prompt, `docs/internal/spec/ETAPA_12_PROJETO_REAL_PERSISTENCIA.md`), dependências `zip`
e `rfd` autorizadas. Formato `.cadcraft` v1 (ZIP + `manifest.json` com
`format`/`min_reader` + `project.json`) em `cad_craft::persistence`; snapshot
JSON lido como v0; leitor defensivo. GUI: Abrir/Salvar/Salvar como
(Ctrl+O/S/Shift+S), recentes, trava `.lock`, recuperação automática,
gravação atômica (`cad-gui/src/project_file.rs`). Inserção com "Novo
dispositivo" (Core `place_symbol`) e Undo de documentos/páginas
(`edit_document`). ADR-0014.

Validação 2026-11-03 (`scripts/validate.ps1`, exit 0): **452 testes Rust**
(418 anteriores + 34 novos) e **15 PowerShell**, 0 falhas/ignorados; fmt,
check, clippy `-D warnings`, `git diff --check` e Release aprovados.
Relatório: `docs/internal/reports/ETAPA_12_PROJETO_REAL_PERSISTENCIA.md`.

Git (somente leitura): `main`, HEAD `34785ab03895f8c691838fb76201427831f28349`
(Etapa 11 commitada); remoto `origin/main` = `f1b173e` (Etapa 11 não enviada).
Em 2026-11-03 o usuário autorizou expressamente o agente a fazer o commit e o
push da Etapa 12 (exceção pontual da política do ADR-0002; não vale para
operações futuras). O SHA desse commit não está registrado aqui: consultar
`git log`. Próxima ação: definir com o usuário o novo plano das etapas.

---

# Estado da Etapa 11 — pronta para revisão (2026-11-02)

Etapa 11 (conectores e especializações de bornes) implementada e validada
automaticamente; **não homologada**. Contatos de conector reutilizam
`Terminal`; `Connector` e `ConnectorMating` (documental, sem continuidade)
tipados; especializações de borne (passagem, PE, multinível, distribuição,
seccionável) com regras verificáveis em `Terminal::check_specialization`;
nível de borne em `ConnectionPoint`; associação ponto↔borne editável
(`SetPointTerminal`). GUI: aba Conectores, editor de especialização na aba
Réguas, navegador Bornes hierárquico, Propriedades; janela limitada à tela
(GUI-PHY-03). ADR-0013. DEC-11-01/02 aguardam decisão do usuário.

Validação 2026-11-02 (`scripts/validate.ps1`, exit 0): **418 testes Rust**
(391 anteriores + 27 novos) e **15 PowerShell**, 0 falhas/ignorados; fmt,
check, clippy `-D warnings`, `git diff --check` e Release aprovados
(10.358.784 bytes). Relatório: `docs/internal/reports/ETAPA_11_CONECTORES_BORNES_ESPECIAIS.md`.

Git (consulta somente leitura): `main`, HEAD
`f1b173e9e5a9d0a485163bfd2c13a871a6b46cbd` (Etapa 10 commitada e enviada).
Toda a Etapa 11 está fora do HEAD. Próxima ação: revisão do usuário e
**commit manual** (etapa ímpar). Não iniciar a Etapa 12.

---

# Estado da Etapa 10 — pronta para revisão (2026-11-02)

Etapa 10 implementada e validada automaticamente; **não homologada**.
Iniciada pelo Codex (interrompido por franquia) e concluída pelo Claude Code
a partir do código real. GUI-ROUTE-09A/09B corrigidos com regressões (causas
no KNOWN_ISSUES/relatório). Engenharia física: `PhysicalEdit`/`PhysicalDelta`
reversíveis, `Batch` atômico, pontes como conexões explícitas de régua,
numeração única por régua, listas técnicas derivadas, `physical_occurrence`.
GUI: janela Engenharia física (réguas/bornes/pontes, cabos/veias, condutores,
listas), comandos nas guias Dispositivos/Conexões/Cabos, navegadores Cabos e
Bornes hierárquicos, dados físicos em Propriedades e no inspetor de conexão,
Undo/Redo no histórico existente. ADR-0012.

Validação 2026-11-02: fmt, check, clippy -D warnings, `git diff --check` e
Release aprovados; **389 testes Rust, 0 falhas/ignorados** (364 anteriores
preservados). Os 15 testes PowerShell não rodaram: `pwsh` ausente (PHY-013).
Release 9.896.448 bytes. Revisão independente: sem P1; P2/P3 corrigidos.
Relatório: `docs/internal/reports/ETAPA_10_BORNES_CABOS_CONDUTORES.md`.

Correção GUI-PHY-01 (homologação manual, 2026-11-02): janela Engenharia física
crescia continuamente na aba Listas técnicas; layout corrigido com painéis
internos e rolagem; 2 regressões novas. Validação: **391 testes Rust, 0 falhas**;
fmt/check/clippy/diff check aprovados. Aguarda reteste manual do usuário.
A correção introduziu GUI-PHY-02 (controles fora da janela; painéis internos
não alocam espaço). Corrigido com layouts bottom_up/top_down alocados; testes
ampliados verificam formas pintadas dentro do fundo. 391 testes Rust, 0 falhas.
Release não reconstruído (exe em uso, os error 5).

Git (consulta somente leitura): `main`, HEAD
`a4f8233799292741c5e93bb65aec28983f1c9d41` (Etapa 9 commitada). Toda a
Etapa 10 está fora do HEAD. Próxima ação: revisão do usuário e **commit +
push manuais** (etapa par). Não iniciar a Etapa 11.

---

# Estado da Etapa 9 — pronta para revisão (2026-11-02)

A correção geométrica da Etapa 8 passou a porta oficial antes dos novos recursos:
350 testes (335 Rust e 15 PowerShell), sem falhas. Na Etapa 9, Core e GUI já
implementam identificação de dispositivos/funções, referências por ocorrência,
marcadores de interrupção/potencial, grupos estrela/cadeia, continuidade
explícita opcional, navegação por ID e deltas reversíveis. A revisão independente
identificou três lacunas de GUI; ordem editável, diagnósticos visíveis e linhas
de ocorrência no navegador foram acrescentados. Validação oficial passou:
364 Rust + 15 PowerShell = 379 testes, zero falhas/ignorados; fmt, check,
clippy e diff check aprovados. Build Release aprovado, 9.130.496 bytes.
Relatório e plano
em docs/internal/reports/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md e
docs/internal/plans/2026-10-31-etapa-09.md. Em 2026-11-02, `main`, HEAD local e
`origin/main` consultado: 5377559ff3b5bd5ce1cd9d19265833ef3714796e.
Todas as mudanças permanecem não commitadas. Commit manual da Etapa 9 pendente;
sem homologação manual nem push executado.

O ai-memory está acessível nesta sessão: consulta de observações retornou
sucesso. A instalação ambiental registrada abaixo é separada da Etapa 9.

## Histórico de retomada de 2026-10-31 (superado pelo estado acima)

## Instalação ambiental de 2026-11-02

`ai-memory 3.3.1` instalado e ativo para a CLI Codex nativa no Windows.
Serviço local, MCP, hooks confiados e skills configurados; teste curto gerou
1 sessão e 4 observações. O Codex desktop já aberto deve ser reiniciado para
carregar a configuração. Relatório: `docs/internal/reports/AI_MEMORY_INSTALACAO_2026-11-02.md`.
Validação geral do checkout nesta intervenção parou em formatação da Etapa 9;
`cargo check`, `test` e `clippy` também falharam no código parcial dessa etapa.
Os 15 testes operacionais PowerShell e `git diff --check` passaram.
HEAD consultado em 2026-11-02: `5377559ff3b5bd5ce1cd9d19265833ef3714796e`,
branch `main`. A Etapa 9 e esta documentação seguem não commitadas; nenhum Git
mutável do CAD-CRAFT foi executado. Continuar a Etapa 9 do estado real, sem
tratar o teste ambiental como validação funcional ou homologação.

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
# Estado atual

**Etapa 8 — editor de conexões implementado e validado automaticamente.**
A homologação manual abrangente permanece adiada até após a Etapa 15.
Etapas 9 e 17 não foram iniciadas.

- Consulta em 2026-10-30, America/Sao_Paulo. Branch `main`; origin existente
  `https://github.com/Vandecao/CAD-CRAFT.git`.
- Último HEAD observado por `git rev-parse HEAD`:
  `6bfa4c24e3b3a39642cea1e387a3a7f502835828` (Etapa 7). A Etapa 7 foi commitada
  manualmente; a memória anterior de commit pendente foi superada por essa consulta.
- A Etapa 8, incluindo esta memória, está fora desse HEAD. Nenhum staging, commit,
  push, fetch, pull, worktree ou outra alteração de Git foi executado pelo Codex.
  Tracking local ahead 1 não confirma o estado atual remoto; ver consulta final no relatório.
- Especificação: `docs/internal/spec/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md`.
  Plano: `docs/internal/plans/2026-10-29-etapa-08-conexoes.md`. Decisão: ADR-0010.
  Evidências, contagem final e inventário: `docs/internal/reports/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md`.
- Core: vínculos explícitos PortRef→ConnectionPoint, validados pelo alvo da
  ocorrência. Rotas compartilham RepresentationId e apontam para ConnectionId.
  Junções têm UUID v7, incidência geométrica e arestas auxiliares de posse exclusiva
  no mesmo grafo; nenhum Device/Function/condutor físico artificial.
- Cruzamentos não criam redes. Criar/excluir rota, alterar vértices/segmentos,
  associar portas e criar/remover junção usam ConnectionDrawingEdit/Delta atômicos.
  Replay valida pré-estado e preserva IDs; snapshots antigos recebem defaults.
- Transforms ajustam endpoints por identidade; grupo mantém percurso. Junções
  acompanham translação comum ou causam recusa explícita se perderem incidência.
  Prévia e confirmação usam o mesmo algoritmo puro de reparo terminal.
- GUI: ferramenta na guia Conexões, rota L alternável por Espaço, waypoints,
  snap porta/junção/vértice/segmento/grade e sugestões visuais de alinhamento.
  Prévia sem mutação/IDs definitivos, cancelamento Esc/foco/modal/pan/página.
- Hit por traço, alças de vértice, arraste de segmento/vértice, menus contextuais,
  propriedades reais, vínculos de porta, destaque de rede derivada e exemplo multifilar.
  Símbolos precedem rotas na mesma camada. Junções têm seleção própria e marca
  configurável ponto/anel. Snapshot técnico continua limitado a 8 MiB.
- Histórico mantém 100 transações/50.000 deltas; inclui rotas/junções/topologia e
  movimento de símbolos. Não é Undo geral nem histórico persistido; dirty conservador.
- Revisão independente Core concluída, sem defeito funcional confirmado. Revisão
  GUI encontrou problemas de seleção/arbitragem/prévia, corrigidos com regressões.
  Após interrupção por limite de uso, a revisão independente final foi concluída;
  GUI-REV-01–04 fechados e nenhum P1/P2 funcional aberto.
- Limites: rotas terminam em portas de símbolos; junções unem rotas existentes.
  Ramo porta→junção sem porta, roteamento automático industrial, rastreamento de
  destinos EPLAN, pontes decorativas, interrupções e potenciais completos são futuros.
  Ângulos são dobras de rota, não novos dispositivos ou catálogo certificado.
- Índices Core são reconstruídos em confirmação; cache gráfico por revisão/página,
  hit/snap lineares na página. Medições locais no relatório não garantem escala industrial.
- Sem nova dependência, alteração de Cargo.lock ou exclusão de testes anteriores.
  Sem alteração da grade lógica, da Etapa 17 ou do layout geral aprovado.

Validação oficial: **339 testes = 324 Rust + 15 PowerShell**, zero falhas/ignorados.
Os 272 anteriores foram preservados; 67 novos. Core 159, GUI 165.
Release aprovado, 8.723.968 bytes, PE Subsystem 2; hash no relatório. Capturas
internas 1600×900/100% e 1280×720/125% inspecionadas. Ensaio denso GUI: cinco
frames CPU com tesselação ~1,992 s; custo elevado registrado, sem garantia industrial.

Próxima ação: revisão da entrega e **commit + push manuais da Etapa 8**.
Backup externo independente recomendado, não criado/verificado. Não iniciar Etapa 9.
