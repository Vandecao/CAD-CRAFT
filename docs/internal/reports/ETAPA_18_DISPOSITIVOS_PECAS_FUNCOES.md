# Etapa 18 — dispositivos, peças, funções e pontos

Data: 2026-11-04. Prompt aprovado: `docs/internal/spec/ETAPA_18_DISPOSITIVOS_PECAS_FUNCOES.md`. Decisões do usuário: `.cadpart` por peça, `Component` como atribuição, CSV e inserção de função não representada nesta etapa.

## Entrega

- Core: `Part` com dados técnicos, traduções, modelo de funções e validação; peças copiadas para o projeto; `Component` com ID da peça e quantidade; atribuição, remoção e atualização reversíveis, criação de funções/pontos faltantes sem representação; edição de dispositivo, função e ponto com recusas por dependências; inserção de função existente com mapeamento de portas por rótulo ou por ordem.
- Persistência e intercâmbio: `.cadpart` defensivo; CSV de peças com erros por linha e ida e volta de 100 registros; BOM derivada e CSV; `.cadcraft` formato 6, migração 5→6 e fixture v6, leitura de formatos anteriores preservada.
- GUI: pasta da biblioteca nas Configurações, janela Peças e dispositivos, cadastro e CSV, atribuição e atualização das peças do projeto, edição de dispositivo/função/ponto e associação a borne, navegação à ocorrência, lista de peças e Centro de inserção com função não representada. As edições de projeto usam o histórico comum.

## Evidências

- Testes direcionados: `cargo test -p cad-craft --test parts --locked`: 10 aprovados. `cargo test -p cad-gui --test parts_ui --locked`: 3 aprovados, incluindo atribuição com cinco funções/dez pontos por eventos egui, inserção da bobina com clique real na folha, conservação de A1/A2 e Desfazer/Refazer.
- `pwsh -NoProfile -File scripts/validate.ps1`: saída 0 na execução final, 539 testes Rust e 15 testes operacionais PowerShell aprovados; `fmt --check`, `check --workspace --all-targets`, Clippy `-D warnings` e `git diff --check` passaram. A primeira passagem detectou uma condição aninhada no seletor de nível do borne; foi corrigida antes da execução final.
- `cargo build --release -p cad-gui --locked`: saída 0; `target/release/cad-gui.exe`, 14.716.928 bytes, SHA-256 `C88F808F7F0C057847D3208AAA6BE3AD5B948171499664D144C318BCCDA5D5B1`. O aplicativo não foi executado; o teste manual do executável das Etapas 17–19 ficou para o fim da Etapa 19 mediante autorização. Homologação geral é da Etapa 28.

## Inventário da Etapa 18

Criados: `domain/part.rs`, `domain/project/{parts,function_edit}.rs`, `interchange/parts_csv.rs`, `tests/parts.rs`, fixture `etapa18-v6.cadcraft`, `cad-gui/src/part_editor.rs`, `cad-gui/tests/parts_ui.rs`, prompt, plano, ADR-0020 e este relatório.

Modificados: `Cargo.lock`, `cad-craft/Cargo.toml`, `cad-craft/src/domain/{component,connection_point,function,identification,mod,project}.rs`, `domain/project/{identification,integrity,placement,snapshot}.rs`, `interchange/mod.rs`, `persistence.rs`, `tests/persistence.rs`, `cad-gui/src/{app,backstage,commands,history,interaction,lib,preferences,state}.rs`, `panels/insertion.rs`, `docs/internal/memory/{CURRENT_STATE,DECISIONS,HANDOFF,KNOWN_ISSUES,ROADMAP}.md`. Excluídos: nenhum.

## Git e continuidade

`main`, HEAD `088637983e0933259f230f123a3777c8eadd17de` (Etapa 17), consultado em 2026-11-04. `origin/main` consultado com `ls-remote` sem fetch coincide. Etapa 18 integralmente fora do HEAD; nenhum commit ou push desta etapa foi autorizado. `docs.rar` é arquivo não rastreado do usuário e fica fora do versionamento. Próxima ação após validação: revisão do usuário e OK expresso para commit + push somente da Etapa 18. Não iniciar a 19.
