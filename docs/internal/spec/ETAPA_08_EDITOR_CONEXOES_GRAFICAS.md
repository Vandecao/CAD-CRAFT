# CAD-CRAFT — ETAPA 8

## EDITOR GRÁFICO DE CONEXÕES ELÉTRICAS E REDES MULTIFILARES

### Rust + egui/eframe — Windows Desktop

# 0. AUTORIZAÇÃO

Implementar integralmente a Etapa 8 do CAD-CRAFT.

Diretório:

`C:\Dev\cad-craft`

Branch:

`main`

Repositório:

`Vandecao/CAD-CRAFT`

Esta é uma nova conversa de desenvolvimento.

Recuperar todo o contexto do repositório e da memória persistente.

O usuário autorizou a evolução funcional do CAD-CRAFT tendo o EPLAN Electric P8 como referência principal de ferramentas, comandos e fluxos de trabalho.

A aparência da Etapa 5 já foi aprovada.

Agora devemos priorizar o comportamento funcional de um ECAD industrial profissional.

## REGRAS INEGOCIÁVEIS

1. Não executar commit.
2. Não executar push.
3. Não iniciar a Etapa 9.
4. Não solicitar homologação manual ao usuário.
5. Executar todos os testes automatizados.
6. Preservar os 272 testes existentes.
7. Utilizar TDD sempre que possível.
8. Executar revisão independente quando disponível.
9. Atualizar a memória persistente.
10. Preservar a arquitetura elétrica existente.
11. Não copiar código ou recursos proprietários do EPLAN.
12. Não antecipar a implementação do assistente de IA da Etapa 17.

O usuário realizará o commit e o push manualmente após a conclusão técnica.

A homologação manual abrangente permanece programada para depois da Etapa 15.

---

# 1. RECUPERAÇÃO DO CONTEXTO

Ler obrigatoriamente:

* AGENTS.md.
* docs/internal/memory/HANDOFF.md.
* docs/internal/memory/CURRENT_STATE.md.
* docs/internal/memory/ROADMAP.md.
* docs/internal/memory/ARCHITECTURE.md.
* docs/internal/memory/DECISIONS.md.
* docs/internal/memory/KNOWN_ISSUES.md.
* ADRs existentes.
* Relatórios das Etapas 4, 5, 6 e 7.
* Contratos documentais.
* Especificação da Etapa 7.
* Código do Core elétrico.
* Código da biblioteca de símbolos.
* Código do viewport.
* Código do histórico transacional.

Executar:

```powershell
git status -sb
git log -3 --oneline
```

Confirmar que a Etapa 7 foi commitada.

Não realizar sincronização remota automaticamente.

## REFERÊNCIAS DO EPLAN

Procurar os materiais visuais disponibilizados anteriormente pelo usuário.

Local histórico das referências:

