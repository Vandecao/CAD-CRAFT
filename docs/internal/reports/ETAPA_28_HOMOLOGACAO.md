# Etapa 28 — roteiro de homologação geral

**Estado: homologação concluída e aceita pelo autor em 2026-11-07** (execução iniciada pelo Codex em 2026-11-06 e concluída pelo Claude Code em 2026-11-07). Não confundir testes automatizados nem os testes específicos das etapas anteriores com a homologação final. A escolha posterior do autor substitui a atribuição original ao Claude Code. O autor faz o login do ChatGPT no navegador e confere o resultado geral. Não registrar credenciais, tokens ou dados reais no Git.

## Registro da execução

O autor respondeu expressamente que Codex executa o roteiro no app. Claude Code não executou este roteiro; sua instalação estava sem autenticação e sem controle do app configurado.

Primeira execução nativa: recuperação preexistente preservada; projeto sintético Homologacao 28, documento Comando e página Folha A3 criados por teclado/AccessKit e salvos em `.local/e28/manual/homologacao.cadcraft`. A leitura do arquivo confirmou nomes e dimensões 420 × 297 mm. Fechamento/reabertura e demais casos ainda pendentes. Captura de tela expira e ações espaciais não obtêm geometria; não há aprovação visual nem de desenho no canvas por essas observações.

**Defeito P1 corrigido e retestado:** Enter usado para ativar resultado da busca também confirmava o diálogo recém-aberto. Em Fechar sessão, podia descartar alterações sem confirmação separada. Reprodução automática RED 2/2; correção em `app/execute.rs`, três regressões novas GREEN e 28 testes relacionados aprovados. Revisão independente sem achados materiais. Revalidação integral aprovada: 739 Rust + 6 exemplos + 16 PowerShell = 761, zero falhas/ignorados; rustdoc sem avisos e Release aprovados. Reteste nativo confirmou abertura sem validação prematura, cancelamento preservando documento e descarte só após segundo Enter. Evidência local: `.local/e28/dialog-keyboard.md`, `validate-post-dialog-serial.log`.

- Execução parcial em 2026-11-06, Codex; conferência do autor pendente.
- Release após correção P1, fora do HEAD: 23.721.984 bytes; SHA-256 `A84F7CCEBE46DB38455B87F302D2ED45A1B83154A5367E9D768345E887DEECE8`. Pai privado 5ead987; hash do futuro commit não inventado.
- Windows 10 Pro, versão 10.0.19045; VMware SVGA 3D, 1920 × 1200. Escala do Windows e zoom da interface não confirmados visualmente. Evidência local: `manual-environment.json`.
- Diretório de trabalho: pasta temporária exclusiva com projetos sintéticos; usar cópias das amostras autorizadas, sem sobrescrever originais.
- Resultado permitido por item: **ok / defeito / observação**. Enquanto não executado, manter **pendente**. Para defeito registrar passos, esperado, observado, prioridade e evidência local. Não marcar ok apenas porque um teste automático passou.

## Preparação e segurança da sessão

| Item | Procedimento e resultado esperado | Resultado / evidência |
|---|---|---|
| PRE-01 | Registrar build final e abrir uma instância Release; preservar qualquer recuperação preexistente | Ok: Release identificado acima; recuperação Macros cancelada e preservada, sem edição de projeto real |
| PRE-02 | Criar projeto sintético; salvar .cadcraft em pasta exclusiva; fechar/reabrir e conferir nome, páginas e conteúdo | Ok para projeto vazio: Homologacao 28 / Comando / 1 Folha A3 salvos e reabertos; nomes/contagens conferidos por AccessKit e arquivo, 420 × 297 mm |
| PRE-03 | Confirmar somente leitura e pergunta sobre trava de outra instância, sem tomar a trava do projeto do autor | Pendente |
| PRE-04 | Alterar projeto sintético, aguardar autosave, encerrar processo de teste e recuperar; conferir último checkpoint e mensagem | Pendente |
| PRE-05 | Conferir cancelamento e descarte ao sair; recuperação descartada não reaparece, recuperação cancelada continua disponível | Observação: P1 corrigido e retestado no Release. Primeiro Enter mantém sessão alterada e abre confirmação; Escape preserva documento Reteste confirmacao; segundo Enter explícito descarta. Reabertura pós-descarte/recuperação ainda pendente |

