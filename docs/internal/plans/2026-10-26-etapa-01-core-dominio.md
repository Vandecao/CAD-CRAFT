# Etapa 1 — Plano de implementação

> Execução: superpowers:executing-plans, inline, com TDD e revisão independente final.
> A autorização integral/autônoma do anexo prevalece sobre pausas genéricas de skills.
> AGENTS/ADR-0002 prevalecem: sem worktree, staging, commit, alteração de refs ou push.

**Objetivo:** implementar o modelo elétrico independente da GUI, com integridade e Serde.
**Especificação:** [instrução integral](../spec/ETAPA_01_CORE_DOMINIO.md).
**Arquitetura:** `Project` controla registros por ID; entidades possuem campos privados
e consultas imutáveis. Relações inversas são consultadas pelos IDs dos proprietários,
evitando listas redundantes. Snapshot Serde em listas preserva detecção de duplicados.
**Tecnologias:** Rust existente; serde e uuid (v7/serde); serde_json somente nos testes.

## Decisões de implementação

Escolha do agente: agregado Project com registros tipados, em vez de objetos aninhados
duplicados ou referências mutáveis compartilhadas. Custo: operações relacionais
passam pelo projeto; benefícios: validação central, serialização sem ciclos de objetos.
IDs, textos, designações, decimais e geometria validam também na desserialização.
Importação completa valida referências após reunir todas as entidades, aceitando ordem
arbitrária e referências cíclicas em propriedades. Inserções individuais exigem alvos
já registrados; edição de propriedades pode completar ciclos depois da inserção.
Localizações/estruturas funcionais são caminhos de partes validadas na designação;
não são catálogos nem implementação normativa IEC. Classificações extensíveis.
Terminal físico pertence a dispositivo ou componente; ponto funcional pertence a função,
com terminal físico opcional. Associação exige proprietário compatível.
Páginas referenciam componentes/funções, sem criar representações gráficas.

## Restrições globais

- Preservar workspace e toolchain; Core sem dependência de GUI.
- Milímetros; origem inferior esquerda; X à direita, Y para cima; f64 finito.
- Somente Etapa 1. Sem conectividade, ERC, rotas, condutores, editor, persistência física.
- Homologação manual pendente; Etapa 2 não iniciada.
- Trabalhar no checkout solicitado `C:\Dev\CAD-CRAFT`, branch existente main.

## Foco da revisão

1. Dados Serde malformados devem encontrar as mesmas barreiras que construtores.
2. Duplicados em listas de importação não podem sobrescrever entidades silenciosamente.
3. IDs existentes em outro projeto não autorizam relações locais ou inserção estrangeira.
4. Falha numa operação deve preservar o agregado anterior.
5. Overflow geométrico e referências em propriedades não podem contornar validação.

## Tarefa 1 — Valores fundamentais

Arquivos: `src/error.rs`, `src/domain/{mod,ids,geometry,designation,properties}.rs`,
`tests/value_types.rs`, manifests e lockfile do Core.
Interfaces: IDs `new`, `TryFrom<Uuid>`, `FromStr`; `Point2D::new`, `Size2D::new`,
`Rect2D::new` retornam Result; Text/DesignationPart e Decimal validam entrada.
`TechnicalProperties` separa valores conhecidos (unidade documentada) e customizados;
`EntityRef` é enum de IDs. Erros estruturados via `DomainError`.
- [x] Escrever testes para unicidade/conversão/versão, designações, geometria e propriedades.
- [x] Executar testes e observar falha pela API ausente.
- [x] Implementar tipos, Serde validada e compile_fail para IDs não intercambiáveis.
- [x] Rodar `cargo test --workspace --locked`: todos os testes existentes aprovados.

## Tarefa 2 — Entidades e agregado

Arquivos: `src/domain/{project,document,page,component,device,function,terminal,connection_point}.rs`,
`src/domain/project/{integrity,snapshot}.rs`, `tests/project_model.rs`.
Interfaces: entidades `new(ProjectId, ...)->Result<Self>`; `Project::new(name)->Result<Self>`;
`insert_*(&mut self, entity)->Result<Id>`; consultas `Option<&Entity>`;
`validate()->Result<()>`; `set_properties(EntityRef, TechnicalProperties)->Result<()>`;
`add_page_element(PageId, PageElement)->Result<()>`.
- [x] Testar criação, coleções, múltiplas funções, terminais/pontos e vínculos em páginas.
- [x] Testar duplicação, proprietários ausentes/estrangeiros, vínculos incompatíveis e atomicidade.
- [x] Implementar modelos, consultas derivadas e validação em cada operação pública.
- [x] Rodar suíte completa, incluindo testes negativos.

