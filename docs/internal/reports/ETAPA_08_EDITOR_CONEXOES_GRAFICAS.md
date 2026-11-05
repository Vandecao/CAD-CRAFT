# Etapa 8 — editor gráfico de conexões elétricas

Data de fechamento: 2026-10-30, America/Sao_Paulo.
Estado: implementação, validação automática, Release e revisão independente concluídos.
Homologação manual abrangente adiada até após a Etapa 15.

## Autorização e base

Anexo integral preservado em `docs/internal/spec/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md`.
Plano em `docs/internal/plans/2026-10-29-etapa-08-conexoes.md`; decisão ADR-0010.
Branch `main`, origin `https://github.com/Vandecao/CAD-CRAFT.git`, preservados.

`git rev-parse HEAD`, consultado em 2026-10-30, retornou
`6bfa4c24e3b3a39642cea1e387a3a7f502835828`: Etapa 7 commitada manualmente.
Árvore inicial limpa; a antiga memória de commit da Etapa 7 pendente foi corrigida.
`git ls-remote --exit-code origin refs/heads/main`, na mesma data, retornou
`4a27a8fea6711bd589c544f242afc9c5481e4a89` (Etapa 6), sem fetch/sincronização.
A Etapa 7 está localmente um commit à frente desse remoto observado; toda a Etapa 8
e esta documentação ainda não fazem parte do HEAD. Não há SHA futuro presumido.

Foram lidos AGENTS/memória/plano/contratos, Core, histórico, símbolos e viewport.
A auditoria independente leu também ADRs, relatórios 4–7 e especificação 7.
Não houve instalação de dependências, agentes externos, cópia de ativos EPLAN,
mudança de conexão/remoto ou antecipação de etapas posteriores.

## Implementação

**Core:** PortRef identifica ocorrência e porta local, com associação explícita a
ConnectionPoint existente e compatível. O inspetor consulta os candidatos válidos;
símbolo sem entidade associada não produz conexão lógica. ConnectionRoute mantém
RepresentationId compartilhado com a ocorrência documental, ConnectionId, página,
portas e polilinha ortogonal em mm. Segmentos nulos/diagonais, endpoints inválidos,
autoarestas e conexões duplicadas são recusados.

Junções têm identidade UUID v7, posição e participantes explícitos. Criar junção
exige incidência em todas as rotas e cria arestas auxiliares de posse exclusiva no
mesmo grafo. T, múltiplas ramificações, extremidades, coincidências e cruzamentos
têm testes; interseção sem operação de junção mantém redes separadas. Nenhum ponto
funcional, dispositivo ou condutor físico fictício é criado para representar a marca.

ConnectionDrawingEdit/Delta cobrem vínculos, criação/exclusão, rota, junções e
transforms. Validação do lote, publicação de índices e rollback impedem mutação
parcial. O delta guarda somente entidades afetadas. Replay confere estado esperado,
preserva IDs e restaura geometria/topologia. APIs antigas que deixariam referências
pendentes são protegidas; conexões com dependências físicas/documentais não são
removidas silenciosamente.

Mover símbolos recalcula seus terminais gráficos. Translação comum conserva o
percurso; junções acompanham o grupo quando possível. Alterações que invalidariam
incidência são recusadas integralmente. O método puro points_with_endpoints é
compartilhado entre a prévia e a confirmação.

**Interface:** a guia Conexões oferece inserir conexão/junção, editar rota,
adicionar/remover vértice, excluir, destacar rede, convenção da marca e exemplo
multifilar. Inserir ângulo inicia a mesma ferramenta ortogonal na guia Inserir.
Comandos usam registro/dispatcher existentes; Ribbon e Backstage foram preservados.

Fluxo: ativar Conexão, clicar porta associada, escolher pontos intermediários e
clicar porta de destino. Espaço alterna a rota em L. Prévia tracejada não altera
Project, redes, histórico ou IDs persistentes. Esc/foco/modal/pan/página cancelam.
Sugestões de alinhamento são visuais; só um destino válido confirmado cria conexão.

Snap usa porta elétrica, junção, vértice, segmento e grade, com tolerância em pontos
lógicos e desempate determinístico. Uma grade de 4 mm mantém resolução de 4 mm em
qualquer zoom, inclusive oculta. Câmera/pan não alteram o documento. Seleção e hit
usam distância ao traço; vértices e segmentos podem ser arrastados com endpoints
preservados. Símbolos são desenhados/atingidos acima de rotas na mesma camada.

