# CAD-CRAFT — ETAPA 7

## BIBLIOTECA DE SÍMBOLOS ELÉTRICOS E EDITOR DE INSERÇÃO GRÁFICA

### Rust + egui/eframe — Windows Desktop

## 0. AUTORIZAÇÃO E REGRAS

Você está autorizado a desenvolver a Etapa 7 do CAD-CRAFT.

Diretório:

`C:\Dev\cad-craft`

Repositório:

`Vandecao/CAD-CRAFT`

Branch:

`main`

Esta é uma nova conversa de desenvolvimento.

Todo o contexto deve ser recuperado dos arquivos do próprio repositório.

**Regras obrigatórias:**

1. Não executar commit.
2. Não executar push.
3. Não executar homologação manual com o usuário.
4. Não iniciar a Etapa 8.
5. Não antecipar funcionalidades da Etapa 17.
6. Não modificar o roadmap sem justificativa técnica e rastreabilidade.
7. Preservar integralmente o Core elétrico existente.
8. Executar todos os testes automatizados.
9. Utilizar TDD sempre que possível.
10. Realizar revisão independente quando disponível.
11. Atualizar a memória persistente ao concluir.

O usuário decidiu concentrar a homologação manual abrangente após a Etapa 15.

Isso não elimina a obrigação de executar testes automatizados, análises estáticas e validações técnicas em todas as etapas.

---

# 1. RECUPERAÇÃO DO CONTEXTO

Antes de implementar qualquer funcionalidade, leia:

* `AGENTS.md`
* `docs/internal/memory/HANDOFF.md`
* `docs/internal/memory/CURRENT_STATE.md`
* `docs/internal/memory/ROADMAP.md`
* `docs/internal/memory/ARCHITECTURE.md`
* `docs/internal/memory/DECISIONS.md`
* `docs/internal/memory/KNOWN_ISSUES.md`
* ADRs existentes.
* Especificação e relatório da Etapa 5.
* Especificação e relatório da Etapa 6.
* Contratos documentais da interface.

Verifique também:

`docs/internal/spec/ETAPA_17_ASSISTENTE_IA_OPENAI.md`

Esse último arquivo estabelece a futura implementação do assistente nativo de IA.

**Não implementar o assistente nesta etapa.**

Confirme que seu planejamento permanece registrado.

## Verificação do Git

Execute:

```powershell
git status -sb
git log -3 --oneline
git remote -v
```

Confirme que a Etapa 6 foi incorporada ao histórico e que não existem alterações pendentes inesperadas.

Se houver pendência de commit, push ou arquivos não registrados da Etapa 6, comunique-a antes de modificar o projeto.

Não resolva divergências de Git automaticamente.

## Prioridade do roadmap

Leia a descrição oficial da Etapa 7 no roadmap.

Se ela definir um escopo diferente do apresentado neste prompt, preserve o planejamento existente.

Não substitua silenciosamente decisões anteriores.

A proposta abaixo descreve a evolução natural para uma biblioteca de símbolos e inserção gráfica. Ela deverá ser conciliada com o roadmap antes da implementação.

---

# 2. OBJETIVO PRINCIPAL

Implementar a infraestrutura necessária para trabalhar com símbolos elétricos reais no CAD-CRAFT.

Até a Etapa 6, o viewport permitia manipular representações documentais simplificadas.

Agora queremos evoluir para um sistema com:

* Definições de símbolos.
* Geometria vetorial.
* Pontos de conexão.
* Biblioteca de símbolos.
* Inserção gráfica.
* Posicionamento preciso.
* Rotação.
* Espelhamento.
* Associação com entidades elétricas.
* Integração com propriedades.
* Undo/Redo.
* Persistência documental.

O funcionamento deverá se aproximar dos fluxos de trabalho de um ECAD profissional, especialmente do EPLAN Electric P8.

Não copiar código-fonte, arquivos proprietários ou recursos licenciados do EPLAN.

---

# 3. ARQUITETURA DOS SÍMBOLOS

Criar uma separação explícita entre:

**SymbolDefinition**

Representa a definição reutilizável de um símbolo.

Deverá conter:

* Identificador estável.
* Nome.
* Categoria.
* Geometria vetorial.
* Pontos de conexão.
* Orientações permitidas.
* Metadados.
* Versão da definição, quando necessária.

**SymbolInstance**

