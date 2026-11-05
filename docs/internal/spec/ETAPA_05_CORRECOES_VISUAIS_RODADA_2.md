# CAD-CRAFT — ETAPA 5

## CORREÇÕES DE HOMOLOGAÇÃO VISUAL — GRADE, CONTRASTE, BARRA DE TÍTULO E RIBBON

### AUTORIZAÇÃO

Continuar a implementação da Etapa 5 do CAD-CRAFT para corrigir os problemas identificados na homologação visual realizada pelo usuário.

**Diretório do projeto:**

`C:\Dev\cad-craft`

**Tecnologias:**

* Rust.
* egui/eframe 0.33.3, conforme a versão efetivamente utilizada no projeto.
* Windows 10 e Windows 11.

**Branch:** `main`.

**Commit-base conhecido:** `5eac964`.

**Linha de base conhecida:** 179 testes automatizados aprovados.

Esta solicitação NÃO constitui uma nova etapa.

Não iniciar a Etapa 6.

Não executar commit.

Não executar push.

Preservar todas as alterações existentes e as funcionalidades que já foram aprovadas pelo usuário.

## RESULTADO DA PRIMEIRA HOMOLOGAÇÃO

O usuário executou o CAD-CRAFT Release no Windows e confirmou o funcionamento dos seguintes recursos:

* Aplicativo gráfico inicializa corretamente.
* Projeto demonstrativo abre.
* Navegador de páginas funciona.
* Hierarquia documental funciona.
* Abas de documentos funcionam.
* Painéis são redimensionáveis.
* Centro de inserção funciona.
* Propriedades são exibidas.
* Navegação entre documentos funciona.
* Zoom e pan funcionam.
* Viewport permanece disponível independentemente do tamanho da folha.
* Menu Arquivo abre corretamente o Backstage.
* Navegação lateral do Backstage funciona.

A interface foi considerada visualmente satisfatória em sua estrutura geral.

Entretanto, quatro problemas foram identificados:

1. A grade está quadriculada, mas deverá ser composta por pontos.
2. A tela inicial apresenta botões com contraste inadequado.
3. A barra de título nativa do Windows está branca, mas deverá utilizar tema escuro.
4. Os grupos e botões da Ribbon desaparecem intermitentemente ao alternar guias.

Esses quatro problemas deverão ser investigados e corrigidos.

---

# 1. AUDITORIA INICIAL OBRIGATÓRIA

Antes de modificar arquivos:

1. Ler `AGENTS.md`.
2. Consultar `docs/internal/memory/HANDOFF.md`.
3. Consultar `docs/internal/memory/CURRENT_STATE.md`.
4. Consultar `docs/internal/memory/ARCHITECTURE.md`.
5. Consultar as decisões arquiteturais relevantes.
6. Ler `docs/internal/reports/ETAPA_05_INTERFACE_GRAFICA_EGUI.md`.
7. Inspecionar a implementação atual da GUI.
8. Verificar o estado do Git.
9. Identificar os componentes envolvidos nos quatro problemas.
10. Planejar as correções preservando a arquitetura existente.

Inspecionar especialmente:

* `crates/cad-gui/src/viewport.rs`
* `crates/cad-gui/src/camera.rs`
* `crates/cad-gui/src/theme.rs`
* `crates/cad-gui/src/app.rs`
* `crates/cad-gui/src/main.rs`
* `crates/cad-gui/src/ribbon.rs`
* `crates/cad-gui/src/state.rs`
* `crates/cad-gui/src/preferences.rs`

Os caminhos são referências conhecidas. Confirmar sua existência e organização atual antes de modificar.

Não reimplementar módulos que já funcionam.

Não alterar o Core elétrico desnecessariamente.

---

# 2. CORREÇÃO 1 — GRADE DE PONTOS ESTILO ECAD

## 2.1. Problema

Atualmente, o viewport apresenta uma malha quadriculada.

São desenhadas linhas horizontais e verticais que atravessam toda a região gráfica.

O usuário deseja uma grade composta exclusivamente por pequenos pontos, semelhante à visualização utilizada no EPLAN Electric P8.

## 2.2. Resultado desejado

Substituir a grade de linhas por uma grade de pontos.

Exemplo conceitual:

```text
·       ·       ·       ·       ·

·       ·       ·       ·       ·

·       ·       ·       ·       ·

·       ·       ·       ·       ·

·       ·       ·       ·       ·
```

