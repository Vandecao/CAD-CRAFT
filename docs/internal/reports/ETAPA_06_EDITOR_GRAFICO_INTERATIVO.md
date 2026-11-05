# Etapa 6 — Editor gráfico interativo

Referência: 2026-10-28, America/Sao_Paulo.
Estado: implementada, validada automaticamente e pronta para revisão. Homologação manual
da Etapa 6 pendente. Etapa 7 não iniciada.

## 1. Autorização, base e resumo executivo

Executado o anexo integral em `docs/internal/spec/ETAPA_06_EDITOR_GRAFICO_INTERATIVO.md`.
Plano: `docs/internal/plans/2026-10-28-etapa-06-editor.md`. Decisão: ADR-0008.
O roadmap anterior não definia uma divisão concorrente entre Etapas 6 e 7.
O novo anexo confirma a homologação visual da Etapa 5 pelo usuário. O Git confirma
seu commit, corrigindo a memória anterior que ainda a descrevia como não commitada.

Branch `main`; origin preservado `https://github.com/Vandecao/CAD-CRAFT.git`.
HEAD observado por `git rev-parse HEAD` em 2026-10-28:
`5a5526d4ef4b093222218a9194450a3974225f66` —
`feat: implement egui desktop interface and visual refinements`.
`git ls-remote --exit-code origin refs/heads/main` retornou o mesmo SHA nessa data.
Árvore limpa no início. Consulta remota sem fetch/sincronização.
Esta entrega está somente na árvore de trabalho e não integra esse HEAD.

O viewport permite selecionar, mover e transformar as representações existentes,
com prévia transitória e confirmação atômica reversível. O domínio elétrico permanece
no Core existente. Não foram criados símbolos IEC, roteamento nem conexões implícitas.
Sem dependências novas, alteração de schema ou formato `.cadcraft` definitivo.

## 2. Arquitetura da interação e seleção

`Session` continua proprietária de um único Project privado. `Interaction` mantém
ferramenta Select/Move, `SelectionState` com BTreeSet de RepresentationId e primário,
hover separado, Gesture e DragSession. Abas/câmeras são independentes.
`selection_revision` invalida rascunhos do inspetor após mudar seleção/página,
inclusive quando o painel não observou os estados intermediários.

Clique simples seleciona um objeto; vazio limpa seleção gráfica e mostra página.
Ctrl+clique adiciona/remove sem duplicar IDs. Arrastar objeto já selecionado mantém
o conjunto; clique sem arraste volta à seleção simples. Arraste no vazio mostra
janela azul (esquerda→direita, contenção) ou verde (direita→esquerda, cruzamento).
Esc cancela a janela conservando a seleção anterior. Troca de página limpa seleção
gráfica e prévia, preservando Project, histórico e câmera de cada aba.

## 3. Hit testing, desenho e cursores

Consulta restrita à página pelo índice do Core; invisíveis são ignorados.
Cursor é convertido para mm e depois para espaço local pela inversa da transformação.
Tolerância: 6 pontos lógicos de tela, divididos pela escala da câmera; dimensões
degeneradas e âncoras continuam selecionáveis. Testes incluem escala lógica/DPI,
zoom, pan, rotação, espelhos e área vazia da AABB de um retângulo rotacionado.
Sobreposição usa ordem determinística `(layer, RepresentationId)`, maior no topo.
`hit_candidates` retorna lista ordenada, preparando cycling futuro sem implementá-lo.

Janela cruzada testa geometria transformada, interseção de segmentos e contenção
da janela pelo objeto. Não usa somente a AABB como teste final. A geometria atual
continua sendo limites documentais ou âncora, não geometria de símbolos elétricos.

Azul indica seleção; hover usa ciano discreto; alça quadrada de 8 pontos marca a
âncora manipulável. Pintura e seleção compartilham os cantos transformados.
Recorte antecede a tesselação; objetos fora do viewport são descartados da pintura.
Cursores: padrão no vazio, seleção sobre objeto, Move no arraste/ferramenta Mover,
mão aberta/fechada no pan. Links e divisores mantêm cursores anteriores.

## 4. Movimento, snap e transformações

O gesto captura transforms originais e o ponto inicial do cursor. Prévia não escreve
Project nem cria histórico por frame. Soltura confirma um lote; Esc, perda de foco,
modal, seleção substituída e mudança de página descartam o gesto. Soltura fora do
viewport ainda confirma um gesto válido. Botão central cancela o gesto esquerdo e
faz pan; esquerdo não move câmera. Roda preserva zoom no cursor.

