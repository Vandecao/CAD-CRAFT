# CAD-CRAFT — Etapa 24 (Fase C): verificações de consistência; DEC-11-01/02 e CON-001

**Estado: aprovado pelo usuário em 2026-11-04 ("Aprovo todas").** Decisões
aprovadas (todas as recomendadas): (1) regras do item 2 do escopo; (2)
DEC-11-01 proposta A: acoplamento dá continuidade por conexões de posse; (3)
DEC-11-02: seccionável documental, verificação informa abertos; (4) esquema
no projeto, `.cadcraft` formato 9; (5) sob demanda com reavaliação
automática das mensagens listadas.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md` (Fase C, Etapa 24:
"Verificações de consistência; DEC-11-01/02, CON-001", depende da 20).
Base: Etapa 22 em `0b3b65c`; Etapa 23 implementada e validada (618 Rust + 15
PowerShell), ainda não versionada.

## Objetivo

Verificar o projeto com regras de consistência que gerem mensagens
navegáveis, e resolver as três pendências antigas do modelo:
- **DEC-11-01:** continuidade elétrica pelo acoplamento de conectores;
- **DEC-11-02:** efeito do estado do borne seccionável;
- **CON-001:** designações de conector repetidas.

É o equivalente às "Verificações" e à "Navegação de mensagens" do EPLAN.
Não é ERC normativo nem cálculo elétrico (corrente, queda de tensão,
seletividade).

## O que já existe (não refazer)

- `Project::validate()`: integridade **estrutural**, que recusa estados
  inválidos. O comando Ferramentas > Verificação > "Integridade do Core"
  (`CommandId::Validate`) o executa.
- `designation_conflicts()` (Etapa 12/20) e os conflitos de numeração de
  dispositivos e fios da Etapa 20 (potenciais e números travados).
- Redes derivadas (`networks`, `network_for_point`), marcadores de
  potencial e interrupção com grupos (`markers.rs`), listas físicas
  (`physical_lists`), peças e funções (Etapas 18–19), relatórios e o estado
  "desatualizado" (Etapa 23).
- Conectores e acoplamento documental (`ConnectorMating`, Etapa 11, ADR-0013);
  borne seccionável com estado `Closed/Open` apenas documental.
- Navegação até ocorrências (`navigate_reference`, Etapa 13/20), painel
  Mensagens da GUI, CSV/XLSX (`report_export.rs`, Etapa 23).

## Escopo

1. **Motor de verificação (Core, sem GUI)**:
   `Project::check(&CheckScheme) -> CheckReport`.
   - Cada mensagem tem:
     - **código estável** da regra (por exemplo `VER-001`);
     - gravidade: erro, aviso ou informação;
     - texto em português;
     - **alvos** (página, ocorrência, dispositivo, conexão, borne, cabo)
       para navegação.
   - As regras são funções puras do cadastro, em ordem determinística.
   - 10.000 ocorrências são verificadas em menos de 2 s no Release.
2. **Regras desta etapa** (decisão 1), cada uma com teste próprio:
   - **Designações:**
     - dispositivo com designação repetida no mesmo caminho `=`/`+`;
     - conector com designação repetida (**CON-001**);
     - número de página repetido no documento.
   - **Funções:**
     - dispositivo com auxiliares e sem função principal;
     - mais de uma função principal;
     - função da peça não representada (informação).
   - **Conexões:**
     - ponto de conexão sem nenhuma conexão (aviso, configurável);
     - fio sem número quando há regra de numeração aplicada;
     - rede com dois potenciais nomeados diferentes.
   - **Marcadores:** interrupção sem par no grupo; marcador de potencial sem
     ponto.
   - **Físico:**
     - veia de cabo usada em mais de uma conexão;
     - cabo com mais veias usadas do que tem;
     - borne sem destino em nenhum lado (informação);
     - ponte entre bornes de réguas diferentes (aviso);
     - borne seccionável **aberto** (informação, decisão 3).
   - **Conectores:** acoplamento cujo contato não tem ponto associado (aviso,
     decisão 2).
   - **Peças:** dispositivo sem peça (informação); peça atribuída sem código.
   - **Relatórios:** relatório desatualizado (Etapa 23).
3. **Esquema de verificação** (decisão 4): por regra, ativar ou desativar e
   escolher a gravidade, mais a opção de ignorar mensagens específicas, com
   justificativa. É configuração do projeto, editada em Configurações >
   Verificações, e as alterações entram no Desfazer.
4. **Execução**:
   - Ferramentas > Verificação > **Verificar projeto** (todas as regras) e
     **Verificar página** (página ativa);
   - a execução roda em segundo plano, com progresso e cancelamento
     (padrão de `pdf_export.rs`);
   - as mensagens existentes são recalculadas automaticamente depois de
     cada edição, só para os alvos tocados (decisão 5).
5. **Navegação de mensagens (GUI)**:
   - painel "Verificações" com filtros por gravidade, regra e página;
   - contadores, com duplo clique que abre a página, centraliza a
     ocorrência e a seleciona (`navigate_reference`);
   - "Próxima"/"Anterior" (F8/Shift+F8);
   - "Ignorar…" com justificativa;
   - exportar a lista de mensagens em CSV/XLSX com o `report_export`.
6. **DEC-11-01, acoplamento de conectores** (decisão 2): conforme a opção
   aprovada, o acoplamento passa a criar, numa só transação reversível,
   uma `Connection` lógica de posse do acoplamento por par de contatos com
   ponto associado, como as pontes de borne. As redes atravessam o conector
   acoplado. Desacoplar remove essas conexões; condutores físicos externos
   não são criados. A migração recalcula os acoplamentos existentes.
7. **DEC-11-02, borne seccionável** (decisão 3): conforme a opção aprovada,
   o estado continua documental, a rede segue contínua e a verificação
   informa os bornes abertos.
8. **Persistência**: o esquema de verificação, as mensagens ignoradas e as
   conexões de posse do acoplamento entram no `.cadcraft` formato 9
   (`min_reader` 9), com migração 8→9 e a fixture `etapa24-v9.cadcraft`; as
   fixtures v0–v8 continuam abrindo.

## Critérios de aceitação

1. Para cada regra, um projeto mínimo que a viola gera exatamente a
   mensagem esperada (código, gravidade e alvos), e o mesmo projeto
   corrigido não gera nenhuma.
2. CON-001: dois conectores `-X1` geram o aviso; nomes iguais continuam sem
   unir redes.
3. DEC-11-01: acoplar `-X1` (macho) a `-X2` (fêmea) com contatos associados
   faz as redes dos dois lados virarem uma só. Desacoplar separa de novo.
   Um Desfazer restaura exatamente as duas operações. Um projeto v8 com
   acoplamento existente, depois de migrado, tem as conexões de posse.
4. DEC-11-02: borne seccionável aberto aparece como informação, e a rede
   continua a mesma (sem mudança de semântica).
5. Esquema: desativar a regra tira as mensagens; mudar a gravidade muda o
   contador; "Ignorar" com justificativa esconde a mensagem e sobrevive a
   salvar e reabrir; Desfazer restaura.
6. Com eventos egui reais:
   - Verificar projeto mostra o painel com os contadores;
   - duplo clique numa mensagem abre a página certa e seleciona a
     ocorrência;
   - F8 vai para a próxima mensagem;
   - corrigir o problema (por exemplo, renumerar) remove a mensagem sem
     verificar de novo;
   - exportar CSV.
7. Desempenho do item 1 do escopo.
8. Salvar e reabrir conserva o esquema, as mensagens ignoradas e as
   conexões de acoplamento; v0–v8 abrem; v9 versionada.
9. Testes anteriores preservados (ajustes só se a decisão 2 mudar
   explicitamente uma expectativa antiga, registrados no relatório);
   `scripts/validate.ps1`; Release; relatório
   `docs/internal/reports/ETAPA_24_VERIFICACOES.md`, ADR-0026 e memória; commit +
   push somente após o OK expresso. **Sem teste no executável nesta etapa**
   (o próximo é ao fim da 25).

## Decisões pedidas ao usuário

1. **Regras desta etapa**:
   - **Recomendado:** as do item 2 do escopo (designações, funções,
     conexões, marcadores, físico, conectores, peças e relatórios).
   - Ficam para depois as regras normativas e elétricas (seção do condutor
     pela corrente, proteção, queda de tensão) e as do painel/3D.
2. **DEC-11-01, acoplamento de conectores**:
   - **Recomendado (proposta A da Etapa 11):** o acoplamento **dá
     continuidade**. Cria uma conexão lógica de posse por par de contatos,
     como as pontes, e as redes atravessam o conector. É o comportamento do
     EPLAN e o que o usuário espera ao acoplar.
   - Alternativa (proposta B): o acoplamento continua só documental; a
     continuidade exige fios desenhados, e a verificação avisa contatos
     acoplados sem ligação.
3. **DEC-11-02, borne seccionável**:
   - **Recomendado:** o estado continua **documental**. A rede fica
     contínua, como no esquema, que se desenha fechado, e a verificação
     informa os bornes abertos.
   - Alternativa: pontos separados por lado, com ligação interna
     controlada pelo estado. Aberto separaria as redes, o que muda o modelo
     de bornes e o grafo.
4. **Onde fica o esquema de verificação**:
   - **Recomendado:** no **projeto** (`.cadcraft` formato 9), que viaja com
     o arquivo e vale para quem abrir, como no EPLAN.
   - Alternativa: nas preferências do app, por usuário, sem mudar o
     formato. Cada máquina teria regras diferentes.
5. **Quando verificar**:
   - **Recomendado:** sob demanda (Verificar projeto ou página), com as
     mensagens já listadas **reavaliadas automaticamente** depois de cada
     edição nos alvos tocados, para sumirem quando corrigidas.
   - Alternativa: só sob demanda, sem reavaliação, ou verificação completa
     contínua a cada edição, mais cara em projetos grandes.

## Fora do escopo

ERC normativo e cálculos elétricos; verificações de painel/3D e de CLP;
correção automática ("corrigir tudo"); regras definidas pelo usuário em
script; importação de esquemas de verificação do EPLAN (Etapa 26).

## Instruções para o agente

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md` e
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory
   (`notes/etapa-23-relatorios.md` e, se existir, `notes/etapa-24-*`). ADRs
   úteis: 0013 (conectores e bornes especiais), 0022 (numeração) e 0025
   (relatórios); o relatório da Etapa 11 tem as propostas A/B de DEC-11-01.
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). A Etapa 23
   está pronta e ainda não versionada; não misturar commits sem o OK do
   usuário. O `docs.rar` e o rascunho da Etapa 26 nunca entram neste commit.