Cada ponto deverá representar uma interseção real da grade documental.

Não utilizar linhas contínuas.

Não utilizar cruzes grandes como substitutas dos pontos.

Não modificar a geometria do documento.

## 2.3. Sistema de coordenadas

Preservar rigorosamente:

* Coordenadas em milímetros.
* Origem inferior esquerda.
* X positivo para a direita.
* Y positivo para cima.
* Rotação positiva anti-horária.
* Transformação world-to-screen existente.
* Transformação screen-to-world existente.

Os pontos deverão ser posicionados utilizando a câmera atual.

Não usar coordenadas absolutas de tela como origem da grade.

A origem da grade deverá continuar associada ao sistema documental.

## 2.4. Aparência visual

Utilizar:

* Cor cinza discreta.
* Pontos pequenos.
* Boa definição em diferentes DPI.
* Antialiasing quando apropriado.
* Contraste suficiente sobre fundo branco.
* Aparência semelhante à grade técnica do EPLAN.

O raio visual inicial poderá ficar aproximadamente entre 0,6 e 1,0 ponto lógico do egui.

Ajustar conforme a qualidade da renderização real.

Não utilizar pontos excessivamente grandes.

Não permitir que a grade prejudique a leitura dos desenhos.

## 2.5. Zoom

Durante o zoom in:

* A distância visual entre os pontos deverá aumentar.
* Os pontos permanecerão alinhados ao documento.
* O posicionamento deverá continuar estável.

Durante o zoom out:

* A distância visual diminuirá.
* A grade deverá reduzir sua densidade visual quando necessário.
* O sistema deverá evitar excesso de pontos.
* A renderização não poderá produzir uma massa cinza ou uma malha visual contínua.

O espaçamento lógico configurado pelo usuário NÃO deverá mudar automaticamente.

Somente a densidade de renderização poderá ser adaptada.

O snap continuará utilizando a resolução lógica original.

## 2.6. Desempenho

Implementar renderização limitada à região visível.

Calcular somente pontos que possam aparecer no viewport.

Utilizar:

* Culling.
* Limites de densidade.
* Ajuste adaptativo.
* Limites seguros de quantidade de pontos.
* Transformações numéricas estáveis.

Não iterar sobre uma área documental ilimitada.

Não gerar milhões de primitivas gráficas durante zoom muito afastado.

Não introduzir dependências externas desnecessárias.

## 2.7. Área externa à folha

A grade deverá preservar seu alinhamento mesmo quando aparecer fora dos limites da folha.

O usuário poderá realizar pan e visualizar regiões externas sem deslocamento incoerente dos pontos.

A folha continuará sendo um objeto documental independente.

## 2.8. Controles da barra inferior

Preservar:

* Mostrar grade.
* Ocultar grade.
* Selecionar espaçamento.
* Snap.
* Zoom.
* Coordenadas.

Manter os espaçamentos disponíveis:

* 1 mm.
* 2 mm.
* 2,5 mm.
* 4 mm.
* 8 mm.
* 16 mm.

Preservar também os demais valores configuráveis já suportados.

## 2.9. Pré-visualização

Verificar a renderização das miniaturas de páginas.

Evitar que a grade de pontos provoque ruído excessivo em miniaturas pequenas.

Quando necessário, ocultar a grade nas miniaturas.

Não alterar suas dimensões documentais.

## 2.10. Testes obrigatórios

Adicionar regressões para:

1. Posição dos pontos.
2. Espaçamento.
3. Coordenadas negativas.
4. Zoom.
5. Pan.
6. Visibilidade.
7. Snap independente.
8. Densidade adaptativa.
9. DPI.
10. Limites de renderização.
11. Ausência de linhas contínuas da grade.
12. Viewport independente da folha.

---

# 3. CORREÇÃO 2 — CONTRASTE DA TELA INICIAL

## 3.1. Problema

Quando não existe página aberta, o viewport apresenta a tela inicial do CAD-CRAFT.

O usuário identificou botões com fundo escuro e texto igualmente escuro.

O contraste é insuficiente e dificulta a leitura.

## 3.2. Objetivo

Corrigir todos os problemas de legibilidade da tela inicial.

Preservar:

* Logotipo próprio do CAD-CRAFT.
* Título.
* Subtítulo.
* Instruções.
* Botões de ação.
* Centralização.
* Organização atual.

## 3.3. Botões

Revisar:

* Criar sessão em memória.
* Abrir demonstração.
* Importar snapshot técnico.

Para botões escuros, utilizar texto claro.

Para botões claros, utilizar texto escuro.

Utilizar azul para destacar a ação principal, quando adequado.

Garantir contraste suficiente nos estados:

* Normal.
* Hover.
* Pressionado.
* Focado.
* Desabilitado.

## 3.4. Estados desabilitados

Não confundir estado desabilitado com erro de contraste.

O botão desabilitado deverá possuir aparência distinta, mas seu texto precisa continuar identificável.

Não permitir texto praticamente invisível.

## 3.5. Texto auxiliar

Revisar:

* Nome do aplicativo.
* Descrição.
* Instruções.
* Mensagens de estado.
* Informações adicionais.

Garantir contraste adequado com o fundo da região central.

## 3.6. Estilos

Investigar possíveis conflitos entre:

* Estilo global do egui.
* Estilo local dos widgets.
* Cor de fundo.
* Cor do texto.
* Estados visuais.
* Widgets desabilitados.

Centralizar as correções na camada apropriada.

Evitar alterar globalmente todos os controles da interface quando apenas a tela inicial estiver afetada.

## 3.7. Responsividade

Verificar a tela inicial em:

* 1280 × 720.
* 1366 × 768.
* 1600 × 900.
* 1920 × 1080.

DPI:

* 100%.
* 125%.
* 150%.
* 200%.

Os elementos deverão permanecer centralizados e legíveis.

---

# 4. CORREÇÃO 3 — BARRA DE TÍTULO NATIVA ESCURA

## 4.1. Problema identificado

A barra superior nativa da janela do Windows aparece branca.

O restante do CAD-CRAFT utiliza um tema escuro.

Isso provoca uma inconsistência visual evidente.

O usuário deseja que a barra de título seja preta ou cinza quase preto, harmonizada com o restante do aplicativo.

## 4.2. Objetivo

Implementar uma barra de título escura no Windows 10 e Windows 11, quando suportada pelo sistema.

Preservar:

* Nome CAD-CRAFT.
* Nome do projeto.
* Ícone.
* Minimizar.
* Maximizar.
* Restaurar.
* Fechar.
* Redimensionamento.
* Movimento da janela.
* Comportamento nativo.

## 4.3. Investigação técnica

Verificar:

1. Como o eframe 0.33.3 configura a janela nativa.
2. Como o winit disponibiliza o identificador da janela.
3. Se o egui está utilizando tema escuro apenas na área cliente.
4. Se a decoração nativa está herdando o tema claro do Windows.
5. Quais mecanismos oficiais estão disponíveis para solicitar decoração escura.

Considerar inicialmente a API oficial do Windows Desktop Window Manager.

Investigar:

`DwmSetWindowAttribute`

E o atributo:

`DWMWA_USE_IMMERSIVE_DARK_MODE`

Confirmar a disponibilidade e o comportamento da API na versão real do Windows.

Não presumir suporte uniforme entre versões do Windows 10 e Windows 11.

## 4.4. Implementação

Preferir manter a barra de título nativa.

Solicitar ao Windows uma decoração escura compatível com o tema do aplicativo.

Utilizar APIs documentadas.

Evitar:

* Manipulação improvisada da janela.
* Substituição desnecessária das bordas nativas.
* Quebra dos controles do Windows.
* Interferência em Aero Snap.
* Interferência no redimensionamento.
* Bordas invisíveis.
* Duplicação dos botões minimizar/maximizar/fechar.

Se a solução exigir uma pequena integração Win32, isolá-la em um módulo específico e protegido por compilação condicional Windows.

Priorizar APIs Rust seguras quando disponíveis.

Se for necessário utilizar FFI, manter qualquer bloco `unsafe` mínimo, justificado, documentado e revisado.

Não espalhar chamadas Win32 pelo código da GUI.

## 4.5. Cor

A aparência deverá ser:

* Fundo preto ou cinza quase preto.
* Texto claro.
* Ícones dos controles legíveis.
* Boa distinção entre janela ativa e inativa.

O objetivo é compatibilidade visual com a barra superior escura do CAD-CRAFT.

