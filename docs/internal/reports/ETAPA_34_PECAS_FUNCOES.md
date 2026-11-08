# Relatório — Etapa 34 (peças e funções sem duplicação)

Data: 2026-11-08. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_34_PECAS_FUNCOES.md` (aprovado; plano `PLANO_ETAPAS_32_35.md`, item 7).

## Causa

`assign_part` só reaproveitava a função do dispositivo de mesmo tipo cujos pontos estavam contidos nos da peça. No teste da partida estrela-triângulo, isso deixou dois casos sem casar e a peça criou funções novas em silêncio (K1 com duas bobinas, 23 avisos VER-006):
- funções criadas por símbolos do usuário, que tinham tipo `Other(nome do símbolo)`;
- funções com pontos "1/2".

## Entregue

- **`Project::part_assignment_preview`** (`physical/part_matching.rs`) decide, para cada função da peça:
  1. mesmo tipo e pontos contidos nos da peça → **usa** a função existente;
  2. qualquer tipo com exatamente os pontos da peça → **usa** (cobre funções feitas com símbolos sem tipo);
  3. mesmo tipo e no máximo a mesma quantidade de pontos → **usa renomeando** os pontos para os nomes da peça, em ordem natural (1, 2… 13, 14; A1, A2);
  4. senão → **nova** função.

  Os casamentos exatos são feitos antes dos renomes, para um renome nunca tomar a função que outra função da peça casaria sem mudanças.
- **`assign_part` aplica exatamente a prévia.** O delta guarda os pontos renomeados (antes/depois), e desfazer/refazer restaura o estado exato num passo de Ctrl+Z. Os pontos mantêm o ID, então vínculos de portas, conexões e referências cruzadas continuam ligados; o desenho passa a mostrar os nomes novos.
- **Prévia no editor de peças e dispositivos.** Antes de "Atribuir peça", cada função aparece como "Bobina (A1, A2) → usa -K1 · Bobina (1, 2) renomeando 1→A1, 2→A2", "→ usa …" ou "→ nova função". Nada é criado sem aparecer.
- **VER-006** mostra o tipo legível ("função Contato auxiliar da peça não representada") no lugar do nome interno. Sem funções duplicadas, a lista de dispositivos e a VER-006 deixam de repetir entradas.

## Testes

- `crates/cad-craft/tests/part_function_matching.rs` (3):
  - K1 desenhado com bobina, três polos e auxiliares 13-14/21-22: a peça do contator não cria funções novas, e não sai nenhuma VER-006;
  - bobina com pontos "1/2": a prévia propõe A1/A2; depois de aplicar, os vínculos das portas e os IDs continuam os mesmos e a folha mostra A1; desfazer devolve o projeto idêntico e refazer reaplica;
  - função que a peça tem e o dispositivo não (33-34): aparece como "nova" e vira uma VER-006 informativa.
- `parts_ui.rs`: a prévia aparece no editor antes de atribuir, sem alterar o projeto.

## Validação

`scripts/validate.ps1`: 798 testes Rust e 16 PowerShell aprovados, exit 0 (fmt, check, test, clippy `-D warnings`, `git diff --check`). Release: 24.082.432 bytes.
