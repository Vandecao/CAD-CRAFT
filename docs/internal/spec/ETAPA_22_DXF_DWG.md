# CAD-CRAFT — Etapa 22 (Fase C): exportação DXF e DWG

**Estado: aprovado pelo usuário em 2026-11-04 ("Aprovo todas").**
Decisões aprovadas (todas as recomendadas): (1) DXF R12 com o escritor
próprio ampliado; (2) ANSI 1252 com `\U+XXXX` para o resto; (3) um DXF por
página numa pasta; (4) DWG por conversor externo opcional (ODA File
Converter configurado pelo usuário); (5) geometria explodida por camadas.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase C, Etapa 22,
depende da lista de exibição da 21; risco 2: "DWG: biblioteca comercial (ODA)
ou conversor externo; DXF primeiro"). Base: Etapas 20 e 21 versionadas em
`1fe3659` no `origin/main` (586 Rust + 15 PowerShell).

## Objetivo

Exportar as páginas do projeto para DXF, que abre no AutoCAD, no LibreCAD, no
BricsCAD e no EPLAN, com a mesma geometria do PDF. A exportação DWG fica
disponível conforme a decisão de licença. É o equivalente a "Exportar
DXF/DWG" do EPLAN, sem copiar código ou dados de terceiros.

## O que já existe (não refazer)

- Lista de exibição da página no Core (`cad-craft/src/display.rs`,
  `Project::page_display_list`): caminhos com espessura e tracejado, curvas
  tesselizadas, textos com matriz caixa → página, imagens e links, com cores
  lógicas. A Etapa 21 a criou para servir também a esta etapa.
- Exportação DXF R12 **de moldura** (`interchange/dxf.rs::export`, Etapa 15):
  escritor próprio, com a espessura no nome da camada e sem imagens.
- Alcance de páginas e relatório no PDF (`pdf.rs::pages_in_scope`,
  `PdfReport`), seletor de arquivo (`FilePicker`), diálogo com thread,
  progresso e cancelamento (`cad-gui/src/pdf_export.rs`) e `SystemShell`
  substituível nos testes.
- Métricas Helvetica (`font.rs`) para a largura dos textos.

## Escopo

1. **Escritor DXF de página (Core, sem GUI)**, decisões 1 e 2.
   `export_dxf(project, page, options) -> DxfOutput { bytes, report }`, feito
   a partir da lista de exibição:
   - unidades em mm, com `$INSUNITS = 4`; limites e extensão da folha;
   - linhas, polilinhas e polígonos preenchidos (`SOLID`/hachura simples);
   - textos com posição, altura, rotação e escala horizontal (largura
     forçada); texto espelhado mantém a geometria;
   - camadas por categoria: moldura, símbolos, fios, textos, referências,
     junções e guias, cada uma com cor ACI;
   - espessura pela camada (`CAD-CRAFT 0.35mm`, como na Etapa 15) e tipos
     de linha para o tracejado;
   - imagens viram um retângulo marcado e são relatadas, porque o DXF R12
     não tem a entidade `IMAGE`.
2. **Várias páginas** (decisão 3): um arquivo `.dxf` por página na pasta
   escolhida, com nome `<projeto>_<documento>_<página>.dxf` sanitizado, ou
   uma única página quando o alcance for a página ativa.
3. **DWG** (decisão 4): opção nas Configurações para indicar o conversor
   externo instalado pelo usuário. Com ele configurado, "Exportar DWG" gera
   o DXF numa pasta temporária, chama o conversor por linha de comando
   (sem shell, com tempo limite) e copia os `.dwg` para o destino. Sem
   conversor, o comando fica desabilitado e explica o motivo. Nenhum
   binário de terceiros entra no repositório nem é baixado pelo app.
4. **GUI**:
   - Arquivo > Exportar DXF e Arquivo > Exportar DWG, no mesmo modelo do
     diálogo de PDF: alcance (projeto, documento, página ativa, páginas
     escolhidas), pasta de destino, progresso, cancelamento, relatório e
     "abrir pasta ao terminar";
   - versão DWG de saída (2000/2007/2013/2018), quando houver conversor;
   - reaproveitar o diálogo de PDF, com abas ou modo.
5. **Desempenho**: 200 páginas típicas em DXF em menos de 10 s no Release.
6. **Teste no executável das Etapas 20 a 22** (cadência a cada 3 etapas),
   ao fim desta etapa e **somente com a autorização expressa do usuário**
   para usar o computador:
   - numerar dispositivos, fios e páginas e conferir Desfazer/Refazer;
   - ver o espelho de contatos e os números dos fios na folha;
   - exportar PDF e abrir no leitor; conferir links e marcadores;
   - exportar DXF e conferir que o arquivo existe e o tamanho; abrir num
     visualizador só se houver um instalado;
   - imprimir só se o usuário quiser.

## Critérios de aceitação

1. Projeto de 3 páginas (o mesmo cenário dos testes da Etapa 21): cada
   página gera um DXF lido de volta por um leitor independente de códigos
   de grupo (no teste) e pelo importador de molduras da Etapa 15, com:
   - as entidades esperadas: moldura, símbolos, fios, textos especiais
     resolvidos, designações, número do fio e `/página.coluna`;
   - as camadas por categoria e a extensão igual à folha.