Não utilizar a barra branca atual como resultado satisfatório.

## 4.6. Compatibilidade

Verificar:

* Windows 10.
* Windows 11.
* Janela maximizada.
* Janela restaurada.
* Janela minimizada e reaberta.
* Alternância de foco.
* Ambiente de alto contraste, quando aplicável.

Se o Windows não permitir a aparência solicitada em determinada configuração, documentar a limitação e oferecer a melhor alternativa tecnicamente segura.

Não declarar compatibilidade universal sem teste.

## 4.7. Alternativa

Somente avaliar barra de título personalizada se a solução nativa demonstrar ser insuficiente e existir justificativa técnica consistente.

Uma solução personalizada não poderá comprometer:

* Arraste da janela.
* Maximização por duplo clique.
* Redimensionamento.
* Aero Snap.
* Teclado.
* Acessibilidade.
* Controles nativos.
* Múltiplos monitores.

Não substituir a decoração nativa apenas por conveniência estética.

## 4.8. Testes

Verificar:

1. Tema escuro aplicado.
2. Título visível.
3. Ícone visível.
4. Minimizar.
5. Maximizar.
6. Restaurar.
7. Fechar.
8. Redimensionar.
9. Arrastar janela.
10. Alternar foco.
11. Ausência de console adicional no Release.

Registrar evidências reais da janela nativa.

Capturas apenas do conteúdo egui não são suficientes para comprovar a cor da barra de título do Windows.

---

# 5. CORREÇÃO 4 — RIBBON COM CONTEÚDO INTERMITENTE

## 5.1. Problema identificado

O usuário confirmou que as guias da Ribbon funcionam.

Entretanto, ocorre um comportamento inconsistente:

Às vezes, ao selecionar uma guia, seus grupos e botões aparecem normalmente.

Em outras ocasiões, a guia é selecionada, mas a região de comandos não aparece.

O problema não segue um padrão claro.

O usuário precisa que todas as guias tenham um comportamento estável e previsível.

## 5.2. Investigação obrigatória

Inspecionar detalhadamente:

* Registro das guias.
* Estado da guia selecionada.
* Estado de recolhimento da Ribbon.
* Recolhimento automático.
* Cálculo de altura disponível.
* Ordem de construção dos painéis egui.
* Repaint.
* Persistência do estado da Ribbon.
* Tratamento do duplo clique.
* Tratamento de Ctrl+F1.
* Mudanças de foco.
* Alternância de documentos.
* Abertura e fechamento do Backstage.
* Redimensionamento da janela.
* Alteração de DPI.

Não presumir a causa.

Reproduzir o defeito antes de aplicar a correção.

## 5.3. Hipóteses a investigar

Investigar especialmente se:

1. A Ribbon é recolhida automaticamente quando a altura da janela diminui.
2. A altura está sendo calculada com base no espaço restante do layout.
3. O estado de recolhimento está sendo compartilhado incorretamente entre guias.
4. A seleção de uma guia altera involuntariamente a visibilidade da Ribbon.
5. Um evento de duplo clique é interpretado incorretamente.
6. O Backstage altera o estado da Ribbon.
7. Alguma região egui está consumindo o espaço vertical reservado aos comandos.
8. A interface não está solicitando repaint após alteração de estado.
9. A Ribbon está sendo renderizada, mas recortada pelo layout.
10. As preferências persistidas estão restaurando um estado incoerente.

## 5.4. Comportamento desejado

Quando a Ribbon estiver expandida:

* Clicar em Página Inicial mostra seus comandos.
* Clicar em Inserir mostra seus comandos.
* Clicar em Editar mostra seus comandos.
* Clicar em Visualização mostra seus comandos.
* Clicar em Dispositivos mostra seus comandos.
* Clicar em Conexões mostra seus comandos.
* Clicar em Ferramentas mostra seus comandos.
* Clicar em Pré-planejamento mostra seus comandos.
* Clicar em Dados-mestre mostra seus comandos.
* Clicar em Integrações mostra seus comandos.

O conteúdo da Ribbon deverá permanecer visível até que o usuário decida recolhê-la.

Alternar de guia NÃO deverá esconder automaticamente os comandos.

## 5.5. Recolhimento explícito

Manter:

* Ctrl+F1.
* Duplo clique na guia ativa, caso já suportado.
* Botão de recolher/expandir.

