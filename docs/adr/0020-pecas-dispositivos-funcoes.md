# ADR-0020 — peças, dispositivos, funções e pontos

Estado: implementado e testado automaticamente na Etapa 18. Homologação manual geral na Etapa 28.

## Contexto

A Etapa 18 precisa de um cadastro reutilizável de peças, atribuição a dispositivos e edição de funções e pontos sem confundir definição de peça, instância instalada e representação gráfica.

## Decisão

- Cada peça da biblioteca é um pacote `.cadpart` individual. O projeto conserva cópia das peças usadas; o catálogo externo não é dependência para abrir o projeto.
- `PartId` é textual e estável. `Part` guarda fabricante, código, descrições, classificação, dados técnicos e modelo de funções com pontos e símbolo sugerido.
- `Component` registra a peça instalada e a quantidade. A atribuição completa funções e pontos ausentes sem criar representações. Atualizações de peças são atômicas e recusam perda de função representada.
- Edições de dispositivo, função e ponto usam deltas reversíveis e validação de dependências. Inserir função não representada vincula as portas a pontos existentes por rótulo ou, com aviso, por ordem.
- CSV serve ao cadastro em lote e à lista de peças derivada. Linhas inválidas são relatadas. O pacote `.cadcraft` passa ao formato 6, com migração 5→6.

## Consequências

- A versão anterior do aplicativo não abre arquivos de formato 6.
- Peças inválidas no projeto ou na biblioteca são recusadas; a recarga da biblioteca preserva o catálogo anterior se encontrar um pacote inválido.
- A interface e os testes automáticos não constituem homologação manual. O teste do executável das Etapas 17–19 está previsto para o fim da Etapa 19, com autorização.
