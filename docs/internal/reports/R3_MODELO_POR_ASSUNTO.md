# R3 — modelo organizado por assunto

Data: 2026-11-05 (America/Sao_Paulo). Especificação: ../spec/R3_MODELO_POR_ASSUNTO.md, decisões recomendadas aprovadas. Plano/ledger: ../plans/2026-11-05-r3.md.

## Resultado

Entidades em electrical, physical, document, symbols, checks e geometry. Operações Project em pastas dos mesmos assuntos; integrity e snapshot permanecem junto do agregado por cruzarem assuntos. domain/mod.rs mantém os reexports planos; a GUI e seus imports não mudaram. Nenhuma dependência ou versão de formato alterada.

Todos os arquivos do modelo têm menos de 800 linhas (maior: project/electrical/numbering.rs, 781 linhas). Três arquivos grandes foram divididos nas fronteiras de itens existentes, sem reescrever algoritmos. Nos novos níveis, pub(super) virou pub(in crate::domain) ou pub(in crate::domain::project), mantendo o mesmo ancestral de acesso de antes. Helpers dentro dos arquivos divididos continuam privados ao módulo ancestral original. Não foram mantidas fachadas públicas de módulos antigos; o contrato de reexports planos permanece por requisito.

geometry/tolerance.rs disponibiliza domain::tolerance::{EPS_MM, nearly_equal, is_zero}. EPS_MM = 1e-6 mm. A constante de molduras usa o mesmo valor via import; nenhuma igualdade exata foi substituída. Inventário e justificativas: [R3_COMPARACOES_F64.md](R3_COMPARACOES_F64.md).

## Verificação e revisão

Validação 2026-11-05: scripts/validate.ps1 exit 0; **694 testes Rust + 6 exemplos executados + 16 PowerShell = 716**, zero falhas/ignorados. fmt, check, Clippy -D warnings e diff checks aprovados; baselines e fixtures v0–v9 preservados. Release compilado (23609856 bytes), sem executar o app.

Logs locais ignorados: .local/r3/validate.log e .local/r3/release.log. Total anterior 692 Rust; dois testes novos de tolerância. Não foram removidos/enfraquecidos testes ou regravados hashes de referência.

- Teste novo de tolerância primeiro falhou por API ausente (.local/r3/tolerance-red.log). Dois testes agora cobrem epsilon inclusivo, próximo f64 fora da fronteira, sinal, NaN/infinito e limites de zona.
- 36 testes focados passaram: tolerance, frames, macros, symbol_geometry, crate_layers e baseline_outputs; .local/r3/focused-final.log. Baselines não regenerados.
- 59 arquivos movidos são equivalentes ao snapshot pré-R3 após normalizar somente espaços, vírgulas finais de parâmetros e pub(super) para o mesmo ancestral explícito. Os três restantes foram divididos. Snapshot local ignorado em .local/r3/before.
- Links locais dos quatro guias conferidos; TOUR e ARCHITECTURE atualizados para os novos caminhos.
- Revisão independente read-only review_r3: sem achados materiais. Comparação com snapshot pré-R3: 1.096 funções preservadas (somente is_zero/nearly_equal adicionadas), 229 tipos completos equivalentes e 232 atributos Serde idênticos; exports, visibilidade, introduções e links conferidos. A validação de bytes em runtime, Release e publicação ficam sob responsabilidade do executor; nenhuma homologação GUI é declarada.

Sem execução do app ou homologação manual, conforme escopo. O teste no app fica para R5. Nenhum R4 iniciado.

## Movimentações

62 arquivos originais movidos com git mv (65 operações: macros.rs passou ao diretório de divisão; dois módulos homônimos receberam nomes de subassunto). Caminhos abaixo relativos a crates/cadcraft-model/src/.

