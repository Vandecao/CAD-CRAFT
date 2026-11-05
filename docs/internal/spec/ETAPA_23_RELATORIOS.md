# CAD-CRAFT — Etapa 23 (Fase C): relatórios em página e CSV/XLSX

**Estado: aprovado pelo usuário em 2026-11-04 ("Aprovo todas") para execução
pelo Codex.** Decisões aprovadas (todas as recomendadas): (1) página guarda a
definição e a tabela é calculada do cadastro; Gerar/Atualizar ajustam o
número de páginas, com aviso de desatualizado; (2) XLSX com
`rust_xlsxwriter`; (3) CSV com `;` e UTF-8 com BOM; (4) os seis tipos em
tabela; (5) definição persistida no `.cadcraft` formato 8 com migração.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase C, Etapa 23:
"Relatórios em página e CSV/XLSX", depende das Etapas 20 e 21). Base:
Etapa 22 versionada em `0b3b65c` (597 Rust + 15 PowerShell; teste no
executável das Etapas 20–22 feito).

## Objetivo

Gerar os relatórios da documentação elétrica:
- **páginas de relatório** dentro do projeto, com a moldura e tabelas
  paginadas, que entram no PDF e no DXF como qualquer outra página;
- **arquivos de lista** em CSV e XLSX.

É o equivalente a "Relatórios > Gerar/Atualizar" do EPLAN, sem copiar código,
formulários ou dados de terceiros.

## O que já existe (não refazer)

- Lista de materiais: `Project::bom_rows()` e `interchange::export_bom_csv`
  (Etapa 18).
- Listas físicas: `Project::physical_lists()` → `PhysicalLists`, com
  bornes, conectores e contatos, réguas, cabos, veias, condutores e
  atribuição de fiação (`domain/project/physical_lists.rs`, Etapas 9–11).
- Designações, números de fio, páginas e referências `/página.coluna`
  (Etapas 13 e 20); `short_reference` e `sheet_references` (Etapa 21).
- Lista de exibição da página (`display.rs`), com PDF (`pdf.rs`) e DXF
  (`dxf_export.rs`). Uma página de relatório que gere itens na lista de
  exibição já sai no PDF e no DXF sem código novo de exportação.
- Moldura com zonas e textos especiais (Etapas 13–14).
- Marcadores de lugar na faixa: Ferramentas > Relatórios ("Gerar",
  "Atualizar", "Avaliar projeto"…) e Dispositivos > "Lista de materiais".
- Padrão de diálogo com prévia, transação única, thread, progresso e
  cancelamento (`numbering_ui.rs`, `pdf_export.rs`).

## Escopo

1. **Tipos de relatório** (decisão 4). Cada tipo é uma tabela de linhas
   calculada só do cadastro, sem GUI:
   - **Índice de páginas**: documento, número, nome, tipo, data e revisão,
     quando existirem;
   - **Lista de dispositivos**: designação completa, classificação, funções,
     peças e ocorrências `/página.coluna`;
   - **Lista de materiais**: código, fabricante, descrição, quantidade e
     dispositivos; agrupada e somada, ou por dispositivo;
   - **Lista de conexões (fios)**: número do fio ou rede, origem
     (dispositivo:ponto), destino, `/página.coluna`, condutor físico,
     seção e cor quando houver;
   - **Lista de bornes**: régua, borne, ponte, lado interno e externo
     (destinos) e cabo/veia;
   - **Lista de cabos**: cabo, tipo, veias usadas e livres, origem, destino,
     comprimento quando houver.

   Para cada tipo: colunas padrão, larguras e cabeçalhos; filtro por
   documento ou página e por caminho `=`/`+`; ordenação estável e
   documentada.