`C:\Dev\cad-craft\.local\references\eplan\`

O conjunto original possuía capturas de tela e vídeos demonstrando interações do EPLAN.

Verificar quais arquivos realmente estão disponíveis no ambiente atual.

**Essas referências não servem apenas para reproduzir aparência.**

Analisar especialmente:

* Ferramentas disponíveis.
* Comandos da Ribbon.
* Menus contextuais.
* Sequência de operações.
* Inserção de símbolos.
* Inserção de conexões.
* Interações com pontos de conexão.
* Manipulação de linhas.
* Seleção.
* Navegação.
* Propriedades.
* Atalhos.
* Convenções de edição elétrica.

Se alguma referência estiver indisponível, registrar essa limitação.

Não inventar observações sobre vídeos não examinados.

Consultar também, quando necessário, a documentação pública oficial do EPLAN Electric P8, especialmente sobre:

* Connections.
* Autoconnecting.
* Connection symbols.
* Connection definition points.
* T-nodes.
* Junctions.
* Electrical networks.

Não copiar arquivos proprietários nem reproduzir código protegido.

---

# 2. OBJETIVO PRINCIPAL

Implementar um editor gráfico capaz de criar, visualizar e modificar conexões elétricas entre os pontos de conexão dos símbolos existentes.

O usuário deverá conseguir construir esquemas multifilares utilizando o mouse.

O sistema deverá distinguir rigorosamente:

1. Representação gráfica da conexão.
2. Conectividade elétrica lógica.
3. Rede elétrica derivada.
4. Condutores e fiação física.

Esses conceitos já existem parcialmente no Core e não devem ser confundidos.

A representação gráfica não deve criar automaticamente um condutor físico.

A conectividade lógica deve ser modificada somente por operações validadas.

---

# 3. AUDITORIA DO CORE EXISTENTE

Antes de implementar novas estruturas, examinar:

* Electrical Connections.
* ConnectionPoints.
* Nets.
* Representations.
* SymbolDefinition.
* SymbolInstance.
* SymbolPorts.
* Project.
* DocumentIndex.
* Histórico transacional.
* Serialização.

Determinar quais estruturas já permitem estabelecer conexões entre terminais elétricos.

Identificar as APIs existentes para:

* Criar conexão.
* Remover conexão.
* Consultar conexão.
* Validar endpoints.
* Atualizar redes.
* Identificar dispositivos relacionados.

**Não criar um segundo modelo de conectividade.**

Se for necessário acrescentar informações gráficas às conexões existentes, criar estruturas complementares corretamente associadas aos IDs do domínio.

---

# 4. PONTOS DE CONEXÃO DOS SÍMBOLOS

A Etapa 7 implementou pontos de conexão gráficos.

Agora será necessário utilizá-los na edição de esquemas.

## Requisitos

Cada ponto deverá possuir:

* Identidade estável.
* Coordenadas locais.
* Coordenadas documentais transformadas.
* Orientação.
* Identificação visual.
* Associação à ocorrência.
* Eventual vínculo explícito a um ponto elétrico do Core.

## Associação elétrica

Um ponto gráfico não deve ser automaticamente confundido com um ConnectionPoint do domínio.

Implementar um mecanismo validado de associação quando necessário.

Se uma ocorrência não possuir entidade elétrica associada:

* Não inventar conexões lógicas.
* Informar a ausência de associação.
* Permitir somente operações gráficas explicitamente diferenciadas, caso suportadas.

O editor deve impedir que uma conexão aparentemente válida seja silenciosamente registrada como rede elétrica sem endpoints válidos.

---

# 5. FERRAMENTA DE CONEXÃO

Implementar uma ferramenta específica para desenhar conexões elétricas.

Integrar à Ribbon existente.

Local preferencial:

**Guia Conexões.**

## Fluxo esperado

1. Usuário ativa a ferramenta Conexão.
2. Posiciona o cursor sobre um ponto de conexão.
3. O ponto disponível recebe destaque.
4. Clica para iniciar a conexão.
5. O mouse apresenta uma prévia do caminho.
6. O usuário pode definir pontos intermediários.
7. Seleciona o ponto de destino.
8. O sistema valida os endpoints.
9. A conexão é confirmada.
10. O modelo documental é atualizado.
11. A rede elétrica derivada é atualizada.
12. A operação é registrada no histórico.

## Cancelamento

Esc cancela a operação em andamento.

Cancelar não pode modificar:

* Project.
* Conexões.
* Redes.
* Histórico.
* IDs persistentes.

## Pré-visualização

Durante o movimento:

* Não criar conexões permanentes.
* Não criar transações por frame.
* Não alterar redes elétricas.
* Não criar IDs definitivos desnecessariamente.

---

# 6. DESENHO ORTOGONAL

O editor deverá oferecer roteamento ortogonal.

Priorizar segmentos:

* Horizontais.
* Verticais.

Permitir criar conexões com múltiplos segmentos.

## Comportamento

Durante o desenho, apresentar uma prévia da rota.

O usuário deverá conseguir controlar:

* Início.
* Destino.
* Pontos intermediários.
* Direção inicial.
* Mudanças de direção.

Quando apropriado, oferecer alternativas determinísticas de rota em L.

Não exigir que o usuário desenhe manualmente cada pequeno segmento.

## Requisitos geométricos

* Coordenadas em milímetros.
* Snap independente do zoom.
* Segmentos alinhados.
* Ausência de segmentos inválidos.
* Tratamento de segmentos de comprimento zero.
* Caminhos determinísticos.
* Transformações consistentes.

Não implementar um roteador automático industrial completo nesta etapa.

---

# 7. PONTOS DE CONEXÃO E SNAP

Implementar snap específico para pontos de conexão.

Diferenciar:

* Snap à grade.
* Snap a ponto elétrico.
* Snap a vértice.
* Snap a segmento.
* Snap a junção.

## Prioridades

Definir uma política determinística.

Quando o cursor estiver próximo de um ponto de conexão válido, esse ponto deverá poder receber prioridade sobre o snap à grade.

A tolerância visual deverá ser calculada corretamente em pixels lógicos.

A distância real entre pontos não pode variar conforme o zoom.

## Regra principal

Uma grade configurada para 4 mm permanece com resolução de 4 mm em qualquer zoom.

A densidade dos pontos renderizados não interfere no cálculo do snap.

---

# 8. JUNÇÕES ELÉTRICAS

Implementar representação explícita de junções.

Priorizar:

* Junção simples.
* Junção em T.
* Junção com múltiplas ramificações.

Representar junções conectadas por marcação apropriada.

## Regra obrigatória

**Cruzamento gráfico não significa conexão elétrica.**

Dois fios podem se cruzar sem compartilhar uma rede.

A conexão entre eles deve ser determinada por um elemento ou operação elétrica explícita.

Não utilizar somente coincidência de coordenadas para criar conectividade.

## Junções

Quando o usuário criar uma junção:

* Validar endpoints ou segmentos envolvidos.
* Atualizar topologia.
* Atualizar representação.
* Preservar IDs quando possível.
* Registrar uma transação reversível.

Evitar redes inconsistentes, ramificações órfãs ou conexões duplicadas.

---

# 9. CRUZAMENTOS SEM CONEXÃO

Suportar cruzamentos sem conexão elétrica.

O usuário deverá distinguir visualmente:

1. Dois condutores que se cruzam.
2. Dois condutores unidos por uma junção.

A convenção visual deverá ser consistente e configurável conforme a arquitetura existente.

Não inferir junções apenas porque segmentos se interceptam.

Criar testes específicos para:

* Cruzamento sem junção.
* Cruzamento com junção.
* Segmentos coincidentes.
* Junção em extremidade.
* Junção intermediária.
* Mudança posterior da geometria.

---

# 10. CONEXÕES ASSISTIDAS

Tomar como referência funcional o conceito de autoconnecting do EPLAN.

Implementar, se compatível com o Core, uma infraestrutura de **sugestão de conexão por alinhamento**.

Exemplo:

Dois pontos elétricos estão alinhados horizontalmente ou verticalmente.

O editor pode sugerir graficamente uma conexão.

## Requisitos de segurança

* Alinhamento visual não cria conexão permanente por si só.
* A sugestão deve ser distinguível de uma conexão confirmada.
* A confirmação deve passar pelas APIs do Core.
* Endpoints devem ser válidos.
* Não criar conectividade por mera proximidade.
* Não duplicar conexões existentes.

Priorizar uma versão assistida e determinística.

Se a implementação integral comprometer o escopo, registrar a automação avançada como evolução futura.

Não marcar como concluída uma funcionalidade implementada apenas como prévia.

---

# 11. EDIÇÃO DE CONEXÕES EXISTENTES

Após criar uma conexão, permitir:

* Selecionar.
* Destacar.
* Consultar propriedades.
* Modificar geometria.
* Adicionar ponto intermediário.
* Remover ponto intermediário.
* Movimentar segmento.
* Excluir conexão.
* Desfazer.
* Refazer.

## Hit-testing

A seleção deve considerar a distância real ao traço renderizado.

Não utilizar apenas o retângulo delimitador da conexão.

Preservar tolerância visual adequada em diferentes níveis de zoom.

## Movimento de segmentos

Ao mover um segmento:

* Atualizar sua geometria.
* Preservar seus endpoints elétricos.
* Manter a rota válida.
* Evitar alterações inesperadas de topologia.

Quando a operação não puder ser realizada sem violar os vínculos elétricos, recusá-la de forma explícita.

---

# 12. MOVIMENTAÇÃO DE SÍMBOLOS CONECTADOS

Este é um requisito importante.

Quando um símbolo associado a uma conexão for movimentado:

* Seus pontos de conexão mudam de posição.
* Os endpoints gráficos devem acompanhar a nova posição.
* A geometria da conexão deve permanecer consistente.
* A conectividade elétrica válida não deve ser perdida silenciosamente.

## Estratégia

Avaliar:

* Recalcular segmentos terminais.
* Preservar pontos intermediários.
* Ajustar rota ortogonal.
* Detectar impossibilidade de manutenção da geometria.

Se uma edição exigir desconexão lógica, ela deverá ser uma operação explícita.

Não permitir que uma simples movimentação destrua conexões elétricas sem aviso.

## Operações em grupo

Quando símbolos conectados forem movimentados juntos:

* Preservar a relação entre eles.
* Evitar distorção desnecessária dos caminhos.
* Manter os endpoints vinculados por identidade.

---

# 13. CONECTIVIDADE ELÉTRICA

Reutilizar o Core desenvolvido nas etapas anteriores.

O editor deverá permitir consultar:

* Origem.
* Destino.
* Pontos elétricos associados.
* Dispositivos relacionados.
* Rede elétrica.
* Identificador da conexão.

## Redes derivadas

Ao criar ou remover uma conexão lógica:

* Atualizar a rede derivada pelo mecanismo existente.
* Evitar caches desatualizados.
* Preservar determinismo.
* Não criar redes independentes no módulo gráfico.

## Separação obrigatória

Uma conexão lógica não equivale automaticamente a:

* Um fio físico instalado.
* Uma veia de cabo.
* Um condutor dimensionado.
* Um percurso físico de instalação.

Essas associações dependem das informações específicas do Core.

---

# 14. PROPRIEDADES DAS CONEXÕES

Integrar ao painel de propriedades existente.

Exibir os campos realmente disponíveis no domínio.

Priorizar:

* ID.
* Origem.
* Destino.
* Rede associada.
* Identificação.
* Tipo de conexão.
* Página.
* Informações elétricas existentes.
* Dados de condutor físico somente quando vinculados explicitamente.

Não criar valores fictícios.

Não marcar parâmetros de engenharia como calculados sem uma regra verificável.

As edições deverão utilizar comandos validados.

---

# 15. SÍMBOLOS ESPECIAIS DE CONEXÃO

Criar a infraestrutura gráfica necessária para símbolos próprios de conexões.

Considerar:

* Ângulos.
* Junções em T.
* Junções duplas.
* Pontes.
* Marcadores de conexão.

Priorizar aqueles necessários para os esquemas multifilares desta etapa.

Não duplicar o modelo de SymbolDefinition existente.

Quando houver regras topológicas específicas, modelá-las explicitamente.

Os símbolos de conexão não devem ser confundidos automaticamente com dispositivos físicos.

Recursos avançados, como pontos de interrupção entre páginas e definições completas de potencial, podem ficar para etapas posteriores, desde que a arquitetura preserve sua futura implementação.

---

# 16. RIBBON E MENUS

Utilizar a interface implementada na Etapa 5.

Integrar os comandos nas guias existentes.

## Guia Conexões

Priorizar:

* Inserir conexão.
* Inserir junção.
* Inserir ângulo.
* Editar conexão.
* Excluir conexão.
* Propriedades.
* Destacar rede.
* Atualizar conexões, quando necessário.

## Guia Inserir

Disponibilizar símbolos especiais de conexão quando apropriado.

## Menu contextual

Sobre uma conexão:

* Selecionar.
* Propriedades.
* Editar rota.
* Adicionar vértice.
* Remover vértice.
* Excluir.
* Destacar rede.

## Regras

* Utilizar o dispatcher central.
* Não criar comandos decorativos.
* Não implementar ações independentes com semânticas divergentes.
* Preservar a Ribbon e o Backstage.
* Não reconstruir o layout aprovado.

---

# 17. CURSORES E INTERAÇÃO

Implementar cursores contextuais.

Durante o desenho de uma conexão:

* Cursor de ferramenta apropriado.

Sobre ponto de conexão:

* Destaque da porta.
* Indicação de endpoint válido.

Sobre segmento:

* Cursor de seleção ou edição.

Durante movimentação de vértice:

* Cursor correspondente à operação.

Durante pan:

* Mão aberta/fechada.

## Atalhos

Esc cancela o comando atual.

Ctrl+Z desfaz.

Ctrl+Y refaz.

Preservar comportamento de campos textuais.

Evitar conflitos entre:

* Seleção.
* Inserção de símbolos.
* Desenho de conexões.
* Pan.
* Zoom.
* Edição de propriedades.

---

# 18. UNDO/REDO TRANSACIONAL

Estender o histórico existente.

Operações desta etapa:

* Criar conexão.
* Remover conexão.
* Alterar rota.
* Inserir junção.
* Remover junção.
* Alterar geometria de segmento.
* Movimentar símbolos com atualização gráfica associada.

## Requisitos

Cada operação confirmada deve produzir uma única transação lógica.

Operações compostas devem ser atômicas.

Em caso de erro:

* Nenhuma mutação parcial.
* Nenhuma rede inconsistente.
* Nenhum histórico inválido.

Undo/Redo deve restaurar:

* Identidades.
* Geometria.
* Endpoints.
* Topologia.
* Associações documentais.

Evitar snapshots integrais do Project por frame.

Não registrar pré-visualizações.

---

# 19. PERSISTÊNCIA

Preservar compatibilidade com os snapshots anteriores.

Serializar os novos elementos gráficos e vínculos necessários.

Verificar:

* IDs.
* Páginas.
* Endpoints.
* Junções.
* Roteamento.
* Definições de símbolo.
* Associações elétricas.

A importação deve validar referências.

Dados incompletos ou inválidos não podem substituir silenciosamente um projeto válido.

Não introduzir um formato final `.cadcraft` nesta etapa se ele não estiver previsto no planejamento aprovado.

---

# 20. CONSIDERAÇÕES DE ENGENHARIA

O CAD-CRAFT deverá funcionar como um ECAD, não apenas como um programa de desenho.

Portanto:

* Geometria não substitui conectividade.
* Conexões devem possuir identidade.
* Redes devem ser derivadas corretamente.
* Junções precisam de semântica explícita.
* Cruzamentos não criam redes.
* Movimentação não deve destruir vínculos.
* Undo/Redo deve preservar consistência.
* Propriedades devem refletir o Core.
* Representações devem compartilhar identidades elétricas corretamente.

Não criar regras elétricas artificiais apenas para facilitar a renderização.

---

# 21. DESEMPENHO

Considerar projetos com milhares de conexões.

Evitar:

* Varreduras integrais por frame sem necessidade.
* Recalcular todas as redes durante hover.
* Recriar a geometria de todas as conexões durante mouse move.
* Clonar projetos inteiros.
* Criar transações a cada atualização de cursor.
* Renderizar segmentos completamente fora do viewport.

Avaliar índices ou caches somente quando justificados.

Medir cenários representativos.

Não declarar desempenho industrial com base apenas em microbenchmarks.

---

# 22. TESTES AUTOMATIZADOS

Preservar os 272 testes existentes.

Adicionar testes para:

1. Identificação de portas.
2. Associação de porta gráfica a endpoint elétrico.
3. Endpoint sem associação elétrica.
4. Criação de conexão.
5. Validação de origem e destino.
6. Prevenção de conexão duplicada.
7. Cancelamento.
8. Pré-visualização.
9. Rotas ortogonais.
10. Segmentos inválidos.
11. Snap.
12. Snap com grade oculta.
13. Coordenadas negativas.
14. Junção em T.
15. Junção múltipla.
16. Cruzamento sem conexão.
17. Cruzamento com conexão explícita.
18. Hit-testing.
19. Seleção.
20. Edição de vértices.
21. Movimentação de segmentos.
22. Exclusão.
23. Undo.
24. Redo.
25. Operações atômicas.
26. Movimentação de símbolos conectados.
27. Movimentação em grupo.
28. Preservação de endpoints.
29. Atualização de redes.
30. Separação de condutores físicos.
31. Propriedades.
32. Ribbon.
33. Menus contextuais.
34. Compatibilidade documental.
35. Round-trip de serialização.
36. IDs estáveis.
37. Zoom.
38. Pan.
39. Diferentes níveis de DPI.
40. Integridade elétrica após operações sucessivas.

Os testes deverão validar o comportamento real.

Não remover os testes anteriores.

---

# 23. VALIDAÇÃO

Executar:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

Executar também:

```powershell
cargo build --release -p cad-gui --bin cad-gui --locked

