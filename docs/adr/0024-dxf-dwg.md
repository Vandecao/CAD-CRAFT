# ADR-0024 — exportação DXF de página e DWG por conversor externo

Estado: implementado e testado automaticamente na Etapa 22. Homologação manual geral na Etapa 28.

## Contexto

A Etapa 22 exporta as páginas para programas CAD. A Etapa 21 criou a lista de exibição no Core justamente para servir também a esta etapa. O DWG é um formato fechado; o Plano Mestre já previa "biblioteca comercial (ODA) ou conversor externo; DXF primeiro".

## Decisão

- **Camadas na lista de exibição.** `DisplayLayer` classifica cada item em Moldura, Símbolos, Fios, Textos, Referências, Junções ou Guias. O PDF ignora a camada.
- **DXF R12 (AC1009), escritor próprio** (`cad-craft/src/dxf_export.rs`, decisão 1):
  - unidades em mm, `$DWGCODEPAGE ANSI_1252`, limites e extensão da folha;
  - tabelas `LTYPE` (um tipo por padrão de traço), `LAYER` (`CAD-CRAFT <Categoria> <espessura>mm`, cor ACI por categoria) e `STYLE`;
  - `LINE` e `POLYLINE`; preenchimento em `SOLID` (leque);
  - `TEXT` com altura de maiúscula, rotação, fator de largura e espelhamento, derivados da matriz da lista;
  - imagens como caixa marcada e relatada, porque o R12 não tem `IMAGE`.
- **Texto** (decisão 2): bytes ANSI 1252; os outros caracteres como `\U+XXXX`, relatados; `%%` gravado literal.
- **Um arquivo por página** (decisão 3), com nome `<projeto>_<documento>_<página>.dxf` sanitizado e único (`_2`, `_3`…).
- **Geometria explodida** (decisão 5): igual ao PDF, sem `BLOCK`/`INSERT`.
- **DWG por conversor externo opcional** (decisão 4, `cad-gui/src/dwg.rs`):
  - o usuário instala e configura o ODA File Converter em Configurações; o CAD-CRAFT não distribui nem baixa o conversor;
  - os DXF vão para uma pasta temporária, e o conversor é chamado com argumentos separados (`entrada saída ACADxxxx DWG 0 1 *.DXF`), sem shell e sem janela de console, com tempo limite que encerra o processo;
  - os `.dwg` gerados são copiados para o destino, e as pastas temporárias são sempre removidas;
  - sem conversor, o botão do diálogo fica desabilitado com a explicação.

## Consequências

- Os arquivos abrem em qualquer leitor DXF. A espessura vem do nome e da configuração da camada, não de cada entidade (o R12 não tem *lineweight*).
- Os textos usam a fonte do programa que abre o arquivo, então a largura pode variar um pouco em relação ao PDF.
- O DWG depende de software de terceiros instalado pelo usuário; os testes usam conversores falsos (`.cmd`).
- A base fica pronta para blocos com atributos, se pedidos na Etapa 26 (EPLAN).
