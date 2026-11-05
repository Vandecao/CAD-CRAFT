# Problemas conhecidos

## R3 — modelo por assunto (2026-11-05)

Sem novo problema funcional identificado; validação completa e Release aprovados. Comparações exatas de coordenadas/ângulos/topologia foram mantidas para não alterar contratos; inventário conservador em ../reports/R3_COMPARACOES_F64.md. Aplicação geral de tolerância fora dos contratos atuais exige etapa comportamental própria. Problemas anteriores permanecem abaixo.


## R2 — documentação de entrada (2026-11-05)

Sem novo problema funcional identificado. Os limites existentes (incluindo GUI-L01 e perdas dos importadores) permanecem registrados abaixo e em docs/FORMATS.md. Nenhum comportamento alterado; validação completa aprovada (692 Rust + 6 exemplos + 16 PowerShell, exit 0). Teste no app não se aplica ao bloco R2.


## Etapa 27 — assistente de IA e ajuda

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| IA-L01 | Resolvido | Login e respostas reais testados no app em 2026-11-05 (relatório da Etapa 27) | — |
| GUI-L01 | Pendente | O app parado usa CPU alta (~45% de um núcleo já no R1 minimizado; ~75% maximizado), provavelmente redesenho contínuo | Investigar o que pede repaint a cada quadro; corrigir num bloco próprio |
| IA-L02 | Limite deliberado | Nada da conta é salvo: cada abertura do app pede login e faz um registro dinâmico novo na conta | Opção "manter conectado" no cofre do Windows, se o autor pedir |
| IA-L03 | Limite | Propostas cobrem numeração, número de fio, documento e página; inserir símbolos, ligar fios ou corrigir regras VER específicas ainda não | Ampliar o catálogo de propostas por etapa |
| IA-L04 | Limite | Escopo e sigilo dependem também do modelo; a defesa principal é estrutural (sem acesso a código, arquivos ou rede; ações validadas) | Conjunto de perguntas de teste no teste do app |
| IA-L05 | Observação | Protocolo de ações em texto porque a documentação do fluxo não descreve o formato de chamada de ferramentas | Migrar para o formato nativo quando documentado |

## Bloco R0 — preparação para código aberto

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| R0-L01 | Limite | Saídas de referência (`baseline_outputs.rs`) conferidas só no Windows; no Linux o teste fica ignorado (funções trigonométricas podem diferir no último bit) | Gravar uma referência por plataforma, se o CI Linux passar a exigir |
| R0-L02 | Pendente | Captura de tela do README ainda não feita | Próximo teste no app, com autorização |
| R0-L03 | A confirmar | `repository` dos `Cargo.toml` aponta para `Vandecao/CAD-CRAFT`; o repositório público será novo e pode ter outro nome | Ajustar na publicação |
| R0-L04 | Observação | O CI em GitHub Actions ainda não rodou: o workflow só executa depois do push | Conferir a primeira execução após o commit |
| R0-L05 | Observação | O código de conduta adota o Contributor Covenant 2.1 por referência (link e atribuição), sem copiar o texto integral | Copiar o texto se o autor preferir |

## Etapa 26 — EDZ (limites conhecidos)

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| EDZ-L01 | Ajuste aprovado pelo usuário | Amostras têm referências ESymLib sem definições; conversão de gráficos inline e caixa com portas conhecidas | Definições ESymLib dependem de amostra demonstrativa |
| EDZ-L02 | Conversão parcial relatada | Códigos sem mapeamento comprovado, ligações PXF e posições de múltiplas portas não são inferidos; tipos numéricos de função são preservados como EPLAN.categoria/grupo/id | Relatório por pacote e tabela de códigos no ADR-0028 |
| EDZ-L03 | Conversão parcial relatada | Primeira variante 2D; estilos/textos dinâmicos sem equivalência e arcos elípticos parciais são relatados | Revisão técnica e visual do material importado |
| EDZ-L04 | Limite deliberado | Imagens, documentos e 3D somente relatados; listas/AutomationML/DXF/DWG EPLAN fora desta entrega | Etapa 26b mediante amostras e aprovação |
| EDZ-L05 | Limite de segurança | Leitor 7z aceita subconjunto limitado de cabeçalhos/codecs; metadados internos até 4 MiB são lidos pela crate antes da conferência de quantidade de itens; limites não são um teto de RAM do processo | Entradas incompatíveis são recusadas com erro |
| EDZ-L06 | **Feito** (2026-11-05) | Teste no app das Etapas 23–26 pelo Claude Code; macros não relidas (JSON, `float_roundtrip`) e tipo de função no editor de peças corrigidos | `docs/internal/reports/TESTE_APP_ETAPAS_23_26.md`; homologação geral na 28 |
| EDZ-L09 | Observação | A prévia lista uma linha por código PXF e por propriedade preservada (centenas por pacote) | Agrupar a prévia, se pedido |
| EDZ-L07 | Limite operacional | Gravação atômica por arquivo, sem transação de lote; falha/cancelamento informa arquivos já concluídos | Reanalisar e resolver conflitos antes de tentar novamente |
| EDZ-L08 | Entrada recusada corretamente | RIT.2383210.edz referencia macro 2D ausente; conversão do pacote é recusada, sem gravação | Reobter pacote completo junto ao fabricante; não fabricar arquivo ausente |

