# ADR-0008 — Editor gráfico e histórico de transformações

Data: 2026-10-28. Implementado na Etapa 6; validação final registrada no relatório
da etapa. Homologação manual da Etapa 6 pendente.
Requisito: `docs/internal/spec/ETAPA_06_EDITOR_GRAFICO_INTERATIVO.md`.

## Contexto

O Core já possui RepresentationId, Transform2D, limites locais, estilo e APIs de
edição individual. Não existe transação agrupada nem histórico. A GUI da Etapa 5
tem Project privado em Session, câmera por aba, dispatcher e snapshot técnico.
O novo anexo autoriza editar representações, sem alterar conectividade ou símbolos.

## Decisões

1. `Project::transform_representations(&[(RepresentationId, Transform2D)])` valida
   IDs distintos e limites transformados do lote inteiro antes de escrever.
   Altera somente transforms; não altera página, alvo, registros elétricos ou índices.
   A validação de finitude é contrato dos tipos Point2D/Transform2D; overflow dos
   limites é recusado. Não clona o Project nem implementa um segundo agregado.
2. `Session::dispatch_graphic` é o caminho comum de transformações. Ribbon,
   menus, atalhos, gesto e inspetor chegam a ele. `Edit::RepresentationTransform`
   na mesma página delega a esse caminho para preservar consumidores anteriores.
3. `History` guarda deltas por RepresentationId, antes/depois, somente na sessão.
   Um lote confirmado equivale a uma operação. No-op não suja documento, não cria
   entrada nem invalida Redo. Nova edição efetiva remove Redo. Undo/Redo verifica
   o valor esperado antes de aplicar lote atômico. Limites: 100 operações e 50.000
   deltas retidos; remove operações mais antigas inteiras. Lote com mais de 50.000
   alvos é recusado antes de qualquer mutação. Câmera/seleção ficam fora do histórico.
4. Edições anteriores fora do histórico gráfico (página, propriedades visuais,
   cadastro documental) encerram as duas pilhas após sucesso. Evita aplicar deltas
   sobre contexto incompatível e deixa explícito que não existe Undo geral de projeto.
   Importação bem-sucedida/fechamento limpa histórico e transientes; importação
   inválida preserva tudo. Dirty é conservador: Undo não presume ponto de salvamento.
5. `Interaction` guarda ferramenta Select/Move, conjunto BTreeSet de IDs, primário,
   hover e gesto Move/Rectangle. DragSession contém transforms originais e prévia;
   não escreve Project por frame. Movimento usa deslocamento comum e snap na âncora
   do objeto agarrado; nunca arredonda cada objeto isoladamente. Retorno ao ponto
   inicial preserva a transformação original, inclusive se fora da grade.
6. Rotação CCW de 90/180/270 graus e ângulo numérico usa a âncora individual como
   pivô. Espelho horizontal inverte X da página, vertical inverte Y da página,
   ambos em torno da âncora. Converte para a convenção local do Core negando ângulo
   e alternando a flag correspondente. Inspetor explicita que suas flags são locais.
7. Hit testing consulta a página por índice existente, ignora invisíveis, transforma
   cursor para espaço local e usa tolerância de 6 pontos lógicos/escala da câmera.
   Retângulos são limites documentais, não símbolos. Sem limites, usa âncora.
   Ordem visual `(layer, id)`; maior vence. Lista de candidatos prepara futura
   alternância, sem comando de cycling nesta etapa. Janela esquerda→direita exige
   contenção; direita→esquerda testa interseção real de retângulos transformados,
   inclusive segmento atravessando a janela e janela dentro do objeto.
8. Pintura usa cantos transformados compartilhados com seleção, recorte e descarte
   fora do viewport. Azul indica seleção; hover usa outro tom. Alça quadrada na
   âncora serve ao movimento já existente; não há alça de escala/rotação fictícia.
9. Esquerdo seleciona/arrasta; Ctrl alterna membros; meio cancela gesto esquerdo e
   faz pan; roda mantém zoom no cursor. Esc, perda de foco, modais, troca/fechamento
   de página e seleção substituída cancelam a prévia. Soltura fora do viewport
   confirma se ainda há gesto válido. Menus não modificam conectividade.
10. Inspetor simples confirma X/Y/ângulo/espelhos em botão Aplicar transformação;
    arrastar um campo numérico não cria dezenas de entradas. Múltiplo exibe comuns
    ou Valores diferentes e permite delta numérico comum. selection_revision e
    revisão documental invalidam rascunhos, inclusive com painel oculto durante
    ida/volta à mesma página/seleção.

## Alternativas e consequências

Snapshots integrais por frame/gesto foram descartados: aumentariam memória e custo
e poderiam restaurar dados elétricos fora do escopo. Aplicar mutações individuais
em sequência com rollback foi descartado em favor de validação integral do lote.
Índice espacial permanente é adiado; medição local proporcional antecede otimização.
Consulta de hit percorre objetos da página; pintura ordena por camada/ID por frame.
Não são garantias de capacidade industrial; ver medidas e limites no relatório.

Nenhuma mudança de schema, dependência ou formato de arquivo. Transforms existentes
continuam serializados pelo snapshot técnico da Etapa 5. Histórico, hover, seleção,
gestos e ferramentas não são serializados. Formato final, migrações, símbolos,
roteamento, ERC e Etapa 7 permanecem fora desta entrega.

Homologação da Etapa 5 foi confirmada pelo usuário no anexo atual; isso não
homologa automaticamente o editor da Etapa 6. Git/backup continuam manuais.
