# CAD-CRAFT — Etapa 20 (Fase B): numeração automática e referências cruzadas completas

**Estado: aprovado pelo usuário em 2026-11-04 para execução pelo Codex.**
Decisões aprovadas (todas as recomendadas): (1) sequencial por prefixo como
padrão, com a opção por página/coluna (-K2.4) disponível na regra; (2) um
número de fio por rede/potencial; (3) número do fio guardado na conexão
lógica, `.cadcraft` formato 7; (4) espelho de contatos derivado, não
editável; (5) numeração ao inserir pela regra do projeto nesta etapa.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase B, Etapa 20,
depende de 13 e 18). Base de código: Etapas 18 e 19 implementadas,
validadas (556 Rust + 15 PowerShell), testadas no executável e versionadas
em `98ef54a` no `origin/main`.

## Objetivo

Numerar automaticamente dispositivos, fios (conexões) e páginas, e mostrar
as referências cruzadas completas entre função principal e auxiliares
(espelho de contatos sob a bobina, referência da bobina em cada contato) —
o equivalente à "Numeração de dispositivos", "Numeração de conexões" e
"Referências cruzadas" do EPLAN, sem copiar código ou dados de terceiros.

## O que já existe (não refazer)

- Designações `Designation` (caminho funcional `=`, local `+`, dispositivo,
  terminal) e `next_device_designation(prefixo)` (próxima livre, usada na
  inserção, na colagem e nas macros) — `domain/designation.rs`,
  `project/placement.rs`.
- Edições reversíveis de identificação: `DeviceEdit::Rename`,
  `RenameFunction`, `PageNumber`, `PageName`, `PageStructure` com
  `DeviceDelta` (`project/identification.rs`); conflitos de designação
  (`designation_conflicts`).
- Referências: `device_references`/`function_references` com
  `/página.coluna` (`reference_label`, Etapa 13), função principal/auxiliar
  (`FunctionRole`), comandos Localizar função principal/auxiliares e
  navegação (`cad-gui/src/references.rs`), texto dinâmico de símbolo
  `SymbolTextField::Reference` (Etapa 17).
- Redes derivadas da conectividade (`project/connectivity.rs`:
  `networks`, `network_for_point`), marcadores de potencial e de
  interrupção com grupos (`project/markers.rs`).
- Condutores físicos com designação (`domain/conductor.rs`, Etapa 10).
- Placeholders já registrados na faixa de opções: "Numerar dispositivos",
  "Numerar bornes", "Numerar cabos" (`cad-gui/src/commands.rs`).

## Escopo

