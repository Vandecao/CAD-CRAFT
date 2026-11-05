# Etapa 7 — Biblioteca de símbolos e inserção gráfica

Data: 2026-10-29, America/Sao_Paulo. **Implementação, testes automatizados, build
Release e revisão independente concluídos. Pronta para revisão e commit manual.**
Homologação manual abrangente adiada pelo usuário até após a Etapa 15; não foi
solicitada nem declarada. Etapa 8 não iniciada. Especificação da Etapa 17 preservada.

## Escopo e rastreabilidade

Requisito integral em [ETAPA_07_BIBLIOTECA_SIMBOLOS](../spec/ETAPA_07_BIBLIOTECA_SIMBOLOS.md),
plano em [2026-10-29-etapa-07-simbolos](../plans/2026-10-29-etapa-07-simbolos.md)
e decisão em [ADR-0009](../../adr/0009-simbolos-vetoriais-e-insercao.md).
O roadmap anterior não atribuía outro escopo à Etapa 7. Sua atualização registra
o anexo autorizado, o estado implementado e o adiamento de homologação; preserva
as etapas futuras e a melhoria planejada de suavização visual da grade.

## Resultado funcional

A Ribbon abre a biblioteca no centro de inserção existente. Pesquisa por nome e
categoria filtra miniaturas; clique seleciona, botão ou duplo clique inicia a
ferramenta. Orientação, espelhos e associação opcional a uma entidade existente
podem ser definidos antes da inserção. No viewport, a prévia segue o cursor e o
snap; cada clique insere uma ocorrência. O modo permanece ativo para repetição
até Esc. Botões direito/central, perda de foco, modal ou troca de página/contexto
também cancelam sem inserir. O botão central mantém o pan previsto no editor.

Símbolos inseridos usam seleção simples/múltipla/retangular, hover, arraste,
movimento em grupo, rotação, espelhamento, comandos numéricos e menus existentes.
Propriedades mostram nome/categoria/definição/versão, ID da ocorrência, página,
alvo, transformação e portas transformadas. Aplicar transformação e Aplicar
aparência confirmam transações; aparência inclui visibilidade, camada e rótulo.