| Origem | Destino |
|---|---|
| domain/component.rs | domain/electrical/component.rs |
| domain/device.rs | domain/electrical/device.rs |
| domain/function.rs | domain/electrical/function.rs |
| domain/representation.rs | domain/electrical/representation.rs |
| domain/connection.rs | domain/electrical/connection.rs |
| domain/connection_point.rs | domain/electrical/connection_point.rs |
| domain/connection_drawing.rs | domain/electrical/connection_drawing.rs |
| domain/network.rs | domain/electrical/network.rs |
| domain/designation.rs | domain/electrical/designation.rs |
| domain/identification.rs | domain/electrical/identification.rs |
| domain/marker.rs | domain/electrical/marker.rs |
| domain/numbering.rs | domain/electrical/numbering.rs |
| domain/part.rs | domain/physical/part.rs |
| domain/terminal.rs | domain/physical/terminal.rs |
| domain/terminal_strip.rs | domain/physical/terminal_strip.rs |
| domain/terminal_bridge.rs | domain/physical/terminal_bridge.rs |
| domain/cable.rs | domain/physical/cable.rs |
| domain/cable_core.rs | domain/physical/cable_core.rs |
| domain/conductor.rs | domain/physical/conductor.rs |
| domain/connector.rs | domain/physical/connector.rs |
| domain/physical_elements.rs | domain/physical/physical_elements.rs |
| domain/physical_values.rs | domain/physical/physical_values.rs |
| domain/document.rs | domain/document/definition.rs |
| domain/document_structure.rs | domain/document/document_structure.rs |
| domain/page.rs | domain/document/page.rs |
| domain/page_graphic.rs | domain/document/page_graphic.rs |
| domain/frame.rs | domain/document/frame/mod.rs |
| domain/asset.rs | domain/document/asset.rs |
| domain/symbol.rs | domain/symbols/symbol/mod.rs |
| domain/symbol_library.rs | domain/symbols/symbol_library.rs |
| domain/macro_def.rs | domain/symbols/macro_def.rs |
| domain/check.rs | domain/checks/check.rs |
| domain/report.rs | domain/checks/report.rs |
| domain/geometry.rs | domain/geometry/values.rs |
| domain/transform.rs | domain/geometry/transform.rs |
| domain/project/connection_drawing.rs | domain/project/electrical/connection_drawing.rs |
| domain/project/connectivity.rs | domain/project/electrical/connectivity.rs |
| domain/project/function_edit.rs | domain/project/electrical/function_edit.rs |
| domain/project/identification.rs | domain/project/electrical/identification.rs |
| domain/project/markers.rs | domain/project/electrical/markers.rs |
| domain/project/numbering.rs | domain/project/electrical/numbering.rs |
| domain/project/representations.rs | domain/project/electrical/representations.rs |
| domain/project/route_clearance.rs | domain/project/electrical/route_clearance.rs |
| domain/project/sheet_references.rs | domain/project/electrical/sheet_references.rs |
| domain/project/connectors.rs | domain/project/physical/connectors.rs |
| domain/project/parts.rs | domain/project/physical/parts.rs |
| domain/project/physical_edit.rs | domain/project/physical/physical_edit.rs |
| domain/project/physical_integrity.rs | domain/project/physical/physical_integrity.rs |
| domain/project/physical_lists.rs | domain/project/physical/physical_lists.rs |
| domain/project/physical_queries.rs | domain/project/physical/physical_queries.rs |
| domain/project/physical_wiring.rs | domain/project/physical/physical_wiring.rs |
| domain/project/clipboard.rs | domain/project/document/clipboard.rs |
| domain/project/document_edit.rs | domain/project/document/document_edit.rs |
| domain/project/document_queries.rs | domain/project/document/document_queries.rs |
| domain/project/documentation.rs | domain/project/document/documentation.rs |
| domain/project/frames.rs | domain/project/document/frames.rs |
| domain/project/macros.rs | domain/project/symbols/macros/mod.rs |
| domain/project/placement.rs | domain/project/symbols/placement.rs |
| domain/project/symbol_texts.rs | domain/project/symbols/symbol_texts.rs |
| domain/project/symbols.rs | domain/project/symbols/operations.rs |
| domain/project/checks.rs | domain/project/checks/operations.rs |
| domain/project/reports.rs | domain/project/checks/reports.rs |

## Divisões adicionais

| Arquivo movido | Conteúdo extraído | Responsabilidade |
|---|---|---|
| domain/document/frame/mod.rs | domain/document/frame/factory.rs; domain/document/frame/editing.rs | Factory frames and draft/translation operations; shared types and validation remain in mod.rs. |
| domain/symbols/symbol/mod.rs | domain/symbols/symbol/primitives.rs | Primitive bounds/transforms; shared bounds_points retains original effective visibility. |
| domain/project/symbols/macros/mod.rs | domain/project/symbols/macros/capture.rs; domain/project/symbols/macros/insertion.rs | Capture versus insertion/replay; shared transaction data stays in mod.rs. |

Criados também os mod.rs dos seis assuntos de entidades e dos cinco assuntos de operações Project; geometry/tolerance.rs; crates/cad-craft/tests/tolerance.rs; plano e dois relatórios R3. Modificados domain/mod.rs, domain/project.rs, docs/TOUR.md, docs/ARCHITECTURE.md e memória. Nenhuma entidade/API removida. Os antigos caminhos físicos foram substituídos pela tabela; nenhuma exclusão de teste, fixture ou recurso.

## Git e publicação

Trabalho em C:/Dev/CAD-CRAFT, main, origin privado Vandecao/CAD-CRAFT1. HEAD observado em 2026-11-05: 06a6cbf1e3d37d1e7679ca55d98f41f43e7ee001. R2 ainda não versionado na entrada; autor autorizou commit/push ao concluir R3, incluindo R2, e publicação nos dois repositórios.

Cópia pública verificada em C:/Dev/CAD-CRAFT-public, main, origin Vandecao/CAD-CRAFT. HEAD público inicial 79b1a48d6984d22440e31873eedfd1083f558ac8; consultas ls-remote confirmaram ambos os HEADs iniciais iguais aos respectivos main remotos. Histórico privado nunca será enviado ao público: somente arquivos selecionados serão sincronizados para o checkout público, com commit próprio e push sem força. Sem novos remotos, branches, pull/fetch, rebase ou reescrita de histórico.

Versionamento e publicação autorizados pelo autor ao concluir R3. Este relatório integra a preparação validada do commit R2+R3. O recibo local .local/r3/PUBLICATION.md registrará o resultado real de cada commit/push e sua confirmação por ls-remote; o resumo final ao autor fornecerá os links dos commits. Os prompts futuros R4–R6 e Etapa 28 permanecem fora do commit.

Esta seção registra o estado observado antes do versionamento; não inventa SHA futuro. O resultado com SHAs reais será registrado em .local/r3/PUBLICATION.md e informado ao autor. Consultar git log para os commits da entrega. Nenhum backup externo criado/verificado.

## Ocorrência durante a validação

A primeira execução passou 694 testes Rust e 6 exemplos, mas parou no Clippy por module_inception em document/document e geometry/geometry. Os módulos privados foram renomeados com git mv para definition.rs e values.rs, sem suprimir lint nem alterar funções/Serde. Clippy do modelo, todos os targets, passou após a correção (.local/r3/clippy-fix.log). A validação integral é repetida na árvore final; a tentativa inicial está em .local/r3/validate-first.log.
