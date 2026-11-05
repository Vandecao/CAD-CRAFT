# CAD-CRAFT — ETAPA 2

## IMPLEMENTAÇÃO DO MODELO DE CONECTIVIDADE ELÉTRICA E REDES LÓGICAS

### 1. AUTORIZAÇÃO

Implementar integralmente a **Etapa 2 — Conectividade Elétrica e Modelo de Redes Lógicas** do CAD-CRAFT.

**Projeto:** CAD-CRAFT
**Diretório:** `C:\Dev\CAD-CRAFT`
**Repositório:** `https://github.com/Vandecao/CAD-CRAFT.git`
**Branch principal:** `main`

A Etapa 1 foi concluída tecnicamente e seu commit foi realizado manualmente pelo usuário.

A homologação manual das funcionalidades está adiada até a conclusão da Etapa 15.

Executar autonomamente a Etapa 2, utilizando as ferramentas de planejamento, implementação, testes e revisão disponíveis.

**Não iniciar a Etapa 3. Não realizar operações Git modificadoras.**

---

### 2. RECUPERAÇÃO OBRIGATÓRIA DE CONTEXTO

Antes de qualquer alteração:

1. Ler integralmente `AGENTS.md`.
2. Ler `docs/internal/memory/HANDOFF.md`.
3. Ler `docs/internal/memory/CURRENT_STATE.md`.
4. Consultar `docs/internal/memory/ARCHITECTURE.md`.
5. Consultar `docs/internal/memory/DECISIONS.md`.
6. Consultar `docs/internal/memory/ROADMAP.md`.
7. Consultar os ADRs existentes.
8. Examinar o código-fonte implementado na Etapa 1.
9. Examinar os testes automatizados existentes.
10. Consultar `docs/internal/reports/ETAPA_01_CORE_DOMINIO.md`.
11. Verificar o estado do Git utilizando comandos exclusivamente de leitura.

Confirmar a arquitetura real antes de implementar novas funcionalidades.

Não presumir APIs, tipos ou estruturas que não existam no código.

Não recriar entidades já implementadas.

Preservar as decisões arquiteturais anteriores.

Se houver divergência entre este prompt e contratos existentes, analisar tecnicamente a divergência, documentar a resolução e preservar a compatibilidade sempre que possível.

---

### 3. OBJETIVO PRINCIPAL

Implementar o sistema de conectividade elétrica lógica do CAD-CRAFT.

O Core deverá ser capaz de:

* Representar conexões elétricas entre pontos.
* Identificar pontos eletricamente conectados.
* Derivar redes lógicas.
* Identificar componentes pertencentes a uma rede.
* Consultar a conectividade entre dispositivos.
* Validar conexões.
* Evitar relações inválidas.
* Preservar a integridade do projeto.
* Serializar e reconstruir o modelo elétrico.
* Preparar a arquitetura para futuras representações gráficas.

A implementação deverá permitir representar circuitos elétricos industriais com múltiplos dispositivos e pontos de conexão.

**Não implementar ainda o roteamento visual de fios.**

---

### 4. PRINCÍPIO ARQUITETURAL FUNDAMENTAL

O CAD-CRAFT deverá distinguir rigorosamente:

**Conectividade lógica:** relação elétrica entre pontos de conexão.

**Representação gráfica:** linhas, símbolos e elementos utilizados para apresentar visualmente essa conectividade.

**Condutor físico:** elemento real de cabeamento, com características como seção, cor, identificação e comprimento.

Esses três conceitos não são equivalentes.

Uma conexão lógica não representa automaticamente um fio físico.

Uma linha gráfica não representa automaticamente um condutor físico.

Um condutor físico poderá futuramente possuir atributos, terminais e rastreabilidade próprios.

O modelo desenvolvido nesta etapa deverá representar exclusivamente a conectividade lógica.

---

### 5. ORGANIZAÇÃO DO CÓDIGO

Criar ou adaptar módulos no crate `cad-craft`.

Estrutura conceitual sugerida:

```text
crates/cad-craft/src/
├── domain/
│   ├── connection.rs
│   ├── connection_point.rs
│   ├── network.rs
│   └── project/
│       └── connectivity.rs
│
├── graph/
│   ├── mod.rs
│   ├── connectivity.rs
│   └── algorithms.rs
│
└── error.rs
```

A estrutura é orientativa.

O agente deverá respeitar a organização real estabelecida na Etapa 1.

Evitar código duplicado.

Não criar abstrações desnecessárias.

O Core deverá continuar independente da GUI.

---

### 6. IDENTIFICADORES TIPADOS

Estender o sistema existente de IDs para contemplar conexões.

Criar:

`ConnectionId`

Utilizar UUID v7, seguindo exatamente os contratos já estabelecidos.

Preservar:

* Tipagem forte.
* Serialização.
* Desserialização.
* Comparação.
* Ordenação determinística.
* Validação de UUID.
* Separação entre categorias de identificadores.

Avaliar se `NetworkId` é realmente necessário.

**Preferir redes derivadas a entidades persistentes com identificadores independentes**, evitando múltiplas fontes de verdade.

Caso uma identidade de rede estável seja necessária futuramente, registrar essa necessidade como decisão arquitetural pendente.

---

### 7. MODELO DE CONEXÃO LÓGICA

Implementar a entidade `Connection`.

Ela deverá possuir:

* `ConnectionId`.
* `ProjectId`.
* Ponto de origem.
* Ponto de destino.
* Propriedades aplicáveis à conexão lógica, quando justificadas.

Requisitos:

1. Os extremos deverão utilizar `ConnectionPointId`.
2. Ambos deverão existir no mesmo projeto.
3. Não permitir conexões entre projetos diferentes.
4. Não permitir uma conexão entre um ponto e ele próprio.
5. Não permitir duplicação lógica da mesma conexão.
6. Tratar conexões como não direcionadas, salvo justificativa arquitetural específica.
7. Preservar identificadores persistentes.
8. Permitir consultas eficientes.
9. Garantir comportamento determinístico.

As conexões A–B e B–A deverão ser consideradas equivalentes para fins de duplicação.

Definir uma representação canônica dos extremos, utilizando a ordenação dos IDs.

---

### 8. INTEGRAÇÃO COM PROJECT

Integrar as conexões ao agregado `Project` criado na Etapa 1.

Implementar operações seguras como:

```rust
add_connection(...)
get_connection(...)
connections(...)
remove_connection(...)
```

As assinaturas exatas deverão ser definidas conforme a arquitetura existente.

Requisitos:

* Validação antes de mutação.
* Operações atômicas no nível do agregado.
* Erros estruturados.
* Nenhuma inserção parcial em caso de falha.
* Preservação das relações existentes.
* Consultas imutáveis quando possível.

A remoção de uma conexão não deverá excluir seus pontos ou dispositivos proprietários.

Não implementar operações genéricas de exclusão em cascata nesta etapa.

---

### 9. MODELO DE GRAFO ELÉTRICO

Representar a conectividade por um grafo não direcionado.

**Vértices:** pontos de conexão.

**Arestas:** conexões lógicas.

A estrutura deverá suportar:

* Pontos isolados.
* Pares conectados.
* Cadeias.
* Ramificações.
* Ciclos.
* Redes independentes.
* Redes com múltiplos dispositivos.

A implementação poderá utilizar estruturas da biblioteca padrão Rust.

Não adicionar uma biblioteca de grafos externa sem necessidade técnica demonstrável.

Evitar dependências desnecessárias.

---

### 10. REDES ELÉTRICAS DERIVADAS

Implementar identificação de componentes conexas do grafo.

Cada componente conexa representará uma rede lógica.

A rede deverá ser derivada das conexões existentes e não armazenada como segunda fonte persistente de verdade.

Requisitos:

1. Identificar todos os pontos pertencentes a uma rede.
2. Identificar conexões pertencentes à rede.
3. Identificar dispositivos e funções associados aos pontos.
4. Identificar pontos isolados.
5. Diferenciar redes independentes.
6. Atualizar corretamente o resultado após inserções.
7. Atualizar corretamente o resultado após remoções.
8. Garantir resultados determinísticos.
9. Evitar duplicação de pontos nos resultados.

Pontos isolados deverão formar componentes unitárias do grafo, mesmo sem conexões explícitas.

Estabelecer ordenação determinística das redes e de seus membros.

---

### 11. ALGORITMOS DE CONECTIVIDADE

Implementar operações para:

* Verificar se dois pontos estão conectados.
* Consultar a rede de determinado ponto.
* Listar as redes de um projeto.
* Listar pontos diretamente conectados.
* Listar conexões incidentes.
* Identificar redes independentes.
* Detectar ciclos topológicos, quando necessário.

Utilizar algoritmos adequados, como:

* Busca em profundidade (DFS).
* Busca em largura (BFS).
* Union-Find, caso tecnicamente justificável.

**Atenção:** Union-Find é eficiente para inserções, mas não oferece remoção dinâmica de arestas diretamente.

Como o CAD-CRAFT deverá permitir remoção de conexões, escolher uma estratégia que mantenha a correção dos resultados após exclusões.

Priorizar correção e simplicidade nesta etapa.

Otimizações poderão ser introduzidas mediante medições reais.

---

### 12. DISTINÇÃO ENTRE CONECTIVIDADE E FUNÇÃO ELÉTRICA

O grafo deverá representar relações explícitas de conectividade externa.

**Não presumir que todos os terminais pertencentes ao mesmo dispositivo estejam eletricamente conectados.**

Exemplo:

