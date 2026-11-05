# Etapa 15 — molduras de outros sistemas

**Estado:** implementada e validada automaticamente; pronta para revisão.
Homologação manual geral permanece na Etapa 28. O usuário determinou nesta
retomada que não sejam executados commit nem push.

Prompt: `docs/internal/spec/ETAPA_15_MOLDURAS_OUTROS_SISTEMAS.md` (aprovado).
Plano: `docs/internal/plans/2026-11-03-etapa-15.md`. Decisão: ADR-0017.
Base: `main`, HEAD `7bacd28fd5f2e86af6a1091f0e13ab52e2659b5a`
(Etapa 14, consulta em 2026-11-03), 480 testes Rust + 15 PowerShell.

## Implementado

- Core `interchange/`: importação de `.fn1` PXF, `.titleblock` XML e DXF
  ASCII; exportação DXF R12 em milímetros. Conversão para `FramePackage` e
  `ImportReport`, com motivos para itens ignorados e aproximações.
- `.fn1`: tipos por `A19`, propriedades EPLAN mapeadas para campos especiais,
  chaves desconhecidas preservadas, textos, zonas e imagens. A borda com
  dimensões da folha ancora o deslocamento de origem.
- `.titleblock`: grade, campos, variáveis e logotipos PNG/JPEG. O carimbo fica
  na base da folha escolhida; SVG é relatado e omitido.
- DXF: entidades de geometria, texto e blocos do escopo; unidades do
  cabeçalho ou mm por unidade escolhidos. Tamanho da folha pode ser definido
  na prévia. Exportação usa comentário `999` para preservar área útil e zonas
  na reimportação pelo CAD-CRAFT.
- GUI: comandos em Dados-mestre > Molduras, seletor de arquivo, opções,
  prévia, relatório e escolha de página. Confirmar usa um `FrameEdit::Batch`
  para importar imagens/moldura e atribuir a página em um passo de Desfazer;
  Cancelar não altera o projeto. Exportação DXF usa gravação atômica.
- `quick-xml` 0.42 no Core; formato `.cadcraft` permanece 3. As 19 fixtures
  `.fn1` autorizadas somam 1.606.065 bytes (maior: 156.038 bytes).

## Evidências

- `pwsh -NoProfile -File scripts/validate.ps1`: **exit 0** em 2026-11-03.
  `cargo fmt --check`, `cargo check --workspace --all-targets --locked`,
  `cargo test --workspace --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  15 testes PowerShell operacionais, `git diff --check` e
  `git diff --cached --check` passaram. **495 testes Rust**, sem falhas nem
  ignorados (inclui 3 doc-tests); 15 acima da base. Log local ignorado:
  `.local/etapa-15/validate.log`.
- Testes leem 19 fixtures versionáveis `.fn1`, 44 `.fn1` da instalação EPLAN
  local e 13 `.titleblock` de referência externa quando disponíveis. Para
  `FN1_001` e `FN1_002`, borda, divisão em 10 colunas, textos e campos
  especiais são verificados; `FN1_001` compara borda e linha do carimbo com
  a prévia SVG embutida, tolerância de 0,5 mm.
- DXF gerado é reimportado com igualdade de folha, área útil e elementos.
  Outro teste cobre `$INSUNITS`, `INSERT`, polilinha, círculo, arco, MTEXT e
  campo especial. XML com entidade externa e DXF truncado são recusados.
- Testes egui exercitam prévia, confirmação, atribuição, Cancelar,
  Desfazer/Refazer e exportação DXF. Diálogos nativos são substituídos pelo
  seletor roteirizado.
- `cargo build --release --workspace --locked`: **exit 0**. Executável
  `target/release/cad-gui.exe` de 13.112.832 bytes; SHA-256
  `724768B99549D682EB0A15EC2748F261B8AF8609C15C5E1A7D8F4A2BBEDF9C14`.

## Limites e pendências

- DXF R12 não leva imagens nem variantes traduzidas; o relatório avisa.
  Não há `.fn1`/`.titleblock` de saída, DXF binário, DWG, splines ou SVG.
- Para `.fn1` sem borda que corresponda à folha, coordenadas são mantidas e
  podem exigir ajuste manual. Valores sem equivalente viram propriedade
  especial com chave original. Itens omitidos constam do relatório.
- Não houve validação funcional no executável nesta etapa, conforme o
  cronograma de teste do app ao fim da Etapa 16. Homologação geral na 28.
- Commit, push e backup externo não foram executados. `docs.rar` não
  rastreado já existia e não integra a entrega; não adicioná-lo ao Git.
- `origin/main` consultado sem fetch em 2026-11-03 por `git ls-remote`:
  `7bacd28fd5f2e86af6a1091f0e13ab52e2659b5a`, igual ao HEAD local.

## Inventário da etapa

Criados: `crates/cad-craft/src/interchange/{mod,xml,build,fn1,titleblock,dxf}.rs`,
`crates/cad-craft/tests/interchange.rs`, 19 fixtures em
`crates/cad-craft/tests/fixtures/eplan/`, especificação e plano da Etapa 15,
este relatório e ADR-0017. Modificados: `Cargo.lock`,
`crates/cad-craft/Cargo.toml`, `crates/cad-craft/src/lib.rs`, arquivos GUI
`app.rs`, `commands.rs`, `dialogs.rs`, `frame_editor.rs`, `project_file.rs` e
`tests/frame_editor_ui.rs`; `docs/internal/memory/{CURRENT_STATE,HANDOFF,KNOWN_ISSUES,DECISIONS}.md`
e ADR-0016. Os exemplos temporários `probe_fn1.rs`/`probe_tb.rs` da sessão
interrompida foram removidos após virarem testes de integração. Nenhum arquivo
rastreado foi excluído.
