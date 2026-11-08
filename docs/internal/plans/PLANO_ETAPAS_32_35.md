# Plano — Etapas 32 a 35 (problemas achados no teste da partida estrela-triângulo)

**Estado: APROVADO pelo autor em 2026-11-07 ("Aprovo o plano"), com as três decisões recomendadas.** Prompts: `docs/internal/spec/ETAPA_32_CONEXOES_EDICAO.md`, `ETAPA_33_SIMBOLOS_FUNCOES.md`, `ETAPA_34_PECAS_FUNCOES.md`, `ETAPA_35_BORNES_COORDENADAS_UNIFILAR.md`.

Base: privado `5d43331`, público `fe6cee3` (Etapa 31). Antes da Etapa 32 é preciso versionar as duas correções do editor de moldura feitas depois da 31 (divisões desenhadas no editor e na prévia; rótulos de linha dentro da moldura), que estão validadas mas sem commit.

## Origem

O autor pediu um teste de capacidade: projeto completo de partida estrela-triângulo feito só pela interface (capa, unifilar, potência, comando, lista de materiais, tabela de fios), com as macros que faltassem salvas na biblioteca. O projeto ficou pronto (`Documentos\CAD-CRAFT\Projetos\Partida estrela-triângulo.cadcraft`, PDF de 10 páginas, XLSX com todas as listas; 8 símbolos, 10 peças e 10 macros na biblioteca). Os problemas abaixo apareceram durante o trabalho.

## Diagnóstico

| # | Problema visto | Causa encontrada no código |
|---|---|---|
| 1 | "Inserir junção" num ponto em T falha com "desenho de conexão inconsistente ou referenciado" | `point_on_segment` (modelo) compara coordenadas com `==`. As rotas que saem de portas usam a posição calculada pela transformação do símbolo (f64), e a junção usa o ponto da grade; uma diferença mínima faz `routes_at`/`contains` acharem menos de duas rotas |
| 2 | Primeiro clique numa porta, logo depois de escolher "Inserir conexão", às vezes falha ("inicie em uma porta elétrica associada"); um canto pode ser perdido e a rota sai em L errado, passando por cima de outro fio | O ponto de encaixe vem da cena de conexões e do quadro anterior; sem um quadro de hover, o clique não acha a porta ou o canto |
| 3 | Janelas nativas (salvar, confirmar substituição) abriam atrás do app e travavam a tela sem aviso | O diálogo do sistema é aberto sem janela-mãe |
| 4 | Ao editar uma propriedade de página (Enter), o inspetor pula para a página da aba ativa e a edição seguinte vai para a página errada; X/Y de um símbolo só valem depois de "Aplicar transformação", escondido no fim do painel | Inspetor ligado à aba ativa depois de cada edição; transformação só aplicada pelo botão |
| 5 | Símbolos de fábrica (bobina, contatos NA/NF, botão) não mostram a tag (-K1) e usam portas "1/2"; o contato 95-96 de F1 aparece na lista como "F1:1", igual ao polo principal; o desenho do contato NA lembra um capacitor | Os símbolos de fábrica em `symbol_library.rs` não têm texto dinâmico de designação e têm portas fixas "1"/"2" |
| 6 | Símbolos do usuário entram com prefixo "D" e não casam com as peças | `default_prefix` e `function_kind` (`gui/src/state/mod.rs`) só conhecem os IDs de fábrica; os demais viram `FunctionKind::Other(nome do símbolo)` |
| 7 | Atribuir peça duplica funções (K1 com duas bobinas; 23 avisos VER-006) | `assign_part` só reaproveita função de mesmo `kind` e com pontos contidos nos da peça; as funções criadas pelos símbolos têm outro `kind` e pontos "1/2" |
| 8 | Lista de funções existentes mostra "Função 01a1b832-…" | `panels/insertion.rs` usa o ID no rótulo |
| 9 | Bornes inseridos pelo símbolo "Borne" não aparecem na lista de bornes | O símbolo cria um dispositivo X1, X2…; a lista usa bornes de régua |
| 10 | Editor de símbolos: categoria digitada não foi gravada num símbolo (salvo como "Usuário"); identificação de uma porta não mudou na primeira tentativa; o zoom muda a cada símbolo novo e não há leitura de X/Y do cursor | Campos de texto que perdem a edição quando o foco muda; câmera reiniciada por símbolo |
| 11 | "Inserção de coordenadas" existe só como item planejado | Grupo "Opções" do Editar ainda sem implementação |
| 12 | Unifilar: não há conexão própria; usei linhas gráficas para não mexer nas redes do multifilar | Toda conexão entre portas vira conexão elétrica |

