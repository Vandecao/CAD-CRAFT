# Etapa 28 — revisão de pendências

Revisão em 2026-11-06. Este inventário complementa KNOWN_ISSUES, preservando o histórico. Prazos abaixo são marcos para reavaliar pendências, não declaração de funcionalidades já entregues nem autorização para novos projetos.

| Grupo / IDs | Tratamento nesta etapa | Próximo marco / prazo de revisão |
|---|---|---|
| GUI-L01 | Causa reproduzida: Title enviado por quadro solicita novo repaint no egui. Cache por título e despertar pelo prazo real de autosave, com regressões. Medição final Release no relatório técnico | Fechar após medição final e reteste na homologação |
| GUI-016 | Campo de descrição da moldura passa a multiline, com preservação de linhas e Undo/Redo | Fechar após teste automático e confirmação A-05 |
| CLP-007 | Reproduzido e corrigido: foco da árvore não bloqueia atalhos; edição textual continua protegida. Dois testes novos, paste real com posicionamento e Undo | Confirmar em B-03 na homologação; tecnicamente resolvido |
| FILE-005b / APP-022-4 | Já há proteção contra recriar recuperação após saída confirmada, com regressão project_lifecycle. Novo teste mata processo real durante escrita temporária e conserva checkpoint | PRE-04/05 na homologação; não inferir comportamento do diálogo nativo por teste de Core |
| R0-L03 | repository do workspace aponta para o público Vandecao/CAD-CRAFT efetivamente criado; condição histórica superada | Encerrado por conferência do Cargo.toml e remoto público |
| R0-L01 | Baselines exatos permanecem Windows; não inventar equivalência trigonométrica no Linux | Revisar antes de exigir baseline exato no CI Linux, até 2026-11-13 |
| R0-L02 / R0-L04 | Screenshot do README e confirmação do CI remoto não são comprovados por validação local | Revisar na homologação/publicação da Etapa 28; manter pendentes até evidência |
| FRM-005 / FILE-004 / PDF-005 / DXF-006 | Nova carga grande mede abrir/salvar, display, PDF/DXF, numeração, verificações e relatórios | Evidências do benchmark final; os limites de dados e de impressão continuam explícitos |
| REL-003 / PDF-006 / NUM-007 / SYM-004 / CON-004 / PART-002 / MAC-001 / APP-022-3 | Relatos anteriores não substituem roteiro completo. Função não representada, espelho, editores e fluxos físicos têm itens específicos | Homologação B/C; se reproduzidos, corrigir com regressão antes do aceite |
| VER-L04 | Navegação física precisa de reteste em B-08/C-06; não prometer posição de folha para entidade que não tem representação | Homologação; correção pequena se reproduzida, caso contrário plano de editor na próxima revisão até 2026-11-13 |
| DWG-002 / PDF-004 | Dependem de ODA/leitor configurados pelo autor. Sem ambiente adequado registrar observação, não sucesso | C-02/04 na homologação; instalar/configurar somente por decisão do autor |
| EDZ-L01–05,07–09 / QET-L01–06 / FRM-015–016 | Limites de interoperabilidade preservados e relatados; definições ausentes não podem ser fabricadas. QETZ, ESymLib/3D/AutomationML e novos codecs exigem amostras/escopo próprio | D-01–05; revisar prioridades e amostras até 2026-11-13, implementação só com prompt aprovado |
| IA-L02–05 | Sessão não persistida, catálogo de propostas restrito e protocolo textual são limites deliberados; nenhuma alteração de autenticação nesta etapa | E-01–04, conferência do autor; expansões exigem decisão própria |
| Demais limites deliberados do Core/GUI/exportação | Não são regressões ocultas: manter aviso/perda e contratos atuais. Incluem clipboard interno, macros sem engenharia física, campos sem histórico, fontes PDF, geometria R12, preenchimento côncavo e atualizações de referências | Conferir impactos no roteiro; priorização após aceite, revisão até 2026-11-13 |
| Relatórios da carga grande | Medir o custo de gerar todas as tabelas; a especificação não fixa meta para relatórios. Custo alto deve constar como observação com causa, sem alterar tabelas ou enfraquecer testes para parecer rápido | Reavaliar perfil por dispositivo na próxima revisão de desempenho até 2026-11-13 |

Qualquer defeito impeditivo encontrado na homologação bloqueia a conclusão geral do Plano Mestre. A revisão de limites deliberados não os transforma em funcionalidades implementadas.