2. **Páginas de relatório** (decisões 1 e 5):
   - Ferramentas > Relatórios > Gerar: escolher o tipo, o filtro, o
     documento de destino, a moldura e o formato do papel; prévia do número
     de páginas;
   - Gerar cria as páginas do relatório numa **transação reversível**;
   - cada página guarda a **definição do relatório** (tipo, filtro,
     ordenação, colunas e índice da página dentro do relatório);
   - a tabela é desenhada pela lista de exibição a partir do cadastro atual;
   - quando o cadastro muda e o relatório passa a precisar de mais ou menos
     páginas, ele é marcado **desatualizado** (aviso no painel e na página);
   - **Atualizar** (um ou todos) ajusta o número de páginas numa transação
     reversível, reaproveitando as páginas existentes, sem duplicar e sem
     apagar páginas de esquema;
   - Desfazer/Refazer exatos.
3. **Formato da tabela**:
   - linhas com altura fixa (mm) e cabeçalho repetido em cada página;
   - texto cortado com "…" quando não cabe na coluna (a lista de exibição
     mede pela Helvetica, `font.rs`);
   - linhas de grade;
   - rodapé "página N de M do relatório";
   - a área da tabela respeita a área útil da moldura;
   - na tela e no PDF, as referências `/página.coluna` das listas viram
     links (o PDF já transforma `Link` em anotação; usar o mesmo item).
4. **Exportar listas** (decisões 2 e 3):
   - Arquivo > Exportar > Lista: tipo, filtro e formato (CSV ou XLSX),
     destino pelo diálogo nativo;
   - mesmas colunas e linhas das páginas;
   - XLSX com uma aba por tipo quando o usuário exportar vários de uma vez,
     cabeçalho em negrito, larguras ajustadas e filtro automático;
   - números como número e textos como texto.
5. **Persistência**: a definição do relatório na página entra no
   `.cadcraft` formato 8 (`min_reader` 8), com migração 7→8 e a fixture
   `etapa23-v8.cadcraft`; as fixtures v0–v7 continuam abrindo.
6. **Desempenho**: lista de materiais e de conexões com 5.000 linhas geram
   as páginas e o XLSX em menos de 5 s no Release; o teste usa um limite
   folgado em Debug.

## Critérios de aceitação

1. Para cada tipo, um projeto de teste com dados conhecidos gera as linhas
   esperadas, conferidas campo a campo, inclusive o número do fio, a
   referência `/página.coluna` e a quantidade somada na lista de materiais.
2. Gerar a lista de conexões com 120 linhas numa moldura A3 cria o número
   certo de páginas, com o cabeçalho repetido e o rodapé "página N de M".
   Um único Desfazer remove todas as páginas; Refazer as recria iguais.
3. Mudar o cadastro (acrescentar dispositivos até passar de uma página):
   o relatório fica desatualizado e o aviso aparece. Atualizar acrescenta a
   página que falta; remover dados e Atualizar tira a página excedente.
   Páginas de esquema nunca são tocadas.
4. As páginas de relatório saem no PDF (com links nas referências) e no
   DXF; o texto que não cabe na coluna termina em "…".
5. CSV com o separador e a codificação da decisão 3: abre no Excel em pt-BR
   com os acentos certos. XLSX lido de volta no teste (leitor independente
   só nos testes, ou o próprio XML do pacote) com abas, cabeçalhos, tipos e
   valores certos.
6. Com eventos egui reais: Gerar relatório pelo diálogo, prévia do número de
   páginas, confirmar, navegar até a página; Atualizar após uma mudança;
   Exportar lista em CSV e XLSX.
7. Salvar e reabrir conserva as definições; fixtures v0–v7 abrem; v8
   versionada.
8. Desempenho do item 6 do escopo.
9. Testes anteriores preservados; `scripts/validate.ps1`; Release compilado
   sem executar; relatório `docs/internal/reports/ETAPA_23_RELATORIOS.md`, ADR-0025 e
   memória; commit + push somente após o OK expresso. **Sem teste no
   executável nesta etapa** (o próximo é ao fim da Etapa 25, com
   autorização do usuário).

## Decisões pedidas ao usuário

