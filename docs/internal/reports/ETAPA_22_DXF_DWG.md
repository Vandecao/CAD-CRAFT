# Relatório — Etapa 22 (exportação DXF e DWG)

Data: 2026-11-04. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_22_DXF_DWG.md`, aprovado com as cinco decisões recomendadas. Plano: `docs/internal/plans/2026-11-04-etapa-22.md`. ADR-0024.

## Entregue

| Item | Onde |
|---|---|
| Camada por item na lista de exibição | `display.rs` (`DisplayLayer`: Moldura, Símbolos, Fios, Textos, Referências, Junções, Guias) |
| DXF R12 de página | `cad-craft/src/dxf_export.rs` (`export_dxf_list`, `export_dxf_pages`, `dxf_file_names`, `DxfReport`) |
| Texto ANSI 1252, `\U+XXXX`, `%%` literal | `dxf_export.rs::encode_text` |
| DWG por conversor externo | `cad-gui/src/dwg.rs` (`converter_problem`, `oda_arguments`, `run_with_timeout`, `convert_to_dwg`) |
| Diálogo | `pdf_export.rs`: modos Exportar DXF e Exportar DWG, pasta de destino, "abrir pasta ao terminar", versão DWG 2000/2007/2013/2018, aviso sem conversor |
| Configurações | campo "Conversor DWG (ODA File Converter)" (`preferences.dwg_converter`) |
| Comandos | Início > Sessão e Arquivo: Exportar DXF e Exportar DWG |
| Testes comuns | `cad-craft/tests/output_common/` (cenário de 3 páginas e projeto de escala, compartilhados com o PDF) |

Não houve dependência nova e o formato do `.cadcraft` não mudou (continua no 7).

## Validação automática

`pwsh -NoProfile -File scripts/validate.ps1` (2026-11-04) terminou com exit 0. Etapas executadas, todas com sucesso:
- `cargo fmt --check`;
- `cargo check --locked`;
- `cargo test --locked`: **596 testes Rust** antes do teste no executável (597 depois das correções), sendo 586 anteriores mais 10 novos:
  - 6 em `cad-craft/tests/dxf_export.rs` (mais 1 após o teste no executável: `wire_number_sits_beside_the_wire`);
  - 4 egui em `cad-gui/tests/cad_export_ui.rs`;
- Clippy com `-D warnings`;
- `git diff --check`;
- **15 testes PowerShell**.

O teste de PDF passou a usar o cenário comum, sem mudar as verificações. Nenhum teste foi removido.

Critérios cobertos por teste:

| Critério | Teste |
|---|---|
| 1. Cabeçalho em mm, extensão 420 × 297, camadas por categoria, textos esperados; o importador da Etapa 15 lê o arquivo | `page_dxf_has_units_extent_layers_and_expected_texts` |
| 2. Pontos e origens de texto iguais aos da lista de exibição (0,001 mm); imagem relatada | `geometry_matches_the_display_list` |
| Preenchimento em `SOLID`, tracejado, texto girado e espelhado | `fill_dash_and_text_orientation_are_written` |
| 3. ç/ã em 1252; Ω como `\U+03A9`, relatado | `accents_are_ansi_and_other_characters_are_escaped` |
| 4. Nomes seguros e únicos; alcance, vazio, cancelamento | `file_names_are_safe_unique_and_scope_is_respected` |
| 5. Conversor falso: argumentos `ACAD2013 DWG 0 1 *.DXF`, DWG no destino, sem temporários; tempo limite e falha relatados | `dwg_export_runs_the_configured_converter_and_cleans_up`, `dwg_converter_timeout_and_failure_are_reported` |
| 6. Diálogo com cliques reais: página ativa e projeto; DWG desabilitado sem conversor | `dxf_dialog_exports_active_page_then_project`, `dwg_is_disabled_without_converter` |
| 7. 200 páginas | `two_hundred_pages_export_dxf_within_the_time_limit` |

Desempenho (200 páginas, 7,3 MiB de DXF):

| Build | Tempo | Limite |
|---|---|---|
| Release | **1,45 s** | meta de 10 s |
| Debug | 9,6 s | 120 s |

Build Release: `cargo build --release --locked -p cad-gui` concluído; `target/release/cad-gui.exe` tem 16.565.248 bytes (16.526.848 após as correções do teste no executável).

## Teste no executável (Etapas 20–22) — 2026-11-04

Feito pelo agente com autorização do usuário, no Release. O projeto de teste
foi gerado fora do repositório (`teste-etapas-20-22.cadcraft`: 3 páginas A3
com moldura e logotipo, bobinas -K7/-K3 ligadas por um fio, contato da bobina
na página 2, interrupções nas páginas 2 e 3) por um teste temporário, apagado
em seguida.

| Verificação | Resultado |
|---|---|
| Abrir o projeto; moldura, logotipo, bobinas, referência `/2.2` e espelho "NF 1-2 /2.2" sob a bobina | Passou |
| Numerar dispositivos: prévia -K7 → -K1 e -K3 → -K2, Aplicar, painel atualizado | Passou |
| Ctrl+Z volta a -K7/-K3; Ctrl+Y reaplica | Passou |
| Numerar páginas com prefixo "E": E1, E2, E3; referências na folha passam a "/E2.2" | Passou |
| Propriedades da conexão: número "L1", travado, registrado no histórico; rótulo posicionado em X/Y | Passou |
| Numerar conexões mantém o travado ("Rede L1: 1 fio(s), antes L1") | Passou |
| Exportar PDF (projeto): "3 página(s), 5 link(s) de referência"; arquivo com 3 páginas, 5 anotações de link e marcadores | Passou |
| Exportar DXF (projeto): 3 arquivos `…_Esquema elétrico_E1..E3.dxf`, 155 entidades, 3 imagens trocadas por caixa; camadas por categoria | Passou |
| Exportar DWG sem conversor: botão desabilitado e aviso "Configure o conversor DWG…" | Passou |
| Imprimir | Não testado, por escolha (precisa de impressora; coberto pelo teste egui com o sistema substituído) |
| Conversor DWG real | Não testado (ODA File Converter não instalado) |

**Defeitos encontrados e corrigidos:**
1. O número do fio ficava centralizado **sobre** o traço. Num fio vertical o
   "1" se confundia com a linha. Agora fica à direita do trecho vertical ou
   acima do horizontal, pela mesma regra na tela, no PDF e no DXF
   (`display::wire_number_placement`, teste
   `wire_number_sits_beside_the_wire`).
2. Na tela, o número do fio tinha tamanho fixo de 11 px e não acompanhava o
   zoom. Agora é texto da folha, com 2,5 mm.
3. As setas "→" (e outros símbolos) apareciam como quadrados, porque as
   fontes padrão do egui não as têm. Isso afetava a prévia da numeração e
   textos antigos do editor físico. Agora a Segoe UI Symbol do sistema entra
   como fonte reserva (`theme::install_symbol_font`).

Revalidação após as correções: `scripts/validate.ps1` exit 0, com **597 Rust
+ 15 PowerShell**. Release recompilado, com 16.526.848 bytes. As três
correções foram conferidas de novo no executável.

**Observações:**
- O texto do espelho, posicionado abaixo do canto esquerdo da bobina, cruza
  o fio que sai dela (posição herdada da Etapa 13).
- Na primeira saída do app, o clique em Confirmar não fechou a janela. Na
  segunda vez fechou normalmente.
- A pasta `C:\cad-craft-teste22`, com os 3 DXF do teste, não pôde ser
  removida pelo agente: a proteção de caminhos do sistema bloqueou a
  exclusão. Ela pode ser apagada pelo usuário.

## Limitações

Ver `KNOWN_ISSUES.md` (DXF-001 a DXF-006, DWG-001 e DWG-002). Os principais:
- espessura pelo nome da camada (R12);
- imagens como caixa;
- DWG só com o ODA File Converter instalado pelo usuário; o conversor real não foi testado.

Validação automática não é homologação manual, que fica para a Etapa 28.

## Git

`main`, HEAD `1fe365998aada547d48bc84a01a4b770e86f0930` (Etapas 20–21), observado em 2026-11-04. A Etapa 22 está fora do HEAD; o `docs.rar` é do usuário. Commit + push somente após o OK expresso.
