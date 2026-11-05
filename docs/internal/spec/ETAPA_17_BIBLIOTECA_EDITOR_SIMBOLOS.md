# CAD-CRAFT — Etapa 17 (Fase B): biblioteca externa e editor de símbolos

**Estado: aprovado pelo usuário em 2026-11-03 com as quatro decisões abaixo: (1) biblioteca em `.cadsym` por símbolo (ZIP, como o `.cadframe`); (2) escala padrão do `.elmt` = 0,5 mm por px, ajustável na importação; (3) "Atualizar símbolos do projeto" incluído nesta etapa; (4) elipse e estilos de traço fora das molduras (só símbolos e conteúdo livre).**

Plano: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase B). Base: commit
`4a32b54` (Etapas 15 e 16), 513 testes Rust + 15 PowerShell. Símbolos hoje:
ADR-0009 (definições vetoriais imutáveis, biblioteca embutida, portas
explícitas); editor de molduras reaproveitável: ADR-0016.

## Objetivo

Deixar o usuário criar, editar, guardar e trocar símbolos como no EPLAN:
símbolos com elipses, polígonos, estilos de traço e textos que mostram a
designação do dispositivo; uma biblioteca em pasta própria; um editor de
símbolos; e importação dos elementos `.elmt` do QElectroTech.

## Escopo

1. **Primitivas ampliadas** (símbolos e conteúdo livre da folha): elipse,
   polígono (fechado, com preenchimento opcional), estilo de traço
   (contínuo, tracejado, traço-ponto, pontilhado), espessura e
   preenchimento (nenhum/sólido) por primitiva. Desenho, hit test e DXF
   (exportação de moldura não muda) coerentes.
2. **Textos dinâmicos no símbolo**: campos posicionados no símbolo que
   mostram, na ocorrência, a designação do dispositivo (`-K1`), a função, o
   rótulo de cada ponto de conexão, a referência cruzada e propriedades do
   dispositivo por chave. Altura, alinhamento e rotação como nos textos de
   moldura.
3. **Editor de símbolos**: modo de edição como o de molduras (Etapa 14) com
   ferramentas linha, polilinha, retângulo, círculo, elipse, arco, polígono,
   texto, texto dinâmico e **porta** (posição, orientação 0/90/180/270,
   identificação). Propriedades da primitiva e do símbolo (nome, categoria,
   descrição). Novo, duplicar (inclusive a partir de um símbolo embutido),
   validação (portas únicas e dentro do símbolo, pelo menos uma porta para
   símbolos com função), Desfazer/Refazer.
4. **Biblioteca externa**: pasta configurável (como a de molduras) com
   arquivos `.cadsym` (ZIP com manifesto versionado e `symbol.json`, mesmo
   leitor defensivo do `.cadcraft`/`.cadframe`). O Centro de inserção mostra
   embutidos + biblioteca por categoria, com busca. Exportar símbolo do
   projeto para a biblioteca e importar da biblioteca para o projeto. O
   projeto continua autossuficiente (guarda as definições usadas).
5. **Versões e atualização**: editar um símbolo gera nova versão (mesmo ID,
   versão + 1). Comando "Atualizar símbolos do projeto" troca as ocorrências
   pela versão mais nova da biblioteca **somente quando as portas são
   compatíveis** (mesmos IDs); caso contrário, lista o que impede e não
   altera nada. Um passo de Desfazer.
6. **Importar `.elmt`** (QElectroTech): `line`, `rect`, `ellipse`, `circle`,
   `arc`, `polygon`, `text`, `dynamic_text` (rótulo → designação) e
   `terminal` (→ porta com orientação n/e/s/o); nomes multilíngues; escala
   px → mm configurável; importação de um arquivo ou de uma pasta inteira
   para a biblioteca; relatório do que foi ignorado/aproximado (como na
   Etapa 15). Os elementos do QElectroTech **não são versionados** no
   repositório e **não servem para treinar modelos de IA**; os testes leem
   `<pasta de amostras>\elmt` quando a pasta existe.
7. **Persistência**: `.cadcraft` formato 5 / `min_reader` 5 (primitivas e
   textos dinâmicos novos), migração 4→5 e fixture `etapa17-v5.cadcraft`;
   `.cadsym` formato 1.

## Critérios de aceitação

1. Com eventos egui reais: criar um símbolo do zero (ex.: bobina com
   retângulo, duas portas e texto dinâmico de designação), salvar na
   biblioteca, inserir com "Novo dispositivo" e ver `-K1` no texto dinâmico
   e nas portas ligadas a pontos novos.
2. Elipse, polígono preenchido e traços tracejado/traço-ponto aparecem na
   folha e no editor; hit test pelo traço concorda em dois zooms.
3. Editar o símbolo (versão + 1) e "Atualizar símbolos do projeto" troca as
   ocorrências com portas compatíveis; com porta removida, a atualização é
   recusada sem alterar nada; Desfazer/Refazer exatos.
4. `.cadsym` exportar/importar idêntico; arquivos inválidos recusados.
5. Importar `.elmt`: amostra de pelo menos 50 elementos da coleção de
   referência sem erro, com portas e textos convertidos; relatório coerente.
6. Salvar/reabrir; fixtures v0–v4 abrem e migram; v5 versionada.
7. Testes anteriores preservados; `scripts/validate.ps1`; Release; relatório,
   ADR e memória. Sem teste no app nesta etapa (próximo ao fim da Etapa 19,
   só com autorização para usar o computador). Commit + push após o OK.

## Decisões pedidas ao usuário

- **Arquivo da biblioteca**: `.cadsym` por símbolo (ZIP, como o
  `.cadframe`) — recomendado — ou uma pasta de JSON simples.
- **Escala padrão do `.elmt`**: 0,5 mm por px do QElectroTech (ajustável na
  importação) — confirmar ou indicar outra.
- **Atualizar ocorrências pela biblioteca** (item 5) nesta etapa —
  recomendado — ou adiar.
- **Estilos e elipse nas molduras** também? Proposta: não nesta etapa (só
  símbolos e conteúdo livre).

## Fora do escopo

Cadastro de peças e editor de funções (Etapa 18); macros (Etapa 19);
símbolos EPLAN (`.ema`/bibliotecas de símbolos EPLAN, Etapa 26); SVG;
símbolos paramétricos.

## Continuidade entre agentes

Se aprovado, este prompt e o estado da etapa ficam também no ai-memory
(página do projeto CAD-CRAFT) para que outro agente (ex.: Codex) retome de
onde parou: ler `AGENTS.md`, este arquivo, o plano
`docs/internal/plans/2026-11-03-etapa-17.md` (seção Andamento, marcada a cada passo
concluído), `HANDOFF.md` e `git status`.
