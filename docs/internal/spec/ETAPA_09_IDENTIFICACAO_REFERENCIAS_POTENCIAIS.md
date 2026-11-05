# CAD-CRAFT — ETAPA 9

## CORREÇÃO DO REDIMENSIONAMENTO DOS FIOS, IDENTIFICAÇÃO DE DISPOSITIVOS, REFERÊNCIAS CRUZADAS E POTENCIAIS

**Projeto:** CAD-CRAFT
**Linguagem:** Rust
**Interface:** egui/eframe
**Sistema:** Windows Desktop
**Diretório:** `C:\Dev\cad-craft`
**Branch:** `main`

Esta é uma nova conversa do Codex.

Implementar integralmente a Etapa 9, começando obrigatoriamente pela correção do problema geométrico descrito abaixo.

## 0. PRIORIDADE ABSOLUTA — CORRIGIR O REDIMENSIONAMENTO DOS FIOS

### 0.1. Problema identificado pelo usuário

Durante a homologação excepcional da Etapa 8, o usuário identificou um defeito na atualização geométrica das conexões elétricas.

**Comportamento observado:**

1. Um componente está conectado a outro por uma rota elétrica.
2. O usuário seleciona um componente e o movimenta horizontalmente.
3. Ao afastar o componente, a conexão acompanha o movimento corretamente.
4. Ao aproximá-lo novamente, a conexão não recupera adequadamente sua geometria anterior.
5. Segmentos e prolongamentos permanecem na rota, formando sobras visuais.
6. O mesmo acontece com movimentos verticais.
7. Ao rotacionar o componente em 90° e movimentá-lo verticalmente, a rota aumenta corretamente, mas deixa prolongamentos quando o componente retorna.

**Informação fundamental:**

A conectividade elétrica permanece correta.

O defeito observado está na geometria das rotas, não na conectividade lógica.

Não modificar o modelo elétrico para tentar corrigir um problema exclusivamente gráfico.

### 0.2. Comportamento esperado

Ao movimentar um componente conectado:

* A rota deverá acompanhar o componente.
* O trecho necessário deverá aumentar quando o componente se afastar.
* O trecho deverá diminuir quando o componente se aproximar.
* O caminho não poderá acumular prolongamentos desnecessários.
* A geometria deverá retornar à configuração anterior quando os endpoints retornarem às posições anteriores, desde que o contexto geométrico seja equivalente.
* A movimentação horizontal e vertical deverá apresentar comportamento consistente.
* A rotação do componente não poderá introduzir segmentos residuais.
* A conectividade elétrica deverá permanecer intacta.

### 0.3. Investigação obrigatória

Inspecionar especificamente:

* `connection_geometry.rs`
* `connection_editor.rs`
* `connection_view.rs`
* `connection_drawing.rs`
* `project/connection_drawing.rs`
* `interaction.rs`
* `history.rs`

Verificar também o método `points_with_endpoints` e os mecanismos existentes de atualização de rotas durante movimentações.

Investigar se a geometria derivada está sendo reutilizada incorretamente como entrada das próximas transformações.

A hipótese é que segmentos gerados automaticamente estejam sendo preservados mesmo depois de deixarem de ser necessários.

**Não assumir essa hipótese como causa confirmada sem examinar o código.**

### 0.4. Requisito arquitetural

Distinguir claramente:

* Endpoints elétricos.
* Posições geométricas dos endpoints.
* Vértices definidos intencionalmente pelo usuário.
* Dobras calculadas automaticamente pelo editor.
* Geometria derivada para apresentação.

Uma dobra introduzida automaticamente não deverá se tornar permanente apenas porque o componente foi movimentado.

Por outro lado, vértices inseridos deliberadamente pelo usuário não deverão ser apagados indiscriminadamente.

A solução deve evitar que a rota dependa de um histórico acumulativo de redimensionamentos.

Quando apropriado, recalcular a geometria a partir dos endpoints atuais e das restrições geométricas persistentes.

