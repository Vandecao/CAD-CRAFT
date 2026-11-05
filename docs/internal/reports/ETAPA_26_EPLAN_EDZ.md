# Etapa 26 — importação EPLAN EDZ

Data: 2026-11-04, America/Sao_Paulo. Especificação: `docs/internal/spec/ETAPA_26_EPLAN_EDZ.md`, decisões 2–5 e ajuste ESymLib aprovados. Plano: `docs/internal/plans/2026-11-04-etapa-26.md`. Decisão: ADR-0028.

## Resultado implementado

- Core lê EDZ por assinatura ZIP/7z, sem extração em disco, e valida manifesto/referências/XML com limites. Única nova dependência direta: `sevenz-rust2 =0.24.2` (Apache-2.0), sem recursos padrão; encoder somente em testes.
- Peças `.cadpart`: fabricante/código/descrição localizada/classificação, funções e pontos; dados não equivalentes preservados com chaves originais. Não converte códigos numéricos de função em uma classificação adivinhada. Coordenadas técnicas de conexão preservadas como propriedades, separadas das portas gráficas das macros.
- Macros `.cadmacro`: primeira variante 2D, primitivas inline (linha, polilinha, retângulo, círculo, arco circular, elipse completa, texto), transformação, símbolos locais e fallback com portas conhecidas. Inserção usa o mecanismo existente com IDs novos e Undo/Redo.
- GUI Dados-mestre > Importar .edz: seletor múltiplo ou pasta, análise/prévia/progresso/cancelamento e confirmação nas bibliotecas configuradas. Conflitos exigem substituir/manter ambas/pular; manter ambas ajusta IDs e vínculos peça/macro. Relatório por pacote separa importado/parcial/ignorado/erro.
- Serialização do lote validada antes da primeira escrita; gravação atômica por arquivo. Falha/cancelamento após uma escrita informa os arquivos concluídos, sem alegar rollback de lote.

## Ajuste aprovado e limites

O levantamento inicial da especificação falava em 47 arquivos e 44 contêineres 7z. O inventário desta execução encontrou 46 `.edz` (32 7z e 14 ZIP); os números efetivos constam da cobertura. Não foram removidas ou alteradas amostras.

RIT.2383210.edz tem referência necessária a macro 2D ausente: é recusado sem gravação. O teste opcional reconhece essa recusa de validação para essa amostra comprovadamente incompleta e continua exigindo ausência de outros erros/pânicos; os testes sintéticos de itens ausentes permanecem estritos.

Nenhuma ESymLib inspecionada tinha filhos/definições. O usuário aprovou concluir com gráficos inline e caixa com portas conhecidas. Definições ESymLib permanecem pendentes de amostra demonstrativa. Não foi criada uma fixture fictícia para simular suporte ao esquema desconhecido.

Não há mapeamento comprovado para condutores PXF nem posições de múltiplas portas. São relatados como não convertidos; linhas não criam conexão elétrica por inferência. Variantes adicionais, estilos/textos dinâmicos, arcos elípticos parciais, imagens, documentos e 3D aparecem no relatório. A importação é parcial quando houver essas perdas. Conferir EDZ-L01–L08 e tabela PXF no ADR.

## Segurança

Limites: 128 MiB de entrada, 10.000 entradas, 64 MiB/item, 256 MiB de saída. Caminhos perigosos e duplicados, links, CRC inválido, truncamento, XML malformado e referência necessária ausente são recusados. Anexos fora do escopo ausentes e referências vazias são diagnosticados.

Preflight 7z anterior ao leitor limita metadados a 4 MiB, dicionários a 64 MiB (ou limite total menor), CRCs/offsets e subconjunto de cabeçalhos. A contagem de entradas é conferida após o parser de metadados da crate: limite de bytes de metadados não equivale a um teto absoluto de RAM do processo. Codecs/estruturas não suportados são recusados explicitamente. Não foi copiado código proprietário.

## Validação

**Validação final aprovada:** `pwsh -NoProfile -File scripts/validate.ps1`, **exit 0**, em 2026-11-04. São **666 testes Rust (incluindo doctests) + 15 PowerShell = 681**, zero falhas/ignorados na execução final. Os 640 testes Rust anteriores foram preservados; 26 novos. `cargo fmt --check`, `cargo check --workspace --all-targets --locked`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, testes operacionais e os dois diff checks passaram.

Evidências locais não versionadas: `.local/etapa-26-validate-complete.log`, `.local/etapa-26-coverage.log`, `.local/etapa-26-release.log`.

Build `cargo build --release --workspace --locked`: exit 0, 7m07s; `target/release/cad-gui.exe` com **19.963.392 bytes**, SHA-256 `16224A331A5DBD542B55120863BAD345624D102455F9A1578925739E24B83207`. Não executado.