Movimento múltiplo aplica o mesmo delta a todos, conservando posições relativas,
rotações, espelhos, IDs e alvos. Snap ajusta a âncora do objeto agarrado, não cada
objeto. Grade lógica, grade desenhada e snap permanecem independentes. Ocultar
grade não altera snap; desligar snap não altera espaçamento documental. Posições
negativas e espaçamentos fracionários são testados. Retornar exatamente ao ponto
de captura é no-op mesmo quando a âncora original estava fora da grade.

Rotação positiva CCW: 90°, 180°, 270° e ângulo finito arbitrário no inspetor. Pivô
individual: âncora de cada representação. Espelho horizontal inverte X da página;
vertical inverte Y da página, em torno da mesma âncora. Inspetor expõe flags locais
do Transform2D, explicitamente rotuladas. NaN, infinito e overflow são recusados.

## 5. Core e histórico transacional

`Project::transform_representations` valida IDs/duplicados/limites do lote completo
antes de escrever qualquer transform. Nenhuma mutação de página, alvo, índices,
Connections, Nets, Conductors, terminais, dispositivos, cabos ou veias.
Testes com fiação real comparam todo o snapshot não gráfico e redes derivadas.

`Session::dispatch_graphic` centraliza movimento, rotação, espelhos, valores
numéricos, Undo e Redo. `History` guarda deltas antes/depois, sem cópia do Project.
Uma confirmação efetiva produz uma entrada; no-op não cria entrada nem suja a
sessão. Undo/Redo aplica lote atômico e verifica valores esperados. Nova edição
divergente invalida Redo; seleção, pan e zoom não entram nem apagam histórico.

Retenção: até 100 operações e 50.000 deltas, removendo operações antigas inteiras.
Lotes maiores que 50.000 alvos são recusados antes da mutação. Histórico apenas em
memória; importação bem-sucedida/fechamento o limpa. Falha de importação o preserva.
Dirty é conservador, inclusive após Undo: não há rastreamento de ponto de salvamento.
Edições documentais/visuais anteriores fora desse histórico encerram ambas as pilhas
após sucesso. Não existe Undo genérico de toda operação do projeto nesta etapa.

## 6. Comandos, menus, Ribbon, inspetor e navegação

Registro central habilita Selecionar, Mover, giros, espelhos, Desfazer e Refazer na
guia Editar, propriedades e localização de ocorrência na Página Inicial, além dos
comandos de enquadramento/grade/snap da Visualização. Remove os placeholders
equivalentes; outras operações futuras continuam indisponíveis. Layout e correção
de estabilidade da Ribbon foram preservados. Atalhos Ctrl+Z/Ctrl+Y usam dispatcher,
exceto quando campos textuais estão focados. Esc cancela gestos antes de limpar seleção.

Clique direito em objeto não selecionado estabelece seleção simples; em membro
selecionado conserva o conjunto. Menu oferece propriedades, mover, giros, espelhos,
histórico e referência quando existente. Vazio oferece enquadrar página/conteúdo,
grade, snap/espaçamento e propriedades da página, conservando a seleção até comando.
Menu é estável enquanto aberto e não depende do hover momentâneo dos itens.

Inspetor simples mostra identificação, RepresentationId, PageId, alvo, geometria,
X/Y em mm, ângulo CCW, espelhos locais, visibilidade, camada/rótulo e dados elétricos
já disponíveis. X/Y/ângulo/espelhos são rascunhos até **Aplicar transformação**;
uma confirmação equivale a uma entrada. Prévia exibe X/Y propostos quando acessíveis.
Seleção múltipla mostra quantidade, propriedades comuns ou **Valores diferentes**,
delta X/Y comum e comandos aplicáveis. Undo atualiza os rascunhos pela revisão.

Navegação entre ocorrências conserva IDs e abre a página referenciada. Selecionar
graficamente não reorganiza árvore nem cria vínculos por proximidade. Backstage,
miniaturas, documentos, preferências e painéis da Etapa 5 foram preservados.

## 7. Snapshots e desempenho

Transformações usam os mesmos campos serializados da Etapa 4/5. Round-trip preserva
IDs, alvos e valores. Compatibilidade antiga exercitada também removendo campos
documentais aditivos, além das suítes anteriores. Esses cenários simulam os schemas
anteriores; não constituem coleção congelada de arquivos históricos reais.
Sem serialização de histórico, seleção, hover ou preview; preferências continuam
separadas. Exportação técnica existente mantém validação e substituição segura.

Sem clonagem integral do Project por frame/arraste. Revisão documental só muda na
confirmação, não em hover/prévia; hierarquia não é reconstruída a cada mouse move.
Pintura usa array de quatro cantos, evitando alocação por retângulo. Prévia mantém
somente objetos selecionados. Consulta de hit ainda percorre a página, e pintura
ordena por camada/ID; índice espacial fica para medição em projetos reais.