## Etapa 25 — importação QElectroTech

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| QET-L01 | Pendente | `.qetz` (projeto em SQLite) não é lido: não há amostras e exigiria uma dependência nova | Etapa futura, com amostra e aprovação da dependência |
| QET-L02 | Aproximação | Textos girados viram horizontais e arcos elípticos viram circulares (importador `.elmt`, Etapa 17) | Primitivas novas no Core, se pedido |
| QET-L03 | Limite | Imagens e formas desenhadas na folha (`<images>`, `<shapes>`) e campos `input` dos elementos não são importados; aparecem no relatório | Conteúdo livre com imagem, se pedido |
| QET-L04 | Aproximação | Rótulo repetido fora de mestre/escravo vira o mesmo dispositivo; mestre repetido recebe a próxima designação livre | Revisar no projeto importado; VER-002 aponta repetições |
| QET-L05 | Limite | Condutor com terminal inexistente ou ligado a elemento fora da coleção é ignorado (ex.: 24 de 130 em `tableau_domestique.qet`) | Relatório de importação |
| QET-L06 | Limite | Fórmulas de rótulo e de número do QElectroTech (`%prefix%l%c`, `formula`) não são reavaliadas: vale o texto gravado | Numeração do CAD-CRAFT (Etapa 20) |
| QET-L07 | **Feito** (2026-11-05) | Teste no app das Etapas 23–26; rótulos e textos da ocorrência e traços dos relatórios de folha corrigidos | `docs/internal/reports/TESTE_APP_ETAPAS_23_26.md` |

## Etapa 24 — verificações

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| VER-L01 | Limite deliberado | Não é ERC normativo nem cálculo elétrico (seção pela corrente, proteção, queda de tensão) | Etapas futuras, se pedido |
| VER-L02 | Limite | A reavaliação após edições recalcula o projeto inteiro em segundo plano (0,24 s para 20.040 ocorrências) e só remove mensagens; problemas novos só aparecem ao verificar de novo | Por decisão 5 |
| VER-L03 | Limite | Mudar o ponto de um contato acoplado é recusado; é preciso desacoplar, mudar e acoplar de novo | Editar o acoplamento em um passo, se pedido |
| VER-L04 | Observação | Mensagens de cabos e conectores não têm posição na folha; a navegação só informa | Navegar para o editor físico (Etapa 28) |
| VER-L05 | **Feito** (2026-11-05) | Teste no app das Etapas 23–26 | `docs/internal/reports/TESTE_APP_ETAPAS_23_26.md` |
| DEC-11-01 | **Decidido** (Etapa 24, proposta A) | O acoplamento possui conexões lógicas e dá continuidade | ADR-0026 |
| DEC-11-02 | **Decidido** (Etapa 24) | Seccionável documental; VER-016 informa os abertos | ADR-0026 |
| CON-001 | **Decidido** (Etapa 24) | Designações de conector repetidas são avisadas (VER-002) e não unem redes | ADR-0026. O outro CON-001 desta lista (extremidade livre de rota) é um item diferente e continua aberto |

## Etapa 23 — relatórios

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| REL-001 | Decisão de domínio | O cadastro identifica `TerminalSide::A/B`, sem vínculo com interno/externo; a lista mostra Lado A/B e Sem lado, conforme resposta do usuário | Mapear somente se o cadastro passar a definir o significado |
| REL-002 | Limite deliberado | As linhas são derivadas em tempo de exibição; “desatualizado” indica diferença na quantidade necessária de páginas, não mudança de valores com a mesma quantidade | Decisão aprovada da Etapa 23 |
| REL-003 | Pendente | Sem teste no executável nesta etapa, conforme prompt; homologação manual geral na Etapa 28 | Próximo teste no app após a Etapa 25 |


## Etapa 22 — DXF e DWG

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| DXF-001 | Limite deliberado | DXF R12 sem espessura por entidade: a espessura está no nome da camada (`CAD-CRAFT Fios 0.35mm`); o programa que abrir deve atribuir *lineweight* por camada | R2000 se pedido |
| DXF-002 | Limite deliberado | Sem imagens no R12: o logotipo vira caixa com X na camada Guias e é relatado | R14+ se pedido |
| DXF-003 | Limite | Os textos usam a fonte do programa que abre o arquivo; a largura pode variar em relação ao PDF (o fator de largura é calculado pela Helvetica) | — |
| DXF-004 | Limite | O preenchimento em `SOLID` usa leque a partir do primeiro ponto: correto para as formas convexas da folha (junções, retângulos, círculos), não para polígonos côncavos | Hachura em R2000+ |
| DXF-005 | Limite deliberado | Geometria explodida, sem `BLOCK`/`INSERT` com atributos | Etapa 26 (EPLAN), se pedido |
| DWG-001 | Limite deliberado | DWG depende do ODA File Converter instalado e configurado pelo usuário (licença da Open Design Alliance); sem ele, o botão fica desabilitado no diálogo. O comando da faixa continua disponível e abre o diálogo com a explicação | Configurações |
| DWG-002 | Pendente | O conversor real não foi testado, nem no executável (ODA não instalado); os testes usam scripts `.cmd` que imitam a linha de comando do ODA | Testar quando o usuário instalar o conversor |
| DXF-006 | Observação | 200 páginas em DXF: 1,45 s no Release e 9,6 s em Debug; o custo dominante é montar a lista de exibição (mesmo do PDF, PDF-005) | Otimizar na Etapa 28 |
| APP-022-1 | Corrigido | Teste no executável: número do fio sobre o traço e com 11 px fixos; agora ao lado do traço, 2,5 mm, mesma regra na tela, no PDF e no DXF | — |
| APP-022-2 | Corrigido | Teste no executável: "→" e outros símbolos como quadrados (fonte do egui); Segoe UI Symbol como fonte reserva | Em outros sistemas, DejaVu/Apple Symbols, se existirem |
| APP-022-3 | Observação | O espelho, abaixo do canto esquerdo da bobina, cruza o fio que sai dela | Avaliar a posição das referências (Etapa 28) |
| APP-022-4 | Observação | Na primeira saída do app, Confirmar não fechou a janela; na segunda fechou | Reobservar na Etapa 28 |

