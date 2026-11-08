# CAD-CRAFT — Etapa 33: símbolos de fábrica, tipo de função e nomes de pontos

**Estado: APROVADO pelo autor em 2026-11-07** (plano `docs/internal/plans/PLANO_ETAPAS_32_35.md`, itens 5, 6, 8 e 10; decisões 1 e 2).

- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE; código em `cadcraft-model/src/domain/symbols/` (`symbol_library.rs`), `cadcraft-gui/src/state/mod.rs` (`default_prefix`, `function_kind`), `panels/insertion.rs`, `symbol_editor/`; `cadcraft-format` (símbolos e migrações).

## Problemas

- Símbolos de fábrica sem texto de designação (não mostram -K1) e com portas fixas "1"/"2"; o contato 95-96 de F1 aparece como "F1:1", igual ao polo principal; o contato NA lembra um capacitor.
- `default_prefix` e `function_kind` só conhecem os IDs de fábrica: símbolos do usuário entram com prefixo "D" e função `Other(nome do símbolo)`.
- O seletor "Entidade existente" mostra "Função 01a1b832-…".
- Editor de símbolos: zoom muda a cada símbolo novo; sem leitura de X/Y do cursor; pontos das primitivas só pelo mouse.

## Requisitos

1. **Símbolos de fábrica novos** (decisão 1): desenho no estilo IEC (contato NA com lâmina; NF com gancho; botão com acionador; bobina), texto de designação e portas nomeadas (bobina A1-A2; contato NA 13-14; NF 21-22). Nova versão dos símbolos; projetos existentes ficam com a versão anterior até "Atualizar símbolos do projeto". Formato versionado com migração e fixture, se o arquivo mudar.
2. **Tipo de função e prefixo no símbolo:** o editor de símbolos grava "Tipo de função" (bobina, contato principal, contato auxiliar, outro) e "Prefixo" (Q, K, F, M, S, X, KT…). A inserção usa esses dados; os símbolos antigos sem eles continuam como hoje.
3. **Nomes de pontos** (decisão 2, IEC 60947): ao inserir uma função em dispositivo existente, o centro de inserção sugere os próximos pontos livres conforme o tipo (13-14, 23-24…; 21-22, 31-32…; relé térmico 95-96/97-98; temporizador 15-16/15-18; bobina A1-A2) e deixa editar antes de inserir.
4. **Verificação nova:** designação de ponto repetida no mesmo dispositivo (aviso, com navegação).
5. **Seletor legível:** "-K1 · Bobina (A1, A2)", "-F1 · Contato principal (1…6)"; busca por tag.
6. **Editor de símbolos:** leitura de X/Y do cursor; campos numéricos para os pontos da primitiva selecionada; zoom e posição mantidos entre símbolos.

## Testes

- Símbolos de fábrica: designação desenhada; nomes de portas; projeto antigo abre com a versão anterior e atualiza sob comando.
- Símbolo do usuário com tipo "bobina" e prefixo "KT" entra como KT1 com função `Coil`.
- Inserir dois contatos NA em K1 sugere 13-14 e 23-24; repetir 13-14 gera o aviso novo.
- Seletor mostra tag e tipo.
- Saídas de referência regravadas (desenho dos símbolos de fábrica) e explicadas.

## Entrega

- `validate.ps1`, Release, relatório `docs/internal/reports/ETAPA_33_SIMBOLOS_FUNCOES.md`, ADR se o formato mudar, `docs/FORMATS.md`, HANDOFF, CURRENT_STATE, ROADMAP. Commit + push só com o OK do autor.
