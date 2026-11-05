# Etapa 5 — plano de implementação da interface

> Execução: superpowers:executing-plans, nesta sessão, com testes de comportamento
> e revisão independente final. A autorização integral consta do anexo do usuário.
> AGENTS prevalece: sem worktree, staging, commit, push ou outra mutação Git.

**Objetivo:** construir a aplicação desktop documental egui/eframe da Etapa 5.
**Arquitetura:** cad-gui consome Project sem modificar o Core. Estado transitório
de sessão/abas/seleção é separado das preferências visuais e dos dados derivados.
Viewport ocupa o espaço central disponível; folha é conteúdo transformado pela câmera.
**Stack:** Rust 1.99.1; eframe/egui 0.33.3, Glow, fontes embutidas, AccessKit,
persistência de preferências; serde e serde_json para preferências/snapshot técnico.
**Spec:** `docs/internal/spec/ETAPA_05_INTERFACE_GRAFICA_EGUI.md` (anexo integral).

## Restrições globais e decisões

- Somente Etapa 5; main/base 5eac9647458434a4d7dc87f6592c97627882b398.
- Milímetros, X direita/Y cima no domínio. Y invertido somente na câmera.
- Sem símbolos fictícios, roteamento, editor multifilar, ERC ou Undo/Redo.
- Representações desenham limites conhecidos e âncoras, identificados como tais.
- Sem recursos proprietários: referências somente para análise; ícones vetoriais próprios.
- Docking por regiões laterais/inferiores, com redimensionamento e ocultação;
  flutuação livre adiada. Prioridade explícita do anexo.
- eframe 0.37.0 e 0.33.3 consultados no registry. Escolhida série 0.33.3 com API
  estável de painéis e backend Glow (OpenGL), evitando WGPU/serviços adicionais.
  MSRV 1.88 compatível com toolchain 1.99.1. Windows 10 precisa de driver OpenGL;
  teste nativo local não equivale a matriz homologada de hardware/Windows/DPI.
- Preferências pelo storage local eframe; nenhum Project persistido automaticamente.
- Importação/exportação explicitamente denominadas snapshot técnico JSON;
  sem formato .cadcraft, histórico, migração ou backup. Limite de entrada e
  substituição de sessão confirmada quando houver alterações.
- Título/controles de janela nativos; Release usa subsistema Windows.
- Registro central inclui comandos futuros desabilitados e motivo contextual.

## Referências verificadas

28 PNG presentes em `.local/references/eplan`; 00, 01 e 001 vistas diretamente,
demais imagens examinadas em pranchas. Dois MP4 examinados por amostragem:
vídeo 1 em 0,45,90,150,210,270,330,390,450,520s; vídeo 2 em
0,5,10,15,20,30,40,50,60,70,80,86s. Não se declara reprodução integral.
FFmpeg 7.0.2 do pacote imageio-ffmpeg 0.6.0 instalado apenas no diretório ignorado
`.local/etapa-05/tools`; quadros e pranchas ignorados, nunca recursos do produto.
Constatações: Ribbon ~130px a 1920px, temas cinza escuro/preto/azul, navegador
~18%, inserção/propriedades ~12% cada; divisores ajustáveis. Vídeo 2: aos 40s a
folha está pequena, mas área central e painéis conservam tamanho.

## Foco da revisão

1. Zoom extremo, coordenadas negativas e limites grandes: valores finitos e área
   interativa estável; testes câmera, grid limitado, layout headless.
2. Páginas homônimas/renomeadas/removidas: IDs e seleção distintos das abas;
   testes de sessão e exclusão bloqueada sem perda de entidades.
3. Foco textual/modal/Backstage: atalhos não editam ou navegam acidentalmente;
   testes de eventos egui e seleção por teclado.
4. Janelas pequenas/DPI/painéis grandes: região central continua utilizável;
   testes de layout em resoluções/escalas e restauração de divisores.
5. Snapshot inválido/preferências antigas: rejeição antes de substituir sessão,
   valores saneados; testes roundtrip, limites e estado preservado.

## Tarefas e interfaces

### 1. Câmera, grade e preferências
Arquivos: `crates/cad-gui/src/{lib,camera,preferences}.rs`, Cargo.toml e testes.
Camera: world_to_screen, screen_to_world, zoom_at, pan, fit; geometria f64 e
retângulo egui independente. Grid usa espaçamento mm e densidade adaptativa.
Preferences: versão, painel/guia/grade, saneamento e roundtrip JSON.
- [x] Escrever testes de inversa/Y/âncora/pan/fit/grid/snap/preferências.
- [x] Observar falha pelo comportamento ausente, implementar e executar testes.