Teste opcional `CAD_CRAFT_EDZ_SAMPLES=<pasta de amostras>\eplan\edz; cargo test -p cad-craft --test edz_import`: **6/6**, exit 0, 71,44 s de testes. Cobertura: **45 convertidos e 1 recusado por referência necessária ausente; zero erros inesperados/pânicos**. Totais por pacote (sem deduplicar entre arquivos): 16.783 peças, 76 macros e 5.477 definições gráficas, incluindo desenhos.

Uma tentativa da validação oficial durante compilação concorrente parou no teste anterior `dwg_converter_timeout_and_failure_are_reported`: o script fictício de falha ultrapassou os 500 ms definidos pelo próprio teste. Nenhum prazo, teste ou código DWG foi alterado. O teste passou novamente sem mudanças. Uma segunda tentativa completou Rust/Clippy, mas o teste operacional de metadados Git encontrou um arquivo transitório de `.git/refs/codex/turn-diffs/captures` removido durante a leitura. A repetição isolada passou 15/15, sem modificar o teste ou Git. Por fim, o script completo terminou com exit 0, registrado acima.

Regressões já reproduzidas e corrigidas:
- cabeçalho codificado com CRC PackInfo/SubStreamsInfo do escritor 7z;
- porta com identificação normalizada não pode impedir a caixa fallback;
- análise de conflitos e gravação devem permanecer fora da thread de desenho;
- manter ambas deve alterar somente o ID da peça e preservar o código do fabricante (RED reproduzido e GREEN nos 9 testes GUI);
- colisão de caminho sanitizado/case, substituição com nome de arquivo arbitrário e erro de serialização antes de qualquer escrita.

Os testes usam pacotes sintéticos próprios; o teste opcional `CAD_CRAFT_EDZ_SAMPLES` lê arquivos externos e gera `ETAPA_26_COBERTURA_EDZ.md` apenas com nomes de pacotes, contagens e códigos. “Definições gráficas” inclui desenhos livres, não equivale a símbolos ESymLib convertidos.

## Inventário da Etapa 26

Criados:
- `crates/cad-craft/src/interchange/edz/{mod,archive,sevenz_guard,parts,pxf}.rs`;
- `crates/cad-craft/tests/edz_import.rs`;
- `crates/cad-gui/src/edz_import_ui.rs`, `crates/cad-gui/tests/edz_import_ui.rs`;
- plano da 26, ADR-0028 e os dois relatórios da 26.

Modificados nesta etapa (além das alterações anteriores preservadas):
- `Cargo.lock`, `crates/cad-craft/Cargo.toml`, `interchange/mod.rs`;
- GUI `app.rs`, `commands.rs`, `lib.rs`, `part_editor.rs`, `macro_library.rs`, `project_file.rs`;
- especificação da 26 (ajuste aprovado), memória `CURRENT_STATE.md`, `HANDOFF.md`, `KNOWN_ISSUES.md`, `DECISIONS.md`, `ROADMAP.md`;
- página ai-memory fixada `notes/etapa-26-eplan-edz.md`.

Excluídos: nenhum arquivo de projeto. `docs.rar`, builds e amostras não integram a entrega para versionamento. As Etapas 23–25 e suas alterações anteriores foram preservadas.

## Roteiro pendente para teste no app (não executado pelo Codex)

1. Configurar pastas de peças e macros em diretórios de teste e importar um lote/pasta; conferir prévia e relatório antes da gravação.
2. Confirmar o lote e conferir peças/macros nos navegadores; repetir para exercitar substituir, manter ambas e pular. Conferir que manter ambas preserva o código do fabricante.
3. Inserir uma macro 2D e usar Ctrl+Z/Ctrl+Y; revisar visualmente geometria, portas conhecidas e avisos de aproximação.
4. Cancelar análise/gravação e conferir os resultados informados. Importar o pacote incompleto e conferir erro sem artefatos desse pacote.
5. Testar as Etapas 23–25 pelos roteiros próprios, somente com a autorização correspondente. Isso não substitui a homologação geral da Etapa 28.

## Git, revisão e próxima ação

Branch `main`, último HEAD local observado em 2026-11-04: `0b3b65c1f20951ffef5990ceb38249ba19387c07` (Etapa 22). Esta documentação e as Etapas 23–26 não pertencem a esse HEAD. Nenhum comando Git mutável foi executado. Consulta somente de leitura em 2026-11-04 (`git ls-remote origin refs/heads/main`) confirmou o mesmo SHA no origin existente `https://github.com/Vandecao/CAD-CRAFT.git`. Isso não inclui as alterações ainda não commitadas nem confirma um futuro push.

**Entrega técnica pronta para revisão e para versionamento após o teste no app e o OK expresso previstos na especificação.**

O Codex não abriu o app. Testes egui automáticos não são homologação manual. Próxima ação: Claude Code testar o app das Etapas 23–26 mediante autorização; depois, versionar somente com o OK expresso do usuário. Homologação geral permanece na Etapa 28. Não iniciar outra etapa. Backup externo independente permanece responsabilidade/pendência do usuário; não foi criado ou verificado.