Junções têm seleção própria, inclusive sobre uma porta, e marca configurável
ponto cheio/anel. Cruzamentos sem junção não têm marca. O inspetor mostra IDs,
origem/destino, proprietários, rede derivada, página e condutores realmente
associados. Identificação gráfica e rota passam por comandos validados, sem
parâmetros de engenharia inventados. Ctrl+Z/Y respeita foco em campos textuais.

**Histórico/persistência:** uma confirmação produz uma transação, inclusive
operações compostas e movimento de grupo. No-op preserva Redo. Limites anteriores
de 100 operações/50.000 deltas permanecem; não há histórico persistido ou Undo geral.
Snapshots embutem vínculos/rotas/junções, aceitam defaults antigos e validam
referências/IDs/geometria antes de substituir uma sessão. JSON técnico até 8 MiB;
sem formato final `.cadcraft`, autosave ou migrações físicas.

## Testes e correções

Os 272 testes anteriores foram preservados. Nenhum teste anterior foi excluído ou
desabilitado. Novos testes exercitam o Core real, o histórico e eventos egui reais.

| Requisitos | Evidência principal |
|---|---|
| Portas, associação, sem alvo, endpoints, duplicatas, IDs | `cad-craft/tests/connection_drawing.rs` |
| Ortogonalidade, negativos, zero/diagonal, vértice/segmento, precisão extrema | `cad-gui/tests/connection_geometry.rs`, testes Core |
| Cruzamentos, T, múltiplas junções, coincidência, remoções, movimento, snapshot | Testes Core de desenho e serialização |
| Criar/excluir/editar, Undo/Redo, atomicidade, grupo, redes/físicos separados | `cad-gui/tests/connection_history.rs` |
| Snap, grade oculta, zoom/pan, cancelamento, alinhamento | `cad-gui/tests/connection_editor.rs` e testes anteriores de câmera |
| Mouse real, menus, seleção, arraste, foco textual, duas escalas/DPI emulado | `cad-gui/tests/connection_ui.rs` |
| Prévia/commit e marcador de junção no grupo | Testes unitários em `connection_view.rs` |
| Milhares de rotas, cache e custo de operações | `connection_drawing_scale.rs`, `connection_scale.rs` |

TDD registrado nos logs ignorados em `.local/etapa-08/`. Novas APIs primeiro
produziram erros de compilação esperados; os seguintes defeitos também tiveram
falhas comportamentais observadas antes da correção:

- PortRef vazio aceito por Serde e junção duplicada na mesma posição.
- Geometria com cursor extremo e translação finita que colapsa segmentos.
- Seleção antiga de junção sobrevivendo ao clique em outro objeto.
- Junção sobre porta não selecionável pela prioridade do snap.
- Drag de rota interceptando símbolo e Ctrl+arraste alterando a rota.
- Grupo contendo dois símbolos e sua rota recusando confirmação.
- Prévia usando reparo diferente do commit e marcador de junção ficando parado.

A expectativa inicial do novo teste de grupo foi corrigida para o contrato
preexistente: âncora Y=110, deslocamento do cursor +8 e snap de 4 mm resulta Y=120,
delta comum +10. Não foi alterada a regra de snap para satisfazer o teste.

## Revisões

A revisão independente Core examinou atomicidade, índices, replay, posse de
arestas, remoções, movimento e snapshots. Executou 26 casos em harness isolado
(22 copiados + 4 próprios); nenhum defeito funcional concreto confirmado. Os
quatro casos locais não entram na contagem oficial versionável.

A revisão independente GUI identificou os problemas de seleção, arbitragem e
prévia descritos acima. O principal confirmou e corrigiu cada um com regressões
do produto. A revisão incremental final confirmou as correções GUI-REV-01–04;
nenhum achado funcional P1/P2 permanece aberto. O harness recompilado aprovou
sete casos, dos quais cinco independentes e dois copiados. São evidências extras,
não somadas à contagem oficial. A interrupção temporária por limite de uso foi
superada e não ficou como pendência da revisão.
Não confundir revisão de código, eventos simulados ou capturas com homologação.

## Referências visuais e funcionais

Encontrados 29 PNGs e dois MP4 em `.local/references/eplan/`. A auditoria examinou
os 29 PNGs, 22 quadros históricos e dois novos quadros extraídos dos vídeos.
Foram observados Ribbon/paleta de símbolos, grade, navegação e símbolos no canvas.
Não houve reprodução integral ou sequência comprovável de criar/editar junções;
nenhum comportamento não observado foi atribuído aos vídeos.