3. Só começar depois de a aprovação estar registrada no topo deste arquivo;
   dúvidas fora das decisões: parar e perguntar.
4. Criar `docs/internal/plans/<data>-etapa-24.md` com o desenho técnico, a seção
   "Retomada por outro agente" e a lista "Andamento"; marcar cada item só
   com teste passando. Gravar no ai-memory a página fixada
   `notes/etapa-24-verificacoes.md`.
5. Padrões:
   - o Core não depende de GUI;
   - regras puras e determinísticas, cada uma com código estável
     documentado numa tabela no ADR;
   - mudanças de grafo (DEC-11-01) por deltas reversíveis com reconstrução
     dos índices e `validate()` completo, revertendo em erro;
   - nomes nunca criam conectividade;
   - migração encadeada em `persistence.rs::migrate`, com fixture nova;
   - testes egui com accesskit (ajudante que ignora rótulos e títulos de
     janela, como em `cad-gui/tests/cad_export_ui.rs`);
   - não remover nem afrouxar testes.
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; compilar o
   Release sem executar o app. Se o disco encher, `cargo clean`.
7. Atualizar relatório, ADR-0026, `DECISIONS.md` (registrando DEC-11-01/02
   como decididas), `ROADMAP.md` (linha 24), `KNOWN_ISSUES.md` (fechar
   DEC-11-01/02 e CON-001 conforme a decisão), `CURRENT_STATE.md` e
   `HANDOFF.md`; informar e aguardar o OK para versionar. Não iniciar a
   Etapa 25.