git diff --check
git status --short
```

Todas as verificações devem passar.

Corrigir falhas antes de concluir.

Não desabilitar testes nem verificações estáticas.

---

# 24. REVISÃO INDEPENDENTE

Executar revisão independente quando disponível.

Prioridades:

1. Consistência da conectividade elétrica.
2. Integridade das redes.
3. Cruzamentos e junções.
4. Sincronização entre geometria e domínio.
5. Movimentação de símbolos conectados.
6. Undo/Redo.
7. Persistência.
8. Integridade de IDs.
9. Desempenho.
10. Regressões da Etapa 7.

Documentar os achados e correções.

---

# 25. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md`

Criar especificação e plano correspondentes em:

* docs/internal/spec/
* docs/internal/plans/

Criar ADR se houver nova decisão arquitetural relevante.

Atualizar:

* docs/internal/memory/CURRENT_STATE.md
* docs/internal/memory/HANDOFF.md
* docs/internal/memory/ARCHITECTURE.md
* docs/internal/memory/ROADMAP.md
* docs/internal/memory/DECISIONS.md
* docs/internal/memory/KNOWN_ISSUES.md

## Planejamento futuro

Registrar no roadmap uma proposta técnica para as Etapas 9 a 15, alinhada à evolução funcional de um ECAD semelhante ao EPLAN.

