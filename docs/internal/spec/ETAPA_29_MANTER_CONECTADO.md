# CAD-CRAFT — Etapa 29: manter conectado ao ChatGPT

**Estado: APROVADO pelo autor em 2026-11-07** (plano `docs/internal/plans/PLANO_ETAPAS_29_31.md`, decisões 1 e 2 recomendadas).

- **Base:** privado `9d1cbdf`, público `5c79524`; validação 743 Rust + 6 exemplos + 16 PowerShell.
- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE, ADR-0030 e `crates/cadcraft-gui/src/assistant/`.

## Objetivo

Hoje cada abertura do app exige um novo login no ChatGPT (IA-L02): tokens e `client_id` ficam só na memória. O autor quer a opção de continuar conectado entre aberturas.

## Requisitos

1. Caixa **"Manter conectado neste computador"** no painel do assistente, **desmarcada por padrão**. A escolha fica nas preferências (só o booleano).
2. Marcada, depois de um login bem-sucedido, guardar **somente** o refresh token e o `client_id` emitido no **Gerenciador de Credenciais do Windows**, pela crate `keyring` (dependência nova aprovada; conferir a licença no `deny.toml`). Um item só, com nome fixo (ex.: serviço `CAD-CRAFT`, conta `chatgpt` + host ID). Nunca senha, e-mail, access token, conversa ou dados do projeto. Nada em arquivo do app.
3. Ao abrir o app com a opção marcada e um item no cofre: renovar em segundo plano (`grant_type=refresh_token`), carregar a lista de modelos e mostrar "Conectado (sessão mantida)". O refresh token rotativo novo **substitui** o antigo no cofre a cada renovação, inclusive as renovações feitas durante o uso.
4. Falha na renovação (expirado, revogado, 400/401, rede): apagar o item do cofre, mostrar mensagem clara e voltar ao botão "Entrar com ChatGPT". Erro de rede transitório não apaga: mantém e oferece "Tentar de novo".
5. **"Sair"** apaga o item do cofre. **Desmarcar** a caixa também apaga, mesmo conectado (a sessão atual continua até fechar o app).
6. Cofre indisponível (Linux sem serviço de segredos, erro do Windows): a caixa fica desabilitada com a explicação; o login normal continua funcionando.
7. O acesso ao cofre fica atrás de um trait (`SessionVault` ou similar) com implementação real e uma falsa para testes. Rede e cofre só em `cadcraft-gui/src/assistant/` (regra do `AGENTS.md`).

## Testes (sem rede e sem o cofre real)

- Cofre falso: login com a caixa marcada grava refresh token e `client_id`; desmarcada não grava nada.
- Abertura com item no cofre e provedor de renovação falso: fica conectado sem abrir navegador; o token novo substitui o antigo.
- Renovação recusada: item apagado e estado "entrar"; erro de rede: item mantido.
- "Sair" e desmarcar apagam o item.
- As preferências serializadas continuam sem token, e-mail, `oaiapp_` ou "refresh" (ampliar o teste existente).
- `Debug` de qualquer tipo com o token continua sem o segredo.

## Documentação

- Adendo no ADR-0030: o que é guardado, onde, por quanto tempo (30 dias, rotativo) e como apagar.
- KNOWN_ISSUES: IA-L02 passa a "opção disponível".
- README e README.pt-BR: uma linha na seção do assistente.

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; saídas de referência idênticas; Release compilado.
- Sem teste no app nesta etapa: o teste com o login do autor fica para o fim da Etapa 31 (entrar com a caixa marcada, fechar, reabrir já conectado, sair e reabrir pedindo login).
- Relatório `docs/internal/reports/ETAPA_29_MANTER_CONECTADO.md`; HANDOFF, CURRENT_STATE e ROADMAP atualizados. Commit + push só com o OK do autor.
