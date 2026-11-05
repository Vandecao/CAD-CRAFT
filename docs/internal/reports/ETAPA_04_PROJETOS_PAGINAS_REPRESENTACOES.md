# Etapa 4 — Projetos, páginas e representações

Data: 2026-10-27, America/Sao_Paulo.
Estado: implementada e testada automaticamente; revisão do usuário e homologação
manual pendentes. Etapa 5 não iniciada e sem autorização.

## 1. Autorização e base verificável

Executado somente o anexo preservado integralmente em
`docs/internal/spec/ETAPA_04_PROJETOS_PAGINAS_REPRESENTACOES.md`.
Plano: `docs/internal/plans/2026-10-27-etapa-04-documental.md`.
AGENTS e memória lidos; arquitetura/ADRs pertinentes consultados.

Branch: `main`. Origin preservado: `https://github.com/Vandecao/CAD-CRAFT.git`.
HEAD observado por `git rev-parse HEAD` em 2026-10-27:
`df6b482a14b91aad571ae38d7ef7e9c1c26a2e3d` — commit manual da Etapa 3,
`feat: implement physical wiring cables and terminal strips`.
Árvore de trabalho estava limpa. A memória anterior que indicava commit pendente
da Etapa 3 foi corrigida a partir da evidência Git, sem inventar histórico.

`git ls-remote --exit-code origin refs/heads/main` retornou, com exit 0:
`98445717f16b9f464aa45664422b6eec12c35781`, Etapa 2. Consulta em 2026-10-27,
sem fetch. Tracking local indicou 1 à frente/0 atrás. O commit local da Etapa 3
ainda não constava da main remota consultada. Nenhuma operação de sincronização.

Todos os arquivos desta entrega estão somente na árvore de trabalho, sem staging.
Este relatório e a memória atualizada não integram o HEAD observado. Nenhum SHA
de commit futuro é presumido. Commit + push permanecem exclusivamente manuais.

## 2. Implementação e APIs

O agregado existente Project foi ampliado. Nenhuma entidade elétrica foi duplicada.
Document/Page/PageId e os cinco tipos iniciais de página foram reutilizados.
O Core mantém somente std + serde + uuid; não recebeu dependência gráfica.

| Área | Entidades e operações disponíveis |
|---|---|
| Projeto | identification/set_identification opcionais; nome, descrição, metadata e properties anteriores preservados |
| Página | order, structure, orientation; rename_page, identify_page, set_page_format, set_page_structure, reorder_pages e remove_page |
| Hierarquia | DocumentStructure, StructureAxis, PageGroup; page_tree por documento/eixo e ordered_pages |
| Ocorrências | RepresentationId UUID v7, Representation, RepresentationTarget e VisualProperties |
| Edição documental | insert_representation, move_representation, set_representation_visual, set_representation_bounds, remove_representation |
| Matemática | Transform2D::new/apply/inverse_apply/transform_bounds; rotação, espelhamento e translação em mm |
| Consulta | representation(s), representations_on_page, representations_for_entity, related_representations, representation_location, document_bounds |
| Integridade | DocumentIndex privado por página/alvo; reconstrução na importação e comparação em Project::validate |

RepresentationTarget cobre Device, Function, Component, Terminal, ConnectionPoint,
Connection, Conductor, Cable, CableCore e TerminalStrip. Um alvo pode ter várias
ocorrências na mesma página ou em páginas diferentes. VisualProperties armazena
visibilidade, camada e rótulo opcional. local_bounds é extensão local opcional,
sem catálogo de símbolos. PageElement legado continua preservado e independente.

Orientação deriva das dimensões, sem duplicar estado. Ordem documental é separada
de ID e de number: reordenação valida a permutação completa de um documento.
Ausência de ordem é válida; empate é determinístico por PageId. Grupos funcionais
e de localização são caminhos independentes, sem criar funções/dispositivos elétricos.

## 3. Integridade e coordenadas

- Página precisa pertencer ao Project e referenciar Document existente.
- Representação exige ProjectId correto, página e alvo existentes, ID único e
  limites transformados válidos. Alvos documentais não entram no enum elétrico.
- Espelhamento dos componentes locais X/Y precede rotação anti-horária em graus;
  translação ocorre por último. Origem inferior esquerda e Y para cima preservados.
  Rotação finita normalizada; não finitos e overflow retornam DomainError.
- Representações podem ficar fora da folha. DocumentBounds separa sheet/content;
  conteúdo inclui invisíveis. Sem dimensão local, apenas a âncora é conhecida.
  Nenhum limite documental restringe pan/zoom do viewport futuro.
- Página com representações ou referências de propriedades de outras entidades
  não pode ser removida. Referência da própria página desaparece com ela.
- Remover representação preserva alvo, redes, condutores, cabos, veias e réguas.
  Connection representada também bloqueia remove_connection, além do bloqueio
  físico existente. Desvinculação/remoção prévia deve ser explícita.
- Sem cascatas e sem falhas previstas deixando índices/registros parcialmente
  alterados. Demais entidades elétricas continuam sem exclusão genérica.

