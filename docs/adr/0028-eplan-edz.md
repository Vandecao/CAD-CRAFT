# ADR-0028 — Importação de pacotes EPLAN EDZ

Data: 2026-11-04. Estado: implementado e validado automaticamente (666 Rust + 15 PowerShell; script oficial exit 0 e Release compilado). Homologação manual pendente.

## Contexto e decisões aprovadas

A especificação `docs/internal/spec/ETAPA_26_EPLAN_EDZ.md` aprovou somente EDZ, `sevenz-rust2` exata, peças em `.cadpart`, macros 2D em `.cadmacro` e anexos/3D somente no relatório. O usuário aprovou durante a implementação concluir com desenhos inline e fallback: nenhuma `ESymLib` inspecionada continha definições. A leitura dessas definições depende de amostra demonstrativa; não se inventa uma gramática para aparentar cobertura.

## Decisão

O Core (`interchange/edz/`) recebe bytes e retorna peças, macros, contêiner, relatório e conjunto de códigos desconhecidos, sem gravar arquivos. A GUI analisa arquivos/pasta em worker, mostra prévia por pacote e só confirma em bibliotecas configuradas. Código e dados proprietários não são incorporados. O formato `.cadcraft` permanece 9.

### Contêiner e segurança

Assinatura determina ZIP/7z; extensão serve ao seletor da GUI. Dependência de produção `sevenz-rust2 =0.24.2`, Apache-2.0, sem recursos padrão. Encoder ativado somente em testes sintéticos. Nada é extraído para disco.

Limites padrão: entrada 128 MiB, 10.000 itens, 64 MiB/item, 256 MiB descomprimidos. Caminhos absolutos, drives, UNC, componentes `..`/`.`, controles, links e duplicatas normalizadas são recusados. Catálogo e leituras são limitados; checksums/truncamento viram erro.

Antes de construir o leitor 7z, `sevenz_guard` verifica CRC e limites do cabeçalho. Metadados limitados a 4 MiB e dicionários a 64 MiB (ou total menor). Cabeçalho codificado aceita o subconjunto comprovado Copy/LZMA/LZMA2, com um bloco e um coder; estruturas não suportadas são recusadas. Após o catálogo, os coders, dicionários, tamanhos intermediários e blocos são conferidos antes de descomprimir itens. Não há promessa de aceitar todos os codecs 7z.

Manifesto 2.0 precisa de referências seguras. Dados necessários ausentes impedem a conversão do pacote. Referências vazias e anexos/3D ausentes são diagnosticados; não se tornam entidades. XML usa o leitor defensivo existente.

### Peças

`P_ARTICLE_PARTNR`, fabricante (resolvido por `P_PART_ADDRESS_SHORTNAME`), descrição pt-BR/en-US e grupo/tipo mapeiam os campos de `Part`. Atributos e filhos (variantes/userproperty incluídos) são preservados em propriedades técnicas com chave original ou caminho hierárquico. Pontos de `terminalPosition` mantêm nomes e coordenadas originais nas propriedades técnicas; não viram terminais físicos instalados.

`functiontemplate.connectionDesignation` gera pontos. Tipo preserva a tripla numérica `EPLAN.categoria/grupo/id`, sem inferir bobina/contato; papel só é atribuído quando `mainfunction` é explícito. Função sem pontos/inválida/repetida é relatada e preservada nos atributos. Rótulos com controles são normalizados para o contrato do domínio; valores originais são preservados. IDs incompatíveis recebem chave estável derivada, com aviso.

O manifesto liga a peça às macros convertidas por `CADCRAFT.MacroIds`; atributos EPLAN originais de macro continuam nas propriedades. Manter ambas na GUI atualiza o vínculo para os novos IDs.

### Tabela PXF

Mapeamento independente observado nos arquivos do usuário, sem consulta/cópia de código EPLAN. Coordenadas em mm cartesianas, sem inversão de Y. A identidade do objeto vem de `A1="tipo/id"` (ou `A19` no esquema anterior), não do sufixo do nome `O`.