## Tarefa 3 — Round-trip e fronteira de importação

Arquivos: `tests/serialization.rs`; refinamentos do snapshot e validadores.
- [x] Testar projeto completo (IDs, metadados, propriedades/referências), importação fora de ordem.
- [x] Testar adulteração de IDs, projetos, dimensões, designações e relações; duplicados.
- [x] Implementar/corrigir desserialização via snapshot validado, independente de formato físico.
- [x] Rodar suíte e Clippy; revisar Core/GUI e regressões.

## Tarefa 4 — Entrega e memória

Arquivos: relatório `docs/internal/reports/ETAPA_01_CORE_DOMINIO.md`; memória CURRENT_STATE,
HANDOFF, ARCHITECTURE, ROADMAP, KNOWN_ISSUES, DECISIONS; ADR-0003; README e índice spec.
- [x] Atualizar diagnóstico cad-gui, sem iniciar interface desktop.
- [x] Revisão independente somente leitura; resolver achados relevantes com regressões.
- [x] Executar `scripts/validate.ps1`, diagnóstico offline e smoke do binário.
- [x] Registrar contagens reais, limitações, inventário e SHA/data observados.
- [x] Encerrar com revisão/commit manual pendentes, sem Etapa 2.

## Registro de execução

- Base em 2026-10-26 (America/Sao_Paulo): main, HEAD
  `69f8d6c2877ba617f1c41f8c28d333b67f51e9a8`, origin/main remoto com mesmo SHA.
  Árvore inicialmente limpa; memória antiga citava 99b4161 e foi identificada como desatualizada.
- Interfaces pré-verificadas: Tarefa 2 consome IDs/valores/EntityRef da 1;
  Tarefa 3 consome agregado/validação da 2; sem conflito identificado.
- Adaptação operacional: progresso neste plano, logs locais ignorados em `.local/`;
  nenhum script de skill que crie worktree, altere Git ou commite será executado.
- Tarefa 1: valores implementados; 8 testes de integração e 1 compile_fail aprovados.
  RED observado: import domain ausente; GREEN: cargo test --workspace, exit 0.
  A resolução inicial de dependências gerou Cargo.lock; validações posteriores usam --locked.
- Tarefa 2: entidades e agregado implementados; 7 testes novos, 16 Rust totais aprovados.
  RED observado: APIs de entidades ausentes; GREEN: cargo test --workspace --locked, exit 0.
- Tarefa 3 concluída: 10 testes de serialização adicionados; RED observado por ausência
  de Serialize/Deserialize no Project. Snapshot usa listas e valida todo o agregado.
  GREEN: cargo test --workspace --locked, 26/26 Rust; Clippy -D warnings aprovado.
- Tarefa 4: scripts/validate.ps1 em 2026-10-27, exit 0, 26 Rust +15 PowerShell.
  Smoke do consumidor cad-gui, rustdoc e diagnóstico offline aprovados.
  Normalizados finais de linha de arquivos alterados conforme atributos/configuração
  existentes, sem mudar configuração Git; diff --check posterior sem avisos.
- Revisão final independente: nenhum achado acionável; não foram necessárias correções.
  Limites examinados e aceitos: entidades destacadas, unicidade por tipo de ID,
  designações repetidas/tamanhos zero, CRUD/persistência futuros e substituição em
  mapas de propriedades/metadados. Justificativa: preservam o escopo explícito e
  a integridade contratada. Custo se requisitos mudarem: novas regras/APIs e futura
  migração de schema; não declarar compatibilidade física ou conformidade IEC agora.
- Entrega concluída tecnicamente em 2026-10-27: relatório/memória atualizados,
  23 arquivos criados, 12 modificados, nenhum excluído; sem commit/staging/push.
  Homologação manual pendente. Etapa 2 aguardando autorização.