A documentação oficial sustenta a separação entre símbolos gráficos de conexão e
alvos funcionais. Referências: [Connection Symbols](https://www.eplan.help/en-us/infoportal/content/plattform/2023/content/htm/egedgui_k_start.htm),
[Connections: Principle](https://www.eplan.help/en-US/Infoportal/Content/Plattform/2.9/Content/htm/connectionbrowsergui_k_prinzipverbindungen.htm),
[Connection direction](https://eplan.help/en-us/Infoportal/Content/Plattform/2027/Content/htm/egedgui_k_darstellungverbindungspunkte.htm).
O CAD-CRAFT exige confirmação explícita, conforme o anexo; não reproduz integralmente
o autoconnecting ou target tracking do EPLAN. Desenhos/código continuam originais.

## Desempenho e limitações

Medição Core Debug local: 1.001 rotas, 2.002 símbolos, 3.003 representações e
snapshot de 3.245.849 bytes. Importação ~368 ms, transformação de dois símbolos
~41 ms, edição de rota ~60 ms, junção ~69 ms e validate ~147 ms. Deltas compactos;
Undo das três operações restitui a serialização byte a byte.

Medição GUI Debug local, cenário adversarial denso com sobreposição de 500 cópias
da demonstração: 1.000 rotas, 2.000 símbolos, 4.000 portas associadas e JSON de
4.555.791 bytes. Preparação do lote ~629 ms; importação ~431 ms; cache inicial
~16,8 ms; 100 sincronizações sem revisão nova ~7,7 µs; 100 consultas de snap
~122 ms; 20 prévias ~35,2 µs; edição confirmada ~61,3 ms; reconstrução de cache
~33,5 ms. Cinco frames completos de CadApp em CPU, incluindo tesselação, levaram
~1,992 s, totalizando 188.618 shapes e 2.366.090 vértices de malha. Esse custo
de renderização é elevado e permanece uma limitação observada. A sobreposição
proposital não representa um projeto industrial típico; não há meta de FPS
aprovada nem garantia de desempenho. O teste verifica integridade/cache e mede
custos sem impor um limiar temporal dependente da máquina.

Cache por revisão/página evita reconstrução da geometria em cada frame. Snap/hit
continuam lineares nos candidatos da página; pintura faz clipping. Índices Core
são reconstruídos em confirmação/replay, nunca em hover. Medições não demonstram
latência de GPU/apresentação, pico de memória, Windows/DPI físicos ou escala industrial.

Limites funcionais explícitos:

- Junções unem rotas existentes; não há extremidade gráfica livre que termine
  diretamente numa junção/segmento sem porta. Fluxo porta→porta está implementado.
- Junções não se apropriam de arestas preexistentes de terceiros. Edição que
  invalidaria uma junção é recusada, sem desconexão lógica implícita.
- Ângulos são dobras da rota e T/múltiplas ramificações são junções explícitas.
  Sem novos dispositivos físicos ou segundo modelo de SymbolDefinition. Pontes
  decorativas, variantes de rastreamento, interrupções e potenciais completos são futuros.
- Sugestão por alinhamento não é conexão automática nem roteador industrial.
- ERC, dimensionamento, persistência física final e homologação permanecem futuros.
  ENV-003, Windows 11/alto contraste/DPI nativo e backup externo continuam pendentes.

## Evidências finais

Todas as execuções abaixo terminaram com código 0 em 2026-10-30:

| Comando / verificação | Resultado |
|---|---|
| `pwsh -NoProfile -File scripts/validate.ps1` | fmt, check, testes, clippy com warnings negados, testes PowerShell e diff checks aprovados |
| `cargo build --release -p cad-gui --bin cad-gui --locked` | Executável otimizado gerado |
| `cargo build -p cad-gui --example render_qa --locked` | Harness da interface de produção compilado |
| `cargo test -p cad-gui --test connection_scale --locked -- --nocapture` | Medição GUI acima e integridade aprovadas |
| `cargo test --offline --manifest-path .local/etapa-08/gui-review-project/Cargo.toml --target-dir target --test review -- --nocapture review_` | 7 aprovados, 0 falhas; evidência auxiliar não versionável |
| `render_qa.exe connections 1600 900 1` e `render_qa.exe connections 1280 720 1.25` | Framebuffers gerados e inspecionados visualmente |

**339 testes oficiais aprovados: 324 Rust + 15 PowerShell, zero falhas e zero
ignorados.** Core: 159; GUI: 165. Foram preservados os 272 anteriores e adicionados
67: 25 Core e 42 GUI. Os testes auxiliares de revisão não inflaram essa contagem.
Uma execução anterior parou no lint de inicialização de campo em teste novo;
após a correção, o script completo foi reexecutado e aprovado.

Release: `target/release/cad-gui.exe`, **8.723.968 bytes**, PE Subsystem **2**
(Windows GUI). SHA-256:
`1D3FE250F3E9CD16FE2F83EAFFF4E41867CF05992FE4EEE54B392883344BF8F7`.
Build aprovado não equivale a homologação do executável em outros computadores.

Capturas internas reais: `.local/etapa-08/connections-final-1600.png` (1600×900,
escala 1,0) e `connections-final-1280.png` (1280×720, escala egui 1,25). Inspeção
confirmou guia Conexões, página, rotas cruzadas sem marca de junção, seleção e
destaque de rede, portas e propriedades reais. Na janela compacta, os painéis
reduzem a área útil e o conteúdo excedente exige navegação/rolagem do layout
existente. O cenário das capturas é cruzamento sem junção; marca/seleção de junção
e sua prévia têm testes específicos. Não houve homologação manual nem teste de
DPI nativo, decoração do Windows, alto contraste ou hardware diverso.

Logs: `.local/etapa-08/validate-final.log`, `release-build.log`, `qa-build.log`,
`gui-review-final.log`, `gui-scale-final.log` e pareceres `core-review.md`,
`gui-review.md`, `final-review.md`, `context-audit.md`. Logs/capturas/harness
auxiliares e `target/` são ignorados; não são conteúdo Git nem backup.

## Inventário final

Consulta de 2026-10-30: 18 arquivos criados, 29 modificados, nenhum excluído.
Nenhum arquivo staged. Código, testes e documentos abaixo estão fora do HEAD observado.
Cargo.toml, Cargo.lock, scripts e testes anteriores não foram alterados.

### Criados

- `crates/cad-craft/src/domain/connection_drawing.rs`
- `crates/cad-craft/src/domain/project/connection_drawing.rs`
- `crates/cad-craft/tests/connection_drawing.rs`
- `crates/cad-craft/tests/connection_drawing_scale.rs`
- `crates/cad-gui/src/connection_demo.rs`
- `crates/cad-gui/src/connection_editor.rs`
- `crates/cad-gui/src/connection_geometry.rs`
- `crates/cad-gui/src/connection_properties.rs`
- `crates/cad-gui/src/connection_view.rs`
- `crates/cad-gui/tests/connection_editor.rs`
- `crates/cad-gui/tests/connection_geometry.rs`
- `crates/cad-gui/tests/connection_history.rs`
- `crates/cad-gui/tests/connection_scale.rs`
- `crates/cad-gui/tests/connection_ui.rs`
- `docs/adr/0010-editor-de-conexoes-e-juncoes.md`
- `docs/internal/plans/2026-10-29-etapa-08-conexoes.md`
- `docs/internal/reports/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md`
- `docs/internal/spec/ETAPA_08_EDITOR_CONEXOES_GRAFICAS.md`

### Modificados

- `crates/cad-craft/src/domain/ids.rs`
- `crates/cad-craft/src/domain/mod.rs`
- `crates/cad-craft/src/domain/project.rs`
- `crates/cad-craft/src/domain/project/connectivity.rs`
- `crates/cad-craft/src/domain/project/integrity.rs`
- `crates/cad-craft/src/domain/project/representations.rs`
- `crates/cad-craft/src/domain/project/snapshot.rs`
- `crates/cad-craft/src/domain/representation.rs`
- `crates/cad-gui/examples/render_qa.rs`
- `crates/cad-gui/src/app.rs`
- `crates/cad-gui/src/commands.rs`
- `crates/cad-gui/src/dialogs.rs`
- `crates/cad-gui/src/editing.rs`
- `crates/cad-gui/src/editor_view.rs`
- `crates/cad-gui/src/history.rs`
- `crates/cad-gui/src/hit_test.rs`
- `crates/cad-gui/src/interaction.rs`
- `crates/cad-gui/src/lib.rs`
- `crates/cad-gui/src/panels/properties.rs`
- `crates/cad-gui/src/preferences.rs`
- `crates/cad-gui/src/state.rs`
- `crates/cad-gui/src/viewport.rs`
- `docs/internal/memory/ARCHITECTURE.md`
- `docs/internal/memory/CURRENT_STATE.md`
- `docs/internal/memory/DECISIONS.md`
- `docs/internal/memory/HANDOFF.md`
- `docs/internal/memory/KNOWN_ISSUES.md`
- `docs/internal/memory/ROADMAP.md`
- `docs/internal/spec/CONTRATOS_DOCUMENTAIS_GUI.md`

## Próxima ação

Revisão da entrega e **commit + push manuais da Etapa 8**, incluindo a Etapa 7 local
ainda não observada no remoto. Backup externo independente recomendado, não criado
ou verificado. Nenhum staging/commit/push/sincronização/worktree foi executado.
Encerrar sem iniciar Etapas 9 ou 17 e sem solicitar homologação manual.
