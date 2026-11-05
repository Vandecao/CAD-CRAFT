# CAD-CRAFT — ETAPA 5

## INTERFACE GRÁFICA PROFISSIONAL COM RUST + EGUI/EFRAME

### REPRODUÇÃO FUNCIONAL E VISUAL DA EXPERIÊNCIA DO EPLAN ELECTRIC P8

# 0. AUTORIZAÇÃO E DIRETRIZ PRINCIPAL

Implementar integralmente a Etapa 5 do CAD-CRAFT.

**Diretório do projeto:**

`C:\Dev\cad-craft`

**Linguagem:** Rust.

**Framework gráfico:** egui/eframe.

**Sistema operacional principal:** Windows 10 e Windows 11.

**Branch:** main.

**Commit-base esperado:** `5eac964`.

**Linha de base conhecida:** 128 testes aprovados, sendo 113 Rust e 15 PowerShell.

A Etapa 4 foi concluída, registrada manualmente no Git e enviada ao GitHub.

Esta etapa inaugura a interface gráfica desktop do CAD-CRAFT.

O objetivo é desenvolver uma experiência visual e operacional extremamente semelhante à do EPLAN Electric P8, respeitando as limitações técnicas e legais.

A interface deverá reproduzir, sempre que viável:

* Organização das janelas.
* Estrutura da Ribbon.
* Hierarquia visual.
* Distribuição dos comandos.
* Comportamento dos navegadores.
* Painéis acopláveis.
* Área central de edição.
* Sistema de documentos e abas.
* Painéis de propriedades.
* Centro de inserção.
* Área de pré-visualização.
* Menu Arquivo.
* Barra de status.
* Navegação.
* Zoom e pan.
* Comportamento dos controles.
* Atalhos e menus contextuais.
* Feedback visual.

**A fidelidade funcional e a semelhança da experiência de utilização são prioridades desta etapa.**

Não utilizar código proprietário, logotipos, ícones exclusivos, fontes restritas ou outros recursos protegidos do EPLAN.

Criar os elementos visuais próprios do CAD-CRAFT, mantendo organização, proporções, comportamento e ergonomia semelhantes.

Não copiar o programa original nem recorrer a engenharia reversa de componentes proprietários.

---

# 1. RECUPERAÇÃO DO CONTEXTO

Antes de implementar:

1. Ler `AGENTS.md`.
2. Ler `docs/internal/memory/HANDOFF.md`.
3. Ler `docs/internal/memory/CURRENT_STATE.md`.
4. Ler `docs/internal/memory/ARCHITECTURE.md`.
5. Ler `docs/internal/memory/ROADMAP.md`.
6. Ler `docs/internal/memory/DECISIONS.md`.
7. Consultar os ADRs relevantes.
8. Ler `docs/internal/spec/CONTRATOS_DOCUMENTAIS_GUI.md`.
9. Consultar o relatório da Etapa 4.
10. Inspecionar os crates reais do workspace.
11. Inspecionar o estado atual de `cad-gui`.
12. Consultar o Git.

Executar:

```powershell
git log -1 --oneline
git status --short
```

Confirmar a estrutura real do projeto antes de escolher dependências ou modificar arquivos.

Não assumir que funcionalidades de versões anteriores do CAD-CRAFT existam neste repositório.

O estado atual deste desenvolvimento possui um Core elétrico e documental funcional, mas ainda não possui uma interface gráfica egui operacional.

A Etapa 5 deve construir essa interface sobre o código existente.

---

# 2. REFERÊNCIAS VISUAIS OBRIGATÓRIAS

O usuário forneceu o arquivo:

`referencias visuais.zip`

Seu conteúdo foi organizado para consulta em:

