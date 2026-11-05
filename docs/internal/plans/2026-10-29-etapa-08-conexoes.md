# Etapa 8 — plano de implementação

> Execução: habilidades writing-plans, test-driven-development e subagent-driven-development. A autorização integral do anexo cobre as decisões de implementação; não solicitar homologação. Git exclusivamente de leitura, sem worktrees ou commits.

**Objetivo:** editar esquemas multifilares com portas vinculadas, rotas ortogonais, junções explícitas, histórico e snapshots válidos.

**Arquitetura:** preservar Connection/ConnectionPoint e redes derivadas. Complementar Project com vínculos de portas e rotas documentais associadas a conexões existentes. Junções estabelecem conectividade explícita com arestas do mesmo grafo, sem dispositivos físicos artificiais. Prévia pertence apenas à GUI. Operações confirmadas retornam deltas reversíveis; nenhum snapshot integral por frame.

**Tecnologias:** Rust, serde, uuid v7, egui/eframe existentes; nenhuma dependência nova prevista.

**Especificação:** `docs/internal/spec/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md` (anexo integral).

## Restrições globais

- Somente Etapa 8; Etapas 9–15 serão propostas, Etapa 17 preservada e não iniciada.
- mm, origem inferior esquerda, X direita/Y cima; snap lógico independente de zoom/DPI/grade visível.
- Preservar 272 testes anteriores; não confundir revisão automática com homologação.
- Portas sem associação não podem produzir conectividade. Condutores físicos nunca são criados implicitamente.
- Cruzamentos e coincidência geométrica não criam redes. Junção somente por confirmação explícita.
- Branch main e origin existentes; commit/push/backup somente manuais.

## Foco de revisão

1. Portas de ocorrências diferentes apontando ao mesmo endpoint: impedir autoarestas e duplicatas.
2. Remover/mover ocorrência, página ou conexão referenciada: não deixar vínculos pendentes.
3. Erro em operação composta e replay fora do estado esperado: preservar agregado e pilhas.
4. Movimento simultâneo de símbolos, junções e rotas: manter endpoints por identidade e geometria válida.
5. Snap de segmento em cruzamento: destacar candidato determinístico, nunca unir redes por proximidade.

## Tarefa 1 — Core e contratos documentais

Arquivos: novos `domain/connection_drawing.rs`, `domain/project/connection_drawing.rs`, testes `connection_drawing*.rs`; adaptar registros, snapshot, integridade, transforms e remoções pertinentes.

Interfaces planejadas: `PortRef` (RepresentationId + String), `ConnectionRoute` (identidade documental, ConnectionId, página, portas, pontos em mm), `ConnectionJunction` (página, posição, rotas e arestas explícitas), `ConnectionDrawingEdit` e `ConnectionDrawingDelta`. Métodos Project para consulta de portas/vínculos/rotas/junções e aplicar/reverter deltas. Nomes finais publicados pelo implementador antes da integração.

- [x] Escrever testes e observar RED: vínculo inválido, rota ortogonal, criação/duplicata, cruzamento isolado, junção T/múltipla, remoção atômica, replay, movimento e serialização.
- [x] Implementar operações validadas e complementar snapshots com defaults; preservar APIs anteriores.
- [x] Executar testes Core e revisar invariantes/compatibilidade.

## Tarefa 2 — Geometria e interação gráfica

Arquivos: novos `cad-gui/src/connection_editor.rs`, `connection_view.rs`, testes `connection_editor.rs`; integrar `interaction.rs`, `editor_view.rs`, `viewport.rs`, `hit_test.rs`.

Interfaces: rascunho sem IDs definitivos; rota em L horizontal/vertical alternável, waypoints; snap com prioridade porta/junção/vértice/segmento/grade; hit por distância a segmentos em pontos lógicos. Pintura recortada, destaque de portas e redes, cruzamentos sem ponto e junções com ponto.

