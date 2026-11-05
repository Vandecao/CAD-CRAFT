# CAD-CRAFT — Etapa 14 (Fase A): editor de molduras

**Estado: aprovado pelo usuário em 2026-11-03 ("Aprovar Etapa 14") com as decisões propostas: dependência `image` (PNG/JPEG) autorizada; limites de 10 MB e 4096 × 4096 px por imagem; QR e hiperlink ficam para etapa futura.**

Plano: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md`. Base: commit `98d57af`
(Etapa 13), 466 testes Rust + 15 PowerShell. Modelo de moldura: ADR-0015.

## Objetivo

Criar molduras do zero e editar as existentes dentro do CAD-CRAFT, com
logotipos, e trocar molduras entre projetos por um arquivo próprio — o
equivalente ao editor de molduras do EPLAN (Dados-mestre → Moldura). A
importação de DXF, `.titleblock` e `.fn1` continua na Etapa 15.

## Escopo

1. **Modo de edição de moldura**: abrir uma moldura do projeto numa aba
   própria ("Moldura: nome"), com a folha no tamanho da moldura, grade, snap,
   zoom/pan e o desenho atual da Etapa 13. Comandos: Nova moldura (em branco
   ou a partir de outra), Abrir, Fechar, Duplicar, Excluir (só sem uso).
2. **Ferramentas de desenho**: selecionar, linha, retângulo, círculo, arco,
   texto fixo, texto especial e imagem; mover, copiar, excluir e alinhar à
   grade os elementos selecionados (inclusive seleção múltipla).
3. **Propriedades do elemento** no painel: coordenadas, tamanhos, espessura,
   texto padrão e traduções, campo especial e índice, altura, alinhamento
   (9 pontos) e rotação; para imagens: posição, tamanho e manter proporção.
4. **Propriedades da moldura**: nome, descrição, tamanho da folha (A0–A4 ou
   personalizado), área útil, colunas e linhas (quantidade, largura/altura
   uniforme ou individual, estilo de rótulo 1/A, início, lados).
5. **Imagens/logotipos**: inserir PNG e JPG; a imagem fica guardada no
   projeto (`assets/` do `.cadcraft`, sem duplicar imagens iguais) e é
   desenhada na folha e no editor.
6. **Marca d'água**: texto fixo ou especial marcado como marca d'água
   (desenhado atrás do conteúdo, cor clara), como o EPLAN oferece.
7. **Biblioteca e intercâmbio**: exportar uma moldura para arquivo próprio
   `.cadframe` (ZIP com manifesto versionado, `frame.json` e as imagens) e
   importar de volta em qualquer projeto (novo ID se já existir, sem perder
   imagens). Pasta de biblioteca configurável.
8. **Histórico**: todas as edições do editor com Desfazer/Refazer; salvar a
   moldura incrementa a versão; páginas que a usam passam a desenhar a nova
   versão.
9. **Persistência**: `.cadcraft` formato 3 se o conteúdo do projeto mudar
   (imagens e marca d'água), com migração do formato 2 e fixtures.

## Critérios de aceitação

1. Com eventos egui reais: criar moldura A3 do zero, desenhar borda, linhas do
   carimbo, um texto fixo, três textos especiais (projeto, página, data) e
   um logotipo PNG; atribuir a uma página e ver tudo desenhado com os valores
   do projeto.
2. Editar elementos de uma moldura em uso (mover linha, trocar campo
   especial, mudar zonas) atualiza as páginas e as referências por coluna.
3. Desfazer/Refazer de cada tipo de edição restaura exatamente.
4. Exportar `.cadframe` e importar em outro projeto reproduz a moldura e as
   imagens (igualdade do modelo, imagens idênticas byte a byte).
5. Arquivos/imagens inválidos (PNG corrompido, tamanho excessivo, ZIP
   inseguro) são recusados sem alterar o projeto.
6. Salvar/reabrir preserva molduras, imagens e marcas d'água; `.cadcraft` da
   Etapa 13 abre e migra; fixtures versionadas.
7. Testes anteriores preservados; `scripts/validate.ps1`; Release; relatório,
   ADR e memória; validação funcional na interface; commit + push após o OK.

## Decisões pedidas ao usuário

- **Dependência `image`** (MIT/Apache) em produção, com PNG e JPEG, para ler e
  validar logotipos (hoje só existe como dependência de teste).
- **Limites de imagem** propostos: até 10 MB por arquivo e 4096 × 4096 px.
- **QR e hiperlink** (existem no EPLAN): ficar para depois ou entrar aqui.

## Fora do escopo

DXF, `.titleblock` e `.fn1` (Etapa 15); formulários de relatório (Etapa 23);
outros formatos de imagem (SVG, BMP) e edição de imagem.
