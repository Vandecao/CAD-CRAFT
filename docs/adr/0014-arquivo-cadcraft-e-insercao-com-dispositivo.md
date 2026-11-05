# ADR-0014 — arquivo `.cadcraft` e inserção com criação de dispositivo

Estado: implementado e testado automaticamente na Etapa 12 (Marco M0.1).
Homologação manual abrangente permanece após a Etapa 15.

## Contexto

A auditoria CAD-CRAFT × QElectroTech (2026-11-03) mostrou que não era possível
criar, salvar e reabrir um projeto real: só existia o snapshot técnico JSON
(8 MiB, caminho digitado) e a interface não criava dispositivos, funções nem
pontos de conexão. O usuário aprovou a Etapa 12 com as dependências `zip` e
`rfd`. O QElectroTech (GPL) serviu apenas como referência de conceitos do
formato `.qetz` (manifesto com `format`/`min-reader`, limites de ZIP); nenhum
código foi copiado.

## Decisão

- **Formato no Core, sistema de arquivos na GUI.** `cad_craft::persistence`
  lê/grava bytes (`Read + Seek` / `Write + Seek`) e nunca extrai para o disco.
  Gravação atômica, trava, recuperação, recentes e diálogos ficam em
  `cad-gui/src/project_file.rs`.
- **`.cadcraft` v1 = ZIP**: `mimetype` (primeira entrada, sem compressão,
  `application/x-cad-craft-project+zip`), `manifest.json`
  (`format`, `min_reader`, `writer`; campos extras tolerados) e `project.json`
  (o mesmo DTO estrito do snapshot). `assets/` reservado.
- **Versões**: `min_reader > suportado` → recusa; `format > suportado` →
  somente leitura. Como `project.json` rejeita campos desconhecidos, um formato
  futuro que altere esse conteúdo **deve elevar `min_reader`**; entradas novas
  no ZIP podem ser acrescentadas sem elevá-lo.
- **v0** = snapshot técnico JSON (com ou sem BOM), migrado por cadeia
  `migrate(vN→vN+1)`; abre sem arquivo associado e exige "Salvar" para gravar
  `.cadcraft`. Limite de leitura v0: 256 MiB.
- **Leitor defensivo**: recusa nomes absolutos, `..`, `.`, `\`, `:`, NUL,
  duplicados (exatos ou por maiúsculas), entradas criptografadas, compressão
  diferente de Stored/Deflated, mais de 4.096 entradas, mais de 1 GiB
  declarado e bytes reais acima do limite; valida consistência de cabeçalhos
  antes de ler o projeto. O agregado é validado na gravação e na leitura.
- **Gravação atômica**: serialização completa em memória, temporário na
  mesma pasta, `sync_all`, substituição (`snapshots::write_from`).
- **Trava** `<arquivo>.lock` (JSON: usuário, máquina, processo, instante,
  token) criada com `create_new`; removida ao soltar somente se ainda for da
  sessão. Conflito → diálogo: somente leitura (padrão) ou assumir a trava por
  decisão explícita. Somente leitura transforma "Salvar" em "Salvar como".
- **Recuperação**: com alterações não salvas, cópia `.cadcraft` + descrição
  JSON em `%LOCALAPPDATA%\CAD-CRAFT\recovery` (primeira alteração e depois a
  cada 120 s). Salvar ou descartar confirmado remove a cópia; encerramento
  abrupto a mantém e ela é oferecida na abertura seguinte. `CadApp::default`
  (testes) não ativa recuperação.
- **`Project::place_symbol` / `restore_placement`**: uma transação cria
  dispositivo novo (ou usa existente), função (principal se o dispositivo não
  tiver principal, senão auxiliar), um ponto de conexão por porta, a ocorrência
  ligada à função e os vínculos porta→ponto. Falha parcial é desfeita. O delta
  guarda entidades completas (mesmos IDs no Refazer); Desfazer confere o
  estado e revalida o agregado, restaurando tudo se algo passou a referenciar
  as entidades.
- **`DocumentEdit`/`DocumentDelta`**: inserir documento, inserir/excluir página
  e reordenar entram no histórico comum; `Project::remove_document` sem
  cascata. Abas de páginas removidas por Desfazer são fechadas.
- **GUI**: associação padrão "Novo dispositivo" com prefixo sugerido por
  símbolo (K, M, X, S, B, PE, G) e designação `{prefixo}{n}` livre no clique;
  "Função em dispositivo existente"; "Entidade existente ou sem associação"
  preserva o comportamento anterior. Comandos Abrir (Ctrl+O), Salvar (Ctrl+S),
  Salvar como (Ctrl+Shift+S), Backstage Abrir com recentes e recuperação.
  `FilePicker` injetável (`rfd` em produção, roteiro em testes).

## Consequências

- Projetos reais podem ser criados, salvos, fechados, reabertos e editados.
- O snapshot técnico JSON continua disponível (Importar/Exportar) e legível
  como v0.
- Histórico de Undo continua por sessão e não é persistido.
- Desfazer uma inserção com dispositivo revalida o projeto inteiro (custo
  proporcional ao projeto, como as edições físicas — PHY-012).
- Trava é cooperativa: após falha, a trava antiga exige "Ignorar a trava".
- Renomear/excluir documento continuam inexistentes (não eram funcionalidade).
