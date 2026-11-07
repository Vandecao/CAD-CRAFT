# CAD-CRAFT — Etapa 31: área de trabalho, grade e cursor (como no EPLAN)

**Estado: APROVADO pelo autor em 2026-11-07** (plano `docs/internal/plans/PLANO_ETAPAS_29_31.md`, decisões 4, 5 e 6: grade em toda a área, como no EPLAN).

- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE; código em `cadcraft-gui/src/viewport.rs`, `camera.rs`, `frame_view.rs`, `theme.rs`, `panels/properties/`.
- **Referência:** vídeos do autor de 2026-11-07 (EPLAN e QElectroTech), descritos na tabela do plano. O EPLAN é o modelo.

## Problemas (imagens 2 e 3 do autor e vídeo do CAD-CRAFT)

- A tela pinta a folha de branco (`theme::PAPER`) sobre fundo cinza (`theme::CANVAS`); a parte da moldura fora da folha fica no cinza.
- A folha tem tamanho próprio, independente da moldura (página "Especial" 500 × 250 com moldura A3 420 × 297).
- `camera::grid_points` multiplica o espaçamento ao afastar o zoom e nunca oculta a grade: a 2,3% de zoom a tela fica coberta de pontos.
- Não há linhas-guia do cursor.

## Requisitos

1. **Fundo uniforme branco** em toda a área de trabalho (como no EPLAN), sem retângulo de folha mais claro. Temas e cores continuam centralizados em `theme.rs`. A moldura é o limite visual; a área continua além dela.
2. **Origem:** (0,0) no canto inferior esquerdo da moldura de cada página (como hoje, a moldura ancorada na origem da página). A barra de status mostra X/Y nesse sistema também fora da moldura, inclusive negativos (EPLAN: "Y: -60,05 mm").
3. **A folha segue a moldura:**
   - ao escolher uma moldura para a página (propriedades, nova página, troca da moldura padrão do projeto), a página adota o tamanho da moldura, numa operação reversível (um Ctrl+Z);
   - páginas existentes com tamanho diferente da moldura mantêm o aviso atual e ganham o botão **"Ajustar folha à moldura"**;
   - página **sem moldura**: contorno fino da folha (decisão 4), sem preenchimento.
   - PDF e DXF continuam usando o tamanho da página.
4. **Grade como no EPLAN:**
   - pontos finos no espaçamento real escolhido (4 mm etc.), desenhados em toda a área visível;
   - quando o espaçamento na tela fica abaixo de ~6 px, a grade **some** (sem multiplicar o espaçamento) e volta ao aproximar;
   - limite de pontos por quadro mantido, por desempenho;
   - o encaixe na grade continua funcionando mesmo com a grade oculta.
5. **Linhas-guia do cursor como no EPLAN:**
   - uma linha horizontal e uma vertical, finas e cinza, atravessando toda a área de trabalho e passando pelo cursor, em qualquer zoom;
   - seguem o ponto encaixado na grade quando o encaixe está ligado;
   - somem quando o mouse sai da área de trabalho e não aparecem por cima de diálogos e painéis;
   - opção "Linhas-guia do cursor" em Visualização, **ligada por padrão** (decisão 5), salva nas preferências.
6. Desempenho: a CPU parada continua abaixo de 5% de um núcleo (GUI-L01); as linhas-guia só pedem redesenho quando o mouse se move.

## Testes

- Render da tela: nenhum retângulo `PAPER` sobre `CANVAS`; fundo único; página sem moldura com contorno.
- Grade: com espaçamento na tela < 6 px nenhum ponto; acima, pontos no espaçamento real (não múltiplo); limite de pontos respeitado; testes atuais de `grid_points` adaptados sem perder cobertura.
- Linhas-guia: com o ponteiro dentro da área, duas linhas que passam pelo ponto (encaixado quando o encaixe está ligado); fora, nenhuma; opção desligada, nenhuma.
- Coordenadas: ponto abaixo da moldura mostra Y negativo na barra de status.
- "Ajustar folha à moldura" e a troca de moldura: tamanho igual ao da moldura, um Ctrl+Z desfaz.
- `idle_frames.rs` e `screen_scales.rs` continuam passando.
- PDF/DXF e saídas de referência inalterados.

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; saídas de referência idênticas; Release compilado.
- **Teste no app** pelo Claude Code com autorização do autor, cobrindo as Etapas 29, 30 e 31 (o "manter conectado" com o login do autor no navegador).
- Relatório `docs/internal/reports/ETAPA_31_AREA_TRABALHO.md`; HANDOFF, CURRENT_STATE e ROADMAP. Commit + push só com o OK do autor.
