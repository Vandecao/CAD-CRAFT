# Etapa 6 — plano de implementação

> Execução nativa nesta sessão com as skills writing-plans, executing-plans,
> test-driven-development e revisão independente final. Autorização de execução
> integral/autônoma no anexo prevalece sobre perguntas de aprovação das skills.
> Sem worktree, staging, commit, push ou qualquer mutação Git.

**Objetivo:** editar representações existentes sem alterar o modelo elétrico.
**Arquitetura:** Project continua fonte única. Core recebe lote atômico de
transformações; GUI guarda seleção por IDs, gesto transitório e histórico de deltas.
**Tecnologias:** Rust, egui/eframe 0.33.3 existentes; nenhuma dependência nova.
**Spec:** docs/internal/spec/ETAPA_06_EDITOR_GRAFICO_INTERATIVO.md (anexo integral).
**Base:** main, HEAD 5a5526d4ef4b093222218a9194450a3974225f66 em 2026-10-28;
árvore limpa na retomada. Etapa 5 homologada visualmente conforme anexo atual.
Roadmap não define divisão conflitante com Etapa 7, que permanece não autorizada.

## Restrições globais

Milímetros, X direita/Y cima, rotação CCW; GUI independente dos dados elétricos.
Manter IDs, formato Serde, Backstage, Ribbon, grade lógica e os 196 testes anteriores.
Não implementar símbolos, roteamento, conectividade implícita ou persistência final.
Homologação da Etapa 6 depende do usuário. Retenção de histórico em memória.

## Decisões de implementação

- Lote no Core valida todos os IDs/limites antes de escrever. Não copiar Project.
- Histórico guarda antes/depois de cada transformação (100 operações, no máximo
  50.000 deltas retidos). Lote maior que a capacidade é recusado antes da mutação.
- Alterações documentais fora do histórico invalidam o histórico para evitar
  restaurar referências incompatíveis. Câmera/seleção não o invalidam.
- Rotação individual em torno da âncora de cada representação; espelho horizontal/
  vertical nos eixos da página em torno da âncora. Inspetor expõe flags locais.
- Movimento coletivo usa a âncora do objeto arrastado como base de snap; único
  delta para todo o conjunto. Seleção simples em objeto já selecionado conserva
  conjunto durante arraste; clique sem arraste seleciona somente aquele objeto.
- Geometria atual é retângulo local transformado ou âncora; tolerância 6 pontos
  lógicos, ordem (camada, ID), topo vence. Seleção cruzada usa interseção geométrica.
- Clique direito em não selecionado substitui seleção; em selecionado mantém
  conjunto; vazio abre ações da página sem alterar o conjunto.
- Troca/fechamento de página, perda de foco, modal e Esc cancelam gestos.
  Botão central prevalece sobre gesto esquerdo; zoom mantém âncora no cursor.

## Foco da revisão

1. Lote com último ID inválido/overflow não pode alterar os anteriores.
2. Soltura fora do viewport, perda de foco e Ctrl durante gesto não podem vazar prévia.
3. Retângulo rotacionado não pode ser selecionado só pela AABB vazia.
4. Undo com campo textual focado deve preservar edição textual e câmera.
5. Seleção/página trocada e edição numérica não podem reutilizar rascunho antigo.

## Tarefas e verificações

- [x] 1. Core e histórico: testes RED em document_transactions.rs e editor_history.rs;
  implementar Project::transform_representations(&[(RepresentationId, Transform2D)]),
  history.rs, gráfico em editing.rs/state.rs. Testar atomicidade, integridade,
  no-op, Undo/Redo, divergência, retenção, snapshots e compatibilidade.
  Verificar: cargo test -p cad-craft --test document_transactions --locked;
  cargo test -p cad-gui --test editor_history --locked (todos aprovados).
- [x] 2. Interação: testes RED em editor_interaction.rs; implementar interaction.rs
  (SelectionState, Interaction, DragSession) e hit_test.rs. Interfaces consomem
  Project imutável, PageId, Camera, pontos; produzem IDs ou lote de transforms.
  Testar clique/Ctrl/vazio, hover, janela/cruzamento, tolerância/zoom/pan/DPI,
  prévia/cancelamento/snap de grupo/movimento nulo. Rodar a suíte específica.
- [x] 3. UI: testes RED por eventos egui em editor_ui.rs; ligar viewport, dispatcher,
  comandos/Ribbon, propriedades, menus e atalhos às APIs das tarefas 1–2.
  Testar entradas reais, estado contextual, foco textual, troca de página,
  isolamento de câmera, cancelamento e grupo. Preservar testes da Etapa 5.
- [x] 4. QA/revisão: executar validação oficial, Release e captura real via
  render_qa; inspecionar imagens, medir cenário proporcional, revisão independente
  somente leitura e corrigir achados com regressões.
- [x] 5. Entrega: atualizar memória/roadmap/ADR/contratos, relatório técnico com
  inventário, contagem real e roteiro A–L; consultar Git local/remoto; não versionar.

## Registro de execução

Pre-flight: tarefa 2 produz lote consumido pela 1; tarefa 3 só confirma lote via
dispatcher e usa mesma consulta geométrica na pintura/hit testing. Sem conflito.
Ruling: execução sem pausa de aprovação e sem commits/worktree — autorização
autônoma e política explícita do anexo/AGENTS substituem defaults das skills.

Execução concluída: Core 4 testes novos; GUI 36 novos. APIs ausentes comprovadas
nos logs RED; regressões de foco/rascunho/seleção reproduzidas e corrigidas.
Ruling: comparação fracionária usa epsilon f64; localização do menu em testes foi
restrita ao viewport para não confundir seu comando com o do inspetor.
Ruling: selection_revision invalida rascunhos mesmo com painel oculto em ida/volta;
seleção substituída cancela gesto. Nova API Core só muda transforms.
Ruling: reutilizado helper physical_common sem editar o original para provar
preservação de fiação real; escala ampliada para 5.000 âncoras e 5.000 retângulos.
Validação oficial final exit 0: 236 testes (221 Rust + 15 PowerShell), sem falhas/
ignorados; Release exit 0. Revisão independente concluída após correção do P2.
Capturas reais válidas e memória/relatório/ADR atualizados. Sem commit/push/worktree.
