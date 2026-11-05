# CAD-CRAFT — Etapa 25 (Fase D): importação de projetos do QElectroTech (`.qet`/`.qetz`)

**Estado: aprovado pelo usuário em 2026-11-04 ("aprovo todas").** Decisões
aprovadas (todas as recomendadas): (1) só `.qet` nesta etapa, `.qetz`
pendente; (2) tamanho pela grade e escala do `.elmt`, moldura com zonas por
configuração; (3) mestre/escravos = um dispositivo; (4) número do fio
travado; (5) cria projeto novo.

**Uso do código-fonte do QElectroTech (autorizado pelo usuário em
2026-11-04):** consultar o código para entender o formato (significado dos
atributos, versões, geometria da borda). A implementação é **própria** em
Rust: não traduzir funções nem estruturas do código GPL. Tradução para outra
linguagem é obra derivada e sujeitaria o CAD-CRAFT à GPL.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase D, Etapa 25,
depende das Etapas 13, 17 e 18). Base: Etapa 22 em `0b3b65c`; Etapas 23 e 24
implementadas e validadas (630 Rust + 15 PowerShell), ainda não versionadas.

## Objetivo

Abrir no CAD-CRAFT um projeto do QElectroTech e convertê-lo num projeto
CAD-CRAFT editável:
- folhas com moldura e grade;
- símbolos embutidos;
- dispositivos, com as funções principal e auxiliares ligadas pelas
  referências cruzadas;
- fios com número, como conexões lógicas e rotas;
- relatórios de folha;
- textos e desenhos livres.

O que não tiver equivalente sai num relatório claro. Nenhum código do
QElectroTech é copiado: só o formato dos arquivos é lido.

## Amostras e licenças (fora do repositório)

- `<pasta de amostras>\qet`: **27 projetos `.qet`**
  das versões 0.80 a 0.200.1 do formato, com 1 a 50 folhas (por exemplo
  `industrial.qet`, com 50 folhas, e `741.qet`, com 1).
- **Nenhum `.qetz`** nas amostras.
- O mesmo diretório tem o código-fonte do QElectroTech (**GPL**): é
  referência de formato apenas e **não pode ser copiado nem traduzido**.
  `sources/container/QETZ-FORMAT.md` descreve o `.qetz`.
- Os elementos do QElectroTech são **CC-BY 3.0** (`ELEMENTS.LICENSE`):
  - a importação do projeto do próprio usuário é uso dele;
  - o relatório registra a origem dos símbolos importados;
  - pelo Plano Mestre, os elementos não servem de dados para treinar
    modelos.
- As amostras **nunca entram no Git**. Os testes automáticos usam projetos
  sintéticos gerados no próprio teste; há um teste opcional com as amostras
  por variável de ambiente.

## Formato observado (análise só para leitura, 2026-11-04)

- `.qet`: XML `<project title=… version=…>` com:
  - `<properties>`;
  - `<newdiagrams>` (padrões de borda, rótulos e condutores);
  - `<diagram …>`, uma folha cada, com `order`, `cols`/`rows`,
    `colsize`/`rowsize` (px), `height`, campos do carimbo (`title`,
    `author`, `folio`, `date`, `indexrev`…) e `titleblocktemplate`;
  - `<collection>`, com os elementos embutidos em categorias, no formato
    `.elmt`.
- Na folha:
  - `<element type="embed://import/…/x.elmt" uuid x y orientation(0–3)
    prefix …>`, com `<terminals>` (`<terminal id …>`),
    `<elementInformations>` (label, comment, function…) e `<dynamic>`
    (textos dinâmicos);
  - `<conductor terminal1 terminal2 num type …>` com `<segment>` (caminho);
  - `<inputs>` (textos livres), formas (`<line>`, `<polygon>`, `<circle>`),
    `<xrefs>` e `<report>` (relatório de folha).
- IDs de terminal numéricos nos formatos antigos e uuid nos novos.
- `.qetz` (formato 4 do QElectroTech): ZIP com `mimetype`, `manifest.xml`,
  `project.xml` com partes externas, **`project.sqlite`** (dados de
  engenharia), `folios/*.xml`, `elements/…`, `titleblocks/…` e
  `images/…`.

## O que já existe (não refazer)

- Importação `.elmt` → `SymbolDefinition`, com escala em mm por px e
  relatório (`interchange/elmt.rs`, Etapa 17, preferência
  `elmt_mm_per_px`).
- Importação `.titleblock` do QElectroTech → moldura (`interchange/
  titleblock.rs`, Etapa 15); molduras com zonas (Etapa 13).
- Dispositivos, funções principal e auxiliares, peças (Etapa 18);
  inserção com IDs novos e designação livre (Etapa 19).