### 0.5. Critérios da correção

A solução deverá garantir:

1. Crescimento e redução corretos dos segmentos.
2. Movimento horizontal.
3. Movimento vertical.
4. Movimento combinado.
5. Rotação em 90°, 180° e 270°.
6. Snap ativo e desativado.
7. Grade oculta.
8. Preservação de vértices manuais.
9. Ausência de segmentos nulos ou residuais.
10. Preservação da ortogonalidade.
11. Preservação dos endpoints.
12. Preservação das junções explícitas.
13. Preservação das redes elétricas.
14. Undo/Redo correto.
15. Operações transacionais.
16. Repetibilidade determinística.

### 0.6. Testes de regressão obrigatórios

Criar testes RED/GREEN que reproduzam o defeito.

Executar sequências como:

**Teste A:**

Posição inicial → +40 mm → posição inicial.

A geometria final deverá ser equivalente à inicial, considerando apenas diferenças de representação canônica legitimamente necessárias.

**Teste B:**

Posição inicial → +40 mm → +20 mm → posição inicial.

Não poderão permanecer prolongamentos.

**Teste C:**

Repetir com movimentos verticais.

**Teste D:**

Rotacionar o componente em 90°, movimentar verticalmente e retornar.

**Teste E:**

Repetir com movimentação de dois componentes conectados.

**Teste F:**

Repetir com junções explícitas e vértices manuais.

**Teste G:**

Executar Undo/Redo depois de cada transformação.

**Teste H:**

Comparar a conectividade elétrica e as redes derivadas antes e depois.

Utilizar testes de integração e, quando aplicável, eventos reais do egui.

A correção deverá passar pela validação automatizada antes de iniciar a implementação funcional da Etapa 9.

### 0.7. Restrições

Não reconstruir o editor de conexões.

Não criar outro motor de roteamento.

Não alterar a semântica elétrica das conexões.

Não eliminar vértices manuais legítimos.

Não introduzir simplificações que provoquem alterações inesperadas em junções.

Se a geometria não puder ser preservada validamente, a operação deverá ser rejeitada de maneira transacional.

**Concluir, testar e documentar essa correção antes de avançar para o restante da etapa.**

---

# 1. RECUPERAÇÃO DA MEMÓRIA DO PROJETO

Após estabelecer o diagnóstico inicial da correção, recuperar o contexto completo utilizando os documentos existentes.

Ler obrigatoriamente:

* `AGENTS.md`
* `docs/internal/memory/HANDOFF.md`
* `docs/internal/memory/CURRENT_STATE.md`
* `docs/internal/memory/ROADMAP.md`
* `docs/internal/memory/ARCHITECTURE.md`
* `docs/internal/memory/DECISIONS.md`
* `docs/internal/memory/KNOWN_ISSUES.md`

Consultar ADRs e contratos relevantes.

Ler os relatórios das Etapas 7 e 8.

Inspecionar os componentes existentes de:

* Dispositivos.
* Funções elétricas.
* Representações.
* Conectividade.
* Páginas.
* Símbolos.
* Histórico transacional.
* Persistência.
* Navegadores.

Não recriar estruturas que já existam no Core.

## Estado esperado do projeto

Etapas 1 a 8 concluídas.

**Último commit conhecido:**

`5377559`

Mensagem:

`feat: implement graphical electrical connections and junctions`

O usuário confirmou o push da Etapa 8.

**Baseline automatizado: 339 testes aprovados.**

Verificar o estado real do Git antes de alterar arquivos.

Não executar commit, push, pull ou alterações de branch.

---

# 2. OBJETIVO FUNCIONAL DA ETAPA 9

Implementar as funcionalidades fundamentais de identificação e rastreabilidade elétrica utilizadas em sistemas ECAD profissionais.

O CAD-CRAFT deverá reconhecer e apresentar:

1. Identificação de dispositivos.
2. Identificadores estruturais.
3. Funções principais e auxiliares.
4. Ocorrências de um dispositivo em diferentes páginas.
5. Referências cruzadas.
6. Referências entre bobinas e contatos.
7. Pontos de interrupção.
8. Potenciais elétricos.
9. Navegação entre ocorrências relacionadas.
10. Propriedades e informações elétricas associadas.

Utilizar como referência funcional o EPLAN Electric P8.

Não copiar código, bibliotecas proprietárias ou arquivos licenciados.

---

# 3. REFERÊNCIAS FUNCIONAIS DO EPLAN

Consultar os materiais fornecidos pelo usuário, localizados historicamente em:

`C:\Dev\cad-craft\.local\references\eplan\`

Os materiais incluem imagens e vídeos de utilização do EPLAN.

Examinar as ferramentas e interações relevantes desta etapa.

Prestar atenção especial a:

* Device Navigator.
* Identificação de dispositivos.
* Propriedades de funções.
* Referências cruzadas.
* Contact image.
* Pontos de interrupção.
* Identificação de potenciais.
* Links entre páginas.
* Localização de ocorrências.

Consultar também a documentação pública do EPLAN 2027 quando necessário.

Referências oficiais:

https://eplan.help/en-us/Infoportal/Content/Plattform/2027/Content/htm/xessettingsgui_k_grundlagen.htm

https://eplan.help/en-us/Infoportal/Content/Plattform/2027/Content/htm/interruptionpointgui_k_darstellungabbruchstellen.htm

https://eplan.help/en-us/Infoportal/Content/Plattform/2027/Content/htm/xessettingsgui_h_qvwanzeigeoptimieren.htm

As referências devem orientar os comportamentos do programa, não apenas a aparência.

Registrar quais funcionalidades foram efetivamente implementadas e quais permanecem futuras.

---

# 4. IDENTIFICAÇÃO DE DISPOSITIVOS

Implementar ou completar o gerenciamento de identificações de dispositivos elétricos.

Exemplos:

* `-K1` — contator ou relé.
* `-Q1` — dispositivo de manobra.
* `-M1` — motor.
* `-S1` — elemento de comando.
* `-X1` — conjunto de bornes.

Esses exemplos não substituem uma política de classificação configurável.

## Requisitos

Permitir:

* Consultar a identificação.
* Alterar a identificação.
* Localizar o dispositivo.
* Visualizar funções associadas.
* Identificar ocorrências em outras páginas.
* Detectar conflitos de identificação.
* Preservar associações elétricas.

Não utilizar a identificação textual como substituta do ID interno.

O identificador persistente continua sendo a identidade real do objeto.

---

# 5. ESTRUTURA DE IDENTIFICAÇÃO

Preparar suporte para estruturas de identificação utilizadas na engenharia elétrica.

Considerar os blocos:

* `=` — designação funcional.
* `+` — localização.
* `-` — identificação de dispositivo.

Exemplo ilustrativo:

`=BOMBEAMENTO+PAINEL_01-K1`

## Requisitos

Separar conceitualmente:

* Identificação completa.
* Identificação exibida.
* Identificação funcional.
* Localização.
* Identificação do dispositivo.

Não realizar concatenações arbitrárias que produzam identificadores ambíguos.

Reutilizar as estruturas documentais e IEC já existentes no Core.

Criar validação de consistência e unicidade no escopo apropriado.

A edição de identificadores não poderá provocar perda de associações.

---

# 6. FUNÇÕES PRINCIPAIS E AUXILIARES

Um mesmo dispositivo pode possuir várias funções elétricas.

Exemplo:

Contator `-K1`:

* Bobina principal.
* Contato normalmente aberto.
* Contato normalmente fechado.
* Contatos auxiliares adicionais.

Essas funções podem estar distribuídas em páginas diferentes.

## Implementação

Permitir:

* Associar funções existentes a um dispositivo.
* Identificar a função principal.
* Identificar funções auxiliares.
* Localizar representações de cada função.
* Consultar a relação entre funções.
* Atualizar referências quando ocorrer alteração válida.

A associação deverá ser explícita e baseada no modelo elétrico.

Não agrupar funções somente porque apresentam textos semelhantes.

Não criar funções duplicadas automaticamente.

Não permitir que uma mesma função pertença indevidamente a dispositivos incompatíveis.

---

# 7. REFERÊNCIAS CRUZADAS AUTOMÁTICAS

Implementar um mecanismo de referências cruzadas derivadas dos dados do projeto.

## Objetivo

Permitir localizar outras ocorrências e funções de um dispositivo.

Exemplo:

A bobina de `-K1` está na página 3.

Um contato auxiliar de `-K1` está na página 7.

O CAD-CRAFT deverá apresentar referências que permitam navegar entre essas representações.

## Requisitos

As referências deverão incluir:

* Identificação do dispositivo.
* Função relacionada.
* Página.
* Localização documental, quando disponível.
* Identidade da representação de destino.

A posição deverá utilizar informações reais do documento.

Não inventar números de coluna ou linha quando a página não possuir uma estrutura de coordenadas documentais apropriada.

## Atualização

As referências deverão atualizar corretamente após:

* Movimentação.
* Mudança de página.
* Alteração de identificação.
* Alteração de associação.
* Inserção.
* Exclusão.
* Undo/Redo.
* Reimportação de snapshot.

Não persistir como fonte autoritativa informações que podem ser derivadas de relações existentes.

---

# 8. REFERÊNCIAS ENTRE BOBINAS E CONTATOS

Implementar um mecanismo equivalente, em conceito, à representação de contatos associados de um contator.

## Exemplo

Dispositivo `-K1`.

Bobina:

`A1 / A2`

Contatos:

`13 / 14`

`21 / 22`

`33 / 34`

O sistema deverá identificar as funções existentes, suas posições e seus vínculos com a bobina.

## Interface

Ao selecionar a bobina, apresentar:

* Contatos associados.
* Identificações.
* Páginas.
* Referências.
* Links de navegação.

Ao selecionar um contato, permitir localizar sua função principal.

Não gerar contatos fictícios apenas para preencher uma representação.

A visualização deverá refletir exclusivamente funções realmente presentes ou explicitamente cadastradas no Core.

Não implementar uma tabela de contatos apenas decorativa.

---

# 9. PONTOS DE INTERRUPÇÃO

Implementar pontos de interrupção utilizados para representar continuidade de circuitos entre regiões ou páginas.

## Objetivo

Permitir representar circuitos distribuídos sem exigir que todos os segmentos gráficos estejam na mesma página.

Exemplos de identificação:

* `L1`
* `N`
* `24VDC`
* `0V`
* `PE`

Esses nomes são exemplos, não garantem equivalência elétrica.

## Modelo

Cada ponto de interrupção deverá possuir:

* Identidade persistente.
* Identificação.
* Página.
* Posição.
* Referência documental.
* Associação elétrica explícita, quando existir.
* Relação com outros pontos pertinentes.

## Conectividade

Não unir redes simplesmente porque dois pontos possuem o mesmo nome.

A continuidade elétrica deverá decorrer de regras explícitas e validadas pelo Core.

Quando a semântica de continuidade entre páginas for implementada, deverá ser verificável e reversível.

Não criar redes elétricas fictícias.

---

# 10. REFERÊNCIAS DOS PONTOS DE INTERRUPÇÃO

Implementar referências cruzadas entre pontos relacionados.

Priorizar dois modos conceituais:

**Referência em estrela**

Um ponto principal apresenta referências aos demais pontos associados.

**Referência em cadeia**

Os pontos apresentam referências sucessivas em uma ordem determinada.

## Requisitos

* Ordenação determinística.
* Identidades estáveis.
* Localização documental real.
* Atualização após movimentações.
* Atualização após mudanças de página.
* Navegação entre pontos.
* Identificação de referências sem destino válido.
* Compatibilidade com Undo/Redo.

Quando houver ambiguidade, não escolher destinos arbitrariamente.

Distinguir associação documental de continuidade elétrica.

---

# 11. POTENCIAIS ELÉTRICOS

Implementar identificação de potenciais associados aos circuitos existentes.

Exemplos:

* `+24VDC`
* `0VDC`
* `L1`
* `L2`
* `L3`
* `N`
* `PE`

## Requisitos

O sistema deverá permitir:

* Identificar um potencial.
* Associá-lo explicitamente a uma rede ou ponto elétrico válido.
* Consultar sua identificação.
* Exibir a identificação no esquema.
* Localizar ocorrências relacionadas.
* Detectar informações incompatíveis.

A simples repetição de um nome não pode criar uma conexão elétrica.

A representação gráfica e a identidade da rede devem permanecer separadas.

## Propriedades

Permitir informações como:

* Nome.
* Tipo.
* Descrição.
* Associação elétrica existente.

Não inventar valores de tensão, frequência ou outras características elétricas.

Não realizar cálculos elétricos sem regras verificáveis.

Preparar a infraestrutura para verificações de engenharia posteriores.

---

# 12. NAVEGAÇÃO ENTRE REFERÊNCIAS

Integrar referências cruzadas ao navegador existente.

## Funcionalidades

Ao selecionar uma referência:

1. Identificar o destino.
2. Abrir sua página.
3. Centralizar a representação.
4. Destacar o elemento.
5. Atualizar propriedades.
6. Preservar a navegação do projeto.

Implementar links clicáveis quando apropriado.

O usuário deverá conseguir navegar diretamente entre:

* Bobina e contatos.
* Função principal e auxiliares.
* Ocorrências de um dispositivo.
* Pontos de interrupção relacionados.
* Identificações de potenciais.

Não criar um navegador paralelo desnecessário.

---

# 13. INTEGRAÇÃO COM O PAINEL DE PROPRIEDADES

O painel existente deverá apresentar as informações correspondentes ao objeto selecionado.

## Dispositivos

* ID.
* Identificação exibida.
* Identificação completa.
* Funções associadas.
* Representações.
* Referências cruzadas.

## Funções

* Dispositivo associado.
* Papel principal ou auxiliar.
* Identificação.
* Página.
* Referências.

## Pontos de interrupção

* Identificação.
* Associação elétrica.
* Referências.
* Página.
* Localização.

## Potenciais

* Identificação.
* Rede associada.
* Propriedades disponíveis.

As alterações deverão utilizar comandos validados.

Não modificar diretamente estruturas internas ignorando as APIs do Core.

---

# 14. RIBBON E FERRAMENTAS

Utilizar a interface existente.

Preservar a identidade visual aprovada na Etapa 5.

Integrar os novos recursos às guias apropriadas.

## Dispositivos

* Identificação.
* Funções associadas.
* Referências cruzadas.
* Localizar dispositivo.
* Localizar função principal.
* Localizar funções auxiliares.

## Inserir

* Ponto de interrupção.
* Identificação de potencial.

## Conexões

* Consultar potencial.
* Localizar rede.
* Navegar entre referências.

## Propriedades

Disponibilizar edição dos campos suportados.

Não criar comandos fictícios.

Todos os comandos deverão utilizar o dispatcher central do CAD-CRAFT.

---

# 15. INTEGRIDADE ELÉTRICA

Esta seção é obrigatória.

Preservar:

* DeviceId.
* FunctionId.
* RepresentationId.
* PageId.
* ConnectionPointId.
* ConnectionId.
* Net.
* Condutores físicos.
* Bornes e cabos existentes.

A identificação de um dispositivo não substitui sua identidade interna.

A alteração de uma referência documental não poderá modificar sua conectividade elétrica por efeito colateral.

Não unir redes apenas porque apresentam o mesmo potencial textual.

Não criar dispositivos ou funções fictícias.

Não modificar condutores físicos para representar referências cruzadas.

---

# 16. HISTÓRICO TRANSACIONAL

Integrar as operações ao sistema existente.

Cobrir:

* Alterações de identificação.
* Associações entre funções e dispositivos.
* Inserção e edição de pontos de interrupção.
* Alterações de potenciais.
* Alterações documentais relacionadas.

Cada operação confirmada deverá produzir uma transação lógica.

Undo/Redo deverá restaurar:

* Identidades.
* Associações.
* Identificações.
* Relações documentais.
* Geometria aplicável.
* Referências derivadas.

Não registrar mudanças de seleção, zoom ou pan no histórico documental.

Não criar snapshots integrais do projeto a cada interação.

---

# 17. PERSISTÊNCIA E COMPATIBILIDADE

Preservar compatibilidade com o sistema de snapshots existente.

Validar:

* Identificadores.
* Associações.
* Referências.
* Relações funcionais.
* Pontos de interrupção.
* Informações de potencial.
* Integridade elétrica.

Snapshots anteriores deverão continuar sendo aceitos.

Novos campos precisam possuir regras de migração ou valores padrão apropriados.

Não criar o formato `.cadcraft` definitivo nesta etapa.

Não implementar autosave ou backup avançado fora do roadmap.

---

# 18. DESEMPENHO E CONSISTÊNCIA

Projetos industriais podem conter milhares de dispositivos e referências.

Evitar:

* Reconstruir índices em cada frame.
* Recalcular todas as referências durante hover.
* Consultas integrais desnecessárias.
* Clonagem constante do Project.
* Caches que não sejam invalidados após operações.
* Referências baseadas em índices transitórios.

Preferir índices ou consultas derivadas determinísticas.

A invalidação deverá ocorrer quando uma alteração relevante modificar os dados utilizados.

Não otimizar prematuramente sem evidências.

---

# 19. TESTES AUTOMATIZADOS

Preservar os 339 testes existentes.

Adicionar testes para:

1. Correção do redimensionamento horizontal.
2. Correção do redimensionamento vertical.
3. Retorno à geometria inicial.
4. Rotação e redimensionamento.
5. Preservação de vértices manuais.
6. Preservação de junções.
7. Undo/Redo da movimentação conectada.
8. Identificação de dispositivos.
9. Identificação completa.
10. Unicidade e conflitos.
11. Alteração de identificadores.
12. Funções principais.
13. Funções auxiliares.
14. Referências entre bobinas e contatos.
15. Referências entre páginas.
16. Atualização automática de referências.
17. Navegação por IDs.
18. Links do painel de propriedades.
19. Pontos de interrupção.
20. Referências em estrela.
21. Referências em cadeia.
22. Potenciais elétricos.
23. Associação explícita a redes.
24. Prevenção de conexões implícitas.
25. Preservação de condutores físicos.
26. Persistência.
27. Compatibilidade retroativa.
28. Undo/Redo.
29. Integridade referencial.
30. Eventos reais da interface egui.

Testar operações compostas e cenários negativos.

Não remover ou desabilitar testes anteriores.

---

# 20. ESTRATÉGIA DE EXECUÇÃO — EFICIÊNCIA SEM REDUZIR QUALIDADE

O objetivo é reduzir consumo desnecessário de recursos do agente.

## Regras

1. Consultar a memória persistente em vez de reescrever toda a arquitetura.
2. Criar um plano técnico objetivo.
3. Utilizar testes direcionados durante a implementação.
4. Executar a validação completa ao concluir a etapa.
5. Evitar repetir builds integrais sem alterações relevantes.
6. Evitar revisões independentes redundantes.
7. Utilizar subagentes somente quando agregarem valor técnico real.
8. Não reler arquivos extensos sem necessidade.
9. Documentar decisões e resultados de forma objetiva.
10. Não reduzir os critérios de aceitação.

**Estas regras não autorizam diminuir a cobertura necessária, ignorar falhas ou encerrar a implementação incompleta.**

Manter TDD, revisões e validações aplicáveis.

---

# 21. VALIDAÇÃO FINAL

Executar:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

Executar:

```powershell
cargo build --release -p cad-gui --bin cad-gui --locked
git diff --check
git status --short
```

Todas as verificações devem ser aprovadas.

Realizar uma revisão independente focada principalmente em:

* Correção geométrica dos fios.
* Integridade das conexões.
* Referências cruzadas.
* Associações funcionais.
* Potenciais.
* Undo/Redo.
* Compatibilidade documental.

Corrigir os problemas identificados.

Não desabilitar verificações nem testes para obter aprovação artificial.

---

# 22. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md`

