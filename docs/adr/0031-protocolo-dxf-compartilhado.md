# ADR-0031 — Protocolo DXF compartilhado sem nova crate

Data: 2026-11-05. Estado: aceito no R4 (decisão recomendada aprovada); validação registrada no relatório do bloco.

## Contexto

Leitura de moldura DXF e escrita de moldura/página pertencem a crates diferentes. Importação e exportação já dependem de cadcraft-model; criar dependência de import para export ou o inverso quebraria as camadas. Os dois escritores repetiam a mesma codificação de pares de grupo R12, mas suas políticas numéricas e de texto são distintas e não podem ser unificadas sem alterar bytes.

## Decisão

Usar um módulo pequeno, std-only, cadcraft-model::dxf, para o protocolo de grupos e unidades: decode, parse_groups, unit_mm e write_group. Não contém Project, entidades gráficas, leitura de disco ou rede. DxfError preserva literalmente os diagnósticos anteriores de pares inválidos; o leitor adapta o erro ao ImportError público existente.

Os escritores de moldura e de página usam o mesmo write_group, mantendo o espaçamento e os LF existentes. O caminho de valores binários de texto da página mantém pair_bytes, sem passar por Unicode. Arredondamento, escape/ANSI, metadados e relatórios continuam nos adaptadores originais. Testes fixam pares, unidades, BOM/Latin-1 e mensagens de corrupção; os baselines fixam os bytes completos.

## Consequências

As dependências continuam em uma direção, verificadas por crate_layers.rs. Não há dependência/crate nova no R4 nem alteração de formato. Model recebe somente primitivas de protocolo, conforme alternativa expressamente permitida no prompt. Uma futura expansão independente poderá justificar cadcraft-dxf, mas não é necessária para estas funções pequenas. Nenhuma mudança comportamental é autorizada por este ADR.
