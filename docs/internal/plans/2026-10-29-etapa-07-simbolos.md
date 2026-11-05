# Etapa 7 — biblioteca de símbolos e inserção

> Execução: habilidades writing-plans, TDD e subagent-driven-development,
> adaptadas à política obrigatória de Git manual. Nenhum commit/worktree.

**Objetivo:** escolher símbolos vetoriais, inseri-los e manipulá-los no editor
existente, com histórico e snapshot técnico compatível.
**Arquitetura:** definição imutável reutilizável no Core; instância é a própria
Representation, com referência opcional à definição. Biblioteca inicial única;
inserção é estado transitório de Interaction e usa Session/History existentes.
**Tecnologia:** Rust, Serde, egui/eframe já fixados; sem dependências novas.
**Spec:** `docs/internal/spec/ETAPA_07_BIBLIOTECA_SIMBOLOS.md`.

## Restrições e conciliação

- Autorizada somente Etapa 7; main existente, sem operações Git modificadoras.
- HEAD 4a27a8fea6711bd589c544f242afc9c5481e4a89, árvore limpa em 2026-10-29.
  Etapa 6 incorporada ao histórico. Roadmap não traz outro escopo para Etapa 7.
- Homologação manual abrangente adiada até após Etapa 15. Sem solicitação ao usuário.
- Milímetros, origem inferior esquerda, X direita/Y cima; grade lógica constante.
- Core independente da GUI; nenhum dispositivo, ponto lógico ou conexão criado
  implicitamente por inserir símbolos ou coincidir portas gráficas.
- Preservar 236 testes anteriores; formato final, fios/ERC/IA Etapa 17 fora do escopo.
- Decisões de implementação abaixo são do agente, sujeitas à revisão, dentro da
  autorização explícita do anexo. Não dependem de uma rodada de homologação.

## Desenho e alternativas

Um modelo paralelo de instâncias duplicaria índices, seleção e transformações;
foi descartado. Guardar apenas uma referência a biblioteca externa perderia a
geometria ao mudar a instalação; guardar definições usadas no Project torna o
snapshot reconstituível. Escolha: Representation ampliada, biblioteca imutável
embutida e cópia única da definição usada no documento. Definições possuem ID
textual tipado estável, versão, metadados, primitivas e portas gráficas próprias.
RepresentationId continua UUID v7. Um alvo explicitamente não associado permite
inserção gráfica sem inventar dispositivo físico; só símbolos admitem esse estado.

Inserção repetida permanece ativa após clique; Esc encerra. Seleção de entidade
existente no painel é opcional e explícita. Sem editor de definições nem exclusão
de instâncias nesta etapa (Undo da inserção remove a ocorrência). Transformações
e propriedades visuais editáveis usam o histórico; demais edições de projeto
continuam encerrando as pilhas. Biblioteca filtra por nome/categoria com cache;
miniaturas e viewport compartilham pintura. Curvas e texto devem acompanhar
rotação/espelhos e possuir limites geométricos validados.

## Foco da revisão

- Snapshot inválido não substitui sessão nem admite definição/porta ausente.
- Undo/Redo intercalado com transforms/propriedades conserva identidade e índices.
- Geometria vazia/extrema, arcos negativos e espelhos não causam panic/overflow.
- Cancelar/trocar página/foco/modal e navegar com pan/zoom não insere objetos.
- Pontos coincidentes e múltiplas representações do alvo não mudam conectividade.

## Tarefas e evidências

- [x] 1. Core e biblioteca (`domain/symbol*.rs`, `project/symbols.rs`, representação,
  índices e snapshot). Testes de definições, primitivas/limites, portas, referências,
  orientação, atomicidade, identidade, snapshots antigos/inválidos e conectividade.
  API consumida pela GUI: biblioteca estática; definições imutáveis consultáveis;
  criação de representação simbólica validada; Project registra definição usada.
- [x] 2. Histórico/sessão (`history.rs`, `state.rs`, `editing.rs`, `interaction.rs`).
  Transações de inserção e estilo; prévia sem ID/Project, repetição e cancelamento.
  Testes RED/GREEN de inserção/Undo/Redo, no-op, falha atômica e seleção removida.
- [x] 3. Geometria/pintura (`symbol_view.rs`, `hit_test.rs`, `viewport.rs`).
  Compartilhar primitivas transformadas e limites, recorte, hit de traços, seleção
  retangular. Testes de zoom, rotação/espelho, negativos e primitivas visíveis.
- [x] 4. UI (`panels/insertion.rs`, `properties.rs`, `commands.rs`, `app.rs`,
  `editor_view.rs`). Biblioteca por categorias/busca/miniatura, associação opcional,
  preview/snap/clique/Esc, Ribbon e propriedades, mantendo controles anteriores.
  Testes reais de eventos egui e integração, grade oculta e snapshots.
- [x] 5. Revisão independente somente leitura e correções com testes de regressão.
- [x] 6. Validação oficial `pwsh -NoProfile -File scripts/validate.ps1`, Release,
  `git diff --check`, medição proporcional e inspeção de captura técnica interna.
- [x] 7. ADR/contratos/memória/roadmap/relatório com contagens, arquivos, evidências,
  limitações, HEAD/remoto observados e necessidade de commit/backup manuais.

## Registro de execução

2026-10-29: contexto recuperado; anexo preservado; política Git consultada.
Nenhum conflito com roadmap; especificação Etapa 17 mantida exclusivamente futura.
Trabalho será acompanhado neste plano, sem commits automáticos nem etapa seguinte.

2026-10-29: tarefas 1–4 implementadas. Core: 17 testes novos; revisão independente
aprovou após correção RED/GREEN da canonização angular. GUI: histórico visual
RED2/GREEN2; geometria RED5/GREEN5; atlas de texto RED1/GREEN1; biblioteca RED1/GREEN1.
Sessão e eventos reais egui exercitados (inserção, repetição, cancelamento, snap
oculto, múltipla seleção/arraste, propriedades e transformações). Logs locais em
.local/etapa-07.

2026-10-29: revisões Core e integração aprovadas, sem achado importante aberto.
Captura compacta detectou recorte da lista; teste RED seguido de correção da
rolagem e GREEN dos seis testes UI; revisão focada aprovada. Validação global
final: 257 Rust +15 PowerShell =272, zero falhas/ignorados. Release aprovado.
Capturas finais 1600×900 e 1280×720/1,25 inspecionadas. Medição de 10.000 símbolos
distribuídos em cinco páginas concluída; resultados e limites no relatório.

2026-10-29: ADR-0009, contratos, memória e relatório concluídos. Inventário final:
14 arquivos criados, 27 modificados, nenhum excluído. HEAD/remoto consultados
continuam em 4a27a8fea6711bd589c544f242afc9c5481e4a89; entrega fora desse HEAD.
Commit manual da Etapa 7 e backup independente pendentes; sem mutação Git,
solicitação de homologação manual ou início da Etapa 8.
