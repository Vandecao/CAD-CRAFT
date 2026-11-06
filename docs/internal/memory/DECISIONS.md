# Decisões arquiteturais

R6 (2026-11-06): decisões de manutenção executadas conforme prompt aprovado: comentários de código em inglês, constantes privadas apenas para frases repetidas pt-BR, lints de workspace e missing_docs restrito a model/format. Exceções de testes/exemplos justificadas individualmente no relatório ../reports/R6_LIMPEZA.md. Sem mudança arquitetural ou novo ADR; ADR-0031/0032 permanecem. Versionamento R6 aguarda OK por etapa.

| ADR | Estado | Assunto |
|---|---|---|
| [0001](../../adr/0001-bootstrap-e-contratos.md) | Aceito pelo Prompt 0 | Core/GUI, contratos e bootstrap mínimo |
| [0002](../../adr/0002-controle-manual-git.md) | Aceito em 2026-10-26; adendo de 2026-11-03 | Controle do versionamento Git pelo usuário; desde 2026-11-03 o agente faz commit + push de cada etapa após OK expresso; armazenamento e backup |
| [0003](../../adr/0003-core-eletrico-e-integridade.md) | Implementado na Etapa 1; revisão do usuário pendente | Agregado Project, IDs v7, valores validados, relações e Serde estrutural |
| [0004](../../adr/0004-conectividade-logica-e-redes-derivadas.md) | Implementado e testado automaticamente na Etapa 2; revisão do usuário pendente | Conexões canônicas, índice reconstruível, DFS iterativa e redes derivadas |
| [0005](../../adr/0005-condutores-cabos-e-terminacoes.md) | Implementado e testado automaticamente na Etapa 3; revisão do usuário pendente | Terminações físicas, condutores, cabos/veias, réguas e rastreabilidade explícita |
| [0006](../../adr/0006-estrutura-documental-e-representacoes.md) | Implementado e testado automaticamente na Etapa 4; revisão do usuário pendente | Organização documental, ocorrências elétricas, transformações, índices e remoção sem cascata |
| [0007](../../adr/0007-interface-desktop-e-sessao-documental.md) | Implementado/testado; homologação visual da Etapa 5 confirmada pelo anexo da Etapa 6 | Desktop egui/eframe, sessão/seleção, viewport independente, grade de pontos, Ribbon estável, tema nativo, preferências e snapshot técnico |
| [0008](../../adr/0008-editor-grafico-e-historico.md) | Implementado/testado na Etapa 6; homologação manual pendente | Lote atômico no Core, seleção e prévia na GUI, dispatcher gráfico, histórico de deltas limitado e compatibilidade de snapshots |
| [0009](../../adr/0009-simbolos-vetoriais-e-insercao.md) | Implementado/testado na Etapa 7; homologação manual abrangente adiada até após Etapa 15 | Definições vetoriais imutáveis, instância na Representation, portas gráficas explícitas, biblioteca/inserção, histórico de inserção/aparência e snapshot autossuficiente |
| [0010](../../adr/0010-editor-de-conexoes-e-juncoes.md) | Implementado/testado e revisado na Etapa 8; homologação após Etapa 15 | Vínculos explícitos de portas, rotas com identidade documental, junções no grafo existente, deltas reversíveis, prévia/commit compartilhados e snapshots compatíveis |
| [0011](../../adr/0011-identificacao-referencias-potenciais.md) | Implementado/testado e revisado na Etapa 9; homologação após Etapa 15 | Intenção geométrica separada da rota derivada; designações, papéis funcionais, referências e marcadores por IDs; grupos documentais com continuidade opcional explícita |
| [0012](../../adr/0012-engenharia-fisica-bornes-cabos.md) | Implementado/testado automaticamente na Etapa 10; homologação após Etapa 15 | Edições físicas por deltas reversíveis e lotes atômicos, pontes como conexões explícitas de régua, numeração não identitária, listas técnicas derivadas e navegadores físicos hierárquicos |
| [0013](../../adr/0013-conectores-e-especializacoes-de-bornes.md) | Implementado/testado automaticamente na Etapa 11; homologação após Etapa 15; DEC-11-01/02 decididas na Etapa 24 (ADR-0026) | Contatos de conector como bornes existentes, conector e acoplamento documental tipados, especializações de borne com regras verificáveis, níveis isolados por ponto, associação ponto↔borne editável |
| [0014](../../adr/0014-arquivo-cadcraft-e-insercao-com-dispositivo.md) | Implementado/testado automaticamente na Etapa 12 (M0.1); homologação após Etapa 15 | Formato `.cadcraft` v1 (ZIP + manifesto versionado) no Core, v0 = snapshot JSON migrado, leitor defensivo; trava, recuperação e gravação atômica na GUI; inserção que cria dispositivo/função/pontos e Undo documental |
| [0015](../../adr/0015-molduras-zonas-e-referencias.md) | Implementado/testado automaticamente na Etapa 13; homologação na Etapa 28 | Molduras no Core com zonas, textos especiais e moldura por página; referências /página.coluna; `.cadcraft` formato 2 |
| [0016](../../adr/0016-editor-de-molduras-imagens-cadframe.md) | Implementado/testado automaticamente na Etapa 14; homologação na Etapa 28 | Editor de molduras, imagens do projeto em `assets/`, marca d'água, `.cadframe` e `.cadcraft` formato 3 |
| [0017](../../adr/0017-intercambio-de-molduras.md) | Implementado na Etapa 15; homologação na Etapa 28 | Importação `.fn1`, `.titleblock`, DXF ASCII; exportação DXF; relatório e lote reversível |
| [0018](../../adr/0018-conteudo-livre-e-area-de-transferencia.md) | Implementado/testado automaticamente na Etapa 16; homologação na Etapa 28 | Conteúdo livre na folha; colar símbolo cria dispositivo novo; recortar remove na hora; `.cadcraft` formato 4 |
| [0019](../../adr/0019-biblioteca-e-editor-de-simbolos.md) | Implementado/testado automaticamente na Etapa 17; homologação na Etapa 28 | Biblioteca `.cadsym`, editor de símbolos, estilos/elipse/textos dinâmicos, versões e atualização, importação `.elmt`; `.cadcraft` formato 5 |
| [0020](../../adr/0020-pecas-dispositivos-funcoes.md) | Implementado/testado automaticamente na Etapa 18; homologação na Etapa 28 | Peças `.cadpart`, atribuição via `Component`, edição reversível de dispositivos/funções/pontos, CSV e `.cadcraft` formato 6 |
| [0021](../../adr/0021-macros.md) | Implementado/testado automaticamente na Etapa 19; homologação na Etapa 28 | Macros `.cadmacro` autossuficientes, IDs novos e próxima designação na inserção, fios internos, macro de página, colar com fios |
| [0022](../../adr/0022-numeracao-referencias.md) | Implementado/testado automaticamente na Etapa 20; homologação na Etapa 28 | Regra de numeração no projeto, prévia + transação única para dispositivos/fios/páginas, número do fio na conexão lógica (formato 7), espelho de contatos derivado |
| [0023](../../adr/0023-pdf-lista-exibicao.md) | Implementado/testado automaticamente na Etapa 21; homologação na Etapa 28 | Lista de exibição no Core, referências da folha com destino, PDF `pdf-writer` (Helvetica WinAnsi, links, marcadores), impressão pelo sistema |
| [0024](../../adr/0024-dxf-dwg.md) | Implementado/testado automaticamente na Etapa 22; homologação na Etapa 28 | DXF R12 de página pela lista de exibição (camadas por categoria, ANSI 1252, um arquivo por página), DWG por conversor externo opcional |
| [0025](../../adr/0025-relatorios.md) | Implementado/testado automaticamente na Etapa 23; homologação na Etapa 28 | Seis relatórios derivados do cadastro em páginas e CSV/XLSX; formato 8, deltas reversíveis e lados A/B dos bornes |
| [0026](../../adr/0026-verificacoes.md) | Implementado/testado automaticamente na Etapa 24; homologação na Etapa 28 | Motor de verificação com 15 regras e esquema no projeto (formato 9); DEC-11-01 proposta A (acoplamento dá continuidade); DEC-11-02 documental; CON-001 como aviso |
| [0027](../../adr/0027-importacao-qet.md) | Implementado/testado automaticamente na Etapa 25; homologação na Etapa 28 | Importação `.qet` como projeto novo: folha pela grade e carimbo, mestre e escravos num dispositivo, fio travado, relatórios de folha como interrupções; `CreateRouteExact` preserva o percurso importado |
| [0028](../../adr/0028-eplan-edz.md) | Implementado/testado automaticamente na Etapa 26; homologação pendente | EDZ ZIP/7z em memória, peças e macros inline/fallback; ESymLib com definições pendente de amostra, conforme ajuste aprovado |
| [0029](../../adr/0029-codigo-aberto-gpl.md) | Implementado no bloco R0 (2026-11-05); publicação pendente | GPL-3.0-or-later, SPDX e deny.toml; amostras de terceiros fora do Git (CADCRAFT_SAMPLES); saídas de referência como garantia da reorganização; AGENTS público e local; docs/internal e docs/adr |
| [0030](../../adr/0030-assistente-ia.md) | Implementado na Etapa 27; teste com login real pendente; homologação na Etapa 28 | Assistente e ajuda: ChatGPT por OAuth (PKCE, 127.0.0.1), nada da conta no disco; crate cadcraft-assistant sem rede; protocolo de ações em texto validado; consultas só de leitura; propostas com cartão e Desfazer; defesas de escopo, sigilo e injection |
| [0031](../../adr/0031-protocolo-dxf-compartilhado.md) | Aceito no R4 | Protocolo DXF std-only compartilhado no modelo, políticas de saída preservadas |
| [0032](../../adr/0032-estado-e-erros-gui.md) | Aceito no R5 | Estado por assunto, erros tipados na GUI e abertura com opener |

