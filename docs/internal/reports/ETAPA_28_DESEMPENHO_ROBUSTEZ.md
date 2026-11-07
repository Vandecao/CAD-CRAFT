# Etapa 28 — desempenho, robustez e telas

Execução iniciada em 2026-11-06, a partir de `5ead9879522fb2549741a7f97d16345f06b31c61` no privado C:/Dev/CAD-CRAFT, branch main. R6 já publicado também no público em `51b19b01e134db94c0cec14368c5df02348f29cf`. Especificação: [Etapa 28](../spec/ETAPA_28_FECHAMENTO_HOMOLOGACAO.md). Plano: [execução](../plans/2026-11-06-etapa28.md).

## Implementação e diagnóstico

GUI-L01 foi reproduzido no Release anterior: 71,52% de um núcleo em 10,31 segundos, app parado na tela inicial. `CadApp::show` enviava `ViewportCommand::Title` em cada quadro; o egui 0.33.3 solicita repaint em `send_viewport_cmd_to`, mesmo com título idêntico. A regressão nova falhou com o comando repetido e atraso de repaint de 0 ns. O app guarda o último título e só emite comando quando muda. Tema/foco e tarefas ativas mantêm seus eventos próprios.

Ao retirar esse ciclo, o autosave precisa de um despertar explícito: projeto sujo com recuperação disponível agenda o tempo restante até o próximo checkpoint. Não há polling permanente de recuperação; projeto limpo ou saída confirmada não agenda autosave. Testes cobrem título inicial, troca de projeto, dirty marker, ausência de repaint contínuo e prazo da recuperação.

Os testes reproduziram cortes no editor de peças e no navegador de macros em escalas altas, pouco espaço vertical para a folha e cabeçalho do Centro de inserção cortado. As janelas passam a respeitar a área lógica disponível com rolagem nos dois eixos; a faixa de comandos fica compacta em telas baixas, mantendo rolagem. O cabeçalho pode quebrar linha. A descrição de moldura usa campo multilinha, preservando linhas, aplicação, Undo e Redo.

A medição completa da GUI, incluindo tesselação, revelou outro gargalo: antes das otimizações, desenho 67,719 ms, zoom 71,403 ms e pan 68,791 ms. O perfil isolou widgets de dispositivos fora da tela e recálculo de referências por página/miniatura. Um teste demonstrou 500 nós de dispositivos no AccessKit apesar de poucas linhas visíveis. A lista agora virtualiza seus widgets com IDs estáveis, altura compatível com o ícone de favorito e acesso ao rótulo completo por hover/acessibilidade. O teste rola até o último dispositivo e o seleciona por clique real; usa tempo de entrada determinístico para concluir animações antes do clique.

O perfil descartou a suspeita de miniaturas ilimitadas: a faixa já virtualizava as linhas. A correção preserva essa virtualização e deixa de pintar folhas completamente recortadas. Referências derivadas são reutilizadas por ProjectId/revisão/página; editar, desfazer, refazer e trocar projeto invalidam o cache inteiro, pois renumerar uma página pode alterar referências de outras. Não há cache de geometria editável nem cópia de estado de domínio. Regressões verificam reutilização e igualdade dos dados após renumeração/Undo/Redo/troca de projeto. No perfil Debug, o trabalho UI passou de 239,382 ms para 21,499 ms (tesselação separada); a aceitação usa a medição Release final abaixo.

CLP-007 foi reproduzido: `wants_keyboard_input()` também é verdadeiro para foco da árvore e bloqueava Colar. A proteção agora identifica estado real de TextEdit focado (incluindo edição textual de valores numéricos), preservando as proteções de diálogo/backstage. Dois testes novos demonstram a falha anterior e a correção: Copy/Paste com foco persistente na árvore permite posicionar na página ativa e Undo exato; Copy/Cut/Paste em campo textual não altera o projeto nem ativa uma ferramenta. Rodada direcionada: 27 testes aprovados, incluindo os testes antigos de atalhos e foco. A validação integrada foi repetida após essa correção final.

Formato nativo, modelo, importadores, exportadores, dependências e baselines não foram alterados. Não houve mudança de versão nem regeneração de saída de referência.

## Carga grande sintética

Fixture gerada em tempo de teste, não versionada: 200 páginas A3, 5.000 dispositivos, 5.000 símbolos e 20.000 conexões/rotas desenhadas. Operações públicas constroem uma página real, replicada pelo helper existente com desserialização validada e reconstrução de índices. O teste pequeno confere contagens, redes separadas, texto desenhado e round trip exato. A carga grande roda em Release ou por variável explícita; limites CI folgados são separados das metas impressas.