Medição proporcional: `editor_scale`, 10.000 objetos (5.000 âncoras e 5.000 retângulos
rotacionados com rótulos), 40 consultas de hit, 20 prévias de conjunto e um commit/
Undo, com integridade validada. Debug local: 40 consultas **328,6837 ms** (~8,22 ms
por consulta), 20 prévias **122,2053 ms** (~6,11 ms por atualização), commit
**46,9233 ms**. Log `.local/etapa-06/scale-final.log`, execução `cargo test -p cad-gui
--test editor_scale --locked -- --nocapture`, exit 0. Sem limiar temporal frágil no teste.
O cenário não mede um frame completo com esses objetos; não comprova capacidade
industrial, desempenho em todas as GPUs nem latência nativa de interação.

## 8. Testes, validação e revisão

Baseline oficial: 196 testes aprovados (181 Rust + 15 PowerShell).
Testes anteriores preservados. Novos: `document_transactions`, `editor_history`,
`editor_interaction`, `editor_ui`, `editor_scale` e regressão privada do inspetor.
Cobrem os 38 tópicos do anexo através de testes de Core, sessão, geometria e eventos
egui, incluindo dados elétricos reais, menus, dispatcher e campos numéricos.

Evidências RED de APIs ausentes e regressões reais em `.local/etapa-06/`:
`core-red.log`, `history-red.log`, `interaction-red.log`, `ui-red.log`,
`ui-expanded.log`, `review-red.log`, `selection-red.log`. Ajuste de comparação
fracionária usa epsilon para erro f64; menu é localizado dentro do viewport para
não confundir ação do inspetor. Nenhum teste antigo foi removido ou desabilitado.

Primeira validação completa passou nos testes e parou no Clippy por inicialização
de campo após Default em teste novo. Corrigido sem desabilitar a regra.
Validação oficial final `pwsh -NoProfile -File scripts/validate.ps1`: **exit 0**.
Log `.local/etapa-06/validation-final.log`; código de saída no arquivo adjacente.

| Verificação | Resultado |
|---|---|
| cargo fmt --check | Aprovado |
| cargo check --workspace --all-targets --locked | Aprovado |
| cargo test --workspace --locked | 221 Rust aprovados; zero falhas/ignorados |
| cargo clippy --workspace --all-targets --locked -- -D warnings | Aprovado |
| scripts/tests/check-repository.Tests.ps1 | 15 aprovados; zero falhas |
| git diff --check e git diff --cached --check | Aprovados |

**Total: 236 aprovados = 221 Rust + 15 PowerShell; 40 novos**, preservando 196.
Core: 117 (113 anteriores + 4); GUI: 104 (68 anteriores + 36).
Novos: document_transactions 4, editor_history 11, editor_interaction 9,
editor_ui 14, editor_scale 1, regressão unitária do inspetor 1.

`cargo build --release -p cad-gui --bin cad-gui --locked`: **exit 0**;
`.local/etapa-06/release.log`. Binário `target/release/cad-gui.exe`, 7.980.032 bytes,
PE Subsystem **2 (Windows GUI)**. SHA-256:
`243F1BC4249163431F1A78D0BB7440B4AA33068800F33E27A16B3C5A6DD3390B`.

Revisão independente somente leitura realizada por agente local da mesma sessão.
Encontrou P2: deslocamento numérico abandonado podia reaparecer ao retornar ao
mesmo conjunto/página. Reproduzido RED e corrigido por selection_revision; GREEN
confirma descarte inclusive com inspetor oculto. Revisão retomada após interrupção
por limite de uso e encerrada sem novo defeito funcional confirmado. O revisor não
executou a suíte concorrente nem homologação visual. Medição foi ampliada após
observação sobre cenário inicial somente com âncoras. Nenhum agente alterou Git.

## 9. Inspeção visual e limites

Capturas reais pelo exemplo `render_qa`, usando screenshot do framebuffer eframe:
`.local/etapa-06/editor-final-1600.png` (1600×900, escala 1) e
`.local/etapa-06/group-final-1280.png` (1280×720, escala lógica 1,25).
Imagens válidas inspecionadas: grade, folha, representação girada/espelhada e azul,
alça, Ribbon Editar, propriedades e seleção múltipla. Em pouca altura, inspetor
usa rolagem e miniaturas recolhem conforme comportamento anterior. Captura interna
não inclui decoração do Windows; não comprova título nativo nem DPI entre monitores.
Não foi retomada a investigação do timeout do Computer Use: o anexo atual autoriza
captura interna. Imagem preta não foi tratada como falha de interface.

O exemplo de QA descarta apenas o indicador dirty de sua sessão descartável após
salvar a captura, permitindo encerrar cenários editados sem ficar na confirmação
de descarte da aplicação. A proteção da aplicação normal permanece intacta.
Uma recompilação do helper encontrou LNK1104 enquanto seu executável ainda estava
em uso; após encerrar o próprio processo de QA, a compilação e capturas passaram.
Diagnóstico offline `scripts/check-repository.ps1`: nenhum achado nas regras
auditadas; não representa auditoria completa de segredos/histórico nem consulta remota.

