# CAD-CRAFT — ETAPA 10

## CORREÇÕES DE ROTEAMENTO, EDITOR DE BORNES, CABOS E CONDUTORES

**Projeto:** CAD-CRAFT
**Tecnologia:** Rust + egui/eframe
**Diretório:** `C:\Dev\cad-craft`
**Branch:** `main`

Implementar integralmente a Etapa 10.

Esta é uma nova conversa do Codex.

Recuperar o contexto do repositório e da memória persistente.

**Prioridade absoluta: corrigir os defeitos GUI-ROUTE-09A e GUI-ROUTE-09B antes de implementar as novas funcionalidades.**

## REGRAS PERMANENTES

* Seguir `AGENTS.md` e `docs/internal/memory/`.
* Não executar commit ou push.
* Não iniciar a Etapa 11.
* Não solicitar homologação manual.
* Preservar os 379 testes existentes.
* Utilizar TDD e regressões automatizadas.
* Executar revisão independente focada nos módulos críticos.
* Executar validação completa e build Release ao final.
* Atualizar a documentação e a memória persistente.
* Preservar o Core elétrico e a separação entre conexões lógicas e condutores físicos.
* Não antecipar o assistente de IA da Etapa 17.

A implementação deve ser completa, sem placeholders apresentados como funcionalidades concluídas.

---

# PARTE I — CORREÇÕES OBRIGATÓRIAS DA ETAPA 9

## 1. GUI-ROUTE-09A — PROLONGAMENTOS HORIZONTAIS

### Problema

O usuário confirmou que a correção da Etapa 9 resolveu o redimensionamento vertical, mas segmentos horizontais ainda apresentam prolongamentos residuais.

Comportamento:

1. Componente conectado por fio ortogonal.
2. Movimentação horizontal aumenta o comprimento da rota.
3. Movimento inverso deveria reduzir o comprimento.
4. A rota mantém segmentos excedentes ou pequenas pontas.
5. Movimentações sucessivas podem preservar essa geometria indesejada.

A conectividade elétrica permanece correta.

### Implementação

Investigar:

* `ConnectionRoute`.
* `RouteIntent`.
* `routing_base`.
* `points_with_endpoints`.
* `with_endpoints`.
* `repair_endpoints`.
* Normalização de segmentos ortogonais.
* Transformações das portas.
* Geometria derivada.
* Preservação de waypoints manuais.

Identificar a causa real no código.

Corrigir:

* Prolongamentos horizontais.
* Segmentos de comprimento zero.
* Dobras redundantes.
* Segmentos colineares desnecessários.
* Acúmulo de geometria derivada.

A normalização não poderá eliminar vértices manuais com significado geométrico ou alterar junções explícitas.

### Testes

Reproduzir:

* Movimento horizontal positivo e negativo.
* Afastamento e aproximação.
* Retorno à posição inicial.
* Movimentos sucessivos.
* Diferentes espaçamentos de snap.
* Símbolos rotacionados.
* Operações em grupo.
* Undo/Redo.
* Junções existentes.

A geometria deverá ser estável, determinística e reversível quando o estado geométrico retornar à configuração original.

---

## 2. GUI-ROUTE-09B — CONDUTOR ATRAVESSANDO O COMPONENTE

### Problema

Após determinadas rotações, o CAD-CRAFT mantém a conexão lógica correta, mas desenha segmentos atravessando o interior do símbolo.

Situações reproduzidas:

* Componente horizontal após rotação de 270°.
* Componente vertical após rotação de 180°.
* Movimentações posteriores dos componentes conectados.

O fio deve alcançar o terminal correto, respeitando o espaço ocupado pelo símbolo.

### Requisito principal

**A rota gráfica não deverá atravessar o interior da geometria do componente quando existir um percurso ortogonal válido até o terminal.**

### Implementação

Investigar a relação entre:

* Geometria vetorial do SymbolDefinition.
* Transformações do SymbolInstance.
* Posição da porta elétrica.
* Direção da porta.
* Bounding boxes e contornos transformados.
* Segmentos do fio.
* Algoritmo de reconstrução da rota.

Implementar um mecanismo determinístico de afastamento e contorno local dos símbolos.

Considerar:

1. Posição real do terminal.
2. Orientação de saída da porta.
3. Rotação.
4. Espelhamento.
5. Geometria do símbolo.
6. Espaçamento de segurança visual.
7. Rotas ortogonais existentes.
8. Waypoints manuais.
9. Junções.
10. Outros terminais do mesmo símbolo.

