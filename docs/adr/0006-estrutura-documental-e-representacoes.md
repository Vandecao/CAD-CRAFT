# ADR-0006 — Estrutura documental e representações

Data: 2026-10-27. Estado: implementado na Etapa 4, testado automaticamente;
decisões técnicas sujeitas à revisão do usuário. Homologação manual pendente.

## Contexto

O prompt da Etapa 4 autoriza organização documental e representações no Core,
sem GUI e preservando as entidades elétricas e snapshots das Etapas 1–3.
Project/Document/Page, PageId e geometria em mm já existem. ADRs anteriores
permanecem válidos; este ADR amplia apenas seus contratos documentais.

## Decisão

Project conserva seus registros e acrescenta identificação textual opcional e
representações em BTreeMap. Não há segundo modelo de projeto nem grafo elétrico.
Document continua agrupando páginas por DocumentId. Page mantém seu UUID como
identidade, e number como identificação legível sem exigência de unicidade normativa.

Page recebe ordem opcional e DocumentStructure (caminhos funcional e de localização
independentes, com DesignationPart). Caminhos vazios indicam ausência de classificação.
Não equivalem a FunctionId, DeviceId ou designação do dispositivo. A árvore é derivada
por documento/eixo em uma lista pré-ordenada de PageGroup: path identifica o grupo,
o caminho sem a última parte identifica o pai e pages contém apenas folhas diretas.
Ancestrais aparecem mesmo sem páginas diretas. Não há entidade/cadastro de grupos.
Grupos ordenam lexicalmente pelos segmentos; páginas dentro de cada grupo seguem
a ordem documental. Uma árvore vazia não cria dados para projetos antigos.

ordered_pages ordena por presença de ordem explícita, valor da ordem e PageId.
Ausência de ordem é preservada, e empates importados têm desempate por ID.
reorder_pages exige permutação completa das páginas de um documento e escreve
posições contíguas baseadas em zero. Exclusão pode deixar lacunas válidas.
pages/pages_in_document mantêm a ordenação por ID das etapas anteriores.

Orientação é derivada de largura/altura, evitando fonte redundante: Portrait,
Landscape ou Square. As dimensões reais são as dimensões armazenadas. Size2D
mantém dimensões não negativas, inclusive zero, por compatibilidade com a Etapa 1;
uma folha degenerada não constitui formato de impressão homologado.

Representation tem UUID v7 tipado (14º tipo de ID), ProjectId, PageId, alvo elétrico,
Transform2D, limites locais opcionais e VisualProperties. RepresentationTarget
aceita Device, Function, Component, Terminal, ConnectionPoint, Connection,
Conductor, Cable, CableCore e TerminalStrip. Não admite projeto/página/representação
como alvo. EntityRef continua sem Connection e sem Representation: não são alvos
genéricos de TechnicalProperties. O estilo tem visibilidade, camada i32 e rótulo
opcional validado; não duplica características elétricas do alvo.

Múltiplas ocorrências do mesmo alvo, inclusive na mesma página, são válidas.
PageElement legado permanece uma referência elétrica não gráfica; nenhuma
representação é gerada automaticamente a partir dele. Inserir/excluir/mover uma
representação não modifica PageElement, alvo elétrico, rede ou fiação.

Transform2D aplica espelhamento dos componentes locais X/Y, rotação em graus
anti-horária e translação em mm, nessa ordem. Ângulo finito é normalizado para
[0,360); ângulos ortogonais têm seno/cosseno exatos. A inversa desfaz na ordem
oposta. Origem inferior esquerda, X à direita, Y para cima; conversão de tela
e inversão do Y pertencem exclusivamente à GUI futura.

Limites da representação são AABB dos quatro cantos transformados. Sem limites
locais, retorna retângulo degenerado na âncora, sem presumir tamanho de símbolo.
DocumentBounds separa folha e união do conteúdo, incluindo invisíveis. Não há
recorte na folha nem restrição de viewport. Entradas e resultados devem ser finitos;
overflow produz Result::Err. A união de elementos muito distantes pode exceder
f64, mesmo quando cada representação é válida; a consulta retorna erro.

Índices privados por página e alvo usam BTreeMap/BTreeSet. São reconstruídos na
importação e comparados por validate, nunca serializados. Mutação valida antes
de escrever registros/índices. related_representations retorna as outras ocorrências
do mesmo alvo por ID; não infere equivalência por nome ou continuidade elétrica.

## Remoção e integridade

- Página: bloqueada por representações ou referências em propriedades de outras
  entidades, incluindo Project e registros físicos. Referência da própria página
  não bloqueia, pois desaparece junto dela. Não há cascata.
- Representação: remoção explícita mantém todas as entidades elétricas.
- Connection: além do bloqueio por condutor da Etapa 3, bloqueada por representação.
  ConnectionInUse informa ambos os motivos possíveis. Desvincular/remover antes.
- Demais entidades elétricas e Document: continuam sem API genérica de exclusão.
- IDs novos ausentes/duplicados, projeto estrangeiro, página/alvo inexistente,
  transformações e limites inválidos são rejeitados por APIs/importação.

## Compatibilidade e consequências

O DTO acrescenta representations com default vazio, identification com None,
Page.order com None e Page.structure com caminhos vazios. Não inventa folhas,
grupos, identificação ou representações para snapshots antigos. Listas persistem
por ID, detectam duplicações e aceitam referências adiantadas após coleta completa.
Retorno ao snapshot das etapas antigas não é garantido: leitores antigos rejeitam
campos novos. Não há formato .cadcraft, schema final, migração física ou salvamento.

Alternativas descartadas: aninhar objetos elétricos em páginas duplicaria identidade;
um cadastro separado de grupos acrescentaria IDs/CRUD sem necessidade nesta etapa;
orientação persistida duplicaria dimensões; EntityRef ampliado para conexões exigiria
novas regras para propriedades e remoção. A opção implementada é aditiva e restrita.

Limites: sem símbolos, segmentos de fios, renderização, seleção exata, vínculos de
grupo estáveis após renomeação, edição global de caminhos ou Undo/Redo. Consultas
documentais por documento percorrem páginas; árvore materializa prefixos dos caminhos.
Não existe limite de recursos de importação; persistência futura deverá defini-lo.
Contratos para consumo estão em `docs/internal/spec/CONTRATOS_DOCUMENTAIS_GUI.md`.