A biblioteca inicial contém dez desenhos originais: contato normalmente aberto,
contato normalmente fechado, bobina de relé/contator, motor, borne, botão de
comando, chave/interruptor, sensor, terra de proteção e alimentação. São exemplos
industriais simplificados, não um catálogo completo nem símbolos certificados.
O escopo público da IEC 60617 foi consultado nas páginas oficiais
[IEC Webstore](https://webstore.iec.ch/en/publication/2723) e
[IEC TC3](https://tc3.iec.ch/standard-as-database/); não foram copiados arquivos
proprietários, desenhos licenciados ou recursos EPLAN, nem declarada conformidade.

## Arquitetura, geometria e domínio

- SymbolDefinition é imutável e reutilizável: SymbolDefinitionId textual tipado
  estável, versão positiva, nome/categoria, metadados, orientações permitidas,
  primitivas locais e portas. Biblioteca embutida usa OnceLock/BTreeMap.
- SymbolInstance é o papel da própria Representation, que mantém UUID v7, página,
  transform e VisualProperties e acrescenta uma referência à definição. Não há
  coleção paralela de instâncias. Project mantém uma cópia por definição usada.
- Unassociated permite inserir sem alvo elétrico e só é aceito para símbolos.
  Não participa do índice por alvo nem relaciona ocorrências independentes.
  Associação existente é explícita e não duplica informações do domínio.
- Linha, polilinha, retângulo, círculo, arco, texto e âncora usam milímetros locais.
  Definição valida finitude, degenerações, portas duplicadas e overflow; limites
  locais são derivados uma vez e conferidos nas ocorrências. Portas têm ID local,
  identificação, posição, direção e papel opcional. Transformações aplicam rotação
  e espelhos também à direção das portas.
- Origem inferior esquerda, X à direita e Y para cima preservados. Grade lógica
  e snap continuam constantes em mm, inclusive com grade oculta ou zoom diferente.
  Não foi implementada a suavização futura da grade.
- Hit usa distância analítica aos traços, tolerância visual, portas e âncora de
  inserção; seleção retangular usa segmentos e interseções de curvas. Pintura
  compartilha geometria com miniaturas, faz descarte por visibilidade e recorte.
  Curvas têm tesselação dependente do zoom, erro alvo de 0,2 ponto lógico e teto
  de 2.048 partes. Texto usa caixa nominal e malha transformada, inclusive espelhos.
- Nenhum dispositivo, função, ConnectionPoint, conexão, rede ou condutor físico
  é criado/alterado implicitamente. Portas gráficas coincidentes não significam
  conexão. Domínio elétrico das etapas anteriores e Core sem GUI preservados.

## Prévia, histórico e persistência

InsertionDraft integra Interaction e guarda definição/página/alvo/transformação,
sem ID definitivo nem alteração de Project/histórico durante mouse move. Clique
confirmado passa pelo dispatcher comum e validação do Core. Biblioteca é criada
uma vez; resultados de filtro são cacheados; linhas são virtualizadas. Em painéis
compactos, controles e resultados rolam em conjunto para manter acesso aos itens.

History contém transações de transforms, inserção e aparência. Cada confirmação
efetiva cria uma entrada; no-op preserva Redo. Undo/Redo preservam IDs e associação.
Undo da inserção remove a ocorrência e retira a definição quando introduzida pela
transação e sem outro uso; Redo restaura ambas. Verificação de estado esperado
precede a alteração. Permanecem 100 transações/50.000 deltas, dirty conservador
e encerramento das pilhas por outras edições documentais/elétricas. Não há Undo
geral, exclusão genérica de instância ou histórico persistido.

Snapshot técnico embute definições usadas e referências das ocorrências. Defaults
vazios/opcionais mantêm compatibilidade com snapshots anteriores simulados. A
importação valida duplicações, referências ausentes, geometria, limites e regras
de orientação antes de substituir a sessão. Ordenação por IDs mantém serialização
determinística; seleção/preview/histórico não são gravados. A geometria pode ser
reconstituída sem depender da biblioteca instalada. Definição divergente com mesmo
ID é recusada; não há atualização automática ou versões simultâneas para esse ID.
Preferências, Backstage, exportação segura e limite técnico JSON de 8 MiB continuam.
Não foram introduzidos formato final, migrações físicas, autosave ou backup.

## Validação automática

Validação final executada após as correções, com sucesso:

| Comando/verificação | Resultado |
|---|---|
| `pwsh -NoProfile -File scripts/validate.ps1` | Aprovado; fmt, check, testes, clippy, testes operacionais e diff checks |
| `cargo fmt --check` | Aprovado |
| `cargo check --workspace --all-targets --locked` | Aprovado |
| `cargo test --workspace --locked` | 257 aprovados, zero falhas/ignorados |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Aprovado |
| `pwsh -NoProfile -File scripts/tests/check-repository.Tests.ps1` | 15 aprovados, zero falhas |
| `cargo build --release -p cad-gui --bin cad-gui --locked` | Aprovado |
| `git diff --check` e `git diff --cached --check` | Aprovados |

**Total: 272 testes aprovados = 257 Rust + 15 PowerShell; zero falhas/ignorados.**
Core 134, incluindo três doctests; GUI 123. Preservados os 236 anteriores, com
36 novos. Nenhum teste anterior removido/alterado, aviso ocultado ou check desabilitado.

| Novos testes | Quantidade | Comportamento exercitado |
|---|---:|---|
| Core `symbol_geometry` | 9 | Primitivas/limites, IDs, portas, orientações, geometria inválida, canonização e round-trip |
| Core `symbol_project` | 8 | Inserção atômica, identidade, associação, integridade elétrica, referências/versões e snapshots |
| GUI `symbol_history` | 5 | Inserção/prévia/cancelamento/repetição, no-op, mistura de estilo/transforms, Undo/Redo e IDs |
| GUI `symbol_geometry` | 6 | Hit de traço, curvas/arcos rotacionados, crossing, recorte/texto e crescimento do atlas |
| GUI `symbol_ui` | 6 | Eventos reais egui: biblioteca/Ribbon, inserção, snap oculto, cancelamento, seleção/movimento e rolagem compacta |
| GUI `symbol_scale` | 1 | 10.000 ocorrências, definições compartilhadas, transforms/Undo/Redo e isolamento de páginas |
| Unitário do navegador | 1 | Pesquisa/categoria e cache de resultados |

TDD/RED-GREEN registrado para histórico visual, geometria, biblioteca e regressões
de canonização angular, atlas de fonte e painel compacto. Testes de eventos egui
exercitam controles reais, além das verificações de domínio/sessão/serialização.
Logs locais ignorados em `.local/etapa-07/`, especialmente `validation-final.log`,
`release-final.log`, `real-events.log`, `text-atlas-red.log`, `compact-red.log`,
`compact-green.log` e `core-report.md`. Não são arquivos a versionar.

Executável local `target/release/cad-gui.exe`: **8.319.488 bytes**, PE Subsystem **2**
(Windows GUI); SHA-256:
`444CAE407795813F722949387EDFCD3F788154B9599056FA3BD55A18EC245CCA`.
Esse hash identifica o artefato desta compilação, não um futuro commit.

## Medição e inspeção visual técnica

Teste Debug local com 10.000 símbolos em cinco páginas, dez definições compartilhadas:

| Operação | Tempo observado |
|---|---:|
| Inserir 10.000 ocorrências | 278,9891 ms |
| 40 consultas de hit em página com 2.000 ocorrências | 374,5774 ms |
| Transformar 2.000 ocorrências | 29,1642 ms |

Fonte: `cargo test -p cad-gui --test symbol_scale --locked -- --nocapture`, log
`scale.log`. O teste também verifica histórico e preservação das outras páginas.
Medição proporcional, não benchmark de frame completo/pico de memória nem garantia
industrial. Hit continua linear na página; pintura ordena por camada/ID.

O exemplo render_qa recebeu modo symbols e executou a UI eframe real, sem gravar
projeto ou preferências. Capturas finais válidas e inspecionadas após a correção:

- `.local/etapa-07/symbols-final-1600.png`: 1600×900, escala lógica 1; biblioteca,
  símbolos com rotações/espelhos, seleção, propriedades e objetos antigos coexistem.
- `.local/etapa-07/symbols-final-1280-125.png`: 1280×720, escala lógica 1,25; layout
  compacto. A lista inicia abaixo dos controles e pode ser alcançada por rolagem,
  cujo comportamento é verificado pelo teste de eventos reais.

Capturas internas não abrangem decoração nativa nem homologam Windows 11, alto
contraste, múltiplos monitores ou DPI físico. Não houve homologação com o usuário.

## Revisão independente e correções

Revisores independentes internos inspecionaram Core e integração, sem alterar o
produto, executar Cargo em paralelo ou realizar operações Git modificadoras.
Pareceres locais: `core-review.md` e `integration-review.md` em `.local/etapa-07/`.
Conclusão: conformidade e qualidade aprovadas no escopo examinado, sem achado
importante aberto. Revisão não equivale a homologação manual ou prova exaustiva.

1. **P2 Core:** ângulo negativo muito próximo de zero podia virar 360 em regras e
   portas, divergindo de Transform2D e mudando no round-trip. Canonização exata
   compartilhada corrigiu o caso; dois testes RED/GREEN e revisão focada aprovaram.
2. **Regressão de pintura encontrada na implementação:** normalização precoce de
   UV corrompia texto quando o atlas crescia no mesmo frame. UVs permanecem em
   texels até a tesselação egui; reprodução e teste de regressão aprovados.
3. **Painel compacto detectado pela captura técnica:** controles fixos consumiam a
   área da lista. ScrollArea única para controles/resultados, preservando
   virtualização, corrigiu o acesso. Teste RED/GREEN com eventos reais de roda e
   revisão independente focada aprovados; capturas finais regeneradas.

## Limitações e pendências

- Biblioteca inicial limitada, sem certificação normativa, editor de definições,
  catálogo de fabricante ou atualização/versionamento múltiplo de definições.
- Associação escolhida antes da inserção; edição posterior dessa associação e
  exclusão genérica não fazem parte da etapa. Undo desfaz a inserção.
- Portas são geometria/identificação; editor de fios, roteamento, ERC, referências
  cruzadas e demais funções futuras não foram antecipados.
- AABB documental conservadora para curvas em rotação oblíqua; texto usa caixa
  nominal (largura 0,6 × altura × caracteres), não métrica tipográfica normativa.
- Limites anteriores de persistência/histórico, tema nativo e ENV-003 (reinício não
  confirmado) permanecem. Homologação em outros ambientes continua pendente.
- Dependências, manifests, lockfile, scripts, testes anteriores e especificação
  da Etapa 17 não foram alterados. Nenhum novo módulo vazio foi antecipado.

## Inventário de arquivos

**Criados (14):**

```text
crates/cad-craft/src/domain/symbol.rs
crates/cad-craft/src/domain/symbol_library.rs
crates/cad-craft/src/domain/project/symbols.rs
crates/cad-craft/tests/symbol_geometry.rs
crates/cad-craft/tests/symbol_project.rs
crates/cad-gui/src/symbol_view.rs
crates/cad-gui/tests/symbol_geometry.rs
crates/cad-gui/tests/symbol_history.rs
crates/cad-gui/tests/symbol_scale.rs
crates/cad-gui/tests/symbol_ui.rs
docs/adr/0009-simbolos-vetoriais-e-insercao.md
docs/internal/plans/2026-10-29-etapa-07-simbolos.md
docs/internal/reports/ETAPA_07_BIBLIOTECA_SIMBOLOS.md
docs/internal/spec/ETAPA_07_BIBLIOTECA_SIMBOLOS.md
```

**Modificados (27):**

```text
crates/cad-craft/src/document_index.rs
crates/cad-craft/src/domain/mod.rs
crates/cad-craft/src/domain/project.rs
crates/cad-craft/src/domain/project/representations.rs
crates/cad-craft/src/domain/project/snapshot.rs
crates/cad-craft/src/domain/representation.rs
crates/cad-craft/src/domain/transform.rs
crates/cad-gui/examples/render_qa.rs
crates/cad-gui/src/app.rs
crates/cad-gui/src/commands.rs
crates/cad-gui/src/editing.rs
crates/cad-gui/src/editor_view.rs
crates/cad-gui/src/history.rs
crates/cad-gui/src/hit_test.rs
crates/cad-gui/src/interaction.rs
crates/cad-gui/src/lib.rs
crates/cad-gui/src/panels/insertion.rs
crates/cad-gui/src/panels/properties.rs
crates/cad-gui/src/state.rs
crates/cad-gui/src/viewport.rs
docs/internal/memory/ARCHITECTURE.md
docs/internal/memory/CURRENT_STATE.md
docs/internal/memory/DECISIONS.md
docs/internal/memory/HANDOFF.md
docs/internal/memory/KNOWN_ISSUES.md
docs/internal/memory/ROADMAP.md
docs/internal/spec/CONTRATOS_DOCUMENTAIS_GUI.md
```

**Excluídos: nenhum.** Build, capturas, logs e pareceres locais permanecem em
`target/` e `.local/etapa-07/`, ignorados; regras de ignore não foram ampliadas.

## Git e próxima ação

Branch `main`; origin existente `https://github.com/Vandecao/CAD-CRAFT.git`.
`git rev-parse HEAD` e `git ls-remote --exit-code origin refs/heads/main`, consultados
em **2026-10-29**, retornaram `4a27a8fea6711bd589c544f242afc9c5481e4a89` (Etapa 6).
A árvore estava limpa ao iniciar. Toda Etapa 7 e este relatório estão **fora desse
HEAD**, sem staging. Não foi inventado SHA futuro nem inferida atualização remota
somente do tracking local.

Nenhum commit, push, fetch, pull, worktree ou outra mutação Git foi executado.
Etapa 7 requer **commit manual do usuário**. Push permanece manual conforme a
política, com próxima frequência prevista na Etapa 8; a igualdade observada entre
HEAD/remoto não protege estas alterações ainda locais. Backup externo independente
recomendado, não criado/verificado. Entrega encerrada sem iniciar a próxima etapa.