Representa uma ocorrência inserida em uma página.

Deverá conter ou referenciar:

* Identificador da representação.
* Definição utilizada.
* Página.
* Transformação.
* Rotação.
* Espelhamento.
* Visibilidade.
* Associação à entidade elétrica, quando aplicável.

**ElectricalEntity**

Continua pertencendo ao Core existente.

Uma instância gráfica não deve duplicar as informações elétricas de sua entidade associada.

Reutilize as estruturas existentes sempre que possível.

Evite criar uma arquitetura paralela às representações documentais da Etapa 4.

---

# 4. GEOMETRIA VETORIAL

Implementar primitivas vetoriais suficientes para representar símbolos elétricos básicos.

Considerar:

* Linha.
* Polilinha.
* Retângulo.
* Círculo.
* Arco.
* Texto.
* Pontos de ancoragem.

As primitivas devem utilizar coordenadas locais em milímetros.

A transformação da instância será aplicada sobre essa geometria.

## Requisitos

* Renderização independente da resolução.
* Zoom sem deformações.
* Rotação correta.
* Espelhamento correto.
* Coordenadas determinísticas.
* Hit-testing compatível.
* Bounding boxes calculadas a partir da geometria.
* Recorte pelo viewport.
* Evitar alocações desnecessárias por frame.

Reutilizar o motor de câmera e transformações existente.

---

# 5. BIBLIOTECA INICIAL DE SÍMBOLOS

Criar uma biblioteca inicial extensível, contendo exemplos representativos de símbolos elétricos industriais.

Priorizar símbolos como:

1. Contato normalmente aberto.
2. Contato normalmente fechado.
3. Bobina de relé ou contator.
4. Motor elétrico.
5. Borne.
6. Botão de comando.
7. Chave ou interruptor.
8. Sensor.
9. Terra de proteção.
10. Elemento de alimentação.

Utilizar convenções gráficas IEC pertinentes.

Não declarar conformidade normativa completa sem verificação.

Não tentar implementar todas as classes de símbolos existentes no EPLAN nesta etapa.

O mais importante é construir uma arquitetura que permita ampliar a biblioteca posteriormente.

---

# 6. PONTOS DE CONEXÃO

Cada definição de símbolo poderá possuir pontos de conexão explícitos.

Esses pontos deverão ter:

* ID estável.
* Posição local.
* Orientação.
* Identificação.
* Tipo ou papel, quando aplicável.

A posição documental de cada ponto será calculada pela transformação da instância.

## Regra crítica

A presença de dois pontos próximos ou coincidentes não cria automaticamente uma conexão elétrica.

A conectividade deve continuar explícita no Core.

Nesta etapa, preparar a geometria e a identificação dos pontos de conexão.

Não antecipar o editor completo de fios e o roteamento elétrico, caso estejam reservados para etapas posteriores.

---

# 7. NAVEGADOR DA BIBLIOTECA

Integrar a biblioteca à interface existente.

Utilizar preferencialmente o painel de inserção já implementado na Etapa 5.

Funcionalidades:

* Navegar por categorias.
* Localizar símbolos por nome.
* Visualizar miniatura.
* Selecionar símbolo.
* Iniciar inserção na página ativa.

Não reconstruir a Ribbon.

Não criar uma segunda interface independente para a biblioteca.

Manter aparência consistente com o restante do CAD-CRAFT.

---

# 8. INSERÇÃO GRÁFICA

Implementar uma ferramenta de inserção de símbolos.

Fluxo esperado:

1. Selecionar símbolo na biblioteca.
2. Ativar ferramenta de inserção.
3. Movimentar cursor sobre o viewport.
4. Mostrar pré-visualização do símbolo.
5. Aplicar snap, quando habilitado.
6. Clicar para inserir.
7. Criar representação válida no Project.
8. Atualizar navegadores e propriedades.
9. Registrar operação no histórico.

## Pré-visualização

Durante a movimentação:

* Não modificar o Project.
* Não criar histórico por frame.
* Não gerar IDs definitivos desnecessariamente.
* Preservar a câmera.
* Respeitar o zoom.

## Cancelamento

Esc deverá cancelar a inserção.

Cancelar não deve criar objetos nem modificar o histórico.

## Inserção repetida

Avaliar suporte para inserir várias instâncias do mesmo símbolo.

O comportamento deverá ser explícito e consistente.

---

