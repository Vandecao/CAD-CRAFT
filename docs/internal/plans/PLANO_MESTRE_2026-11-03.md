# Plano Mestre — CAD-CRAFT até o final (aprovado em 2026-11-03)

Meta final (usuário): ECAD utilizável em projetos reais + interoperabilidade
com EPLAN 2027 + assistente de IA. Etapas 1–12 concluídas (12 = `.cadcraft` e
inserção com dispositivo, commit `6612e82`).

## Fases e etapas

| Fase | Etapa | Entrega | Depende de |
|---|---|---|---|
| A — Folha profissional | 13 | Modelo de moldura: zonas (colunas/linhas, rótulos, área útil), textos fixos e especiais, moldura por projeto/página, desenho na folha, referências página.coluna, formatos A0–A4/personalizados | 12 |
| | 14 | Editor de molduras: gráficos, textos, textos especiais, imagens PNG/JPG (logotipos) em `assets/`, biblioteca e arquivo próprio de moldura (importar/exportar) | 13 |
| | 15 | Molduras de outros sistemas: DXF, `.titleblock` (QElectroTech), `.fn1` do EPLAN (XML PXF) | 13, 14; molduras reais do usuário |
| B — Conteúdo | 16 | Texto livre e gráficos na página; copiar/recortar/colar | 12 |
| | 17 | Biblioteca externa de símbolos, primitivas ampliadas, editor de símbolos, `.elmt` | 14 (editor gráfico reaproveitável) |
| | 18 | Dispositivos e peças; editor de funções e pontos | 17 |
| | 19 | Macros | 16, 17, 18 |
| | 20 | Numeração automática e referências cruzadas completas | 13, 18 |
| C — Saídas | 21 | PDF multipágina com links; impressão | 13, 14 |
| | 22 | DXF; DWG conforme licença | 21 (lista de exibição) |
| | 23 | Relatórios em página e CSV/XLSX | 20, 21 |
| | 24 | Verificações de consistência; DEC-11-01/02, CON-001 | 20 |
| D — Interoperabilidade | 25 | Importação `.qet`/`.qetz` | 17, 18, 13 |
| | 26 | EPLAN 2027 (listas, XML/AutomationML, DXF/DWG, `.edz`) | amostras do usuário |
| E — IA | 27 | Assistente de IA (spec `ETAPA_17_ASSISTENTE_IA_OPENAI.md`) | funcionalidades estáveis |
| F — Fechamento | 28 | Desempenho, robustez, telas/escalas, correções, homologação geral | todas |

## Regras de trabalho

- Cada etapa: prompt registrado em `docs/internal/spec/` e aprovado pelo usuário,
  plano em `docs/internal/plans/`, testes automáticos de aceitação (eventos egui reais
  quando houver GUI), `scripts/validate.ps1` aprovado, relatório, ADR quando
  houver decisão, memória atualizada.
- Validação manual: homologação geral somente na Etapa 28 (decisão do
  usuário). Verificação rápida opcional ao fim de cada fase.
- Git: ao fim de cada etapa o agente avisa; com o OK expresso do usuário
  ("pode versionar"), faz commit + push daquela etapa (ADR-0002, adendo).

## Riscos

1. Moldura EPLAN `.fn1`: XML PXF com propriedades codificadas (`P14020`,
   `A1819`…) sem documentação pública; o mapeamento exige molduras reais com
   conteúdo (a amostra `FN1_00A1.fn1` recebida tem `NumMainObjects="0"`).
2. DWG: biblioteca comercial (ODA) ou conversor externo; DXF primeiro.
3. Validação só no final acumula risco; mitigado por testes de aceitação.
4. Elementos do QElectroTech (CC-BY 3.0) não servem de dados para construir
   modelos de aprendizado de máquina; o assistente não treina modelos com eles.
