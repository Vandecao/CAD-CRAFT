# CAD-CRAFT — Etapa 32: conexões e edição confiáveis

**Estado: APROVADO pelo autor em 2026-11-07** (plano `docs/internal/plans/PLANO_ETAPAS_32_35.md`, itens 1–4 e 10 do diagnóstico).

- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE; código em `cadcraft-model/src/domain/electrical/connection_drawing.rs`, `domain/project/electrical/connection_drawing.rs`, `cadcraft-gui/src/connection_view.rs`, `connection_editor.rs`, `connection_geometry.rs`, `panels/properties/`, `symbol_editor/`.
- **Pré-requisito:** as duas correções do editor de moldura feitas depois da Etapa 31 já versionadas.

## Problemas (teste da partida estrela-triângulo)

1. "Inserir junção" num ponto em T, onde uma rota termina sobre outra, falha com "desenho de conexão inconsistente ou referenciado". Causa provável: `point_on_segment` compara com `==`; as rotas que saem de portas usam posições calculadas por transformação (f64) e a junção usa o ponto da grade.
2. Logo depois de escolher "Inserir conexão", o primeiro clique numa porta às vezes falha ("inicie em uma porta elétrica associada"), e um canto pode ser perdido, gerando uma rota em L que passa por cima de outro fio.
3. Diálogos nativos (salvar, confirmar substituição) abrem atrás do app e bloqueiam a tela sem aviso.
4. Depois de Enter numa propriedade de página, o inspetor pula para a página da aba ativa; X/Y/ângulo de um símbolo só valem com "Aplicar transformação", no fim do painel.
5. Editor de símbolos: a categoria digitada não foi gravada num símbolo (salvo como "Usuário"); a identificação de uma porta não mudou na primeira tentativa.

## Requisitos

1. **Junção em T:** comparações de pontos em rotas com a tolerância do domínio (`EPS_MM`); o ponto da junção é encaixado na rota existente. A junção é aceita onde uma rota termina num ponto interno de outra e onde duas rotas se cruzam com conexão desejada. Funciona com símbolos girados e espelhados.
2. **Clique confiável:** porta ou canto encontrados no mesmo quadro em que a ferramenta foi escolhida (cena sincronizada antes do encaixe). O canto clicado nunca é descartado em silêncio.
3. **Diálogos nativos à frente:** janela do CAD-CRAFT como mãe; enquanto um diálogo estiver aberto, a área de trabalho mostra "Aguardando o diálogo do sistema".
4. **Inspetor estável:** a página ou objeto selecionado continua no inspetor depois de cada edição. Enter em X, Y ou ângulo aplica a transformação, com um passo de Ctrl+Z; o botão continua.
5. **Editor de símbolos:** nome, categoria e campos de porta gravados também quando o foco sai do campo (Tab, clique fora, salvar).

## Testes

- Junção em T com portas de símbolos transladados, girados (90/180/270) e espelhados; rede unificada e um Ctrl+Z desfaz.
- Interface: escolher "Inserir conexão" e clicar na porta sem quadro de hover; rota com dois cantos sai exatamente como clicada.
- Inspetor: editar nome e identificação de uma página não ativa altera só ela.
- X/Y com Enter move o símbolo; Ctrl+Z restaura.
- Editor de símbolos: categoria e identificação de porta gravadas após Tab e após clicar em "Salvar na biblioteca".

## Entrega

- `validate.ps1`, Release, relatório `docs/internal/reports/ETAPA_32_CONEXOES_EDICAO.md`, HANDOFF, CURRENT_STATE e ROADMAP. Commit + push só com o OK do autor.
