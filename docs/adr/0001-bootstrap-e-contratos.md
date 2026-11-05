# ADR-0001 — Bootstrap e contratos estruturais

Data: 2026-10-26. Estado: aceito conforme Prompt 0.

## Contexto

Ambiente anterior perdido. Necessário recuperar continuidade sem antecipar
funcionalidades cujos prompts ainda não foram recebidos.

## Decisão

Workspace Rust com Core `cad-craft` independente e consumidor `cad-gui`.
egui/eframe é a escolha futura para GUI nativa. Não adicionar bibliotecas nem
criar módulos vazios antes de necessidade autorizada. Milímetros e eixos
cartesianos, UUID v7 tipado, separação de entidades e conectividade conforme
ARCHITECTURE.md. Histórico transacional e persistência versionada com migrações.
Memória em Markdown no Git. Toolchain e lockfile versionados.

Decisão operacional do agente: binário diagnóstico de console para comprovar
linkedição; não constitui interface CAD. Branch main existente e origin
existente preservados. Sem nova conexão ou novo repositório.

## Consequências

Reconstrução simples e Core testável sem GUI. Integração egui/eframe e testes de
domínio ficam para etapas autorizadas. UUID, schemas e formatos não recebem
implementações prematuras. Nenhuma função planejada é contabilizada como pronta.
