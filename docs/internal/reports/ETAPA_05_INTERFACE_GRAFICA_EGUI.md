# Etapa 5 — Interface gráfica egui/eframe

Referência: 2026-10-28, America/Sao_Paulo. Implementação autorizada pelo anexo
integral em `docs/internal/spec/ETAPA_05_INTERFACE_GRAFICA_EGUI.md`.
Plano: `docs/internal/plans/2026-10-27-etapa-05-interface.md`. Decisão: ADR-0007.

**Estado atual: correções visuais da segunda rodada implementadas; homologação parcial
confirmada pelo usuário. Evidências por captura oficial e homologação restante pendentes.**
As seções 1–28 descrevem a entrega original. A seção final da segunda rodada prevalece
para correções, validações, estado manual e limitações atuais.

## 1. Objetivo

Converter o binário diagnóstico em aplicação desktop documental, com navegação,
Ribbon, Backstage, painéis, inspetor, miniaturas e viewport independente da folha.
Somente Etapa 5; nenhuma implementação da Etapa 6 ou alteração do Core.

## 2. Referências analisadas

28 PNG locais em `.local/references/eplan`: 00, 01 e 001 examinadas individualmente;
demais imagens examinadas em pranchas. Dois MP4 examinados por amostragem, sem
afirmar reprodução integral: vídeo 1 em 0, 45, 90, 150, 210, 270, 330, 390, 450,
520 segundos; vídeo 2 em 0, 5, 10, 15, 20, 30, 40, 50, 60, 70, 80, 86 segundos.
Pranchas e quadros permanecem ignorados em `.local/etapa-05/`.
FFmpeg 7.0.2, distribuído por imageio-ffmpeg 0.6.0, foi usado apenas nessa inspeção
local. Não é dependência, recurso distribuído nem código do CAD-CRAFT.

## 3. Arquitetura

`cad-gui → cad-craft`; Core preservado. CadApp coordena módulos concretos de sessão,
comandos, navegação, edição, câmera, preferências e painéis. Project é a única fonte
de entidades. PageId/RepresentationId identificam seleção e abas; nomes não são
chaves. Índice de páginas/árvore e rascunho do inspetor são caches derivados.
Não há clonagem integral do projeto a cada frame, serviços novos ou threads próprias.

## 4. Dependências

| Dependência direta | Versão resolvida | Uso | Licença declarada |
|---|---|---|---|
| eframe / egui reexportado | 0.33.3 | Janela, widgets, entrada, Glow, AccessKit, storage | MIT OR Apache-2.0 |
| serde | 1.0.229 | Preferências visuais | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | Preferências e snapshot técnico | MIT OR Apache-2.0 |
| tempfile | 3.27.0 | Preparação segura da exportação | MIT OR Apache-2.0 |
| image, somente dev | 0.25.10 | PNG do exemplo de inspeção | MIT OR Apache-2.0 |
| cad-craft | workspace 0.1.0 | Domínio existente | Política do projeto |

Rust 1.99.1; eframe exige Rust 1.88. Eframe fixado em `=0.33.3`, com Glow,
default_fonts, accesskit, persistence, x11 e wayland; sem backend WGPU.
O lockfile contém 369 pacotes, incluindo alvos não compilados no Windows;
essa contagem não equivale ao número de bibliotecas embarcadas. Licenças acima
conferidas nos manifests locais. Winit 0.30.13 declara Apache-2.0; Glow 0.16.0,
MIT OR Apache-2.0 OR Zlib. A distribuição futura exige revisar avisos transitivos.
Nenhuma biblioteca comercial foi acrescentada.