# 9. POSICIONAMENTO E SNAP

Preservar completamente o sistema de coordenadas aprovado.

Origem:

Canto inferior esquerdo.

Eixos:

X positivo para a direita.

Y positivo para cima.

Unidade:

Milímetros.

## Regra de grade

Se a grade lógica estiver configurada para 4 mm, deverá continuar sendo de 4 mm em qualquer zoom.

O snap utiliza a grade lógica.

A renderização visual poderá reduzir a densidade dos pontos sem alterar o espaçamento real.

Não confundir:

* Grade lógica.
* Grade renderizada.
* Snap.

A melhoria futura de suavização visual da grade já está registrada no roadmap.

Preservar essa pendência.

Não alterar a escala real para melhorar a aparência do viewport.

---

# 10. INTEGRAÇÃO COM A ETAPA 6

Todos os símbolos inseridos deverão utilizar as ferramentas existentes.

Verificar:

* Seleção simples.
* Seleção múltipla.
* Seleção retangular.
* Hit-testing.
* Hover.
* Arraste.
* Movimentação em grupo.
* Rotação.
* Espelhamento.
* Edição numérica.
* Undo/Redo.
* Menus contextuais.

Não desenvolver um segundo controlador de interação exclusivo para símbolos.

Quando necessário, generalizar os componentes existentes para trabalhar com geometria vetorial.

Preservar o funcionamento das representações documentais anteriores.

---

# 11. HISTÓRICO TRANSACIONAL

A Etapa 6 implementou histórico de transformações.

Verifique sua arquitetura e estenda-a de forma controlada.

Operações que precisam ser reversíveis:

* Inserção de símbolo.
* Exclusão de instância, se implementada.
* Movimentação.
* Rotação.
* Espelhamento.
* Alterações editáveis de propriedades.

## Requisitos

* Uma operação confirmada equivale a uma transação.
* Undo deve restaurar o estado anterior.
* Redo deve reaplicar a operação.
* IDs precisam permanecer consistentes.
* Associações elétricas devem ser preservadas.
* Não criar histórico durante pré-visualizações.

Não implementar Undo genérico de todo o sistema se isso estiver reservado para etapa posterior.

Estender apenas o necessário para esta implementação.

---

# 12. PAINEL DE PROPRIEDADES

Ao selecionar um símbolo, apresentar:

* Nome.
* Categoria.
* Identificador.
* Página.
* Referência da definição.
* Referência da entidade elétrica, quando houver.
* X.
* Y.
* Rotação.
* Espelhamento.
* Pontos de conexão existentes.
* Outras propriedades pertinentes já suportadas.

As propriedades editáveis deverão utilizar comandos validados.

Não permitir alteração direta de estruturas internas sem passar pelas regras do Core.

---

# 13. RELAÇÃO COM DISPOSITIVOS ELÉTRICOS

Uma definição de símbolo poderá ser associada a uma função elétrica.

Entretanto, não presumir que todos os símbolos representem dispositivos físicos independentes.

Um mesmo equipamento poderá ter várias funções ou representações.

Preservar a possibilidade futura de trabalhar com:

* Contatores com múltiplos contatos.
* Bobinas.
* Relés.
* Bornes.
* Módulos de CLP.
* Sensores.
* Motores.
* Dispositivos multifuncionais.

Não criar automaticamente um novo dispositivo físico para cada representação gráfica sem uma regra de domínio explícita.

---

# 14. PERSISTÊNCIA

Verificar o mecanismo documental existente.

Garantir que os símbolos e suas transformações sejam preservados corretamente.

Requisitos:

* IDs estáveis.
* Referências válidas.
* Geometria reconstituível.
* Compatibilidade com snapshots antigos.
* Serialização determinística.
* Validação de definições ausentes ou inválidas.

Não implementar formatos definitivos fora do roadmap.

Não alterar schemas desnecessariamente.

Se for necessária uma migração, documentar sua justificativa e criar testes de compatibilidade.

---

# 15. DESEMPENHO

Evitar:

* Clonar o Project a cada frame.
* Recriar toda a biblioteca por renderização.
* Recalcular geometria imutável sem necessidade.
* Criar transações durante mouse move.
* Desenhar símbolos fora da região visível.
* Executar buscas integrais desnecessárias em bibliotecas grandes.

A arquitetura deverá permitir a expansão futura para milhares de instâncias distribuídas em várias páginas.