Definir uma única fonte de verdade para o estado expandido/recolhido.

O recolhimento deverá acontecer apenas em circunstâncias previstas e claramente identificáveis.

Evitar recolhimento automático inesperado.

Se o espaço vertical for insuficiente, priorizar uma estratégia responsiva que não provoque desaparecimento silencioso dos grupos.

## 5.6. Comportamento com Ribbon recolhida

Quando o usuário recolher explicitamente a Ribbon:

* As guias continuam visíveis.
* A região de comandos pode ficar oculta.
* Selecionar uma guia deverá ter comportamento documentado e previsível.
* Deve existir uma forma clara de expandir novamente.

Não permitir estados nos quais os comandos desapareçam sem indicação visual de que a Ribbon está recolhida.

## 5.7. Tamanho e proporções

Verificar a altura da Ribbon.

Preservar:

* Títulos das guias.
* Grupos.
* Separadores.
* Botões grandes.
* Botões pequenos.
* Ícones.
* Textos.
* Menus suspensos.
* Títulos inferiores dos grupos.

A interface deverá manter densidade visual profissional.

Não transformar a Ribbon em uma barra excessivamente alta.

Não permitir sobreposição com as abas de documentos.

## 5.8. Responsividade

Testar:

* 1280 × 720.
* 1366 × 768.
* 1600 × 900.
* 1920 × 1080.
* 1920 × 1200.
* 2560 × 1440.

Testar também:

* Maximizada.
* Restaurada.
* Redimensionamento horizontal.
* Redimensionamento vertical.
* DPI 100%.
* DPI 125%.
* DPI 150%.
* DPI 200%.

A redução do espaço horizontal poderá exigir rolagem horizontal entre os grupos.

Não deverá ocultar silenciosamente toda a Ribbon.

## 5.9. Backstage

Este requisito é especialmente importante.

**O usuário aprovou explicitamente o comportamento do menu Arquivo.**

Quando clica em Arquivo, o CAD-CRAFT apresenta uma tela própria com:

* Navegação lateral escura.
* Informações do projeto.
* Propriedades.
* Estatísticas.
* Estrutura.
* Status.
* Botão Voltar.
* Operações de projeto.

O usuário confirmou que essa implementação está muito próxima do comportamento desejado do EPLAN.

PORTANTO:

**PRESERVAR INTEGRALMENTE O BACKSTAGE EXISTENTE.**

Não substituir por menu suspenso.

Não alterar a navegação lateral.

Não modificar desnecessariamente o layout.

Não alterar a organização das opções.

Não prejudicar as operações que já funcionam.

A única integração necessária é garantir que, ao retornar do Backstage:

1. A guia anteriormente selecionada continue selecionada.
2. A Ribbon apresente seus grupos corretamente.
3. O estado expandido/recolhido seja preservado.
4. Os documentos permaneçam abertos.
5. O viewport preserve sua câmera.
6. Os painéis mantenham suas dimensões.
7. Nenhuma seleção seja perdida indevidamente.

## 5.10. Teste de alternância

Criar teste que execute sucessivamente:

Página Inicial → Inserir → Editar → Visualização → Dispositivos → Conexões → Ferramentas → Pré-planejamento → Dados-mestre → Integrações.

Repetir a sequência diversas vezes.

Confirmar que o estado da Ribbon permanece consistente.

Testar também:

* Abrir Arquivo.
* Retornar.
* Alternar guia.
* Recolher Ribbon.
* Expandir.
* Maximizar.
* Restaurar.
* Alterar página.
* Abrir nova aba.
* Fechar aba.

## 5.11. Testes automatizados

Adicionar regressões para:

1. Seleção de guia.
2. Estado expandido.
3. Estado recolhido.
4. Ctrl+F1.
5. Duplo clique.
6. Retorno do Backstage.
7. Redimensionamento.
8. Mudança de documento.
9. Persistência de preferências.
10. Limites mínimos do layout.
11. Ausência de sobreposição.
12. Conteúdo da guia renderizado no espaço disponível.

Não considerar suficiente um teste que apenas confirme a alteração de um enum de guia.

Os testes deverão verificar também o comportamento do layout e a disponibilidade dos comandos.

---

# 6. PRESERVAÇÃO DAS FUNCIONALIDADES EXISTENTES

Nenhuma das quatro correções poderá prejudicar:

* Backstage.
* Ribbon.
* Navegadores.
* Árvore documental.
* Centro de inserção.
* Painel de propriedades.
* Pré-visualização.
* Abas de documentos.
* Painéis redimensionáveis.
* Barra de mensagens.
* Barra de status.
* Zoom.
* Pan.
* Snap.
* Persistência de preferências.
* Importação/exportação de snapshot.
* Operações documentais.
* Integração com o Core.

Preservar especialmente a arquitetura que separa:

* Viewport.
* Folha.
* Câmera.
* Coordenadas documentais.
* Renderização.
* Estado da sessão.

A interface não poderá criar um segundo modelo elétrico.

---

# 7. TESTES E VALIDAÇÃO AUTOMATIZADA

Preservar todos os 179 testes anteriores.

Adicionar testes para as quatro correções.

Executar:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

Executar também:

```powershell
cargo build --release -p cad-gui --bin cad-gui --locked

git diff --check
```

Corrigir eventuais regressões.

Não remover testes.

Não desabilitar verificações.

Não alterar funcionalidades do Core fora do escopo.

Quando disponível, executar uma revisão independente.

A revisão deverá procurar especificamente:

* Erros de transformação na grade.
* Problemas de desempenho com muitos pontos.
* Contraste inadequado.
* Falhas na decoração nativa.
* Incompatibilidades Windows 10/11.
* Estado inconsistente da Ribbon.
* Problemas de layout.
* Regressões no Backstage.

---

# 8. EVIDÊNCIAS VISUAIS OBRIGATÓRIAS

Utilizar o mecanismo de captura da interface real implementado na Etapa 5.

Gerar evidências em:

`.local/etapa-05/`

## Captura A — Grade de pontos

Projeto demonstrativo aberto.

Grade visível em zoom intermediário.

Os pontos deverão estar claramente separados.

## Captura B — Zoom afastado

Mesma página com zoom reduzido.

Viewport ocupando toda a região central.

Pontos com densidade adequada.

## Captura C — Grade desligada

Mesma página sem grade visível.

Snap preservado.

## Captura D — Tela inicial

Nenhuma página aberta.

Botões e textos legíveis.

## Captura E — Barra de título

Captura nativa da janela Windows.

Deverá mostrar a barra de título escura e os controles da janela.

Não utilizar somente screenshot da área cliente do egui para esta verificação.

## Captura F — Ribbon Página Inicial

Guia expandida com grupos e botões visíveis.

## Captura G — Ribbon Inserir

Guia expandida com seus próprios grupos e botões.

## Captura H — Ribbon Conexões

Guia expandida com seus próprios comandos.

## Captura I — Backstage

Menu Arquivo aberto.

Comportamento e aparência preservados.

## Captura J — Retorno do Backstage

Após pressionar Voltar, a Ribbon deverá recuperar corretamente seus grupos.

Não utilizar imagens simuladas.

Não afirmar que uma captura foi realizada se a ferramenta não tiver conseguido capturar a janela real.

---

# 9. DOCUMENTAÇÃO

Atualizar:

`docs/internal/reports/ETAPA_05_INTERFACE_GRAFICA_EGUI.md`

Criar uma seção intitulada:

**Correções após homologação visual — segunda rodada**

Registrar:

1. Os quatro problemas identificados.
2. As causas técnicas confirmadas.
3. Arquivos alterados.
4. Alterações na grade.
5. Estratégia de renderização por pontos.
6. Correções de contraste.
7. Implementação da barra de título escura.
8. Compatibilidade Windows.
9. Correções do gerenciamento da Ribbon.
10. Preservação do Backstage.
11. Testes adicionados.
12. Evidências visuais.
13. Limitações conhecidas.
14. Roteiro de homologação final.

Atualizar a memória persistente somente quando necessário.

Não registrar homologação manual como concluída antes da confirmação do usuário.

---

# 10. CRITÉRIOS DE ENTREGA

A correção estará pronta para homologação quando:

### Grade

* [ ] Não existirem linhas quadriculadas.
* [ ] A grade for composta por pontos.
* [ ] Os pontos estiverem alinhados às coordenadas.
* [ ] Zoom e pan funcionarem.
* [ ] O snap permanecer independente da grade visível.
* [ ] A densidade visual for adequada.

### Contraste