Ambiente observado: Windows, Intel Core i9-11950H apresentado com 4 núcleos/4 processadores lógicos; rustc 1.99.1. Tempos finais Release:

| Operação | Medição | Meta aprovada |
|---|---:|---:|
| Salvar em disco, ZIP/validação e sync_all | 740,676 ms | 3.000 ms |
| Abrir do disco e validar | 1.927,814 ms | 3.000 ms |
| Lista de exibição de página, mediana de 11 | 6,199 ms | 16 ms |
| Numerar dispositivos e conexões, prévia + aplicar | 839,729 ms | 2.000 ms |
| Verificações do projeto | 1.608,029 ms | Medir |
| PDF, 200 páginas | 1.529,490 ms | 30.000 ms |
| DXF, 200 páginas | 2.582,948 ms | Medir |
| Seis tabelas de relatórios | 16.458,187 ms | Medir |
| XLSX dessas tabelas | 301,823 ms | Medir |

Teste Release do Core: 2 aprovados, zero falhas/ignorados. O PDF é relido e deve ter 200 páginas; DXFs não vazios, números das 20.000 conexões e contagens de relatórios são conferidos. Projeto reaberto deve ser exatamente igual ao original. Carga gera 5.000 mensagens esperadas por ausência de peça atribuída, sem esconder custo da verificação.

Abrir usa cache aquecido do SO logo após salvar; não é garantia de armazenamento frio ou latência do diálogo nativo. Geração da fixture levou 11,06 s e fica fora do orçamento de abrir/salvar. Não foi necessário otimizar Core: metas aprovadas já atendidas. Primeira rodada tinha outro Cargo ativo e media serialização/memória e apenas fios; os números inicial/final não são apresentados como ganho de otimização. Relatórios são o maior custo: consultas globais por dispositivo foram identificadas como oportunidade de perfil, com marco em [pendências](ETAPA_28_PENDENCIAS.md).

Frames reais egui com tesselação, mediana de 11 lotes de 32 quadros por operação: desenho 5.206 ms, zoom pela roda 5.341 ms e pan 5.09 ms. Cada entrada deve realmente mudar a câmera; movimentos alternados mantêm o conteúdo na folha. O primeiro ensaio de frames isolados retornou mediana zero, portanto não foi usado como evidência de velocidade; o ensaio final agrupa quadros, exige relógio resolvido e consome os meshes. Isso mede construção/tesselação CPU, não apresentação GPU.

CPU nativa do Release final, três janelas de 10 s por estado: tela inicial 0.62%, 0.00%, 0.47%; projeto sintético grande aberto 0.00%, 0.47%, 2.81% de um núcleo. Todas abaixo de 5%, após estabilizar e sem tarefa ativa. Antes: uma janela de 10,31 s, 71,52%; não se pretende equivalência de hardware/estado além deste diagnóstico local. Executável final 23,721,984 bytes; SHA-256 `6231A881C47319C53630D8FC4B8FFBCAB00032405A5466EA7C672C8E4D3F22BA`. Projeto grande aberto por diálogo nativo; recuperação preexistente preservada, nenhum projeto real editado.

## Robustez e telas

Fuzz leve determinístico: sete testes, dez caminhos de leitura, 192 mutações por caminho, além da semente válida. Cobre `.cadcraft` em arquivo/payload, QET, ELMT, titleblock, FN1, EDZ ZIP/7z/payload e DXF. Binários aleatórios limitados a 2.048 bytes, truncamento, bit flips, inserção numérica/XML, exclusão e duplicação. Semente fixa, IDs nativos fixos e diagnóstico de caso permitem reprodução. Nenhum panic encontrado na rodada; não se declara prova exaustiva de segurança.

Recuperação: processo filho usa autosave real em dois checkpoints, faz edição validada ainda dentro do próximo intervalo e entra na escrita atômica de uma substituição. O pai mata o filho após bytes parciais no temporário, recria os serviços e confere igualdade integral com o último checkpoint, incluindo ausência da edição posterior. O temporário remanescente comprova o ponto da interrupção. Timeout e guarda de processo evitam filhos abandonados. Não se mata instância do usuário.

Matriz automática: 17 diálogos × 12 combinações de resolução/zoom (204 casos), áreas externas dentro da tela; Confirmar/Cancelar dos modais comuns integralmente pintados. A folha mantém pelo menos 140×65 pontos. Rolagem por eventos reais alcança o botão inferior de Peças em 12 combinações, a última guia da faixa e a última categoria do painel; o cabeçalho Assistente permanece visível. A cobertura é representativa, não inspeção de todos os estados possíveis. Regressão antiga de rolagem de símbolos foi preservada e passou após manter o cabeçalho em uma linha nos painéis largos. Logs finais de telas/símbolos e a validação integrada confirmam as correções.

