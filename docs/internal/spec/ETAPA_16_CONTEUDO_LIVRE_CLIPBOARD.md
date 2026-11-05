# CAD-CRAFT — Etapa 16 (Fase B): conteúdo livre e copiar/recortar/colar

**Estado: aprovado pelo usuário em 2026-11-03; revisão das regras de colar/recortar aceita no mesmo dia (seções 3–5).**

Base de trabalho: Etapa 15 implementada e validada automaticamente, mas ainda
não commitada por instrução do usuário. A Etapa 16 depende funcionalmente da
Etapa 12; a sequência aprovada passa pela Etapa 15.

## Objetivo

Permitir anotações e formas independentes de entidades elétricas na folha e
reutilizar uma seleção gráfica na mesma página ou em outra página do projeto.
O conteúdo deve salvar no `.cadcraft`, aparecer na folha e participar de
seleção, propriedades, camadas e Desfazer/Refazer.

## Escopo proposto

1. **Conteúdo livre de página no Core.** IDs tipados UUID v7, página dona,
   posição em milímetros e variantes `Text`, `Line`, `Rectangle`, `Circle` e
   `Arc`. Texto fixo Unicode, altura, alinhamento e rotação; traço com
   espessura. Conteúdo livre não cria dispositivo, ponto, conexão ou rede.
   Validação rejeita valores não finitos, geometria degenerada, texto vazio
   e referências a página ausente. Elementos podem ficar fora da folha,
   seguindo a política de representações existente.
2. **Edição gráfica.** Inserir por ferramentas na faixa Inserir; prévia antes
   da confirmação; selecionar, mover, editar propriedades e excluir.
   Seleção múltipla usa o mecanismo atual e respeita camada e visibilidade.
   Cada operação confirmada é uma transação reversível, sem mutação durante
   arraste ou prévia.
3. **Copiar, recortar e colar.** Área de transferência estruturada da sessão
   com os elementos livres e ocorrências de símbolos selecionados; o ponto
   de referência é o canto inferior esquerdo do conjunto. Colar mostra a
   prévia, encaixa na grade e confirma por clique na página ativa, inclusive
   outra página do mesmo projeto; o clique seguinte cola outra vez, Esc
   encerra. Cópias recebem IDs novos; arranjo e propriedades visuais são
   preservados. **Revisão de 2026-11-03:** colar um símbolo de função cria um
   **dispositivo novo** com a próxima designação do mesmo prefixo, mesma
   classificação, tipo de função, símbolo e portas ligadas a pontos novos
   (como no EPLAN). O comando "Colar no mesmo dispositivo" cria, em vez
   disso, uma função nova do mesmo dispositivo. Símbolo sem função e
   conteúdo livre viram cópias simples. Colar com dispositivos novos é um
   único passo de Desfazer. Duplicar (Ctrl+D) cola a seleção deslocada sem
   usar a área de transferência.
4. **Recorte (revisado em 2026-11-03).** Recortar remove a seleção na hora,
   num passo de Desfazer, como no Windows e no EPLAN; o cadastro elétrico
   (dispositivo, função, pontos) permanece. A primeira colagem devolve os
   itens com os mesmos IDs e vínculos de porta; as seguintes criam cópias.
   Símbolo ligado a fios não pode ser recortado (mensagem explícita).
5. **Limite de conectividade.** Rotas, junções e marcadores elétricos não são
   copiados/recortados nesta etapa, pois sua topologia não pode ser duplicada
   como desenho isolado. A seleção mista é recusada com mensagem; nada é
   alterado. O mesmo vale para operações entre projetos distintos. Copiar um
   circuito com os fios entre os símbolos copiados fica para as macros
   (Etapa 19). Limite de 10.000 itens por operação.
6. **Persistência.** O `.cadcraft` passa ao formato 4 / `min_reader` 4 para
   armazenar conteúdo livre; leitura de v0–v3 permanece suportada; fixture
   versionada `etapa16-v4.cadcraft`. Clipboard e prévias são transitórios e
   não entram no arquivo ou snapshot.
7. **Interface.** Comandos reais Inserir texto/gráfico, Copiar (Ctrl+C),
   Recortar (Ctrl+X), Colar (Ctrl+V), Colar no mesmo dispositivo, Duplicar
   (Ctrl+D) e Excluir para a seleção pertinente; duplo clique em texto livre
   abre a edição do texto. Atalhos não interceptam edição de campos de
   texto, modais ou editor de molduras. Cancelar com Esc não altera o projeto.

## Critérios de aceitação

1. Inserir as cinco variantes, editar, mover, selecionar e excluir com
   eventos egui reais; render e hit test concordam em zoom e DPI diferentes.
2. Copiar e colar seleção múltipla na mesma página e em outra mantém o
   arranjo e cria IDs novos; colar símbolo cria dispositivo novo com outra
   designação (ou função do mesmo dispositivo, se pedido), sem alterar as
   ligações existentes; um passo de Desfazer restaura exatamente.
3. Recortar remove na hora e pode ser desfeito; recortar e colar entre
   páginas preserva IDs e vínculos; seleção não admitida é recusada
   atomicamente e deixa histórico/clipboard anteriores intactos.
4. Salvar/reabrir pela interface conserva conteúdo livre, propriedades,
   camada e IDs; fixtures v0–v4 abrem. Entradas inválidas não substituem
   projeto ou sessão.
5. `scripts/validate.ps1` e Release aprovados; testes anteriores preservados;
   relatório, ADR e memória atualizados. Teste funcional no executável ao fim
   da Etapa 16, conforme a cadência definida pelo usuário e somente com sua
   autorização expressa para usar o computador; homologação geral somente na
   Etapa 28.

## Fora do escopo

Elipse, estilos ampliados e textos dinâmicos de símbolo (Etapa 17); macros e
cópia de circuitos com fios (Etapa 19); clipboard entre projetos ou entre
execuções; exportação do conteúdo livre em DXF/PDF (Etapas 21–22).

## Decisões aprovadas

- Formas básicas e conteúdo do clipboard (aprovação inicial, 2026-11-03).
- Formato `.cadcraft` 4 / `min_reader` 4 e teste funcional no app ao fim da
  etapa.
- Revisão aceita pelo usuário em 2026-11-03 (substitui a regra anterior de
  "mesmo alvo elétrico"): colar símbolo cria dispositivo novo, com opção de
  mesmo dispositivo; recortar remove na hora; circuitos com fios ficam para a
  Etapa 19; itens menores (duplo clique, Ctrl+D, colar repetido com grade,
  fixture v4, testes dos cinco tipos e de salvar/reabrir, limite).
