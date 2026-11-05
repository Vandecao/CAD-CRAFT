# ADR-0009 — Símbolos vetoriais e inserção documental

Data: 2026-10-29. Implementado na Etapa 7; validação e revisão registradas no
relatório da etapa. Homologação manual abrangente adiada até após Etapa 15.
Requisito: `docs/internal/spec/ETAPA_07_BIBLIOTECA_SIMBOLOS.md`.

## Contexto e decisões

O editor existente possui Representation, índices documentais, transforms em mm,
Interaction e histórico de transforms em Session. O anexo exige definições
reutilizáveis e inserção, preservando todas as entidades elétricas existentes.

1. SymbolDefinition é imutável, com SymbolDefinitionId textual tipado estável,
   versão positiva, nome/categoria, metadados, primitivas locais e SymbolPort.
   IDs da biblioteca são constantes `cad-craft.*`; não são identidades físicas.
   RepresentationId continua UUID v7. **SymbolInstance é o papel desempenhado
   pela própria Representation**, com referência opcional à definição, página,
   transform e VisualProperties. Não existe um segundo registro de instâncias.
2. RepresentationTarget::Unassociated explicita ausência de associação e só é
   válido em símbolos. Alvos elétricos existentes continuam disponíveis, sem
   duplicar propriedades. Unassociated fica fora do índice por alvo e não gera
   referências relacionadas entre símbolos independentes. Nenhum dispositivo,
   ponto lógico, conexão, rede ou condutor é criado por uma inserção gráfica.
3. SymbolPort possui ID local estável, identificação, posição, direção e papel
   opcional. Transformações produzem posição/direção documentais; proximidade
   não significa conectividade. Não há mapeamento implícito a ConnectionPoint.
4. Line, Polyline, Rectangle, Circle, Arc, Text e Anchor compõem a geometria.
   Definição valida finitude, degenerações, portas repetidas e overflow. Bounds
   locais são derivados uma vez, incluindo portas, e não são serializados na
   definição. Representation guarda a AABB local conferida contra a definição;
   editar seus limites arbitrariamente é recusado. AABB documental é conservadora
   para curvas sob rotação oblíqua. Orientações permitidas são validadas no Core.
5. Texto tem caixa nominal monoespaçada (largura = altura × 0,6 × caracteres),
   origem inferior esquerda. A GUI ajusta o mesh da fonte à caixa e transforma
   cada vértice, inclusive espelhos; egui normaliza UVs ao fim do frame, evitando
   corrupção quando o atlas cresce. Métrica nominal não promete tipografia CAD
   normativa. Símbolos iniciais são desenhos originais, sem certificação IEC.
6. SymbolLibrary::builtin usa OnceLock/BTreeMap, criada uma vez. Project armazena
   uma cópia de cada definição usada. insert_symbol_representation valida tudo
   antes de inserir definição e ocorrência; ID já registrado com conteúdo/versão
   diferente é recusado. Definições não são editadas in-place. Versões futuras
   exigirão regra explícita de atualização; não há migração automática.
7. Snapshot técnico acrescenta symbol_definitions vazio por default e símbolo
   opcional em Representation, compatível com dados anteriores. Importação reúne
   definições antes de validar representações/índices. Referências ausentes,
   duplicatas e limites divergentes são rejeitados. Saída ordenada por IDs;
   campos derivados, biblioteca global, seleção, gestos e histórico não persistem.
8. InsertionDraft integra Interaction: definição, página, alvo opcional, transform
   e posição válida; nenhum ID de ocorrência nem escrita em Project por frame.
   Clique confirmado passa pelo dispatcher/History. Inserção é repetida até Esc;
   botão direito, botão central, foco perdido, modal ou mudança de contexto também
   cancelam. Roda preserva zoom ancorado; snap usa grade lógica mesmo oculta.
9. Histórico guarda transações de transforms, inserção e aparência (visibilidade,
   camada/rótulo). Cada confirmação cria uma entrada; Undo/Redo reutiliza IDs e
   associações. Undo retira a definição se foi introduzida pela transação e não
   tem outros usos; Redo restaura-a. Estado esperado é verificado antes de mudar.
   Limites anteriores de 100 transações / 50.000 deltas permanecem. Não existe
   Undo geral: edições de página/elétricas encerram pilhas; dirty é conservador.
10. Biblioteca usa o centro de inserção, busca/categoria com cache de resultados,
    miniaturas compartilhando pintura, lista virtualizada, orientação e associação
    explícita. Ribbon abre esse painel. O inspetor mostra definição/categoria,
    versão, página/alvo, transforms e portas transformadas; confirma transformação
    e aparência por botões próprios. Não permite editar estruturas internas.
11. Hit usa distância analítica aos traços, tolerância em pontos lógicos, portas
    e âncora de inserção. Janela de seleção usa segmentos e interseções analíticas
    de curvas; texto usa sua caixa nominal. Pintura faz culling e recorte, com
    tesselação de curvas dependente do zoom (erro alvo 0,2 ponto, teto 2.048 partes).
    Consulta por página continua linear; medidas locais não garantem capacidade
    industrial. Não há clone integral de Project por frame.

## Alternativas e consequências

Instância paralela duplicaria seleção, índices e identidade. Referência exclusiva
a biblioteca externa não garantiria reconstrução do snapshot. Snapshot integral
por operação ampliaria custo e poderia restaurar domínio fora do escopo. A opção
aditiva preserva os contratos anteriores e limita a ampliação de histórico.

Não implementados: editor de definições, exclusão genérica de instância, associação
editável após inserção, múltiplas versões simultâneas do mesmo ID, biblioteca de
fabricante, roteamento, ERC, formato final, migrações físicas ou IA da Etapa 17.
Biblioteca inicial contém dez exemplos; não representa catálogo normativo completo.
Política Git manual do ADR-0002 preservada; nenhum commit/push é autorizado.
