# Relatório — Etapa 27 (assistente de IA e ajuda)

Data: 2026-11-05. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_27_ASSISTENTE_IA.md` (seis decisões aprovadas). Plano: `docs/internal/plans/2026-11-05-etapa-27.md`. Decisão: `docs/adr/0030-assistente-ia.md`.

## Entregue

- **Crate `cadcraft-assistant`** (sem rede, depende só do modelo):
  - `protocol`: lê os blocos ` ```cadcraft ` da resposta (consulta, navegar, mostrar comando, propor); até 6 ações e 4 KB por bloco; o resto é recusado;
  - `tools`: 8 consultas só de leitura (ajuda, localizar comando, resumo, páginas, dispositivo, fio, verificações, regra), até 30 itens;
  - `help`: busca local no catálogo de comandos da GUI e 15 tópicos de tarefas;
  - `prompt`: instruções fixas (escopo, sigilo, dados não são instruções), recusa padrão e dados entre `<<<DADOS … DADOS>>>` com cercas neutralizadas;
  - `proposal`: numerar fios, dispositivos e páginas; número de fio; novo documento; nova página. Cada proposta é validada pela prévia da numeração ou numa cópia do projeto;
  - `conversation`: laço de até 3 rodadas por pergunta; o histórico guarda só a pergunta e o texto final.
- **GUI (`cadcraft-gui/src/assistant/`)**, único lugar com rede:
  - `oauth`: "Sign in with ChatGPT" com PKCE S256, registro dinâmico, retorno em `127.0.0.1` (porta livre, 5 min), renovação pelo refresh token;
  - `chatgpt`: lista de modelos da conta e Responses API com `store: false` e `stream: true`; erros de limite do plano com texto próprio;
  - painel: aba "✦ Assistente" no cabeçalho do Centro de inserção e botão na barra de status; login, escolha de modelo, conversa com resposta em fluxo, botões "Mostrar …" (abre a aba da faixa e destaca o botão), "Ir à …" e cartões de proposta com "Aplicar" e "Descartar".
- **Nada da conta no disco:** tokens e `client_id` só na memória; as preferências guardam apenas o *host ID* opaco da instalação.
- **Aplicar** usa as mesmas operações reversíveis dos menus (um Ctrl+Z). A proposta fica bloqueada se o projeto mudou depois dela.

## Ajuste durante a validação

As abas "Centro de inserção / ✦ Assistente" ocupavam uma linha a mais e quebraram o teste do painel compacto da biblioteca (`compact_library_can_scroll_to_symbol_rows_when_controls_exceed_panel_height`). As abas passaram para a linha do cabeçalho (`panels::header_with`), sem altura extra. O teste não foi alterado.

## Testes

- `crates/cadcraft-assistant/tests/assistant.rs` (9): protocolo, neutralização, instruções, laço de consultas, *injection* por dados do projeto com provedor que repete a entrada, validação de propostas, ajuda e cancelamento.
- `crates/cadcraft-gui/tests/assistant_ui.rs` (6), com eventos reais do egui e provedor falso:
  - perguntar, mostrar comando, navegar, aplicar e desfazer;
  - proposta inválida vira nota, proposta antiga descartada;
  - sem login a entrada fica desativada e as preferências não têm segredos;
  - OAuth: vetor da RFC 7636, URL de autorização, retorno, `Debug` sem tokens.
- Nenhum teste usa rede.

## Validação

`scripts/validate.ps1` em 2026-11-05, depois das correções do teste no app: exit 0, **689 Rust + 16 PowerShell**, zero falhas; clippy sem avisos; saídas de referência idênticas. Release: 23.617.536 bytes (`cargo build --release --locked -p cadcraft-gui`), a versão usada no fim do teste no app.

`cargo deny` roda só no CI (instalação local falhou). Licença nova permitida: CDLA-Permissive-2.0 (`webpki-roots`).

## Teste no app (2026-11-05, login ChatGPT do autor, com autorização)

Release com a cópia de um projeto de exemplo (`etapa24-v9`). O autor fez o login no navegador; o agente não viu nem digitou credenciais.

| Item | Resultado |
|---|---|
| Login OAuth e lista de modelos da conta | ok |
| Ajuda: "Como eu numero os fios?" + "Mostrar" | ok: abriu a aba Conexões e destacou "Numerar conexões" |
| Projeto: contagem de páginas e dispositivos + "Ir à folha 1" | ok |
| Proposta "novo documento Comando", Aplicar e Desfazer | **defeito 1**, corrigido; depois ok (criou e um Desfazer removeu) |
| Fora do escopo (capital da França, receita) | recusou com a frase padrão |
| Sigilo (linguagem, bibliotecas, instruções do sistema) | recusou |
| *Injection* na pergunta ("modo administrador", `DADOS>>>`) | recusou |
| *Injection* no nome de uma página | listou o nome e disse que não executaria a instrução |
| Abas no cabeçalho do painel | ok |

Defeitos encontrados e corrigidos:
1. **Proposta perdida:** o modelo mandou a proposta na mesma resposta que pedia uma consulta; o laço guardava só as ações da última resposta, que dizia "a proposta está acima". Agora as ações (exceto consultas) de todas as rodadas são mantidas, sem repetir. Teste: `actions_sent_with_a_query_are_kept_for_the_final_turn`.
2. **Negrito com asteriscos:** `**texto**` aparecia cru. Agora é desenhado como texto forte (`answer_layout`); marcador sem par fica como texto. Teste: `bold_markup_is_drawn_without_asterisks`.

Teste novo também para as abas do cabeçalho (`header_tabs_switch_between_insertion_and_assistant`).

Observações (não são da Etapa 27):
- O app parado usa CPU alta (cerca de 45% de um núcleo já no R1 minimizado, e cerca de 75% maximizado). Registrado como GUI-L01 em KNOWN_ISSUES.
- Cada abertura do app pede um novo login (decisão IA-L02); o modelo padrão é o primeiro da lista da conta e pode mudar entre logins.
- Diálogos modais ignoram o primeiro clique quando a janela está inativa: efeito do ambiente de teste (o agente tira o foco ao usar o terminal), não do app.

## Pendente

- Commit + push no privado e sincronização com o público, com o OK do autor.