- Conexões, rotas, junções, número do fio (Etapas 6–8 e 20); marcadores de
  interrupção com grupos (Etapa 13); conteúdo livre (Etapa 16).
- Leitor XML (`interchange/xml.rs`, `quick-xml`) e ZIP (`zip`).
- `.cadcraft`, recuperação e diálogos de arquivo (Etapa 12); verificações
  (Etapa 24), úteis para revisar o projeto importado.

## Escopo

1. **Leitor `.qet` (Core, sem GUI)**:
   - XML com limites de tamanho e de quantidade de elementos;
   - versões 0.80 até 0.200.1;
   - erro claro em XML malformado.
   - Resultado: `QetImport { project, report }`.
2. **Folhas e moldura** (decisão 2):
   - cada `<diagram>` vira uma `Page`, na ordem `order`, com o tamanho
     calculado da grade e do carimbo e convertido em mm pela escala do
     `.elmt`;
   - a grade (`cols`/`rows`, tamanhos e rótulos visíveis) vira uma moldura
     com zonas;
   - o carimbo usa o `titleblocktemplate` embutido, pelo importador da
     Etapa 15, quando existir;
   - os campos do carimbo viram metadados da página ou do projeto (textos
     especiais).
3. **Símbolos**:
   - cada elemento da `<collection>` vira uma `SymbolDefinition`, pelo
     importador `.elmt` da Etapa 17, com ID estável derivado do caminho
     `embed://…`;
   - as aproximações do `.elmt` (SYM-002) vão para o relatório.
4. **Ocorrências e dispositivos** (decisão 3):
   - cada `<element>` vira uma representação na posição e orientação certas
     (0/90/180/270);
   - a designação vem do rótulo (`label`) ou do prefixo, e um rótulo vazio
     recebe a próxima designação livre (relatado);
   - elementos ligados por referência cruzada (mestre e escravos) viram
     **um dispositivo** com a função principal (mestre) e funções
     auxiliares (escravos);
   - as informações (comment, function, fabricante…) viram propriedades
     técnicas.
5. **Fios** (decisão 4):
   - cada `<conductor>` liga os pontos dos dois terminais e vira uma
     `Connection` lógica e uma `ConnectionRoute` com os segmentos;
   - `num` vira o número do fio (travado, porque veio do projeto de origem);
   - condutor com terminal inexistente é relatado e descartado.
6. **Relatórios de folha e textos**:
   - `<report>` ligados entre folhas viram pares de interrupção num grupo;
   - `<inputs>` e as formas viram conteúdo livre.
7. **`.qetz`** (decisão 1): conforme a opção aprovada.
8. **GUI**:
   - Arquivo > Importar > Projeto QElectroTech;
   - prévia (folhas, símbolos, dispositivos, fios, avisos);
   - importar cria um **projeto novo** em memória, que você salva como
     `.cadcraft`;
   - roda em segundo plano, com progresso e cancelamento;
   - relatório final, com opção de abrir o painel Verificações (Etapa 24).
9. **Relatório de cobertura**: um teste opcional percorre as amostras e gera
   `docs/internal/reports/ETAPA_25_COBERTURA_QET.md`, só com contagens (folhas,
   símbolos, ocorrências, fios, itens não suportados por tipo), **sem
   conteúdo dos projetos**.

## Critérios de aceitação

1. Um projeto sintético de 2 folhas gerado no teste (moldura 17 × 8,
   bobina mestre na folha 1 e contato escravo na folha 2, fio numerado
   "12", relatório de folha entre as duas e texto livre) é importado com:
   - 2 páginas, com o tamanho e as zonas certas;
   - um dispositivo com função principal e auxiliar, e a referência
     `/2.x` funcionando (Etapa 13/20);
   - a conexão com número "12" travado e a rota pelos segmentos;
   - o par de interrupção agrupado;
   - o texto livre.
2. Orientações 0/90/180/270 e IDs de terminal numéricos (formato antigo) e
   uuid (formato novo) dão as portas nas posições esperadas.
3. XML malformado, terminal inexistente, elemento `embed://` ausente da
   coleção e arquivo acima do limite: recusados ou relatados, sem pânico.
4. O projeto importado passa no `validate()` e pode ser salvo e reaberto
   como `.cadcraft`; o PDF/DXF da página importada é gerado.
5. Teste opcional com as amostras: com `CAD_CRAFT_QET_SAMPLES` apontando
   para a pasta, os 27 `.qet` importam sem pânico, e a cobertura é gerada.
   Sem a variável, o teste é ignorado.
6. Com eventos egui reais: importar um `.qet` sintético pelo diálogo, ver
   a prévia, confirmar, navegar nas páginas importadas e verificar o
   projeto (Etapa 24).
7. Desempenho: o maior exemplo (`industrial.qet`, 50 folhas) importa em
   menos de 10 s no Release (medido pelo teste opcional).
