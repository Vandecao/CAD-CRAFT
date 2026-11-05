# Etapa 9 — identificação, referências e potenciais

Estado em 2026-11-02: implementada e validada automaticamente; pronta para revisão do usuário.
Especificação integral em `docs/internal/spec/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md`.
Plano em `docs/internal/plans/2026-10-31-etapa-09.md`.

## Base e autorização

Branch main; HEAD observado em 2026-10-30: 5377559ff3b5bd5ce1cd9d19265833ef3714796e.
Árvore inicial limpa. O anexo confirma commit/push manuais da Etapa 8.
Origin preservado. Todas as mudanças desta etapa permanecem fora desse HEAD.
Sem staging, commit, sincronização, worktree ou alteração de branches.
Baseline documentado: 339 testes (324 Rust + 15 PowerShell).

## Correção da Etapa 8

Investigação confirmou que points_with_endpoints usa points da rota atual e que
o commit chama with_points com a geometria reparada. No movimento seguinte,
repair_endpoints preserva esses pontos internos, incluindo dobras automáticas.
ConnectionDraft também incorporava a prévia inteira a cada waypoint confirmado.
Isso mistura intenção manual e geometria derivada, acumulando dobras intermediárias.
Testes RED reproduziram sobras após +40/+20/retorno, inclusive com junção explícita.
Correção concluída antes dos recursos novos; evidências em `.local/etapa-09/`.

## Funcionalidades implementadas

- Designações estruturadas, classificação configurável, conflitos e edição por IDs.
- Papéis funcionais explícitos, uma função principal por dispositivo, referências
  derivadas para ocorrências/contatos reais, páginas e coordenadas reais em mm.
- Marcadores de interrupção e potencial ligados à Representation existente;
  associação opcional e explícita a ConnectionPoint, sem objetos elétricos fictícios.
- Grupos estrela/cadeia em ordem explícita; associação documental separada de
  continuidade elétrica opcional por arestas explícitas do grafo existente.
- Potenciais referenciados por ponto persistente, rede derivada consultada após
  mutações; diagnósticos de incompatibilidade sem inferir conexão por texto.
- Histórico de deltas, snapshots compatíveis e integração ao dispatcher,
  propriedades, navegador e canvas existentes. Links abrem a página, centralizam
  a ocorrência e a selecionam; sem transação documental para navegar.

As funções recebem papel principal, auxiliar ou não especificado. A designação
completa é validada em inserção/renomeação; conflitos legados são diagnosticados.
Referências de dispositivo/bobina/contato consultam ocorrências, função, terminais,
página e posição reais e se atualizam ao mover/renomear. Os marcadores compartilham
RepresentationId, podem ser vinculados explicitamente a ConnectionPoint e têm
identidade estável. Grupos estrela/cadeia têm ordem editável; continuidade opcional
usa arestas do grafo existente e sua remoção é reversível. Potenciais iguais não
criam redes; potenciais incompatíveis na mesma rede geram diagnóstico visível.
Undo/Redo usa DeviceDelta/MarkerDelta e operações documentais de página no
histórico comum. Novos campos do snapshot têm defaults para versões anteriores.

O painel mostra identificações, função/papel, conflitos, propriedades do potencial,
ocorrências e links. O navegador Dispositivos lista ocorrências; Conexões lista
marcadores com destinos selecionáveis. A Ribbon existente inicia inserção de
interrupção/potencial e localização. Não foi criado navegador paralelo.

## Referências funcionais consultadas

