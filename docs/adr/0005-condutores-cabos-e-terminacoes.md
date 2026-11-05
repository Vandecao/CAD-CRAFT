# ADR-0005 — Condutores, cabos e terminações físicas

Data: 2026-10-27. Estado: implementado e testado automaticamente na Etapa 3;
decisões técnicas do agente sujeitas à revisão do usuário. Homologação manual pendente.
Referências: [anexo](../internal/spec/ETAPA_03_CONDUTORES_CABOS_BORNES.md), ADR-0003/0004.

## Contexto

Terminal já é entidade física pertencente a Device ou Component. ConnectionPoint
é funcional e pode mapear esse Terminal. Connection liga pontos funcionais e redes
são derivadas somente dessas conexões. Faltavam posições físicas, fios, cabos,
veias e agrupamentos de bornes, com rastreabilidade sem alterar essa topologia.

## Decisão

- Acrescentar ConductorId, CableId, CableCoreId e TerminalStripId à macro UUID v7.
  Treze categorias de IDs no total; sem implementação UUID adicional.
- Terminal mantém proprietário, designação e propriedades. Posições são códigos
  DesignationPart únicos, cadastrados explicitamente. PhysicalTermination combina
  TerminalId e posição: nenhum segundo conceito de Terminal ou UUID desnecessário.
  Dois extremos no mesmo terminal e em posições diferentes são permitidos; isso
  não declara continuidade elétrica interna. Snapshots antigos têm posições vazias.
- Conductor mantém origem/destino documentais, designação e identidade própria.
  Rejeita o mesmo extremo nas duas pontas. No mesmo par físico não direcionado,
  condutores paralelos exigem designações diferentes. Designações podem se repetir
  em outros pares; UUID continua a identidade. Não se impõe capacidade de ocupação
  do borne sem requisitos de catálogo/ERC.
- CrossSectionMm2 é finito e estritamente positivo; LengthMm é finito e não negativo,
  com conversão explícita de metros e rejeição de overflow. Comprimento pode faltar
  e não é calculado do esquema. Cor e identificação são independentes de função.
  ConductorMarking exige pelo menos cor ou código validado. Material opcional e
  função usam enums controlados, sem inferência normativa.
- Cable declara capacidade nominal positiva. CableCore contém CableId, identificação
  única no cabo, marcação, seção opcional e ConductorId opcional. Cadastro de menos
  veias que a capacidade é incompleto, mas válido; ultrapassá-la é erro estrutural.
  Veias sem condutor são representáveis. Seção declarada deve coincidir com a do
  condutor; cores devem coincidir quando ambas declaradas. Numeração/identificação
  da veia e do fio não precisam coincidir, pois têm escopos diferentes.
- A única fonte persistente do vínculo cabo–condutor é CableCore.conductor_id.
  Um condutor pode ocupar no máximo uma veia; Project deriva sua referência opcional
  ao cabo. Não se duplica CableId ou CableCoreId em Conductor. Cabos e veias não unem redes.
- TerminalStrip agrupa TerminalId em ordem explícita de associação. Um terminal
  pertence a no máximo uma régua; seu proprietário físico permanece Device/Component.
  Não há união de lados, níveis, bornes PE, pontes ou acessórios por classificação.
- ConductorConnection, opcional no condutor, contém ConnectionId e source_point.
  Source_point identifica qual extremo da conexão corresponde à origem física;
  o outro ponto é derivado da conexão. Ambos devem mapear os terminais físicos
  correspondentes pelo ConnectionPoint.terminal_id já existente. Essa orientação
  documental não indica direção elétrica. Diferentes posições do mesmo terminal
  não são automaticamente mapeadas entre si, nem recebem continuidade interna.
- Uma conexão pode documentar vários condutores físicos distintos; um condutor
  documenta no máximo uma conexão direta. Nenhuma conexão ou condutor é criado
  automaticamente. Estados sem associação são válidos.
- Assign exige alvo livre; unassign é explícito e retorna o vínculo anterior, ou
  None se já livre. Troca exige liberar primeiro; cada operação individual é atômica.
  Não se promete transação para uma sequência de duas chamadas.
- `remove_connection` retorna ConnectionInUse enquanto houver condutor associado.
  O usuário da API deve desvincular antes de remover. Isso amplia o contrato do
  ADR-0004 sem modificar comportamento de conexões sem documentação física.
- Entidades físicas não são removíveis e integram EntityRef/TechnicalProperties.
  Connection continua fora de EntityRef. CrossSection em propriedades de condutor
  ou veia é rejeitada: seção nominal tem fonte única no campo tipado.
- Índices privados de nomes/pares, veias por cabo, condutor→veia, terminal→régua,
  conexão→condutores e terminal→condutores são reconstruíveis, não serializados.
  Operações conferem todos os erros recuperáveis antes da escrita. validate reconstrói
  e compara índices. BTreeMap/BTreeSet mantêm ordem determinística e evitam varreduras
  completas em inserções simples. Consultas devolvem referências ou fotografias de IDs.
- Snapshot acrescenta quatro listas com default vazio; mantém compatibilidade de
  leitura das Etapas 1/2. Listas detectam duplicações, referências são resolvidas
  após coleta de todos os registros e índices são reconstruídos. Ordem de entrada
  dos registros não afeta saída; ordem dos bornes na régua é dado e é preservada.

## Alternativas e consequências

Usar ConnectionPointId como extremo físico confundiria função e terminação.
Uma nova entidade de terminal duplicaria o modelo anterior. Um UUID para cada
posição seria útil se renomeação/reuso independente fosse requisito, mas não é
necessário agora. A identidade composta implica que renomear/remover posições
exigirá contrato de atualização referencial futuro; essas operações não são expostas.

Listas inversas persistentes duplicariam verdade. Consultas por varredura em cada
inserção tornariam milhares de associações quadráticas. Índices derivados custam
memória linear e fatores logarítmicos; importação/validate podem reconstruí-los.
O cadastro parcial de cabo evita confundir engenharia incompleta com corrupção.

Não implementados: exclusão genérica, reordenação/remoção de bornes, alterações
genéricas de extremos/características, múltiplos níveis especializados, continuidade
interna automática, normas, simulação, dimensionamento, gráfico, WireRoute,
WireJunction, ERC completo, formato .cadcraft, migrações ou backup versionado.
ADR-0002 permanece integralmente válido. Etapa 4 não iniciada.
