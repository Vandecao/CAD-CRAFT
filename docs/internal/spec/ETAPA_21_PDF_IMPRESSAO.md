# CAD-CRAFT — Etapa 21 (Fase C): PDF multipágina com links e impressão

**Estado: aprovado pelo usuário em 2026-11-04 ("Aprovo todas").**
Decisões aprovadas (todas as recomendadas): (1) `pdf-writer` + `lopdf` só
em testes; (2) Helvetica WinAnsi, caracteres fora substituídos e relatados;
(3) impressão pelo PDF via sistema; (4) lista de exibição no Core, tela com
o desenho atual e textos de referência/espelho do Core; (5) links de
referências cruzadas e marcadores documento → página.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase C, Etapa 21,
depende de 13 e 14; a Etapa 22, DXF, reaproveita a lista de exibição desta).
Base de código: Etapas 18–19 em `98ef54a`; Etapa 20 implementada e validada
(577 Rust + 15 PowerShell), ainda não versionada. O commit + push das
Etapas 20 e 21 será feito junto, após o OK expresso do usuário.

## Objetivo

Gerar a documentação do projeto em PDF vetorial, com várias páginas, igual
ao que se vê na folha. As referências cruzadas devem ser clicáveis e o PDF
deve ter marcadores (bookmarks) por documento e página. O projeto também
deve poder ser impresso. É o equivalente a "Exportar PDF" e "Imprimir" do
EPLAN, sem copiar código ou dados de terceiros.

## O que já existe (não refazer)

- A folha é desenhada só na GUI, com o pincel do egui:
  - `viewport.rs`: página, símbolos, fios, junções, rótulo do número do fio;
  - `frame_view.rs`: moldura, zonas, imagens, referências cruzadas, espelho
    de contatos, interrupções;
  - `symbol_view.rs`: primitivas e estilos de símbolo, textos dinâmicos.
- No Core já existem: `reference_label` e `resolve_special` (textos
  especiais da moldura) em `project/frames.rs`; `symbol_text` (textos
  dinâmicos) em `project/symbol_texts.rs`; `marker_references`,
  `contact_mirror`, `device_references` e `function_references`.
- Os textos de referência cruzada e do espelho que aparecem na folha são
  montados na GUI (`frame_view.rs::cross_reference_texts` e
  `contact_mirror_texts`).
- Imagens da moldura ficam em `assets/` do `.cadcraft`; a GUI decodifica
  PNG/JPEG com o crate `image` (aprovado na Etapa 14).
- Já existe exportação DXF de **moldura** (`interchange/dxf.rs::export`, Etapa 15).

## Escopo

1. **Lista de exibição no Core (sem GUI)** (decisão 4). `Project::page_display_list(page)`
   devolve, em milímetros, tudo o que a folha mostra:
   - linhas, polilinhas, polígonos com preenchimento, círculos, elipses e
     arcos, com cor, espessura e tipo de traço;
   - textos com altura, rotação, alinhamento e texto já resolvido;
   - imagens, com o asset e o retângulo;
   - áreas de link com o destino (página + ponto).

   O conteúdo coberto é: moldura e zonas, textos especiais, símbolos com
   textos dinâmicos, fios, junções, marcadores (potencial, interrupção),
   conteúdo livre, referências cruzadas, espelho de contatos e números de
   fio. Os textos de referência e do espelho passam a ser montados no Core;
   a GUI consome a mesma função, para que tela e PDF mostrem o mesmo texto.
2. **Escritor de PDF** (decisões 1 e 2):
   - uma página PDF por página do projeto, com o tamanho real (mm → pt) e
     geometria vetorial;
   - textos selecionáveis e pesquisáveis;
   - imagens JPEG embutidas como estão e PNG decodificado e comprimido;
   - metadados (título, autor e data do projeto);
   - opção colorido ou preto e branco.
3. **Links e marcadores** (decisão 5):
   - links clicáveis em cada referência cruzada (auxiliar → principal,
     itens do espelho, interrupções do mesmo grupo), que levam à página e à
     posição do destino;
   - marcadores em árvore: documento → páginas (número e nome);
   - um destino que fica fora das páginas exportadas não vira link e é
     relatado.
4. **Exportar PDF (GUI)**: Arquivo > Exportar > PDF. O diálogo oferece:
   - alcance: projeto, documento ativo, página ativa ou páginas escolhidas;
   - cor ou preto e branco;
   - marcadores sim/não;
   - abrir ao terminar;
   - destino pelo diálogo nativo.

   A exportação mostra progresso, roda sem travar a interface e termina com
   um relatório (páginas, links, avisos, por exemplo caracteres
   substituídos).
5. **Imprimir** (decisão 3): Arquivo > Imprimir gera o PDF do alcance
   escolhido num arquivo temporário e o envia à impressão do sistema. As
   opções são o mesmo alcance e cor/preto e branco, mais a escala: ajustar
   ao papel ou 1:1.
6. **Desempenho**: 200 páginas com conteúdo típico (moldura, ~30 símbolos e
   ~60 fios por página) exportam em menos de 10 s no Release. O teste
   automático usa um limite folgado em Debug.

## Critérios de aceitação

1. Projeto com 3 páginas (moldura com logotipo, símbolos, fios, textos
   livres, bobina e contato em páginas diferentes): o PDF gerado tem 3
   páginas do tamanho certo. A lista de exibição de cada página contém os
   itens esperados: textos especiais resolvidos, designações, número do
   fio e `/página.coluna`.
2. O PDF é estruturalmente válido:
   - é lido de volta por um leitor independente (dependência só de teste);
   - a tabela xref está correta;
   - as fontes e as imagens estão declaradas.