`C:\Dev\cad-craft\.local\references\eplan\`

Caso o material esteja disponível nesse diretório, examine efetivamente os arquivos antes de definir o layout.

Não presuma que o arquivo foi copiado ou extraído: verifique sua existência.

## 2.1. Imagens fornecidas

### Imagem 00.png

Referência para o menu Arquivo e a visualização de informações do projeto.

Observar:

* Navegação lateral preta.
* Seleção azul.
* Botão de retorno.
* Títulos grandes.
* Subcategorias.
* Formulários.
* Tabelas de propriedades.
* Rolagem vertical.
* Separação entre navegação e conteúdo.

### Imagem 001.PNG

Referência ampliada da Ribbon Página Inicial.

Observar:

* Distribuição dos comandos.
* Botões grandes e pequenos.
* Ícones sobre textos.
* Agrupamento.
* Separadores.
* Menus suspensos.
* Títulos inferiores dos grupos.

### Imagem 01.png

Principal referência da interface completa.

Observar cuidadosamente:

* Barra de título.
* Barra de acesso rápido.
* Ribbon.
* Navegador de páginas.
* Árvore hierárquica.
* Abas internas do navegador.
* Área de pré-visualização inferior esquerda.
* Editor central.
* Abas de documentos.
* Centro de inserção.
* Painel de propriedades.
* Barra de status.
* Divisores redimensionáveis.
* Espaçamento e proporções entre regiões.

Esta deverá ser uma das principais referências para a primeira versão gráfica.

### Imagens 02.png até 10.PNG

Referências das guias da Ribbon:

* Inserir.
* Editar.
* Visualização.
* Dispositivos.
* Conexões.
* Ferramentas.
* Pré-planejamento.
* Dados-mestre.
* EPLAN Cloud.

Analisar grupos, comandos, tamanhos dos ícones, alinhamentos, separadores e menus.

### Imagem 11.PNG

Referência de um documento integrado ao ambiente central do EPLAN.

Observar a utilização de abas de documentos, painel central e áreas auxiliares.

### Imagens 12.PNG e 14.PNG

Referências detalhadas do navegador de páginas.

Observar:

* Diferentes tipos de páginas.
* Ícones específicos por categoria.
* Documentos organizados hierarquicamente.
* Nós expansíveis.
* Folhas de título.
* Índices.
* Esquemas.
* Listas de materiais.
* Diagramas de bornes.
* Diagramas de cabos.
* Identificação funcional e de localização.

### Imagens 15.PNG e 16.PNG

Referências detalhadas do ambiente de trabalho.

Observar:

* Distribuição horizontal dos painéis.
* Documentos tabulados.
* Centro de inserção.
* Propriedades.
* Elementos selecionados.
* Painéis auxiliares.
* Redimensionamento.

### Imagens 17.png até 26.png

Referências da barra inferior de controles.

Observar cada ícone, tooltip e comportamento associado:

* Seleção de grade.
* Modo de construção.
* Fixação de objetos.
* Snap.
* Ativação/desativação da lógica.
* Visibilidade da grade.
* Escala.
* Janela de detalhes.
* Detalhe da página inteira.

### Imagem 27.PNG

Referência importante para a organização de painéis.

A imagem mostra a área de um navegador ocupando grande parte da interface, com os demais painéis reorganizados.

Implementar uma arquitetura que permita redimensionamento significativo e reorganização das regiões.

## 2.2. Vídeos fornecidos

### Vídeo 1

`2026-10-27 14-16-42.mp4`

Duração aproximada: 8 minutos e 52 segundos.

Analisar os momentos de:

* Navegação entre documentos.
* Mudança de guias.
* Abertura do menu Arquivo.
* Exibição de informações do projeto.
* Navegação em páginas.
* Utilização dos painéis auxiliares.
* Mudança de contexto.
* Redimensionamento das regiões.
* Zoom.
* Retorno ao ambiente de edição.

### Vídeo 2

`2026-10-27 14-29-16.mp4`

Duração aproximada: 1 minuto e 27 segundos.

Este vídeo é especialmente importante para compreender:

* Zoom in.
* Zoom out.
* Reposicionamento do documento.
* Visualização da folha em diferentes escalas.
* Área de trabalho independente das dimensões da folha.
* Permanência dos painéis laterais durante a navegação.

**Não reproduzir o erro de reduzir ou recortar a área interativa do editor conforme a folha diminui.**

Se a análise direta dos vídeos não estiver disponível no ambiente do agente, utilizar quadros extraídos com FFmpeg ou ferramenta equivalente, quando instalada.

Não declarar que os vídeos foram analisados se não tiver sido possível acessá-los.

---

# 3. OBJETIVO ARQUITETURAL DA ETAPA 5

Construir a infraestrutura completa da aplicação desktop, sem implementar prematuramente todas as funcionalidades elétricas futuras.

A interface deve ser um ambiente de trabalho real.

Não criar somente uma imagem estática ou protótipo visual.

A arquitetura deverá permitir implementar posteriormente:

* Editor multifilar.
* Biblioteca IEC.
* Inserção de símbolos.
* Desenho de conexões.
* Macros.
* Relatórios.
* ERC.
* Gerenciamento de dispositivos.
* Bornes.
* Cabos.
* Conectores.
* Engenharia documental.
* Layout de painel.
* Funcionalidades de inteligência artificial.

Nesta etapa, criar a base gráfica e a integração com os recursos que já existem no Core.

Não desenvolver um segundo modelo elétrico dentro da interface.

Não duplicar entidades do Core.

---

# 4. ESTRUTURA GERAL DA JANELA

A janela principal deverá seguir aproximadamente esta organização:

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ CAD-CRAFT                      Projeto atual              Controles janela │
├────────────────────────────────────────────────────────────────────────────┤
│ Acesso rápido       Abas da Ribbon         Procurar comando                │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│                    RIBBON CONTEXTUAL                                       │
│       Grupos de comandos / Ícones / Menus / Separadores                    │
│                                                                            │
├──────────────┬─────────────────────────────┬──────────────┬────────────────┤
│ NAVEGADORES  │ ABAS DE DOCUMENTOS         │ CENTRO DE    │ PROPRIEDADES   │
│              ├─────────────────────────────┤ INSERÇÃO     │                │
│ Páginas      │                             │              │ Busca          │
│ Dispositivos │                             │ Favoritos    │                │
│ Estruturas   │       VIEWPORT CAD          │ Símbolos     │ Propriedade    │
│              │                             │ Dispositivos │ Valor          │
│ Hierarquia   │                             │ Macros       │                │
│              │                             │              │                │
│──────────────│                             │              │                │
│ PRÉVIA       │                             │              │                │
│              │                             │              │                │
├──────────────┴─────────────────────────────┴──────────────┴────────────────┤
│ Mensagens / Informações / Avisos / Erros                                   │
├────────────────────────────────────────────────────────────────────────────┤
│ Coordenadas  |  Modo  |  Snap  |  Grade  |  Escala  |  Zoom  |  Estado      │
└────────────────────────────────────────────────────────────────────────────┘
```

Essa representação é conceitual.

Adaptar as proporções às referências reais.

---

# 5. IDENTIDADE VISUAL

## 5.1. Tema

Utilizar tema escuro profissional.

Referências de cores aproximadas:

* Barra de título: preto ou quase preto.
* Ribbon: cinza-escuro.
* Painéis laterais: cinza-escuro.
* Separadores: cinza discreto.
* Área gráfica: fundo de documento branco.
* Texto principal: branco ou cinza-claro.
* Texto secundário: cinza.
* Seleção: azul.
* Destaques: azul.
* Campos editáveis: cinza intermediário.
* Elementos desabilitados: cinza de baixo contraste.
* Avisos: amarelo ou laranja.
* Erros: vermelho.

Utilizar valores de cor centralizados em um sistema de estilos.

Não espalhar constantes visuais por diferentes arquivos.

## 5.2. Tipografia

Criar uma hierarquia consistente:

* Título da janela.
* Guias da Ribbon.
* Nomes de comandos.
* Títulos de grupos.
* Cabeçalhos dos painéis.
* Itens de árvores.
* Propriedades.
* Status.
* Tooltips.

Evitar fontes excessivamente grandes.

A interface deverá apresentar alta densidade de informação, como um software CAD profissional.

## 5.3. Ícones

Utilizar ícones vetoriais próprios ou provenientes de bibliotecas com licenciamento compatível.

Priorizar:

* Traços finos.
* Boa legibilidade.
* Consistência dimensional.
* Estado normal.
* Estado selecionado.
* Estado desabilitado.
* Estado hover.

Não utilizar emojis como substitutos de ícones profissionais.

Não copiar os ícones proprietários do EPLAN.

## 5.4. Dimensionamento

O layout deve funcionar adequadamente em:

* 1280 × 720.
* 1366 × 768.
* 1600 × 900.
* 1920 × 1080.
* 1920 × 1200.
* Resoluções superiores.

Testar escalas DPI de 100%, 125%, 150% e, quando possível, 200%.

A interface não poderá depender de posições absolutas fixas.

---

# 6. BARRA SUPERIOR E ACESSO RÁPIDO

Implementar uma barra superior semelhante à referência.

## 6.1. Informações exibidas

Mostrar:

* Ícone próprio do CAD-CRAFT.
* Nome do aplicativo.
* Nome do projeto aberto.
* Página ativa, quando houver.
* Indicador de modificações não salvas, quando aplicável.

Exemplo:

`CAD-CRAFT — Projeto de automação — Página 12`

Quando não houver projeto:

`CAD-CRAFT — Nenhum projeto aberto`

## 6.2. Controles da janela

Preservar:

* Minimizar.
* Maximizar/restaurar.
* Fechar.

Respeitar comportamento nativo do Windows.

## 6.3. Barra de acesso rápido

Preparar botões compactos para:

* Novo.
* Abrir.
* Salvar.
* Desfazer.
* Refazer.
* Repetir operação.
* Acesso a configurações.

Habilitar apenas operações realmente implementadas.

## 6.4. Comportamento

* Tooltip ao passar o mouse.
* Realce no hover.
* Estado desabilitado quando necessário.
* Atalhos consistentes.
* Resposta visual ao clique.

