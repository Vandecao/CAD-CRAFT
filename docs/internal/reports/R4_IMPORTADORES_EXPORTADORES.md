# R4 — importadores e exportadores

Especificação aprovada: [R4_IMPORTADORES_EXPORTADORES.md](../spec/R4_IMPORTADORES_EXPORTADORES.md). Plano: [2026-11-05-r4.md](../plans/2026-11-05-r4.md). ADR: [0031](../../adr/0031-protocolo-dxf-compartilhado.md). Execução em 2026-11-05; fechamento junto com R5 em 2026-11-06.

Importadores, display list e exportadores foram organizados por assunto sem alterar APIs, formato 9, ordem dos relatórios ou bytes de saída. Nove movimentos com git mv precederam as divisões. O protocolo DXF compartilhado permanece std-only em cadcraft-model, sem criar nova dependência entre importação e exportação.

## Organização

| Assunto | Organização final |
|---|---|
| QET | mod.rs: API e registros compartilhados; read.rs: bytes/XML/limites; convert.rs e convert/: definições, folhas, ocorrências, conexões e textos; report.rs: resumo final. DiagramState e PlacementState tornam explícita a passagem do estado entre as fases, mantendo a ordem original. |
| ELMT | mod.rs: API; read.rs: limites/XML; convert.rs: símbolo; report.rs: primitivas, portas e escala. |
| Titleblock | mod.rs: API; read.rs: limites/XML; convert.rs: grade, células, campos e logos; report.rs: resumo. |
| FN1 | mod.rs: API e reexports EPLAN; read.rs: limites/XML; convert.rs: moldura, origem e zonas; report.rs: substituições de rótulos. |
| DXF | mod.rs: API e metadados; read.rs: grupos/seções/entidades/blocos; convert.rs: geometria e textos; report.rs: unidades e avisos; write.rs: escritor de molduras R12. |
| EDZ | mod.rs: API e limites; read.rs: arquivo e manifesto; convert.rs: pré-verificação de referências, peças/macros e cache; report.rs: resultado reexportado no caminho original. Leitores defensivos já separados de arquivo, peças e PXF preservados. |
| CSV de peças | parts_csv/mod.rs permanece coeso, com 131 linhas e fases de leitura, conversão e relatório identificadas, conforme permitido pela especificação. |
| Display list | display/mod.rs: tipos e Builder; page, frame, geometry, symbol, text, connection e report separados. |
| PDF | pdf/mod.rs: API e Writer; page: montagem e marcadores; geometry: pintura; images: incorporação/compressão; text: codificação; content: emissão da display list. |
| DXF de páginas | dxf_export/mod.rs: API e orquestração; entity.rs: geometria, textos e entidades. |

Reexports preservam os caminhos públicos e a visibilidade efetiva dos auxiliares. Notas de perda e aproximação continuam emitidas no ponto original de cada evento. A coleta de registros QET permanece intercalada com criação de folhas/molduras para conservar a ordem de validação e relatórios. Todos os fontes novos têm SPDX, copyright e introdução de módulo em inglês. Maior fonte dos importadores: edz/pxf.rs, 614 linhas; render/export, no máximo 353 linhas.

## Protocolo DXF e conservação

cadcraft-model::dxf expõe decode, parse_groups, unit_mm e write_group. A decodificação UTF-8/BOM/Latin1, os limites, a tabela de unidades e os diagnósticos do leitor foram conservados. O erro de protocolo é adaptado ao ImportError existente.

O leitor e os escritores de molduras/páginas compartilham somente essas primitivas. Arredondamento, escapes, codificação e metadados particulares de cada escritor continuam locais. Writer::pair delega ao escritor comum com o mesmo resultado de formatação ignorado; pair_bytes permanece intacto.

Auditoria dos literais: QET/ELMT/titleblock/FN1/EDZ/CSV sem adições ou remoções; DXF apenas desloca os quatro literais do protocolo comum, verificados literalmente no novo módulo. Os quatro corpos principais das fases QET coincidem após normalizar sintaxe de empréstimos e transferência explícita de estado. APIs/opções/resultados conservam assinaturas e campos.

A comparação dos fontes de display/PDF/DXF preservou o multiconjunto das linhas após excluir imports, wrappers de módulos, documentação, visibilidade ancestral e delegação aprovada de pair. Essa evidência estática foi complementada pelas saídas de referência e revisão independente. Evidências locais: .local/r4/import_audit.cjs, output-source-comparison.json e snapshots before.

## Validação

- 49 testes focados passaram: importadores, PDF/DXF, baselines e novos casos.
- Nove testes novos: dois para o protocolo DXF e sete para entradas corrompidas/truncadas sintéticas, um por importador, exigindo erro sem pânico.
- scripts/validate.ps1 passou com 703 testes Rust + 6 exemplos + 16 PowerShell, zero falhas/ignorados; fmt, check, Clippy com -D warnings e verificações de whitespace aprovados.
- Saídas de referência e fixtures nativas v0–v9 preservadas sem regeneração.
- Release compilado com exit 0. Conforme o escopo, o app não foi executado no R4; o roteiro nativo pertence ao R5.
- Revisão independente final sem achados materiais: fases QET, conservação dos fontes, APIs, DXF e cabeçalhos conferidos.

Evidências: .local/r4/focused.log, clippy.log, validate.log e release.log. R5 acrescentou três testes de erros tipados, chegando a 706 testes Rust; ver o relatório desse bloco para a validação integrada final.

## Amostras reais opcionais

As quatro suítes opcionais passaram em Release antes (checkout público R3) e depois (privado R4), com CADCRAFT_SAMPLES=C:/Dev/CAD-CRAFT-REFERENCIAS/samples. As 27 linhas QET tiveram contagens/perdas idênticas, desconsiderando duração. A cobertura EDZ foi byte a byte idêntica: 45 convertidos, uma recusa esperada por referência obrigatória ausente e zero erros inesperados. FN1, ELMT e titleblock também passaram. Nenhuma amostra real ou nome proprietário foi versionado.

Evidências locais: .local/r4/samples-{before,after}.log, coverage-{before,after}.md e sample-equivalence.json. O primeiro teste pós-divisão encontrou metadados antigos no target compartilhado com o checkout R3; invalidar timestamps dos fontes privados forçou recompilação e a repetição passou. Os relatórios gerados pelas suítes foram restaurados aos bytes originais para evitar alterações alheias.

## Git

Inventário conjunto: [R4_R5_ARQUIVOS.md](R4_R5_ARQUIVOS.md). Checkout privado C:/Dev/CAD-CRAFT, main, origin CAD-CRAFT1; HEAD pré-versionamento observado em 2026-11-05 e novamente em 2026-11-06: 7f5463a956aaa731c77105a4142719db76ba264a. Movimentos em .local/r4/moves.json. R4 e R5 têm publicação final conjunta autorizada nos dois repositórios, com commit público independente e sem ancestralidade privada. SHAs efetivos em .local/r4-r5/PUBLICATION.md após a publicação; este documento não inventa SHA futuro.
