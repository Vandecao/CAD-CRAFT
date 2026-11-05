# CAD-CRAFT — ETAPA 6

## EDITOR GRÁFICO INTERATIVO, SELEÇÃO, TRANSFORMAÇÕES E HISTÓRICO TRANSACIONAL

### Evolução do viewport CAD com Rust + egui/eframe

# 0. AUTORIZAÇÃO

Implementar integralmente a Etapa 6 do CAD-CRAFT.

**Diretório:**
`C:\Dev\cad-craft`

**Tecnologias:**

* Rust.
* egui/eframe.
* Windows 10 e Windows 11.

**Branch:** `main`.

**Commit-base:** identificar no Git antes de qualquer alteração.

**Linha de base conhecida:** 196 testes aprovados após as correções da Etapa 5.

A Etapa 5 foi homologada visualmente pelo usuário, registrada em Git e encaminhada para sincronização com o repositório remoto.

Esta é uma NOVA conversa do Codex.

Recuperar o contexto exclusivamente da memória persistente e do repositório.

Não depender de conversas anteriores.

## OBJETIVO PRINCIPAL

Transformar o viewport documental implementado na Etapa 5 em um editor CAD interativo capaz de selecionar, movimentar, transformar e editar representações gráficas existentes.

O comportamento deverá aproximar-se funcionalmente do EPLAN Electric P8.

Preservar integralmente a arquitetura elétrica construída nas etapas anteriores.

## DIRETRIZES

* Implementar autonomamente o escopo autorizado.
* Utilizar planejamento, TDD e revisão independente quando disponíveis.
* Seguir as instruções existentes em AGENTS.md.
* Respeitar o roadmap versionado do projeto.
* Não criar uma arquitetura paralela à existente.
* Não executar commit.
* Não executar push.
* Não iniciar a Etapa 7.
* Não modificar arquivos pessoais, backups ou artefatos não rastreados.

Se o roadmap existente definir uma divisão diferente entre as Etapas 6 e 7, verificar essa divergência e preservar a sequência arquitetural aprovada, documentando eventuais ajustes de escopo.

---

# 1. RECUPERAÇÃO OBRIGATÓRIA DA MEMÓRIA

Antes de modificar qualquer arquivo:

1. Ler `AGENTS.md`.
2. Ler `docs/internal/memory/HANDOFF.md`.
3. Ler `docs/internal/memory/CURRENT_STATE.md`.
4. Ler `docs/internal/memory/ARCHITECTURE.md`.
5. Ler `docs/internal/memory/ROADMAP.md`.
6. Ler `docs/internal/memory/KNOWN_ISSUES.md`.
7. Ler `docs/internal/memory/DECISIONS.md`.
8. Consultar todos os ADRs relevantes.
9. Consultar os relatórios das Etapas 4 e 5.
10. Consultar os contratos documentais da GUI.
11. Inspecionar o código real do Core e da interface.

Executar:

```powershell
git log -1 --oneline
git status --short
```

Verificar também:

* Estrutura dos crates.
* Dependências existentes.
* Modelo de representações.
* Transformações geométricas.
* Modelo de páginas.
* Estado transitório da GUI.
* Comandos disponíveis.
* Sistema de câmera.
* Sistema de grade.
* Implementação do viewport.
* APIs atuais de edição do Project.
* Serialização de snapshots.

Não presumir funcionalidades que ainda não existam.

O código-fonte e os testes prevalecem sobre documentação desatualizada.

---

# 2. PRINCÍPIOS ARQUITETURAIS OBRIGATÓRIOS

Preservar:

1. Core independente de egui/eframe.
2. IDs tipados.
3. Coordenadas documentais em milímetros.
4. Origem inferior esquerda.
5. X positivo para a direita.
6. Y positivo para cima.
7. Rotação positiva anti-horária.
8. Transformações determinísticas.
9. Integridade referencial.
10. Representações separadas de entidades elétricas.
11. Conectividade elétrica explícita.
12. Redes elétricas derivadas.
13. Condutores físicos separados de conexões lógicas.
14. Persistência compatível com snapshots anteriores.
15. Estado transitório da GUI separado do Project.

Não criar um segundo motor elétrico.

Não duplicar entidades elétricas para permitir movimentação gráfica.

Não utilizar coordenadas de tela como coordenadas persistentes.