* [ ] Os botões da tela inicial estiverem legíveis.
* [ ] Textos auxiliares apresentarem bom contraste.
* [ ] Hover, foco e estados desabilitados forem consistentes.
* [ ] A tela inicial permanecer centralizada.

### Barra de título

* [ ] A barra de título nativa utilizar aparência escura nas configurações Windows suportadas.
* [ ] O título estiver legível.
* [ ] Os controles da janela funcionarem.
* [ ] Maximização e restauração funcionarem.
* [ ] Redimensionamento funcionar.
* [ ] Nenhum console adicional aparecer no Release.
* [ ] Eventuais limitações por versão do Windows estiverem documentadas.

### Ribbon

* [ ] Todas as guias apresentarem seus grupos corretamente.
* [ ] Alternar guias não esconder os comandos inesperadamente.
* [ ] Recolher e expandir funcionarem.
* [ ] Ctrl+F1 funcionar.
* [ ] O estado da Ribbon permanecer consistente.
* [ ] O layout adaptar-se ao redimensionamento.
* [ ] O Backstage continuar funcionando exatamente como antes.
* [ ] Retornar do Backstage não esconder os grupos.

### Qualidade

* [ ] Todos os testes anteriores passarem.
* [ ] Os novos testes passarem.
* [ ] O build Release for gerado.
* [ ] As capturas de QA estiverem disponíveis.
* [ ] A documentação estiver atualizada.
* [ ] Nenhum commit ou push tiver sido realizado.

---

# 11. ROTEIRO DE HOMOLOGAÇÃO MANUAL

Ao concluir, fornecer instruções objetivas para o usuário testar:

## Teste 1 — Grade

Abrir demonstração.

Confirmar pontos.

Alterar zoom.

Alterar espaçamento.

Ligar/desligar grade.

## Teste 2 — Tela inicial

Fechar as páginas abertas.

Verificar legibilidade dos botões.

Passar o mouse sobre eles.

Verificar os estados visuais.

## Teste 3 — Barra de título

Abrir Release.

Confirmar barra escura.

Maximizar.

Restaurar.

Redimensionar.

## Teste 4 — Ribbon

Alternar repetidamente todas as guias.

Verificar os grupos.

Testar Ctrl+F1.

Recolher e expandir.

## Teste 5 — Backstage

Abrir Arquivo.

Navegar pelas opções existentes.

Retornar.

Confirmar que a Ribbon continua visível.

## Teste 6 — Regressões

Abrir páginas.

Alternar documentos.

Utilizar zoom.

Redimensionar painéis.

Confirmar funcionamento anterior.

---

# 12. RELATÓRIO FINAL DO CODEX

Ao concluir, apresentar:

## CAD-CRAFT — ETAPA 5: CORREÇÕES VISUAIS CONCLUÍDAS

Informar:

1. Estado da implementação.
2. Causa confirmada do problema da grade.
3. Causa confirmada do contraste.
4. Causa confirmada da barra de título clara.
5. Causa confirmada do desaparecimento da Ribbon.
6. Correções implementadas.
7. Arquivos modificados.
8. Quantidade total de testes.
9. Resultado das validações.
10. Resultado do build Release.
11. Evidências visuais geradas.
12. Limitações restantes.
13. Procedimento de homologação manual.
14. Estado do Git.

Confirmar explicitamente:

* Nenhum commit realizado.
* Nenhum push realizado.
* Etapa 6 não iniciada.
* Backstage preservado.
* Core elétrico preservado.

---

# 13. RESTRIÇÕES FINAIS

NÃO executar git commit.

NÃO executar git push.

NÃO iniciar a Etapa 6.

NÃO reimplementar a interface completa.

NÃO modificar o Backstage desnecessariamente.

NÃO substituir o menu Arquivo por menu suspenso.

NÃO alterar a conectividade elétrica.

NÃO criar funcionalidades elétricas fictícias.

NÃO comprometer a arquitetura do viewport.

NÃO apagar projetos pessoais ou backups.

NÃO declarar homologação manual sem autorização.

A prioridade é corrigir precisamente os quatro problemas identificados, preservando tudo o que já funciona.

**Resultado esperado: uma interface CAD-CRAFT com grade de pontos, contraste adequado, barra de título escura e Ribbon estável, mantendo a experiência de navegação e o Backstage já aprovados pelo usuário.**
