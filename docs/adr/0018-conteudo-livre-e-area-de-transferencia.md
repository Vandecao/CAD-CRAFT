# ADR-0018 — conteúdo livre e área de transferência

Estado: implementado e testado automaticamente na Etapa 16.
Homologação manual geral na Etapa 28.

## Contexto

A Fase B pede texto e gráficos livres na folha e copiar/recortar/colar na
página e entre páginas. A primeira versão (Codex) colava símbolos como nova
ocorrência do mesmo alvo elétrico e só removia a origem do recorte na
colagem. O usuário aceitou a revisão: comportamento igual ao do EPLAN.

## Decisão

- **Conteúdo livre** é uma `Representation` sem alvo (`Unassociated`) com
  `PageGraphic`; não cria dispositivo, ponto, conexão ou rede.
- **Colar símbolo de função cria um dispositivo novo** com a próxima
  designação do prefixo, mesma classificação, tipo de função, símbolo e
  portas ligadas a pontos novos (`place_symbol`). "Colar no mesmo
  dispositivo" cria uma função nova do mesmo dispositivo.
- **Recortar remove na hora** (ocorrência e vínculos porta→ponto), numa
  transação; a primeira colagem devolve os mesmos IDs e vínculos, as
  seguintes criam cópias. Símbolo ligado a fios não é recortado.
- `SheetEditDelta` agrupa os passos (representações, vínculos, inserções com
  cadastro, aparência) de uma colagem ou recorte; aplicação atômica com
  reversão; um passo no histórico (`Transaction::Sheet`).
- Fios, junções e marcadores ficam fora da área de transferência; circuitos
  com fios ficam para as macros (Etapa 19). Limite de 10 000 itens.
- `.cadcraft` formato 4 / `min_reader` 4; área de transferência não persiste.

## Consequências

- Versões da Etapa 15 recusam arquivos da 16.
- Recorte de símbolo deixa o cadastro elétrico sem ocorrência até a colagem.
- Designação nova não herda caminhos `=`/`+` (limite da Etapa 12).