Não utilizar nomes visíveis como substitutos de identificadores internos.

---

# 3. ARQUITETURA DO EDITOR INTERATIVO

Separar claramente:

* Estado da ferramenta ativa.
* Seleção atual.
* Hover.
* Operação em andamento.
* Pré-visualização de movimentação.
* Câmera.
* Grade.
* Snap.
* Transformações.
* Comandos.
* Histórico.
* Modelo documental persistente.

Criar uma organização modular compatível com o projeto existente.

Responsabilidades sugeridas:

* InteractionController.
* SelectionState.
* HitTest.
* DragSession.
* TransformCommand.
* EditTransaction.
* History.

Os nomes são sugestões, não exigências.

Reutilizar estruturas existentes sempre que possível.

A GUI deverá comandar alterações por APIs validadas do Core.

---

# 4. SELEÇÃO SIMPLES

## 4.1. Comportamento

Ao clicar com o botão esquerdo sobre uma representação:

* Identificar o objeto.
* Selecioná-lo.
* Destacá-lo visualmente.
* Atualizar o painel de propriedades.
* Preservar a página ativa.

O clique simples não deverá movimentar o objeto.

## 4.2. Clique no vazio

Ao clicar em uma região sem objetos:

* Limpar a seleção gráfica atual.
* Não modificar o Project.
* Não modificar o histórico.
* Não alterar o zoom.

## 4.3. Identidade

A seleção deverá utilizar IDs persistentes de representação.

Não utilizar índices temporários de arrays como identidade.

## 4.4. Seleção e documento

Selecionar um objeto não deverá alterar sua página, dispositivo ou conectividade.

---

# 5. HIT-TESTING

Implementar identificação precisa dos objetos sob o cursor.

## 5.1. Requisitos

Considerar:

* Posição.
* Dimensões.
* Rotação.
* Espelhamento.
* Geometria.
* Transformação da câmera.
* Zoom.
* Tolerância em pixels.
* Ordem visual.

## 5.2. Tolerância

A seleção deverá utilizar tolerância visual apropriada.

A tolerância não poderá variar incorretamente com o zoom.

Preferir tolerâncias expressas em pontos lógicos de tela e convertidas corretamente para coordenadas documentais.

## 5.3. Elementos sobrepostos

Quando houver objetos sobrepostos:

* Definir prioridade determinística.
* Considerar ordem visual.
* Permitir selecionar o objeto superior.
* Preparar mecanismo futuro de alternância entre objetos sobrepostos.

Não selecionar aleatoriamente.

## 5.4. Transformações

O hit-testing deverá funcionar corretamente para representações:

* Rotacionadas.
* Espelhadas.
* Deslocadas.
* Próximas das bordas da página.
* Parcialmente fora da folha.

## 5.5. Culling

Consultar preferencialmente os objetos relevantes à região visível.

Evitar varreduras geométricas excessivas quando o projeto crescer.

---

# 6. DESTAQUE VISUAL DA SELEÇÃO

Reproduzir a experiência visual de um editor CAD profissional.

## 6.1. Objeto selecionado

Utilizar:

* Contorno azul discreto.
* Alças de manipulação quando apropriado.
* Destaque consistente.
* Identificação visual clara.

Não ocultar a geometria original.

## 6.2. Hover

Ao posicionar o cursor sobre um objeto:

* Aplicar destaque temporário.
* Não modificar a seleção.
* Não modificar o Project.
* Não gerar Undo/Redo.

## 6.3. Alças

Preparar alças para:

* Movimentação.
* Rotação.
* Ajustes geométricos futuros.

Somente habilitar as operações realmente implementadas.

Não apresentar alças funcionais fictícias.

## 6.4. Desenho

O destaque deverá respeitar zoom e pan.

As alças devem manter tamanho visual adequado em pixels, sem crescer excessivamente junto com o documento.

---

# 7. SELEÇÃO MÚLTIPLA

## 7.1. Ctrl + clique

Implementar seleção aditiva.

Com Ctrl pressionado:

* Adicionar objeto.
* Remover objeto já selecionado.
* Preservar os demais.

## 7.2. Seleção retangular

Permitir arrastar uma área de seleção sobre o viewport.

Comportamento:

* Iniciar no espaço vazio.
* Mostrar retângulo translúcido.
* Atualizar visualmente durante o arraste.
* Finalizar ao soltar o mouse.
* Selecionar os objetos conforme a regra definida.

