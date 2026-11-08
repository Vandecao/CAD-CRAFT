# CAD-CRAFT — Etapa 34: peças e funções sem duplicação

**Estado: APROVADO pelo autor em 2026-11-07** (plano `docs/internal/plans/PLANO_ETAPAS_32_35.md`, item 7).

- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE; código em `cadcraft-model/src/domain/project/physical/parts.rs` (`assign_part`), `cadcraft-gui/src/part_editor/`.
- **Depende da Etapa 33** (tipo de função e nomes de pontos corretos nos símbolos).

## Problema

`assign_part` só reaproveita a função do dispositivo que tem o mesmo `kind` e pontos contidos nos da peça. Funções criadas por símbolos com outro `kind` ou pontos "1/2" não casam, e a peça cria funções novas: K1 ficou com duas bobinas, e o teste gerou 23 avisos VER-006.

## Requisitos

1. **Casamento por tipo e pontos:** cada função da peça casa com a função do dispositivo de mesmo tipo cujos pontos coincidem; se o tipo coincide e os pontos diferem, propõe renomear os pontos do dispositivo para os da peça.
2. **Prévia antes de atribuir:** o editor de dispositivo mostra o mapeamento (função da peça → função existente, nova ou renomeada); nada é criado em silêncio. Aplicar é uma operação validada com um passo de Ctrl+Z.
3. **Sem perder dados:** representações, conexões e referências cruzadas continuam ligadas depois de renomear pontos.
4. Lista de dispositivos e VER-006 sem duplicatas.

## Testes

- K1 com contatos principais (1…6), bobina A1-A2 e auxiliares 13-14/21-22 desenhados: atribuir a peça do contator não cria funções novas.
- Pontos "1/2" numa bobina: a prévia propõe renomear para A1-A2; depois de aplicar, conexões e referências continuam válidas; Ctrl+Z restaura.
- Peça com função que não existe no dispositivo: aparece como "nova" na prévia e vira VER-006 informativo.

## Entrega

- `validate.ps1`, Release, relatório `docs/internal/reports/ETAPA_34_PECAS_FUNCOES.md`, HANDOFF, CURRENT_STATE, ROADMAP. Commit + push só com o OK do autor.
