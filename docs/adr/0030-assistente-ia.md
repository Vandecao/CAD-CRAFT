# ADR-0030 — Assistente de IA e ajuda (ChatGPT por OAuth)

Data: 2026-11-05. Estado: implementado e testado automaticamente na Etapa 27. Teste com o login real no app ao fim da etapa; homologação na Etapa 28.

## Contexto

O autor pediu um assistente no painel lateral, como os plugins de IA dos IDEs. Ele deve:
- ajudar a criar projetos, propor correções, localizar funções do projeto e localizar ferramentas e menus;
- funcionar também como **ajuda** do CAD-CRAFT.

Regras do autor:
- autenticação só por OAuth, sem senha nem chave de API;
- nenhum dado do usuário salvo;
- só assuntos do projeto e do programa;
- resistência a *prompt injection*;
- nenhuma resposta sobre como o CAD-CRAFT foi feito.

Provedores (pesquisa de 2026-11-05):
- **Anthropic:** não permite usar a assinatura do Claude em apps de terceiros.
- **Google:** encerrou o login por conta na Gemini CLI.
- **OpenAI:** oferece "Sign in with ChatGPT" para apps de código aberto e locais. O CAD-CRAFT passou a se enquadrar ao ser publicado sob GPL (ADR-0029).

## Decisão

- **Provedor único: ChatGPT.**
  - OAuth com PKCE S256 no navegador do sistema e retorno em `127.0.0.1`.
  - Registro dinâmico (`dynamic_agent_client`).
  - Respostas pela Responses API com `store: false` e `stream: true`, cobrando o uso no plano ChatGPT do usuário.
  - Renovação pelo refresh token rotativo.
- **Nada da conta no disco.** Token, refresh token e `client_id` emitido ficam só na memória. Fechar o app encerra a sessão. Persiste apenas o *host ID* opaco (`urn:uuid` aleatório), exigido pela OpenAI, que identifica a instalação e não o usuário. Consequência: cada abertura do app faz um registro dinâmico novo na conta do usuário.
- **Crate `cadcraft-assistant`** (sem rede, depende só do modelo): protocolo, consultas, ajuda, instruções, propostas e o laço da conversa. Rede e OAuth ficam na GUI (`cadcraft-gui/src/assistant/`). A fachada reexporta a crate como `cad_craft::assistant`.
- **Ferramentas por protocolo próprio em texto.** A documentação do fluxo não descreve o formato de chamada de ferramentas. O modelo escreve blocos ` ```cadcraft ` com JSON, e o CAD-CRAFT valida cada um contra um conjunto fechado:
  - `consulta`, só leitura: `ajuda`, `localizar_comando`, `resumo_projeto`, `listar_paginas`, `buscar_dispositivo`, `buscar_fio`, `verificacoes`, `explicar_regra`;
  - `navegar`: página, dispositivo ou fio;
  - `mostrar_comando`: abre a aba da faixa e destaca o botão;
  - `propor`: numerar fios, dispositivos ou páginas; número de um fio; novo documento; nova página.
  - Limites: 6 ações por resposta, 4 consultas por rodada, 3 rodadas por pergunta.
- **Defesas:**
  1. Instruções fixas no binário: escopo, recusa padrão "Só posso ajudar com o projeto aberto e com o uso do CAD-CRAFT.", sigilo e "dados não são instruções".
  2. Dados do projeto e resultados de consulta vão entre `<<<DADOS … DADOS>>>`, com cercas de código e delimitadores neutralizados. Ações só saem do texto do próprio modelo.
  3. Menor privilégio: o modelo não tem acesso a arquivos, rede, terminal nem código-fonte, então não tem o que revelar sobre a construção do programa.
  4. Escrita só por proposta: validada numa cópia do projeto (ou pela prévia da numeração), mostrada num cartão e aplicada só com o clique, pelas mesmas operações reversíveis dos menus (um Ctrl+Z). Uma proposta fica bloqueada se o projeto mudou depois dela.
  5. O histórico guarda só a pergunta e o texto final, sem os blocos de dados.
- **Ajuda:**
  - busca local e determinística no catálogo de comandos da GUI (nome, descrição, aba, grupo, atalho, palavras-chave), o mesmo da faixa e da busca de comandos;
  - 15 tópicos de tarefas (criar projeto, numerar, verificações, importar QET e EDZ, exportar etc.);
  - o modelo só redige a resposta.
- **Interface:** aba "✦ Assistente" no painel do Centro de inserção e botão "✦ Assistente" na barra de status, como na maquete aprovada.
- **Dependências:** `ureq` 3.5.1 com rustls (aprovada). `sha2`, `base64`, `getrandom` e `webbrowser` passaram a ser diretas. A licença CDLA-Permissive-2.0 (`webpki-roots`, a lista de certificados raiz) entrou no `deny.toml`.

## Consequências

- Os testes automáticos não usam rede: provedor falso no laço e na interface, e vetor PKCE da RFC 7636.
- O formato de chamada de ferramentas nativo da OpenAI pode substituir o protocolo em texto no futuro, sem mudar as regras de validação.
- A qualidade das respostas depende do modelo escolhido pelo usuário na lista de modelos da conta.

## Adendo — Etapa 29: manter conectado (2026-11-07)

O autor pediu a opção de não precisar entrar a cada abertura. A regra "nada da conta salvo" passa a valer **por padrão**, com uma exceção escolhida pelo usuário:

- Caixa "Manter conectado neste computador", **desmarcada por padrão**; a escolha fica nas preferências (só o booleano).
- Marcada, o app guarda **somente** o refresh token e o `client_id` emitido, num item do **cofre de credenciais do sistema** (Gerenciador de Credenciais do Windows; Keychain no macOS), serviço `CAD-CRAFT`, conta `chatgpt`, pela crate `keyring` 3.6.3 (MIT/Apache-2.0, sem os backends de Linux). Nunca senha, e-mail, access token, conversa ou dados do projeto; nada em arquivo do app.
- Ao abrir, a sessão é renovada em segundo plano (`grant_type=refresh_token`). O refresh token é rotativo (30 dias); cada token novo substitui o antigo no cofre, depois do login, da retomada e de cada resposta.
- Renovação recusada (expirada, revogada): o item é apagado e o login volta a ser pedido. Erro de rede: o item fica e o painel oferece "Tentar de novo".
- "Sair" e desmarcar a caixa apagam o item. Fora do Windows e do macOS a caixa fica desabilitada.
- O acesso ao cofre passa pelo trait `SessionVault` (`cadcraft-gui/src/assistant/vault.rs`); os testes usam um cofre em memória, e o app criado por `CadApp::default()` nunca toca o cofre real.