## 7.3. Direção

Preparar comportamento semelhante a CADs profissionais:

Da esquerda para a direita:

Selecionar objetos inteiramente contidos no retângulo.

Da direita para a esquerda:

Selecionar objetos que cruzem ou estejam contidos no retângulo.

Diferenciar visualmente os dois modos, quando apropriado.

## 7.4. Cancelamento

Esc deverá cancelar uma seleção retangular em andamento.

Cancelar não poderá alterar o histórico.

## 7.5. Estado

A seleção múltipla deverá ser armazenada como conjunto determinístico de IDs.

Evitar duplicidades.

---

# 8. MOVIMENTAÇÃO DE OBJETOS

## 8.1. Objetivo

Permitir movimentar representações existentes com o mouse.

O movimento deverá modificar somente sua transformação documental, preservando a entidade elétrica associada.

## 8.2. Interação

Fluxo:

1. Selecionar objeto.
2. Iniciar arraste.
3. Mostrar pré-visualização.
4. Atualizar posição durante o movimento.
5. Aplicar snap quando habilitado.
6. Soltar o mouse para confirmar.
7. Registrar uma transação no histórico.

## 8.3. Pré-visualização

Durante o arraste:

* Mostrar o objeto na posição proposta.
* Preservar visualmente a posição original quando útil.
* Não gravar continuamente o Project.
* Não criar uma transação por frame.

O estado transitório pertence à GUI.

## 8.4. Confirmação

Ao soltar o mouse:

* Validar coordenadas.
* Aplicar uma operação persistente.
* Atualizar o documento.
* Atualizar propriedades.
* Atualizar seleção.
* Registrar Undo/Redo.

## 8.5. Cancelamento

Esc durante o arraste:

* Cancela.
* Restaura a posição original.
* Não modifica histórico.
* Não deixa estados transitórios pendentes.

## 8.6. Movimento nulo

Arrastar e soltar no mesmo ponto não deverá criar uma entrada de histórico desnecessária.

---

# 9. MOVIMENTAÇÃO MÚLTIPLA

Permitir movimentar vários objetos selecionados simultaneamente.

Requisitos:

* Preservar posições relativas.
* Aplicar deslocamento comum.
* Preservar IDs.
* Preservar associações.
* Preservar rotações individuais.
* Não deslocar objetos não selecionados.

A movimentação do conjunto deverá produzir uma única transação Undo/Redo.

O snap deverá ser aplicado a uma referência ou ponto-base definido, preservando o deslocamento relativo dos demais objetos.

Não arredondar independentemente cada objeto de maneira que deforme o conjunto.

---

# 10. SNAP À GRADE

## 10.1. Regra principal

O snap é calculado em coordenadas reais do documento.

Exemplo:

Grade configurada para 4 mm.

O ajuste continua sendo de 4 mm em qualquer zoom.

A distância visual entre os pontos poderá mudar, mas a precisão geométrica não.

## 10.2. Independência

Separar:

* Grid lógico.
* Grid renderizado.
* Snap.

A visibilidade da grade não controla o snap.

## 10.3. Cálculo

Tratar corretamente:

* Coordenadas positivas.
* Coordenadas negativas.
* Origem.
* Zoom.
* Pan.
* DPI.
* Valores não inteiros.
* Diferentes espaçamentos.

## 10.4. Durante movimentação

O usuário deverá conseguir:

* Movimentar com snap ativo.
* Desabilitar snap pela barra inferior.
* Movimentar sem snap.
* Reativar snap.

Não alterar a resolução do documento ao desabilitar o snap.

## 10.5. Melhoria futura já registrada

Existe uma melhoria planejada sobre suavização da densidade visual da grade durante zoom out.

Preservar essa pendência no roadmap.

Não alterar o espaçamento lógico para reduzir a quantidade de pontos.

Não implementar mudanças adicionais nessa renderização sem necessidade comprovada para esta etapa.

---

# 11. TRANSFORMAÇÕES GEOMÉTRICAS

Implementar operações básicas sobre representações existentes.

## 11.1. Rotação

Permitir:

* Rotação de 90°.
* Rotação de 180°.
* Rotação de 270°.
* Edição de ângulo quando suportada pelo Core.

Preservar convenção positiva anti-horária.