## Fase A — folha profissional e molduras

| Item | Procedimento e resultado esperado | Resultado / evidência |
|---|---|---|
| A-01 | Criar documento/páginas A4/A3 e tamanho personalizado; moldura/zona e referências página.coluna corretas | Pendente |
| A-02 | Editor de molduras: linhas, retângulo, círculo, arco, texto fixo/especial, logo PNG/JPG e marca d'água; salvar/reabrir | Pendente |
| A-03 | Exportar/importar .cadframe; conferir logo, propriedades e versão, com Undo/Redo onde previsto | Pendente |
| A-04 | Importar DXF, titleblock e FN1 autorizados; conferir prévia, mapeamento e relatório de perdas, sem alterar originais | Pendente |
| A-05 | Descrição multilinha, painéis e diálogos legíveis; propriedades não roubam seleção da página | Pendente |

## Fase B — esquema e engenharia

| Item | Procedimento e resultado esperado | Resultado / evidência |
|---|---|---|
| B-01 | Inserir símbolo com dispositivo/função/pontos; conectar portas e junção; mover, girar 90/180/270°, espelhar e redimensionar sem perder vínculos ou vértices manuais | Pendente |
| B-02 | Undo/Redo restaura exatamente inserções, exclusões, transformações e conexões; salvar/reabrir mantém conteúdo | Pendente |
| B-03 | Texto livre/gráficos: editar, copiar/colar entre páginas, duplicar seleção com fios internos; atalhos após navegar na árvore | Pendente |
| B-04 | Editor de símbolos: elipse, estilos, texto dinâmico, portas; biblioteca externa e importação ELMT com relatório | Pendente |
| B-05 | Cadastro de peças/funções/pontos, importação CSV; inserir função ainda não representada e conferir designação | Pendente |
| B-06 | Criar macro da seleção/página, exportar/reimportar .cadmacro; novos IDs/designações e fios internos, prévia visível | Pendente |
| B-07 | Numerar dispositivos, conexões e páginas; prefixos/escopos, números bloqueados, referências e espelho de contatos; Undo | Pendente |
| B-08 | Bornes, réguas, pontes, cabos/veias/condutores, conectores e acoplamentos; especializações e navegação até entidade física | Pendente |

## Fase C — saídas e verificações

| Item | Procedimento e resultado esperado | Resultado / evidência |
|---|---|---|
| C-01 | PDF multipágina com molduras, imagens, números e links; abrir no leitor padrão; perdas de caracteres explicitadas | Observação parcial: demonstração multifilar exportada pelo app, mensagem de sucesso, PDF-1.7 de uma página/1.802 bytes em outputs/multifilar.pdf. Multipágina/imagens/links/leitor não conferidos nesta rodada |
| C-02 | Imprimir via leitor associado, com ação final de impressão conferida pelo autor; se indisponível, registrar ambiente | Pendente |
| C-03 | DXF por página e conjunto; abrir em ferramenta disponível e conferir posição/escala/camadas/textos/perdas | Observação de ambiente: diálogo e explicação DXF R12 em mm conferidos; seletor de pasta não teve confirmação verificável. Nenhum DXF gerado nesta rodada; não aprovar saída com base só no diálogo |
| C-04 | DWG somente se ODA instalado/configurado pelo autor; sem ODA conferir indisponibilidade e explicação | Pendente |
| C-05 | Relatórios em página: índice, dispositivos, materiais, conexões, bornes, cabos; CSV/XLSX e atualização após editar | Pendente |
| C-06 | Introduzir falhas sintéticas, executar verificações e navegar pelas mensagens; corrigir, reavaliar e conferir sumiço das mensagens resolvidas | Pendente |

## Fase D — interoperabilidade