## Etapa 21 — PDF e impressão

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| PDF-001 | Limite deliberado | Helvetica WinAnsi sem fonte embutida: caracteres fora (Ω, →, ideogramas) saem como `?` e aparecem no relatório | Fonte embutida na Etapa 28, se pedido |
| PDF-002 | Limite deliberado | A tela continua desenhada pelo egui; o PDF usa a lista de exibição do Core. Os textos de referência são os mesmos, mas a tipografia difere (fonte do egui × Helvetica) | Unificar na Etapa 28, se necessário |
| PDF-003 | Limite | Imagens no modo preto e branco ficam como estão (só o vetorial vira preto/cinza) | Futuro |
| PDF-004 | Limite | A impressão depende do leitor de PDF padrão do Windows aceitar o verbo "imprimir"; sem leitor associado, o sistema recusa e a mensagem aparece | Instalar um leitor de PDF |
| PDF-005 | Observação | Exportar 200 páginas típicas: 1,07 s no Release (meta 10 s), 9,4 s em Debug; o tempo cresce um pouco além do linear porque as referências cruzadas percorrem o projeto a cada página | Otimizar na Etapa 28 |
| PDF-006 | Pendente | Sem teste no executável nesta etapa | Fim da Etapa 22, com autorização |
| PDF-007 | Limite | O PDF temporário da impressão fica na pasta temporária do sistema (o leitor ainda pode estar usando o arquivo) | Limpeza pelo sistema |

## Etapa 20 — numeração e referências cruzadas

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| NUM-001 | Limite deliberado | Alterar a regra de numeração (diálogo ou Configurações) é configuração do projeto e não entra no Desfazer; as renumerações entram | Futuro, se pedido |
| NUM-002 | Limite | Espelho classifica NA/NF pelos rótulos dos pontos (x3-x4 / x1-x2); outros rótulos aparecem como "Contato" | Etapa 24 (tipos de contato) |
| NUM-003 | Limite | Numerar conexões vale para o projeto todo (sem alcance por página/seleção) | Futuro |
| NUM-004 | Limite | Posição do rótulo do fio só por X/Y em Propriedades da conexão; sem arrastar na folha | Futuro |
| NUM-005 | Limite | Ordem "coluna" usa a coordenada X (equivalente às zonas verticais da moldura); contadores sequenciais por prefixo são comuns a todos os caminhos `=`/`+` | Avaliar na Etapa 24 |
| NUM-006 | Limite | Espelho na folha é texto sob a função principal, não o desenho dos contatos | Futuro |
| NUM-007 | Pendente | Sem teste no executável nesta etapa | Fim da Etapa 22, com autorização |
| TST-020 | Corrigido | `project_lifecycle` usava o botão "Identificação" da faixa como rótulo depois do novo botão "Numerar páginas"; o ajudante ignora botões | — |

## Etapa 19 — macros

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| MAC-001 | Feito em parte | Teste no executável das Etapas 18–19 em 2026-11-04: macros, colar com fios, peças OK; inserir função não representada não concluído por cliques bloqueados no painel direito (ambiente) | Repetir na Etapa 28 |
| MAC-006 | Corrigido | Prévias sob o cursor (macro, Colar, texto/gráfico) ficavam escondidas pela folha; agora em camada acima | Sem teste automático visual |
| MAC-007 | Limite | Prévia de Colar com fios mostra só os símbolos | Futuro |
| PART-004 | Observação | Peça com rótulos de ponto diferentes dos do símbolo já inserido cria funções novas não representadas | Avaliar casamento por tipo |
| MAC-002 | Limite deliberado | Engenharia física (bornes, cabos, condutores físicos) e vínculos de borne ficam fora da macro (relatados) | Futuro |
| MAC-003 | Limite deliberado | Recortar com fios é recusado; colar "no mesmo dispositivo" com fios é recusado | Copiar + excluir |
| MAC-004 | Limite deliberado | Sem placeholders/conjuntos de valores nem atualização de ocorrências já inseridas | Futuro |
| MAC-005 | Limite deliberado | Sem pasta configurada, macros criadas ficam só na sessão | Configurações |

## Etapa 18 — peças e dispositivos

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| PART-001 | Limite deliberado | Sem pasta configurada, peças cadastradas na janela ficam apenas na sessão | Configurar biblioteca de peças |
| PART-002 | Feito em parte | Teste no executável em 2026-11-04 (ver MAC-001) | Etapa 28 |
| PART-003 | Limite | Associação por ordem de portas exige conferir rótulos no Centro de inserção | Preferir símbolo com rótulos correspondentes |

## Etapa 17 — biblioteca e editor de símbolos

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| SYM-001 | Limite deliberado | Preenchimento de polígono não convexo desenhado como convexo | Triangulação futura |
| SYM-002 | Aproximação registrada | `.elmt`: setas, cantos arredondados, cores, arcos elípticos e textos fixos girados aproximados (relatório avisa) | — |
| SYM-003 | Limite deliberado | Sem pasta configurada, a biblioteca de símbolos vale só para a sessão | Configurações |
| SYM-004 | Pendente | Teste no executável das Etapas 17–19 | Fim da Etapa 19, com autorização |