Foram consultadas as APIs oficiais de
[eframe 0.33.3](https://docs.rs/eframe/0.33.3/eframe/) e
[egui 0.33.3](https://docs.rs/egui/0.33.3/egui/), além do código fonte local
correspondente. A série 0.33.3 foi escolhida pela API de painéis utilizada;
não se apresenta como a versão mais recente disponível.

## 5. Tema visual

Tema centralizado em `theme.rs`: títulos quase pretos, painéis cinza escuro,
seleção azul, papel branco e área externa cinza clara. Tipografia compacta,
fontes embutidas, ícones vetoriais próprios em `icons.rs`. Recursos do EPLAN não
foram copiados. Controles de janela são nativos. Release usa subsistema Windows;
Debug conserva console para diagnóstico.

## 6. Ribbon

Dez guias: Página Inicial, Inserir, Editar, Visualização, Dispositivos, Conexões,
Ferramentas, Pré-planejamento, Dados-mestre e Integrações. Grupos do anexo combinam
botão principal, colunas compactas, separadores, nomes e menu. Há rolagem horizontal
em áreas estreitas. Ctrl+F1 e duplo clique recolhem/expandem; pouca altura reduz a
Ribbon automaticamente. Registro único de comandos oferece busca por nome,
palavras relacionadas, guia e grupo, com motivos de indisponibilidade.

## 7. Backstage

Arquivo contém a navegação solicitada. Informações mostra propriedades, estatística,
estrutura e status reais. Novo cria sessão, Abrir/Importar recebem snapshot técnico,
Exportar gera JSON, Configurações altera preferências, Extras valida o Core/restaura
layout, Ajuda explica navegação. Backup, impressão, revisão e demais operações futuras
permanecem desabilitadas. Descarte de alterações e sobrescrita exigem confirmação.

## 8. Navegadores

Páginas, Dispositivos, Estruturas, Conexões, Cabos e Bornes consultam registros reais.
Seleção elétrica alimenta o inspetor e permite navegar a ocorrências existentes.
Estruturas exibem caminhos derivados, com destaque de seleção, sem inventar IDs de grupo.
Navegadores sem registros mostram estado vazio explícito.

## 9. Árvore documental

Projeto → documento → caminhos funcionais → caminhos de localização → páginas.
Modos árvore/lista, filtros por texto, tipo, documento e estrutura; tipos possuem
ícones distintos. Clique seleciona, duplo clique/Enter abre; setas navegam e expandem.
Menu contextual abre, cria, renomeia, identifica, reordena, mostra propriedades ou
solicita exclusão validada. Sincronizar seleção limpa filtros e revela a página ativa,
inclusive fora das linhas virtualizadas. Identidade permanece estável após renomear.

## 10. Pré-visualização

Miniaturas em grade/lista usam a mesma função de renderização e as páginas reais,
com câmera própria de enquadramento. Clique seleciona, duplo clique abre. Área
inferior esquerda redimensionável e com rolagem virtualizada. Rótulos gráficos são
omitidos em miniaturas estreitas. Em pouca altura, o painel é temporariamente
recolhido, com indicação de aumentar a janela ou recolher a Ribbon; preferência
do usuário é preservada. A árvore é recortada antes da região de miniaturas.

## 11. Abas

Cada aba referencia PageId e conserva câmera/vista anterior. Ativar uma aba não
troca sua identidade; fechar mantém a página no Project. Selecionar no navegador
não abre automaticamente. Cabeçalhos refletem nome/identificação atuais.

## 12. Viewport

Ocupa o retângulo central disponível. Folha é um objeto no mundo, nunca o tamanho
da área interativa. Papel, margem visual, exterior, grade, limites e âncoras são
desenhados pelo mesmo pipeline. Entidades fora da folha continuam válidas e podem
ser enquadradas. Hit testing usa a transformação inversa e limites locais, ou
proximidade de seis pontos da âncora; não há geometria de símbolos.

## 13. Câmera

Centro/escala em f64, milímetros cartesianos, X à direita e Y para cima. Conversão
para egui inverte somente Y. Centro limitado a ±10¹² mm e escala a 10⁻⁴–10⁴
pontos/mm. Recorte de folha/segmentos em f64 evita desaparecer quando os cantos
ultrapassam coordenadas seguras de tesselação. Entradas inválidas preservam a câmera.

## 14. Zoom

Roda ancora o ponto do mundo sob o cursor; botões oferecem aumentar, diminuir,
100%, página inteira, conteúdo e vista anterior. Enquadramento inclui limites do
Core, inclusive fora da folha; sem conteúdo, usa folha. 100% é 96/25,4 pontos/mm,
referência de tela, sem garantia de dimensão física no monitor ou impressão.

## 15. Pan

Arraste do botão central desloca a câmera sem editar entidades ou escala. Cursor
de mão indica o gesto. Testes enviam eventos de mouse reais ao Context egui.

## 16. Grade

Espaçamento real em mm, alinhado à origem, presente dentro e fora do papel. Densidade
visual adapta-se por potências de dois ao zoom; até 2.048 linhas candidatas por frame.
Valores 1, 2, 2,5, 4, 8 e 16 mm na barra; configuração validada de 0,1–1.000 mm.
Mostrar/ocultar grade não altera coordenadas ou dimensão documental.

## 17. Snap

Alternância independente da visibilidade da grade. Indicador aproxima o cursor ao
múltiplo do espaçamento real, inclusive coordenadas negativas. Não insere nem move
entidades nesta etapa; não implica ferramenta gráfica elétrica implementada.

## 18. Centro de inserção

Categorias Página inicial, Últimos utilizados, Favoritos, Tags, Símbolos, Dispositivos,
Macros e Catálogos; busca, voltar/avançar/início e favoritos da sessão. Dispositivos
existentes são selecionáveis. Categorias sem infraestrutura mostram estado vazio;
nenhuma biblioteca, catálogo ou inserção gráfica fictícia foi criada.

## 19. Propriedades

Categorias/tabela, perfis e pesquisa por propriedade; contexto projeto, documento,
página, representação ou alvo elétrico. Edição de descrição/identificação, página,
tipo/formato/estrutura, transformação e atributos visuais passa pelas APIs do Core.
Campos técnicos sem mutação disponível são somente leitura. Erros não substituem
dados válidos. Rascunho é atualizado por seleção/revisão, e não clona Project.

## 20. Painéis acopláveis

Navegador à esquerda, inserção/propriedades à direita, miniaturas e mensagens abaixo.
Divisores alteram espaço realmente disponível. Cabeçalhos oferecem fechar, recolher,
pin e menu; em painéis estreitos as ações permanecem no menu. Pin preserva painéis
ao recolher os desafixados. Menu Painéis e Configurações restauram visibilidade/layout.
Preferências visuais persistem pelo storage local do eframe; projeto não é salvo
automaticamente. Docking flutuante e redistribuição livre permanecem adiados.

## 21. Barra de mensagens

Mensagens de informação, aviso e erro com filtros, contadores, limpeza e área
redimensionável/recolhível. Limite de 500 registros por sessão. Validar chama
Project::validate e não se apresenta como verificação elétrica normativa/ERC.

## 22. Barra de status

Indica sessão/demonstração e mensagem recente; área CAD mostra coordenadas mm,
snap, grade, espaçamento e zoom/enquadramento. Modo de construção, fixação gráfica
e visualização lógica são futuros e desabilitados, com tooltip. Controles usam
quebra/rolagem conforme o espaço disponível.

## 23. Atalhos

Esc cancela diálogo/busca/Backstage ou retorna seleção à página. Enter confirma
diálogos e abre no navegador. Ctrl+F1 recolhe Ribbon; F11 alterna tela cheia;
Ctrl+PageUp/PageDown navega páginas fora de texto/modal/Backstage. Setas pertencem
à árvore quando focada. Ctrl+C/V dos campos não são interceptados por comandos CAD.

## 24. Integração com Core

Todos os arquivos e testes de cad-craft foram preservados. GUI usa Project,
Document, Page, Representation e consultas documentais/elétricas existentes.
Demonstração original: dois documentos, seis páginas de cinco tipos, dispositivo
K1/função e quatro ocorrências, uma fora da folha. Não é esquema executável.
Importação tem limite de 8 MiB e validação integral antes da troca; exportação
prepara arquivo temporário vizinho antes de publicar, removendo-o em caso de falha.
Snapshot não substitui o formato físico futuro nem backup independente.

## 25. Testes e evidências

Baseline: 113 Rust + 15 PowerShell = 128 aprovados. Resultado final em 2026-10-28:
**164 Rust + 15 PowerShell = 179 aprovados; zero falhas e zero ignorados.**
Core: 110 testes de integração + três compile_fail, preservados. GUI: 46 testes de
integração + cinco internos, todos novos. Logs ignorados em `.local/etapa-05/`.

| Comando/verificação | Resultado | Evidência local |
|---|---|---|
| `cargo fmt --check` | Aprovado | validation-final.log |
| `cargo check --workspace --all-targets --locked` | Aprovado | validation-final.log |
| `cargo test --workspace --locked` | 164 aprovados | validation-final.log |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Aprovado, sem avisos | validation-final.log |
| Testes PowerShell do diagnóstico | 15 aprovados | validation-final.log |
| `git diff --check` e `git diff --cached --check` | Aprovados | validation-final.log |
| `pwsh -NoProfile -File scripts/validate.ps1` | Exit 0, todas as verificações acima | validation-final.log |
| `cargo build --release -p cad-gui --bin cad-gui --locked` | Exit 0 | release-build.log |
| Cabeçalho PE do Release | Subsystem 2, Windows GUI | Leitura direta do PE |
| `pwsh -NoProfile -File scripts/check-repository.ps1` | Exit 0; nenhum achado nas regras auditadas | repository-check.log |

Binário: `target/release/cad-gui.exe`, 7.856.128 bytes, ignorado pelo Git. Execução:
`cargo run --release -p cad-gui --bin cad-gui --locked`, ou abrir esse executável.
Verificação PE confirma o subsistema; homologação visual de Release/Windows diversos
continua no roteiro A. Inicialização nativa e captura de renderização foram exercitadas
pelo Debug/exemplo usando a mesma UI de produção.

A primeira passagem completa apontou organização de módulo de testes no Clippy;
foi corrigida movendo os testes, sem `allow`, sem remover testes nem reduzir checks.
O aviso Git de normalização futura LF/CRLF do Cargo.lock não é erro de whitespace.
Diagnóstico offline não varre conteúdo sensível/histórico nem comprova estado remoto.
A consulta remota separada está registrada ao fim deste relatório.

Novas suítes: câmera/preferências (14), sessão (16), comandos (5), interface (11)
e regressões internas (5). Cobrem identidade, filtros, edições inválidas, snapshot,
geometria, grade, eventos de mouse/teclado, divisores, proteção de foco e recorte.
Layouts exercitados em 1280×720, 1366×768, 1600×900, 1920×1080, 1920×1200 e
2560×1440, nas escalas lógicas 100%, 125%, 150% e 200%.

Revisão independente interna somente leitura encontrou quatro problemas relevantes:
foco da árvore não registrado, sincronização de linha fora do viewport, exportação
que truncava arquivo antes de concluir e desaparecimento de folha extrema.
Regressões reproduzidas em `review-red.log`/`snapshot-red.log`; correções verificadas
em `review-green.log`. Adicionalmente, inspeção em 200% revelou sobreposição de
árvore/miniaturas, corrigida com recorte e recolhimento por altura.

A captura nativa externa apresentou timeout. `examples/render_qa.rs` usa a UI real
e o evento Screenshot do eframe antes da troca de buffers. Capturas finais locais:
`gui-demo-final.png`, `gui-small.png`, `gui-zoom-out.png`, `gui-dpi200.png` e
`gui-backstage.png`. Não são mocks. A primeira tentativa `gui-demo.png` resultou
preta e foi descartada como evidência. Escala de captura usa zoom do egui, sem mudar
DPI do Windows. O usuário confirmou que via painéis/botões e aprovou a aparência;
isso não foi tratado como execução completa do roteiro de homologação.

## 26. Limitações

- Sem símbolos elétricos, conexões desenhadas/roteadas, CRUD elétrico completo,
  ERC, Undo/Redo, persistência física final, impressão ou exportação CAD/PDF.
- Sem docking flutuante, tela 3D, biblioteca comercial ou serviços externos.
- Representações são ordenadas/transformadas a cada pintura. Não foi homologado
  desempenho de grandes desenhos; cache espacial deve depender de medições reais.
- Faixa numérica da câmera é menor que o conjunto de valores finitos aceitos pelo
  Core. Enquadramentos incompatíveis informam limitação sem alterar domínio.
- Preferências persistem, mas sessão/favoritos/abas precisam de requisitos futuros
  para restauração. JSON técnico não fornece histórico/migrações.
- Windows 10, múltiplas GPUs/monitores, DPI nativo, acessibilidade completa e
  legibilidade em todos os tamanhos aguardam homologação. Escala lógica testada
  automaticamente não substitui essa matriz.
- ENV-003 (reinício do Windows solicitado anteriormente) segue sem confirmação.

## 27. Comparação com as referências

| Aspecto observado | Entrega CAD-CRAFT | Diferença deliberada |
|---|---|---|
| Tema escuro, azul e Ribbon densa | Mesma organização de regiões, guias/grupos e hierarquia | Ícones/identidade próprios, controles nativos |
| Navegador esquerdo com árvore e miniaturas | Árvore derivada, filtros e miniaturas funcionais | Apenas entidades e relações existentes |
| Inserção e propriedades à direita | Dois painéis com busca, seleção e edição contextual | Bibliotecas e inserção gráfica futuras |
| Vídeo 2: folha pequena com área central estável | Captura de zoom reduzido e teste de retângulo invariável | Limites/âncoras documentais, sem esquema elétrico |
| Divisores, status e mensagens | Redimensionamento, restauração, coordenadas e logs | Docking livre adiado |
| Menu Arquivo em tela própria | Backstage com dados/ações reais e itens futuros visíveis | Snapshot técnico explicitamente experimental |

Sem alegação de reprodução pixel a pixel. Similaridade visual também foi confirmada
informalmente pelo usuário; fidelidade final depende do roteiro abaixo.

## 28. Roteiro de homologação futura

Todos os itens permanecem **pendentes de homologação manual**, adiada até após
Etapa 15. Resultados automáticos são registrados separadamente.

| Teste do anexo | Procedimento e resultado esperado |
|---|---|
| A — Aparência | Abrir Release; conferir tema, proporções, títulos, tooltips e ausência de console |
| B — Ribbon | Percorrer dez guias, menus e busca; ações reais habilitadas por contexto, futuras desabilitadas |
| C — Arquivo | Abrir/retornar Backstage; conferir dados; criar, importar/exportar snapshot e cancelar descarte |
| D — Navegador | Expandir/filtrar/reordenar; clique seleciona, duplo/Enter abre; sincronizar página distante |
| E — Propriedades | Alternar seleção, editar campo válido/inválido e conferir resposta do Core e mensagens |
| F — Documentos | Abrir várias abas; câmeras independentes; fechar aba preserva página; exclusão usada é bloqueada |
| G — Zoom | Roda nos cantos/centro, zoom extremo, enquadramentos e pan; área interativa mantém dimensões |
| H — Painéis | Arrastar divisores, ocultar/recolher/fixar e restaurar; espaço liberado vai ao viewport |
| I — Barra inferior | Conferir coordenadas, grade, snap, zoom, página inteira e tooltips sem editar domínio |
| J — DPI | Windows 100/125/150/200%, resoluções e monitores diversos; verificar recortes/legibilidade |
| K — Regressão | Executar scripts/validate.ps1; conferir contagem integral e nenhum teste anterior removido |

## Git, inventário e próxima ação

Branch `main`; último HEAD observado em 2026-10-28 por `git rev-parse HEAD`:
`5eac9647458434a4d7dc87f6592c97627882b398` (Etapa 4).
`git ls-remote --exit-code origin refs/heads/main` confirmou o mesmo SHA nessa data.
Origin existente: `https://github.com/Vandecao/CAD-CRAFT.git`. Nenhum fetch.
O estado antigo da memória (Etapa 4 não commitada) foi corrigido conforme Git.
Arquivos desta entrega não integram esse HEAD; nenhuma mutação Git realizada.

### Inventário final

Alterações de código restritas a cad-gui e Cargo.lock. Core, testes anteriores,
scripts, manifest raiz e .gitignore inalterados. Versões anteriores de serde,
serde_json e uuid preservadas no lockfile. Arquivos novos/alterados ainda sem staging.

Após revisão: commit manual da Etapa 5; push conforme política/perfil do usuário.
Backup externo independente recomendado, não criado/verificado. Encerrar sem Etapa 6.

Arquivos modificados (10):

- `Cargo.lock`
- `crates/cad-gui/Cargo.toml`
- `crates/cad-gui/src/main.rs`
- `docs/internal/memory/ARCHITECTURE.md`
- `docs/internal/memory/CURRENT_STATE.md`
- `docs/internal/memory/DECISIONS.md`
- `docs/internal/memory/HANDOFF.md`
- `docs/internal/memory/KNOWN_ISSUES.md`
- `docs/internal/memory/PROJECT_CONTEXT.md`
- `docs/internal/memory/ROADMAP.md`

Arquivos criados (31):

- `crates/cad-gui/examples/render_qa.rs`
- `crates/cad-gui/src/app.rs`
- `crates/cad-gui/src/backstage.rs`
- `crates/cad-gui/src/camera.rs`
- `crates/cad-gui/src/commands.rs`
- `crates/cad-gui/src/demo.rs`
- `crates/cad-gui/src/dialogs.rs`
- `crates/cad-gui/src/editing.rs`
- `crates/cad-gui/src/icons.rs`
- `crates/cad-gui/src/lib.rs`
- `crates/cad-gui/src/navigation.rs`
- `crates/cad-gui/src/panels/insertion.rs`
- `crates/cad-gui/src/panels/messages.rs`
- `crates/cad-gui/src/panels/mod.rs`
- `crates/cad-gui/src/panels/navigator.rs`
- `crates/cad-gui/src/panels/preview.rs`
- `crates/cad-gui/src/panels/properties.rs`
- `crates/cad-gui/src/preferences.rs`
- `crates/cad-gui/src/ribbon.rs`
- `crates/cad-gui/src/snapshots.rs`
- `crates/cad-gui/src/state.rs`
- `crates/cad-gui/src/theme.rs`
- `crates/cad-gui/src/viewport.rs`
- `crates/cad-gui/tests/camera_preferences.rs`
- `crates/cad-gui/tests/commands.rs`
- `crates/cad-gui/tests/interface.rs`
- `crates/cad-gui/tests/session.rs`
- `docs/adr/0007-interface-desktop-e-sessao-documental.md`
- `docs/internal/plans/2026-10-27-etapa-05-interface.md`
- `docs/internal/reports/ETAPA_05_INTERFACE_GRAFICA_EGUI.md`
- `docs/internal/spec/ETAPA_05_INTERFACE_GRAFICA_EGUI.md`

Arquivos excluídos: nenhum. Artefatos locais de build, logs, capturas e ferramentas
de inspeção permanecem ignorados em target/ e .local/; não foram adicionados ao Git.

## Correções após homologação visual — segunda rodada

Referência: 2026-10-28. Spec integral: `docs/internal/spec/ETAPA_05_CORRECOES_VISUAIS_RODADA_2.md`.
Plano: `docs/internal/plans/2026-10-28-etapa-05-correcoes-visuais.md`. Escopo exclusivamente Etapa 5.
Implementação e validação automática concluídas; capturas oficiais A–J e homologação
restante pendentes. Não declarar encerrada a evidência visual exigida pelo anexo.

### Homologação recebida e causas confirmadas

O anexo confirma aprovação parcial de inicialização Release, demonstração, navegadores,
hierarquia, abas, painéis, inserção, propriedades, documentos, zoom/pan, viewport e
Backstage. Em resposta direta nesta sessão, o usuário confirmou: “A barra está escura
e os controles legíveis”. Isso valida esse aspecto no Windows 10 local, sem homologar
todos os controles, Windows 11, alto contraste ou todas as correções.

| Problema | Causa reproduzida | Correção |
|---|---|---|
| Grade quadriculada | `draw_page` pintava segmentos retornados por `grid_lines`; teste encontrou zero círculos | `grid_points` calcula interseções visíveis e pinta círculos |
| Texto escuro nos botões iniciais | `override_text_color=INK` local atingia também botões escuros; contraste medido 1,217:1 | Estilo local distingue rótulos e estados interativos, com texto claro nos botões |
| Título Windows claro | Tema escuro aplicado apenas ao estilo egui, sem comando para decoração nativa | API pública segura `ViewportCommand::SetTheme(Dark)` |
| Ribbon sem comandos | Ocultação automática abaixo de 480 pontos; duplo clique global do egui ao clicar rapidamente em guias diferentes; painel reutilizava altura recolhida como recorte no primeiro frame expandido | Remoção da ocultação por altura, duplo clique somente na guia já ativa e altura explícita por estado |

Logs RED: `round2-red.log` (quatro sintomas), `round2-interactions.log` (cliques entre
guias), `round2-backstage.log` (comandos pintados fora do recorte de 29 pontos),
`round2-review-red.log` (foco não reaplicava tema). Todos em `.local/etapa-05/`.

### Grade e contraste

- Grade ancorada no mundo documental em milímetros, inclusive coordenadas negativas
  e região externa à folha. Transformações da câmera e geometria do Core preservadas.
- Raio 0,8 ponto lógico, cinza RGB 139/149/160; círculos antialiasados pelo egui.
  Passo visual é múltiplo por potência de dois do passo lógico, mínimo visual de
  8 pontos e teto de 12.000 círculos. Cálculo restrito ao viewport; iteração limitada.
- Zoom afastado reduz somente densidade visual. Snap, valores 1/2/2,5/4/8/16 mm e
  configuração 0,1–1000 mm preservados. Miniaturas continuam sem grade.
- `grid_lines` permanece por compatibilidade dos testes anteriores; produção não
  usa essa função para pintar a grade. Linhas de contorno/margem da folha permanecem.
- Tela inicial mantém identidade, ações, centralização e textos. Rótulos escuros
  sobre canvas claro, texto claro em botões escuros; hover/pressionado/foco próprios.
  Opacidade desabilitada local 0,85, evitando perda de contraste sobre fundo claro.
- Teste calcula contraste de texto e fundo efetivos, incluindo composição alfa;
  exige pelo menos 4,5:1 nos cinco estados e nos textos auxiliares. Estilo global
  permanece inalterado. Scroll vertical permite acessar ações em pouca altura.

### Título nativo e compatibilidade

Windows local identificado: **Windows 10 Pro 22H2, build 19045.6466**. `ctx.set_theme`
define a preferência egui; `SetTheme(Dark)` passa por egui-winit ao `Window::set_theme`
do winit 0.30.13. Não removemos decoração, ícone ou botões, não criamos barra própria,
dependência nova ou FFI/unsafe no CAD-CRAFT. Release mantém subsistema Windows GUI.

Foram consultados [ViewportCommand 0.33.3](https://docs.rs/egui/0.33.3/egui/enum.ViewportCommand.html),
[tema Windows](https://learn.microsoft.com/windows/apps/desktop/modernize/apply-windows-themes)
e [atributos DWM](https://learn.microsoft.com/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute).
O suporte documentado de DWMWA_USE_IMMERSIVE_DARK_MODE é Windows 11 build 22000+;
não o presumimos uniforme no Windows 10. Preferimos a API Rust pública existente.
Limitação transparente: o backend winit usa internamente mecanismos específicos
do Windows, inclusive SetWindowCompositionAttribute; esta entrega não afirma que
seu caminho interno seja a API DWM documentada nem garante compatibilidade universal.

A revisão identificou que winit pode reaplicar preferência do sistema em
WM_SETTINGCHANGE sem preservar a solicitação anterior nem emitir ThemeChanged.
Reaplicamos o tema ao recuperar foco ou detectar mudança de tema informado ao egui,
sem enviar comandos continuamente. Limite restante: mudança sem evento nem perda de
foco pode exigir alternar para outra janela e voltar. Não alteramos configurações do
Windows. Windows 11, alto contraste, múltiplos monitores, Aero Snap e toda a matriz
de movimento/redimensionamento/foco nativo ainda requerem homologação.

### Ribbon e preservação do Backstage

`ribbon_collapsed` continua única fonte persistida. Troca de guia mantém esse estado;
quando recolhida, seleção muda a guia e os comandos ficam ocultos até expansão
explícita. Botão **Expandir**, Ctrl+F1 ou duplo clique na guia ativa permitem expandir.
Alturas de 137/31 pontos definem recorte coerente com estado expandido/recolhido.
Scroll horizontal é independente por guia. Os grupos existentes são preservados.

Testes exercitam cinquenta seleções por eventos de mouse, duplo clique, botão de
expansão, atalhos anteriores, scroll até último grupo, troca/retorno, estado de
preferências, documentos e Arquivo → Voltar. Verificam comandos inteiramente
visíveis, seleção, câmeras, abas e dimensões dos painéis. Matriz de seis resoluções
× quatro escalas inclui dez guias e acesso às ações iniciais por scroll quando
necessário. Escala lógica não equivale a DPI nativo de vários monitores.

`backstage.rs`, Core, lockfile, manifests, scripts e testes anteriores estão
byte-idênticos ao início desta rodada, conferidos por SHA-256 em
`round2-before-hashes.json`. Nenhuma substituição do menu Arquivo ou modelo elétrico.

### Verificação e revisão independente

- Baseline: 179 testes aprovados. Resultado final: **181 Rust + 15 PowerShell = 196**,
  zero falhas/ignorados. Core 113, GUI 68; **17 novos testes** (6 de grade, 10 de
  regressão visual/layout/eventos, 1 de estados de contraste).
- `pwsh -NoProfile -File scripts/validate.ps1`: exit 0. fmt, check de todos os alvos,
  testes, Clippy com `-D warnings` e diff checks aprovados. Log
  `.local/etapa-05/round2-validation-final.log`.
- `cargo build --release -p cad-gui --bin cad-gui --locked`: log
  `.local/etapa-05/round2-release-final.log`. Binário `target/release/cad-gui.exe`.
  Exit 0; 7.858.688 bytes; PE Subsystem 2 (Windows GUI). SHA-256 final:
  `E35D194BBE33B8B9E6BA787FD1F5122750FAC9C3432E9A1E5CF6413EDB6774FA`.
  Release final reaberto pelo Computer Use oficial para revisão do usuário.
- Diagnóstico offline do repositório sem achados nas regras auditadas; não equivale
  a auditoria completa de conteúdo/histórico/segredos. ENV-003 continua pendente.
- Revisão independente somente leitura encontrou risco de reset de tema e cobertura
  insuficiente de scroll. Mitigação e regressões incluídas; revisão incremental
  não identificou novo problema. Nenhum agente alterou Core/Backstage ou executou Git mutador.

### Evidências visuais e limitações da ferramenta

O usuário determinou inspeção somente pelo **Computer Use oficial** e autorizou o
acesso à janela. O plugin está disponível: localizou/abriu o Release e leu por
acessibilidade título, controles nativos, ações iniciais e grupos da Ribbon.
Após Ctrl+F1, nova leitura exibiu comandos; fechamento por Alt+F4 foi observado pela
ausência posterior da janela. AX não verifica pixels ou cores.

Tentativas de captura oficial, inclusive via cua_repl com seleção atualizada,
falharam: `FrameArrived timed out: timed out waiting on channel` e
`window capture timed out: timed out waiting on channel`. Não há captura nativa E
válida. Não usamos screenshots PowerShell nem captura interna egui como substituto
nesta rodada. Evidências A–D e F–J também **pendentes**. Os PNGs da entrega anterior
são históricos e não comprovam as correções atuais. A cor foi confirmada pelo usuário.

### Inventário desta rodada

Modificados em código: `crates/cad-gui/src/app.rs`, `camera.rs`, `ribbon.rs`,
`theme.rs`, `viewport.rs`. Novos testes: `crates/cad-gui/tests/grid_points.rs` e
`visual_regressions.rs`.
Documentação modificada: este relatório, CURRENT_STATE, HANDOFF, KNOWN_ISSUES,
DECISIONS, ROADMAP e ADR-0007. Novos: spec integral da rodada e plano de 2026-10-28.
Excluídos: nenhum. Logs/hashes/build locais ignorados, sem staging.

### Homologação restante e Git

1. Abrir demonstração e página: conferir pontos, pan/zoom, espaçamentos e desligar
   grade mantendo snap. Conferir miniaturas sem ruído.
2. Fechar abas e conferir botões iniciais, hover/foco e acesso por scroll em pouca altura.
3. Barra escura já confirmada nesta máquina; testar minimizar/reabrir, maximizar/
   restaurar, mover/redimensionar, foco e mudanças de tema. Validar Windows 11/alto
   contraste separadamente. Obter captura oficial E quando ferramenta permitir.
4. Alternar repetidamente dez guias, rolar grupos, Ctrl+F1, duplo clique e Expandir;
   nenhuma troca simples deve recolher a Ribbon.
5. Arquivo → opções existentes → Voltar: conferir guia, câmera, seleção, abas e painéis.
6. Repetir navegação documental/zoom/divisores e obter capturas oficiais A–J.

Branch **main**. HEAD observado por `git rev-parse HEAD` em 2026-10-28:
`5eac9647458434a4d7dc87f6592c97627882b398`. Consulta direta ao `origin` existente
por `git ls-remote --exit-code origin refs/heads/main` retornou o mesmo SHA nessa data.
Essa consulta não sincroniza. Toda Etapa 5 e estes documentos continuam fora desse
HEAD, sem staging do Codex. **Nenhum commit, push, fetch, worktree ou operação Git
modificadora realizado. Etapa 6 não iniciada. Core elétrico e Backstage preservados.**
Após revisão: commit manual; push conforme política do usuário. Backup externo
independente recomendado, não criado automaticamente.
