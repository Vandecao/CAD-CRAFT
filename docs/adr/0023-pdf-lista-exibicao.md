# ADR-0023 — lista de exibição e PDF multipágina

Estado: implementado e testado automaticamente na Etapa 21. Homologação manual geral na Etapa 28.

## Contexto

A Etapa 21 exporta a documentação em PDF e permite imprimir. Até aqui a folha só existia como desenho egui na GUI, e o Core não sabia "o que a folha mostra". A Etapa 22 (DXF) precisa da mesma informação.

## Decisão

- **Lista de exibição no Core** (`cad-craft/src/display.rs`). `Project::page_display_list(page)` devolve, em mm, os itens da folha:
  - caminhos: linhas, polilinhas, polígonos preenchidos, curvas tesselizadas com erro de 0,01 mm;
  - textos, com a matriz caixa → página;
  - imagens;
  - áreas de link.

  Os itens seguem a ordem de desenho da tela, sem os auxílios de edição. As cores são lógicas (tinta, referência, marca d'água, guia); quem escreve decide o valor.
- **Referências da folha no Core** (`project/sheet_references.rs`). As referências cruzadas, o espelho e as interrupções são segmentos de texto com destino. A GUI desenha o texto concatenado, igual ao anterior; o PDF usa os segmentos para criar os links.
- **PDF com `pdf-writer` 0.15** (decisão 1; `lopdf` 0.47 só nos testes):
  - uma página PDF por página do projeto, no tamanho real ou no papel de impressão (ajustar ou 1:1, centralizado, na mesma orientação);
  - conteúdo comprimido com Flate;
  - Helvetica WinAnsi sem embutir (decisão 2), com a tabela de larguras AFM em `font.rs`;
  - textos de símbolo esticados até a caixa nominal da tela;
  - caracteres fora da fonte trocados por `?` e relatados;
  - JPEG embutido como DCT; PNG decodificado (`image`, aprovado na Etapa 14) em RGB + máscara alfa.
- **Links** (decisão 5): anotações `/Link` com `GoTo` para a página e a posição do destino; destino fora do alcance não vira link e é contado no relatório. Os marcadores seguem documento → páginas.
- **Impressão** (decisão 3): gera um PDF temporário e chama o verbo "imprimir" do sistema (`Start-Process -Verb Print` no Windows). O acesso ao sistema fica atrás de `SystemShell`, e os testes usam um gravador.
- **GUI**: diálogo único Exportar PDF/Imprimir. A geração roda numa thread com o projeto clonado, envia progresso por canal e é cancelável; a gravação do arquivo também acontece na thread.

## Consequências

- Tela e PDF compartilham os textos de referência, mas a geometria da tela continua sendo desenhada pelo egui (decisão 4). Pequenas diferenças de tipografia são esperadas: a tela usa a fonte do egui, o PDF usa Helvetica.
- A lista de exibição é a base do DXF da Etapa 22.
- Ficam fora: fonte embutida (Unicode completo), PDF/A, camadas e impressão nativa.
