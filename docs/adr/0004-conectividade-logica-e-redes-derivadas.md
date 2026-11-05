# ADR-0004 — Conexões lógicas e redes derivadas

Data: 2026-10-27. Estado: implementado e testado automaticamente na Etapa 2;
decisões técnicas sujeitas à revisão do usuário. Homologação manual pendente.
Referências: [anexo](../internal/spec/ETAPA_02_CONECTIVIDADE_REDES.md), ADR-0001/0003.

## Contexto

O Project da Etapa 1 já controla dispositivos, funções, componentes, terminais e
pontos funcionais. A Etapa 2 exige arestas explícitas, redes derivadas, remoção e
reconstrução validada, sem GUI, condutor físico ou simulação de estados.

## Decisão

- Connection possui ConnectionId UUID v7, ProjectId e extremos ConnectionPointId.
  Menor ID é source, maior é target; esses nomes não indicam direção elétrica.
  Construtor e Serde rejeitam autoaresta e normalizam a ordem sem trocar o ID.
- Project guarda conexões em BTreeMap e expõe insert_connection, connection,
  connections e remove_connection, seguindo a nomenclatura existente.
  Existência dos pontos, projeto, ID e par duplicado são validados antes da escrita.
- Nenhuma propriedade adicional foi necessária. Connection não é alvo de EntityRef;
  as propriedades da Etapa 1 mantêm seus alvos anteriores. Erros de conexão usam
  ConnectionId diretamente. Assim remover uma conexão não deixa referências em
  propriedades. Referenciar conexões futuramente exigirá política de remoção própria.
- Um índice privado BTreeMap de adjacências simétricas permite consulta direta e
  detecção de duplicata sem percorrer todas as arestas. É reconstruível, não
  serializado, mantido apenas pelas operações do agregado e conferido por validate.
  Conexões e pontos são a única fonte persistente; não há cache persistente de redes.
- DFS iterativa, marcada na descoberta, deriva componentes conexas incluindo pontos
  isolados. Network é uma fotografia possuída, somente leitura, sem Serde ou NetworkId.
  Deve ser consultada novamente após mutação. Membros por ID crescente; redes pelo
  menor ponto. Arestas indiretas não bloqueiam uma nova aresta direta/ciclo válido.
- Funções derivam dos pontos e dispositivos das funções. Componentes associados
  derivam de vínculo explícito com a função ou de terminal mapeado pelo ponto.
  Vínculo somente com dispositivo não associa todos os seus componentes à rede.
  Essas associações não acrescentam arestas nem indicam passagem interna de corrente.
- are_connected(A,A) é true para ponto existente e erro para inexistente.
  are_devices_connected indica alguma rede compartilhada por pontos de ambos;
  para o mesmo dispositivo, true se possui pontos, false se não possui.
- Ciclos são detectados em cada componente simples conexa por |E| >= |V|;
  resultado topológico, sem ERC. Não se conectam automaticamente pontos da mesma
  bobina, contato, função, dispositivo ou terminal físico.
- Snapshot acrescenta connections como lista com default vazio para snapshots da
  Etapa 1. Rejeita IDs/pares duplicados, desconhecidos, relações inválidas e valores
  malformados. Ordem de entrada é irrelevante; a saída é canônica por IDs.

## Alternativas e consequências

Reconstruir/varrer todas as arestas a cada inserção tornaria a montagem de cadeias
quadrática. O índice simétrico custa O(E) memória e atualizações logarítmicas.
Union-Find complicaria remoções; recálculo iterativo permite divisões imediatamente.
HashMap precisaria de ordenação adicional para saída estável. Não há dependência
externa nova. BTreeMap/BTreeSet introduzem fatores logarítmicos, documentados no relatório.

Componentes são indexados por função uma vez por consulta de redes, evitando uma
varredura de todos os componentes para cada rede. Se a mesma função/componente
participa de várias redes, o volume da saída pode crescer proporcionalmente a essas
associações. Resultados não são identificadores estáveis: nomeação/identidade de
rede persistente continua decisão futura, condicionada a requisito explícito.

Conexão lógica não equivale a fio/condutor físico nem linha gráfica. Sem WireRoute,
WireJunction, ERC completo, simulação, persistência física ou migração de arquivos.
Compatibilidade de leitura com o snapshot anterior não define formato .cadcraft.
ADR-0002 permanece integralmente válido; nenhum comando Git modificador é autorizado.
