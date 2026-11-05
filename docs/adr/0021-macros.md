# ADR-0021 — macros

Estado: implementado e testado automaticamente na Etapa 19. Homologação manual geral na Etapa 28.

## Contexto

A Etapa 19 precisa guardar trechos de esquema (símbolos, dispositivos, fios, junções, textos) e reinseri-los em qualquer página ou projeto sem quebrar a separação entre cadastro elétrico, representação e conectividade derivada.

## Decisão

- A macro guarda as entidades como estavam na origem (`MacroContent`), sem pertencer a nenhum projeto. Cada macro é um `.cadmacro` individual (ZIP com manifesto) com símbolos e peças embutidos; o `.cadcraft` não guarda macros e continua no formato 6.
- Capturar leva os fios com as duas pontas na seleção, as junções cujas rotas entram todas, os dispositivos com todas as funções e pontos (as não representadas na seleção ficam não representadas no destino) e as peças instaladas. Engenharia física (bornes, cabos, condutores físicos) e vínculos de ponto com borne são descartados e relatados.
- Inserir troca todos os IDs por UUID v7 novos de uma vez (substituição no JSON das entidades), dá a cada dispositivo a próxima designação livre por prefixo no mesmo caminho `=`/`+`, compõe a transformação de posicionamento com a de cada ocorrência e transforma rotas e junções. Símbolo ou peça já existente com conteúdo diferente: mantém a versão do projeto e avisa.
- A inserção escreve tudo, reconstrói os índices e valida o projeto inteiro; qualquer erro desfaz a escrita. O delta guarda as entidades finais para desfazer/refazer com conferência de estado. A validação da macro é a mesma inserção num projeto de rascunho.
- Copiar e Duplicar uma seleção que contém fios usam o mesmo motor (CLP-001). Recortar com fios continua recusado; colar "no mesmo dispositivo" com fios é recusado.

## Consequências

- Macros são independentes do projeto de origem e podem ser trocadas entre projetos.
- Rotação/espelho exigem orientações permitidas pelos símbolos; coordenadas na grade (ponto base ajustado) mantêm a geometria exata.
- Placeholders/conjuntos de valores, variantes por macro e atualização de ocorrências já inseridas ficam fora do escopo.
