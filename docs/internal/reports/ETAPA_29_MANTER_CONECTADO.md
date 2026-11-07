# Relatório — Etapa 29 (manter conectado ao ChatGPT)

Data: 2026-11-07. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_29_MANTER_CONECTADO.md` (aprovado). Plano: `docs/internal/plans/PLANO_ETAPAS_29_31.md`.

## Entregue

- Caixa **"Manter conectado neste computador"** no painel do assistente, desmarcada por padrão, com a explicação ao passar o mouse. A escolha fica nas preferências (`assistant_keep_signed_in`, só o booleano).
- Módulo `cadcraft-gui/src/assistant/vault.rs`:
  - trait `SessionVault` (`available`, `load`, `save`, `clear`);
  - `OsVault`: Gerenciador de Credenciais do Windows (Keychain no macOS) pela crate `keyring` 3.6.3, item `CAD-CRAFT` / `chatgpt` com `{client_id, refresh_token}` em JSON; em outros sistemas, indisponível;
  - `MemoryVault` para testes;
  - `Saved` com `Debug` sem segredos.
- Fluxo:
  - login com a caixa marcada grava o item; status "sessão mantida neste computador";
  - ao abrir o app (`CadApp::new`), `assistant_restore` renova a sessão em segundo plano sem abrir o navegador e carrega os modelos;
  - o refresh token rotativo novo substitui o antigo no cofre depois do login, da retomada e de cada resposta;
  - renovação recusada apaga o item e pede login ("expirou ou foi revogada; entre de novo"); erro de rede mantém o item e mostra "Tentar de novo";
  - "Sair" e desmarcar apagam o item; desmarcar não encerra a sessão atual;
  - cofre indisponível: caixa desabilitada com a explicação.
- Segurança dos testes: o app criado por `CadApp::default()` usa um cofre em memória indisponível; só `CadApp::new` (o executável) instala o cofre real. A suíte nunca lê nem apaga a credencial do desenvolvedor.

## Dependência nova (aprovada no plano)

`keyring = "=3.6.3"`, sem recursos padrão, com `windows-native` e `apple-native`. Licença MIT OR Apache-2.0. O `Cargo.lock` só ganhou pacotes (keyring, byteorder, windows-sys 0.60, security-framework e dependências), sem mudar versões existentes. Escolhida a 3.6.3, estável e com a API simples (`Entry`); a 4.x mudou a arquitetura em várias crates.

## Testes

`crates/cadcraft-gui/tests/assistant_session.rs` (7), sem rede e sem o cofre real:
- retomada sem navegador, com o token rotativo substituindo o antigo, e rotação durante o uso salva;
- opção desligada: nada retomado nem salvo;
- recusa apaga, erro de rede mantém e oferece "Tentar de novo";
- "Sair" e desmarcar apagam; a sessão atual continua ao desmarcar;
- cofre indisponível: nada acontece;
- preferências sem token, `client_id` ou "refresh";
- `CadApp::default()` não usa o cofre do sistema.

Mais 1 teste de unidade em `vault.rs` (JSON e `Debug` sem segredo). Os 7 testes do painel (`assistant_ui.rs`) continuam passando.

## Validação

`scripts/validate.ps1` em 2026-11-07: exit 0, **751 Rust + 6 exemplos + 16 PowerShell** (antes 743 + 6 + 16; 8 testes novos), zero falhas; clippy sem avisos; saídas de referência idênticas. Release: 23.820.800 bytes, não executado (o teste no app fica para o fim da Etapa 31).

## Pendente

- Teste no app com o login do autor ao fim da Etapa 31: entrar com a caixa marcada, fechar, reabrir já conectado, "Sair" e reabrir pedindo login; conferir o item no Gerenciador de Credenciais do Windows (Credenciais do Windows → genéricas → `CAD-CRAFT`).
- Commit + push só com o OK do autor.
