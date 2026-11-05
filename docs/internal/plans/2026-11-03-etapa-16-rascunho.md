# Plano preliminar — Etapa 16: conteúdo livre e clipboard

**Estado:** plano de execução do prompt aprovado pelo usuário em 2026-11-03.

## Base e limites

- Plano Mestre aprovado: Fase B, Etapa 16 = texto livre, gráficos na página,
  copiar/recortar/colar. Não avançar para a Etapa 17.
- Etapa 15 permanece fora do HEAD por instrução do usuário. Não misturar
  versionamento da Etapa 16 com o da 15; não executar Git mutável.
- Core `cad-craft` sem dependência de GUI. Coordenadas em milímetros, origem
  inferior esquerda. IDs tipados v7, deltas reversíveis e persistência
  versionada seguem os contratos atuais.

## Mapa de implementação proposto

1. **Modelo e integridade:** novo módulo de conteúdo de página em
   `crates/cad-craft/src/domain/`, registro no `Project`, valores validados,
   relação com página e testes Core. Conferir `domain/representation.rs` e
   `domain/project/representations.rs` antes de fixar a API.
2. **Edição transacional:** edições/deltas para inserir, atualizar, mover,
   excluir e lote atômico, integrados ao histórico de `cad-gui`. Testar
   falha tardia, replay e Desfazer/Refazer exatos.
3. **Persistência:** elevar `.cadcraft` a formato 4; migração de v3 e
   fixtures v0–v3, incluindo entradas inválidas. Clipboard transitório.
4. **Desenho e interação:** render, hit test, prévia, seleção e propriedades
   de texto/linha/retângulo/círculo/arco na GUI. Testar eventos egui em
   duas escalas e DPI, sem alterar seleção/gestos existentes.
5. **Clipboard:** comandos e atalhos, seleção múltipla, colagem na mesma
   página e entre páginas, IDs/associações conforme prompt aprovado,
   recorte/cancelamento, atomicidade e histórico.
6. **Porta de saída:** `scripts/validate.ps1`, build Release, teste funcional
   no executável conforme cadência, relatório, memória e ADR pertinentes.

## Questões de revisão

- Política de copiar ocorrências elétricas e exclusão de rotas/junções.
- Se o conjunto básico de primitivas corresponde ao esperado para esta etapa.
- Se o clipboard deve ser somente interno à sessão.

## Interfaces e verificação

- `Representation` recebe conteúdo livre validado e mantém a identidade tipada
  `RepresentationId`. Conteúdo sem alvo elétrico é aceito apenas quando livre;
  conteúdo livre não pode ter símbolo nem alvo elétrico. O mesmo índice de
  página, seleção e transformação é compartilhado.
- Uma edição de lote no Core valida todos os estados anterior/posterior antes
  da escrita e devolve delta reversível. A GUI registra uma transação por
  inserção, edição, exclusão, cópia ou movimento entre páginas.
- Render e hit test usam a mesma primitiva geométrica. A inserção de dois
  pontos conserva apenas prévia até o clique final; texto usa ponto de âncora.
- O clipboard interno guarda snapshots da seleção. A primeira colagem de um
  recorte substitui origem/destino atomicamente; cópias posteriores geram IDs
  novos. Rejeições mantêm seleção, clipboard e histórico.
- Testes Core cobrem validação, lote, reversão e serialização; testes GUI
  cobrem seleção, clipboard, prévia, comandos e coordenadas. Rodar validação,
  Release e teste funcional no app; registrar evidências no relatório.

Commit e push exigem autorização expressa posterior para a etapa concluída.

## Andamento (2026-11-03)

- [x] Core conteúdo livre e formato 4 (Codex) · [x] área de transferência
  revisada (dispositivo novo, recorte imediato) · [x] Ctrl+D, colar repetido,
  duplo clique · [x] fixture v4 e testes de interface · [x] validação e
  Release · [x] documentação · [x] teste no app (2026-11-03)
