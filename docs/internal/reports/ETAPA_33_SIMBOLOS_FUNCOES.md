# Relatório — Etapa 33 (símbolos de fábrica, tipo de função e nomes de pontos)

Data: 2026-11-08. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_33_SIMBOLOS_FUNCOES.md` (aprovado; plano `PLANO_ETAPAS_32_35.md`, decisões 1 e 2).

## Entregue

1. **Símbolos de fábrica v2** (`symbol_library.rs`). Contato NA com lâmina aberta; NF com lâmina apoiada no gancho; bobina retangular; botão e chave com acionador tracejado. Mostram a designação do dispositivo (-K1) e os pontos ao lado dos terminais. As identificações seguem a IEC 60947: bobina A1-A2, NA 13-14, NF 21-22, botão e chave 13-14.
   - Os IDs e as posições das portas continuam os da v1 ("1" embaixo, "2" em cima). Assim, "Atualizar símbolos do projeto" aceita a v2 em projetos antigos sem tocar nos fios.
   - Projetos existentes ficam com a v1 até esse comando (decisão 1). O comando agora também oferece os símbolos de fábrica, não só a biblioteca externa.
   - Motor, borne, sensor e alimentação passam à v2 só para mostrar a designação. O terra continua na v1.
2. **Tipo de função e prefixo no símbolo** (`function_hints.rs`). Ficam no mapa `metadata` que o símbolo já tinha (`function-kind`, `designation-prefix`), então os arquivos `.cadsym` e `.cadcraft` não mudam de formato e não há migração.
   - O editor de símbolos tem os campos "Tipo de função" (Outro, Bobina, Contato principal, Contato auxiliar) e "Prefixo".
   - A inserção usa esses dados. Símbolos sem eles continuam como antes: função com o nome do símbolo e prefixo D.
3. **Nomes de pontos** (decisão 2, `point_numbering.rs`). Uma função nova em dispositivo existente recebe os próximos pontos livres:
   - NA 13-14, 23-24…; NF x1-x2 com o próximo dígito de sequência livre;
   - relé térmico F: 95-96 / 97-98; temporizador KT: 15-16 / 15-18;
   - bobina A1-A2 (depois B1-B2); contato principal 1-2, 3-4, 5-6.

   No centro de inserção, os pontos sugeridos aparecem em campos editáveis. Pontos digitados valem para o primeiro clique; os cliques seguintes da inserção repetida recebem os próximos livres. O núcleo grava os rótulos com `place_symbol_with_point_labels`, na mesma transação desfazível.
4. **VER-021 "Designação de ponto repetida no dispositivo"** (aviso), com navegação para o dispositivo e os pontos. ADR-0026 atualizado.
5. **Seletores legíveis.** "Função não representada existente" e "Entidade existente" mostram "-K1 · Bobina (A1, A2)" (`Project::function_label`). Um campo "Filtrar por tag" fica acima dos seletores de dispositivo e de função.
6. **Editor de símbolos:**
   - leitura de X/Y do cursor no canto da área de desenho;
   - campos numéricos para os pontos da primitiva selecionada (início/fim, vértices, canto e tamanho, centro e raio, origem do texto, ângulos do arco);
   - zoom e posição mantidos ao fechar e abrir outro símbolo ("Ajustar à tela" continua disponível).

## Decisões de implementação

- As portas da v2 estão listadas de cima para baixo (porta "2" = 13, porta "1" = 14), para os pontos nascerem na ordem IEC. Os IDs internos ficaram "1"/"2" para manter a compatibilidade; só a identificação mudou.
- A família do contato (NA ou NF) vem do dígito final das identificações das portas (x3-x4 = NA, x1-x2 = NF). Para isso funcionar num símbolo do usuário, basta ele usar identificações IEC.
- Saídas de referência (`baseline_outputs.rs`): **não mudaram**. Os projetos fixos usam símbolos gravados nas fixtures, não os de fábrica, e o teste passou sem regravar. Mudou só o espelho de contatos do teste de PDF ("NA 13-14" no lugar de "1-2"), atualizado em `pdf_export.rs`.

## Testes

- `crates/cad-craft/tests/factory_symbols.rs` (6):
  - identificações IEC e designação desenhada;
  - IDs e posições das portas preservados;
  - projeto antigo com a v1 até atualizar;
  - 13-14, 23-24, 31-32, 95-96, 97-98, 15-18 e B1-B2;
  - VER-021 com navegação;
  - rótulo "-K1 · Bobina (A1, A2)";
  - tipo e prefixo no metadata.
- `crates/cadcraft-gui/tests/function_points_ui.rs` (3):
  - símbolo do usuário "bobina + KT" entra como -KT1 com função Bobina;
  - inserção repetida sugere 13-14 e 23-24; pontos digitados valem só no primeiro clique;
  - "Atualizar símbolos do projeto" leva o projeto antigo à v2.
- `symbol_editor_ui.rs`: tipo e prefixo gravados ao salvar; zoom mantido entre símbolos.

## Validação

`scripts/validate.ps1`: 795 testes Rust e 16 PowerShell aprovados, exit 0 (fmt, check, test, clippy `-D warnings`, `git diff --check`). Release: 24.028.160 bytes.
