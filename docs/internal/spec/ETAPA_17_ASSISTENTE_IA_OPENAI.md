# ETAPA 17 — CAD-CRAFT AI

Status: PLANEJADA — requisitos aprovados pelo usuário.
Implementação: exclusivamente na Etapa 17.

## Arquitetura

O CAD-CRAFT AI será um plugin nativo do CAD-CRAFT,
com funcionamento semelhante aos assistentes de IA
integrados ao PyCharm e ao VS Code.

O plugin será integrado à Ribbon e ao ambiente gráfico,
com painel próprio de conversação e acesso contextual
ao projeto elétrico atualmente aberto.

## Provedor e autenticação

- Provedor exclusivo: OpenAI.
- Não implementar suporte a Google, Gemini ou outros provedores.
- Priorizar autenticação OAuth oficialmente autorizada.
- Não exigir chaves de API fornecidas pelo usuário.
- Verificar a disponibilidade real do fluxo oficial antes de implementar.
- Não utilizar cookies extraídos, sessões não autorizadas ou métodos improvisados.
- Caso o OAuth pretendido não esteja disponível oficialmente,
  documentar a limitação e solicitar decisão arquitetural.

## Funcionalidades obrigatórias

1. Consultar componentes, dispositivos, terminais e conexões.
2. Compreender as relações elétricas existentes no projeto.
3. Analisar erros de engenharia e inconsistências.
4. Consultar documentação técnica.
5. Sugerir alterações no projeto.
6. Utilizar ferramentas internas do CAD-CRAFT.
7. Apresentar prévia das alterações antes da execução.
8. Executar alterações somente após autorização do usuário.
9. Integrar operações ao histórico transacional Undo/Redo.
10. Explicar o funcionamento do CAD-CRAFT.
11. Ensinar a utilizar funções, comandos e ferramentas.
12. Orientar configurações e procedimentos de operação.
13. Compreender o contexto da página e dos objetos selecionados.
14. Permitir interação conversacional semelhante aos agentes de IDEs.
15. Preservar a integridade elétrica e documental do projeto.

## Segurança e execução

O agente não poderá modificar diretamente estruturas internas
ignorando as validações do Core.

As alterações deverão utilizar comandos e ferramentas
validados pelo CAD-CRAFT.

Ações de escrita exigem autorização do usuário.
Operações autorizadas devem produzir histórico reversível.

Não transmitir indiscriminadamente projetos, credenciais
ou informações confidenciais.

## Planejamento

Não antecipar a implementação para etapas anteriores.

Etapa 15: homologação geral do desenvolvimento acumulado.
Etapa 17: implementação do plugin nativo CAD-CRAFT AI.

Este documento constitui requisito funcional aprovado
e deverá ser consultado obrigatoriamente pelo Codex.