Definir explicitamente o ponto de rotação.

## 11.2. Espelhamento

Permitir:

* Espelhar horizontalmente.
* Espelhar verticalmente.

Preservar propriedades documentais.

## 11.3. Movimentação numérica

Permitir editar X e Y diretamente no painel de propriedades ou em diálogo apropriado.

Utilizar milímetros.

## 11.4. Validação

Rejeitar:

* NaN.
* Infinito.
* Coordenadas inválidas.
* Transformações inconsistentes.

## 11.5. Histórico

Cada operação confirmada deverá ser reversível.

---

# 12. CURSORES CONTEXTUAIS

Implementar comportamento consistente.

## 12.1. Área vazia

Cursor padrão do editor.

## 12.2. Objeto selecionável

Cursor de seleção apropriado.

Não utilizar permanentemente o cursor de cruz sobre todos os objetos.

## 12.3. Movimentação

Cursor de movimento.

## 12.4. Pan

Botão central:

* Mão aberta antes de arrastar.
* Mão fechada durante o arraste.

## 12.5. Alças

Utilizar cursor adequado à operação.

## 12.6. Referências

Links de navegação deverão utilizar cursor apropriado, quando já disponíveis.

## 12.7. Divisores

Preservar cursores de redimensionamento dos painéis.

---

# 13. PAN E SELEÇÃO — EVITAR CONFLITOS

Esta seção é crítica.

A Etapa 5 implementou corretamente o viewport independente da folha.

A Etapa 6 NÃO poderá comprometer esse comportamento.

Definir prioridades:

* Botão esquerdo: seleção/manipulação.
* Botão central: pan.
* Roda: zoom.
* Ctrl+clique: múltipla seleção.
* Arraste sobre espaço vazio: seleção retangular.
* Esc: cancelar operação ativa.

Não permitir que o botão esquerdo movimente a câmera quando a intenção for selecionar um objeto.

Não permitir que o botão central modifique a posição documental de uma representação.

Não permitir que a roda do mouse altere inadvertidamente uma propriedade elétrica.

Preservar zoom ancorado no cursor.

---

# 14. HISTÓRICO TRANSACIONAL — UNDO/REDO

## 14.1. Objetivo

Implementar infraestrutura de histórico compatível com as próximas etapas do CAD-CRAFT.

Inspecionar primeiro as APIs reais do Core.

Se já existir mecanismo de edição transacional, reutilizá-lo.

Caso não exista, projetar uma solução proporcional ao escopo atual.

## 14.2. Operações

Cobrir inicialmente:

* Movimentação.
* Movimentação múltipla.
* Rotação.
* Espelhamento.
* Alterações numéricas de transformação.

## 14.3. Transações

Uma operação confirmada deverá produzir uma única entrada no histórico.

Não criar centenas de entradas durante arrastes.

## 14.4. Identidades

Undo/Redo deverá preservar:

* IDs.
* Associações.
* Entidades elétricas.
* Integridade documental.

## 14.5. Redo

Após Undo:

* Redo restaura a operação.

Após nova operação divergente:

* Invalidar adequadamente a pilha de Redo.

## 14.6. Limites

Criar política de retenção apropriada para evitar crescimento ilimitado da memória.

## 14.7. Estado da câmera

Zoom, pan e seleção não devem entrar no histórico de edição do projeto.

## 14.8. Atalhos

Implementar:

* Ctrl+Z — Desfazer.
* Ctrl+Y — Refazer.

Preservar comportamento dos campos textuais.

---

# 15. MENUS CONTEXTUAIS

Implementar menu contextual sobre representações.

## 15.1. Objeto

Opções compatíveis com o Core:

* Selecionar.
* Propriedades.
* Mover.
* Rotacionar.
* Espelhar.
* Localizar referência, quando disponível.

## 15.2. Área vazia

Opções apropriadas:

* Enquadrar página.
* Enquadrar conteúdo.
* Mostrar grade.
* Configurar snap.
* Propriedades da página.

## 15.3. Habilitação

Não criar comandos fictícios.

Comandos futuros deverão permanecer desabilitados ou ausentes.

## 15.4. Seleção

Clique direito sobre objeto não selecionado deverá estabelecer comportamento previsível e documentado.

O menu contextual não poderá modificar a conectividade elétrica inadvertidamente.

---

