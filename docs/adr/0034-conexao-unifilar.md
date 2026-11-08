# ADR-0034 — Conexão unifilar e bornes de régua pelo símbolo (formato 11)

Data: 2026-11-08. Estado: aceito na Etapa 35, conforme o plano `docs/internal/plans/PLANO_ETAPAS_32_35.md` aprovado pelo autor (decisão 3).

## Contexto

No teste da partida estrela-triângulo, o unifilar foi desenhado com linhas gráficas: ligar símbolos com a ferramenta de conexão criaria conexões elétricas e redes, duplicando o multifilar. O símbolo "Borne" criava dispositivos X1, X2…, e a lista de bornes, que lê bornes de régua, saía vazia.

## Decisão

- **`SingleLineLink`**: nova entidade do projeto com página, representação de origem, representação de destino, pontos da polilinha e número de fases (1 a 9).
  - É só documentação: não cria conexões nem redes, não entra na lista de conexões e não recebe número de fio.
  - Na lista de exibição (tela, PDF e DXF) aparece como polilinha de 0,5 mm com traços oblíquos de fase no meio do trecho mais longo. De 1 a 4 fases há um traço por fase; acima disso, um traço e o número.
  - A integridade exige que as duas representações existam na mesma página, então uma representação ligada não pode ser removida antes da ligação.
- **Ferramenta "Conexão unifilar"** (aba Conexões): clique no símbolo de origem, nos cantos e no símbolo de destino. As teclas 1–9 escolhem as fases; Delete sobre a linha a remove. Criar ou remover é um passo de Ctrl+Z.
- **Formato `.cadcraft` 11** (`min_reader` 11). O campo `single_lines` é opcional e omitido quando vazio, e a migração v10 → v11 não converte conteúdo.
- **Bornes pelo símbolo**: com a associação "Borne de régua", `Project::place_strip_terminal` faz tudo numa operação reversível:
  - usa ou cria a régua (X1) e o dispositivo dela (-X1);
  - acrescenta o próximo borne (X1:1, X1:2…);
  - cria um ponto de conexão ligado ao borne, vinculado a todas as portas do símbolo, para que os fios dos dois lados formem a mesma rede.

  A ocorrência aponta para o borne, mostra "-X1:n" e aparece na lista de bornes. Os bornes não precisam de formato novo, porque réguas e bornes já existiam.

## Consequências

- Arquivos gravados no formato 11 não abrem em versões anteriores do CAD-CRAFT.
- As saídas de referência `.cadcraft` foram regravadas só por causa do número de formato no manifesto. PDF, DXF e listas ficaram idênticos.
- Fixture nova: `etapa35-v11.cadcraft` (página unifilar com Q1 e K1 ligados por uma linha trifásica).
