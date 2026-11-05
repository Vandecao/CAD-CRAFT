# CAD-CRAFT — Etapa 18 (Fase B): dispositivos, peças, funções e pontos

**Estado: aprovado pelo usuário em 2026-11-03 para execução pelo Codex.**
Decisões aprovadas: um `.cadpart` por peça; reutilizar `Component` para a
peça instalada; CSV nesta etapa; inserir função não representada nesta etapa.
O usuário autorizou o commit e push da Etapa 17, concluídos em `0886379`.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase B, Etapa 18,
depende da 17). Base de código: Etapa 17 implementada, validada (526 Rust +
15 PowerShell) e versionada em `0886379` no `origin/main`.

## Objetivo

Cadastrar peças reais (fabricante, código, dados técnicos) numa base de
peças reutilizável, atribuí-las a dispositivos e editar, pela interface, os
dispositivos, suas funções e seus pontos de conexão — o equivalente ao
"Gerenciamento de peças" e à "Navegação de dispositivos" do EPLAN, sem
copiar código ou dados de terceiros.

## O que já existe (não refazer)

- Core: `Device` (designação, classificação, `TechnicalProperties`),
  `Function` (dispositivo, designação, `FunctionKind`, `FunctionRole`,
  propriedades), `ConnectionPoint` (função, rótulo, borne/nível,
  propriedades), `Component` (designação, classificação, `manufacturer`,
  `catalog_code`, vínculo opcional a dispositivo/função, propriedades),
  `TechnicalProperties` (conhecidas: tensão, corrente, potência,
  frequência, resistência, seção; e personalizadas).
- Inserção com dispositivo novo/existente (`place_symbol`, Etapa 12),
  identificação e referências (Etapa 9), navegador de dispositivos,
  propriedades do dispositivo (`cad-gui/src/device_properties.rs`),
  engenharia física (Etapas 10–11), textos dinâmicos de propriedade
  (Etapa 17, `SymbolTextField::DeviceProperty`).

## Escopo

1. **Base de peças** (catálogo, independente do projeto): `Part` com ID
   textual estável (ex.: `SIEMENS.3RT2015-1BB41`), fabricante, código,
   descrição (pt-BR + traduções), classificação, dados técnicos
   (`TechnicalProperties`), **modelo de funções** (lista de funções com
   `FunctionKind`, papel principal/auxiliar e rótulos de pontos, ex.: bobina
   A1/A2; contato principal 1/2, 3/4, 5/6; contato auxiliar 13/14) e símbolo
   sugerido por função (ID de `SymbolDefinition`, opcional). Validação
   completa (rótulos únicos por função, campos não vazios).
2. **Biblioteca de peças**: pasta configurável (como símbolos/molduras) com
   arquivos de peça (formato na decisão 1), leitor defensivo no padrão
   `.cadsym`; importação/exportação **CSV** para cadastro em lote (decisão
   3) com relatório de linhas recusadas.
3. **Peças no projeto**: o projeto guarda as peças usadas (autossuficiente,
   como os símbolos). Atribuir uma ou mais peças a um dispositivo com
   quantidade (decisão 2 define se reaproveita `Component`). Remover/trocar
   peça. "Atualizar peças do projeto" pela biblioteca, como na Etapa 17.
4. **Aplicar modelo de funções**: ao atribuir a peça, criar as funções e
   pontos que faltam no dispositivo (sem duplicar os existentes) — funções
   ainda **não representadas** em página.
5. **Editor de dispositivo** (janela/aba): designação, classificação,
   propriedades conhecidas e personalizadas, peças atribuídas, lista de
   funções com estado "representada em /página.coluna" ou "não
   representada"; navegar até a ocorrência.
6. **Editor de funções e pontos**: criar, renomear, mudar tipo e papel,
   excluir função (só sem representação e sem ligações); criar, renomear,
   excluir pontos (só sem vínculo de porta, conexão, condutor ou borne);
   associar ponto a borne existente (reaproveitar `SetPointTerminal`).
   Tudo por deltas reversíveis no histórico comum.
7. **Inserir função não representada**: no Centro de inserção, para
   "Função em dispositivo existente", escolher uma função ainda não
   representada e ligar as portas do símbolo aos pontos existentes por
   rótulo (ou por ordem, com aviso), em vez de criar função nova (decisão 4).
8. **Lista de peças** (BOM) derivada: dispositivo, peça, fabricante, código,
   quantidade, descrição; exportar CSV. Relatórios formatados ficam para a
   Etapa 23.