Não criar botões sem identificação acessível.

---

# 7. RIBBON PROFISSIONAL

Esta é uma das prioridades visuais da Etapa 5.

Não utilizar uma sequência simples de botões horizontais sem organização.

Implementar uma Ribbon com:

* Guias.
* Grupos.
* Botões grandes.
* Botões pequenos.
* Ícones.
* Rótulos.
* Separadores.
* Menus suspensos.
* Galerias preparadas para expansão.
* Tooltips.
* Estados de habilitação.
* Comandos contextuais.

## 7.1. Guias principais

Criar as seguintes guias:

1. Arquivo.
2. Página Inicial.
3. Inserir.
4. Editar.
5. Visualização.
6. Dispositivos.
7. Conexões.
8. Ferramentas.
9. Pré-planejamento.
10. Dados-mestre.
11. Integrações.

A última guia será própria do CAD-CRAFT.

Não utilizar a marca EPLAN Cloud como se o CAD-CRAFT possuísse acesso a esse serviço.

## 7.2. Aparência

Cada guia deverá:

* Possuir texto compacto.
* Utilizar destaque azul quando selecionada.
* Atualizar o conteúdo da Ribbon.
* Manter altura consistente.
* Preservar o alinhamento dos grupos.
* Utilizar divisores discretos.

A Ribbon deverá ter comportamento semelhante ao apresentado nas imagens 001 e 02–10.

## 7.3. Arquitetura dos comandos

Criar um registro central de comandos.

Exemplo conceitual:

```rust
enum CommandId {
    NewProject,
    OpenProject,
    SaveProject,
    NewPage,
    OpenPage,
    RenamePage,
    DeletePage,
    ShowPageNavigator,
    ShowProperties,
    FitPage,
    FitContent,
    ToggleGrid,
    ToggleSnap,
}
```

A enumeração apresentada é ilustrativa.

Adaptar à arquitetura real.

Cada comando deverá possuir metadados:

* Identificador.
* Nome.
* Descrição.
* Ícone.
* Categoria.
* Grupo.
* Atalho, quando houver.
* Estado de habilitação.
* Ação.
* Requisitos de contexto.

Evitar lógica de negócio duplicada dentro de cada botão.

O despacho deverá permitir evolução futura sem reestruturar toda a Ribbon.

---

# 8. GUIA PÁGINA INICIAL

Reproduzir a estrutura observada em `001.PNG`.

## Grupo: Área de transferência

Preparar:

* Colar.
* Recortar.
* Copiar.
* Excluir.
* Copiar formato.
* Atribuir formato.

## Grupo: Página

* Navegador.
* Nova página.
* Numerar.
* Macro de página.

A criação de páginas deverá utilizar o Core existente.

## Grupo: Espaço do layout 3D

* Navegador.
* Novo.
* Medir.

Os comandos dependentes de infraestrutura 3D deverão permanecer indisponíveis nesta etapa.

## Grupo: Visualização gráfica

* Abrir.
* Alternar documento ativo.

## Grupo: Texto

* Inserir.
* Texto de função do caminho.
* Mover.

## Grupo: Editar

* Propriedades.
* Visão geral de propriedades.
* Tabular.

## Grupo: Procurar

* Procurar.
* Mostrar resultados.
* Sincronizar seleção.

Os recursos de busca sobre páginas e documentos existentes podem ser implementados nesta etapa.

Não simular pesquisa elétrica avançada ainda indisponível.

---

# 9. GUIA INSERIR

Utilizar `02.png` como referência principal.

Organizar os grupos:

## Macros

* Navegador.
* Inserir macro.

## Símbolos

* Galeria de símbolos.
* Biblioteca de símbolos.
* Variantes gráficas.

## Dispositivos

* Caixa de dispositivos.
* Ponto de conexão do dispositivo.
* Definição da peça.
* Caixa de CLP.
* Pontos de conexão de CLP.
* Caixa de estrutura.
* Funções conectadas.

## Cabos / Conexões

* Cabo.
* Blindagem.
* Conexão.

## Bornes

* Borne.
* Borne distribuído.
* Definição da régua de bornes.

## Conectores

* Contato do conector.
* Definição do conector.

## Gráfico

* Texto.
* Galeria de gráficos.

## Dimensionamento

* Dimensionamento linear.

## Outros grupos

* Externo.
* Topologia.
* Visualização.
* Layout do painel 2D.

**Importante:** nesta etapa, preparar visualmente esses comandos.

Não implementar símbolos elétricos, bibliotecas ou novas entidades que não estejam previstas no Core atual.

Os comandos ainda não disponíveis deverão permanecer desabilitados, com tooltip explicando sua futura finalidade.

---

# 10. GUIA EDITAR

Utilizar `03.png` como referência.

Criar os grupos:

## Área de transferência

* Colar.
* Recortar.
* Copiar.

## Gráfico

* Mover.
* Rotacionar.
* Espelhar.
* Esticar.
* Escalar.
* Aparar.
* Arredondar.
* Chanfrar.
* Duplicar.

## Agrupar

* Agrupar.
* Desfazer agrupamento.

## Bloco

* Formar.
* Desfazer.
* Converter.

## Ordenar

* Alinhar à grade.
* Alinhar horizontalmente.
* Distribuir horizontalmente.
* Primeiro plano.
* Plano de fundo.

## Proteção

* Configurar.
* Bloquear.
* Desbloquear.

## Opções

* Edição direta.
* Inserção relativa de coordenadas.
* Inserção de coordenadas.
* Intervalo.
* Janela de entrada de dados.
* Conexão inteligente.

Não implementar prematuramente o editor gráfico avançado.

Manter a estrutura preparada para futuras ferramentas.

---

# 11. GUIA VISUALIZAÇÃO

Utilizar `04.png` como referência.

Organizar:

## Navegadores

* Abrir.
* Mostrar/ocultar navegadores.

## Geral

* Acompanhamento de rede.
* Acompanhamento do potencial.
* Acompanhamento do sinal.
* Pontos de inserção.
* Elementos ocultos.
* Caminhos.
* Marcadores de revisão.
* Alvos externos.
* Textos vazios.
* Conexões de cabos.
* Deslocar conteúdo.
* Designações de pontos de conexão.

Habilitar inicialmente os controles que possuírem implementação real.

Adicionar também comandos úteis ao viewport:

* Enquadrar página.
* Enquadrar conteúdo.
* Zoom in.
* Zoom out.
* Zoom 100%.
* Mostrar grade.
* Mostrar propriedades.
* Mostrar pré-visualização.
* Mostrar centro de inserção.

---

# 12. GUIA DISPOSITIVOS

Utilizar `05.png` como referência.

Criar os grupos:

## Dispositivos

* Navegador.
* Lista de materiais.
* Numerar.
* Extras.

## CLP

* Navegador.
* Endereçar.
* Lista de atribuição.
* Endereços iniciais.
* Pontos de conexão.
* Mover.