Um contator possui bobina, contatos auxiliares e contatos principais.

Essas funções não podem ser tratadas como um único nó elétrico simplesmente por pertencerem ao mesmo dispositivo.

Da mesma forma:

* Dois contatos de um relé não estão permanentemente conectados.
* Dois terminais de uma bobina não devem ser unidos automaticamente.
* Dispositivos podem possuir múltiplos circuitos independentes.
* Relações internas dependentes de estado não devem ser confundidas com conectividade permanente.

Esta etapa não deverá implementar simulação elétrica nem lógica de estados.

Registrar essas distinções na documentação.

---

### 13. VALIDAÇÃO DE CONEXÕES

Implementar regras para rejeitar:

* Pontos inexistentes.
* Pontos de outros projetos.
* Conexões autorreferentes.
* Conexões duplicadas.
* IDs duplicados.
* Relações inconsistentes.
* Referências inválidas após desserialização.

A remoção de conexão inexistente deverá retornar erro estruturado, sem alterar o estado do projeto.

Conectar pontos já ligados indiretamente por uma rede deverá ser permitido quando representar uma nova aresta explícita válida, inclusive formando ciclos.

**Não confundir conexão direta duplicada com conectividade indireta existente.**

---

### 14. SERIALIZAÇÃO E RECONSTRUÇÃO

Estender a serialização Serde criada na Etapa 1.

Requisitos:

1. Serializar conexões lógicas.
2. Preservar os IDs.
3. Preservar os extremos.
4. Reconstruir corretamente o projeto.
5. Validar referências na desserialização.
6. Rejeitar conexões inválidas.
7. Rejeitar conexões duplicadas.
8. Preservar determinismo.
9. Verificar round-trip.
10. Recalcular redes derivadas após reconstrução.

Não implementar o formato definitivo `.cadcraft`.

Não implementar ZIP, backups versionados ou persistência física.

Esses recursos pertencem às etapas posteriores.

---

### 15. CENÁRIOS DE TESTE

Criar testes automatizados para, no mínimo:

**Cenário A — Rede simples**

A conectado a B.

Resultado: uma rede contendo A e B.

**Cenário B — Cadeia**

A–B–C–D.

Resultado: quatro pontos na mesma rede.

**Cenário C — Ramificação**

A–B, B–C, B–D.

Resultado: rede única com quatro pontos.

**Cenário D — Redes independentes**

A–B e C–D.

Resultado: duas redes independentes.

**Cenário E — Ponto isolado**

Ponto E sem conexões.

Resultado: componente unitária independente.

**Cenário F — Ciclo**

A–B, B–C, C–A.

Resultado: rede única sem duplicação de membros.

**Cenário G — Remoção**

A–B–C.

Remover B–C.

Resultado: A–B permanece conectado; C torna-se isolado.

**Cenário H — Conexão duplicada**

Inserir A–B e depois B–A.

Resultado: segunda inserção rejeitada.

**Cenário I — Referência inválida**

Tentar conectar ponto inexistente.

Resultado: erro, sem mutação.

**Cenário J — Serialização**

Serializar e desserializar um projeto com múltiplas redes.

Resultado: estrutura e conectividade equivalentes.

---

### 16. TESTES DE INTEGRIDADE E REGRESSÃO

Além dos cenários anteriores, criar testes para:

* Identificadores UUID v7.
* Ordem determinística.
* Conexões incidentes.
* Dispositivos com múltiplas funções independentes.
* Redes com pontos de diferentes dispositivos.
* Remoção de arestas em ciclos.
* Remoção de arestas que dividem uma rede.
* Reconstrução após desserialização.
* Dados adulterados.
* IDs duplicados no snapshot.
* Falhas sem alteração do agregado.
* Ausência de associação implícita entre terminais do mesmo dispositivo.

Preservar todos os testes da Etapa 1.

A suíte existente possuía 26 testes Rust e 15 testes PowerShell aprovados.

Não reduzir essa cobertura para facilitar a implementação.

---

### 17. DESEMPENHO

Preparar a arquitetura para projetos industriais de médio e grande porte.

Evitar algoritmos desnecessariamente quadráticos.

Implementar testes com grafos sintéticos de tamanhos crescentes.

Avaliar cenários com:

* 100 pontos.
* 1.000 pontos.
* 10.000 pontos.

Não estabelecer metas de desempenho arbitrárias.

Registrar tempos observados quando houver medições confiáveis.

Evitar recursão profunda que possa causar estouro de pilha em grafos grandes.

Priorizar algoritmos iterativos para travessias extensas.

---

### 18. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_02_CONECTIVIDADE_RED​ES.md`

Utilizar nome de arquivo ASCII, sem caracteres invisíveis:

`ETAPA_02_CONECTIVIDADE_REDES.md`

Documentar:

1. Objetivos.
2. Arquitetura do grafo.
3. Modelo de conexão.
4. IDs.
5. Integração com Project.
6. Operações públicas.
7. Algoritmos.
8. Redes derivadas.
9. Integridade.
10. Serialização.
11. Testes.
12. Limitações.
13. Próximas dependências.

Incluir diagramas Mermaid quando úteis.

Documentar explicitamente que conexões lógicas não equivalem a condutores físicos.

---

### 19. ATUALIZAÇÃO DA MEMÓRIA PERSISTENTE

Atualizar:

* `docs/internal/memory/CURRENT_STATE.md`
* `docs/internal/memory/HANDOFF.md`
* `docs/internal/memory/ARCHITECTURE.md`
* `docs/internal/memory/ROADMAP.md`
* `docs/internal/memory/KNOWN_ISSUES.md`, se necessário.
* `docs/internal/memory/DECISIONS.md`, se necessário.

Criar ADR se houver decisão arquitetural significativa.

Registrar o estado:

**Etapa 2 — Implementada e testada automaticamente; homologação manual pendente.**

Somente registrar esse estado após concluir os testes.

Não declarar a Etapa 3 como iniciada.

---

### 20. VALIDAÇÕES OBRIGATÓRIAS

Executar:

```powershell
cargo fmt --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
git diff --check
```

Executar também:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

Verificar os testes PowerShell de diagnóstico existentes.

Registrar os resultados reais.

Se o script `validate.ps1` já executar os mesmos comandos, evitar execuções duplicadas desnecessárias, desde que o relatório apresente evidências de todas as validações obrigatórias.

Corrigir falhas encontradas.

Não ignorar warnings.

---

### 21. REVISÃO TÉCNICA

Executar revisão independente, se a infraestrutura disponível permitir.

Avaliar:

* Integridade do grafo.
* Correção das componentes conexas.
* Comportamento após remoções.
* Serialização.
* Determinismo.
* Complexidade dos algoritmos.
* Acoplamento entre módulos.
* Independência da GUI.
* Tratamento de erros.
* Regressões da Etapa 1.
* Conformidade com as decisões arquiteturais.

Corrigir problemas identificados dentro do escopo.

Não apresentar revisão não executada como concluída.

---

### 22. POLÍTICA GIT — OBRIGATÓRIA

Respeitar integralmente o `AGENTS.md` e o ADR-0002.

**Todas as operações Git modificadoras são exclusivas do usuário.**

O Codex não poderá executar:

* `git add`
* `git commit`
* `git push`
* `git pull`
* `git fetch`
* `git merge`
* `git rebase`
* `git reset`
* `git revert`
* `git checkout`
* `git switch`
* Operações equivalentes.

Utilizar somente comandos de consulta permitidos.

Ao concluir, apresentar as alterações para revisão.

Não realizar staging.

Não criar commits.

Não sincronizar o repositório.

---

### 23. CRITÉRIOS DE ACEITAÇÃO

A Etapa 2 estará tecnicamente concluída quando:

* O modelo de conexão estiver implementado.
* O grafo lógico estiver funcional.
* As redes derivadas estiverem corretas.
* A remoção de conexões funcionar.
* A integridade referencial estiver preservada.
* A serialização estiver funcionando.
* Os testes automatizados estiverem aprovados.
* Os testes anteriores continuarem passando.
* A arquitetura permanecer independente da GUI.
* A documentação estiver atualizada.
* A memória persistente estiver atualizada.
* Nenhuma operação Git modificadora tiver sido executada.

Homologação manual permanece pendente.

---

### 24. RELATÓRIO FINAL

Apresentar:

# CAD-CRAFT — ETAPA 2 CONCLUÍDA

**Estado:** concluída / parcialmente concluída / bloqueada.

**Arquitetura:** descrição da implementação.

**Arquivos:** criados, modificados e excluídos.

**Testes:** quantidade total e resultados.

**Validações:** fmt, check, test, Clippy, Git diff e scripts.

**Desempenho:** observações verificadas.

**Memória persistente:** arquivos atualizados.

**Pendências:** limitações e problemas conhecidos.

**Git:** branch, HEAD e alterações pendentes.

**Próxima etapa:** Etapa 3, aguardando autorização.

Encerrar com:

**ETAPA 2 IMPLEMENTADA — AGUARDANDO REVISÃO, COMMIT E PUSH MANUAIS DO USUÁRIO.**

---

## RESTRIÇÕES FINAIS

Não iniciar a Etapa 3.

Não implementar editor gráfico.

Não implementar WireRoute ou WireJunction.

Não implementar condutores físicos.

Não implementar ERC completo.

Não implementar exportação DXF.

Não implementar persistência física definitiva.

Não executar operações Git modificadoras.

Não declarar homologação manual.

**Executar integralmente a Etapa 2, testar, atualizar a memória persistente, produzir o relatório e encerrar.**