2. Geometria fiel: os pontos e textos do DXF coincidem com a lista de
   exibição, com tolerância de 0,001 mm; arcos e círculos com o mesmo
   raio e centro (pontos de tesselagem).
3. Acentos (ç, ã, é) corretos em ANSI 1252; caracteres fora dessa
   codificação saem como `\U+XXXX` e são relatados (decisão 2).
4. Exportar o projeto grava N arquivos com nomes únicos e seguros. O
   nome repetido de página ou caractere inválido é tratado.
5. DWG: com um conversor falso (script de teste), o comando monta a
   chamada certa, respeita o tempo limite, relata a falha do conversor e
   não deixa temporários. Sem conversor, o comando fica desabilitado com
   explicação.
6. Com eventos egui reais: o diálogo exporta a página ativa e o projeto em
   DXF e mostra o relatório; "Exportar DWG" desabilitado sem conversor e
   funcional com o conversor falso.
7. Desempenho do item 5 do escopo.
8. Testes anteriores preservados; `scripts/validate.ps1` aprovado; Release
   compilado; relatório `docs/internal/reports/ETAPA_22_DXF_DWG.md`, ADR-0024 e
   memória atualizados; teste no executável (item 6 do escopo) com
   autorização; commit + push somente após o OK expresso.

## Decisões pedidas ao usuário

1. **Versão do DXF**: R12 (AC1009), com o mesmo escritor próprio da Etapa
   15, ampliado. É a versão lida por praticamente todos os programas e não
   exige *handles* nem tabelas extensas — **recomendado**. A alternativa é
   R2000 (AC1015), que tem espessura real por entidade e cor verdadeira,
   com bem mais estrutura obrigatória e mais risco de incompatibilidade.
2. **Textos**: codificação ANSI 1252 (`$DWGCODEPAGE`), com os acentos do
   português diretos e os outros caracteres como `\U+XXXX` — **recomendado**.
3. **Várias páginas**: um DXF por página, numa pasta — **recomendado**. A
   alternativa é um único DXF com as páginas lado a lado, mais difícil de
   usar em outros programas.
4. **DWG**: conversor externo **opcional**, instalado e configurado pelo
   usuário. O padrão é o *ODA File Converter*, gratuito mas de licença
   proprietária da Open Design Alliance; o CAD-CRAFT apenas o chama, sem
   distribuí-lo nem baixá-lo — **recomendado**. As alternativas são:
   - não ter DWG por enquanto (só DXF, que o AutoCAD abre);
   - LibreDWG, que é GPLv3 e tem a escrita de DWG ainda experimental;
   - a biblioteca comercial da ODA embutida, que exige assinatura paga.
5. **Símbolos no DXF**: geometria explodida, separada por camadas, igual ao
   PDF — **recomendado** nesta etapa. A alternativa é `BLOCK`/`INSERT` com
   atributos (designação, função), mais editável no AutoCAD mas com mais
   código; pode ficar para a Etapa 26 (EPLAN).

## Fora do escopo

Importação DXF/DWG de esquemas (só molduras, Etapa 15); DXF R2000+; blocos
com atributos (decisão 5); imagens no DXF (`IMAGE` exige R14+); camadas
PDF; relatórios (Etapa 23); troca de dados com o EPLAN (Etapa 26).

## Instruções para o agente

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md` e
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory
   (`notes/etapa-21-pdf-impressao.md`). ADRs úteis: 0017 (DXF de
   moldura) e 0023 (lista de exibição).
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). Base
   `1fe3659`; o `docs.rar` é do usuário e nunca entra em commit.
3. Só começar depois de a aprovação estar registrada no topo deste arquivo;
   dúvidas fora das decisões: perguntar ao usuário.
4. Criar `docs/internal/plans/<data>-etapa-22.md` com o desenho técnico, a seção
   "Retomada por outro agente" e a lista "Andamento"; marcar cada item só
   com teste passando. Gravar no ai-memory a página fixada
   `notes/etapa-22-dxf-dwg.md`.
5. Padrões:
   - o Core não depende de GUI;
   - nenhuma dependência nova sem aprovação; o leitor independente do teste
     é escrito no próprio teste;
   - o conversor DWG é chamado por uma função substituível, com argumentos
     separados (nunca uma linha de shell montada com o caminho) e com
     tempo limite;
   - testes egui com accesskit (ver `cad-gui/tests/pdf_ui.rs`);
   - não remover nem afrouxar testes.
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; compilar o
   Release. O teste no executável (item 6 do escopo) só acontece com a
   autorização expressa do usuário para usar o computador.
7. Atualizar relatório, ADR-0024, `DECISIONS.md`, `ROADMAP.md` (linha 22),
   `KNOWN_ISSUES.md`, `CURRENT_STATE.md` e `HANDOFF.md`; informar e
   aguardar o OK para versionar. Não iniciar a Etapa 23.
