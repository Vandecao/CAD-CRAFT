# CAD-CRAFT — ETAPA 1 — CORE E MODELO DE DOMÍNIO

Data de conclusão técnica: 2026-10-27, America/Sao_Paulo.
Escopo autorizado: [anexo integral](../spec/ETAPA_01_CORE_DOMINIO.md).
Plano e registro: [plano da Etapa 1](../plans/2026-10-26-etapa-01-core-dominio.md).

**Etapa 1 — Implementada e testada automaticamente. Homologação manual pendente.**

## Objetivo e arquitetura implementada

Construído o núcleo estrutural de engenharia elétrica no workspace existente.
`cad-craft` depende diretamente apenas de std, Serde e uuid; `cad-gui` consome
o Core e permanece um diagnóstico de console. Toolchain e divisão de crates
preservadas. A árvore de dependências foi inspecionada, sem egui/eframe/renderização.

O agregado `Project` mantém um registro por tipo de entidade em BTreeMap.
Consultas retornam referências imutáveis; inserções verificam projeto, duplicação,
existência de alvos e coerência das relações antes de alterar o registro.
Listas inversas são derivadas, evitando múltiplas fontes de verdade.
Decisões e alternativas no [ADR-0003](../../adr/0003-core-eletrico-e-integridade.md);
diagrama em [ARCHITECTURE](../memory/ARCHITECTURE.md).

| Entidade | Dados e relações |
|---|---|
| Project | ID estável, nome, descrição, metadados, propriedades e registros |
| Document | ID, ProjectId, nome, metadados; páginas consultadas no agregado |
| Page | Documento, nome/número, tipo, dimensões, metadados, referências elétricas |
| Device | Designação, classificação e propriedades; múltiplas funções por consulta |
| Function | Dispositivo proprietário, designação, classe funcional e propriedades |
| Component | Identidade elétrica, classificação, fabricante/catálogo opcionais; dispositivo/função opcionais |
| Terminal | Terminal físico de dispositivo ou componente, designação e propriedades |
| ConnectionPoint | Ponto funcional de função, designação e terminal físico opcional |

PageKind distingue multifilar, unifilar, visão geral, layout de painel, documentação
técnica e extensão customizada. FunctionKind distingue bobina, contato auxiliar,
contato principal e extensão customizada. Página referencia componente/função sem
criar símbolo, posição ou representação gráfica.

## IDs, geometria e designações

Oito newtypes com UUID v7: ProjectId, DocumentId, PageId, ComponentId, DeviceId,
FunctionId, TerminalId e ConnectionPointId. Campos internos privados, geração
centralizada, validação da versão/variante em conversão e Serde, Eq/Ord/Hash,
Display/FromStr e suporte a coleções. Não existe conversão implícita entre IDs;
um doctest compile_fail comprova essa barreira. Unicidade verificada por tipo
no projeto; 2.048 IDs gerados no teste de unicidade.

Point2D, Size2D e Rect2D usam f64 finito em milímetros. Origem inferior esquerda,
X positivo à direita e Y positivo para cima. Dimensões são não negativas, incluindo
zero; retângulos exigem limites finitos. Translação e distância retornam Result em
overflow; contenção e interseção consideram bordas inclusivas. Sem zoom/pan/viewport.

Designation separa caminhos funcionais, caminhos de localização, dispositivo e
terminal opcional. Partes aceitam letras/números, sublinhado e ponto; rejeitam
vazio, espaços e separadores embutidos. Text valida rótulos e remove espaços externos.
Não há parser IEC 81346 nem alegação de conformidade normativa.

## Propriedades e integridade

TechnicalProperties separa propriedades conhecidas e customizadas.
KnownProperty define tensão (V), corrente (A), potência (W), frequência (Hz),
resistência (ohm) e seção transversal (mm²), com valores finitos não negativos.
Customizadas suportam texto, i64, decimal f64 finito, booleano, enumeração controlada
e EntityRef tipada. Enumerações exigem opções únicas e seleção existente.
Metadados descritivos não substituem os campos estruturais.

DomainError diferencia IDs/textos/designações/números/enumerações inválidos,
entidades duplicadas/ausentes, projeto estrangeiro e relações inválidas.
As principais barreiras são:

- cada entidade pertence ao Project informado;
- páginas referenciam documento e elementos existentes;
- função pertence a dispositivo existente;
- função de componente exige o mesmo dispositivo do componente;
- terminal referencia proprietário físico existente;
- ponto funcional referencia função existente; seu terminal, quando presente,
  precisa corresponder ao dispositivo e, se especificada, à função do componente;
- referências de propriedades resolvem apenas dentro do agregado;
- inserções duplicadas, vínculos repetidos na página e operações inválidas são rejeitados;
- falhas não substituem entidades/propriedades nem deixam inserções parciais.

Não há remoção/reassociação genérica ou acesso mutável às entidades registradas.
Esses recursos não foram necessários ao escopo de criação/consulta e integridade.

## Serialização

Serde está implementado nos valores e entidades. Construtores e desserialização
compartilham validação dos valores restritos. Snapshot interno do Project usa listas,
preservando detecção de IDs duplicados. A importação constrói todos os registros
antes de validar referências, suportando ordem arbitrária e ciclos em propriedades.
A serialização percorre registros sem clonar o agregado completo.

Entidade desserializada isoladamente exige registro no Project para validar suas
relações. Nas inserções individuais, alvos devem existir; referências cíclicas podem
ser adicionadas posteriormente por set_properties. serde_json está apenas nas
dev-dependencies; não há API de arquivo, ZIP, salvamento atômico, schema versionado,
migração ou formato definitivo .cadcraft.