### 2. Sessão, navegação, propriedades e demonstração
Arquivos: `state.rs`, `navigation.rs`, `demo.rs`, `editing.rs`, testes de sessão.
Session mantém Option<Project>, Selection, abas PageId/câmera e dirty; consultas
derivadas em cache só após mutação/filtro. Edits usam APIs verificadas do Core.
- [x] Testar seleção distinta de abertura, identidade, ordem/filtros/hierarquia,
  fechamento sem exclusão, edição inválida, remoção bloqueada, demo e snapshots.
- [x] Implementar e verificar contra Core real; preservar seus testes/arquivos.

### 3. Comandos e estrutura visual
Arquivos: `commands.rs`, `theme.rs`, `icons.rs`, `ribbon.rs`, `app.rs`, `main.rs`.
Registry fornece metadados/requisitos, busca e habilitação; UI despacha CommandId.
- [x] Testar registro único, busca, estados habilitados/futuros e troca de guias.
- [x] Implementar todas as guias/grupos do anexo e acesso rápido; Backstage,
  diálogos, tela inicial, confirmações de descarte e subsistema Windows.

### 4. Painéis e viewport
Arquivos: `panels/*`, `viewport.rs`, `backstage.rs`, `dialogs.rs` e testes headless.
- [x] Testar independência do viewport, redimensionamento, pan/zoom sob cursor,
  foco/atalhos, alternância de abas e painéis com eventos egui reais.
- [x] Implementar árvore/lista/teclado/contexto, miniaturas, inspetor editável,
  centro de inserção com estados vazios e registros existentes, mensagens/status.
- [x] Renderizar folha, grade e limites/âncoras reais; hit test amplo documentado;
  culling/scroll sem clonagem integral por frame. Verificar dimensões/DPI.

### 5. Integração, revisão e validação
- [x] Executar `cargo fmt --check`, check/test/clippy via `scripts/validate.ps1`.
- [x] Compilar `cargo build --release -p cad-gui --bin cad-gui --locked`.
- [x] Teste de inicialização nativa e inspeção visual automatizada, quando possível;
  não declarar homologação manual. Verificar subsistema PE.
- [x] Revisão independente interna somente leitura; corrigir achados relevantes.
- [x] Diagnóstico offline e diff/status; nenhum teste removido ou verificação reduzida.

### 6. Documentação e entrega
- [x] Relatório `docs/internal/reports/ETAPA_05_INTERFACE_GRAFICA_EGUI.md` com cobertura,
  referências, evidências, limitações, dependências/licenças e roteiro A–K.
- [x] Atualizar CURRENT_STATE, HANDOFF, ROADMAP, ARCHITECTURE, KNOWN_ISSUES,
  DECISIONS/ADR pertinente. Inventário criado/modificado/excluído.
- [x] Registrar HEAD/data/remoto verificados, mudanças não commitadas, testes reais.
- [x] Informar necessidade de revisão/commit manual e backup externo independente;
  encerrar sem iniciar Etapa 6.

## Registro de execução

2026-10-27: base limpa; origin/main consultado diretamente em 5eac9647458434a4d7dc87f6592c97627882b398.
Baseline `pwsh -NoProfile -File scripts/validate.ps1`: aprovado, 113 Rust + 15 PS.
Logs ignorados `.local/etapa-05/baseline.log`. Autorizações do anexo prevalecem
sobre gates genéricos de nova aprovação e sugestões de commit/worktree das skills.

2026-10-28: implementação concluída dentro da Etapa 5. Revisão independente somente
leitura identificou quatro problemas relevantes, reproduzidos e corrigidos com
regressões: foco, sincronização, escrita de snapshot e folha extrema. Inspeção em
200% também levou a recorte/recolhimento por altura das miniaturas. 51 testes GUI
novos; validação oficial final exit 0, 164 Rust + 15 PowerShell = 179 aprovados.
Release aprovado e PE Subsystem 2 verificado. Capturas pelo renderizador real, após
timeout da captura externa. Documentação/ADR-0007 e inventário consolidados no relatório.
Homologação completa pendente; usuário confirmou aparência/painéis informalmente.
Sem mutações Git ou início da Etapa 6. Revisão e commit manual são a próxima ação.
