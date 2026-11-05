# ADR-0019 — biblioteca externa, editor e versões de símbolos

Estado: implementado e testado automaticamente na Etapa 17.
Homologação manual geral na Etapa 28.

## Contexto

Os símbolos eram definições imutáveis embutidas (ADR-0009). A Fase B pede
símbolos do usuário (criar, editar, guardar, trocar), primitivas ampliadas,
textos que mostram a designação e importação dos elementos do QElectroTech.
Decisões do usuário: `.cadsym` por símbolo; `.elmt` a 0,5 mm/px ajustável;
atualização de símbolos do projeto nesta etapa; molduras sem estilos novos.

## Decisão

- **Estilo por índice de primitiva** (`SymbolDefinition.styles`) em vez de
  mudar as variantes existentes: construtores antigos e a biblioteca
  embutida continuam iguais; estilos padrão não são gravados.
- **`Ellipse` e `DynamicText`** são variantes novas de `SymbolPrimitive`.
  O valor do texto dinâmico é derivado do agregado a cada desenho
  (`Project::symbol_text`); nada é gravado na ocorrência.
- **Versões**: editar gera `edited` (mesmo ID, versão + 1). O projeto guarda
  uma versão por ID; `update_symbol_definition` troca todas as ocorrências de
  uma vez, mantendo IDs e vínculos porta→ponto, e recusa quando uma porta em
  uso deixa de existir, uma orientação deixa de ser permitida ou o projeto
  resultante fica inválido (ex.: fio ligado a porta deslocada).
- **Biblioteca**: pasta de arquivos `.cadsym` (ZIP com manifesto, mesmo
  leitor defensivo); a sessão guarda as definições lidas; o catálogo de
  inserção junta projeto → biblioteca → embutidos (o projeto prevalece).
- **Editor** com rascunho e Desfazer/Refazer próprios; só "Atualizar no
  projeto" entra no histórico do projeto (`Transaction::Symbols`).
- **`.elmt`** convertido para `SymbolDefinition` (`interchange/elmt.rs`);
  a coleção do QElectroTech não é versionada nem usada como dado de IA.
- `.cadcraft` formato 5 / `min_reader` 5.

## Consequências

- Versões da Etapa 16 recusam arquivos da 17.
- Ocorrências sempre usam a versão registrada no projeto; a biblioteca só
  muda o projeto por "Atualizar símbolos do projeto".
- Preenchimento de polígonos não convexos é aproximado.
