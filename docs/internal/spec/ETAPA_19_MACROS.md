# CAD-CRAFT — Etapa 19 (Fase B): macros

**Estado: aprovado pelo usuário em 2026-11-04, execução pelo Claude Code.**
Decisões aprovadas (todas as recomendadas): um `.cadmacro` por macro;
versão diferente de símbolo/peça mantém a do projeto e avisa; próxima
designação livre por prefixo; macro de página nesta etapa; copiar/colar com
fios nesta etapa. O usuário pediu, ao fim da etapa, teste no executável das
Etapas 18 e 19 com uso do computador e, se tudo passar, commit + push das
Etapas 18 e 19.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase B, Etapa 19,
depende de 16, 17 e 18). Base de código: Etapa 18 implementada e validada
(539 Rust + 15 PowerShell), **ainda fora do HEAD** (`main` em `0886379`,
Etapa 17).

## Objetivo

Guardar um trecho de esquema pronto (símbolos, dispositivos, funções, fios,
junções, textos e gráficos) como **macro** reutilizável e inseri-lo em
qualquer página ou projeto com IDs e designações novos — o equivalente às
macros de janela e de página do EPLAN, sem copiar código ou dados de
terceiros. Fecha o achado CLP-001 (copiar circuito com os fios).

## O que já existe (não refazer)

- Área de transferência da sessão (Etapa 16, `project/clipboard.rs`):
  instantâneos `ClipItem`, colagem que cria dispositivo novo com a próxima
  designação pelo prefixo, `SheetEditDelta` reversível. Hoje recusa rotas,
  junções e marcadores.
- Fios e junções (`domain/connection_drawing.rs`,
  `project/connection_drawing.rs`): `ConnectionRoute` entre `PortRef`,
  `ConnectionJunction`, vínculos porta→ponto, `ConnectionDrawingEdit`
  (`CreateRoute`, `CreateJunction`, `BindPort`…), conexões lógicas derivadas.
- Marcadores de potencial (`project/markers.rs`), conteúdo livre
  (`page_graphic.rs`), símbolos autossuficientes no projeto (Etapa 17),
  peças e modelo de funções (Etapa 18), inserção com dispositivo
  (`project/placement.rs`).
- Padrões: pacote ZIP defensivo (`.cadsym`, `.cadframe`, `.cadpart` em
  `persistence.rs`), biblioteca em pasta configurável
  (`cad-gui/src/symbol_files.rs`), histórico comum `Transaction::*`
  (`cad-gui/src/history.rs`), Centro de inserção
  (`cad-gui/src/panels/insertion.rs`).

## Escopo

1. **Modelo de macro (Core, sem GUI)**: `Macro` com ID textual estável,
   nome, descrição (pt-BR + traduções), categoria, versão, ponto base e
   conteúdo autossuficiente: representações (símbolos e conteúdo livre),
   dispositivos/funções/pontos usados, vínculos porta→ponto, rotas e
   junções **internas** à seleção, marcadores, definições de símbolo e
   peças referenciadas. Validação completa (referências internas fechadas,
   limites como `MAX_CLIPBOARD_ITEMS`).
2. **Criar macro da seleção**: na página ativa, com ponto base escolhido
   pelo usuário (padrão: canto inferior esquerdo da seleção na grade). Fios
   com uma ponta fora da seleção são descartados e relatados; junções só
   entram se todas as suas rotas entram.
3. **Inserir macro**: prévia acompanhando o cursor com grade, rotação de
   90° e espelhamento (reaproveitar `Transform2D`); um clique insere em
   **uma única transação** reversível. Todos os IDs são novos; definições
   de símbolo e peças ausentes no projeto são acrescentadas (iguais são
   reaproveitadas; versão diferente segue a decisão 2). Designações seguem
   a decisão 3. Inserção repetida (continua no modo até Esc).
4. **Macro de página** (decisão 4): criar macro a partir da página inteira
   (sem a moldura) e inserir como página nova no fim do projeto ou após a
   página ativa.
5. **Biblioteca de macros**: pasta configurável nas Configurações, um
   arquivo `.cadmacro` por macro (decisão 1), leitor defensivo no padrão
   `.cadsym`; importar/exportar; recusa de arquivo inválido sem alterar
   nada. Navegador de macros (Inserir > Macros > Navegador): árvore por
   categoria, busca, miniatura vetorial desenhada a partir do conteúdo,
   inserir, renomear, excluir da biblioteca (com confirmação).
6. **Copiar/colar com fios** (decisão 5): Ctrl+C/X/V passam a levar rotas e
   junções internas à seleção, usando o mesmo motor da macro; regras de
   designação da Etapa 16 preservadas.
7. **Comandos**: Inserir > Macros (Navegador, Inserir macro); Dados-mestre >
   Macros (Nova da seleção, Macro de página, Importar, Exportar); menu de
   contexto da folha "Criar macro da seleção".
8. **Persistência do projeto**: o projeto **não** guarda as macros (elas
   ficam na biblioteca); o conteúdo inserido usa as estruturas já
   persistidas. Portanto `.cadcraft` continua no formato 6, sem migração.