| Código/campo | Conversão e limites |
|---|---|
| `MacroVariant`, `MacroFuncType`, `ReferencePoint` | Primeira variante 2D, preferindo tipo 1; base x/y. Demais variantes relatadas |
| `ESymLib.Name/ID` | Referência; definições não vazias são relatadas como esquema ainda desconhecido |
| Tipos `4`, `26` | Contêiner/grupo percorrido |
| Tipo `17` | Ocorrência com gráficos inline; definição local própria e transformação |
| `P11.P19008` | Nome do símbolo; outras propriedades sem mapeamento são relatadas |
| `S40x1201.A762` | Translação x/y |
| `S39x761.A751..A754` | Matriz 2×2: rotação/reflexão; escala/cisalhamento são aproximação relatada |
| `S61x183.A1061`, `O17.A185` | Porta única na origem conhecida e identificação; posições de múltiplas portas não são inferidas |
| Tipo `31`, `A531/A532` | Linha; compatibilidade com `A1079/A1081` do leitor FN1 |
| Tipo `34`, `A621/A623` | Polilinha/fechamento, origem implícita em campo vazio; compatibilidade `A1259/A1263`. Bulges não nulos antigos são aproximados/relatados |
| Tipo `89`, `A1651/A1652` | Retângulo por cantos; compatibilidade `A3319/A3321` |
| Tipo `30`, `A501/A511`, `S54.A961` | Texto localizado, posição/altura; compatibilidade `A1019/A1039/A1939` |
| Tipo `32`, `A553/A554/A555/A556/A557` | Início/fim em radianos, centro, vetor do eixo e razão: círculo/arco circular ou elipse completa; arco elíptico parcial é relatado |
| Tipos `6`, `14`, `37`, `76` | Metadados/configuração relatados; caixa de macro não vira entidade elétrica |
| Demais códigos `O/P/S` | Lista de não convertidos; nunca inferidos como fios/portas |

A leitura de `A553..A557` é inferida da consistência matemática dos valores observados (volta `2π`, eixo e razão), sem alegação de documentação oficial. Testes sintéticos independentes exercitam as formas. Símbolo sem primitiva suportada vira caixa com nome e portas conhecidas. Caracteres incompatíveis na identificação de porta são normalizados, com original no desenho e aviso.

## Consequências e limitações

- A macro resultante é validada pelas APIs existentes, insere IDs novos e participa do Undo/Redo existente.
- Não existe mapeamento comprovado de ligações elétricas PXF nem posições de múltiplas portas. Esses objetos são relatados; traços não criam conectividade por inferência. Estilos, textos dinâmicos e variantes adicionais têm perdas explicitadas.
- Imagens, documentos e 3D permanecem fora do escopo. Listas/AutomationML/DXF/DWG EPLAN ficam para 26b.
- Cancelamento da análise descarta o pacote em andamento; cancelamento na gravação preserva e informa os arquivos já concluídos. Serialização é conferida antes da primeira escrita; arquivos individuais usam gravação atômica. O lote não é uma transação do sistema de arquivos.
- Não houve execução do app nem Git mutável. Teste no app das Etapas 23–26 continua reservado ao Claude Code, mediante autorização; versionamento depende do OK posterior do usuário.

## Adendo — teste no app (2026-11-05)

- **Defeito:** duas das três macros importadas no teste não eram relidas da biblioteca.
- **Causa:** o leitor padrão do `serde_json` pode errar o último bit de números longos, e o Core compara exatamente os limites da ocorrência com os da definição.
- **Decisão:** ligar o recurso `float_roundtrip` do `serde_json` no Core. Não é dependência nova. Vale para toda a persistência JSON (`.cadcraft`, `.cadmacro`, `.cadpart`, `.cadsym`, `.cadframe`): todo `f64` gravado volta igual.
- **Regressões:**
  - `persistence.rs::json_floats_round_trip_bit_exactly`;
  - o teste opcional das amostras `.edz` grava e relê cada peça e macro.