## Etapa 16 — conteúdo livre e área de transferência

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| CLP-001 | Corrigido com teste | Etapa 19: copiar/duplicar seleção com fios leva fios e junções internos (motor de macros); recortar com fios segue recusado | `copy_paste_and_duplicate_carry_internal_wires` |
| CLP-002 | Limite deliberado | Dispositivo criado ao colar recebe designação sem caminho `=`/`+` | Numeração por regras (futuro) |
| CLP-003 | Limite deliberado | Recortar símbolo deixa dispositivo/função/pontos sem ocorrência até a colagem | — |
| CLP-004 | Limite deliberado | Área de transferência só da sessão (sem sistema, sem outro projeto) | Futuro |
| CLP-005 | Feito | Teste funcional no executável das Etapas 14–16 em 2026-11-03 (relatório da Etapa 16) | Próximo ao fim da Etapa 19 |
| CLP-006 | Corrigido com regressão | Ctrl+C/X/V não funcionavam no executável: no Windows chegam como eventos Copy/Cut/Paste | `clipboard_events_from_the_os_drive_copy_cut_and_paste` |
| CLP-007 | Aberto | Atalhos ignorados enquanto um elemento de painel tem foco (ex.: após duplo clique na árvore); clicar na folha resolve | Avaliar foco ao abrir página |
| CLP-008 | Limite | Símbolo com alvo `Device` (modelo antigo) é colado como cópia do mesmo dispositivo | Migrar para função ou estender a regra |
| GUI-016 | Aberto | Descrição multilinha de moldura EPLAN mostra "□" no campo de linha única | Campo multilinha |
| FILE-005b | Aberto | Cópia de recuperação ainda oferecida após descartar ao fechar | Revisar Etapa 12 |

## Etapa 15 — intercâmbio de molduras

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| FRM-015 | Limite deliberado | DXF R12 não guarda imagens e a exportação mantém só o idioma padrão; o relatório avisa | Editor preserva originais no `.cadcraft`/`.cadframe` |
| FRM-016 | Limite deliberado | Moldura EPLAN sem borda do tamanho da folha não fornece âncora segura para normalização automática | Ajustar no editor se necessário |
| FRM-017 | Teste no app feito em 2026-11-03 (importação FN1_011, DXF ida e volta); diálogos nativos usados | Homologação geral na Etapa 28 | — |


## Etapa 14 — editor de molduras, imagens e `.cadframe`

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| FRM-008 | Limite deliberado | Snapshot técnico JSON leva só metadados das imagens; projeto com imagem sem conteúdo não é salvo como `.cadcraft` (recusa explícita) e a folha mostra marcador | — |
| FRM-009 | Fora do escopo | QR code e hiperlink na moldura | Decisão futura |
| FRM-010 | Limite deliberado | Cada edição confirmada no editor aumenta a versão da moldura | Avaliar versão por sessão de edição |
| FRM-011 | Fora do escopo | Editor sem seleção por retângulo, alças de redimensionar e rotação gráfica (feitos pelo painel) | Etapa de refinamento |
| FRM-012 | Limite deliberado | Arquivos da Etapa 14 (formato 3) não abrem na versão da Etapa 13 | Regra do ADR-0014 |
| FRM-013 | Corrigido com regressão | Textura ficava ausente quando a pré-visualização era desenhada antes da inserção da imagem | `frame_is_created_from_scratch_with_texts_specials_and_logo` |
| FRM-014 | Feito em 2026-11-03 | Teste no app das Etapas 14–16 (relatório da Etapa 16) | — |

## Etapa 13 — molduras, zonas e referências

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| FRM-001 | Resolvido na Etapa 14 (exceto QR e hiperlink, FRM-009) | Sem editor gráfico de molduras, imagens/logotipos, marca d'água, QR e hiperlink | Etapa 14 |
| FRM-002 | Resolvido na Etapa 15 | Importadores `.fn1`, `.titleblock` e DXF, mapeamento de propriedades e fluxo GUI disponíveis | Homologação geral na Etapa 28 |
| FRM-003 | Limite deliberado | Textos fixos mostram o valor padrão (pt-BR); variantes por idioma só são preservadas | Seleção de idioma futura |
| FRM-004 | Limite deliberado | Moldura com tamanho diferente da folha é desenhada como está, com aviso no painel | — |
| FRM-005 | Medição pendente | Referências na folha recalculadas a cada quadro (linear nas representações) | Medir com projetos reais (Etapa 28) |
| FRM-006 | Limite deliberado | Arquivos da Etapa 13 (formato 2) não abrem na versão da Etapa 12 (`min_reader` 2) | Regra do ADR-0014 |
| FRM-007 | Corrigido com regressão | Editar moldura/metadados pelo painel fazia a página perder a seleção | `properties_panel_assigns_page_frame_and_edits_project_metadata` |