## Bornes

* Navegador.
* Numerar.
* Régua de bornes.
* Extras.

## Conectores

* Navegador.
* Numerar.
* Conectores.

## Layout do painel 2D

* Navegador.

## Opções do projeto

* Navegador.
* Objetos placeholder.
* Gerar seção.

A navegação básica sobre entidades já disponíveis no Core poderá ser implementada.

Não criar relatórios fictícios.

---

# 13. GUIA CONEXÕES

Utilizar `06.png` como referência.

Criar grupos:

## Cabos

* Navegador.
* Numerar.
* Editar.
* Atribuir tudo novamente.

## Conexões

* Navegador.
* Potenciais.
* Atualizar.
* Fonte e alvo.
* Designar.
* Posicionar pontos de definição.
* Alinhar e formatar.

## Pontos de interrupção

* Navegador.

## Topologia

* Navegador.
* Rotear.
* Capacidade de preenchimento.
* Gerar.

## Chicote

* Navegador.

Os navegadores poderão exibir informações existentes no Core, se houver dados disponíveis.

Não implementar o editor multifilar nesta etapa.

---

# 14. GUIA FERRAMENTAS

Utilizar `07.png` como referência.

Criar grupos:

## Verificação

* Mensagens.
* Verificar.

## Relatórios

* Gerar.
* Atualizar.
* Avaliar projeto.
* Visualizações do modelo.
* Editar automaticamente.

## Gerenciamento

* Identificadores de estrutura.
* Distribuições de propriedades.
* Layers.
* Propriedades.

## Tradução

* Traduzir.
* Dicionário.
* Exportar termos faltantes.
* Entrada de teste.
* Importar textos.
* Remover tradução.

## Normas

* Substituir.

## Sincronização

* Funções.
* Conexões.

## Comentários

* Navegador.
* Importar.

Não declarar conformidade normativa ou resultados ERC ainda não implementados.

---

# 15. GUIAS PRÉ-PLANEJAMENTO, DADOS-MESTRE E INTEGRAÇÕES

Utilizar `08.png`, `09.png` e `10.PNG`.

## Pré-planejamento

Preparar grupos:

* Geral.
* Editar.
* Tubulação.
* Diagrama P&I.
* Macro de pré-planejamento.

## Dados-mestre

Preparar:

* Peças.
* Macros.
* Sincronização.
* Molduras.
* Formulários.
* Símbolos.
* Pontos de conexão.
* Gerenciamento de tags.

## Integrações

Criar guia própria do CAD-CRAFT, preparada para integrações futuras.

Não apresentar serviços externos como conectados sem implementação real.

Manter clara a distinção entre recursos operacionais e funcionalidades planejadas.

---

# 16. MENU ARQUIVO — BACKSTAGE

Utilizar `00.png` como referência principal.

Não implementar Arquivo como um menu suspenso convencional.

Criar uma visualização que substitui temporariamente a região principal por uma tela de gerenciamento.

## 16.1. Navegação lateral

Utilizar fundo quase preto.

Incluir:

* Informações.
* Novo.
* Abrir.
* Fazer backup.
* Copiar projeto.
* Excluir projeto.
* Fechar projeto.
* Gerenciamento de projetos.
* Controle de revisão.
* Imprimir.
* Importar.
* Exportar.
* Extras.
* Configurações.
* Ajuda.
* Sair.

## 16.2. Seleção

A opção selecionada deverá possuir destaque azul.

## 16.3. Retorno

Adicionar botão de retorno claramente visível.

Ao retornar, restaurar a visualização do documento anterior sem perder estado da sessão.

## 16.4. Informações

Mostrar:

* Nome do projeto.
* Descrição.
* Identificação.
* Metadados disponíveis.
* Documentos.
* Quantidade de páginas.
* Estruturas.
* Estatísticas básicas.
* Estado da sessão.

Utilizar os dados efetivamente disponíveis no Core.

Não inventar informações técnicas.

## 16.5. Propriedades

Exibir tabela:

| Propriedade   | Valor           |
| ------------- | --------------- |
| Nome          | Valor existente |
| Descrição     | Valor existente |
| Identificação | Valor existente |
| Tipo          | Valor existente |

Criar edição somente para propriedades suportadas pelas APIs existentes.

## 16.6. Estatísticas

Mostrar números reais da sessão:

* Quantidade de documentos.
* Quantidade de páginas.
* Quantidade de representações.
* Quantidade de componentes, quando disponível.
* Quantidade de dispositivos, quando disponível.

## 16.7. Operações de arquivo

Não fingir que existe persistência física definitiva caso ela ainda não tenha sido implementada.

Caso seja viável reutilizar com segurança a serialização estrutural existente, oferecer operações explicitamente identificadas como importação/exportação de snapshot técnico.

Não anunciar suporte `.cadcraft`, backup versionado ou formatos externos ainda inexistentes.

---

# 17. NAVEGADOR DE PÁGINAS

Esta é uma das regiões mais importantes da interface.

Utilizar como referência:

* `01.png`.
* `12.PNG`.
* `14.PNG`.
* `27.PNG`.

## 17.1. Localização

O navegador deverá ocupar inicialmente a região esquerda.

## 17.2. Cabeçalho

Exibir:

`Páginas — Nome do projeto`

Incluir:

* Menu do painel.
* Fixar/desafixar.
* Minimizar/recolher.
* Fechar.
* Opções de visualização.

## 17.3. Abas

Preparar diferentes navegadores.

Exemplos:

* Páginas.
* Dispositivos.
* Estruturas.
* Conexões.
* Cabos.
* Bornes.

Na Etapa 5, os navegadores deverão reutilizar as entidades disponíveis no Core.

## 17.4. Árvore

Implementar árvore hierárquica com:

* Nós expansíveis.
* Nós recolhíveis.
* Ícones por tipo.
* Indentação.
* Seleção.
* Hover.
* Rolagem vertical.
* Rolagem horizontal quando necessária.
* Navegação por teclado.

Utilizar as estruturas funcionais e de localização da Etapa 4.

## 17.5. Hierarquia documental

Representar:

```text
Projeto
├── Documento
│   ├── Estrutura funcional
│   │   ├── Estrutura de localização
│   │   │   ├── Página 1
│   │   │   ├── Página 2
│   │   │   └── Página 3
│   └── Páginas sem classificação
└── Outro documento
```

Não utilizar a estrutura visual como substituta das identidades do Core.

## 17.6. Tipos de páginas

Diferenciar os tipos existentes no Core por ícones próprios.

Não utilizar o mesmo ícone para todos os tipos documentais.

## 17.7. Seleção e abertura

* Clique simples seleciona.
* Duplo clique abre a página.
* Enter abre a página selecionada.
* Setas navegam pela árvore.
* Clique na seta expande ou recolhe.

A página selecionada e a página aberta deverão ser estados distintos.

## 17.8. Menu contextual

Implementar:

* Abrir.
* Nova página.
* Renomear.
* Alterar identificação.
* Propriedades.
* Reordenar.
* Excluir, quando permitido.

Os comandos devem utilizar APIs reais do Core.

## 17.9. Filtros

Adicionar:

* Campo de filtro.
* Filtro por tipo.
* Filtro por documento.
* Filtro por estrutura.
* Limpar filtro.

Não excluir dados ao filtrar.

## 17.10. Modos Árvore e Lista

Como nas imagens fornecidas, oferecer:

* Árvore.
* Lista.

A lista deverá apresentar os mesmos registros em formato tabular ou linear.

## 17.11. Scroll

O navegador deverá possuir rolagem independente do editor central.

A roda do mouse sobre o navegador não poderá aplicar zoom no desenho.

---

# 18. PAINEL DE PRÉ-VISUALIZAÇÃO INFERIOR ESQUERDO

As referências mostram uma área de visualização gráfica abaixo do navegador.

Implementar um painel independente com:

* Título.
* Botões de fixação.
* Recolher.
* Fechar.
* Área de pré-visualização.

## 18.1. Comportamento

Ao selecionar uma página, apresentar uma miniatura correspondente ao conteúdo documental disponível.

Se a página estiver vazia, apresentar uma folha vazia.

Não utilizar uma imagem estática do EPLAN para simular conteúdo.

## 18.2. Miniaturas

Utilizar:

* Representação simplificada.
* Proporções corretas.
* Margens.
* Identificação da página.
* Destaque para página selecionada.

As miniaturas deverão ser derivadas dos dados do projeto.

## 18.3. Organização

Permitir:

* Lista.
* Grade de miniaturas.
* Rolagem.
* Seleção.

## 18.4. Redimensionamento

O usuário deverá poder aumentar ou diminuir a altura desse painel.

O redimensionamento não poderá modificar o projeto.

---

# 19. ABAS DE DOCUMENTOS

O editor central deverá possuir um sistema de documentos tabulados.

## 19.1. Aparência

Cada aba deverá apresentar:

* Nome da página.
* Identificação resumida.
* Indicador de documento ativo.
* Botão de fechamento.

## 19.2. Comportamento

Permitir:

* Abrir várias páginas.
* Alternar entre páginas.
* Fechar uma aba.
* Reabrir pelo navegador.
* Restaurar a seleção da página.
* Manter o zoom por página, quando apropriado.
* Manter o pan por página.
* Atualizar o inspetor conforme a página ativa.

## 19.3. Fechamento

Fechar uma aba não significa excluir a página do projeto.

Manter essa distinção rigorosamente.

## 19.4. Estado

Usar `PageId`, não o nome visível, para identificar documentos abertos.

Duas páginas poderão possuir nomes semelhantes sem causar conflito interno.

---

# 20. VIEWPORT CAD — ARQUITETURA PRIORITÁRIA

Esta seção possui prioridade máxima.

O comportamento deverá reproduzir a lógica de navegação observada no EPLAN.

## 20.1. Conceitos separados

Implementar separação explícita entre:

1. Dimensões da janela.
2. Dimensões do viewport.
3. Dimensões da folha.
4. Coordenadas do documento.
5. Coordenadas de tela.
6. Zoom.
7. Pan.
8. Grade.
9. Representações.
10. Seleção futura.

A folha é um objeto documental dentro do viewport.

A folha NÃO é o viewport.

## 20.2. Comportamento fundamental

Quando o usuário aplicar zoom out:

* A folha deverá diminuir visualmente.
* O conteúdo deverá diminuir.
* A grade deverá acompanhar a transformação de câmera.
* A região interativa do editor deverá continuar ocupando todo o espaço central disponível.
* O usuário continuará podendo movimentar a câmera.
* O espaço em torno da folha continuará acessível.

Nunca reduzir o tamanho do widget do editor conforme a folha diminui.

Nunca definir o retângulo interativo do viewport a partir dos limites da folha.

## 20.3. Coordenadas

Preservar:

* Milímetros.
* Origem inferior esquerda.
* X positivo para direita.
* Y positivo para cima.
* Rotação positiva anti-horária.

A inversão vertical deverá acontecer somente na transformação de visualização.

Não inverter coordenadas persistentes.

## 20.4. Transformação

Criar uma transformação de câmera consistente:

* World-to-screen.
* Screen-to-world.
* Pan.
* Zoom.
* Ajuste ao viewport.

Utilizar a mesma matemática na renderização e no hit-testing.

## 20.5. Zoom

Implementar:

* Zoom pela roda do mouse.
* Zoom centrado na posição do cursor.
* Zoom in.
* Zoom out.
* Enquadrar página.
* Enquadrar conteúdo.
* Zoom 100%.
* Restaurar visualização anterior, quando viável.

O ponto do documento sob o cursor deverá permanecer aproximadamente na mesma posição da tela durante o zoom.

Definir limites numericamente seguros, sem alegar zoom matematicamente infinito.

Os limites deverão permitir afastamento suficiente para visualizar folhas pequenas e aproximação suficiente para examinar detalhes.

## 20.6. Pan

Implementar:

* Arraste com botão central do mouse.
* Feedback visual durante o arraste.
* Cursor de mão durante pan.
* Movimento contínuo.
* Pan independente das dimensões documentais.

O pan não poderá alterar coordenadas dos objetos.

## 20.7. Página

Desenhar:

* Fundo branco.
* Limites da folha.
* Grade.
* Margens.
* Borda documental.
* Indicação de orientação.

A moldura técnica completa ficará para uma etapa posterior, se ainda não existir suporte no Core.

## 20.8. Área externa

Permitir visualizar e navegar além dos limites da folha.

Respeitar que representações podem possuir coordenadas fora da folha, conforme o contrato documental da Etapa 4.

## 20.9. Grid

A grade deverá existir em coordenadas do documento.

Ela deverá:

* Acompanhar o zoom.
* Acompanhar o pan.
* Manter espaçamento real.
* Preservar alinhamento.
* Não se deslocar independentemente dos objetos.

Em zoom muito afastado, reduzir a densidade visual da grade ou ocultar linhas/pontos excessivamente próximos.

Evitar sobrecarga gráfica.

## 20.10. Regra obrigatória

**Não repetir o problema de fazer a área de trabalho diminuir junto com o desenho.**

Esse comportamento já foi identificado como inadequado pelo usuário.

Criar testes específicos para impedir sua regressão.

---

# 21. BARRA INFERIOR DE CONTROLES

Utilizar as imagens `17.png` até `26.png`.

Implementar uma barra compacta na parte inferior do editor.

## 21.1. Coordenadas

Exibir:

`X: 123,45 mm | Y: 67,89 mm`

Atualizar conforme a posição do cursor dentro do viewport.

## 21.2. Modo de construção

Adicionar ícone e estado visual.

Se ainda não houver ferramentas de construção avançadas, não fingir comportamento inexistente.

Preparar o estado da ferramenta para utilização futura.

