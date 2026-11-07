# Plano — Etapas 29 a 31 (melhorias pedidas pelo autor em 2026-11-07)

**Estado: APROVADO pelo autor em 2026-11-07 ("aprovo o plano como no eplan"), com as decisões recomendadas 1–5 e a grade em toda a área, como no EPLAN (decisão 6).** Prompts: `docs/internal/spec/ETAPA_29_MANTER_CONECTADO.md`, `ETAPA_30_ZONAS_MOLDURA.md`, `ETAPA_31_AREA_TRABALHO.md`.

Base: privado `9d1cbdf`, público `5c79524` (Etapa 28 aceita). Pedido do autor: imagens 1–3 e três vídeos. Os vídeos foram vistos quadro a quadro com o ffmpeg (instalado pelo winget com autorização do autor):
- `2026-11-07 12-17-05.mp4` (41 s): EPLAN 2027;
- `2026-11-07 12-18-38.mp4` (32 s): CAD-CRAFT, página "3 · Especial";
- `2026-11-07 13-29-05.mp4` (44 s): QElectroTech (enviado depois; o primeiro arquivo, `2026-11-02 20-54-11.mp4`, era uma gravação antiga do CAD-CRAFT).

## O que os vídeos mostram

| Aspecto | EPLAN | QElectroTech | CAD-CRAFT hoje |
|---|---|---|---|
| Fundo da área de trabalho | Branco uniforme, dentro e fora da moldura | Branco uniforme | Folha branca sobre fundo cinza; partes da moldura fora da folha ficam no cinza |
| Origem e coordenadas | (0,0) no canto inferior esquerdo da moldura; fora dela as coordenadas continuam, inclusive negativas (vídeo: "Y: -60,05 mm", "X: -1014,41 mm") | A folha é a moldura; pouco espaço além dela | (0,0) no canto inferior esquerdo da folha, onde a moldura é ancorada; a folha delimita a área |
| Grade | Pontos muito finos, espaçamento real em mm, desenhados em toda a área visível; com zoom distante a grade **some** e só a moldura aparece | Pontos finos **só dentro da área de desenho da moldura**; fora dela, branco sem pontos | Pontos em toda a área; ao afastar, o programa multiplica o espaçamento e a tela fica coberta de pontos até em 2,3% de zoom |
| Cursor | Linhas finas cinza, horizontal e vertical, atravessando toda a área e acompanhando o cursor em qualquer zoom | Linhas finas azuladas, iguais às do EPLAN | Não existe |
| Colunas e linhas da moldura | — | Colunas 1–17 e linhas A–H com divisões em todos os lados; ao mudar a quantidade de colunas, as divisões são redesenhadas | Divisões de coluna fixas; linhas sem divisões |

## Diagnóstico

| # | Pedido | Causa encontrada no código |
|---|---|---|
| 1 | Manter conectado no ChatGPT | Por decisão da Etapa 27 (IA-L02), tokens e `client_id` ficam só na memória; fechar o app encerra a sessão |
| 2 | Colunas/linhas da moldura | (a) O editor mantém o "tamanho uniforme" ao mudar a quantidade: 4 × 40 mm deixa 40 mm sobrando; 8 linhas × 40 mm = 320 mm não cabe na área útil de 252 mm e o núcleo recusa com a mensagem genérica "zonas da moldura inválidas ou fora da área útil". (b) Nas molduras de fábrica, as divisões entre colunas são linhas fixas desenhadas na moldura; só os números vêm das zonas. Por isso a divisão da coluna removida continua lá, sem número. Linhas (fileiras) não têm divisões desenhadas |
| 3 | Área de trabalho | A tela pinta a folha de branco sobre fundo cinza (`PAPER` sobre `CANVAS`). A folha tem tamanho próprio, independente da moldura; na página "Especial" (500 × 250) a moldura A3 (420 × 297) fica parte fora da folha. A origem já é o canto inferior esquerdo da folha, onde a moldura é ancorada |
| 4 | Grade e cursor | A grade só espaça os pontos ao afastar o zoom (mínimo ~8 px) e nunca os oculta. Não há linhas-guia acompanhando o cursor |

## Etapas propostas

### Etapa 29 — Manter conectado (prioridade do autor) · pequena

