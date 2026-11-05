# ADR-0011 — identificação, referências e potenciais

Estado: implementado, testado e revisado na Etapa 9.
Homologação manual abrangente permanece após a Etapa 15.

## Contexto

A rota da Etapa 8 promovia dobras automáticas a pontos persistentes após cada
movimento. A Etapa 9 requer identificação e referências cruzadas sem criar
conectividade pela igualdade de nomes ou modificar condutores físicos.

## Decisão

- Separar intenção de roteamento (pontos/autoria/orientação) da polilinha derivada.
  Rotas antigas preservam conservadoramente seus pontos como restrições.
- Manter DeviceId, FunctionId e RepresentationId como identidades; Designation
  estruturada pode mudar sem trocar IDs. Papel funcional principal/auxiliar é
  explícito e não inferido do nome ou tipo da função.
- Derivar referências de ocorrências e páginas reais. Navegação usa IDs,
  posição em milímetros e a câmera/seleção existentes.
- Vincular marcadores de interrupção e potencial a Representation e, quando
  escolhido, a ConnectionPoint existente. Repetição textual não une redes.
- Guardar grupos estrela/cadeia por IDs e ordem explícita. Associação documental
  não implica conexão; continuidade opcional cria arestas explícitas e reversíveis
  no grafo existente, sob posse do grupo.
- Usar deltas de entidades tocadas no histórico e defaults validados no snapshot.

## Consequências

Referências acompanham movimentos e edição documental sem regravar alvos.
Conflitos de designação e potencial são diagnosticados; inserções novas recusam
duplicidade total. A proveniência de pontos legados não pode ser reconstruída.
Não há coordenadas de moldura/coluna nem cálculo de grandezas elétricas nesta
etapa. O formato final `.cadcraft` permanece futuro.
