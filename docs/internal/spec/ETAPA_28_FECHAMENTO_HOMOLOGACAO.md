# CAD-CRAFT — Etapa 28 (Fase F): desempenho, robustez, telas e homologação geral

**Estado: APROVADO pelo autor em 2026-11-05 ("Aprovo todas"), com todas as decisões recomendadas.**

- **Plano Mestre:** `docs/internal/plans/PLANO_MESTRE_2026-11-03.md`, Fase F. Última etapa do plano.
- **Ordem:** depois do R6 (ou, se o autor preferir, antes dos blocos R; ver decisões).
- **Ler antes:** `AGENTS.md`, `AGENTS.local.md`, HANDOFF, CURRENT_STATE, KNOWN_ISSUES e ROADMAP.

## Objetivo

Deixar o CAD-CRAFT pronto para uso em projetos reais: rápido, estável em telas e escalas diferentes, sem pendências abertas graves, e homologado pelo autor de ponta a ponta.

## Parte A — desempenho (implementação)

1. **GUI-L01, CPU alta parado:** o app usa cerca de 45% a 75% de um núcleo sem fazer nada (medido em 2026-11-05; já existia no R1). Descobrir o que pede redesenho a cada quadro (`request_repaint`, animações, cursor de texto, painéis, autosave, *polling* de tarefas) e redesenhar só quando há evento ou tarefa ativa. Meta: menos de 5% de um núcleo parado, medido no Release.
2. **Projeto grande sintético** (gerado por teste, não versionado): 200 páginas, 5.000 dispositivos, 20.000 conexões. Medir e registrar no relatório: abrir, salvar, desenhar uma página, rolar e dar zoom, numerar, verificações, exportar PDF e DXF, relatórios. Corrigir o que passar de: abrir/salvar 3 s, desenhar página 16 ms, numerar 2 s, PDF de 200 páginas 30 s.
3. Teste automático de desempenho com limites folgados (para não falhar em CI lento), marcado para rodar só no Release ou com variável de ambiente.

## Parte B — robustez

1. Fuzz leve sem nightly: testes com entradas aleatórias determinísticas (semente fixa) para `.cadcraft`, QET, ELMT, titleblock, FN1, EDZ e DXF; nenhum pode panicar.
2. Autosave e recuperação: matar o processo no meio de uma edição (teste com processo filho) e recuperar sem perda além do último intervalo.
3. Revisar KNOWN_ISSUES: resolver os pendentes pequenos; os grandes ficam com plano e prazo.

## Parte C — telas e escalas

1. Testes de GUI em 1280×720, 1920×1080 e 3840×2160, com zoom da interface 100%, 125%, 150% e 200%: nada cortado nos painéis, diálogos cabem na tela, faixa de comandos com rolagem quando falta espaço.
2. Corrigir o que falhar, sem diminuir os testes existentes.

## Parte D — homologação geral (manual)

- Roteiro em `docs/internal/reports/ETAPA_28_HOMOLOGACAO.md`, por fase do Plano Mestre (A molduras, B esquema, C saídas, D interoperabilidade, E assistente), com cada item "ok / defeito / observação".
- Executada no app Release pelo Claude Code, com a autorização do autor para usar o computador, e conferida pelo autor. O login do ChatGPT é feito pelo autor no navegador.
- Defeitos encontrados: corrigidos na própria etapa, com teste; ou registrados com prioridade se forem grandes.

## Validação e entrega

- `scripts/validate.ps1` com contagem igual ou maior; saídas de referência idênticas, salvo correção intencional explicada (regravar com `CAD_CRAFT_WRITE_BASELINE=1`).
- Release compilado; relatório com as medições antes e depois; ROADMAP com o Plano Mestre concluído; memória atualizada.
- Commit + push só com o OK; depois, sincronização com o público.

## Decisões pedidas ao autor

1. Ordem: Etapa 28 depois do R6 (recomendado: homologa o código já reorganizado) ou antes dos blocos R.
2. Metas de desempenho acima (recomendado) ou outras.
3. Implementação das partes A–C pelo Codex e homologação (parte D) pelo Claude Code (recomendado).