## Etapas propostas

### Etapa 32 — Conexões e edição confiáveis · média
- Junção em T: comparação com a tolerância do domínio (`EPS_MM`) em `point_on_segment`/`contains`, e o ponto da junção encaixado na rota existente. Teste com símbolos girados e espelhados.
- Clique em porta ou canto válido no mesmo quadro em que a ferramenta é escolhida (cena sincronizada antes do encaixe). Teste de interface: escolher a ferramenta e clicar sem quadro de hover.
- Diálogos nativos com a janela do CAD-CRAFT como mãe, sempre à frente; enquanto um estiver aberto, a área de trabalho mostra "Aguardando o diálogo do sistema".
- Inspetor fica na página selecionada durante a edição; Enter em X, Y ou ângulo aplica a transformação (o botão continua).
- Editor de símbolos: categoria e identificação de porta gravadas também quando o foco muda.

### Etapa 33 — Símbolos de fábrica, tipo de função e nomes de pontos · média
- Símbolos de fábrica redesenhados no estilo IEC (contato NA com lâmina, NF com gancho, botão, bobina), com texto de designação e portas com nomes normais (A1-A2, 13-14, 21-22). Nova versão dos símbolos de fábrica; projetos existentes continuam com a versão antiga até "Atualizar símbolos do projeto".
- Editor de símbolos: "Tipo de função" (bobina, contato principal, contato auxiliar, outro) e "Prefixo" (Q, K, F, M, S, X…) gravados no símbolo; leitura de X/Y do cursor; campos numéricos para os pontos das primitivas; zoom estável entre símbolos.
- Função em dispositivo existente: sugere os próximos pontos livres conforme o tipo (13-14, 23-24, 21-22, 95-96, 15-16/15-18) e permite editar antes de inserir. Verificação nova: designação de ponto repetida no mesmo dispositivo.
- Seletor de funções com rótulo legível: "-K1 · Bobina (A1, A2)".

### Etapa 34 — Peças e funções sem duplicação · pequena
- `assign_part` casa as funções da peça com as do dispositivo pelo tipo e pelos pontos (agora com os nomes certos); quando não casam, mostra o mapeamento proposto (prévia) em vez de criar funções novas em silêncio.
- Lista de dispositivos e VER-006 sem as duplicatas; teste com o caso K1 (contatos principais + bobina + auxiliares).

### Etapa 35 — Bornes, coordenadas e unifilar · média
- Símbolo "Borne" cria (ou usa) borne de régua: X1:1, X1:2… com escolha da régua na inserção; lista de bornes preenchida.
- "Inserção de coordenadas": X/Y digitados para posicionar símbolo, cantos de conexão e linhas gráficas.
- Conexão unifilar: ferramenta própria, com marca de número de fases, que liga representações sem criar conexões no multifilar.

## Decisões para o autor (recomendação primeiro)

1. Símbolos de fábrica novos: **nova versão só para projetos novos e para quem usar "Atualizar símbolos do projeto"** (recomendado) ou trocar em todos os projetos ao abrir.
2. Numeração padrão dos pontos: **IEC 60947 (A1-A2, 13-14, 21-22, 95-96, 15-16/18)** (recomendado) ou numeração livre.
3. Unifilar: **conexão unifilar própria, sem efeito nas redes** (recomendado) ou continuar com linhas gráficas.

## Ordem, testes e entrega
- Ordem: versionar as correções da moldura → 32 → 33 → 34 → 35. Cada etapa com prompt aprovado, `validate.ps1`, Release, relatório e memória; commit + push só com o OK de cada uma.
- Saídas de referência: a 33 muda o desenho dos símbolos de fábrica; as saídas serão regravadas e explicadas.
- Teste no app ao fim da 35: refazer a partida estrela-triângulo usando as macros da biblioteca, conferindo junções em T, tags, nomes de pontos, lista de dispositivos sem duplicatas e lista de bornes.