Não utilizar somente a AABB como representação definitiva da geometria quando isso produzir obstáculos falsos.

Utilizar geometria transformada ou uma região de exclusão coerente com o símbolo.

O afastamento deve ser definido em milímetros documentais, independentemente do zoom.

O fio deverá alcançar o ponto de conexão real sem deslocar artificialmente o endpoint.

### Requisitos de estabilidade

* Não criar loops geométricos.
* Não produzir segmentos residuais.
* Não alterar conectividade.
* Não criar novos ConnectionPoints.
* Não modificar redes.
* Não alterar condutores físicos.
* Não criar junções automáticas.
* Preservar o histórico transacional.
* Preservar a intenção de roteamento manual.

Quando uma rota válida não puder ser construída sem violar restrições, tratar a situação explicitamente.

Não alterar silenciosamente a conectividade para contornar um problema gráfico.

### Testes

Cobrir:

* Rotações 0°, 90°, 180° e 270°.
* Espelhamentos horizontal e vertical.
* Portas em todas as orientações.
* Movimentação horizontal.
* Movimentação vertical.
* Movimentação diagonal.
* Componentes próximos.
* Terminais próximos ao corpo do símbolo.
* Símbolos conectados em grupo.
* Junções.
* Undo/Redo.
* Snap ativo e desativado.
* Diferentes níveis de zoom.

Criar testes de interseção entre segmentos de rota e interiores dos símbolos.

Considerar exceções legítimas de contato no terminal e geometria de símbolos que contenha traços internos.

Realizar testes visuais automatizados usando o mecanismo existente de captura egui.

Não confundir captura gráfica com homologação manual.

---

## 3. CONDIÇÃO PARA AVANÇAR

Antes de iniciar a engenharia de bornes e cabos:

1. Reproduzir os dois defeitos em testes.
2. Identificar suas causas.
3. Corrigir as implementações.
4. Executar testes direcionados.
5. Confirmar preservação da conectividade.
6. Confirmar Undo/Redo.
7. Confirmar que não houve regressão do redimensionamento vertical.
8. Documentar os resultados.

Registrar a correção em `docs/internal/memory/KNOWN_ISSUES.md`.

Não marcar um defeito como resolvido sem evidência automatizada correspondente.

---

# PARTE II — ENGENHARIA DE BORNES, CABOS E CONDUTORES

## 4. OBJETIVO

Implementar ferramentas de engenharia para:

* Réguas de bornes.
* Bornes individuais.
* Pontes entre bornes.
* Cabos.
* Veias de cabos.
* Condutores individuais.
* Atribuição de conexões físicas.
* Identificação de fiação.
* Navegadores de bornes e cabos.
* Propriedades elétricas e documentais.
* Listas técnicas de fiação.

Utilizar o EPLAN Electric P8 como referência funcional.

Não desenvolver apenas representações visuais.

As estruturas devem possuir identidade e relações reais no Core.

---

## 5. AUDITORIA DO CORE

Antes de criar estruturas, investigar os modelos existentes.

Localizar:

* TerminalStrip.
* Terminal.
* Cable.
* CableCore.
* Conductor.
* PhysicalConnection.
* Connection.
* ConnectionPoint.
* Net.
* Device.
* Function.

Identificar quais operações já estão implementadas.

A Etapa 3 criou infraestrutura documental e física para terminais e cabos.

**Reutilizar essas estruturas.**

Não criar um segundo sistema de fiação física.

Estabelecer claramente quais dados representam:

1. Conectividade elétrica lógica.
2. Conexão física.
3. Condutor.
4. Veia de cabo.
5. Borne.
6. Régua de bornes.
7. Representação gráfica.

---

## 6. EDITOR DE RÉGUAS DE BORNES

Criar um editor integrado ao CAD-CRAFT.

Referência funcional:

**Terminal Strip Editor do EPLAN.**

### Funcionalidades

Permitir:

* Criar régua de bornes.
* Editar identificação.
* Adicionar bornes.
* Remover bornes.
* Reordenar bornes.
* Definir numeração.
* Identificar dispositivos associados.
* Consultar conexões.
* Visualizar lados de conexão.
* Identificar pontes.
* Consultar propriedades.
* Navegar para representações.

Exemplos:

`-X1`

Bornes:

`1`, `2`, `3`, `4`, `5`.

A régua deve possuir identidade persistente.