## 21.3. Fixar objeto

Preparar comando para bloquear movimentação de objetos.

Habilitar quando houver seleção compatível e operação real.

## 21.4. Lógica

Preparar controle de visualização lógica.

Não modificar redes elétricas ao alternar camadas visuais.

## 21.5. Grade

Implementar seleção configurável.

Incluir:

* 1,0 mm.
* 2,0 mm.
* 2,5 mm.
* 4,0 mm.
* 8,0 mm.
* 16,0 mm.

A lista deverá ser extensível.

## 21.6. Exibir grade

Permitir ligar e desligar visualmente a grade.

## 21.7. Snap

Permitir habilitar e desabilitar ajuste à grade.

O snap deverá ser independente da visibilidade da grade.

## 21.8. Escala

Exibir a escala ou o estado de visualização de maneira clara.

Não confundir escala documental 1:1 com percentual de zoom na tela.

## 21.9. Janela de detalhes

Preparar ferramenta de zoom por retângulo ou visualização ampliada.

Se implementada nesta etapa, deverá funcionar realmente.

## 21.10. Página inteira

Adicionar botão para enquadrar a página completa.

## 21.11. Tooltips

Todos os ícones deverão possuir descrição.

Exemplos:

* Mostrar grade.
* Ocultar grade.
* Selecionar espaçamento.
* Ajustar página.
* Enquadrar conteúdo.
* Zoom 100%.
* Ativar snap.

---

# 22. CENTRO DE INSERÇÃO

Utilizar como referência principal o painel visível em `01.png`, `15.PNG` e `16.PNG`.

## 22.1. Localização

Posicionar à direita do viewport e à esquerda do painel de propriedades.

## 22.2. Cabeçalho

Exibir:

`Centro de inserção`

Adicionar:

* Menu do painel.
* Fixar.
* Recolher.
* Fechar.

## 22.3. Categorias

Criar navegação para:

* Últimos utilizados.
* Favoritos.
* Tags.
* Símbolos.
* Dispositivos.
* Macros.
* Catálogos.

Não utilizar serviços proprietários do EPLAN como se fossem funcionalidades existentes.

## 22.4. Busca

Implementar campo de pesquisa funcional sobre registros efetivamente disponíveis.

## 22.5. Navegação

Criar:

* Voltar.
* Avançar, quando aplicável.
* Página inicial do centro de inserção.
* Histórico de categorias.

## 22.6. Conteúdo

A arquitetura deverá permitir adicionar futuramente:

* Miniaturas.
* Listas.
* Filtros.
* Categorias.
* Pré-visualizações.
* Favoritos.

Não inventar bibliotecas completas nesta etapa.

## 22.7. Estados vazios

Quando não houver símbolos ou dispositivos disponíveis, mostrar uma mensagem clara.

Não apresentar uma biblioteca fictícia com comandos que não funcionam.

---

# 23. PAINEL DE PROPRIEDADES

Reproduzir funcionalmente o painel apresentado nas referências.

## 23.1. Localização

Painel da extrema direita.

## 23.2. Cabeçalho

Exibir:

`Visão geral de propriedades`

## 23.3. Ferramentas

Adicionar:

* Menu do painel.
* Fixar.
* Fechar.
* Modos de visualização.
* Categoria ou perfil.
* Busca.

## 23.4. Tabela

Utilizar duas colunas:

`Propriedade | Valor`

Suportar:

* Categorias expansíveis.
* Linhas.
* Valores.
* Rolagem.
* Seleção.
* Campos editáveis conforme o Core.

## 23.5. Contexto

O painel deverá reagir ao contexto:

* Nenhum projeto.
* Projeto selecionado.
* Documento selecionado.
* Página selecionada.
* Representação selecionada.
* Entidade elétrica associada, quando disponível.

## 23.6. Edição

Propriedades suportadas pelo Core poderão ser modificadas por meio de operações validadas.

Não modificar diretamente estruturas internas ignorando as regras do domínio.

## 23.7. Tipos

Preparar widgets adequados para:

* Texto.
* Número.
* Booleano.
* Enumeração.
* Coordenadas.
* Ângulo.
* Propriedade somente leitura.

## 23.8. Atualização

Alterações deverão aparecer na interface sem necessidade de fechar e reabrir painéis.

---

# 24. PAINÉIS ACOPLÁVEIS E REDIMENSIONAMENTO

Implementar uma infraestrutura de layout profissional.

## 24.1. Requisitos

Permitir redimensionar:

* Navegador esquerdo.
* Pré-visualização.
* Viewport.
* Centro de inserção.
* Propriedades.
* Painel inferior de mensagens.

## 24.2. Divisores

Adicionar separadores arrastáveis.

O cursor deverá mudar ao passar sobre um divisor.

## 24.3. Recolhimento

Permitir ocultar painéis.

A área central deverá ocupar o espaço liberado.

## 24.4. Restauração

Permitir restaurar o layout padrão.

## 24.5. Persistência

Salvar preferências visuais quando viável.

Exemplos:

* Largura dos navegadores.
* Altura da pré-visualização.
* Visibilidade dos painéis.
* Ribbon recolhida.
* Guia ativa.
* Resolução da grade.

Essas configurações não devem ser confundidas com dados elétricos do projeto.

## 24.6. Acoplamento

Preparar arquitetura para rearranjo de painéis.

Priorizar inicialmente redimensionamento e recolhimento robustos.

Se docking flutuante completo exigir infraestrutura adicional, implementá-lo somente se tecnicamente adequado ao escopo.

Documentar claramente o nível de docking efetivamente suportado.

---

# 25. BARRA DE MENSAGENS

Adicionar painel inferior para:

* Informações.
* Avisos.
* Erros.
* Eventos da interface.
* Resultados de operações.

Implementar filtros básicos.

Exemplos:

* Todos.
* Informações.
* Avisos.
* Erros.

Exibir contadores.

Não gerar mensagens falsas de ERC.

---

# 26. BUSCA DE COMANDOS

A referência apresenta um campo:

`Procurar comando`

Implementar uma busca real sobre o registro de comandos.

## Requisitos

* Buscar por nome.
* Buscar por palavras relacionadas.
* Apresentar resultados.
* Identificar guia e grupo.
* Executar comandos disponíveis.
* Informar quando um comando ainda não está implementado.

Não executar comandos planejados.

A busca não poderá travar a interface.

---

# 27. INTERAÇÕES E CURSORES

Definir política consistente de interação.

## Seleção

Cursor padrão sobre elementos comuns.

## Pan

Mão aberta.

Durante o arraste:

Mão fechada.

## Divisores

Cursor horizontal ou vertical de redimensionamento.

## Campos editáveis

Cursor de texto.

## Links e referências

Cursor apropriado para navegação.

## Operações gráficas futuras

Cursor contextual.

Não utilizar permanentemente um cursor em formato de cruz sobre todos os elementos.

---

# 28. ATALHOS