# 16. PAINEL DE PROPRIEDADES

Integrar totalmente com a seleção gráfica.

## 16.1. Seleção simples

Exibir:

* Identificação.
* ID da representação.
* Página.
* Entidade associada.
* X.
* Y.
* Rotação.
* Espelhamento.
* Visibilidade, quando suportada.
* Outras propriedades existentes.

## 16.2. Edição

Permitir modificar propriedades editáveis por operações validadas.

Não alterar diretamente os dados internos do Project sem validação.

## 16.3. Seleção múltipla

Mostrar:

* Quantidade de objetos.
* Propriedades comuns.
* Operações aplicáveis ao conjunto.

Não apresentar valores arbitrários quando os objetos possuem propriedades diferentes.

## 16.4. Atualização

Ao movimentar um objeto:

* Atualizar X/Y.
* Atualizar a visualização.
* Preservar seleção.

Ao executar Undo:

* Atualizar novamente as propriedades.

---

# 17. RIBBON

Integrar as operações às guias existentes.

Priorizar:

## Editar

* Selecionar.
* Mover.
* Rotacionar.
* Espelhar.
* Desfazer.
* Refazer.

## Página Inicial

* Propriedades.
* Comandos de edição disponíveis.

## Visualização

* Enquadrar página.
* Enquadrar conteúdo.
* Mostrar grade.
* Snap.

Preservar o registro central de comandos.

Os botões devem chamar o mesmo dispatcher utilizado por:

* Atalhos.
* Menus contextuais.
* Painel de propriedades.

Não criar implementações separadas da mesma operação.

## Requisito de estabilidade

A Ribbon não poderá voltar a apresentar desaparecimento intermitente dos grupos.

Preservar a correção da Etapa 5.

---

# 18. NAVEGADOR E REFERÊNCIAS

Integrar seleção gráfica e navegação documental.

Requisitos:

* Selecionar página no navegador.
* Abrir documento.
* Selecionar representação no viewport.
* Exibir propriedades correspondentes.
* Navegar para representação associada, quando suportado.
* Preservar IDs.

Não alterar a árvore documental ao selecionar graficamente um objeto.

Não criar referência elétrica por proximidade visual.

---

# 19. INTEGRIDADE DO CORE

Toda operação deverá preservar:

* ProjectId.
* PageId.
* RepresentationId.
* IDs das entidades associadas.
* Associações elétricas existentes.
* Integridade referencial.

## Regra fundamental

**Movimentar uma representação gráfica não significa movimentar fisicamente um dispositivo industrial nem modificar sua conectividade lógica.**

Não criar nem excluir:

* Connections.
* Nets.
* Conductors.
* Terminais.
* Dispositivos.

como efeito colateral de uma operação puramente gráfica.

---

# 20. PERSISTÊNCIA E SNAPSHOTS

Preservar compatibilidade com o mecanismo documental existente.

## Requisitos

1. Alterações gráficas devem existir no modelo documental.
2. Transformações devem ser serializáveis.
3. Snapshots antigos devem continuar abrindo.
4. IDs devem permanecer estáveis.
5. Serialização deve continuar válida.
6. Dados elétricos não devem ser modificados inadvertidamente.

Não criar nesta etapa um formato `.cadcraft` definitivo se ele ainda não estiver previsto no roadmap atual.

Não confundir preferências da GUI com dados documentais.

## Histórico

O histórico de edição poderá permanecer em memória de sessão.

Não serializar Undo/Redo sem necessidade arquitetural.

Documentar esse comportamento.

---

# 21. DESEMPENHO

A interface deverá continuar responsiva.

Evitar:

* Clonagem integral do projeto por frame.
* Recalcular toda a hierarquia a cada movimento do mouse.
* Persistir estado durante pré-visualização.
* Criar transações por frame.
* Hit-testing desnecessário de objetos invisíveis.
* Alocação excessiva durante drag.
* Renderização de objetos fora do viewport.

Criar testes ou medições proporcionais ao escopo.

Não introduzir otimizações prematuras que dificultem a manutenção.

---

# 22. TESTES AUTOMATIZADOS

Preservar os 196 testes anteriores.

Adicionar regressões para:

1. Seleção simples.
2. Clique no vazio.
3. Ctrl+clique.
4. Seleção retangular.
5. Seleção por interseção.
6. Identidade por ID.
7. Hit-testing.
8. Rotação.
9. Espelhamento.
10. Zoom e hit-testing.
11. Pan e hit-testing.
12. Tolerância em pixels.
13. Hover.
14. Movimentação.
15. Pré-visualização.
16. Cancelamento.
17. Movimento nulo.
18. Movimentação múltipla.
19. Preservação de posições relativas.
20. Snap.
21. Grade invisível com snap ativo.
22. Coordenadas negativas.
23. Edição numérica.
24. Undo.
25. Redo.
26. Histórico transacional.
27. Preservação de IDs.
28. Preservação da conectividade.
29. Serialização.
30. Compatibilidade retroativa.
31. Menus contextuais.
32. Comandos da Ribbon.
33. Integração com propriedades.
34. Troca de páginas.
35. Estados transitórios.
36. Cancelamento por Esc.
37. Independência da câmera.
38. Integridade referencial.

Os testes deverão verificar comportamento, não apenas existência de estruturas.

---

# 23. TESTES VISUAIS

Utilizar os mecanismos de QA existentes.

O Computer Use oficial foi identificado no ambiente, mas apresentou timeouts ao capturar a janela do CAD-CRAFT durante a Etapa 5.

Portanto:

* Não depender exclusivamente do Computer Use.
* Utilizar capturas internas do egui quando disponíveis.
* Verificar imagens reais produzidas pelo renderizador.
* Não tratar uma captura preta como evidência de interface vazia.
* Não declarar inspeção visual concluída sem imagens válidas.

A confirmação final da interação real continuará sendo realizada pelo usuário no Windows.

Não transformar a investigação do Computer Use no objetivo principal desta etapa.

---

# 24. ROTEIRO DE HOMOLOGAÇÃO MANUAL

Preparar um roteiro reproduzível com os seguintes testes.

## TESTE A — Seleção simples

Abrir demonstração.

Selecionar uma representação.

Confirmar destaque visual.

Confirmar painel de propriedades.

## TESTE B — Seleção múltipla

Selecionar duas ou mais representações com Ctrl.

Verificar destaque.

Remover uma representação da seleção.

## TESTE C — Seleção retangular

Arrastar um retângulo.

Testar as duas direções.

Confirmar objetos selecionados.

## TESTE D — Movimentação

Selecionar uma representação.

Arrastar para outra posição.

Conferir X/Y.

## TESTE E — Snap

Configurar grade para 4 mm.

Movimentar representação.

Confirmar ajuste a múltiplos de 4 mm.

Desligar a visualização da grade.

Repetir com snap ativo.

## TESTE F — Transformações

Rotacionar.

Espelhar.

Verificar geometria e propriedades.

## TESTE G — Undo/Redo

Movimentar.

Desfazer.

Refazer.

Repetir com múltiplos objetos.

## TESTE H — Cancelamento

Iniciar arraste.

Pressionar Esc.

Confirmar restauração da posição original.

## TESTE I — Navegação

Executar zoom e pan durante a sessão.

Confirmar que a seleção continua coerente.

## TESTE J — Menus contextuais

Clicar com botão direito sobre objeto.

Verificar ações.

Clicar sobre área vazia.

Verificar ações diferentes.

## TESTE K — Persistência

Exportar um snapshot técnico pelo mecanismo existente.

Reabrir.

Confirmar transformações e IDs.

## TESTE L — Regressão visual

Verificar:

* Grade de pontos.
* Ribbon.
* Backstage.
* Barra de título escura.
* Zoom.
* Pan.
* Layout.
* Painéis redimensionáveis.

---

# 25. VALIDAÇÃO FINAL

Executar o script oficial:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

Executar também:

```powershell
cargo build --release -p cad-gui --bin cad-gui --locked

git diff --check
git status --short
```

Corrigir todas as falhas.

Não remover testes.

Não desabilitar verificações.

Não alterar código elétrico fora do escopo.

Executar revisão independente quando disponível.

A revisão deverá focar em:

* Integridade do Core.
* Transformações.
* Estados transitórios.
* Undo/Redo.
* Hit-testing.
* Interações entre ferramentas.
* Compatibilidade.
* Desempenho.
* Regressões visuais.

---

# 26. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_06_EDITOR_GRAFICO_INTERATIVO.md`

Documentar:

1. Resumo executivo.
2. Arquitetura da interação.
3. Estado da seleção.
4. Hit-testing.
5. Destaques visuais.
6. Movimentação.
7. Transformações.
8. Snap.
9. Histórico transacional.
10. Integração com Core.
11. Menus contextuais.
12. Atalhos.
13. Ribbon.
14. Inspetor de propriedades.
15. Navegação.
16. Persistência.
17. Desempenho.
18. Testes.
19. Revisão independente.
20. Limitações.
21. Roteiro de homologação.
22. Arquivos criados e alterados.

Atualizar:

* `docs/internal/memory/CURRENT_STATE.md`.
* `docs/internal/memory/HANDOFF.md`.
* `docs/internal/memory/ROADMAP.md`.
* `docs/internal/memory/KNOWN_ISSUES.md`, quando necessário.
* `docs/internal/memory/ARCHITECTURE.md`, quando necessário.
* `docs/internal/memory/DECISIONS.md`, quando necessário.

Criar ADRs somente quando houver decisões arquiteturais relevantes.

Não registrar homologação manual antes da confirmação do usuário.

---

# 27. CRITÉRIOS DE ENTREGA

A Etapa 6 estará pronta para homologação quando:

* [ ] A seleção simples funcionar.
* [ ] A seleção múltipla funcionar.
* [ ] A seleção retangular funcionar.
* [ ] O hit-testing for consistente.
* [ ] O hover funcionar.
* [ ] As representações puderem ser movimentadas.
* [ ] O movimento múltiplo preservar posições relativas.
* [ ] O snap funcionar em coordenadas documentais.
* [ ] A rotação funcionar.
* [ ] O espelhamento funcionar.
* [ ] A edição numérica funcionar.
* [ ] Undo/Redo funcionar.
* [ ] Esc cancelar operações transitórias.
* [ ] Os cursores forem contextuais.
* [ ] Os menus contextuais funcionarem.
* [ ] As propriedades refletirem a seleção.
* [ ] A Ribbon utilizar comandos reais.
* [ ] O pan continuar funcionando.
* [ ] O zoom continuar funcionando.
* [ ] O viewport permanecer independente da folha.
* [ ] A grade lógica permanecer estável.
* [ ] A conectividade elétrica permanecer intacta.
* [ ] Os snapshots anteriores continuarem compatíveis.
* [ ] Todos os testes automatizados passarem.
* [ ] O build Release for gerado.
* [ ] A documentação estiver atualizada.
* [ ] Nenhum commit ou push tiver sido realizado.

---

# 28. RESTRIÇÕES FINAIS

NÃO executar git commit.

NÃO executar git push.

NÃO iniciar a Etapa 7.

NÃO reconstruir a interface da Etapa 5.

NÃO alterar o Backstage desnecessariamente.

NÃO modificar a conectividade elétrica.

NÃO desenvolver um segundo modelo de domínio.

NÃO implementar prematuramente o editor completo de símbolos IEC.

NÃO implementar roteamento elétrico nesta etapa.

NÃO transformar proximidade gráfica em conexão lógica.

NÃO alterar a resolução real da grade para melhorar sua aparência.

NÃO modificar projetos pessoais ou backups.

NÃO eliminar testes existentes.

NÃO declarar homologação manual sem confirmação do usuário.

---

# 29. RESULTADO ESPERADO

Ao concluir a Etapa 6, o CAD-CRAFT deverá deixar de ser apenas um visualizador documental com navegação.

O usuário deverá conseguir:

* Selecionar objetos.
* Identificar visualmente o objeto ativo.
* Selecionar vários objetos.
* Movimentar representações.
* Utilizar snap.
* Rotacionar e espelhar representações.
* Editar coordenadas.
* Desfazer e refazer alterações.
* Utilizar menus contextuais.
* Manipular objetos sem comprometer o modelo elétrico.
* Preservar zoom e pan durante as operações.
* Salvar e recuperar as transformações pelo mecanismo documental existente.

A interface deverá permanecer visualmente consistente com o CAD-CRAFT desenvolvido na Etapa 5.

**Princípio fundamental: o usuário manipula representações gráficas, enquanto o Core continua responsável pela identidade e integridade das entidades elétricas.**

Concluir a implementação, executar os testes, atualizar a memória e apresentar o relatório técnico.

Aguardar homologação manual antes de qualquer commit ou push.