A numeração dos bornes não deverá ser utilizada como identificador interno.

### Tipos

Preparar suporte a:

* Borne de passagem.
* Borne PE.
* Borne multinível.
* Borne de distribuição.
* Borne seccionável.

Implementar apenas as propriedades e regras que possam ser validadas pelo Core nesta etapa.

Não declarar suporte completo a tipos especiais sem implementação funcional.

---

## 7. PONTES ENTRE BORNES

Implementar gerenciamento explícito de pontes.

Distinguir:

* Pontes fixas.
* Pontes inseridas.
* Ligações elétricas externas.
* Conexões físicas de fiação.

Uma ponte não poderá ser representada apenas por um desenho sem associação elétrica coerente.

### Requisitos

* Identidade estável.
* Bornes participantes.
* Validação de compatibilidade.
* Consulta das conexões.
* Atualização de redes quando semanticamente aplicável.
* Undo/Redo.
* Persistência.

Não criar conectividade a partir de proximidade gráfica.

---

## 8. EDITOR DE CABOS

Criar editor de cabos inspirado funcionalmente nas ferramentas de gerenciamento de cabos do EPLAN.

### Funcionalidades

Permitir:

* Criar cabo.
* Editar identificação.
* Definir descrição.
* Definir fabricante, quando informado.
* Definir código de produto, quando informado.
* Definir quantidade de veias.
* Identificar veias.
* Consultar origem e destino.
* Associar veias a conexões físicas.
* Identificar veias utilizadas e disponíveis.
* Detectar atribuições duplicadas.
* Consultar propriedades.
* Navegar para dispositivos relacionados.

Exemplo:

Cabo `-W1`.

Veias:

`1`, `2`, `3`, `4`, `PE`.

Esses nomes são exemplos.

Não presumir automaticamente a existência de uma veia PE em todo cabo.

---

## 9. VEIAS E CONDUTORES

Cada veia deverá possuir identificação persistente e relação com o cabo ao qual pertence.

Permitir propriedades como:

* Número ou designação.
* Cor.
* Seção nominal.
* Tipo.
* Descrição.
* Estado de utilização.
* Conexão física associada.

### Regras

* Uma veia não pode ser atribuída de forma incompatível a múltiplas conexões.
* Não criar condutores físicos a partir de linhas gráficas sem confirmação.
* Não confundir uma rede elétrica com um condutor.
* Não assumir que uma conexão lógica exige necessariamente um cabo multicondutor.
* Não inventar seção, cor ou capacidade elétrica.

A seção nominal deve utilizar unidade explícita e validação de valores.

Preservar a distinção entre dados fornecidos pelo usuário e valores efetivamente calculados.

---

## 10. ASSOCIAÇÃO ENTRE FIOS E CONDUTORES FÍSICOS

Implementar um mecanismo de atribuição de fiação.

Exemplo:

Uma conexão lógica entre `-K1:A1` e `-X1:1` poderá estar associada a um condutor físico identificado.

O condutor poderá pertencer a um cabo ou representar um fio individual, conforme o modelo existente.

### Requisitos

Permitir:

1. Selecionar conexão.
2. Consultar endpoints.
3. Criar ou selecionar condutor físico.
4. Associar condutor.
5. Consultar identificação.
6. Consultar cabo, quando houver.
7. Remover associação.
8. Desfazer/refazer.

Toda associação deve ser validada.

A relação entre conectividade lógica e fiação física precisa permanecer explícita.

Não modificar a topologia elétrica apenas para atribuir um condutor.

---

## 11. NAVEGADOR DE BORNES

Utilizar os navegadores existentes.

Organizar:

* Réguas.
* Bornes.
* Identificações.
* Pontos elétricos associados.
* Conexões.
* Pontes.
* Localizações documentais.

Permitir:

* Selecionar.
* Localizar ocorrência.
* Abrir página.
* Centralizar representação.
* Destacar objeto.
* Consultar propriedades.

Evitar consultas integrais desnecessárias a cada frame.

---

## 12. NAVEGADOR DE CABOS

Implementar navegação hierárquica.

Exemplo:

* Cabo W1.

  * Veia 1.
  * Veia 2.
  * Veia 3.
  * Veia 4.
  * Veia PE.

Mostrar:

* Identificação.
* Origem/destino quando conhecidos.
* Utilização.
* Conexões físicas.
* Propriedades relevantes.

Permitir navegação para os elementos associados.

Não criar um sistema paralelo de navegação.