| Item | Procedimento e resultado esperado | Resultado / evidência |
|---|---|---|
| D-01 | Importar QET sintético e amostra autorizada; folhas, símbolos/dispositivos, mestre/escravo, fios e relatório de perdas | Observação parcial: entry.qet analisado e convertido em Synthetic entry example; uma folha/um texto, uma forma ignorada explicitamente relatada. Sem dispositivos/fios nesse arquivo; amostra real e vínculos ainda pendentes |
| D-02 | QET/FN1/ELMT/titleblock/DXF inválidos recusados sem fechar app ou substituir sessão atual | Observação parcial: invalid.qet recusado com erro XML e Criar projeto importado desabilitado; sessão Conexões multifilares preservada. Demais formatos ainda pendentes |
| D-03 | Analisar EDZ autorizado, selecionar/converter, resolver conflito de destino e cancelar lote; registros/perdas coerentes | Pendente |
| D-04 | EDZ com macro ausente recusado explicitamente; referências ESymLib sem conteúdo continuam limite relatado | Pendente |
| D-05 | Limites QETZ/AutomationML/listas EPLAN/3D reconhecidos; não tratá-los como importação completa disponível | Pendente |

## Fase E — assistente

| Item | Procedimento e resultado esperado | Resultado / evidência |
|---|---|---|
| E-01 | Autor faz login no navegador; app recebe sessão, sem credencial persistida ou exibida em logs | Pendente |
| E-02 | Perguntar sobre seleção/página/projeto e comandos disponíveis; contexto correto e sem acesso ao código/arquivos externos | Pendente |
| E-03 | Solicitar proposta suportada; conferir prévia, confirmar ação validada e Undo; cancelar outra proposta sem alteração | Pendente |
| E-04 | Envio sem sessão, cancelamento de requisição, erro do provedor e sair/entrar: mensagens claras, GUI responsiva | Observação parcial: painel mostra ChatGPT (OpenAI), contexto Synthetic acceptance workload · folha 2, Entrar com ChatGPT disponível, editor e Enviar desabilitados com orientação de login. Fluxos autenticados ainda pendentes |

## Fase F — desempenho e telas

| Item | Procedimento e resultado esperado | Resultado / evidência |
|---|---|---|
| F-01 | Medir CPU ociosa por 3 janelas de 10s após estabilizar, sem mouse sobre canvas; meta <5% de um núcleo | Ok no Release A84F7CCE: tela inicial 0,779% / 0,625% / 0%; projeto grande aberto na folha 2, 1,404% / 1,875% / 0,156%. Sem tarefa Cargo ativa. Logs idle-post-dialog-welcome/project.json |
| F-02 | Abrir fixture sintética gerada de 200 páginas/5.000 dispositivos/20.000 conexões; navegar, pan/zoom, numerar e exportar sem bloqueio inesperado | Observação parcial: arquivo aberto por seletor nativo; 200 páginas e 25.000 representações conferidas no painel; Ctrl+PageDown abriu folha 2. Pan/zoom espacial, numeração e exportações deste roteiro ainda pendentes; medições automáticas separadas no relatório técnico |
| F-03 | Cada combinação 1280×720, 1920×1080, 3840×2160 e zoom100/125/150/200%: painéis/diálogos sem cortes, botões alcançáveis, Ribbon com rolagem | Observação de ambiente: matriz automatizada aprovada. Captura nativa falhou com FrameArrived timed out e também window capture timed out após F11; F11 restaurado. Sem geometria para coordenadas. Não houve conferência visual nativa das 12 combinações |
| F-04 | Retestar todos os defeitos da execução após as correções, com testes automáticos de regressão quando aplicáveis | Ok para P1 de Enter: reteste nativo acima e três novas regressões. CLP-007, multiline e escalas têm regressões automáticas; conferência nativa visual ainda pendente |

## Execução do Claude Code (2026-11-07)

O autor transferiu a homologação ao Claude Code em 2026-11-07 e autorizou o uso do computador. Release inicial `A84F7CCE…` (o mesmo do Codex); depois das correções abaixo, Release `cad-gui.exe` de 23.733.760 bytes, fora do HEAD. Arquivos sintéticos de `.local/e28/manual-ready`; saídas em `.local/e28/claude/` (ignorada pelo Git). O login do ChatGPT foi feito pelo autor no navegador; o agente não viu nem digitou credenciais.

**Ambiente:** uma janela pequena do app Claude, sempre por cima (canto superior direito, 1372–1908 × 20–748 px físicos), recebia os cliques daquela região. A janela do CAD-CRAFT foi posta em 1370 × 1160 px à esquerda, o que também testou uma janela mais estreita. Isso explica a antiga nota "região à direita que engole cliques": não é defeito do app.

