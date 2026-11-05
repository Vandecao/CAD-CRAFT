# Etapa 5 — correções visuais, segunda rodada

> Execução direta nesta sessão com superpowers:executing-plans, diagnóstico,
> testes de regressão e uma revisão independente final. AGENTS prevalece:
> sem worktree, staging, commit, push ou qualquer mutação Git.

**Objetivo:** corrigir grade, contraste inicial, título nativo e Ribbon.
**Spec:** `docs/internal/spec/ETAPA_05_CORRECOES_VISUAIS_RODADA_2.md`, anexo integral.
**Stack:** Rust/egui/eframe 0.33.3 existentes. Core e Backstage preservados.
**Base observada:** main/HEAD 5eac9647458434a4d7dc87f6592c97627882b398;
entrega anterior continua não commitada. Baseline esperado: 179 testes.

## Projeto da correção

Reutilizar câmera, comandos e preferências; alterar somente a apresentação afetada.
Grade: pontos em interseções documentais, raio 0,8 ponto lógico, passo visual em
múltiplos de dois, distância mínima de 8 pontos e orçamento de 12.000 pontos.
Snap e espaçamento lógico permanecem independentes. Miniaturas continuam sem grade.
Tela inicial: estilo local para fundo claro, com texto escuro em rótulos e texto
claro em botões escuros; estados visuais próprios, sem mudar o tema de outros painéis.
Ribbon: reproduzir antes de corrigir; preferência de recolhimento como única fonte
de verdade, sem desaparecimento por altura e sem duplo clique propagado entre guias.
Rolagem horizontal por guia e faixa de comandos com altura estável.
Título: priorizar a API segura pública egui/eframe que solicita tema nativo, mantendo
decoração/controles. Avaliar DWM documentado e o comportamento real do backend.

Alternativas descartadas: grade de linhas/cruzes (contrariam anexo), recolhimento
automático (contraria comportamento requerido), barra personalizada e FFI própria
(desnecessárias se API segura funcionar). Não adicionar dependências de apresentação.
Decisões de implementação são do agente dentro da autorização explícita do anexo.

## Foco da revisão

1. Densidade extrema, coordenadas negativas e grandes centros: grade finita, limitada
   e alinhada, snap na resolução original.
2. Trocas rápidas entre guias, duplo clique e Ctrl+F1: conteúdo real permanece visível
   ou recolhido somente por ação explícita, sem estado cruzado de scroll/ID.
3. Altura baixa/DPI alto: Ribbon, documentos e tela inicial acessíveis sem sobreposição.
4. Retorno do Backstage: guia, recolhimento, câmera, seleção, abas e painéis preservados.
5. Windows 10/11 e alto contraste: decoração nativa mantida, limitações documentadas,
   evidência da barra nativa distinta da captura do framebuffer egui.

## Tarefas

### 1. Diagnóstico e regressões
- [x] Executar baseline e registrar hashes dos arquivos anteriores para inventário.
- [x] Reproduzir grade/contraste em saída de pintura e Ribbon com eventos/resize reais.
- [x] Auditar caminho egui → winit → Windows e solicitar captura nativa real.

### 2. Grade de pontos
Arquivos: camera.rs, viewport.rs, theme.rs; novos testes grid_points.rs.
Interface: grid_points(&Camera, Rect, f64) -> Vec<Pos2>, sem estado elétrico.
- [x] Testes RED de posição/espaçamento/negativos/pan/zoom, limite, culling, DPI,
  snap independente e saída de desenho sem linhas da grade.
- [x] Implementar geração limitada visível e pintura circular; verificar GREEN.
- [x] Preservar testes prévios e contrato da câmera.

### 3. Contraste e título
Arquivos: theme.rs, viewport.rs; testes de pintura/contraste e comandos de viewport.
- [x] Reproduzir contraste baixo e ausência de solicitação de tema nativo.
- [x] Corrigir estilo local da tela inicial, incluindo estados e espaço pequeno.
- [x] Solicitar tema escuro pela API pública segura; não introduzir unsafe.
- [x] Verificar testes e capturas; registrar versão Windows e limites da evidência.

### 4. Ribbon estável
Arquivos: ribbon.rs, app.rs somente se necessário; testes de eventos/layout.
- [x] Reproduzir desaparecimento com altura e troca rápida de guias.
- [x] Corrigir fontes confirmadas; preservar recolhimento explícito e indicar estado.
- [x] Repetir dez guias e testar Backstage, documentos, resize, escalas e preferências.

### 5. Verificação e entrega
- [x] Revisão independente somente leitura; corrigir achados relevantes com regressões.
- [x] scripts/validate.ps1, Release --locked, PE e diagnóstico offline.
- [ ] Capturas reais A–J em .local/etapa-05/; nativa E somente se obtida de fato.
- [x] Atualizar relatório com seção exigida, memória e ADR pertinente; contagens reais,
  causas confirmadas, inventário desta rodada e limitações de homologação.
- [x] Informar revisão/commit manual, nenhum push e nenhuma Etapa 6.

## Registro

2026-10-28: auditoria inicial realizada. Anexo confirma homologação manual parcial
do Release e aprovação explícita do Backstage; quatro correções ainda pendentes.
Windows local: Windows 10 Pro 22H2, build 19045.6466 (registro do sistema).
Ruling: execução integral já autorizada pelo usuário; não criar gates adicionais
de aprovação de design/commit das skills. Plano permanece em docs/plans conforme AGENTS.

2026-10-28, encerramento: implementação, revisão independente, validação oficial
e Release concluídos. 196 testes aprovados (181 Rust + 15 PowerShell), zero falhas/
ignorados; 179 anteriores preservados. Hashes confirmam Core/Backstage inalterados.
Correção adicional revisada: reaplicar tema ao foco/tema observado; limite interno
WM_SETTINGCHANGE sem evento/foco registrado. Usuário confirmou título escuro e
controles legíveis no Windows 10 local. Não homologou todos os itens da rodada.

Única tarefa de evidência bloqueada: capturas A–J. Computer Use oficial disponível,
janela acessível e Release aberto, mas screenshot falhou por timeout inclusive após
reseleção. Usuário restringiu inspeção a essa ferramenta; não usar fallback interno
egui/PowerShell. Capturas antigas não comprovam a correção. Continuar essa tarefa
somente quando captura oficial funcionar ou o usuário modificar expressamente a
restrição. Nenhum commit/push/worktree; Etapa 6 não iniciada.