## Validação e revisão

Antes da recompilação, o autor pediu limpeza de target por falta de espaço. `cargo clean --target-dir C:/Dev/CAD-CRAFT/target` removeu 73.464 arquivos (98,5 GiB informados pelo Cargo); o espaço livre observado aumentou 92,05 GiB, ficando em 105,79 GiB. Código, relatórios e evidências em `.local` foram preservados. As compilações Debug seguintes usam variáveis locais `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` e `CARGO_INCREMENTAL=0` para reduzir artefatos, mantendo código não otimizado e debug assertions. Nenhuma configuração de perfil foi alterada no repositório; Release permanece com o perfil normal.

`pwsh -NoProfile -File scripts/validate.ps1` aprovado: **736 testes Rust + 6 exemplos + 16 testes PowerShell = 758**, zero falhas/ignorados; fmt, check de todos os alvos, Clippy -D warnings, baselines e whitespace passaram. Os caminhos grandes opt-in têm execução Release separada (Core 2 testes, GUI 1 teste); sua carga não é executada pelo validate em Debug. Fuzz 7/7, recuperação 2/2 e idle 2/2 passaram também isoladamente. `cargo doc --workspace --no-deps --locked` com RUSTDOCFLAGS=-D warnings e build Release aprovados. Revisão independente sem achados de produção materiais remanescentes; melhorias de medição/interação incorporadas. Inventário: [ETAPA_28_ARQUIVOS.md](ETAPA_28_ARQUIVOS.md).

Evidências locais ignoradas em `.local/e28`: idle-before.json, performance-before.log, performance-final.log, performance.md, robustness.md, logs da matriz, review.md e resultados finais de validação/Release. Instruções reproduzíveis em [Performance checks](../../PERFORMANCE.md). Amostras reais não foram necessárias para os novos testes; nenhum dado de terceiro foi adicionado.

## Homologação, Git e continuidade

O autor posteriormente escolheu Codex para executar a Parte D no app Release; login no navegador e conferência final continuam com o autor. O [roteiro completo](ETAPA_28_HOMOLOGACAO.md) tem fases A–F e registra os resultados observados e itens pendentes. Os testes de diagnóstico desta entrega não equivalem ao aceite geral. O Plano Mestre não pode ser declarado homologado antes dessa confirmação.

O roteiro nativo encontrou um defeito P1 adicional: Enter que ativava um comando era reutilizado pelo diálogo criado no mesmo quadro, permitindo descarte sem confirmação separada. A execução do comando agora consome esse evento. Três regressões novas e 28 testes relacionados passaram, com revisão independente sem achados materiais. O reteste nativo no novo Release confirmou: Novo documento abre sem validar nome vazio; primeiro Enter em Fechar sessão mantém projeto alterado e abre confirmação; Escape preserva o documento; segundo Enter explícito descarta.

Após essa correção, validação integral e rustdoc aprovados: **739 Rust + 6 exemplos + 16 PowerShell = 761**, zero falhas/ignorados; sem alteração de referências. O primeiro validate interrompeu no teste existente de conversor DWG: a simulação que deveria terminar com erro excedeu seu prazo de 500 ms enquanto o Release compilava. Sem mudar código nem enfraquecer testes, repetição isolada 4/4 e nova validação integral sem compilação concorrente passaram. Não se afirma causalidade além dessa observação. Logs: `cad-export-recheck.log`, `validate-post-dialog-serial.log`, `rustdoc-post-dialog.log`.

Novo Release: 23.721.984 bytes, SHA-256 `A84F7CCEBE46DB38455B87F302D2ED45A1B83154A5367E9D768345E887DEECE8`. Benchmark GUI repetido sem outro Cargo ativo: desenho 4,209 ms, zoom 4,698 ms e pan 4,339 ms; 1/1 aprovado (`gui-performance-post-dialog.log`). Os números e o hash anteriores permanecem como histórico da medição antes do P1.

CPU nativa desse novo binário, sem Cargo ativo, três janelas de 10 s: tela inicial 0,779% / 0,625% / 0%; projeto grande na folha 2, 1,404% / 1,875% / 0,156% de um núcleo. Seis resultados abaixo de 5%; arquivos `idle-post-dialog-welcome.json` e `idle-post-dialog-project.json` registram o mesmo SHA do executável.

Revisão de pendências em [ETAPA_28_PENDENCIAS.md](ETAPA_28_PENDENCIAS.md). Nenhum staging, commit, push, sincronização pública ou alteração de refs nesta implementação. A autorização do R6 não abrange versionamento da Etapa 28. Backup externo não criado/verificado. Não foi iniciada etapa futura.