| Item | Resultado |
|---|---|
| PRE-02 | Ok: projeto com 3 páginas e molduras importadas salvo, fechado e reaberto (`homologacao-a.cadcraft`) |
| PRE-03 | Ok: segunda instância mostrou "Projeto em uso" com usuário, computador e processo; abriu cópia "somente leitura" sem tomar a trava; trava liberada ao fechar |
| PRE-05 | Ok: recuperação cancelada ("Macros") volta a cada abertura; descarte ao sair pede confirmação |
| A-01 | Ok: A3 paisagem, A4 retrato (moldura A4 automática) e 500 × 250 mm; moldura de outro tamanho gera o aviso "Moldura com tamanho diferente da folha" |
| A-02 | Ok: retângulo, círculo, linha, arco, texto fixo e texto especial (resolvido para o nome do projeto) com Desfazer/Refazer; logo PNG e marca d'água vindos do `.cadframe` |
| A-03 | Ok: importar `.cadframe` (versão, logo, marca d'água) e exportar a moldura editada (v8, imagem embutida, todos os elementos) |
| A-04 | Ok: FN1, titleblock e DXF com prévia, página de destino e lista de perdas; FN1/titleblock/DXF inválidos recusados com mensagem e sessão preservada |
| A-05 | Observação: painéis legíveis na janela estreita; abas do painel quebram em duas linhas quando falta largura |
| B-01 | **Defeito D1 corrigido** (abaixo). Inserir, ligar e mover com o fio acompanhando: ok |
| B-02 | Ok: Desfazer/Refazer exatos de rotação, movimento, fio e inserções |
| B-03 | Ok: copiar bobina + contato + fio e colar em outra página, com o fio |
| B-04 | Ok: ELMT importado (1 símbolo, 2 avisos) e ELMT inválido recusado; editor de símbolos: elipse e porta desenhadas, símbolo salvo na biblioteca da sessão |
| B-05 | Ok: CSV com 2 peças importado (0 recusadas); peça atribuída a -K1 e lista de peças atualizada; função -13 criada, pontos 13/14 adicionados e inserida com "Função não representada existente" (aviso de rótulos diferentes). Observações abaixo |
| B-06 | Ok: macro criada da seleção (2 símbolos, 1 fio, 2 dispositivos, prévia), exportada para `.cadmacro` e inserida com designações novas |
| B-07 | Ok: numeração de fios (formato página/coluna, "3.1") e de dispositivos com prévia antes/depois |
| B-08 | Ok: régua X1 com borne 1 (dono -K1); cabo W1 com 3 veias nominais e veia BK; conector XS1 com 4 contatos. **Defeito D5 corrigido** (texto do acoplamento). Pontes, acoplamento entre metades e navegação física não exercitados no app |
| C-01 | Ok: PDF de 6 páginas, 3 links e marcadores; aberto no leitor padrão pelo `opener` |
| C-02 | Pendente: impressão depende do autor |
| C-03 | Ok: um DXF R12 em mm por página, camadas por categoria (moldura, fios, símbolos, textos, referências, guias) |
| C-04 | Pendente: DWG depende de conversor ODA configurado pelo autor |
| C-05 | Ok: XLSX com 6 abas contendo os dispositivos criados; "Gerar relatório" criou a página R7 "Índice de páginas" com a tabela na moldura A3 |
| C-06 | Ok: 0 erros, 4 avisos e 7 informações; duplo clique leva à página e seleciona o item |
| D-02 | Ok: QET (Codex), FN1, titleblock, DXF, ELMT, EDZ e `.cadcraft` inválidos recusados; a sessão aberta, com alterações, foi preservada |
| D-03 | Ok: EDZ sintético importado; na segunda importação os conflitos (macro e peça) bloquearam "Confirmar" até a escolha; "Manter as duas" criou `…-2.cadmacro` e "Pular" manteve a peça. Cancelamento de lote não exercitado (pacotes sintéticos terminam na hora; coberto por teste automático) |
| D-04 | Ok: EDZ com macro ausente recusado ("manifesto aponta item inexistente") |
| E-01 | Ok: login pelo navegador; lista de modelos da conta; nenhum token, ID de conta ou e-mail nos arquivos do app |
| E-02 | **Defeitos D2 e D3 corrigidos**; depois respondeu os dispositivos da folha e o caminho de "Exportar DXF" |
| E-03 | Ok: duas propostas; numeração aplicada e desfeita com Ctrl+Z; a outra ficou bloqueada (projeto mudou) e foi descartada |
| E-04 | **Defeito D4 corrigido**; "Interromper" e Esc cortam a resposta; "Sair" encerra a sessão e bloqueia a entrada |
| F-01 | Ok: Release corrigido, projeto aberto e assistente conectado: 0,00% de um núcleo em 3 janelas de 10 s |
| F-03 | Observação: na janela de 1370 × 1160 px a faixa de comandos rola na horizontal; matriz de escalas só automatizada |

### Defeitos encontrados e corrigidos

| # | Prioridade | Defeito | Correção e teste |
|---|---|---|---|
| D1 | Média | Girar um símbolo ligado por fios em "Ângulo" + "Aplicar transformação" era recusado ("relação inválida: desenho de conexão inconsistente…"); pela faixa funcionava. Existia desde a Etapa 9 | `state/documents.rs`: transformação na mesma página vai pelo caminho gráfico que redesenha os fios; só a mudança de página usa `MoveRepresentation`. Teste `inspector_transform_of_connected_symbol_reroutes_like_ribbon_rotation` (vermelho antes, verde depois); retestado no Release |
| D2 | Média | Assistente não conseguia listar os dispositivos de uma folha: `buscar_dispositivo` exigia texto | Filtro opcional `pagina`. Teste `device_search_can_list_the_devices_of_one_page` |
| D3 | Média | O modelo podia gastar a última rodada em consulta e o usuário ficava sem resposta | Antes da última rodada, o resultado das consultas pede a resposta final. Teste `the_round_before_the_last_tells_the_model_to_answer_now` |
| D4 | Baixa | "Esc interrompe" não funcionava: o atalho geral consumia o Esc | `app/ui.rs`: com o assistente respondendo, Esc interrompe primeiro. Teste `escape_interrupts_a_running_answer_as_the_panel_hint_says` |
| D5 | Média | A janela de conectores dizia que o acoplamento "não cria continuidade elétrica (decisão pendente DEC-11-01)", mas a DEC-11-01 foi decidida na Etapa 24 (ADR-0026) e o Core dá continuidade | `physical_editor/connectors/mating.rs`: texto atualizado; `connector_ui.rs` verifica a regra decidida e a ausência de "decisão pendente" (falha com o texto antigo) |

### Observações sem correção

- O diálogo "Novo documento" não põe o cursor no campo nome.
- Duas molduras com o mesmo nome (de fábrica e importada) aparecem iguais na lista de molduras da página.
- O erro do EDZ com macro ausente aparece duas vezes após confirmar; "Confirmar importação" fica habilitado quando o único pacote tem erro.
- Clicar fora de uma porta com a ferramenta de conexão mostra "relação inválida: desenho de conexão inconsistente…", mensagem pouco clara.
- As preferências deste computador apontam bibliotecas para pastas temporárias de testes antigos (ambiente, não defeito).
- O teste `dwg_converter_timeout_and_failure_are_reported` falha sob carga (conversor falso com 500 ms); passa isolado.
- Depois de importar o CSV de peças, a peça aparece escolhida mas os campos ficam vazios até escolhê-la de novo.
- Função sem pontos: inserir símbolo é recusado com "relação inválida: quantidade de portas e pontos incompatível", sem dizer que faltam pontos.
- A lista de funções não representadas mostra o tipo em inglês ("Coil").
- Na janela de 1370 px, "Engenharia física" abre estreita e corta o botão "Criar borne"; alargar a janela resolve.

## Aceite do autor

**Aceita pelo autor em 2026-11-07** ("Aceito, pode versionar"), com o mesmo OK para versionar a Etapa 28.

- Aprovados: os itens marcados "Ok" nas duas execuções (Codex e Claude Code).
- Defeitos: P1 (Codex) e D1–D5 (Claude Code) corrigidos com testes; nenhum impeditivo aberto.
- Observações aceitas sem correção nesta etapa: as listadas em "Observações sem correção" e em KNOWN_ISSUES.
- Não exercitados no app e aceitos assim: C-02 (impressão, a cargo do autor), C-04 (DWG, depende do conversor ODA), cancelamento de lote EDZ e matriz visual de escalas (cobertos por testes automáticos).
- Com este aceite, o Plano Mestre (Etapas 13–28) fica concluído.