## 4. Compatibilidade dos snapshots

Serde estrutural continua sendo o mecanismo disponível. Acrescentados:
representations com default vazio, identification e order com None, structure com
caminhos vazios. Não gera páginas, grupos, metadados presumidos ou representações
para projetos antigos. IDs e relações elétricas são preservados.

Importação reúne registros, rejeita duplicados e referências inválidas, reconstrói
índices e executa validate. Serialização é determinística por ID; ordem documental
é um atributo próprio. Campos desconhecidos são rejeitados nos DTOs.

Evidência: round-trip completo, listas permutadas, dez alvos, transformações/estilo/
limites, snapshots sem novos campos e corrupções são testados. Os testes de legado
simulam esquemas anteriores removendo campos aditivos do snapshot atual; não usam
fixtures históricas congeladas. Os testes anteriores também foram preservados.
Leitores das etapas antigas não recebem garantia de ler campos novos. Sem formato
.cadcraft final, salvamento físico, schema definitivo ou migrações implementadas.

## 5. Validações e testes

Comando oficial: `pwsh -NoProfile -File scripts/validate.ps1`.
Execução final com **exit 0**, registrada em `.local/etapa-04/final-validation.log`
e `final-validation-exit.log`. O log contém todas as verificações aprovadas:

| Comando executado pelo script | Resultado |
|---|---|
| cargo fmt --check | Aprovado |
| cargo check --workspace --all-targets --locked | Aprovado, incluindo cad-gui |
| cargo test --workspace --locked | 113 Rust aprovados; 0 falhas/ignorados |
| cargo clippy --workspace --all-targets --locked -- -D warnings | Aprovado |
| pwsh -NoProfile -File scripts/tests/check-repository.Tests.ps1 | 15 aprovados; 0 falhas |
| git diff --check e git diff --cached --check | Aprovados |

Contagem real: **113 Rust + 15 PowerShell = 128 aprovados**.
São **25 testes Rust novos**, mantendo os 88 anteriores. Nenhum teste removido,
ignorado ou alterado para ocultar falha. Unit tests de lib/bin continuam zero;
as verificações de domínio são testes de integração e compile_fail.

| Suíte | Aprovados |
|---|---:|
| connections | 6 |
| connectivity | 13 |
| connectivity_scale | 3 |
| connectivity_serialization | 6 |
| document_integrity (nova) | 4 |
| document_pages (nova) | 5 |
| document_representations (nova) | 6 |
| document_serialization (nova) | 5 |
| document_values (nova) | 5 |
| physical_insertion | 3 |
| physical_scale | 2 |
| physical_serialization | 8 |
| physical_values | 6 |
| physical_wiring | 13 |
| project_model | 7 |
| serialization | 10 |
| value_types | 8 |
| Doc-tests compile_fail | 3 |
| **Total Rust** | **113** |

Os testes iniciais da etapa falharam por APIs/tipos inexistentes antes da
implementação; os testes de páginas/valores passaram após sua implementação,
seguidos da suíte completa. Foram acrescentados casos para todos os alvos elétricos,
todos os proprietários de referências de propriedades, inversa/rotação oblíqua,
overflow, formatos antigos e invariância dos dez registros elétricos após remoções.
Logs locais em `.local/etapa-04/` estão ignorados e não devem ser versionados.
O aviso de conversão CRLF/LF em CURRENT_STATE/HANDOFF foi eliminado normalizando
esses textos para LF; git diff --check foi repetido após a documentação final.

Diagnóstico adicional: `pwsh -NoProfile -File scripts/check-repository.ps1`, exit 0,
nenhum achado nas regras auditadas. Diagnóstico é offline; não prova ausência de
segredos, não varre histórico completo e não confirma o remoto. A confirmação
remota descrita acima veio da consulta ls-remote separada.

## 6. Revisão independente

Subagente interno independente, contexto novo e somente leitura, inspecionou diff
contra df6b482, arquivos novos, plano, especificação e testes documentais.
Veredito: **aprovado para revisão do usuário; nenhum Critical, Important ou Minor
no código inspecionado**. Não executou testes nem modificou arquivos/Git; consultou
o log já gerado pelo principal. Nenhum agente externo foi utilizado.

O revisor explicitou o limite dos snapshots legados simulados, já registrado acima.
Fora de seu julgamento: documentação final em elaboração (conferida pelo principal),
GUI/egui, símbolos, roteamento, formato físico definitivo, desempenho documental
em escala e homologação manual. O principal manteve esses itens fora do escopo
autorizado e documentou os limites, sem promessas de capacidade ou aprovação visual.
Nenhum achado menor adiado. Revisão independente não é homologação do usuário.

## 7. Contratos para a Etapa 5 e limitações

`docs/internal/spec/CONTRATOS_DOCUMENTAIS_GUI.md` descreve navegador, viewport, seleção,
inspetor, renderização e mapeamento de comandos futuros para APIs atuais.
O consumidor cad-gui permanece diagnóstico de console, sem alteração nesta etapa.