## Etapa 12 — arquivo `.cadcraft` e inserção com dispositivo

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| FILE-001 | Limite deliberado | Trava cooperativa sem verificação de processo vivo: após falha, a trava antiga exige "Ignorar a trava" explícito | Detecção de trava órfã futura, se necessária |
| FILE-002 | Limite deliberado | `project.json` é estrito: formato futuro que altere seu conteúdo precisa elevar `min_reader` (somente leitura vale para entradas novas no ZIP) | Regra no ADR-0014 |
| FILE-003 | Limite deliberado | Histórico Undo/Redo continua por sessão; abrir/salvar não o persiste. Edições de propriedades por `Edit::apply` (descrição, identificação, formato de página, propriedades técnicas) ainda limpam o histórico, como antes | Avaliar no histórico geral |
| FILE-004 | Medição local | Projeto com JSON de 16,2 MiB → `.cadcraft` 1,2 MiB; gravação ~2,9 s e leitura ~2,2 s em Debug (inclui validação completa) | Medir em Release com projetos reais |
| FILE-005 | Sem teste automatizado | Diálogo nativo `rfd` não é exercitado nos testes (substituído por `ScriptedPicker`) | Homologação manual após Etapa 15 |
| PLC-001 | Limite deliberado | Desfazer inserção com dispositivo revalida o agregado inteiro (como PHY-012) | Medir com projetos reais |
| PLC-002 | Limite deliberado | `next_device_designation` considera apenas dispositivos sem caminho funcional/localização; prefixos sugeridos por símbolo não seguem norma | Numeração por regras (etapa futura) |
| PLC-003 | Fora do escopo | Renomear/excluir documento não existem na GUI; `Project::remove_document` existe para o Undo | Futuro |
| M0-001 | Teste manual informado pelo usuário | Em 2026-11-03 o usuário executou o roteiro manual da Etapa 12 (criar, inserir com dispositivo, conectar, salvar, reabrir, Undo, trava, recuperação, demonstrações) e informou "tudo funcionando" | Homologação geral em momento a definir no novo plano |

## Etapa 11 — conectores e especializações de bornes

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| DEC-11-01 | Decidido na Etapa 24 (proposta A; ver ADR-0026) | Acoplamento de conectores é documental: mapa explícito validado, sem arestas nem continuidade. Unir redes pelas metades exige conexões lógicas de posse do acoplamento (análogo às pontes) — mudança de semântica do grafo | Propostas A/B no relatório da Etapa 11; não implementar sem decisão |
| DEC-11-02 | Decidido na Etapa 24 (documental; ver ADR-0026) | Estado de borne seccionável (`Closed/Open`) é estático/documental, sem efeito em redes | Alternativa: pontos por lado + ligação interna explícita controlada pelo estado |
| PHY-010 | Resolvido na Etapa 11 | Tipos PE, multinível, distribuição, passagem e seccionável têm regras verificáveis no Core (`Terminal::check_specialization`, níveis em pontos, PE em condutores/pontes) | — |
| PHY-011 | Parcialmente resolvido | Associação ponto↔borne/contato agora é editável (`SetPointTerminal`, aba Conectores e editor de especialização), restrita a pontos do dispositivo dono; criação de pontos funcionais continua fora do editor físico | Editor de dispositivos/pontos futuro |
| CON-001 | Decidido na Etapa 24 (aviso VER-002) | Designações de conector não são únicas no projeto; nomes iguais nunca unem redes | Rever com numeração/ERC (etapas futuras) |
| CON-002 | Limite deliberado | "Parear por numeração" só preenche um rascunho; o acoplamento vale apenas após "Confirmar acoplamento" | — |
| CON-003 | Limite deliberado | "Excluir conector e contatos" é um lote atômico explícito; recusado se algum contato estiver associado, fiado ou acoplado. `DeleteConnector` no Core remove só o agrupamento | — |
| GUI-PHY-03 | Corrigido com regressão automatizada | Em telas pequenas/escalas altas a janela Engenharia física (850×610 de conteúdo + moldura) podia exceder a tela; agora `default_size`/`max_size` limitam-se à área disponível. Teste `connector_and_specialization_views_are_stable_across_resolutions_and_scales` (5 resoluções/escalas) | Reteste manual após Etapa 15 |
| GUI-PHY-04 | Observação de teste | O corpo com `ScrollArea::both` pode rolar horizontalmente (linhas largas pré-existentes da aba Réguas) e animar rolagem; os testes egui clicam na parte visível e aguardam a animação | Sem impacto funcional observado |
| CON-004 | Pendente de homologação | Fluxos GUI cobertos por testes com cliques egui reais; sem ensaio manual | Homologação após Etapa 15 |

