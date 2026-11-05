# ADR-0007 — Interface desktop e sessão documental

Data: 2026-10-28. Estado: implementado e testado na Etapa 5; homologação parcial
confirmada, incluindo barra escura no Windows 10 local. Revisão completa pendente.
Requisito: `docs/internal/spec/ETAPA_05_INTERFACE_GRAFICA_EGUI.md`.

## Contexto

O usuário autorizou a interface egui/eframe inspirada na organização visual das
referências fornecidas, consumindo as APIs existentes sem duplicar o modelo elétrico.
Persistência final, símbolos, editor multifilar e Undo/Redo continuam futuros.

## Decisões

- `cad-gui` usa eframe/egui 0.33.3, backend Glow e fontes embutidas. A versão exata
  e Cargo.lock tornam a entrega reproduzível. Core continua sem dependência de GUI.
- `CadApp` coordena comandos e painéis. `Session` mantém um Project, seleção por
  IDs, abas e câmeras transitórias; `PageIndex` e árvore são apresentações derivadas,
  invalidadas por revisão/filtro. Nenhum modelo elétrico paralelo é criado.
- `Camera` transforma milímetros cartesianos em pontos lógicos egui. Somente
  essa fronteira inverte Y. Layout determina viewport; folha e conteúdo não o
  redimensionam. Recorte em f64 antecede a conversão de limites para f32.
- Câmera por aba, zoom ancorado no cursor, pan, grade adaptativa alinhada à origem
  e snap visual. Desenho de representações usa limites/âncoras reais, sem símbolos.
- Painéis acoplados a regiões fixas, divisores, ocultação, recolhimento, pin e
  restauração. Flutuação e rearranjo arbitrário são adiados conforme prioridade do anexo.
- Registro central de comandos define contexto, metadados e estados futuros
  indisponíveis. A execução revalida o contexto; não há comandos fictícios.
- Preferências visuais versionadas/saneadas vão ao storage local do eframe;
  sessão, entidades, abas e favoritos não são salvos automaticamente.
- Snapshot técnico JSON é intercâmbio estrutural experimental limitado a 8 MiB.
  Importar valida antes de substituir a sessão. Exportar prepara arquivo temporário
  no mesmo diretório, sincroniza e só então publica; sobrescrita exige confirmação.
  Isto não implementa formato final, migrações, histórico ou backup.
- Release Windows usa subsistema GUI. Debug mantém console diagnóstico.

## Consequências e limites

As próximas etapas podem acrescentar comandos e renderização preservando a fronteira
Core/GUI. Existe dependência de driver OpenGL compatível; execução local não homologa
Windows 10, GPUs diversas, acessibilidade nem DPI nativo de múltiplos monitores.
Escala egui foi exercitada automaticamente, sem alterar a configuração do Windows.

Seleção gráfica é teste dos limites locais transformados ou da proximidade da âncora;
não é hit testing de símbolo. Snap não move entidades. Ordenação e transformação
de representações visíveis ainda ocorrem por frame; grandes desenhos precisam de
medição antes de um cache espacial. Recorte/virtualização limitam a pintura e a
grade, mas não constituem garantia de capacidade industrial.

Faixa da câmera: centro até ±10¹² mm; escala 10⁻⁴ a 10⁴ pontos/mm.
100% corresponde a 96/25,4 pontos/mm; não é escala física de impressão.
Homologação completa permanece pendente. Git, sincronização e backup são exclusivamente
manuais, conforme ADR-0002. Evidências no relatório da Etapa 5.

## Complemento — correções visuais da segunda rodada

- Grade visível passa a círculos de 0,8 ponto lógico ancorados em mm; densidade
  adaptativa por potências de dois, mínimo visual 8 pontos, teto 12.000. Snap lógico
  independente; miniaturas sem grade. Não há alteração de geometria ou Core.
- Contraste inicial é estilo local, incluindo texto auxiliar e estados de botões;
  opacidade desabilitada de 0,85 sobre canvas claro. Scroll preserva acesso às ações.
- `ribbon_collapsed` é a fonte única: sem recolhimento automático por altura;
  duplo clique só na guia ativa, botão explícito Expandir e Ctrl+F1. Altura definida
  por estado impede recorte herdado; scroll horizontal por guia. Backstage preservado.
- Decoração nativa mantida; API segura egui SetTheme solicita Dark. Reaplica ao
  recuperar foco/mudar tema observado, mitigando reset interno do winit 0.30.13.
  Sem FFI própria/dependência nova. Backend tem mecanismos Windows específicos;
  não equivale a afirmar uso interno exclusivo do DWM documentado.
- Windows 10 Pro 22H2 build 19045.6466: usuário confirmou barra escura e controles
  legíveis. Windows 11/alto contraste/DPI nativo e WM_SETTINGCHANGE sem evento/foco
  permanecem limites de homologação. Captura oficial falha por timeout; nenhuma
  imagem desta rodada obtida. AX e confirmação do usuário não são screenshot.