## Critérios de aceitação

1. Com eventos egui reais: desenhar um circuito de partida direta (`-Q1`,
   `-K1` bobina + contatos, `-F1`, fios e uma junção), criar a macro, salvar
   na biblioteca, inseri-la duas vezes em outra página e obter `-Q2`/`-K2`/
   `-F2` e `-Q3`/`-K3`/`-F3` com fios ligados aos pontos certos e a rede
   (conexões lógicas) igual à do original.
2. Desfazer remove a inserção inteira de uma vez; Refazer restaura IDs e
   ligações idênticos ao estado após a inserção.
3. Fio com ponta fora da seleção é descartado e aparece no relatório;
   junção parcial idem.
4. Inserir em outro projeto (sem os símbolos e peças) acrescenta as
   definições e peças; reabrir o projeto salvo conserva tudo.
5. Rotação e espelhamento na inserção mantêm os fios presos às portas.
6. Macro de página cria a página nova com o conteúdo e sem moldura
   duplicada.
7. `.cadmacro` inválido, truncado, com referência quebrada ou grande demais
   é recusado sem alterar a biblioteca nem o projeto.
8. Ctrl+C/V de símbolos com fio entre eles cola os fios (CLP-001 fechado).
9. Testes anteriores preservados; fixtures v0–v6 abrem;
   `scripts/validate.ps1`; build Release; relatório
   `docs/internal/reports/ETAPA_19_MACROS.md`, ADR-0021, memória.
10. **Teste funcional no executável ao fim desta etapa** (Etapas 17–19),
    somente com autorização expressa do usuário para usar o computador.

## Decisões pedidas ao usuário

1. **Arquivo de macro**: um `.cadmacro` por macro (ZIP com manifesto,
   símbolos e peças embutidos, como `.cadsym`) — recomendado — ou um único
   arquivo de biblioteca.
2. **Símbolo ou peça com mesmo ID e versão diferente** ao inserir: manter a
   versão do projeto e avisar — recomendado — ou substituir pela da macro.
3. **Designações ao inserir**: próxima designação livre por prefixo
   (como a colagem; `-K1` vira `-K2`) — recomendado — ou manter as da macro
   e pedir confirmação quando houver conflito.
4. **Macro de página** nesta etapa — recomendado — ou adiar.
5. **Copiar/colar com fios** (CLP-001) nesta etapa, pelo mesmo motor —
   recomendado — ou deixar a área de transferência como está.

## Fora do escopo

Variáveis/espaços reservados e conjuntos de valores (placeholders do EPLAN);
variantes de representação (multifilar/unifilar) por macro; atualizar
ocorrências já inseridas quando a macro muda; macros com engenharia física
(cabos, condutores físicos, pontes — relatados e descartados); importação de
macros EPLAN `.ema` (Etapa 26); numeração por regras com caminho `=`/`+`
(Etapa 20).

## Instruções para o agente (Codex ou Claude)

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md`,
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory do projeto
   CAD-CRAFT (`notes/etapa-18-dispositivos-pecas-funcoes.md` e, se existir,
   `notes/etapa-19-*.md`).
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). Se a Etapa 18
   ainda estiver fora do HEAD, **perguntar ao usuário** se pode versioná-la
   antes de começar; com o OK: `git add` só dos arquivos da Etapa 18 (nunca
   `docs.rar`), um commit e `git push origin main` sem força.
3. Só iniciar após a aprovação expressa deste prompt (com as cinco
   decisões). Registrar a aprovação no topo deste arquivo.
4. Criar `docs/internal/plans/<data>-etapa-19.md` com desenho técnico, seção
   "Retomada por outro agente" e lista "Andamento" numerada; marcar cada
   item ao concluí-lo com teste passando. Gravar no ai-memory a página
   fixada `notes/etapa-19-macros.md` com prompt, decisões, arquivos e
   próximo passo; atualizá-la ao final.
5. Seguir os padrões existentes: deltas reversíveis no Core com reversão em
   erro (`project/clipboard.rs`, `project/connection_drawing.rs`), um
   `Transaction::*` por inserção na GUI, pacotes ZIP defensivos em
   `persistence.rs`, biblioteca em pasta (`symbol_files.rs`), testes egui
   com accesskit (`cad-gui/tests/symbol_editor_ui.rs`,
   `free_clipboard.rs`). No Windows, Ctrl+C/X/V chegam como eventos
   `Copy/Cut/Paste` (`app.rs`).
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; build Release.
   Rodar `cargo clean` só se o disco ficar sem espaço (a pasta `target`
   chegou a 100 GB).
7. Atualizar relatório, ADR-0021, `DECISIONS.md`, `ROADMAP.md` (linha 19),
   `KNOWN_ISSUES.md` (CLP-001, CLP-005), `CURRENT_STATE.md`, `HANDOFF.md`;
   informar que está pronto e aguardar o OK para versionar. Não iniciar a
   Etapa 20.