## Etapa 10 — correções e limites

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| GUI-ROUTE-09A | Corrigido com regressões automatizadas | Causa: `repair_endpoints` reparava mesmo com extremos inalterados e inseria cotovelo colinear ao primeiro/último vértice autoral, desenhando ida-e-volta horizontal. Correção: identidade quando extremos coincidem e remoção desse vértice só na polilinha derivada; base autoral intacta, logo o retorno restaura a geometria. Testes `horizontal_endpoint_repair_does_not_draw_a_residual_backtrack`, ciclos de redimensionamento, grupo, junção e snapshot em `crates/cad-craft/tests/connection_drawing.rs` | Homologação manual após Etapa 15 |
| GUI-ROUTE-09B | Corrigido com regressões automatizadas | Causa: a rota derivava só de endpoints/base, sem o corpo transformado do símbolo; após 180°/270° a saída da porta apontava para dentro do corpo. Correção: `route_clearance.rs` transforma primitivas fechadas em polígonos, afastamento de 2,0 mm documentais, desvio ortogonal determinístico sem mover endpoint; flag `auto_clearance` persistida e reverificada na importação; prévia usa o mesmo algoritmo (`preview_route_points`). Testes de rotações 0/90/180/270, espelhos, proximidade, movimento e Undo; captura egui em 3 zooms (`connection_view.rs`) | Traços abertos (não fechados) não são obstáculos; símbolo sem percurso livre é recusado explicitamente |
| GUI-PHY-01 | Corrigido com regressões automatizadas (aguarda reteste manual) | Relatado na homologação: janela Engenharia física/Listas técnicas crescia sem parar. Causa: `ScrollArea` de altura total seguida da linha de retorno; `egui::Resize` expande a janela até o conteúdo do frame anterior (+28 px/frame). Correção: retorno em painel inferior interno, corpo em painel central com rolagem, cabeçalho truncado. Testes `physical_window_size_is_stable_across_frames_on_every_tab` e `manual_resize_of_physical_window_is_preserved` | Reteste manual pelo usuário |
| GUI-PHY-02 | Corrigido com regressões automatizadas (aguarda reteste manual) | Regressão de GUI-PHY-01: `CentralPanel/TopBottomPanel::show_inside` não alocam no `Ui` da janela; a janela media só as abas (~81 px) e os controles eram pintados fora da moldura. Correção: `Layout::bottom_up` (retorno) + `top_down` (corpo com rolagem), ambos alocados. Testes `physical_window_size_is_stable_and_bounded_on_every_tab` e `manual_resize_of_physical_window_is_preserved_and_bounded` verificam tamanho e formas pintadas dentro do fundo | Reteste manual; Release a refazer com o app fechado |
| PHY-010 | Limite deliberado | Tipos especiais de borne (PE, multinível, distribuição, seccionável) não declarados: sem regras validáveis no Core | Especificar regras antes de expor tipos |
| PHY-011 | Limite deliberado | Associação de condutor exige pontos funcionais com borne físico explícito e borne com posição de conexão; a GUI informa, não cria entidades | Cadastro de terminal↔ponto em editor de dispositivos futuro |
| PHY-012 | Medição local | Cada edição física revalida o projeto inteiro; lote de 4.002 registros e derivação de listas medidos em Debug abaixo de 5 s / 2 s (limites de teste); transform recalcula rotas da página afetada | Sem garantia industrial; medir projetos reais |
| PHY-013 | Resolvido na Etapa 11 | `pwsh` disponível em 2026-11-02 (sessão da Etapa 11); `scripts/validate.ps1` executou os 15 testes PowerShell: 15 aprovados | — |
| PHY-014 | Pendente de homologação | Editor físico, navegadores e listas cobertos por testes Core/GUI e um fluxo com cliques egui reais; sem ensaio manual | Homologação após Etapa 15 |

## Etapa 9 — limites atuais

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| REF-001 | Limite deliberado | Referências usam página e coordenadas reais; sem moldura de coordenadas, não se inventam colunas | Expandir quando houver contrato documental para moldura |
| REF-002 | Limite deliberado | Snapshots anteriores conservam pontos existentes como restrições de rota porque não registravam intenção manual | Rotas novas registram intenção; não inferir proveniência antiga |
| REF-003 | Limite deliberado | Grupos de interrupção são documentais; continuidade elétrica somente com opção explícita e arestas de posse controlada | Não associar rede por nome igual |
| REF-004 | Pendente de homologação | Testes automatizados e revisão independente não substituem ensaio manual abrangente | Homologação após Etapa 15 |
| BACKUP-002 | Pendente do usuário | Etapa 9 e esta memória estão fora do HEAD 5377559f; remoto consultado coincide com esse HEAD em 2026-11-02 | Commit manual da Etapa 9 e backup externo independente; push conforme política na Etapa 10 |


