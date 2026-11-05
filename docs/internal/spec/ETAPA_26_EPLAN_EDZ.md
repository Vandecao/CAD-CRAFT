# CAD-CRAFT — Etapa 26 (Fase D): importação de pacotes `.edz` de fabricantes (EPLAN)

**Estado: APROVADO pelo usuário em 2026-11-04 ("Aprovo todas"), com as
decisões recomendadas 2 a 5:**
- 2: só `.edz`;
- 3: crate `sevenz-rust2` com versão exata;
- 4: peças para a biblioteca `.cadpart`;
- 5: imagens, documentos e macros 3D só no relatório.

**Ajuste de implementação aprovado pelo usuário em 2026-11-04:** as amostras
atuais têm somente referências em ESymLib, sem definições embutidas. A entrega
converte desenhos 2D inline e usa caixa com portas conhecidas quando faltar a
definição. Leitura de definições ESymLib fica pendente de uma amostra que
demonstre o formato; nenhum esquema é inventado para o teste sintético.

A decisão 1 (ordem) está resolvida: a 26 vem depois da 25. Implementação:
Codex.

Plano Mestre: `docs/internal/plans/PLANO_MESTRE_2026-11-03.md`, Fase D, Etapa 26
("EPLAN 2027 (listas, XML/AutomationML, DXF/DWG, `.edz`)", depende das
amostras do usuário). Base: `main` em `0b3b65c` (Etapa 22) **mais as Etapas 23, 24 e 25 prontas,
validadas e ainda não versionadas** (arquivos modificados e novos no working
tree; ver `HANDOFF.md`). O Codex trabalha sobre esse working tree sem
descartar nada. Formato `.cadcraft` atual: 9. Próximo ADR livre: 0028.
Validação de partida: 640 Rust + 15 PowerShell (`scripts/validate.ps1`, exit 0).

## Objetivo

Importar no CAD-CRAFT os pacotes `.edz` que o usuário baixa dos portais dos
fabricantes (EPLAN Data Portal), no estado em que vêm, como faz o EPLAN. A
importação deve:
- abrir o pacote;
- transformar os dados da peça em peça do CAD-CRAFT (`.cadpart`, Etapa 18);
- transformar as macros 2D em macros do CAD-CRAFT (`.cadmacro`, Etapa 19),
  com os símbolos embutidos;
- relatar com clareza o que não puder ser convertido.

Nenhum código ou dado do EPLAN é copiado; só os arquivos do próprio usuário
são lidos.

## Amostras reais (fora do repositório)

`<pasta de amostras>\eplan\edz`: 47 pacotes, cerca de 116 MB, baixados pelo
usuário (Allen-Bradley 1734, ABB ACS550, Banner, Rittal, Omron, Numatics,
cabos etc.). São dados dos fabricantes, para uso local. **Nunca entram no
Git** (tamanho e direitos): servem só para o teste manual e para um teste
automático opcional que lê a pasta por variável de ambiente.

Achados na análise feita apenas para leitura (2026-11-04):
- **Contêiner:** a extensão é sempre `.edz`, mas por dentro 44 pacotes são 7z
  (assinatura `37 7A BC AF 27 1C`) e 3 são ZIP (`50 4B 03 04`: `BAN.85064`,
  `BAN.85069`, `BAN.85070`). O importador detecta pela assinatura, nunca pela
  extensão.
- **Estrutura** (exemplo `BAN.85069`):
  - `manifest.xml` (`<manifest version="2.0">`): `<package type="part" key=…>`
    e `<item type="partxml|macro|picture|document" name=… locator=…>`, com os
    caminhos relativos a `items/<tipo>/`;
  - `items/partxml/<peça>.part.xml`: `<partsmanagement>` →
    `<part P_ARTICLE_PARTNR=… P_ARTICLE_…>` com `<functiontemplate>`
    (funções da peça), `<variant>` e `<userproperty>`;
  - `items/partxml/<peça>.connectionpoints.xml`: `<terminal …>` e
    `<terminalPosition>` (pontos de conexão);
  - `items/partxml/<fabricante>.manufacturer.xml`: dados do fabricante;
  - `items/macro/<fabricante>/<peça>.ema`: XML PXF
    (`<EplanPxfRoot Type="WindowMacro" PxfVersion="1.23">`), com objetos
    codificados (`O17`, `O6`…), propriedades (`P11`, `P150`…), primitivas
    (`S54x505`, `S53x5`…) e **biblioteca de símbolos embutida (`ESymLib`)**;
  - `…_(3D).ema`: macro 3D, fora do escopo;
  - `items/picture`, `items/document`: imagens e documentos (PNG, JPEG).

## O que já existe (não refazer)

- Leitor PXF de molduras `.fn1` do EPLAN (`interchange/fn1.rs`, Etapa 15) e
  infraestrutura XML (`interchange/xml.rs`, `quick-xml`), com relatório de
  importação (`ImportReport`).
- Peças: `Part`, `PartFunction`, `.cadpart`, importação CSV, biblioteca de
  peças e editor (Etapa 18).