Realizar medições proporcionais ao escopo implementado.

---

# 16. TESTES AUTOMATIZADOS

Preservar os 236 testes aprovados na Etapa 6.

Criar testes adicionais para:

1. Definição de símbolos.
2. Identidade das definições.
3. Identidade das instâncias.
4. Renderização das primitivas.
5. Cálculo de limites geométricos.
6. Rotação.
7. Espelhamento.
8. Transformação de pontos de conexão.
9. Hit-testing.
10. Inserção.
11. Cancelamento.
12. Inserção repetida, caso suportada.
13. Snap.
14. Grade oculta.
15. Seleção.
16. Movimentação.
17. Movimentação múltipla.
18. Undo de inserção.
19. Redo de inserção.
20. Exclusão reversível, caso implementada.
21. Integridade dos IDs.
22. Preservação da conectividade.
23. Serialização.
24. Compatibilidade de snapshots.
25. Integração com propriedades.
26. Navegação na biblioteca.
27. Comandos da Ribbon.
28. Comportamento do viewport.
29. Coordenadas negativas.
30. Casos de geometria inválida.

Os testes deverão validar comportamento real, não apenas compilação.

---

# 17. VALIDAÇÃO OBRIGATÓRIA

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

Corrigir todas as falhas.

Não remover testes existentes.

Não silenciar avisos do Clippy para ocultar problemas.

Não desabilitar verificações.

---

# 18. REVISÃO INDEPENDENTE

Executar revisão independente quando disponível.

Priorizar:

* Separação entre definição e instância.
* Integridade do domínio elétrico.
* Consistência geométrica.
* Identidade de objetos.
* Pontos de conexão.
* Histórico transacional.
* Compatibilidade de snapshots.
* Performance.
* Regressões da Etapa 6.

Corrigir problemas identificados antes de encerrar.

---

# 19. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_07_BIBLIOTECA_SIMBOLOS.md`

Documentar:

* Arquitetura.
* Biblioteca inicial.
* Modelo de símbolos.
* Geometria.
* Pontos de conexão.
* Inserção.
* Seleção.
* Transformações.
* Snap.
* Histórico.
* Integração com o Core.
* Persistência.
* Testes.
* Revisão independente.
* Limitações.
* Arquivos modificados.

Atualizar a memória persistente:

* CURRENT_STATE.
* HANDOFF.
* ROADMAP.
* ARCHITECTURE, quando necessário.
* DECISIONS, quando necessário.
* KNOWN_ISSUES, quando necessário.

Preservar a especificação da Etapa 17.

Não registrar homologação manual.

---

# 20. CONDIÇÕES DE ENTREGA

A Etapa 7 deverá entregar:

* Biblioteca inicial funcional.
* Definições reutilizáveis de símbolos.
* Representações vetoriais.
* Instâncias documentais.
* Pontos de conexão explícitos.
* Inserção gráfica.
* Integração com seleção e movimentação.
* Snap em milímetros.
* Transformações.
* Undo/Redo das operações implementadas.
* Propriedades integradas.
* Compatibilidade documental.
* Testes automatizados aprovados.
* Build Release aprovado.
* Documentação atualizada.
* Revisão técnica concluída.

Não executar commit ou push.

Não solicitar testes manuais ao usuário.

Não declarar homologação final.

Não iniciar a Etapa 8.

---

# 21. RESULTADO ESPERADO

O CAD-CRAFT deverá evoluir de um editor capaz de manipular representações documentais para um ECAD com infraestrutura real de símbolos elétricos.

O usuário deverá conseguir escolher um símbolo em uma biblioteca, inseri-lo na página e manipulá-lo utilizando as ferramentas existentes.

A arquitetura deverá permitir a evolução futura para projetos multifilares completos, conexões gráficas, referências cruzadas, macros e bibliotecas de fabricantes.

Não antecipar funcionalidades de etapas futuras sem necessidade arquitetural.

Ao terminar:

1. Execute todas as validações automatizadas.
2. Realize a revisão independente.
3. Atualize a memória persistente.
4. Produza o relatório técnico.
5. Apresente o resumo final e a quantidade de testes aprovados.
6. Encerre sem commit e sem push.

**O desenvolvimento deve continuar respeitando o roadmap aprovado, sem antecipações ou mudanças silenciosas de escopo.**