Implementar atalhos apenas para funcionalidades disponíveis.

Priorizar:

* Esc: cancelar ou retornar ao estado neutro.
* Enter: confirmar quando aplicável.
* Ctrl+F1: recolher ou expandir Ribbon, se implementado.
* F11: alternar tela cheia, quando suportado.
* Atalhos de navegação de páginas.
* Zoom pela roda.
* Pan pelo botão central.

Não interceptar Ctrl+C/Ctrl+V quando campos textuais possuem foco.

Não criar atalhos de edição que ainda não tenham comandos reais.

---

# 29. INTEGRAÇÃO COM O CORE DA ETAPA 4

A interface deverá utilizar:

* Project.
* Document.
* Page.
* PageId.
* DocumentStructure.
* Representation.
* RepresentationId.
* RepresentationTarget.
* VisualProperties.
* Transform2D.
* APIs de consultas documentais.
* APIs de integridade.

O Core deve permanecer independente de egui/eframe.

A GUI não deverá implementar outra árvore documental como modelo persistente.

Estados transitórios de apresentação poderão existir na GUI.

Exemplos:

* Página aberta.
* Seleção.
* Scroll.
* Zoom.
* Pan.
* Painel ativo.
* Guia selecionada.
* Preferências visuais.

Esses estados não deverão modificar o modelo elétrico por acidente.

---

# 30. PROJETO DE DEMONSTRAÇÃO DA INTERFACE

Criar uma forma de visualizar a aplicação sem depender de um projeto externo.

O exemplo deverá ser próprio do CAD-CRAFT.

Não copiar as páginas do projeto demonstrativo do EPLAN.

## Conteúdo sugerido

Criar um projeto demonstrativo em memória contendo:

* Documentos.
* Páginas de tipos diferentes.
* Hierarquias funcionais.
* Hierarquias de localização.
* Representações documentais disponíveis.
* Metadados básicos.

O projeto deverá utilizar APIs reais do Core.

Não inventar componentes gráficos ou conectividade que o Core ainda não suporte.

## Objetivo

Permitir testar:

* Árvore.
* Abas.
* Seleção.
* Propriedades.
* Navegação.
* Zoom.
* Pan.
* Pré-visualização.
* Enquadramento.
* Mudança de página.

Identificar claramente que se trata de demonstração.

---

# 31. EXPERIÊNCIA INICIAL DO APLICATIVO

Ao abrir o CAD-CRAFT:

1. Exibir a janela principal.
2. Carregar as preferências visuais disponíveis.
3. Mostrar a Ribbon.
4. Mostrar os navegadores.
5. Mostrar o painel de propriedades.
6. Mostrar o centro de inserção.
7. Apresentar a área central vazia ou uma tela inicial própria.
8. Permitir criar ou carregar uma sessão compatível com o Core.
9. Permitir abrir o projeto demonstrativo.

Não abrir um terminal preto adicional na versão Release do Windows.

Configurar corretamente o subsistema gráfico do executável.

Preservar o terminal de diagnóstico em Debug, quando apropriado.

---

# 32. RESPONSIVIDADE E DESEMPENHO

A interface deverá permanecer responsiva.

Evitar:

* Clonagem completa do projeto a cada frame.
* Reconstrução desnecessária de árvores.
* Consultas pesadas repetidas.
* Renderização integral de conteúdo fora do viewport.
* Carregamento repetido de recursos.
* Recriação desnecessária de texturas.
* Consumo excessivo de memória.

Separar:

* Estado persistente.
* Estado transitório.
* Dados derivados.
* Cache de apresentação.

Não utilizar threads ou serviços adicionais sem necessidade.

---

# 33. ARQUITETURA SUGERIDA DA GUI

Criar uma organização modular, adaptada ao workspace real.

Estrutura conceitual:

```text
crates/cad-gui/
└── src/
    ├── main.rs
    ├── app.rs
    ├── state.rs
    ├── theme.rs
    ├── commands.rs
    ├── preferences.rs
    │
    ├── ribbon/
    │   ├── mod.rs
    │   ├── tabs.rs
    │   ├── groups.rs
    │   └── buttons.rs
    │
    ├── panels/
    │   ├── mod.rs
    │   ├── page_navigator.rs
    │   ├── preview.rs
    │   ├── insertion_center.rs
    │   ├── properties.rs
    │   ├── messages.rs
    │   └── statusbar.rs
    │
    ├── viewport/
    │   ├── mod.rs
    │   ├── camera.rs
    │   ├── grid.rs
    │   ├── page.rs
    │   └── interaction.rs
    │
    ├── dialogs/
    │   ├── mod.rs
    │   └── pages.rs
    │
    └── demo.rs
```

Esta estrutura é orientativa.

Não criar arquivos vazios apenas para reproduzir o diagrama.

Escolher módulos de acordo com as responsabilidades reais.

---

# 34. DEPENDÊNCIAS

Avaliar versões compatíveis de:

* egui.
* eframe.
* Bibliotecas auxiliares realmente necessárias.
* Ícones vetoriais.
* Serialização de preferências, se implementada.

Não assumir versões de dependências sem consultar o ambiente.

Registrar versões e justificativas.

Evitar introduzir bibliotecas grandes somente para produzir pequenos detalhes estéticos.

Garantir compatibilidade com Windows 10.

---

# 35. TESTES AUTOMATIZADOS

Preservar os 128 testes anteriores.

Adicionar testes para:

1. Estado inicial da GUI.
2. Registro de comandos.
3. Habilitação dos comandos.
4. Troca de guias.
5. Seleção de página.
6. Abertura de página.
7. Fechamento de aba.
8. Identidade das páginas.
9. Navegação hierárquica.
10. Ordenação documental.
11. Filtros.
12. Seleção.
13. Propriedades.
14. Transformação world-to-screen.
15. Transformação screen-to-world.
16. Inversão do eixo Y.
17. Zoom.
18. Zoom ancorado no cursor.
19. Pan.
20. Enquadramento de página.
21. Enquadramento de conteúdo.
22. Grade.
23. Snap.
24. Conversão de coordenadas.
25. Persistência de preferências.
26. Estados dos painéis.
27. Estado das abas.
28. Redimensionamento.
29. Comandos desabilitados.
30. Compatibilidade com o Core.

Criar testes específicos que verifiquem:

**O viewport continua ocupando toda a região central independentemente da escala visual da folha.**

Testar transformação e navegação sem exigir um monitor físico quando possível.

---

# 36. TESTES MANUAIS DE INTERFACE

Preparar um roteiro completo de homologação.

## Teste A — Aparência geral

Abrir o CAD-CRAFT e comparar com `01.png`.

Verificar:

* Proporções.
* Cores.
* Ribbon.
* Painéis.
* Abas.
* Tipografia.
* Ícones.
* Separadores.

## Teste B — Ribbon

Alternar todas as guias.

Verificar:

* Conteúdo.
* Destaques.
* Botões.
* Tooltips.
* Menus.
* Separadores.

