# Contratos documentais do Core para futura GUI

Nota de evolução (2026-10-28): o texto original abaixo registra a Etapa 4;
o complemento da Etapa 6 ao final descreve seleção/edição/histórico hoje existentes.

Preparado pela Etapa 4 em 2026-10-27. Não implementa nem autoriza a Etapa 5.
Core não depende de egui/eframe. Este documento descreve APIs existentes e
responsabilidades futuras, sem prometer recursos gráficos concluídos.

## Navegador de páginas

Usar Project::documents e ordered_pages(DocumentId). PageId é identidade;
Page::number é identificação editável e não precisa ser único. Não usar número,
índice da lista ou nome como chave de estado da GUI. pages/pages_in_document
continuam por ID; para ordem documental usar explicitamente ordered_pages.

page_tree(document, StructureAxis::Function/Location) fornece Vec<PageGroup>
em pré-ordem lexical. O nó raiz tem path vazio; descendentes usam prefixos do
caminho. pages lista somente páginas diretas, já na ordem documental. Não
confundir grupo funcional com FunctionId. A GUI pode manter expansão por caminho,
sabendo que editar o caminho altera essa chave. Sem grupos persistentes vazios.

## Viewport e coordenadas

Unidades em mm, origem da folha (0,0) inferior esquerda, X à direita/Y para cima.
document_bounds(page) retorna sheet e content separados. São sugestões para
enquadrar, nunca restrições de pan/zoom. Conteúdo aceita posições negativas e
fora da folha. A GUI deve tratar Result::Err por overflow ao enquadrar extremos.

A camada de visualização futura converte mm para tela, usando escala/zoom e
inversão do Y uma única vez. Transform2D opera apenas em coordenadas cartesianas
do modelo: espelhar componentes X/Y, girar anti-horário, transladar. inverse_apply
faz o caminho de volta para coordenadas locais. Não adicionar inversão do Y ao Core.

## Seleção e renderização

Selection futura deve guardar RepresentationId para a ocorrência e consultar
RepresentationTarget para o alvo elétrico; selecionar uma ocorrência não significa
selecionar todas as outras. representation(id) permite resolver IDs e detectar
seleções removidas. A seleção em si não é implementada nesta etapa.

representations_on_page retorna todas as ocorrências por ID. A GUI filtra visible
e ordena por (layer, id) quando desenhar; maior layer por último. Transform2D::apply
transforma pontos locais. Representation::bounds fornece AABB, útil como filtro
amplo para seleção. Sem local_bounds, somente a âncora é conhecida: não presumir
área de símbolo, espessura de fio ou hit testing exato. Rótulo é opcional; metadados
elétricos devem ser consultados no alvo. Símbolos/rotas ainda não existem.

## Inspetor e navegação

Page: name, number, kind, size, orientation, order, structure, metadata e properties.
Project: identificação opcional, nome, descrição, metadados e propriedades existentes.
Representation: target, page_id, transform, local_bounds e visual.
Não duplicar propriedades elétricas no estilo da representação.

representations_for_entity encontra ocorrências do alvo;
related_representations exclui a ocorrência atual e retorna outras do MESMO alvo;
representation_location informa PageId e âncora. Relações entre entidades diferentes
devem ser consultadas explicitamente nas APIs elétricas existentes, sem novo grafo.

## Comandos futuros do editor sobre APIs atuais

| Intenção | API do Core já disponível |
|---|---|
| Criar folha | Page::new + Project::insert_page |
| Editar nome/identificação | rename_page / identify_page |
| Editar formato | set_page_format(kind, size); orientação derivada |
| Classificar | set_page_structure(DocumentStructure) |
| Reordenar | reorder_pages(document, permutação completa de PageIds) |
| Excluir folha | remove_page, sem cascata; requer ausência de referências externas e representações |
| Inserir ocorrência | Representation::new + insert_representation |
| Mover/girar/espelhar | move_representation(id, page, transform) |
| Alterar estilo/extensão | set_representation_visual / set_representation_bounds |
| Excluir ocorrência | remove_representation, preserva entidade elétrica |
| Propriedades de página | set_properties(EntityRef::Page, TechnicalProperties) |

Falhas previstas retornam DomainError antes de mutação parcial. Essa atomicidade
individual não é sistema transacional de comandos: Undo/Redo e operações agrupadas
continuam futuros. A GUI deverá recompor consultas após mutações e descartar seleção
pendente após remoção. Não executar remoções em cascata implicitamente.