- Macros: `MacroDefinition`, `MacroContent`, `.cadmacro`, biblioteca de
  macros e inserção com IDs novos (Etapa 19).
- Símbolos: `SymbolDefinition`, primitivas e estilos, `.cadsym`, importação
  `.elmt` com relatório (Etapa 17).
- Leitura de ZIP (`zip`, já no projeto). Ainda não há leitura de 7z.

## Escopo

1. **Abrir o `.edz` (Core)**:
   - detectar 7z ou ZIP pela assinatura;
   - listar e extrair em memória, com limites (tamanho total, número de
     itens, tamanho por item, caminhos com `..` ou absolutos recusados);
   - ler o `manifest.xml`.
   - Pacote inválido ou truncado: erro claro, nada é gravado.
2. **Peça** (decisão 4): `part.xml` + `connectionpoints.xml` +
   `manufacturer.xml` → `Part`.
   - Mapeia: código (`P_ARTICLE_PARTNR`), fabricante, descrição (texto
     multilíngue, preferindo pt-BR, depois en-US), tipo de produto,
     funções (`functiontemplate` → `PartFunction`, com tipo, papel e
     pontos) e propriedades técnicas úteis (as `P_ARTICLE_*` reconhecidas).
   - O que não tiver correspondência fica nas propriedades técnicas com a
     chave original e é contado no relatório.
   - A peça é gravada na biblioteca de peças configurada (`.cadpart`).
3. **Macros 2D** (decisões 2 e 5):
   - cada `.ema` 2D vira uma `MacroDefinition`;
   - os símbolos da `ESymLib` embutida viram `SymbolDefinition`, com as
     primitivas suportadas (linha, polilinha, retângulo, círculo, arco,
     elipse, texto) e as portas (pontos de conexão);
   - os objetos da macro (ocorrências de símbolo, ligações, textos) viram o
     `MacroContent`;
   - símbolo referenciado sem definição embutida → caixa com o nome do
     símbolo e portas na posição conhecida, relatado;
   - a macro é gravada na biblioteca de macros (`.cadmacro`) e fica ligada à
     peça quando o manifesto indicar.
   - O mapeamento dos códigos PXF (`O`/`P`/`S`) deve ser documentado numa
     tabela no ADR da etapa. Nada é inventado: código desconhecido é relatado, não
     adivinhado.
4. **Fora do pacote**: macros 3D, documentos e imagens não são importados;
   cada item ignorado aparece no relatório (decisão 5).
5. **GUI**:
   - Dados-mestre > Importar `.edz`: um ou mais arquivos pelo diálogo
     nativo, ou uma pasta inteira (lote);
   - prévia por pacote (peça, macros, símbolos, avisos);
   - confirmar grava na biblioteca;
   - relatório final por pacote (importado, convertido parcialmente,
     ignorado, erro);
   - a importação roda em segundo plano, com progresso e cancelamento (modelo
     `cad-gui/src/pdf_export.rs`);
   - nome já existente na biblioteca: pergunta (substituir, manter as duas,
     pular).
6. **Relatório de cobertura**: comando de linha de comando ou teste opcional
   que percorre uma pasta de amostras e gera uma tabela (pacote, contêiner,
   peças, macros, símbolos convertidos, itens não suportados), gravada em
   `docs/internal/reports/ETAPA_26_COBERTURA_EDZ.md` **sem copiar conteúdo dos
   fabricantes**: só contagens e códigos PXF não reconhecidos.

## Critérios de aceitação

1. Pacotes sintéticos criados pelos próprios testes (ZIP e 7z, com
   manifesto, `part.xml`, `connectionpoints.xml` e uma `.ema` mínima com
   `ESymLib`) são importados: a peça tem código, fabricante, funções e
   pontos certos; a macro insere numa página e os símbolos têm as portas
   nas posições esperadas.
2. Detecção pela assinatura: o mesmo conteúdo em ZIP ou 7z dá o mesmo
   resultado; arquivo que não é ZIP nem 7z é recusado com mensagem clara.
3. Segurança: caminho com `..` ou absoluto, item acima do limite, pacote
   truncado, XML malformado e manifesto apontando item inexistente são
   recusados ou relatados, sem gravar nada e sem pânico.
4. Símbolo sem definição embutida vira caixa com portas e é relatado; código
   PXF desconhecido é relatado (lista no relatório), nunca descartado em
   silêncio.
5. Macro 3D, documentos e imagens aparecem como "ignorados" no relatório.
6. Com eventos egui reais: importar um `.edz` sintético pelo diálogo, ver a
   prévia, confirmar, achar a peça na biblioteca de peças e a macro no
   navegador de macros; inserir a macro numa página; Ctrl+Z remove a
   inserção.
7. Teste opcional com as amostras reais: com `CAD_CRAFT_EDZ_SAMPLES`
   apontando para a pasta, todos os 47 pacotes abrem sem pânico. O
   relatório de cobertura é gerado. Sem a variável, o teste é ignorado
   (passa sem rodar).