1. **Conteúdo das páginas de relatório**:
   - **Recomendado:** a página guarda a definição, e a tabela é calculada
     do cadastro a cada exibição, sempre atual. Gerar e Atualizar só
     ajustam o **número** de páginas, e o relatório fica "desatualizado"
     quando esse número muda.
   - Alternativa: um instantâneo fixo (textos e linhas gravados na página),
     que só muda ao Atualizar, como no EPLAN. Fica mais simples de
     imprimir, mas pode ficar velho sem aviso.
2. **XLSX**:
   - **Recomendado:** o crate `rust_xlsxwriter` (Rust puro, MIT/Apache-2.0),
     com versão exata.
   - Alternativa: só CSV nesta etapa.
3. **CSV**:
   - **Recomendado:** separador `;` e UTF-8 com BOM, que abrem direto no
     Excel em português com os acentos e as vírgulas decimais certos.
   - Alternativa: `,` sem BOM, o padrão internacional.
4. **Tipos nesta etapa**:
   - **Recomendado:** os seis do escopo (índice, dispositivos, materiais,
     conexões, bornes e cabos), em **tabela**.
   - O diagrama de bornes **gráfico** e o plano de cabos gráfico ficam para
     depois.
5. **Persistência**:
   - **Recomendado:** a definição do relatório na página, com o
     `.cadcraft` no formato 8 e migração.
   - Alternativa: relatórios só em memória, gerados de novo a cada sessão.
     Seria pior para quem abre o projeto depois.

## Fora do escopo

Editor de formulários de relatório (layout livre, logotipos por relatório);
diagrama de bornes e plano de cabos gráficos; relatórios de painel e 3D;
avaliação de projeto e verificações (Etapa 24); importação de formulários do
EPLAN (Etapa 26).

## Instruções para o agente

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md` e
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory do projeto
   (`notes/etapa-22-dxf-dwg.md` e, se existir, `notes/etapa-23-*`). ADRs
   úteis: 0020 (peças), 0022 (numeração), 0023 (lista de exibição e PDF) e
   0024 (DXF).
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). Base
   `0b3b65c`; o `docs.rar` é do usuário e nunca entra em commit.
3. Só começar depois de a aprovação estar registrada no topo deste arquivo;
   dúvidas fora das decisões: parar e perguntar ao usuário.
4. Criar `docs/internal/plans/<data>-etapa-23.md` com o desenho técnico, a seção
   "Retomada por outro agente" e a lista "Andamento" numerada; marcar cada
   item só com teste passando. Gravar no ai-memory a página fixada
   `notes/etapa-23-relatorios.md` e atualizá-la ao final.
5. Padrões:
   - o Core não depende de GUI; as linhas dos relatórios, a paginação e o
     CSV/XLSX ficam em `cad-craft`, e a tabela entra na lista de exibição
     (`display.rs`), de modo que PDF e DXF não mudam;
   - Gerar e Atualizar usam deltas reversíveis com conferência de estado e
     reversão em erro (ver `project/numbering.rs` e `project/macros.rs`);
     cada operação é um `Transaction::*` em `cad-gui/src/history.rs`;
   - migração `.cadcraft` encadeada em `persistence.rs::migrate`, com
     fixture nova e as antigas abrindo;
   - nenhuma dependência nova além da aprovada na decisão 2, com versão
     exata;
   - textos da interface sem caracteres fora das fontes (a fonte reserva de
     símbolos existe, `theme::install_symbol_font`, mas prefira texto
     simples nas tabelas);
   - testes egui com accesskit (ver `cad-gui/tests/cad_export_ui.rs`); no
     ajudante de clique, ignore rótulos e títulos de janela e prefira o
     controle mais baixo quando a faixa repetir o nome do comando;
   - não remover nem afrouxar testes.
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; compilar o
   Release sem executar o app. Se o disco encher, `cargo clean`.
7. Atualizar relatório, ADR-0025, `DECISIONS.md`, `ROADMAP.md` (linha 23),
   `KNOWN_ISSUES.md`, `CURRENT_STATE.md` e `HANDOFF.md`; informar que está
   pronto e aguardar o OK para versionar (commit + push em `main`, sem
   força). Não iniciar a Etapa 24.