Integrar às guias já existentes no painel esquerdo.

---

## 13. PROPRIEDADES E EDIÇÃO TABULAR

Implementar propriedades editáveis para:

* Réguas de bornes.
* Bornes.
* Pontes.
* Cabos.
* Veias.
* Condutores.
* Associações físicas.

Utilizar campos tipados e validação.

Priorizar edições tabulares para operações de engenharia com múltiplos registros.

Preservar:

* IDs.
* Integridade referencial.
* Regras de unicidade.
* Consistência de associações.
* Histórico transacional.

Não alterar estruturas internas diretamente pela interface.

---

## 14. RIBBON E INTERAÇÃO

Utilizar o layout existente.

### Guia Dispositivos

Comandos relacionados a:

* Bornes.
* Réguas.
* Propriedades.
* Localização.

### Guia Conexões

Comandos relacionados a:

* Condutores.
* Associação física.
* Navegação.
* Identificação de fiação.

### Guia Cabos

Disponibilizar:

* Criar cabo.
* Editar cabo.
* Gerenciar veias.
* Associar condutores.
* Consultar conexões.
* Navegar para componentes.

Não inserir comandos decorativos.

Não reconstruir a Ribbon nem o Backstage.

Utilizar o dispatcher central.

---

## 15. REPRESENTAÇÃO GRÁFICA

Integrar os dados físicos ao editor gráfico existente.

O usuário deverá conseguir consultar informações de fiação a partir de conexões desenhadas.

Quando houver associação física válida, exibir os dados correspondentes no painel de propriedades.

Não confundir:

* Rota gráfica.
* Conexão lógica.
* Condutor físico.
* Cabo.

Não implementar roteamento físico tridimensional ou dimensionamento automático sem especificação própria.

---

## 16. LISTAS TÉCNICAS

Preparar visualizações tabulares para:

* Lista de bornes.
* Lista de réguas.
* Lista de cabos.
* Lista de veias.
* Lista de condutores.
* Associações entre conexões e fiação física.

Os dados deverão ser derivados do Core.

Não manter cópias independentes que possam ficar desatualizadas.

Essas visualizações serão a base para os relatórios completos previstos para a Etapa 13.

Não antecipar a implementação integral do módulo de relatórios.

---

## 17. HISTÓRICO TRANSACIONAL

Integrar as operações ao histórico existente.

Cobrir:

* Inserção.
* Exclusão.
* Renumeração.
* Alteração de propriedades.
* Atribuição de veias.
* Associação de condutores.
* Criação e remoção de pontes.
* Edição de cabos.

Requisitos:

* Undo/Redo.
* IDs preservados.
* Integridade referencial.
* Operações atômicas.
* Rejeição de alterações inconsistentes.
* Nenhuma mutação parcial após erro.

Evitar snapshots integrais do projeto por frame.

---

## 18. PERSISTÊNCIA

Preservar compatibilidade com snapshots anteriores.

Validar:

* Réguas.
* Bornes.
* Cabos.
* Veias.
* Condutores.
* Pontes.
* Associações físicas.
* Referências documentais.

Não substituir um projeto válido por dados inválidos.

Não implementar o formato final `.cadcraft`, autosave ou backups avançados nesta etapa.

Preservar a arquitetura para essas funcionalidades futuras.

---

## 19. INTEGRIDADE ELÉTRICA

A implementação deverá preservar todas as relações existentes.

Não criar:

* Dispositivos fictícios.
* Pontos elétricos fictícios.
* Conexões implícitas.
* Redes baseadas apenas em nomes.
* Condutores físicos derivados automaticamente de traços gráficos.

Uma edição física não deverá alterar indevidamente o modelo lógico.

Uma edição gráfica não deverá alterar indevidamente o modelo físico.

As três camadas devem permanecer consistentes:

**Geometria → Conectividade lógica → Fiação física.**

---

## 20. TESTES AUTOMATIZADOS

Preservar os 379 testes existentes.

Adicionar testes relevantes para:

* Os dois defeitos de roteamento.
* Normalização geométrica.
* Contorno de símbolos.
* Identidade das portas.
* Réguas de bornes.
* Bornes individuais.
* Numeração.
* Pontes.
* Cabos.
* Veias.
* Condutores.
* Associações físicas.
* Integridade das redes.
* Conexões entre bornes.
* Exclusões com dependências.
* Edições inválidas.
* Undo/Redo.
* Persistência.
* Compatibilidade de snapshots.
* Navegadores.
* Propriedades.
* Comandos da Ribbon.
* Eventos reais egui.
* Cenários com milhares de registros.

