# ADR-0022 — numeração automática e referências cruzadas

Estado: implementado e testado automaticamente na Etapa 20. Homologação manual geral na Etapa 28.

## Contexto

A Etapa 20 numera dispositivos, fios e páginas e completa as referências cruzadas, sem quebrar a separação entre cadastro elétrico, representação e conectividade derivada.

## Decisão

- A regra (`NumberingScheme`) é dado do projeto. Ela guarda:
  - o formato de dispositivo: sequencial por prefixo (padrão) ou página/coluna;
  - o formato de fio;
  - a ordem: página → coluna → linha, ou linha a linha;
  - o início e o passo;
  - se as designações travadas são mantidas;
  - se a regra vale para inserir, colar e inserir macro.
- Toda renumeração segue o mesmo fluxo:
  - a prévia é imutável e mostra cada linha antes/depois com os conflitos;
  - a aplicação acontece só sem conflitos, confere que a prévia não ficou desatualizada, valida o projeto inteiro e reverte em erro;
  - o delta guarda os estados antes e depois para desfazer e refazer com conferência;
  - cada operação é um `Transaction::*` na GUI.
- Dispositivos:
  - o prefixo é a parte alfabética inicial da designação;
  - todas as funções do dispositivo recebem a designação nova;
  - peças instaladas acompanham pelo ID do dispositivo;
  - no modo página/coluna, dispositivos repetidos na mesma coluna recebem um sufixo (-K2.4, -K2.4.2), como na inserção.
- Fios:
  - o número fica na `Connection` lógica, com trava (decisão 3, formato 7);
  - cada rede derivada recebe um único número;
  - o nome do potencial vence, depois o número travado;
  - esses números ficam reservados e os automáticos não os repetem;
  - o número é só rótulo e nunca cria conectividade;
  - a posição do rótulo pertence à rota.
- Páginas: só o número muda; a ordem e as referências `/página.coluna` são recalculadas da moldura.
- O espelho de contatos é uma consulta derivada (`contact_mirror`) a partir das funções e pontos do dispositivo. Nunca é texto persistido nem editável (decisão 4). Contatos não representados aparecem sem referência.

## Consequências

- O `.cadcraft` sobe para o formato 7 com migração 6→7 (campos opcionais com padrão); v0–v6 continuam abrindo.
- Alterar a regra é configuração do projeto e não entra no Desfazer.
- A classificação NA/NF no espelho usa os rótulos dos pontos (13-14 / 21-22).
- Numeração de bornes e de cabos, verificações completas e relatórios ficam para as Etapas 23–24.