Não marcar essas etapas como aprovadas ou implementadas.

Não eliminar funcionalidades previamente aprovadas.

Preservar obrigatoriamente:

* Homologação geral após a Etapa 15.
* Melhoria visual futura da grade.
* Etapa 17 — CAD-CRAFT AI, plugin nativo com provedor exclusivo OpenAI e autenticação oficial.

---

# 26. CRITÉRIOS DE ACEITAÇÃO TÉCNICA

A etapa estará pronta quando:

* O usuário puder iniciar uma conexão a partir de uma porta válida.
* O editor apresentar pré-visualização do caminho.
* O desenho ortogonal funcionar.
* O destino puder ser selecionado.
* A conexão lógica for validada pelo Core.
* A representação gráfica possuir identidade estável.
* Junções e cruzamentos forem diferenciados.
* Conexões puderem ser selecionadas.
* Rotas puderem ser modificadas.
* Movimentação de símbolos preservar vínculos válidos.
* Undo/Redo funcionar.
* Propriedades refletirem o modelo elétrico.
* Snap permanecer independente do zoom.
* Os snapshots anteriores continuarem compatíveis.
* Todos os testes automatizados passarem.
* O build Release for aprovado.
* A revisão independente for concluída.
* A memória persistente for atualizada.

Funcionalidades não implementadas devem ser registradas como pendências, sem serem apresentadas como concluídas.

---

# 27. ENCERRAMENTO

Ao terminar:

1. Executar validações.
2. Realizar revisão independente.
3. Corrigir problemas encontrados.
4. Atualizar documentos.
5. Registrar decisões arquiteturais.
6. Apresentar relatório técnico.
7. Informar a quantidade total de testes.
8. Informar todos os arquivos criados e modificados.
9. Informar limitações restantes.
10. Encerrar sem commit e sem push.

**Não solicitar homologação manual.**

**Não iniciar a Etapa 9.**

**Não antecipar a Etapa 17.**

O usuário fará o commit e o push manualmente.

O objetivo é transformar o CAD-CRAFT em um editor de esquemas elétricos multifilares com conectividade real, preservando a arquitetura existente e evoluindo progressivamente em direção às funcionalidades de engenharia do EPLAN Electric P8.
