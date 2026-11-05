# CAD-CRAFT — Etapa 13 (Fase A): modelo de moldura, zonas e referências

**Estado: aprovado pelo usuário em 2026-11-03 ("Aprovar Etapa 13") e implementado.**

Plano: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md`. Base: commit `6612e82`
(Etapa 12), 452 testes Rust + 15 PowerShell.

## Objetivo

Dar às páginas uma moldura com carimbo e divisão em colunas e linhas, como no
EPLAN, e passar as referências cruzadas a usar essa divisão. O editor gráfico
de molduras é a Etapa 14; a importação de DXF/EPLAN/QElectroTech é a 15.

## Escopo

1. **Modelo `FrameTemplate` no Core** (milímetros, origem inferior esquerda):
   - tamanho da folha e área útil (margens);
   - zonas: quantidade e largura de colunas, quantidade e altura de linhas,
     rótulos configuráveis (números/letras, início, sentido), lados em que
     aparecem;
   - elementos: linhas, retângulos, círculos, arcos (com espessura de traço),
     textos fixos e **textos especiais** (propriedade do projeto, da página ou
     da moldura, ex.: nome do projeto, número da página, total de páginas,
     data, autor, revisão), com altura, alinhamento e rotação;
   - textos especiais com **índice** (ex.: linhas 1..n do quadro de revisões:
     data, alteração e nome da revisão n), como no EPLAN;
   - textos fixos com variantes por idioma (pt-BR obrigatório; demais
     opcionais), preservadas para a importação da Etapa 15;
   - rótulo de coluna/linha como texto especial "número da zona", gerado
     automaticamente a partir da configuração das zonas;
   - identidade (UUID v7), nome, versão, descrição.
2. **Atribuição**: moldura padrão do projeto e substituição opcional por
   página; mudanças reversíveis no histórico.
3. **Propriedades de projeto e de página** usadas pelos textos especiais,
   editáveis no painel Propriedades (as já existentes mais as novas, como autor,
   data, revisão, cliente, desenhista).
4. **Desenho na folha**: moldura, rótulos de zona e textos especiais com
   valores reais; a área útil passa a ser a referência da grade de zonas.
5. **Referências cruzadas por zona**: a posição de uma ocorrência é
   convertida em página.coluna (formato configurável, ex.: `/3.4` ou `3.B`).
   Navegador, Propriedades e marcadores de interrupção passam a mostrar esse
   formato. Sem moldura, mantém o comportamento atual (página + coordenadas).
6. **Formatos de folha**: A0–A4 em retrato/paisagem e personalizado, na criação
   e nas propriedades da página.
7. **Molduras de fábrica**: pelo menos uma moldura original CAD-CRAFT para
   A3 paisagem e uma para A4 retrato, criadas do zero (sem copiar EPLAN nem
   QElectroTech).
8. **Persistência**: molduras salvas no `.cadcraft`, com migração dos arquivos
   v1 da Etapa 12 (se o formato do `project.json` mudar, elevar `min_reader`,
   conforme ADR-0014).

## Critérios de aceitação

1. Projeto novo recebe a moldura A3 de fábrica; página nova a herda; uma página
   pode usar outra moldura; Desfazer/Refazer restauram exatamente.
2. Alterar o nome do projeto ou o número da página atualiza os textos especiais
   na folha sem editar a moldura.
3. A referência de uma ocorrência mostra a coluna correta em zonas com
   larguras iguais e diferentes, inclusive nas bordas (testes de geometria).
4. Mudar a moldura da página recalcula as referências exibidas.
5. Salvar/reabrir preserva molduras e atribuições (igualdade do agregado);
   arquivo v1 da Etapa 12 abre e migra; fixtures versionadas.
6. Fluxo com eventos egui reais: criar projeto, páginas A3/A4, inserir
   bobina e contato em páginas diferentes, verificar a referência por coluna
   no painel e na folha.
7. Testes anteriores preservados; `scripts/validate.ps1` aprovado; Release;
   relatório, ADR e memória; commit + push somente após o OK do usuário.

## Fora do escopo

Editor gráfico de molduras e imagens/logotipos (Etapa 14); DXF, `.fn1`,
`.titleblock` (Etapa 15); lista de contatos ao lado da bobina (Etapa 20);
PDF (Etapa 21).

## Referência observada no EPLAN 2027 (2026-11-03)

Moldura `<pasta de molduras do EPLAN>\FN1_00A1.fn1`
do usuário, salva após edição (68.842 bytes, XML PXF 1.23), e o editor de
molduras aberto no EPLAN Education 2027. O modelo da Etapa 13 deve conseguir
representar o que foi encontrado, para a importação da Etapa 15:

| Objeto PXF | Interpretação | Atributos observados |
|---|---|---|
| `O314` (1) | retângulo da borda | cantos `A3319`/`A3321` (−177/−109 a 243/188 → 420 × 297 mm, A3), traço `A845=0.35` |
| `O198` (29) | linhas do carimbo e das zonas | início/fim `A1079`/`A1081`, traço `A845=0.25` |
| `O196` (18) | textos fixos multilíngues | posição `A1019`, texto `A1039="pt_BR@Folha;en_US@Page;…"`, alinhamento `A1029` |
| `O242` (39) | textos especiais | posição `A1019`, propriedade `A1559` (ex.: 11070/11071/11074 com índice `A1561` = 1..3 → quadro de revisões), formato `S244x1027` (altura `A1939` 1,5–2,5 mm) |
| `O242` com `A1559=19005` (10) | rótulos de coluna | um por coluna em y = 184, a cada 42 mm → 10 colunas de 42 mm |

- A origem da moldura no EPLAN não é o canto inferior esquerdo (borda
  começa em −177/−109): a importação deve transladar para a origem do CAD-CRAFT.
- As propriedades de moldura (`O178/P39`, ex.: `P12059=24`, `P12060=70`)
  ainda não foram identificadas; confirmar no diálogo de propriedades da
  moldura do EPLAN (a tela estava parcialmente coberta durante a análise).
- Os números de propriedade (`A1559`) não têm documentação pública; o
  mapeamento para textos especiais do CAD-CRAFT será uma tabela verificada
  com molduras reais na Etapa 15.
- O editor do EPLAN permite inserir gráfico, cota, texto, arquivo de imagem,
  hiperlink e código QR; a moldura analisada não tem imagem. Imagens/logotipos
  entram na Etapa 14; hiperlink e QR ficam como itens a decidir.
- Guia Editar → grupo Moldura: "Texto especial" (diálogo "Propriedades (texto
  especial): Propriedades das molduras", abas Posicionamento e Formato, campo
  Propriedade com seletor) e "Outros": encaixar/desencaixar texto, novo
  posicionamento dos textos da coluna e da linha, **inserir texto da coluna**,
  **inserir texto da linha**, inserir marca d'água, propriedades em números.
  Logo, o EPLAN trata rótulos de zona como textos de coluna/linha posicionados
  pela moldura; o CAD-CRAFT deve aceitar rótulos de colunas **e de linhas** e a
  marca d'água como texto da moldura (a marca d'água pode ficar para a 14).
- A pasta `<pasta de molduras do EPLAN>` tem outras
  molduras (`FN1_001…016`, `FN1_FT001…007`, `FN1_PP001…`), úteis como amostras
  de leitura na Etapa 15.

## Pendente para a Etapa 15

Uma moldura do EPLAN com logotipo e, se possível, a mesma moldura exportada
em DXF, para comparar geometria e mapear imagens.