## Teste C — Menu Arquivo

Abrir e fechar o Backstage.

Verificar navegação e restauração da interface.

## Teste D — Navegador

Criar documentos e páginas usando operações disponíveis.

Testar:

* Expansão.
* Recolhimento.
* Filtros.
* Seleção.
* Duplo clique.
* Abertura.
* Ordenação.

## Teste E — Propriedades

Selecionar:

* Projeto.
* Documento.
* Página.
* Representação.

Verificar atualização contextual.

## Teste F — Documentos

Abrir diversas páginas.

Alternar e fechar abas.

Confirmar preservação do modelo documental.

## Teste G — Zoom

Executar zoom in e zoom out repetidamente.

Comparar o comportamento com o segundo vídeo.

Verificar:

* A folha muda de tamanho visual.
* O viewport mantém suas dimensões.
* O pan continua funcionando.
* A grade acompanha o zoom.
* As coordenadas permanecem corretas.

## Teste H — Painéis

Redimensionar os navegadores.

Recolher painéis.

Restaurar o layout.

Verificar que o viewport utiliza o espaço liberado.

## Teste I — Barra inferior

Testar:

* Grade.
* Snap.
* Zoom.
* Página inteira.
* Coordenadas.
* Tooltips.

## Teste J — DPI

Testar a interface em diferentes escalas do Windows.

Verificar legibilidade e ausência de cortes.

## Teste K — Regressão

Executar a suíte completa do Core.

Confirmar que a interface não alterou a conectividade elétrica ou as regras documentais anteriores.

---

# 37. VALIDAÇÃO FINAL

Executar:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

Além disso, executar compilação Release da GUI, identificando o pacote e o binário reais do workspace.

Executar:

```powershell
git diff --check
git status --short
```

Quando disponível, solicitar revisão independente.

Verificar:

* Arquitetura.
* Estado da GUI.
* Renderização.
* Câmera.
* Uso de memória.
* Compatibilidade com o Core.
* Comandos habilitados/desabilitados.
* Dependências.
* Qualidade dos testes.

Não remover testes.

Não desabilitar verificações.

Não utilizar `unsafe` desnecessariamente.

---

# 38. DOCUMENTAÇÃO

Criar o relatório:

`docs/internal/reports/ETAPA_05_INTERFACE_GRAFICA_EGUI.md`

Documentar:

1. Objetivo.
2. Referências analisadas.
3. Arquitetura.
4. Dependências.
5. Tema visual.
6. Ribbon.
7. Backstage.
8. Navegadores.
9. Árvore documental.
10. Pré-visualização.
11. Abas.
12. Viewport.
13. Câmera.
14. Zoom.
15. Pan.
16. Grade.
17. Snap.
18. Centro de inserção.
19. Propriedades.
20. Painéis acopláveis.
21. Barra de mensagens.
22. Barra de status.
23. Atalhos.
24. Integração com Core.
25. Testes.
26. Limitações.
27. Comparação com as referências.
28. Roteiro de homologação.

Atualizar:

* `docs/internal/memory/CURRENT_STATE.md`.
* `docs/internal/memory/HANDOFF.md`.
* `docs/internal/memory/ROADMAP.md`.
* `docs/internal/memory/ARCHITECTURE.md`, quando necessário.
* ADRs, caso existam decisões novas.

Não registrar homologação manual antes da aprovação do usuário.

---

# 39. CRITÉRIOS DE ACEITAÇÃO

A Etapa 5 estará pronta para homologação quando:

* [ ] O aplicativo possuir janela gráfica egui/eframe operacional.
* [ ] A versão Release não abrir console adicional.
* [ ] O tema escuro estiver implementado.
* [ ] A barra superior estiver implementada.
* [ ] A Ribbon estiver organizada em guias e grupos.
* [ ] Os comandos disponíveis executarem operações reais.
* [ ] Os comandos futuros estiverem claramente desabilitados.
* [ ] O Backstage estiver implementado.
* [ ] O navegador de páginas estiver funcional.
* [ ] A hierarquia documental refletir o Core.
* [ ] As abas de documentos estiverem operacionais.
* [ ] O painel de pré-visualização estiver funcional.
* [ ] O centro de inserção estiver preparado.
* [ ] O painel de propriedades responder à seleção.
* [ ] Os divisores forem redimensionáveis.
* [ ] O layout puder ser restaurado.
* [ ] O viewport ocupar toda a área central disponível.
* [ ] A folha não controlar as dimensões do viewport.
* [ ] Zoom e pan funcionarem.
* [ ] A grade permanecer alinhada ao documento.
* [ ] O sistema de coordenadas estiver correto.
* [ ] A barra inferior possuir controles funcionais.
* [ ] Os principais atalhos disponíveis funcionarem.
* [ ] A interface adaptar-se a diferentes resoluções.
* [ ] O Core continuar independente da GUI.
* [ ] Os testes existentes permanecerem aprovados.
* [ ] Os novos testes passarem.
* [ ] A documentação estiver atualizada.
* [ ] Não houver commit ou push.

---

# 40. RESTRIÇÕES FINAIS

Não executar git commit.

Não executar git push.

Não iniciar a Etapa 6.

Não implementar prematuramente editor multifilar completo.

Não implementar biblioteca elétrica fictícia.

Não criar um segundo modelo de conectividade.

Não modificar o Core além do necessário para integrar suas APIs existentes.

Não copiar recursos proprietários do EPLAN.

Não utilizar capturas do EPLAN como interface estática.

Não implementar botões decorativos como se fossem funções completas.

Não descartar arquivos preexistentes.

Não alterar projetos pessoais ou backups.

Não declarar funcionalidades incompletas como concluídas.

Não confundir testes automatizados com homologação manual.

---

# 41. RESULTADO ESPERADO

Ao final da Etapa 5, o CAD-CRAFT deverá apresentar uma interface desktop profissional visualmente muito próxima da organização do EPLAN Electric P8.

O aplicativo deverá possuir:

* Ribbon profissional.
* Menu Arquivo completo em sua estrutura visual.
* Navegadores.
* Documentos tabulados.
* Área gráfica.
* Painéis redimensionáveis.
* Pré-visualização.
* Centro de inserção.
* Inspetor de propriedades.
* Barra de mensagens.
* Barra de status.
* Sistema de navegação.
* Zoom.
* Pan.
* Grade.
* Integração documental com o Core.

A interface deverá estar preparada para receber as funcionalidades elétricas das próximas etapas sem exigir uma reconstrução arquitetural.

**Prioridade máxima: aparência profissional, fidelidade de layout, navegação fluida e separação rigorosa entre viewport, folha e modelo elétrico.**

A experiência visual deve ser reconhecivelmente inspirada no EPLAN, enquanto o código, os recursos gráficos e a identidade do produto permanecem próprios do CAD-CRAFT.

Concluir a implementação, apresentar os resultados e aguardar homologação manual antes de qualquer commit ou push.