Homologação real pelo usuário continua pendente. Windows 11, alto contraste,
hardware diverso, mudanças de tema e DPI nativo não foram homologados nesta etapa.
Sem escala/estiramento, símbolos IEC, fios gráficos, alça de rotação por mouse,
cycling de sobreposição, Undo geral, autosave, persistência final ou docking livre.
Suavização futura da densidade da grade permanece no roadmap, sem alterar passo lógico.

## 10. Roteiro reproduzível de homologação A–L

Abrir `target/release/cad-gui.exe`, **Abrir demonstração**, abrir **1 · Visão geral**.
O retângulo K1 e a âncora de função são as duas representações disponíveis nessa página.

| Teste | Ação e resultado esperado |
|---|---|
| A — simples | Clicar no retângulo; azul/alça e propriedades do mesmo ID. Clicar no vazio limpa seleção gráfica. |
| B — múltipla | Ctrl+clicar na âncora; dois selecionados e inspetor comum. Ctrl no retângulo o remove. |
| C — janela | Arrastar do vazio esquerda→direita contendo objetos; somente contidos. Inverter direção, cruzar parte do retângulo; incluir intersectados. Esc preserva seleção anterior. |
| D — movimento | Arrastar retângulo; verificar prévia, soltar, conferir X/Y e seleção preservada. Arrastar os dois e conferir deslocamento comum. |
| E — snap | Barra inferior: 4 mm e ímã ativo. Mover a âncora; coordenadas em múltiplos de 4. Ocultar grade e repetir. Desligar ímã e mover livremente; passo continua 4 mm. |
| F — transforms | Editar → giros 90/180/270, espelhos horizontal/vertical. Conferir âncora fixa, ângulo/flags e contorno. Editar X/Y/ângulo no inspetor e Aplicar transformação. |
| G — histórico | Mover, Ctrl+Z, Ctrl+Y; repetir no grupo, giro e espelho. Após Undo, nova edição elimina Redo. Campo textual focado deve conservar seu próprio Ctrl+Z. |
| H — cancelar | Durante movimento, Esc; posição e histórico intactos. Repetir com janela, troca de foco e botão central. Voltar ao ponto inicial não acrescenta histórico. |
| I — navegação | Pan central e roda no cursor, inclusive fora da folha. Esquerdo seleciona sem pan. Trocar/fechar aba cancela prévia; voltar preserva câmera. |
| J — menus | Direito no objeto e em membro do grupo: contexto gráfico; direito no vazio: contexto da página. Propriedades e Localizar referência abrem dados/ocorrência existentes. |
| K — snapshot | Exportar snapshot técnico por Arquivo, reimportar, abrir página e verificar transforms/IDs. Histórico e seleção são reiniciados. |
| L — regressão | Alternar dez guias/rolagem/Ctrl+F1, abrir/fechar Backstage, redimensionar painéis, conferir título escuro, pontos, zoom/pan e viewport independente. Repetir em Windows/DPI desejados. |

Não há homologação registrada sem resposta expressa do usuário.

## 11. Inventário e encerramento

Criados: `crates/cad-craft/tests/document_transactions.rs`;
`crates/cad-gui/src/{history,interaction,hit_test,editor_view}.rs`;
`crates/cad-gui/tests/{editor_history,editor_interaction,editor_ui,editor_scale}.rs`;
spec integral, plano, ADR-0008 e este relatório.

Modificados: `crates/cad-craft/src/domain/project/representations.rs`;
`crates/cad-gui/src/{app,commands,editing,lib,state,viewport}.rs`;
`crates/cad-gui/src/panels/properties.rs`; `crates/cad-gui/examples/render_qa.rs`;
`docs/internal/memory/{CURRENT_STATE,HANDOFF,ROADMAP,KNOWN_ISSUES,ARCHITECTURE,DECISIONS}.md`;
`docs/internal/spec/CONTRATOS_DOCUMENTAIS_GUI.md`. Excluídos: nenhum.
Logs/capturas em `.local/etapa-06/` e builds em `target/`, ignorados pelo Git.
Manifests, lockfile, scripts, Backstage e testes anteriores preservados.

Entrega pronta para revisão/homologação. Etapa par requer **commit
e push exclusivamente manuais**, após homologação conforme anexo. Backup externo
independente recomendado, não criado/verificado. Nenhuma operação Git modificadora,
worktree, repositório/conexão nova, cópia de código proprietário ou agente externo.
Encerrar sem iniciar Etapa 7 e sem presumir SHA futuro.
