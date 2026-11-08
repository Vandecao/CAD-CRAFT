# Relatório — Etapa 35 (bornes de régua, coordenadas digitadas e conexão unifilar)

Data: 2026-11-08. Agente: Claude Code. Prompt: `docs/internal/spec/ETAPA_35_BORNES_COORDENADAS_UNIFILAR.md` (aprovado; plano `PLANO_ETAPAS_32_35.md`, decisão 3). ADR-0034.

## Entregue

1. **Borne de régua pelo símbolo.** Ao selecionar um símbolo da categoria "Bornes", o centro de inserção muda para a associação "Borne de régua", com o nome da régua (X1 por padrão) e botões com as réguas existentes. Cada clique chama `Project::place_strip_terminal`, numa só operação desfazível:
   - usa ou cria a régua e o dispositivo dela (-X1);
   - acrescenta o próximo borne (X1:1, X1:2…);
   - cria um ponto de conexão ligado ao borne e vinculado às duas portas, então os fios dos dois lados formam a mesma rede.

   O símbolo mostra "-X1:n". A lista de bornes, as pontes e o editor de réguas passam a ver os bornes.
2. **Inserção de coordenadas** (Editar › Opções, comando "Inserção de coordenadas"). Com a opção ligada, uma barra com X, Y e "Relativo" aparece no canto da área de trabalho durante a inserção de símbolo, conexão ou linha gráfica.
   - Enter ou "Aplicar" age como um clique exatamente no ponto digitado: posiciona o símbolo, faz o canto do fio ou cai na porta que estiver ali, e marca o início ou o fim da linha.
   - "Relativo" soma X/Y ao último ponto (último canto do fio, início da linha ou último ponto digitado).
   - Mouse e teclado se misturam, e cada resultado é um passo de Ctrl+Z.
   - Os dois itens planejados ("Inserção relativa de coordenadas" e "Inserção de coordenadas") viraram esse comando.
3. **Conexão unifilar** (aba Conexões, "Conexão unifilar"): clique no símbolo de origem, nos cantos e no símbolo de destino.
   - As teclas 1–9 escolhem as fases (padrão 3); Delete sobre a linha a remove.
   - Desenho: polilinha com traços oblíquos de fase, igual na tela, no PDF e no DXF.
   - Não cria conexões nem redes e não entra na lista de conexões nem na numeração de fios.
   - Nova entidade `SingleLineLink`, **formato `.cadcraft` 11** com migração v10 → v11 (sem conversão) e fixture `etapa35-v11.cadcraft`.

## Saídas de referência

As 11 saídas `.cadcraft` de `baseline_outputs.rs` foram regravadas (`CAD_CRAFT_WRITE_BASELINE=1`) porque o manifesto agora diz formato 11. PDF, DXF e listas ficaram idênticos (em `hashes.txt` só mudaram as linhas `cadcraft`, e entraram as linhas da fixture nova `etapa35-v11`). O teste da fixture v10 passa a esperar `migrated()`, porque ela agora atravessa a migração v10 → v11.

## Limites conhecidos

- Uma representação ligada por conexão unifilar não pode ser excluída antes da ligação; o núcleo recusa com a mensagem de integridade. A ligação se remove com Delete sobre ela na ferramenta unifilar.
- A ferramenta unifilar funciona em qualquer página, não só nas unifilares.

## Testes

- `crates/cad-craft/tests/strips_and_single_lines.rs` (4):
  - três bornes formam X1:1–X1:3 na lista de bornes, com um só dispositivo -X1, texto "-X1:3" e as duas portas no mesmo ponto; desfazer e refazer exatos;
  - a conexão unifilar está na lista de exibição e no DXF, sem mudar redes nem conexões, e bloqueia a remoção do símbolo ligado;
  - ida e volta no formato 11;
  - fixture v11.