Criar especificação e plano técnico correspondentes.

Registrar separadamente:

## Correção da Etapa 8

* Causa-raiz identificada.
* Arquivos modificados.
* Estratégia adotada.
* Regressões adicionadas.
* Resultados dos testes.

## Funcionalidades da Etapa 9

* Identificação de dispositivos.
* Funções principais e auxiliares.
* Referências cruzadas.
* Referências entre bobinas e contatos.
* Pontos de interrupção.
* Potenciais.
* Navegação.
* Histórico.
* Persistência.

Atualizar os arquivos pertinentes em `docs/internal/memory/`.

Preservar o roadmap futuro.

A melhoria visual da densidade da grade continua pendente.

A Etapa 17 continua reservada ao plugin nativo CAD-CRAFT AI, com OpenAI como único provedor.

---

# 23. CRITÉRIOS DE ENTREGA

A etapa estará tecnicamente concluída quando:

* [ ] O redimensionamento dos fios funcionar nos dois sentidos.
* [ ] Não existirem prolongamentos residuais indevidos.
* [ ] Os fios mantiverem conectividade elétrica correta.
* [ ] Rotação e movimentação funcionarem conjuntamente.
* [ ] Undo/Redo preservar geometria e conectividade.
* [ ] Dispositivos possuírem identificação consistente.
* [ ] Funções principais e auxiliares estiverem associadas corretamente.
* [ ] Referências cruzadas forem geradas a partir de dados reais.
* [ ] Bobinas e contatos puderem ser localizados reciprocamente.
* [ ] Pontos de interrupção funcionarem.
* [ ] Referências entre páginas funcionarem.
* [ ] Potenciais puderem ser identificados e consultados.
* [ ] Não ocorrer criação implícita de redes por textos coincidentes.
* [ ] Navegação por referências funcionar.
* [ ] Propriedades refletirem o Core.
* [ ] Operações implementadas forem transacionais.
* [ ] Snap e grade lógica permanecerem independentes do zoom.
* [ ] Snapshots anteriores permanecerem compatíveis.
* [ ] Testes automatizados forem aprovados.
* [ ] Build Release for aprovado.
* [ ] Revisão independente for concluída.
* [ ] Memória persistente estiver atualizada.

Não considerar um item concluído se estiver implementado apenas como placeholder, protótipo não integrado ou teste isolado sem funcionalidade correspondente.

---

# 24. RESTRIÇÕES FINAIS

Não executar commit.

Não executar push.

Não iniciar a Etapa 10.

Não solicitar homologação manual ao usuário.

Não modificar funcionalidades aprovadas sem justificativa técnica.

Não reconstruir a interface gráfica.

Não criar um segundo modelo elétrico.

Não criar relações elétricas por coincidência textual.

Não antecipar a Etapa 17.

Não eliminar testes anteriores.

Não declarar homologação manual.

**Ao terminar, apresentar um relatório técnico objetivo contendo as funcionalidades implementadas, a correção do redimensionamento, os resultados dos testes, a revisão independente, as limitações e os arquivos modificados.**

Aguardar o usuário para realizar o commit manual da Etapa 9.