- [x] Testes RED para snap/zoom/grade oculta/negativos, rota L, cancelamento, hit e prévia sem mutação.
- [x] Implementar ferramenta mouse, edição de rota, segmento e vértices; cancelar em Esc/foco/modal/pan/página.
- [x] Validar interação egui e cenários densos/adversariais com milhares de rotas.

## Tarefa 3 — Histórico, comandos e propriedades

Arquivos: `history.rs`, `state.rs`, `editing.rs`, `commands.rs`, `app.rs`, `panels/properties.rs`, `demo.rs`; testes `connection_history.rs`, `connection_ui.rs`.

Interfaces: dispatcher único `GraphicEdit`/`CommandId`; History guarda deltas Core e restaura IDs/rotas/topologia. Inspetor associa portas explicitamente a pontos compatíveis, consulta endpoints/rede/condutores realmente existentes e edita geometria validada. Demonstração com pontos existentes associados, sem conexão presumida.

- [x] Testes RED para criação/exclusão/rotas/junções/transforms/Undo/Redo/no-op/erro atômico.
- [x] Implementar comandos na Ribbon e menus existentes, propriedades, atalhos e destaque de rede derivado por revisão.
- [x] Revisar e testar integração, inclusive snapshots inválidos sem substituir sessão.

## Tarefa 4 — Verificação, revisão e documentação

- [x] Revisão independente do Core e integração; corrigir achados relevantes com regressões.
- [x] Executar `pwsh -NoProfile -File scripts/validate.ps1`, build Release locked, diff check e status somente de leitura.
- [x] Inspecionar capturas internas em duas escalas; registrar limitações de DPI físico/homologação.
- [x] Atualizar memória, arquitetura, decisões, roadmap e ADR; relatório com contagem real, evidências, inventário, SHA observado e pendências manuais.

## Registro de execução

- 2026-10-29: árvore inicial limpa; main, HEAD `6bfa4c24e3b3a39642cea1e387a3a7f502835828`; Etapa 7 commitada. Tracking local ahead 1 não confirma remoto atual. Referências locais: 29 imagens e 2 vídeos encontrados; observações visuais serão registradas somente quando examinadas.
- Decisão: executar no checkout autorizado, sem scripts de habilidades que alterem Git/ignore ou criem worktree. Artefatos temporários em `.local/etapa-08/`, já ignorada. Não executar commits previstos genericamente pelas habilidades.

- 2026-10-30: Core/GUI integrados; regressões RED/GREEN para invariantes e achados de revisão. APIs finais em ADR-0010 e contratos documentais. Demonstração separada em connection_demo.rs preserva demo.rs anterior.
- Validação oficial encerrada com código 0: 339 testes (324 Rust + 15 PowerShell), 67 novos e 272 preservados; fmt/check/clippy/diff aprovados. Release locked aprovado, PE Subsystem 2. Revisão independente final sem P1/P2 funcional aberto.
- Framebuffers reais inspecionados em 1600×900/100% e 1280×720/125%; não equivalem a DPI nativo ou homologação. Ensaios de 1.001 rotas Core e 1.000 rotas GUI medidos; cinco frames CPU+tesselação no cenário GUI denso levaram ~1,992 s, custo elevado registrado.
- Limites de implementação: endpoints de rota são portas; junções unem rotas existentes, sem ramo livre porta→junção. Ângulos são dobras; pontes decorativas, automação avançada, target tracking, potenciais/interrupções/ERC e persistência final não foram implementados. Ver relatório e KNOWN_ISSUES.
- Documentação/memória/roadmap e inventário final consolidados. HEAD observado 6bfa4c24e3b3a39642cea1e387a3a7f502835828; remoto consultado sem fetch em 4a27a8fea6711bd589c544f242afc9c5481e4a89. Toda a Etapa 8 fora do HEAD. Commit + push e backup permanecem manuais. Nenhuma Etapa 9/17 iniciada.