3. Links: o contato tem um link para a página e a posição da bobina; o
   espelho tem links para os contatos; as interrupções do mesmo grupo têm
   links entre si; um destino fora do alcance não gera link e aparece no
   relatório.
4. Os marcadores seguem a ordem documento → páginas, com os nomes certos.
5. Textos com acentos (ç, ã, é) saem corretos. Caractere fora da fonte é
   substituído e relatado (decisão 2).
6. Preto e branco: nenhuma cor diferente de preto ou cinza no conteúdo
   vetorial.
7. Com eventos egui reais: o diálogo Exportar PDF exporta a página ativa e
   o projeto e mostra o relatório; Imprimir monta o PDF temporário (o envio
   ao sistema fica isolado e substituído nos testes).
8. Desempenho do item 6 do escopo.
9. A tela continua igual: os testes de folha existentes passam sem mudança
   depois de a GUI passar a usar os textos de referência/espelho do Core.
10. Testes anteriores preservados; `scripts/validate.ps1` aprovado; Release
    compilado sem executar; relatório
    `docs/internal/reports/ETAPA_21_PDF_IMPRESSAO.md`, ADR-0023 e memória
    atualizados; commit + push somente após o OK expresso. Sem teste no
    executável nesta etapa; o próximo é ao fim da Etapa 22, com autorização
    do usuário para usar o computador.

## Decisões pedidas ao usuário

1. **Biblioteca de PDF**: `pdf-writer` (MIT/Apache-2.0, Rust puro, mantida
   pelo projeto Typst), que gera PDF de baixo nível sem dependências
   pesadas — **recomendado**. As alternativas são escrever um gerador
   mínimo próprio (mais código para manter) ou `printpdf` (mais alto nível,
   mais dependências). Para os testes, `lopdf` (MIT) como
   dependência só de teste, para ler o PDF de volta.
2. **Fonte**: Helvetica, uma das fontes padrão do PDF, com codificação
   WinAnsi. Cobre os acentos do português e símbolos como µ e °. Não exige
   embutir fonte e gera arquivos pequenos. Caracteres fora dela (por
   exemplo Ω, →) são substituídos e relatados — **recomendado**. A
   alternativa é embutir uma fonte TrueType com subconjunto: Unicode
   completo, mas mais dependências e arquivos maiores. Pode ficar para a
   Etapa 28.
3. **Impressão**: pelo PDF gerado, enviado à impressão do Windows
   (verbo "imprimir" do sistema, com o leitor de PDF padrão) —
   **recomendado**. A alternativa é a impressão nativa pelo GDI do
   Windows: muito mais código, e só para Windows.
4. **Lista de exibição**: criada no Core e usada pelo PDF (e pelo DXF na
   Etapa 22). A tela continua com o desenho atual do egui, mas os textos
   de referência e do espelho vêm do Core — **recomendado**. A alternativa
   é migrar agora todo o desenho da tela para a lista de exibição: tela e
   PDF ficariam idênticos por construção, com risco maior de regressões
   visuais sem teste no executável.
5. **Links**: referências cruzadas (principal/auxiliar, espelho,
   interrupções) e marcadores documento → página — **recomendado**. A
   alternativa é só os marcadores, deixando os links para depois.

## Fora do escopo

Exportação DXF/DWG da página (Etapa 22); relatórios e índice de páginas
(Etapa 23); PDF/A e assinatura digital; camadas PDF (OCG); fonte embutida
(decisão 2); impressão nativa (decisão 3); migrar o desenho da tela para a
lista de exibição (decisão 4).

## Instruções para o agente

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md` e
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory do projeto
   (páginas `notes/etapa-20-*` e, se existir, `notes/etapa-21-*`). ADRs
   úteis: 0015–0019 (molduras, símbolos) e 0022.
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). A Etapa 20
   está pronta e não commitada; o commit das Etapas 20 e 21 será um só ou
   dois em sequência, após o OK do usuário. O `docs.rar` da raiz é do
   usuário e nunca entra em commit.
3. Só começar depois de a aprovação estar registrada no topo deste arquivo.
   Se surgir dúvida fora das decisões, parar e perguntar ao usuário.
4. Criar `docs/internal/plans/<data>-etapa-21.md` com o desenho técnico, a seção
   "Retomada por outro agente" e a lista "Andamento" numerada; marcar cada
   item só com teste passando. Gravar no ai-memory a página fixada
   `notes/etapa-21-pdf-impressao.md` e atualizá-la ao final.
5. Seguir os padrões existentes:
   - o Core não depende de GUI; a lista de exibição e o escritor de PDF
     ficam em `cad-craft`;
   - novas dependências só as aprovadas na decisão 1, com versão exata no
     `Cargo.toml` e `Cargo.lock` atualizado;
   - a exportação longa roda fora da thread da interface e é cancelável;
   - o envio à impressora fica atrás de uma função substituível nos
     testes, que nunca chamam o sistema;
   - testes egui com accesskit (ver `cad-gui/tests/numbering_ui.rs`);
   - não remover nem afrouxar testes existentes.
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; compilar o
   Release sem executar o app. Se o disco encher, `cargo clean`.
7. Atualizar relatório, ADR-0023, `DECISIONS.md`, `ROADMAP.md` (linha 21),
   `KNOWN_ISSUES.md`, `CURRENT_STATE.md` e `HANDOFF.md`; informar que está
   pronto e aguardar o OK para versionar (commit + push em `main`, sem
   força). Não iniciar a Etapa 22.
