# ADR-0032 — Estado por assunto e erros de operação na GUI

Data: 2026-11-05. Estado: aceito no R5, conforme decisões recomendadas aprovadas.

## Contexto

O app concentrava estado auxiliar e implementações extensas. Operações de arquivo perdiam a categoria e a causa dos erros ao convertê-los cedo em String. A abertura de PDF no Windows passava o caminho por cmd.

## Decisão

Dividir editores por estado, entrada e desenho, e separar o catálogo de comandos por aba, preservando a ordem global. Agrupar 32 campos privados de CadApp em navegação, painéis, diálogos, conteúdo livre e interface. Manter os pontos públicos usados pelos testes, sem alterar seus eventos ou rótulos.

GuiError usa thiserror e preserva fontes de E/S, persistência, importação, domínio e exportação. GuiResult atravessa operações e trabalhos em segundo plano. Context mantém os prefixos anteriores com fonte encadeada. Strings continuam nos registros de apresentação clonáveis e na interface SystemShell injetada pelos testes; a conversão ocorre nessa fronteira. O método contains mantém a consulta textual existente nos testes sem descartar a fonte do erro.

Usar opener para abrir o arquivo padrão sem construir uma linha de shell. A impressão conserva sua implementação anterior. opener 0.8.5 e thiserror 2.0.21 estão fixados no manifesto/lock; licenças MIT OR Apache-2.0 compatíveis com o projeto.

## Consequências

O estado e os erros ficam localizados sem mudar o modelo ou os formatos. Os codecs de cadsym, cadmacro, cadpart e cadframe já pertencem a cadcraft-format desde R1; a GUI conserva seleção de caminho, limites de arquivo e política de gravação atômica. Não há serialização de pacote duplicada a transferir neste bloco. Módulos usados apenas internamente passam a pub(crate). A revisão e a validação estão registradas no relatório R5.