8. Testes anteriores preservados; `scripts/validate.ps1`; Release
   compilado; relatório `docs/internal/reports/ETAPA_25_IMPORTACAO_QET.md`, ADR-0027
   e memória atualizados.
9. **Teste no executável das Etapas 23–25** ao fim desta etapa, somente com
   a autorização expressa do usuário para usar o computador:
   - gerar e atualizar relatórios;
   - exportar lista;
   - verificar o projeto e navegar com F8;
   - acoplar conectores;
   - importar um `.qet` real.

## Decisões pedidas ao usuário

1. **`.qetz`**:
   - **Recomendado:** só `.qet` nesta etapa. O `.qetz` guarda os dados de
     engenharia num banco SQLite, o que exige uma dependência nova (por
     exemplo `rusqlite` com o SQLite embutido), e não há nenhuma amostra
     `.qetz` para testar. Fica registrado como pendência para quando houver
     amostra.
   - Alternativa: ler o `.qetz` agora com `rusqlite`, com testes só em
     arquivos sintéticos.
2. **Folhas e moldura**:
   - **Recomendado:** o tamanho da página vem da grade e do carimbo do
     QElectroTech, convertido em mm pela escala do `.elmt` (preferência da
     Etapa 17). Uma moldura com zonas é criada para cada configuração de
     grade diferente, e o carimbo vem do `titleblocktemplate` embutido.
   - Alternativa: encaixar cada folha no formato ISO mais próximo (A4/A3),
     mudando a escala.
3. **Mestre e escravos** (referências cruzadas):
   - **Recomendado:** um dispositivo, com a função principal (mestre) e as
     auxiliares (escravos). É o modelo do CAD-CRAFT, e o espelho de
     contatos passa a funcionar.
   - Alternativa: um dispositivo por elemento, perdendo as referências.
4. **Número do fio importado**:
   - **Recomendado:** importado como **travado**, para a numeração
     automática não sobrescrever o que veio do projeto de origem.
   - Alternativa: importado sem trava.
5. **Destino**:
   - **Recomendado:** a importação cria um **projeto novo**, que você salva
     como `.cadcraft`.
   - Alternativa: acrescentar as folhas ao projeto aberto, com risco de
     conflito de designações e de moldura. Pode ficar para depois.

## Fora do escopo

Exportação para `.qet`; `.qetz` (se a decisão 1 for a recomendada);
bornes/réguas e cabos do QElectroTech 0.100+ (relatados como não suportados
nesta etapa); GRAFCET e elementos especiais sem equivalente; imagens nas
folhas (relatadas); sincronizar edições de volta para o QElectroTech.

## Instruções para o agente

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md` e
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory
   (`notes/etapa-24-verificacoes.md`). ADRs úteis: 0017 (molduras de outros
   sistemas), 0019 (símbolos e `.elmt`), 0020 (dispositivos/funções), 0022
   (numeração) e 0026 (verificações).
2. Git só leitura. As Etapas 23 e 24 estão prontas e ainda não versionadas;
   não misturar commits sem o OK do usuário. O `docs.rar`, o rascunho da 26
   e as amostras nunca entram no commit.
3. Só começar depois de a aprovação estar registrada no topo deste arquivo;
   dúvidas fora das decisões: parar e perguntar.
4. Criar `docs/internal/plans/<data>-etapa-25.md` com o desenho técnico, a seção
   "Retomada por outro agente" e a lista "Andamento". Gravar no ai-memory
   a página fixada `notes/etapa-25-importacao-qet.md`.
5. Padrões:
   - o Core não depende de GUI; o leitor fica em `interchange/qet.rs` (ou
     `interchange/qet/`), reaproveitando `elmt.rs`, `titleblock.rs` e
     `xml.rs`;
   - não copiar nem traduzir código do QElectroTech;
   - entrada não confiável com limites;
   - tudo o que não for convertido vai para o relatório, nada é inventado;
   - o projeto resultante é validado inteiro;
   - testes com projetos sintéticos; amostras só no teste opcional;
   - testes egui com accesskit (ajudante que ignora rótulos e títulos de
     janela, como em `cad-gui/tests/checks_ui.rs`);
   - não remover nem afrouxar testes.
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; compilar o
   Release. Rodar o teste opcional com
   `CAD_CRAFT_QET_SAMPLES=<pasta de amostras>\qet`.
   O teste no executável só acontece com a autorização expressa.
7. Atualizar relatório, ADR-0027, `DECISIONS.md`, `ROADMAP.md` (linha 25),
   `KNOWN_ISSUES.md`, `CURRENT_STATE.md` e `HANDOFF.md`; informar e
   aguardar o OK para versionar. Não iniciar a Etapa 26.