[Fundamentos de referências cruzadas](https://eplan.help/en-us/Infoportal/Content/Plattform/2027/Content/htm/xessettingsgui_k_grundlagen.htm)
orientaram a localização de funções distribuídas e a apresentação dependente da
estrutura documental. O CAD-CRAFT não presume colunas sem moldura apropriada.
[Referências de interrupção](https://eplan.help/en-us/Infoportal/Content/Plattform/2027/Content/htm/interruptionpointgui_k_darstellungabbruchstellen.htm)
orientaram a origem de estrela, ordem de cadeia e diagnóstico de destino ausente.
O CAD-CRAFT adota vínculo explícito por ID, conforme seu requisito.
A terceira URL do anexo não pôde ser recuperada (cache miss e busca sem resultado).

Materiais locais: folha de contato com 29 PNGs, inspeção ampliada das guias
Visualização/Dispositivos e controles, além de dois quadros extraídos anteriormente
dos vídeos. Não se atribui aos vídeos uma interação de referências não observada.
Nenhum código/ativo proprietário foi incorporado.

## Validação, revisão e inventário

Revisão independente de leitura concluída sem P1 confirmado. Encontrou três
lacunas P2: diagnósticos Core não expostos na GUI, ordem da cadeia sem controles
e ausência de ocorrências no navegador. As três foram corrigidas antes da
validação final. O primeiro `scripts/validate.ps1` da Etapa 9 aprovou fmt, check
e testes, mas Clippy recusou o tamanho de `MarkerEdit::Insert`; o payload foi
encaixotado e a validação foi reiniciada. Não se mascara essa falha intermediária.
Logs em `.local/etapa-09/validate-final.log` e direcionados `*-green.log`.

Validação final oficial `pwsh -NoProfile -File scripts/validate.ps1`: exit 0.
`cargo fmt --check`, `cargo check --workspace --all-targets --locked`,
`cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
15 testes operacionais PowerShell e `git diff --check` passaram. Contagem real:
364 Rust + 15 PowerShell = **379 aprovados**, 0 falhas, 0 ignorados. Os 339
anteriores foram preservados; 40 novos. Build Release `cargo build --release
-p cad-gui --bin cad-gui --locked`: exit 0 em 25,51 s na última execução. Executável
`target/release/cad-gui.exe`, 9.130.496 bytes, SHA-256
`DC3D383840B5BC4B84464D4F25ECA296396C7E9D86F9866673494025B524AA29`.
Não houve homologação manual.

Git somente consulta em 2026-11-02: branch `main`, HEAD local
`5377559ff3b5bd5ce1cd9d19265833ef3714796e`; `ls-remote origin
refs/heads/main` retornou o mesmo SHA nessa fotografia. Todas as alterações
abaixo permanecem fora do HEAD. Nenhum commit, push ou backup foi executado.

Arquivos **modificados**: `crates/cad-craft/src/domain/` —
`connection_drawing.rs`, `device.rs`, `function.rs`, `ids.rs`, `mod.rs`,
`project.rs`, `project/connection_drawing.rs`, `project/connectivity.rs`,
`project/integrity.rs`, `project/representations.rs`, `project/snapshot.rs`,
`representation.rs`; `crates/cad-craft/tests/connection_drawing.rs`;
`crates/cad-gui/src/` — `app.rs`, `commands.rs`, `connection_editor.rs`,
`connection_geometry.rs`, `editing.rs`, `editor_view.rs`, `history.rs`,
`interaction.rs`, `lib.rs`, `panels/navigator.rs`, `panels/properties.rs`,
`state.rs`; `crates/cad-gui/tests/connection_ui.rs`, `editor_history.rs`;
`docs/internal/memory/CURRENT_STATE.md`, `DECISIONS.md`, `HANDOFF.md`, `KNOWN_ISSUES.md`.

Arquivos **criados**: `crates/cad-craft/src/domain/identification.rs`,
`marker.rs`, `project/identification.rs`, `project/markers.rs`;
`crates/cad-craft/tests/identification.rs`, `markers.rs`;
`crates/cad-gui/src/device_properties.rs`, `reference_properties.rs`,
`references.rs`; `crates/cad-gui/tests/reference_ui.rs`, `references.rs`;
`docs/adr/0011-identificacao-referencias-potenciais.md`,
`docs/internal/plans/2026-10-31-etapa-09.md`,
`docs/internal/reports/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md`,
`docs/internal/spec/ETAPA_09_IDENTIFICACAO_REFERENCIAS_POTENCIAIS.md`.
Arquivos excluídos: nenhum. Os dois documentos `2026-11-02-ai-memory.md` e
`AI_MEMORY_INSTALACAO_2026-11-02.md` já presentes e não versionados pertencem à
tarefa ambiental separada; foram preservados, sem atribuí-los à Etapa 9.

Após a revisão, as linhas de referência do navegador receberam cache por
projeto/revisão/guia para evitar reconsultas de todas as ocorrências a cada
frame. Validação oficial, Release e diff check foram repetidos após esse ajuste;
os resultados acima são da última execução.

Limites: snapshots legados não registram proveniência de dobras; seus pontos são
preservados conservadoramente. Sem moldura apropriada, referências documentais
usam página e mm, não coluna fictícia. Sem cálculo elétrico ou rede por nome.
Homologação abrangente permanece após Etapa 15. Densidade visual da grade e
Etapa 17/OpenAI permanecem futuras. Não iniciar Etapa 10.

## Porta geométrica concluída — 2026-10-31

Correção implementada: routing_base separa restrições da polilinha derivada;
RouteIntent guarda waypoints clicados e orientação de cada trecho novo.
CreateRouteWithIntent compartilha construção ortogonal com a prévia; movimentos
usam with_endpoints e não promovem dobras calculadas a edição manual.
Snapshots antigos adotam pontos existentes como restrições conservadoras.
Translação conjunta transporta restrições; movimentos individuais as preservam.
Junções inválidas continuam recusadas atomicamente. Grafo/fiação não mudam.

RED confirmado em geometry-red.log e geometry-red-intent.log. GREEN Core 33/33,
GUI 13/13 na suíte final. Teste novo egui exigiu acomodar layout antes de mapear
cliques; o erro de 0,015625 mm não provinha da rota e não foi mascarado por tolerância.

pwsh -NoProfile -File scripts/validate.ps1 terminou com exit 0:
335 Rust + 15 PowerShell = 350 aprovados, zero falhas/ignorados. Os 339 anteriores
foram preservados; 11 regressões adicionadas. fmt/check/test/clippy/diff aprovados.
Log: .local/etapa-09/geometry-validate.log. Teste adicional direcionado real_connected_drags
aprovado em geometry-gui-cycle.log. Recursos novos liberados após esta evidência.
Arquivos: Core domain/connection_drawing.rs, project/connection_drawing.rs e teste
connection_drawing.rs; GUI connection_editor.rs, connection_geometry.rs e
connection_ui.rs. Revisão independente do conjunto permanece ao final da etapa.
