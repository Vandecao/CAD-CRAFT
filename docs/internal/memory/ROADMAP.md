# Roadmap — evolução do CAD-CRAFT

R0–R6 publicados nos dois repositórios. Etapa 28: partes A–C implementadas e validadas (758 verificações), Release e diagnóstico nativo de CPU concluídos; Parte D/homologação geral pendente de Claude Code e autor. O Plano Mestre permanece aberto para esse aceite. Relatórios ETAPA_28_DESEMPENHO_ROBUSTEZ.md e ETAPA_28_HOMOLOGACAO.md. Versionamento Etapa 28 ainda não autorizado. A tabela abaixo conserva dados históricos das entregas; este parágrafo e a linha 28 registram o estado atual.

Etapas 1–12 receberam prompts. Em 2026-11-03 o usuário aprovou o **Plano
Mestre** com as Etapas 13–28 abaixo. As linhas 13–28 são **planejamento
aprovado, não autorização de implementação**: cada etapa exige prompt próprio
aprovado e execução sequencial. Nenhuma funcionalidade previamente aprovada foi
removida. A homologação manual geral ocorre na Etapa 28; a aprovação visual
histórica da Etapa 5 e o teste manual informado da Etapa 12 não equivalem a
homologação geral de plataforma/hardware.

| Etapa | Escopo / planejamento | Implementação | Testada automaticamente | Homologada manualmente | Versionamento |
|---|---|---|---|---|---|
| 1 | Core elétrico | Implementada | Sim | Pendente | Commit manual dfbbbc1 |
| 2 | Conectividade e redes derivadas | Implementada | Sim | Pendente | Commit manual 9844571 |
| 3 | Condutores, cabos e terminações físicas | Implementada | Sim | Pendente | Commit manual df6b482 |
| 4 | Documentos, páginas e representações | Implementada | Sim | Pendente | Commit manual 5eac964 |
| 5 | Desktop egui/eframe e layout | Implementada | Sim | Visual confirmada no anexo da Etapa 6 | Commit manual 5a5526d |
| 6 | Editor gráfico e histórico de transforms | Implementada | Sim, 236 totais na entrega | Na Etapa 28 | Commit manual 4a27a8f |
| 7 | Biblioteca de símbolos e inserção | Implementada | Sim, 272 totais na entrega | Na Etapa 28 | Commit manual 6bfa4c2 |
| 8 | Conexões gráficas, portas vinculadas, redes e junções explícitas | Implementada e revisada | Sim, 339 totais | Na Etapa 28 | Commit manual 5377559 |
| 9 | Identificação, referências cruzadas, interrupções e potenciais | Implementada e revisada | Sim, 379 totais | Na Etapa 28 | Commit manual a4f8233 |
| 10 | Correções GUI-ROUTE-09A/09B; editor de réguas, bornes e pontes; cabos, veias, condutores e associação física; navegadores e listas técnicas | Implementada e revisada | Sim, 391 Rust | Na Etapa 28 | Commit + push manuais f1b173e |
| 11 | Conectores industriais, acoplamento documental e especializações de bornes (passagem, PE, multinível, distribuição, seccionável) | Implementada e revisada | Sim, 418 Rust + 15 PowerShell | Na Etapa 28 | Commit manual 34785ab; enviado junto com o push da Etapa 12 |
| 12 | Marco M0.1 (auditoria CAD-CRAFT × QElectroTech aprovada): arquivo `.cadcraft` v1, abrir/salvar/recentes, trava, recuperação, v0 migrado; inserção com criação de dispositivo; Undo de documentos/páginas | Implementada | Sim, 452 Rust + 15 PowerShell | Teste manual do roteiro da etapa informado como aprovado pelo usuário (2026-11-03); homologação geral a definir | Commit + push pelo agente com autorização expressa do usuário (2026-11-03) |
| 13 | Fase A — modelo de moldura: colunas/linhas com rótulos e área útil, textos fixos e textos especiais (propriedades de projeto/página), moldura por projeto com troca por página, desenho na folha, referências cruzadas página.coluna (resolve REF-001), formatos A0–A4 e personalizados | Implementada | Sim, 466 Rust + 15 PowerShell | Validação funcional na interface pelo agente (2026-11-03); homologação geral na Etapa 28 | Commit + push pelo agente após OK do usuário (2026-11-03) |
| 14 | Fase A — editor de molduras: criar do zero e editar (linhas, retângulos, círculos, arcos, textos, textos especiais, imagens PNG/JPG em `assets/`), biblioteca de molduras com arquivo próprio para importar/exportar | Implementada | Sim, 480 Rust + 15 PowerShell | Na Etapa 28 | Commit + push pelo agente após OK do usuário |
| 15 | Fase A — molduras de outros sistemas: DXF (importar/exportar, campos variáveis → textos especiais), `.titleblock` (QElectroTech) e `.fn1` do EPLAN (XML PXF), validados com molduras reais do usuário | Implementada, pronta para revisão | Sim, 495 Rust + 15 PowerShell | Na Etapa 28 | Alterações fora do HEAD; usuário proibiu commit/push nesta retomada |
| 16 | Fase B — texto livre e gráficos na página; copiar/recortar/colar na página e entre páginas | Implementada (Codex + Claude Code) | Sim, 512 Rust + 15 PowerShell | Teste no app pendente de autorização; homologação na Etapa 28 | Commit + push após OK do usuário |
| 17 | Fase B — biblioteca externa de símbolos, primitivas ampliadas (elipse, estilos, textos dinâmicos), editor de símbolos, importação `.elmt` | Implementada | Sim, 526 Rust + 15 PowerShell | Teste no app ao fim da Etapa 19; homologação na Etapa 28 | Commit `0886379`, enviado a `origin/main` |
| 18 | Fase B — cadastro de dispositivos e peças, editor de funções e pontos, CSV e inserção de função não representada | Implementada | Sim, 539 Rust + 15 PowerShell | Teste no app ao fim da Etapa 19; homologação na Etapa 28 | Commit `98ef54a` (com a 19), enviado a `origin/main` |
| 19 | Fase B — macros: criar da seleção/página, inserir com novos IDs e designações, biblioteca `.cadmacro`, importar/exportar, colar com fios | Implementada | Sim, 556 Rust + 15 PowerShell | Teste no app das Etapas 18–19 feito em 2026-11-04; homologação na Etapa 28 | Commit `98ef54a`, enviado a `origin/main` |
| 20 | Fase B — numeração automática (dispositivos, condutores, páginas) e referências cruzadas completas (espelho de contatos) | Implementada (Codex + Claude Code) | Sim, 577 Rust + 15 PowerShell | Teste no app ao fim da Etapa 22; homologação na Etapa 28 | Commit `1fe3659` (com a 21), enviado a `origin/main` |
| 21 | Fase C — PDF multipágina com links de referência; impressão | Implementada | Sim, 586 Rust + 15 PowerShell | Teste no app ao fim da Etapa 22; homologação na Etapa 28 | Commit `1fe3659`, enviado a `origin/main` |
| 22 | Fase C — exportação DXF; DWG conforme decisão de licença (ODA ou conversor externo) | Implementada | Sim, 597 Rust + 15 PowerShell | Teste no app das Etapas 20–22 feito em 2026-11-04 (3 defeitos corrigidos); homologação na Etapa 28 | Fora do HEAD; commit + push após OK expresso |
| 23 | Fase C — relatórios em página e CSV/XLSX: índice, dispositivos, materiais, conexões, bornes e cabos | Implementada (Codex + Claude Code) | Sim, 618 Rust + 15 PowerShell | Teste no app feito em 2026-11-05 (Etapas 23–26); homologação na Etapa 28 | Fora do HEAD; commit + push somente após OK expresso |
| 24 | Fase C — verificações de consistência com navegação até erros; decisões DEC-11-01/02 e CON-001 | Implementada | Sim, 630 Rust + 15 PowerShell | Teste no app feito em 2026-11-05 (Etapas 23–26); homologação na Etapa 28 | Fora do HEAD; commit + push após OK expresso |
| 25 | Fase D — importação de projetos QElectroTech (`.qet`; `.qetz` pendente) | Implementada | Sim, 640 Rust + 15 PowerShell | Teste no app feito em 2026-11-05 (Etapas 23–26); homologação na Etapa 28 | Fora do HEAD; commit + push após o teste no app e o OK |
| 26 | Fase D — EDZ EPLAN: peças e macros 2D inline/fallback; definições ESymLib pendentes de amostra (ajuste aprovado); demais formatos em 26b | Implementada | Sim, 666 Rust + 15 PowerShell; Release; 46 amostras (45 convertidas, 1 recusada por referência ausente) | Teste no app feito em 2026-11-05 (Etapas 23–26); homologação geral na 28 | Fora do HEAD; commit + push após teste no app e OK expresso |
| R0 | Preparação para código aberto: GPL-3.0-or-later, amostras fora do Git, saídas de referência, README, CI, AGENTS público/local, docs/internal (plano `docs/internal/plans/PLANO_REORGANIZACAO_R0-R6.md`) | Implementado | Sim | — | Fora do HEAD; commit + push após OK; publicação depois, com autorização |
| Pub. | Publicação em repositório público novo, commit inicial limpo | Executada; R0–R6 sincronizados com históricos separados | — | — | Público Vandecao/CAD-CRAFT; privado Vandecao/CAD-CRAFT1 |
| R1 | Divisão em crates (model, format, render, import, export; fachada cad-craft) | Implementada | Validação registrada no relatório R1 | Teste no app registrado no HANDOFF | Commit local cdca6fc, incluído no HEAD observado em 2026-11-05 |
| R2 | Documentação de entrada: arquitetura, tour, glossário, formatos, seis exemplos e introduções de módulo | Implementado | 692 Rust + 6 exemplos + 16 PowerShell, exit 0 | Não se aplica; sem teste no app | Fora do HEAD; pronto para revisão/versionamento, sem commit/push |
| R3 | Modelo por assunto, arquivos menores, tolerance.rs e inventário de comparações | Implementado | 694 Rust + 6 exemplos + 16 PowerShell, Release | Não se aplica; teste no app no R5 | Commit/push R2+R3 autorizados para privado e público; recibo .local/r3/PUBLICATION.md |
| 27 | Fase E — assistente de IA e ajuda, cadcraft-assistant sem rede; OAuth na GUI | Implementada | 689 Rust + 16 PowerShell registrados na Etapa 27 | Teste com login registrado no HANDOFF; homologação geral na 28 | Commit local 06a6cbf, HEAD observado em 2026-11-05 |
| 28 | Fase F — desempenho, robustez, telas e homologação geral | Implementada e homologada (Codex + Claude Code); 6 defeitos corrigidos | Sim, 743 Rust + 6 exemplos + 16 PowerShell; Release | Aceita pelo autor em 2026-11-07 | Commit + push após o OK de 2026-11-07 |
| 29 | Manter conectado ao ChatGPT (cofre do Windows, opção desmarcada por padrão) | Implementada (Claude Code) | Sim, 751 Rust + 6 exemplos + 16 PowerShell; Release | Fim da 31 | Fora do HEAD; commit + push após OK |
| 30 | Colunas e linhas da moldura: redistribuição, mensagens claras, divisões geradas, formato 10 | Planejada (prompt aprovado) | — | Fim da 31 | — |
| 31 | Área de trabalho como no EPLAN: fundo uniforme, folha segue a moldura, grade que some, linhas-guia do cursor | Planejada (prompt aprovado) | — | Teste no app das 29–31 | — |