9. **Persistência**: `.cadcraft` formato 6 / `min_reader` 6 (peças e
   atribuições), migração 5→6, fixture `etapa18-v6.cadcraft`; v0–v5
   continuam abrindo.

## Critérios de aceitação

1. Com eventos egui reais: cadastrar uma peça (contator com bobina A1/A2,
   3 contatos principais e 1 auxiliar 13/14), salvar na biblioteca,
   atribuí-la a `-K1`, ver as funções criadas como "não representadas",
   inserir a bobina como função existente (portas ligadas a A1/A2) e ver a
   referência atualizada.
2. Editor de funções/pontos: criar, renomear e excluir com recusas corretas
   (ponto ligado, função representada); Desfazer/Refazer exatos.
3. Importar CSV de 100 peças (com linhas inválidas recusadas e relatadas) e
   exportar de volta com igualdade; arquivo de peça inválido recusado sem
   alterar nada.
4. "Atualizar peças do projeto" troca dados técnicos e mantém atribuições;
   modelo de funções incompatível (função representada removida) é recusado.
5. Lista de peças correta (quantidades somadas) e exportada em CSV.
6. Salvar/reabrir conserva peças, atribuições, funções e pontos; fixtures
   v0–v5 abrem; v6 versionada.
7. Testes anteriores preservados; `scripts/validate.ps1`; Release (sem
   executar); relatório `docs/internal/reports/ETAPA_18_...`, ADR-0020, memória;
   commit + push somente após OK expresso. **Sem teste no executável nesta
   etapa** (o próximo é ao fim da Etapa 19, com autorização do usuário para
   usar o computador).

## Decisões pedidas ao usuário

1. **Arquivo de peça**: um `.cadpart` por peça (ZIP com manifesto, como
   `.cadsym`) — recomendado — ou um único arquivo de base de peças.
2. **Instância da peça**: reaproveitar `Component` (já tem fabricante,
   código e vínculo ao dispositivo) acrescentando o ID da peça e a
   quantidade — recomendado — ou criar entidade nova.
3. **CSV** para cadastro em lote nesta etapa — recomendado — ou adiar.
4. **Inserir função não representada** (item 7) nesta etapa — recomendado —
   ou adiar para as macros (Etapa 19).

## Fora do escopo

Macros (Etapa 19); relatórios formatados e formulários (Etapa 23);
importação de bases de peças do EPLAN (`.edz`/XML, Etapa 26); preços,
estoque e fornecedores; imagens de peças.

## Instruções para o agente (Codex ou Claude)

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md`,
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory do projeto
   CAD-CRAFT (página `notes/etapa-17-biblioteca-editor-simbolos.md` e, se
   existir, `notes/etapa-18-*.md`).
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). Se a Etapa 17
   ainda estiver fora do HEAD, **perguntar ao usuário** se pode versioná-la
   antes de começar; com o OK: `git add` só dos arquivos da Etapa 17 (nunca
   `docs.rar`), um commit e `git push origin main` sem força.
3. Só iniciar após a aprovação expressa deste prompt (com as quatro
   decisões). Registrar a aprovação no topo deste arquivo.
4. Criar `docs/internal/plans/<data>-etapa-18.md` com desenho técnico, seção
   "Retomada por outro agente" e lista "Andamento" numerada; marcar cada
   item ao concluí-lo com teste passando. Gravar no ai-memory uma página
   fixada `notes/etapa-18-dispositivos-pecas-funcoes.md` com prompt,
   decisões, arquivos e próximo passo; atualizá-la ao final.
5. Seguir os padrões existentes: deltas reversíveis no Core com reversão em
   erro (ver `project/clipboard.rs`, `project/symbols.rs`), histórico
   `Transaction::*` na GUI, pacotes ZIP defensivos em `persistence.rs`,
   biblioteca em pasta (ver `cad-gui/src/symbol_files.rs`), testes egui com
   accesskit (ver `cad-gui/tests/symbol_editor_ui.rs`). No Windows, Ctrl+C/X/V
   chegam como eventos `Copy/Cut/Paste` (ver `app.rs`).
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; build Release
   sem executar o app; não executar o app sem autorização expressa.
7. Atualizar relatório, ADR-0020, `DECISIONS.md`, `ROADMAP.md` (linha 18),
   `KNOWN_ISSUES.md`, `CURRENT_STATE.md`, `HANDOFF.md`; informar que está
   pronto e aguardar o OK para versionar. Não iniciar a Etapa 19.
