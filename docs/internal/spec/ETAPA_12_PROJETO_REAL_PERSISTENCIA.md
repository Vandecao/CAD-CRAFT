# CAD-CRAFT — Etapa 12 (Marco M0.1)

## Projeto real: arquivo `.cadcraft` e criação de dispositivos na inserção

Origem: auditoria comparativa CAD-CRAFT × QElectroTech de 2026-11-03,
seção 7 ("Primeira entrega operacional recomendada"). Aprovada pelo usuário
em 2026-11-03 com as respostas:

- "Implementar agora": a seção 7 do relatório vale como prompt da Etapa 12.
- Dependências novas autorizadas: `zip` (contêiner) e `rfd` (diálogos nativos
  Abrir/Salvar).

Linha de base: `main`, HEAD `34785ab` (Etapa 11 commitada), 418 testes Rust +
15 PowerShell aprovados na entrega da Etapa 11.

## Escopo autorizado

1. Formato `.cadcraft` v1: arquivo ZIP com `manifest.json` (`format`,
   `min_reader`, `writer`) e `project.json`; pasta `assets/` reservada.
2. Comandos Salvar (Ctrl+S), Salvar como, Abrir (Ctrl+O), Fechar com aviso de
   alterações e lista de arquivos recentes.
3. Trava de arquivo (`.lock`) e recuperação após encerramento abrupto.
4. Leitura do snapshot JSON atual como formato v0 (migração para v1).
5. Inserção de símbolo com "novo dispositivo": cria Dispositivo + Função +
   Pontos de conexão e vincula as portas numa única transação com Undo/Redo,
   com designação automática por prefixo (K1, K2…) editável.
6. Undo/Redo das operações existentes de página e documento (criar, excluir,
   reordenar; renomear/renumerar já eram reversíveis).

## Critérios de aceitação

1. Fluxo automatizado com cliques egui reais: novo projeto → documento →
   2 páginas → 4 símbolos (2 dispositivos) → 3 conexões com rede derivada →
   salvar → fechar → abrir → mover símbolo e renomear dispositivo → salvar →
   reabrir. IDs, redes, rotas, referências e dados físicos idênticos
   (igualdade do agregado).
2. Ida e volta com `.cadcraft` do projeto de demonstração completo
   (Etapas 1–11), sem perda.
3. Arquivos de teste versionados: v0 (JSON atual) e v1 abrem; `min_reader`
   maior que o suportado → recusa com mensagem; `format` maior → somente
   leitura.
4. ZIP malicioso (`..`, caminho absoluto, entrada duplicada, tamanho
   excessivo) recusado sem tocar o disco.
5. Falha simulada durante a gravação preserva o arquivo anterior.
6. Arquivo aberto por outra instância → aviso de trava; recuperação oferecida
   após encerramento abrupto simulado.
7. Undo/Redo da inserção com novo dispositivo e das operações de página
   restauram o estado exato.
8. Tamanho testado acima do limite atual de 8 MiB, com tempo medido.
9. `scripts/validate.ps1` aprovado, 418 testes Rust anteriores preservados e
   novos contados; Release gerado; relatório, ADR e memória atualizados;
   nenhuma operação Git executada.

## Fora do escopo

Molduras/carimbos, editor de símbolos, bibliotecas externas, PDF/DXF,
importação QElectroTech, macros, renomear/excluir documento (não existem como
funcionalidade anterior). Não iniciar a Etapa 13.
