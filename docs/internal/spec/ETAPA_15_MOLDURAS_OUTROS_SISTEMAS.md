# CAD-CRAFT — Etapa 15 (Fase A): molduras de outros sistemas

**Estado: aprovado pelo usuário em 2026-11-03 ("Aprovar Etapa 15") com as recomendações: `quick-xml` (MIT) autorizada; DXF com leitor/escritor próprio; exportar `.fn1` fora desta etapa; amostras `.fn1` versionadas como fixtures.**

Plano: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md`. Base: commit `7bacd28`
(Etapa 14), 480 testes Rust + 15 PowerShell. Modelo e editor de moldura:
ADR-0015 e ADR-0016.

## Objetivo

Trazer para o CAD-CRAFT molduras feitas em outros programas — EPLAN (`.fn1`),
QElectroTech (`.titleblock`) e qualquer CAD (DXF) — e exportar molduras para
DXF. Toda moldura importada vira uma moldura comum do projeto, editável no
editor da Etapa 14 e salva no `.cadcraft`/`.cadframe`.

## O que as amostras mostram

As 7 molduras do usuário em `<pasta de molduras do EPLAN>` (EPLAN 2027, PXF 1.23) são XML com objetos numerados. Leitura
preliminar de `FN1_001.fn1`:

| Objeto | Provável significado | Atributos observados |
|---|---|---|
| `O178` | Dados da moldura | `A539` larguras das colunas (`42;42;…`), `A1821` nome |
| `O196` | Texto fixo | `A1019` posição, `A1039` traduções (`pt_BR@Folha;en_US@Page;…`) |
| `O242` | Texto especial | `A1019` posição, `A1559` número da propriedade (ex.: 11074) |
| `O198` | Linha | `A1079`/`A1081` pontos, `A845` espessura |
| `O314` | Retângulo | `A3319`/`A3321` cantos |

O significado exato de cada código é deduzido comparando as amostras com o
que o EPLAN mostra (o usuário confirma pontos duvidosos). Não há
documentação pública; nenhum código do EPLAN é usado, só o conteúdo dos
arquivos do usuário.

## Escopo

1. **Importar `.fn1`** (EPLAN): borda, linhas, retângulos, círculos/arcos
   quando existirem, textos fixos com traduções, textos especiais, colunas e
   linhas (quantidade, larguras e rótulos), tamanho da folha. Tabela de
   mapeamento `número de propriedade EPLAN → SpecialField` versionada no
   código; propriedade sem equivalente vira texto especial "Propriedade do
   projeto/página" com a chave original (ex.: `EPLAN.11074`), sem se perder.
2. **Importar `.titleblock`** (QElectroTech): grade de linhas/colunas com
   células, textos fixos, variáveis `%{…}` convertidas em textos especiais
   (título, autor, data, folha, etc.) e logotipos embutidos (PNG/JPG; SVG
   fica de fora). O carimbo importado é posicionado no canto inferior
   direito de uma folha escolhida pelo usuário.
3. **Importar DXF** (ASCII, R12 a 2018): `LINE`, `LWPOLYLINE`/`POLYLINE`,
   `CIRCLE`, `ARC`, `TEXT`/`MTEXT` simples e blocos (`INSERT` expandido).
   Unidades pelo cabeçalho (`$INSUNITS`) ou escolhidas pelo usuário. Textos no
   formato `<<campo>>` ou `%{campo}` viram textos especiais; os demais, textos
   fixos. Folha detectada pela extensão do desenho ou informada.
4. **Exportar DXF** de uma moldura: geometria, textos (especiais como
   `<<campo>>`), em milímetros. Reimportar o DXF exportado reproduz a moldura.
5. **Fluxo na interface**: Dados-mestre > Molduras > "Importar de outro
   sistema…" (escolhe o arquivo, mostra uma prévia e um resumo: elementos
   lidos, itens ignorados e por quê, campos não mapeados) e "Exportar DXF".
   A importação é um passo único de Desfazer.
6. **Relatório de importação**: lista do que não foi convertido (entidades
   DXF não suportadas, objetos `.fn1` desconhecidos, imagens SVG), mostrado
   na prévia; nunca descartar em silêncio.
7. **Segurança**: limite de tamanho, XML sem entidades externas, recusa de
   arquivos inválidos sem alterar o projeto.

## Critérios de aceitação

1. As 7 molduras `.fn1` do usuário importam sem erro; para `FN1_001` (e uma
   segunda escolhida pelo usuário), colunas, borda, linhas do carimbo, textos
   fixos e especiais conferem com o EPLAN (posição ±0,5 mm, textos e número
   de colunas iguais). Cópias das amostras viram fixtures de teste versionadas
   (autorizado pelo usuário em 2026-11-03).
2. Os `.titleblock` de referência do QElectroTech usados como teste importam
   com textos e variáveis convertidos (usados só como entrada de teste, sem
   copiar código do QElectroTech).
3. DXF: importar arquivos de teste gerados nos testes (todas as entidades
   suportadas, blocos, unidades) e exportar → reimportar com igualdade.
4. Moldura importada: atribuir a uma página, textos especiais mostram os
   valores do projeto, referências por coluna funcionam, edição no editor e
   Desfazer/Refazer exatos.
5. Arquivos inválidos, grandes demais ou maliciosos (XML com entidade
   externa, DXF truncado) são recusados sem alterar o projeto.
6. Testes anteriores preservados; `scripts/validate.ps1`; Release; relatório,
   ADR e memória; commit + push após o OK. Teste no app: não nesta etapa
   (próximo ao fim da Etapa 16).

## Decisões pedidas ao usuário

- **Leitor XML**: dependência `quick-xml` (MIT) para `.fn1` e `.titleblock`,
  ou escrever um leitor próprio mínimo (mais código, mais risco).
- **DXF**: leitor/escritor próprio só das entidades listadas (sem dependência;
  recomendado) ou a biblioteca `dxf` (MIT), mais completa e maior.
- **Fixtures** — **autorizado pelo usuário em 2026-11-03**: copiar as
  molduras `.fn1` de `<pasta de molduras do EPLAN>`
  para `crates/cad-craft/tests/fixtures/` e versioná-las no GitHub.
- **Exportar `.fn1`** para o EPLAN: fica fora desta etapa (formato fechado,
  risco de gerar arquivo que o EPLAN rejeita) — confirmar.

## Fora do escopo

Exportar `.fn1` e `.titleblock`; DWG (Etapa 22/26); DXF binário; hachuras,
cotas e splines do DXF; imagens SVG; formulários de relatório do EPLAN
(Etapa 23).