### Plano Mestre aprovado (2026-11-03)

Meta final escolhida pelo usuário: ECAD utilizável em projetos reais +
interoperabilidade com EPLAN 2027 + assistente de IA. Fases: A folha
profissional (13–15, prioridade do usuário), B conteúdo do esquema (16–20),
C saídas (21–24), D interoperabilidade (25–26), E IA (27), F fechamento (28).
Plano detalhado: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md`. Cada etapa continua
exigindo prompt próprio aprovado pelo usuário antes da implementação.

- Validação: testes automáticos de aceitação por etapa (com eventos egui
  reais); homologação manual geral **somente na Etapa 28**, por escolha do
  usuário. Verificação rápida opcional ao fim de cada fase.
- Git: ao fim de cada etapa o agente avisa; com o OK expresso do usuário
  ("pode versionar") faz commit + push daquela etapa (ADR-0002, adendo).
- Dependências: 14 depende de 13; 15 de 13/14; 21–22 exigem a moldura (13);
  20 usa zonas (13); 19 usa copiar/colar (16) e símbolos (17); 23 usa
  numeração (20); 26 depende de amostras do EPLAN 2027 do usuário.

### Restrições

QElectroTech (GPL) é referência de conceitos e formatos; nenhum código é
copiado. A coleção de elementos (CC-BY 3.0) não pode servir de base de dados
para construir modelos de aprendizado de máquina; o assistente da Etapa 27
ajuda o usuário a operar o programa e montar projetos, sem treinar modelos com
esses elementos, e os elementos não são embutidos no repositório. Molduras e
dados criados pelo usuário no EPLAN são do usuário; o formato PXF é
interpretado apenas para interoperabilidade, a partir de arquivos fornecidos
por ele. Dependências novas exigem autorização. Sem promessa de conformidade
IEC, do catálogo EPLAN completo ou de formatos comerciais sem
análise/licenciamento.
Melhoria visual da grade abaixo permanece preservada; a seção da antiga
"Etapa 17" descreve a atual Etapa 27.
Último versionamento: commit `6612e82` (Etapa 12) enviado a `origin/main` em
2026-11-03. Próxima ação: aprovação do prompt da Etapa 13.

## Melhoria futura: suavizacao visual da grade

Status: Planejada para uma futura etapa de refinamento grafico.

Objetivo:
Melhorar a renderizacao da grade de pontos em niveis de zoom afastados,
reduzindo a poluicao visual sem alterar a precisao geometrica.

Requisitos obrigatorios:

1. O espacamento logico da grade permanece constante em milimetros.
2. Uma grade configurada para 4 mm continua sendo de 4 mm em qualquer zoom.
3. O snap utiliza sempre o espacamento logico original.
4. A densidade dos pontos renderizados pode diminuir progressivamente
   durante o zoom out.
5. Pontos intermediarios podem ser ocultados, mas nunca deslocados.
6. Pontos visiveis devem permanecer alinhados a origem documental.
7. A transicao entre niveis de densidade deve ser visualmente suave,
   evitando saltos perceptiveis.
8. Zoom e pan nao podem alterar coordenadas nem propriedades do projeto.
9. A renderizacao deve permanecer eficiente mesmo com zoom muito afastado.
10. A melhoria nao deve modificar o modelo eletrico nem a escala real.

Implementacao futura: camada de renderizacao do viewport egui.

Observacao: nao implementar durante a homologacao da Etapa 5.

## Etapa 27 (antiga Etapa 17) — CAD-CRAFT AI

Implementar o plugin nativo de assistente de inteligencia artificial,
com provedor exclusivo OpenAI e autenticacao OAuth oficial.
O assistente deve operar de forma semelhante aos agentes integrados
ao PyCharm e VS Code.

Especificacao obrigatoria:
docs/internal/spec/ETAPA_17_ASSISTENTE_IA_OPENAI.md

Nao antecipar a implementacao para etapas anteriores.
