# Relatório técnico — Etapa 23 (relatórios em página e CSV/XLSX)

Data: 2026-11-04. Prompt aprovado: `docs/internal/spec/ETAPA_23_RELATORIOS.md` (cinco decisões recomendadas). Plano: `docs/internal/plans/2026-11-04-etapa-23.md`. ADR-0025.

## Entrega

O Core calcula índice de páginas, dispositivos, materiais, conexões, bornes e cabos do cadastro atual. A definição de cada relatório fica nas páginas; tabela, filtro e ordenação são derivados. A moldura delimita a área útil; cada página repete cabeçalho e rodapé, desenha grade e corta textos largos com reticências medidas pela Helvetica. Referências com destino geram links na lista de exibição, reaproveitada por PDF e DXF.

Gerar e Atualizar ajustam a contagem de páginas com um delta reversível. Atualizar todos recalcula índices que incluam páginas criadas por outros relatórios no mesmo lote. A GUI oferece prévia, aviso de desatualização, histórico, tarefa em segundo plano, progresso e cancelamento. Exportar Lista grava CSV `;` com BOM ou XLSX, inclusive seis abas. O `.cadcraft` usa formato 8, com migração 7→8 e fixture v8; versões antigas permanecem legíveis.

A lista de bornes mostra **Lado A**, **Lado B** e **Sem lado**. O modelo não atribui interno/externo a A/B; o usuário confirmou essa escolha durante a implementação.

## Validação

Implementação: Codex (itens 1–6 do plano). Validação final e documentação:
Claude Code, em 2026-11-04.

`pwsh -NoProfile -File scripts/validate.ps1` terminou com exit 0, sem
correções no código. Etapas executadas, todas com sucesso:
- `cargo fmt --check`;
- `cargo check --workspace --all-targets --locked`;
- `cargo test --workspace --locked`: **618 testes Rust**, sendo 597
  anteriores mais 21 novos:
  - 17 em `cad-craft/tests/reports.rs`;
  - 3 egui em `cad-gui/tests/reports_ui.rs`;
  - 1 de persistência v8 em `persistence.rs`;
- Clippy com `-D warnings`;
- `git diff --check`;
- **15 testes PowerShell**.

Ajustes mecânicos em testes anteriores:
- `persistence.rs` e `parts.rs`: as fixtures v7 agora passam por migração, e
  os testes de formato futuro usam 9;
- `placement.rs`: `DocumentEdit::InsertPage` passou a receber `Box<Page>`.

Nenhum teste foi removido nem afrouxado.

Desempenho no Release: `five_thousand_connection_rows_…` e
`five_thousand_material_rows_…` passaram com o limite de **5 s**, que mede a
geração, a tabela e o XLSX, sem a preparação do projeto. A execução completa
dos dois testes levou 18,8 s, incluindo a montagem dos projetos de 5.000
registros.

Build Release: `cargo build --release --locked -p cad-gui` concluído;
`target/release/cad-gui.exe` tem 18.607.104 bytes. O app não foi executado:
não há teste no executável nesta etapa, e o próximo é ao fim da Etapa 25.

Critérios cobertos:
- dados conhecidos dos seis tipos, filtros e somas de materiais;
- lados físicos A/B;
- 120 conexões em 3 páginas A3;
- Desfazer/Refazer, crescimento e redução sem tocar páginas de esquema;
- PDF/DXF com link e corte de texto;
- CSV com BOM e Unicode; XML do XLSX com abas e número tipado;
- migração v8;
- eventos reais da GUI.

## Limites e continuidade

- As linhas refletem o cadastro atual; o aviso de desatualização indica mudança da **quantidade de páginas**, conforme decisão aprovada. Uma mudança de conteúdo que caiba na mesma quantidade aparece diretamente, sem aviso.
- A correspondência A/B ↔ interno/externo não existe no cadastro e não foi presumida.
- O arquivo exportado é um instantâneo; o relatório em página acompanha o cadastro.
- Não houve teste no executável nesta etapa, conforme prompt. Homologação manual geral na Etapa 28.

Git: branch `main`; HEAD `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22), observado em 2026-11-04. Alterações da Etapa 23 ainda fora do HEAD; commit e push dependem de OK expresso. `docs.rar` e o rascunho da Etapa 26 não fazem parte da entrega.
