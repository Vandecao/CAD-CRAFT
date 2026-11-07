# Relatório — Etapa 31 (área de trabalho, grade e cursor como no EPLAN)

Data: 2026-11-07. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_31_AREA_TRABALHO.md` (aprovado; grade em toda a área, como no EPLAN). Referência: vídeos do EPLAN e do QElectroTech vistos com ffmpeg (tabela no plano `PLANO_ETAPAS_29_31.md`).

## Entregue

- **Fundo uniforme:** com uma página aberta, a área de trabalho tem uma cor só (`theme::WORK_AREA`, branco); não há mais a folha branca sobre o fundo cinza. A tela de boas-vindas (sem página) mantém o fundo anterior.
- **A moldura é o limite:** com moldura, só a moldura é desenhada; sem moldura, a folha aparece como um contorno fino (o retângulo de margem provisório saiu da tela; o PDF continua igual).
- **Origem:** (0,0) continua no canto inferior esquerdo da moldura (ancorada na origem da página); a barra de status mostra X/Y também fora da moldura, inclusive negativos.
- **A folha segue a moldura** (operação nova `FrameEdit::FitPageToFrame`, com delta `PageSize` reversível):
  - escolher a moldura de uma página no painel de propriedades aplica a moldura e o tamanho dela num só passo (um Ctrl+Z desfaz os dois);
  - trocar a moldura padrão do projeto ajusta, no mesmo passo, as páginas que usam a padrão;
  - páginas com tamanho diferente mostram o aviso e o botão **"Ajustar folha à moldura"**;
  - página sem moldura: o ajuste é recusado.
- **Grade como no EPLAN:** pontos no espaçamento real em toda a área visível; somem quando ficariam a menos de 6 px (`camera::MIN_GRID_PX`) e voltam ao aproximar; o programa não multiplica mais o espaçamento. Limite de 60.000 pontos por quadro. O encaixe (snap) continua no espaçamento lógico.
- **Linhas-guia do cursor:** linhas finas cinza (`theme::CURSOR_GUIDE`) que atravessam a área de trabalho pelo cursor, no ponto encaixado quando o snap está ligado; somem fora da área ou com diálogo aberto. Comando "Linhas-guia do cursor" na aba Visualização (grupo Grade) e botão na barra de status; preferência `cursor_guides`, ligada por padrão.
- `theme` passou a módulo público (os testes conferem as cores).

## Testes

- `crates/cadcraft-gui/tests/work_area_ui.rs` (8): fundo único sem retângulo cinza; linhas-guia de ponta a ponta, ausentes fora da área e com a opção desligada; guias no ponto encaixado; Y negativo abaixo da moldura; página sem moldura só com contorno; botão "Ajustar folha à moldura" e Ctrl+Z; escolha de moldura com tamanho num passo de Desfazer; ajuste recusado sem moldura.
- Atualizados para o comportamento pedido, sem perder verificação:
  - `grid_points.rs`: limite e unicidade dos pontos mantidos; o teste da densidade adaptativa virou "zoom distante oculta a grade sem mudar o snap" (abaixo de 6 px nenhum ponto; acima, espaçamento real);
  - `visual_regressions.rs`: o teste de DPI aproxima antes de conferir os pontos (em 2× DPI a página inteira deixa os pontos de 2,5 mm a ~3 px, e a grade some);
  - `frames_ui.rs`: escolher A4 numa página A3 agora adota 210 × 297 mm, sem o aviso de tamanho, e um Ctrl+Z restaura moldura e tamanho;
  - teste de unidade do viewport: página enorme sem retângulo de papel;
  - `frame_zones_ui.rs` (Etapa 30): rolagem antes de ler a soma das linhas, para o teste não depender do layout (5 de 5 execuções).

## Validação

`scripts/validate.ps1`: 776 testes Rust e 16 PowerShell aprovados, exit 0 (fmt, check, clippy -D warnings, diff --check). Release `cad-gui.exe` 23.927.808 bytes.

## Teste no app das Etapas 29–31 (Release, 2026-11-07, Claude Code com uso do computador autorizado)

- **Etapa 31:** fundo branco uniforme; página sem moldura só com contorno; linhas-guia cruzando a área; X/Y negativos abaixo da página (Y −57,64 mm); grade em toda a área a 67,7 % e oculta a 23,9 %; escolher "Moldura 1" (A4) numa página A3 adotou 210 × 297 e um Ctrl+Z voltou a A3 sem moldura; com tamanho diferente, aviso e botão "Ajustar folha à moldura" funcionaram, e o Ctrl+Z desfez; CPU parado 0 % (10 s).
- **Etapa 30:** moldura nova A4, área útil 200 × 287; 5 × 40 → remover uma coluna deu 4 × 50 (soma 200 de 200); 8 linhas de 35,875 (287 de 287), rótulos à esquerda e à direita visíveis; 4 × 60 deu a mensagem "As 4 colunas somam 240,0 mm, mas a área útil tem 200,0 mm…"; "Distribuir na área útil" corrigiu e aplicou (v2).
- **Etapa 29:** o app abriu já conectado ("sessão mantida neste computador", login anterior do autor); o assistente respondeu (6 páginas); fechar e reabrir manteve a conexão; item `chatgpt.CAD-CRAFT` no Gerenciador de Credenciais; "Sair" apagou o item e, ao reabrir, pede "Entrar com ChatGPT". O autor precisa entrar de novo para manter a conexão.