- Caixa "Manter conectado neste computador", **desmarcada por padrão**, no painel do assistente.
- Marcada: guarda **só** o refresh token e o `client_id` no cofre de credenciais do Windows (Gerenciador de Credenciais), cifrado pela conta do Windows. Nada em arquivo do app, nada de senha, e-mail ou conversa.
- Ao abrir o app: renova a sessão em segundo plano com o refresh token (rotativo; o novo substitui o antigo no cofre). Se falhar (expirado, revogado), apaga do cofre e pede login.
- "Sair" apaga do cofre. Desmarcar a caixa também.
- Testes sem rede: cofre falso; nenhum segredo nas preferências; "Sair" limpa; falha de renovação limpa.
- Atualizar ADR-0030 (adendo), IA-L02 e o README (o que é guardado e onde).

### Etapa 30 — Zonas da moldura (colunas e linhas) · média

- **Editor:** ao mudar a quantidade, o tamanho uniforme é recalculado para preencher a área útil (botão "Distribuir na área útil" para refazer depois de ajustes manuais); a soma aparece ao lado ("8 × 31,5 mm = 252 mm de 252 mm").
- **Mensagens claras** no lugar da genérica: "As 8 linhas somam 320 mm, mas a área útil tem 252 mm".
- **Divisões geradas pelas zonas:** marcas de coluna e de linha (e a faixa de rótulos) desenhadas a partir das zonas, como os números já são. Funciona para colunas e linhas, em cima/embaixo e à esquerda/à direita.
- **Formato `.cadcraft` 10** (e `.cadframe` correspondente), com migração: nas molduras existentes, as linhas fixas que coincidem exatamente com as divisões das zonas são trocadas pelas divisões geradas. Fixture v10 nova; as saídas de referência (PDF/DXF) devem ficar idênticas, porque o desenho resultante é o mesmo.
- Molduras de fábrica passam a usar só as divisões geradas.

### Etapa 31 — Área de trabalho, grade e cursor · média

- **Fundo uniforme, como no EPLAN:** a área de trabalho é branca por inteiro, sem a folha clara sobre fundo cinza. A moldura é o limite visual; a área continua além dela, com coordenadas negativas.
- **A folha segue a moldura:** ao escolher uma moldura, a página adota o tamanho dela (operação reversível). Páginas existentes com tamanho diferente mostram o aviso com o botão "Ajustar folha à moldura". Sem moldura, a folha aparece só como um contorno fino.
- **Origem:** (0,0) continua no canto inferior esquerdo da moldura de cada página; a barra de status mostra X/Y nesse sistema.
- **Grade como no EPLAN:** pontos finos no espaçamento real (4 mm etc.), em toda a área visível; somem quando ficariam a menos de ~6 px na tela e voltam ao aproximar. O programa deixa de multiplicar o espaçamento.
- **Linhas-guia do cursor como no EPLAN:** linhas finas cinza, horizontal e vertical, atravessando toda a área e acompanhando o cursor em qualquer zoom (encaixadas na grade quando o encaixe está ligado); somem quando o mouse sai da área de trabalho. Opção em Visualização, ligada por padrão.
- PDF, DXF e formato não mudam (só a tela).

## Ordem, testes e entrega

- Ordem: 29 → 30 → 31. Cada etapa com seu prompt, `validate.ps1`, Release, relatório e memória; commit + push só com o OK de cada uma.
- Teste no app: um ao fim da 31, cobrindo as três (o "manter conectado" precisa do seu login: entrar, fechar, reabrir já conectado, sair).

## Decisões pedidas ao autor

1. **Cofre:** Gerenciador de Credenciais do Windows pela crate `keyring` (MIT/Apache; também funciona em macOS e Linux) — recomendado — ou DPAPI direto pela `windows-sys` (só Windows). Ambas são dependência nova.
2. **Padrão da caixa "Manter conectado":** desmarcada (recomendado, por ser exceção à regra "nada da conta salvo") ou marcada.
3. **Zonas:** recalcular o tamanho ao mudar a quantidade (recomendado) ou manter o tamanho e só avisar.
4. **Página sem moldura:** contorno fino da folha (recomendado) ou nada.
5. **Linhas-guia:** ligadas por padrão (recomendado) ou desligadas.
6. **Onde desenhar a grade:** em toda a área, sumindo com zoom distante (EPLAN) — recomendado, porque a área de trabalho continua além da moldura — ou só dentro da área de desenho da moldura (QElectroTech).
7. **Quem implementa:** Codex com teste no app por mim (como R2–R6) ou eu do início ao fim.