8. Testes anteriores preservados; `scripts/validate.ps1`; Release compilado;
   relatório `docs/internal/reports/ETAPA_26_EPLAN_EDZ.md`, o próximo ADR livre (consultar `DECISIONS.md`) com a tabela de
   códigos PXF; memória atualizada.
9. Sem teste no executável pelo Codex. Ao fim da 26, o Claude Code faz o
   teste no app das Etapas 23 a 26 (com a autorização do usuário para usar
   o computador); passando, segue o commit + push com o OK do usuário.

## Decisões pedidas ao usuário

1. **Ordem das etapas** — resolvida (2026-11-04): a 26 é implementada depois
   da 25, mantendo o cronograma.
2. **Escopo da 26**:
   - **Recomendado:** só `.edz` (peças e macros 2D), porque é o que há de
     amostra.
   - Listas, XML/AutomationML e importação de DXF/DWG do EPLAN ficam para
     uma etapa 26b, quando houver amostras.
3. **Leitura de 7z**:
   - **Recomendado:** o crate `sevenz-rust2` (Rust puro, Apache-2.0), com
     versão exata no `Cargo.toml`.
   - Alternativa: chamar o 7-Zip instalado pelo usuário, como o conversor
     DWG da Etapa 22. Funciona, mas exige o programa instalado.
4. **Peças**:
   - **Recomendado:** importar para a biblioteca de peças (`.cadpart`), com
     as funções e pontos.
   - Alternativa: importar direto no projeto aberto. Fica mais acoplado e é
     menos reutilizável.
5. **Itens extras**:
   - **Recomendado:** não importar imagens, documentos nem macros 3D nesta
     etapa; só relatar.
   - Alternativa: guardar a imagem da peça como anexo. O modelo `Part` ainda
     não tem esse campo, então seria preciso mudá-lo.

## Fora do escopo

Macros 3D e montagem de painel; documentos e imagens dos fabricantes;
exportação para `.edz`; listas, AutomationML e DXF/DWG do EPLAN (26b);
atualização automática pelo portal; download de qualquer coisa da internet.

## Instruções para o agente (Codex)

1. Ler `AGENTS.md`, este prompt, `docs/internal/memory/HANDOFF.md` e
   `docs/internal/memory/CURRENT_STATE.md`; consultar o ai-memory do projeto
   (`notes/etapa-24-verificacoes.md`, `notes/etapa-25-importacao-qet.md`
   e, se existir, `notes/etapa-26*`). ADRs úteis: 0017 (molduras de outros sistemas,
   `.fn1`), 0019 (símbolos), 0020 (peças), 0021 (macros) e 0027 (importação
   `.qet`, modelo de importação com prévia e relatório).
2. Git só leitura (`git status -sb`, `git log -3 --oneline`). Base: o HEAD
   indicado no `HANDOFF.md`. O `docs.rar` da raiz é do usuário e nunca entra
   em commit; as amostras `.edz` também não.
3. Só começar com a aprovação registrada no topo deste arquivo (decisões 2
   a 5). Dúvida fora das decisões: parar e perguntar.
4. Criar `docs/internal/plans/<data>-etapa-26.md` com o desenho técnico, a seção
   "Retomada por outro agente" e a lista "Andamento" numerada; marcar cada
   item só com teste passando. Gravar no ai-memory a página fixada
   `notes/etapa-26-eplan-edz.md` e atualizá-la ao final.
5. Padrões:
   - o Core não depende de GUI; leitura do pacote e conversão em
     `cad-craft/src/interchange/edz/`, reaproveitando `xml.rs` e `fn1.rs`;
   - a leitura de arquivos não confiáveis tem limites de tamanho e de
     quantidade, recusa caminhos perigosos e nunca extrai para o disco;
   - nenhuma dependência nova além da aprovada na decisão 3, com versão
     exata;
   - toda conversão parcial é relatada; nada é inventado para preencher
     lacunas;
   - testes com pacotes sintéticos gerados no próprio teste; as amostras
     reais só no teste opcional por variável de ambiente;
   - testes egui com accesskit (ver `cad-gui/tests/cad_export_ui.rs` e
     `cad-gui/tests/qet_import_ui.rs`); não remover nem afrouxar testes;
   - a análise em segundo plano com prévia pode seguir o modelo de
     `cad-gui/src/qet_import_ui.rs` (Etapa 25).
6. Validar com `pwsh -NoProfile -File scripts/validate.ps1`; compilar o
   Release sem executar o app. Rodar o teste opcional com
   `CAD_CRAFT_EDZ_SAMPLES=<pasta de amostras>\eplan\edz` e anexar a
   tabela de cobertura ao relatório.
7. Atualizar relatório, o ADR da etapa (próximo número livre), `DECISIONS.md`, `ROADMAP.md` (linha 26),
   `KNOWN_ISSUES.md`, `CURRENT_STATE.md` e `HANDOFF.md`; informar que está
   pronto. **Não executar nenhuma operação Git que altere o repositório**
   (nem `git add`): o Claude Code faz o teste no app das Etapas 23 a 26 e
   o versionamento acontece depois, com o OK do usuário. Não executar o app.
   Não iniciar outra etapa.