## Persistência e restrições

Serde estrutural é o mecanismo existente. Não é contrato definitivo de arquivo,
salvamento em disco ou migrações. Índices privados não são persistidos. Snapshots
anteriores carregam defaults vazios/opcionais, sem gerar conteúdo documental.
Homologação visual depende de GUI futura e permanece adiada até após a Etapa 15.

## Complemento implementado na Etapa 6

Project::transform_representations(&[(RepresentationId, Transform2D)]) valida lote
completo antes de escrever transforms. IDs repetidos/ausentes e overflow de limites
são rejeitados sem escrita parcial. Página, alvo e índices permanecem iguais;
redes, conexões e fiação não mudam. Sem alteração de schema serializado.

GUI possui SelectionState por IDs, hover/gestos transitórios, preview, hit testing
de limites/âncoras e dispatcher gráfico. Histórico de deltas apenas de transforms
em Session: 100 operações/50.000 deltas, Undo/Redo atômico. Outras edições encerram
histórico; câmera/seleção não. Não é Undo geral nem persistência física final.
Snapshots anteriores continuam carregando; transientes/histórico não são salvos.
Detalhes de pivô, snap, espelhos e cancelamento no ADR-0008 e relatório da Etapa 6.
Homologação visual da Etapa 5 foi confirmada no novo anexo; Etapa 6 aguarda usuário.

## Complemento implementado na Etapa 7

Representation::symbol referencia uma SymbolDefinition imutável no Project.
SymbolInstance é essa ocorrência, com o mesmo RepresentationId; não há agregado
gráfico paralelo. RepresentationTarget::Unassociated expressa ausência de alvo,
permitida só para símbolos; consultas por alvo não agrupam essas ocorrências.

SymbolLibrary::builtin fornece definições reutilizáveis. Construir com
Representation::new_symbol e inserir via Project::insert_symbol_representation
valida projeto/página/alvo, definição, limites e orientação antes de mutação.
Definição já registrada com ID igual e conteúdo divergente é recusada. O snapshot
embute definições usadas; defaults preservam compatibilidade de dados antigos.

SymbolPort identifica somente posição/direção gráfica; transformed calcula ambas
no espaço documental. Coincidência não equivale a ligação lógica. Nenhum cadastro
elétrico, associação terminal ou aresta é inferido pela GUI.

Session::begin_symbol_insertion cria draft sem ocorrência. Mouse move atualiza
somente preview; confirm_symbol_insertion usa o dispatcher/histórico e mantém
inserção repetida. Esc/foco/modal/página/meio/direito cancelam. Grade lógica e snap
continuam em mm. A associação é escolhida antes de inserir e consultável depois;
edição dessa associação fica futura.

History contém transforms, inserções e aparência. Undo/Redo valida estado esperado,
preserva IDs/alvos e desfaz/restaura definição introduzida quando não compartilhada.
Inspetor exige Aplicar transformação / Aplicar aparência; edição não gráfica ainda
encerra as pilhas. Não há exclusão genérica, Undo geral ou histórico persistido.
Homologação manual abrangente foi adiada pelo anexo atual até após Etapa 15.

## Complemento implementado na Etapa 8

PortRef identifica uma porta local de Representation. Consultas port_position,
port_binding e port_binding_candidates dão posição transformada e vínculo
compatível. BindPort é explícito e não pode trocar o endpoint de uma rota existente.
ConnectionRoute compartilha RepresentationId e mantém ConnectionId/portas/polilinha.
ConnectionJunction registra incidência e arestas auxiliares no mesmo grafo lógico.

Project::edit_connection_drawing aceita BindPort, CreateRoute, SetRoute, DeleteRoute,
CreateJunction, DeleteJunction e Transform. Retorna ConnectionDrawingDelta opaco e
compacto; restore_connection_drawing valida estado esperado para Undo/Redo atômico.
transform_representations anterior delega ao mesmo mecanismo. O método puro
ConnectionRoute::points_with_endpoints atende prévia e confirmação com a mesma rota.

Rotas não têm transform livre: mover símbolos ajusta seus extremos; editar rota
muda pontos mantendo portas. Junções que perderiam incidência causam recusa explícita.
Múltiplas representações/elétricos anteriores continuam independentes. Nenhuma
operação gráfica cria condutor, cabo ou continuidade interna de dispositivo.
Novos dados são validados no snapshot técnico, com defaults para esquemas anteriores.