1. **Regras de numeração (Core, sem GUI)**: estrutura `NumberingScheme`
   guardada no projeto, com formato de dispositivo (decisão 1),
   início/passo, filtro (todo o projeto, documento, páginas escolhidas,
   seleção), ordem (página → coluna → linha, ou por posição X/Y) e opção de
   manter designações marcadas como fixas (propriedade "travar
   designação" por dispositivo).
2. **Numerar dispositivos**: calcular as novas designações por prefixo da
   classificação (K, Q, F, S, M…) segundo a regra; **prévia** com tabela
   antes/depois e conflitos; aplicar em **uma transação** reversível que
   renomeia dispositivos e funções (e peças instaladas que acompanham o
   dispositivo); funções auxiliares herdam a designação do principal.
3. **Numerar fios (conexões)** (decisão 2 e 3): atribuir número a cada
   rede ou conexão conforme a regra (sequencial, ou por página/coluna),
   respeitando potenciais nomeados (o nome do potencial vence) e números
   travados; mostrar o número sobre o fio na folha (rótulo posicionável,
   sem afetar a conectividade); prévia e transação única.
4. **Numerar páginas**: renumerar as páginas de um documento em sequência
   (1, 2, 3… ou com prefixo), com prévia, sem mudar a ordem; atualiza todas
   as referências `/página.coluna`.
5. **Referências cruzadas completas** (decisão 4):
   - **espelho de contatos** sob a função principal (bobina/disjuntor):
     lista dos contatos do dispositivo com tipo (NA/NF/principal), rótulos
     dos pontos (13-14, 21-22…) e `/página.coluna` de cada ocorrência;
     contatos não representados aparecem sem referência;
   - em cada função auxiliar, a referência da função principal;
   - interrupções de potencial do mesmo grupo mostram a referência umas
     das outras;
   - tudo derivado do cadastro e recalculado ao mover, renumerar, inserir
     ou excluir; clicar na referência navega até a ocorrência.
6. **Numeração ao inserir** (decisão 5): opção nas Configurações para a
   inserção, a colagem e as macros usarem a regra do projeto (por exemplo,
   por página/coluna) em vez de só a próxima livre.
7. **Interface**: comandos Dispositivos > Numerar dispositivos,
   Conexões > Numerar conexões, Página > Numerar páginas (troca os
   placeholders); diálogo com regra, filtro, prévia e Aplicar; propriedade
   "travar designação"; mensagens de conflito.
8. **Persistência**: o que a decisão 3 exigir (por exemplo números de fio
   e regra de numeração) entra no `.cadcraft` **formato 7** / `min_reader` 7
   com migração 6→7 e fixture `etapa20-v7.cadcraft`; v0–v6 continuam
   abrindo.

## Critérios de aceitação

1. Com eventos egui reais: projeto com três páginas e dispositivos
   -K5, -K2, -K9 fora de ordem; "Numerar dispositivos" sequencial mostra a
   prévia, aplica e resulta em -K1, -K2, -K3 na ordem página → coluna;
   funções auxiliares e peças acompanham; um Desfazer volta tudo; Refazer
   repete igual.
2. Dispositivo com designação travada não muda; conflito com designação
   travada é mostrado na prévia e recusado sem alterar nada.
3. Numeração por página/coluna (se aprovada na decisão 1) gera, por
   exemplo, -K2.4 para um contator na página 2, coluna 4.
4. "Numerar conexões" dá números distintos a redes distintas e o mesmo
   número a todos os fios da mesma rede (ou um por fio, conforme a
   decisão 2); potencial nomeado (ex.: L1) mantém o nome; os rótulos
   aparecem na folha; Desfazer/Refazer exatos.
5. "Numerar páginas" renumera e as referências `/página.coluna` de
   bobinas e contatos são atualizadas.
6. Espelho de contatos: bobina -K1 com dois contatos em páginas diferentes
   mostra os dois com tipo, pontos e `/página.coluna`; mover um contato
   atualiza a referência; contato não representado aparece sem
   referência; clicar navega até o contato.
7. Salvar/reabrir conserva números de fio, regras e travas; fixtures v0–v6
   abrem; v7 versionada (se houver formato novo).
8. Testes anteriores preservados; `scripts/validate.ps1`; Release (sem
   executar); relatório `docs/internal/reports/ETAPA_20_NUMERACAO_REFERENCIAS.md`,
   ADR-0022, memória; commit + push somente após OK expresso. **Sem teste
   no executável nesta etapa** (o próximo é ao fim da Etapa 22, com
   autorização do usuário para usar o computador).

## Decisões pedidas ao usuário

1. **Formato da numeração de dispositivos**: sequencial por prefixo
   (-K1, -K2…) como padrão, com a opção por página/coluna (-K2.4)
   disponível na regra — recomendado — ou só sequencial.
2. **Número do fio**: um número por **rede** (todos os fios do mesmo
   potencial com o mesmo número) — recomendado — ou um número por fio.
3. **Onde guardar o número do fio**: na conexão lógica do fio (campo
   novo, com `.cadcraft` formato 7) — recomendado — ou só calculado na
   hora, sem persistir (perde números editados à mão).
4. **Espelho de contatos**: desenho derivado sob a função principal,
   recalculado sempre e não editável — recomendado — ou objeto de texto
   editável na folha.
5. **Numeração ao inserir pela regra do projeto** nesta etapa —
   recomendado — ou manter só a próxima designação livre na inserção.

## Fora do escopo

Numeração de bornes e de cabos (Etapas 23/24 com relatórios e
verificações); verificações de consistência completas (Etapa 24,
DEC-11-01/02, CON-001); relatórios formatados (Etapa 23); PDF com links
(Etapa 21); importação de numeração do EPLAN (Etapa 26).

## Instruções para o agente (Codex)

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md`,
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory do projeto
   CAD-CRAFT (páginas `notes/etapa-18-*`, `notes/etapa-19-*` e, se
   existir, `notes/etapa-20-*`). ADRs úteis: 0005 (condutores), 0016–0021.
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). A Etapa 19
   está em `98ef54a`; o `docs.rar` da raiz é do usuário e nunca entra em
   commit.
3. A aprovação e as cinco decisões já estão registradas no topo deste
   arquivo; não perguntar de novo. Se surgir dúvida fora delas, parar e
   perguntar ao usuário em vez de inventar requisito.
4. Criar `docs/internal/plans/<data>-etapa-20.md` com desenho técnico, seção
   "Retomada por outro agente" e lista "Andamento" numerada; marcar cada
   item ao concluí-lo com teste passando. Gravar no ai-memory a página
   fixada `notes/etapa-20-numeracao-referencias.md` com prompt, decisões,
   arquivos e próximo passo; atualizá-la ao final.
5. Seguir os padrões existentes:
   - deltas reversíveis no Core com conferência de estado e reversão em
     erro (ver `project/identification.rs`, `project/macros.rs`,
     `project/clipboard.rs`); uma renumeração = um `Transaction::*` na GUI
     (`cad-gui/src/history.rs`);
   - nomes nunca criam conectividade: o número do fio é rótulo, a rede
     continua derivada das conexões;
   - migração `.cadcraft` encadeada em `persistence.rs::migrate`, com
     fixture nova e as antigas abrindo;
   - testes egui com accesskit (ver `cad-gui/tests/macros_ui.rs`,
     `parts_ui.rs`); no Windows, Ctrl+C/X/V chegam como eventos
     `Copy/Cut/Paste` (`app.rs`);
   - prévias sob o cursor devem ser pintadas na camada acima da folha
     (ver `editor_view.rs`, camada `editor_preview`), senão a folha as
     cobre.
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; build Release
   sem executar o app; não executar o app sem autorização expressa. Se o
   disco ficar sem espaço, `cargo clean` (a pasta `target` passa de
   100 GB).
7. Atualizar relatório, ADR-0022, `DECISIONS.md`, `ROADMAP.md` (linha 20),
   `KNOWN_ISSUES.md`, `CURRENT_STATE.md`, `HANDOFF.md`; informar que está
   pronto e aguardar o OK para versionar (commit + push em `main`, sem
   força). Não iniciar a Etapa 21.