Decisões futuras devem registrar contexto, decisão e consequências; não substituir
silenciosamente contratos definidos pelo usuário.

Instalação ambiental de ai-memory em 2026-11-02: não introduz decisão
arquitetural do CAD-CRAFT nem novo ADR. O wiki e seu histórico Git ficam no
diretório de dados externo; a política do ADR-0002 para o repositório do projeto
permanece. Evidências em `docs/internal/reports/AI_MEMORY_INSTALACAO_2026-11-02.md`.

**ADR — Controle manual do versionamento Git:** após a perda grave de código-fonte
e histórico local do ambiente anterior, o usuário determinou separar desenvolvimento
automatizado de versionamento e sincronização manuais. O Codex implementa, testa,
documenta e avisa que as alterações estão prontas; não faz commit ou push.
O usuário revisa, versiona e sincroniza. Isso preserva rastreabilidade, reduz
sobrescritas e mantém revisão antes de cada commit e proteção remota periódica.
Detalhes, exceção específica e consequências no ADR-0002; ADR-0001 preservado.

## R2 — decisões de documentação (2026-11-05)

Prompt aprovado com decisões recomendadas: guias de entrada em inglês, glossário bilíngue e seis exemplos na fachada cad-craft. Execução dos exemplos incluída no validate.ps1 e no CI Linux; Windows usa o mesmo script. Não há decisão arquitetural nova nem ADR adicional: mantém os contratos dos ADRs existentes. Relatório: ../reports/R2_DOCUMENTACAO_ENTRADA.md.

## R3 — organização do modelo (2026-11-05)

Pastas por assunto em inglês, reexports planos mantidos e acesso interno preservado com ancestrais explícitos. Snapshot/integrity continuam ao lado de Project; frame, symbol e macros divididos por subassunto. Tolerância absoluta 1e-6 centraliza o valor existente de molduras; igualdade de identidade, ângulos canônicos e ortogonalidade não é relaxada. Sem mudança de contrato/Serde, portanto sem novo ADR comportamental. Autor autorizou versionar R2+R3 e publicar em privado e público, mantendo históricos separados. Relatório: ../reports/R3_MODELO_POR_ASSUNTO.md.

## R5 — organização da GUI (2026-11-05)

ADR-0032: editores por estado/entrada/desenho, catálogo por aba com ordem original, estado auxiliar agrupado e GuiError nas operações. opener/thiserror aprovados; String somente em apresentação e interfaces injetadas compatíveis. Pacotes nativos já serializados em cadcraft-format desde R1. Codex autorizado a executar o roteiro manual de R5. Publicação conjunta R4/R5 autorizada em históricos separados; R6 não iniciado.
