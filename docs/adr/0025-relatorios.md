# ADR-0025 — Relatórios em páginas e arquivos

**Estado:** implementado e testado automaticamente na Etapa 23; homologação manual na Etapa 28.

## Contexto

O cadastro já fornece materiais, listas físicas, conexões, números e referências. PDF e DXF consomem a lista de exibição da página. A Etapa 23 precisa emitir seis relatórios sem duplicar o modelo elétrico.

## Decisão

- A página guarda `ReportDefinition` (grupo UUID v7, tipo, filtro, ordenação, colunas e índice). As linhas são derivadas do projeto em cada leitura. A contagem de páginas é ajustada por Gerar/Atualizar; o aviso de desatualização aparece quando a contagem necessária difere. O índice de páginas inclui as próprias páginas geradas e a contagem converge ao atualizar vários grupos.
- Os tipos são índice de páginas, dispositivos, materiais, conexões, bornes e cabos. A ordem padrão é documental no índice; nos demais, designação ou código em ordem estável. Filtros por documento, página e prefixo dos caminhos `=`/`+` usam as ocorrências cadastradas. Na lista de bornes, as colunas são **Lado A** e **Lado B**, conforme decisão expressa do usuário em 2026-11-04: o cadastro não define qual lado seria interno ou externo. Ligações sem lado atribuído aparecem em **Sem lado**.
- A tabela usa altura fixa de 6 mm, cabeçalho e rodapé repetidos, grade e corte de texto medido pela Helvetica. Referências com destino são `DisplayItem::Link`; PDF e DXF continuam usando os escritores existentes.
- Gerar/Atualizar produzem `ReportDelta` de páginas antes/depois, validado e reversível em uma `Transaction::Report`. A GUI calcula em thread, exibe progresso e cancelamento e aplica o delta na thread principal.
- CSV usa `;` e UTF-8 com BOM. XLSX usa `rust_xlsxwriter` 0.102.0, cabeçalho em negrito, larguras, filtro e valores numéricos tipados.
- `.cadcraft` passa ao formato 8 (`min_reader` 8); migração 7→8 preserva páginas antigas sem definição e fixtures v0–v7 seguem legíveis.

## Consequências e limites

As tabelas acompanham mudanças no cadastro sem gravação adicional; desatualização indica somente diferença no número de páginas, conforme decisão aprovada. O histórico protege os deltas de páginas; alterações do cadastro feitas por outros comandos seguem seu próprio histórico. O arquivo exportado é um instantâneo do momento da exportação. O PDF mantém as limitações de fonte do ADR-0023, e o DXF as do ADR-0024. Não há mapeamento implícito de Lado A/B para interno/externo.
