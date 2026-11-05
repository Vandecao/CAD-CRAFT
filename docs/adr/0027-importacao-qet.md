# ADR-0027 — Importação de projetos QElectroTech (`.qet`)

Data: 2026-11-04. Estado: implementado e testado automaticamente na Etapa 25; homologação na Etapa 28.

## Contexto

A Etapa 25 importa projetos do QElectroTech. O prompt `docs/internal/spec/ETAPA_25_IMPORTACAO_QET.md` foi aprovado com cinco decisões:
1. só `.qet`;
2. folha pela grade e pelo carimbo;
3. mestre e escravos num dispositivo;
4. número de fio travado;
5. projeto novo.

O usuário autorizou consultar o código-fonte do QElectroTech, que é GPL. Ele foi lido só para entender o formato. A implementação é própria, sem cópia nem tradução de funções.

## Decisão

- **Módulo** `cad-craft/src/interchange/qet.rs`:
  - entrada: `import(bytes, &QetOptions)`;
  - saída: `QetImport { project, report, stats }`;
  - usa o leitor XML existente, sem DTD e sem entidades externas;
  - limites: 64 MB e 500.000 nós.
- **Escala:** a do `.elmt` (preferência, padrão 0,5 mm/px). Y é invertido: a origem QET fica no topo da cena; a do CAD-CRAFT, na base da folha.
- **Folha:**
  - largura = 2·5 + 20 + colunas × largura da coluna;
  - altura = 2·5 + 20 + linhas × altura da linha + altura do carimbo;
  - a altura do carimbo é a soma das linhas da grade do modelo embutido; sem modelo embutido, 50 px.
- **Moldura:** uma por combinação de grade, flags de cabeçalho e modelo.
  - Zonas: colunas numeradas a partir de 1, linhas com letras a partir de A.
  - Carimbo: `titleblock::import_template` (refatorado da Etapa 15), entrando por `frame_import_edits` para levar as imagens.
  - Modelo ausente: carimbo simples com nome do projeto, nome da folha e número, registrado no relatório.
  - Autor, data e revisão da folha viram metadados da página.
- **Símbolos:** cada elemento embutido em `<collection>` passa por `elmt::import_definition` (refatorado da Etapa 17). A chave é o caminho `embed://…` da coleção.
- **Ocorrências:**
  - ponto de encaixe = `x`, `y` do elemento;
  - rotação CAD-CRAFT (anti-horária) = (4 − orientação QET) × 90°, porque a orientação QET gira no sentido horário da tela;
  - a porta é escolhida pela posição local do terminal; se a posição não casar, pela ordem.
- **Dispositivos:**
  - mestres primeiro;
  - escravos entram no dispositivo do mestre ligado por `links_uuids`;
  - bobina mestre = função principal; escravo `power` = contato principal; os outros escravos são contatos auxiliares;
  - designação pelo rótulo (a parte depois do último `-`);
  - rótulo repetido fora de mestre/escravo (bornes, componente em várias folhas) = mesmo dispositivo, relatado;
  - mestre repetido ou rótulo inválido recebem a próxima designação livre, relatado.
- **Desenhos sem terminais** (plano de montagem, página de guarda): ocorrência de símbolo sem alvo e sem dispositivo.
- **Condutores:**
  - os terminais são resolvidos por ID de terminal da folha (formato antigo) ou por UUID do elemento + UUID do terminal da definição (QET 0.9+);
  - o percurso vem dos segmentos, simplificado e fechado ortogonalmente;
  - se o percurso for inválido, a rota é refeita em L e relatada;
  - `num` vira número de fio travado.
- **Rota exata:** nova edição do Core `ConnectionDrawingEdit::CreateRouteExact`.
  - Cria a rota com a geometria dada, sem o desvio automático de símbolos do desenho interativo. Rota sem desvio automático já era um estado válido e persistido.
  - Sem ela, cerca de 27% dos condutores de `industrial.qet` eram recusados, porque o percurso original passa sobre corpos de símbolos.
  - O desenho interativo não muda.
- **Relatórios de folha** (`next_report`/`previous_report`):
  - viram marcadores de interrupção;
  - a porta única "p" vai para a origem, com o desenho deslocado e a posição compensada;
  - o ponto é o do outro lado do condutor ligado, e esse condutor não vira fio;
  - os pares por `links_uuids` formam um grupo em estrela sem continuidade.
- **Textos** (`<inputs>`): viram texto livre sem HTML.
- **Relatório:** imagens, formas, elementos fora da coleção e o que não converter vão para o relatório; não há perda silenciosa.
- **GUI:** Arquivo > Importar > Projeto QElectroTech (`qet_import_ui.rs`).
  - Procurar e Analisar, em segundo plano.
  - Prévia com contagens e relatório.
  - Criar projeto importado: substitui a sessão, pedindo confirmação se houver alterações; abre a 1ª folha; o projeto fica pendente de salvar.
  - Opção de abrir as Verificações em seguida.

## Consequências

- O formato `.cadcraft` não muda (continua 9): o projeto importado é um projeto comum.
- `.qetz` (SQLite) fica pendente (QET-L01), porque não há amostras e exigiria uma dependência nova.
- Mapeamentos aproximados ficam no `KNOWN_ISSUES` (QET-L02 a QET-L06).

## Adendo — teste no app (2026-11-05)

- **Rótulo da ocorrência:** no QElectroTech 0.7 e posteriores, o rótulo fica na ocorrência (`dynamic_texts` e `texts_groups`). Quando a definição não tem texto de designação, ela recebe um texto dinâmico `Designation` na posição da 1ª ocorrência, que acompanha renumerações.
- **Textos de usuário da ocorrência:** viram texto livre na posição transformada da ocorrência.
- **Traços dos relatórios de folha:** o condutor ligado a um relatório de folha continua sendo só o ponto elétrico da interrupção, mas o traço é mantido como linhas livres, sem efeito elétrico.
- **Símbolo do marcador:** perde o texto de designação, porque sem dispositivo mostraria "-K?".