Referências técnicas consultadas: [API uuid](https://docs.rs/uuid/1.29.0/uuid/struct.Uuid.html)
e [atributos Serde](https://serde.rs/container-attrs.html). O lockfile resolveu
serde 1.0.229, uuid 1.29.0 e serde_json 1.0.154.

## Evidências de validação automática

`pwsh -NoProfile -File scripts/validate.ps1` terminou com exit 0 em 2026-10-27:

| Comando | Resultado |
|---|---|
| cargo fmt --check | Aprovado |
| cargo check --workspace --all-targets --locked | Aprovado |
| cargo test --workspace --locked | 26 testes Rust aprovados, 0 falhas |
| cargo clippy --workspace --all-targets --locked -- -D warnings | Aprovado, sem warnings |
| pwsh -NoProfile -File scripts/tests/check-repository.Tests.ps1 | 15 aprovados, 0 falhas |
| git diff --check | Aprovado |
| git diff --cached --check | Aprovado; sem staging pelo Codex |

Contagem real Rust: value_types 8, project_model 7, serialization 10, doctest
compile_fail 1. Total: **26 Rust + 15 PowerShell = 41 testes aprovados**.
Os binários de testes unitários lib/main possuem 0 testes; não foram contabilizados
como testes adicionais. Não há testes ignorados.

Os testes cobrem criação, identidade, associação, múltiplas funções, proprietários,
propriedades, geometria, entradas inválidas, falhas sem mutação, round-trip completo
e adulteração dos dados serializados em todos os registros. O bootstrap e os
15 testes operacionais existentes foram preservados. Os ciclos iniciais observaram
falhas por APIs/traits ainda ausentes antes da implementação; registro no plano.

Verificações adicionais aprovadas: `cargo run -p cad-gui --locked` (cria/valida Project),
`cargo doc -p cad-craft --no-deps --locked`, árvore de dependências do Core e
`pwsh -NoProfile -File scripts/check-repository.ps1` (sem achados).
Logs locais de validação em `.local/etapa-01/`, ignorados; não são entregáveis versionados.

## Revisão técnica

Separação Core/GUI, vínculos, erros, IDs, Serde, testes e exclusões de escopo
inspecionados. Revisão independente interna somente de leitura solicitada pela
skill de revisão: **nenhum achado Critical, Important ou Minor acionável**.
O revisor examinou módulos, manifests e testes, sem editar arquivos nem executar
comandos de teste. A evidência de execução pertence ao agente principal; o parecer
independente é por inspeção. A documentação, finalizada em paralelo, foi conferida
pelo agente principal, sem segunda auditoria documental independente.

Deliberações confirmadas no fechamento: entidades destacadas têm integridade local
até registro no agregado; unicidade de IDs é por categoria; designações repetidas,
dimensões zero e validação não normativa são permitidas; CRUD/persistência completos
ficam fora do escopo; mapas de propriedades/metadados usam substituição por chave.
Essas decisões estão coerentes com o ADR-0003 e não exigem correções nesta etapa.
Não há achados menores adiados. A revisão não constitui homologação manual.

Avisos iniciais de conversão LF/CRLF foram resolvidos normalizando somente os
arquivos alterados conforme os atributos/configuração existentes. As conferências
finais de diff --check, diff --cached --check e whitespace dos arquivos novos
passaram sem avisos; nenhuma configuração Git foi alterada.

## Limitações e próximas dependências arquiteturais

- Homologação manual pendente, adiada até após a Etapa 15.
- Validação estrutural não é ERC e não calcula conectividade/redes/condutores.
- Não há gráficos, navegação de referências, símbolos, rotas, histórico ou persistência física.
- Designações/localizações são valores estruturados; escopos normativos de unicidade
  e cadastros hierárquicos especializados ainda não foram definidos.
- f64 oferece precisão de desenho técnico, sem aritmética decimal exata.
- Consultas inversas são lineares; importação completa não tem limite de recursos definido.
- Formato de serialização atual é estrutural e evolutivo; compatibilidade persistente
  dependerá da etapa de armazenamento/versionamento, ainda não autorizada.
- ENV-003 continua sem confirmação de reinício; backup externo não criado/verificado.

## Git e proteção

Branch main; origin preservado: https://github.com/Vandecao/CAD-CRAFT.git.
HEAD observado por git rev-parse HEAD em 2026-10-27:
`69f8d6c2877ba617f1c41f8c28d333b67f51e9a8`.
Consulta git ls-remote --exit-code origin refs/heads/main no mesmo dia retornou
o mesmo SHA. Tracking local indicou 0 à frente/atrás. Nenhuma sincronização executada.

Árvore inicialmente limpa. O estado antigo da memória (99b4161 e política pendente)
foi reconciliado com o HEAD real: política já commitada pelo usuário.
Toda a Etapa 1 permanece não commitada; este relatório não faz parte do HEAD citado.
Nenhuma operação Git modificadora, worktree ou agente externo foi usada.

Necessário: revisão e commit manual da Etapa 1. Push segue a frequência manual de
AGENTS (pares e Etapa 15); as alterações atuais não estão protegidas no GitHub.
Backup externo independente recomendado, sem criação automática.

## Inventário da entrega

Criados (23 arquivos):

- crates/cad-craft/src/domain/component.rs
- crates/cad-craft/src/domain/connection_point.rs
- crates/cad-craft/src/domain/designation.rs
- crates/cad-craft/src/domain/device.rs
- crates/cad-craft/src/domain/document.rs
- crates/cad-craft/src/domain/function.rs
- crates/cad-craft/src/domain/geometry.rs
- crates/cad-craft/src/domain/ids.rs
- crates/cad-craft/src/domain/mod.rs
- crates/cad-craft/src/domain/page.rs
- crates/cad-craft/src/domain/project.rs
- crates/cad-craft/src/domain/project/integrity.rs
- crates/cad-craft/src/domain/project/snapshot.rs
- crates/cad-craft/src/domain/properties.rs
- crates/cad-craft/src/domain/terminal.rs
- crates/cad-craft/src/error.rs
- crates/cad-craft/tests/project_model.rs
- crates/cad-craft/tests/serialization.rs
- crates/cad-craft/tests/value_types.rs
- docs/adr/0003-core-eletrico-e-integridade.md
- docs/internal/plans/2026-10-26-etapa-01-core-dominio.md
- docs/internal/reports/ETAPA_01_CORE_DOMINIO.md
- docs/internal/spec/ETAPA_01_CORE_DOMINIO.md

Modificados (12 arquivos):

- Cargo.lock
- README.md
- crates/cad-craft/Cargo.toml
- crates/cad-craft/src/lib.rs
- crates/cad-gui/src/main.rs
- docs/internal/memory/ARCHITECTURE.md
- docs/internal/memory/CURRENT_STATE.md
- docs/internal/memory/DECISIONS.md
- docs/internal/memory/HANDOFF.md
- docs/internal/memory/KNOWN_ISSUES.md
- docs/internal/memory/ROADMAP.md
- docs/internal/spec/README.md

Excluídos: nenhum. AGENTS, política Git, scripts existentes e configurações
permanecem preservados. Builds/rustdoc/logs locais são ignorados.

Próxima etapa: **Etapa 2 — aguardando autorização.**

**ETAPA 1 IMPLEMENTADA — AGUARDANDO REVISÃO E COMMIT MANUAL DO USUÁRIO.**