Ainda não existem GUI, Ribbon, widgets, símbolos, rotas, seleção/hit testing exato,
comandos agrupados, Undo/Redo ou persistência física final. Árvore usa caminhos sem
identidade estável de grupo; não há CRUD global de grupos. Navegação relacionada
é entre ocorrências do mesmo alvo. PageElement legado não é convertido em gráfico.

Dimensões zero continuam válidas por compatibilidade geométrica, sem homologação
para impressão. A união de extremos finitos pode exceder f64 e produzir erro.
Escala documental não foi medida; consulta por documento percorre páginas e árvore
materializa prefixos. Importação continua sem limite de recursos definido.
Decisões, alternativas e consequências completas no ADR-0006.

## 8. Roteiro de homologação futura

Pendente, conforme adiamento de homologações até após a Etapa 15. Na GUI futura:

1. Abrir projeto antigo; confirmar que nenhuma página/grupo/ocorrência é fabricada.
2. Criar páginas dos cinco tipos; editar identificação/nome e orientação pelas
   dimensões; reordenar e confirmar permanência dos IDs e separação entre documentos.
3. Classificar por função e localização, incluindo caminhos multinível e páginas
   sem classificação; conferir que dispositivos/funções elétricos não são criados.
4. Inserir várias ocorrências de um mesmo dispositivo e de uma conexão em páginas
   distintas; navegar entre ocorrências e conferir identidade elétrica única.
5. Mover para coordenadas negativas/fora da folha, girar CCW e espelhar X/Y;
   conferir que a visualização inverte Y somente na conversão para tela.
6. Conferir seleção por ocorrência, estilo, camada, rótulo e enquadramento sem
   recorte da folha; tratar geometria desconhecida e falhas numéricas explicitamente.
7. Tentar remover página usada e conexão representada: exigir bloqueio; remover
   ocorrências/referências explicitamente e conferir que o restante elétrico permanece.
8. Reabrir snapshot e conferir UUIDs, ordem, caminhos, estilos e ligações; comparar
   redes/fiação antes/depois. Registrar evidências reais e somente então homologar.

## 9. Inventário da entrega

**18 criados, 16 modificados, zero excluídos**, relativos a `C:\Dev\CAD-CRAFT`.
Código/testes anteriores preservados, exceto as extensões explícitas do Core abaixo.

Criados:

```text
crates/cad-craft/src/document_index.rs
crates/cad-craft/src/domain/document_structure.rs
crates/cad-craft/src/domain/project/document_queries.rs
crates/cad-craft/src/domain/project/documentation.rs
crates/cad-craft/src/domain/project/representations.rs
crates/cad-craft/src/domain/representation.rs
crates/cad-craft/src/domain/transform.rs
crates/cad-craft/tests/document_common/mod.rs
crates/cad-craft/tests/document_integrity.rs
crates/cad-craft/tests/document_pages.rs
crates/cad-craft/tests/document_representations.rs
crates/cad-craft/tests/document_serialization.rs
crates/cad-craft/tests/document_values.rs
docs/adr/0006-estrutura-documental-e-representacoes.md
docs/internal/plans/2026-10-27-etapa-04-documental.md
docs/internal/reports/ETAPA_04_PROJETOS_PAGINAS_REPRESENTACOES.md
docs/internal/spec/CONTRATOS_DOCUMENTAIS_GUI.md
docs/internal/spec/ETAPA_04_PROJETOS_PAGINAS_REPRESENTACOES.md
```

Modificados:

```text
crates/cad-craft/src/domain/ids.rs
crates/cad-craft/src/domain/mod.rs
crates/cad-craft/src/domain/page.rs
crates/cad-craft/src/domain/project.rs
crates/cad-craft/src/domain/project/connectivity.rs
crates/cad-craft/src/domain/project/integrity.rs
crates/cad-craft/src/domain/project/snapshot.rs
crates/cad-craft/src/error.rs
crates/cad-craft/src/lib.rs
docs/internal/memory/ARCHITECTURE.md
docs/internal/memory/CURRENT_STATE.md
docs/internal/memory/DECISIONS.md
docs/internal/memory/HANDOFF.md
docs/internal/memory/KNOWN_ISSUES.md
docs/internal/memory/PROJECT_CONTEXT.md
docs/internal/memory/ROADMAP.md
```

Cargo.toml (workspace/crates), Cargo.lock, scripts, cad-gui, testes anteriores e
regras de ignore não foram modificados. Não há artefatos de build ou backups na
entrega. O anexo preservado e o original têm SHA-256 idêntico:
`F450A643B49B28E5CD86884F372C44D40C93C1CCE4BCAD7D3F765AF4834CF85F`.

## 10. Próxima ação e proteção

Entrega destinada à revisão do usuário. Fazer **commit + push manuais** da Etapa 4,
incluindo o commit local da Etapa 3 ainda pendente de envio. Providenciar backup
externo independente; não criado/verificado automaticamente. Nenhum staging,
commit, push, pull, fetch, worktree ou outra operação Git modificadora executada.
Não iniciar a Etapa 5 sem o prompt e a autorização correspondentes.