- `crates/cadcraft-gui/tests/coordinates_and_single_line_ui.rs` (3):
  - coordenadas absolutas e relativas posicionam símbolo e cantos de rota exatamente, e Ctrl+Z desfaz;
  - o símbolo de borne na régua X1 numera 1–3, e desfazer remove o último;
  - a ferramenta unifilar liga Q1 e K1 com três fases, sem conexões, e desfazer remove a ligação.

## Validação

`scripts/validate.ps1`: 807 testes Rust e 16 PowerShell aprovados, exit 0 (fmt, check, test, clippy `-D warnings`, `git diff --check`), depois das correções do teste no app. Release: 24.234.496 bytes.

## Teste no app das etapas 32–35 (2026-11-08)

A partida estrela-triângulo já estava concluída (o autor salvou o projeto e exportou o PDF antes de fechar a janela). Por isso foi montado um projeto novo e simples no Release: "Partida direta - teste etapas 32-35", autor "Vértice automação", páginas 1 Unifilar e 2 Comando. Arquivo e PDF ficaram em `Documentos\CAD-CRAFT\Projetos` e não são versionados.

Funcionou como esperado:
- **Símbolos de fábrica v2 (33):** desenho IEC nas miniaturas. A bobina entrou como -K1 com A1-A2, e "cria -K1" já vem do prefixo gravado no símbolo.
- **Pontos sugeridos (33):** o contato NA em -K1 recebeu 13-14 e a sugestão seguinte passou a 23-24.
- **Editor de símbolos (33):**
  - o "Botão de comando NF" da biblioteca (antes "-D1") ganhou Tipo de função "Contato auxiliar", Prefixo S e pontos 21/22;
  - salvo na biblioteca, entrou como -S2 com 21/22;
  - o editor mostra a leitura X/Y do cursor.
- **Coordenadas digitadas (35):**
  - absolutas e relativas posicionaram todos os símbolos (K1 em 240/100, S1 e o contato de K1 em 160, S2 em 200, bornes em 240 e 60);
  - todos os fios saíram de porta a porta, inclusive os cantos relativos (+0/+10, -30/0).
- **Junções em T (32)** nos dois pontos de selo (240/180 e 240/140).
- **Bornes de régua (35):**
  - o símbolo "Borne" mudou sozinho para "Borne de régua" e inseriu -X1:1 e -X1:2 (um ponto nas duas portas);
  - o navegador de bornes mostra "Régua X1 · 2 bornes".
- **Unifilar (35):** Q1 → K1 → M1 com duas conexões unifilares de 3 fases (traços oblíquos), "sem efeito nas redes".
- **Peças (34):** a prévia do contator GEN-CT3P-12 em -K1 mostrou contatos principais 1…6 → usa, bobina → usa, 13-14 → usa e 21-22 → nova. Atribuir criou só o 21-22, e -K1 ficou com 4 funções sem duplicatas.
- **Arquivo:** salvo no formato 11 e reaberto sem perda.
- **Verificação:** 0 erros, 0 avisos e 6 informações esperadas (VER-006 do 21-22 de K1 e VER-018 dos dispositivos sem peça).
- **PDF:** 2 páginas e 9 links de referência.

Defeitos encontrados e corrigidos no mesmo ciclo:
1. **Prefixo digitado mantido ao trocar de símbolo:** o motor entrou como -Q2. O prefixo digitado agora é limpo quando outro símbolo é escolhido.
2. **VER-004 em -X1:** as funções de borne eram auxiliares sem função principal. Bornes novos usam papel não especificado, e a verificação ignora funções cujos pontos são todos de bornes (vale também para arquivos gravados antes da correção).
3. **VER-007 em todas as portas dos símbolos ligados por conexão unifilar:** essas representações deixam de ser cobradas.
4. **Editor de símbolos mostrava o ID interno da porta** (1/2) no texto do ponto: agora mostra a identificação (13/14).
5. **Leitura do cursor "X -0,0"** passou a "X 0,0".

Testes novos para 2 e 3 em `strips_and_single_lines.rs`. Fora do escopo e só anotados: a ferramenta de junção termina após cada junção, e o editor de peças mostra o UUID nos botões "Localizar função".