| ID | Estado | Evidência | Próxima ação / critério de encerramento |
|---|---|---|---|
| ENV-001 | Resolvido e verificado | MSVC/SDK instalados; cargo test e cargo run linkaram executáveis | Build Tools 17.14.42, MSVC 19.44.35229, SDK 10.0.26100.0 |
| ENV-002 | Não reproduzido no ambiente atual | Leituras e comandos funcionaram após atualização de permissões nesta sessão; antes ocorreu helper_unknown_error | Reavaliar somente se o ambiente restrito voltar a ser usado; não afirmar correção do sandbox |
| GIT-001 | Resolvido e verificado | Autoria local configurada; f902d4784be6072a333e919da66af1150aa7884b idêntico no HEAD e remoto após primeiro push | Origin preservado, main rastreia origin/main |
| ENV-003 | Observação pendente | winget pediu reinício; vswhere informou isRebootRequired=false; compilação funcionou | Reiniciar Windows em momento conveniente e repetir scripts/validate.ps1; não foi reiniciado nesta sessão |
| BACKUP-001 | Superado em parte | O anexo da Etapa 9 confirma commit/push manuais da Etapa 8; em 2026-11-02 HEAD local e remoto consultado em 5377559f. Backup externo não verificado | Backup externo independente continua recomendado; ver BACKUP-002 |
| GIT-002 | Consulta confirmada | git ls-remote --exit-code origin refs/heads/main retornou 4a27a8fea6711bd589c544f242afc9c5481e4a89 em 2026-10-30 | Fotografia da consulta, sem fetch/sincronização |
| DIAG-001 | Limitação documentada | Diagnóstico offline usa tracking local e nomes/tamanhos de arquivos atuais, não conteúdo nem histórico inteiro | Não interpretar relatório limpo como ausência garantida de segredos ou confirmação atual do remoto |
| CORE-001 | Limites deliberados das Etapas 1–4 | Modelo estrutural, Serde, conectividade, fiação e representações documentais testados; sem ERC, renderização no Core, CRUD completo, histórico ou armazenamento físico final; designações não certificadas IEC | Evoluir apenas nos prompts apropriados; manter contratos dos ADRs 0003–0006 |
| NET-001 | Limites deliberados da Etapa 2 | Redes são fotografias derivadas sem identidade persistente; sem simulação/estados internos ou atributos genéricos de Connection; EntityRef não aponta para Connection | Reconsultar após mutação; identidade/propriedades/referências para conexões exigem requisitos futuros |
| PHY-001 | Limites deliberados da Etapa 3 | Sem remoção/renomeação de posições, CRUD físico genérico, especializações multinível, capacidade de borne, continuidade interna, dimensionamento ou verificação normativa; material/cor/função em enums básicos | Expandir apenas sob requisitos futuros; condutor exige extremos existentes, vínculo direto a no máximo uma Connection; detalhamento no ADR-0005 |
| PERF-001 | Medição local limitada | Grafos e fiação física com 100, 1.000 e 10.000 elementos testados; medições Instant locais não são capacidade garantida industrial; importação sem limite de recursos | Medir cargas reais antes de otimizar; consultas por dispositivo percorrem terminais e importação completa reconstrói índices |
| DOC-001 | Limites documentais preservados | Etapa 7 acrescenta símbolos e hit de traços; representações anteriores sem geometria usam limites/âncora; árvore usa caminhos sem IDs de grupo; navegação entre ocorrências do mesmo alvo; dimensões zero preservadas; rotas acrescentadas na Etapa 8 | Consumir contratos documentais com essas distinções; ampliar somente em escopo autorizado |
| DOC-002 | Limitação de evidência/recursos | Compatibilidade testada por esquemas anteriores simulados, sem fixtures históricas congeladas; escala documental das Etapas 6/7 medida proporcionalmente, sem garantia industrial; união de extremos pode exceder f64 e retornar erro | GUI trata erros e limita a câmera; medir cargas reais e definir limites na persistência futura, sem declarar capacidade garantida |
| GUI-001 | Limites deliberados da Etapa 7 | Etapa 8 acrescenta editor multifilar e rotas; ERC, escala/estiramento e docking livre continuam futuros | Evoluir apenas pelos prompts seguintes; comandos futuros desabilitados |
| GUI-002 | Homologação abrangente adiada até após Etapa 15 | Etapa 5 tem confirmação visual histórica; Etapas 6–8 cobertas por testes e capturas internas; nenhuma confirmação manual dessas etapas | Retomar homologação no momento autorizado; Windows 11/DPI nativo/alto contraste/hardware diverso permanecem sem homologação |
| GUI-003 | Medição proporcional, sem garantia industrial | 10.000 objetos mistos, 40 hit queries ~329 ms, 20 previews de grupo ~122 ms e commit ~47 ms (Debug local); não mede frame completo | Hit percorre página e pintura ordena por frame; medir projetos reais antes de índice espacial |
| GUI-004 | Persistência final futura | Preferências visuais salvas; sessão/abas/favoritos transitórios; snapshot técnico JSON limitado a 8 MiB | Não tratar JSON experimental como formato final, migração ou backup |
| ENV-004 | Timeout externo anterior não reinvestigado | Anexo da Etapa 6 autoriza screenshots internos eframe; imagens reais válidas inspecionadas nesta entrega | Não confundir captura interna com correção do Computer Use ou homologação de decoração nativa |
| ENV-005 | Reinício do Codex desktop pendente | ai-memory 3.3.1 validado na CLI nativa com hooks confiados, MCP habilitado e 1 sessão capturada; a sessão desktop atual antecede a instalação | Reiniciar o Codex desktop e consultar `ai-memory doctor` numa sessão nova se a captura não aparecer; relatório `AI_MEMORY_INSTALACAO_2026-11-02.md` |
| GUI-005 | Limite do backend nativo documentado | winit 0.30.13 pode reaplicar tema do sistema no WM_SETTINGCHANGE sem emitir mudança; correção reaplica Dark ao foco/tema observado | Cor escura confirmada pelo usuário no Windows 10 build 19045.6466; mudança sem evento/foco pode exigir alternar janela. Windows 11/alto contraste não homologados |
| GUI-006 | Histórico gráfico limitado por escopo | Transforms, inserção, aparência, rotas e junções; 100 operações / 50.000 deltas; não é Undo geral; edições fora do histórico gráfico encerram as pilhas; dirty conservador | Expandir somente com requisitos futuros; histórico não é persistido |
| GUI-007 | Corrigido e testado na Etapa 6 | Rascunho múltiplo abandonado podia reaparecer após trocar seleção/página; selection_revision invalida mesmo com inspetor oculto | Regressão RED/GREEN aprovada e revisão independente concluída |
| SYM-001 | Limites deliberados da biblioteca | Dez desenhos originais simplificados, sem certificação IEC, editor de definições/catálogo de fabricantes ou versões simultâneas por ID; associação somente antes da inserção; exclusão genérica não implementada | Evoluir sob requisitos futuros; Undo da inserção remove ocorrência; portas não têm vínculo lógico implícito |
| SYM-002 | Limites gráficos documentados | AABB conservadora para curvas oblíquas; texto em caixa nominal, seleção textual pela caixa; tesselação de curvas com teto de 2.048 partes | Não prometer tipografia normativa ou precisão visual ilimitada; hit de curvas é analítico |
| SYM-003 | Medição proporcional, sem garantia industrial | Debug: 10.000 símbolos/5 páginas/10 definições; inserção ~279 ms, 40 consultas em 2.000 ~375 ms, transformação de 2.000 ~29 ms; sem medição de frame completo ou pico de memória | Hit linear por página e ordenação da pintura permanecem; medir projetos reais antes de otimizar |
| SYM-004 | Corrigido e revisado na Etapa 7 | Ângulo negativo quase zero podia virar 360 em regras/portas e divergir de Transform2D/round-trip; canonização compartilhada | Dois testes RED/GREEN e revisão focada aprovados |
| SYM-005 | Corrigido e testado na Etapa 7 | Normalizar UV antes do fim do frame corrompia texto quando atlas crescia; manter texels até tesselação egui | Regressão RED/GREEN aprovada; pintura revisada |
| SYM-006 | Corrigido e revisado na Etapa 7 | Controles fixos ocultavam linhas em painel compacto; controles e lista virtualizada agora rolam juntos | Teste com eventos reais de roda RED/GREEN e revisão focada aprovados |