Priorizar testes de comportamento e integridade.

Não exigir uma quantidade artificial de testes novos.

Executar testes direcionados durante o desenvolvimento.

Executar a suíte completa ao concluir.

---

## 21. EFICIÊNCIA DE EXECUÇÃO

Estamos otimizando o consumo do Codex sem reduzir qualidade.

Requisitos:

1. Consultar a memória persistente existente.
2. Evitar reler arquivos extensos sem necessidade.
3. Criar plano técnico objetivo.
4. Implementar uma unidade funcional por vez.
5. Executar testes direcionados durante o desenvolvimento.
6. Evitar builds completos repetidos sem alterações relevantes.
7. Utilizar subagentes somente quando agregarem valor.
8. Executar uma revisão independente focada.
9. Corrigir todos os defeitos encontrados.
10. Executar validação completa e build Release ao final.

Não substituir testes reais por afirmações de que o código parece correto.

Não reduzir verificações obrigatórias.

---

## 22. VALIDAÇÃO FINAL

Executar:

```powershell
pwsh -NoProfile -File scripts/validate.ps1

cargo build --release -p cad-gui --bin cad-gui --locked

git diff --check
git status --short
```

Todas as verificações devem ser aprovadas.

Realizar revisão independente sobre:

* Geometria dos fios.
* Contorno de símbolos.
* Integridade elétrica.
* Bornes.
* Cabos.
* Veias.
* Associações físicas.
* Histórico.
* Persistência.
* Performance.

Corrigir os achados antes de encerrar.

---

## 23. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_10_BORNES_CABOS_CONDUTORES.md`

Criar especificação e plano correspondentes.

Atualizar:

* CURRENT_STATE.md.
* HANDOFF.md.
* ROADMAP.md.
* ARCHITECTURE.md, quando necessário.
* DECISIONS.md, quando necessário.
* KNOWN_ISSUES.md.

Documentar separadamente:

### Correções gráficas

* Causa-raiz dos dois defeitos.
* Solução implementada.
* Regressões.
* Evidências visuais automatizadas.
* Preservação da conectividade.

### Engenharia física

* Arquitetura.
* Bornes.
* Réguas.
* Pontes.
* Cabos.
* Veias.
* Condutores.
* Associações físicas.
* Navegadores.
* Propriedades.
* Histórico.
* Persistência.

Preservar a melhoria futura da densidade visual da grade.

Preservar a Etapa 17 para o plugin CAD-CRAFT AI, exclusivamente com OpenAI.

---

## 24. CRITÉRIOS DE CONCLUSÃO

A Etapa 10 estará tecnicamente concluída quando:

* Os prolongamentos horizontais forem eliminados.
* As rotas respeitarem o corpo dos símbolos nas rotações suportadas.
* A conectividade elétrica permanecer correta.
* O redimensionamento vertical continuar funcionando.
* Undo/Redo preservar geometria e domínio.
* O editor de réguas estiver funcional.
* O gerenciamento de bornes estiver funcional.
* As pontes possuírem semântica elétrica válida.
* O editor de cabos estiver funcional.
* As veias puderem ser cadastradas e associadas.
* Os condutores físicos puderem ser vinculados a conexões válidas.
* Os navegadores estiverem integrados.
* As propriedades forem editáveis e validadas.
* Os snapshots anteriores permanecerem compatíveis.
* Todos os testes automatizados passarem.
* O build Release for aprovado.
* A revisão independente estiver concluída.
* A memória persistente estiver atualizada.

Não considerar implementada uma funcionalidade que exista apenas como estrutura de dados sem fluxo operacional correspondente.

---

## 25. ENCERRAMENTO

Ao concluir:

1. Apresentar relatório técnico objetivo.
2. Informar quais correções geométricas foram realizadas.
3. Informar os recursos físicos implementados.
4. Informar testes aprovados.
5. Informar limitações.
6. Informar arquivos criados e alterados.
7. Atualizar memória persistente.
8. Encerrar sem commit ou push.

Não iniciar a Etapa 11.

Não solicitar homologação manual.

O usuário realizará commit e push manualmente.

**Objetivo final:** avançar na construção de um ECAD industrial profissional, funcionalmente inspirado no EPLAN Electric P8, com engenharia real de bornes, cabos e condutores e representação gráfica consistente.