Validação oficial da Etapa 7: 257 Rust + 15 PowerShell = **272 aprovados**, zero
falhas/ignorados; 36 novos, mantendo os 236 anteriores. Core 134, GUI 123.
Release aprovado, PE Subsystem 2. Capturas finais internas reais em .local/etapa-07;
revisões independentes concluídas. Relatório ETAPA_07_BIBLIOTECA_SIMBOLOS.md.
Etapa 7 incorporada ao HEAD 6bfa4c2; Etapa 8 requer commit + push manuais. Homologação após Etapa 15.

## Etapa 8 — limites e correções

| ID | Estado | Evidência / limite | Continuidade |
|---|---|---|---|
| CON-001 | Limite explícito | Rotas terminam em portas de símbolos; junções T/múltiplas unem rotas já existentes. Não há ramo com extremidade livre porta→segmento/junção | Evoluir endpoint gráfico tipado em etapa futura; não confundir testes de T topológico com esse fluxo por mouse |
| CON-002 | Limite explícito | Posse de arestas auxiliares é exclusiva; junção que exigiria apropriar aresta alheia é recusada | Projetar compartilhamento apenas com contrato de remoção/replay verificável |
| CON-003 | Limite explícito | Autoconnect é sugestão visual determinística com confirmação; sem roteador industrial, target tracking, interrupção entre páginas ou potencial completo | Requisitos futuros no roadmap, não aprovados |
| CON-004 | Medição local | Core reconstrói índices em confirmação; hit/snap percorrem candidatos da página, cache por revisão. 1.001 rotas Core e 1.000 GUI medidos; cinco frames CPU+tesselação GUI ~1,992 s no cenário adversarial denso, custo elevado | Sem garantia industrial; medir hardware/projetos reais antes de otimizar |
| CON-005 | Corrigido com regressões | Seleção de junção anterior podia sobreviver a clique normal; junção sobre porta não selecionável; rota interceptava símbolo/Ctrl; grupo com rota recusava transform | Correções de arbitragem/seleção e teste de eventos; evidências RED/GREEN no relatório |
| CON-006 | Corrigido com regressões | Prévia de rota usava reparo diferente do Core e marcador de junção não acompanhava translação de grupo | Algoritmo compartilhado points_with_endpoints e prévia do marcador por deslocamento comum |
| CON-007 | Revisão final concluída | Após interrupção temporária, revisor independente conferiu as correções e os logs finais; GUI-REV-01–04 fechados, nenhum P1/P2 funcional aberto | Harness final 7/7 e validação oficial aprovados; não equivale a homologação |

Snapshot continua técnico, até 8 MiB; sem salvamento final/autosave/migrações.
Homologação manual geral, DPI físico, Windows 11, alto contraste, backup externo e
ENV-003 continuam pendentes. Referências EPLAN são funcionais/visuais, sem cópia de
código ou ativos proprietários. Etapa 17 permanece somente planejada.

## Pendencias graficas identificadas na homologacao da Etapa 9

### GUI-ROUTE-09A — Prolongamentos residuais horizontais
Status: CORRIGIDO COM REGRESSÕES AUTOMATIZADAS na Etapa 10 (ver tabela "Etapa 10"); homologação manual após Etapa 15.

Ao movimentar componentes conectados, os segmentos horizontais
podem manter prolongamentos residuais quando o componente retorna.

Movimentacoes verticais apresentam comportamento correto.
Conectividade eletrica e Undo/Redo permanecem funcionais.

### GUI-ROUTE-09B — Rota atravessando simbolo rotacionado
Status: CORRIGIDO COM REGRESSÕES AUTOMATIZADAS na Etapa 10 (ver tabela "Etapa 10"); homologação manual após Etapa 15.

Apos rotacoes de 180 ou 270 graus, dependendo da orientacao,
o condutor pode atravessar o corpo grafico do simbolo em vez de
contorna-lo ate o terminal eletrico correspondente.

A conectividade eletrica permanece preservada.

Requisitos:
- Preservar endpoints eletricos e identidade das conexoes.
- Eliminar prolongamentos graficos desnecessarios.
- Respeitar a geometria dos simbolos.
- Preservar vertices manuais, juncoes e Undo/Redo.
- Implementar testes de regressao.
- Corrigir antes de desenvolver as novas funcoes da Etapa 10.
