# CAD-CRAFT — ETAPA 3

## MODELO DE CONDUTORES FÍSICOS, CABOS, BORNES E CONECTIVIDADE FÍSICA

### 1. AUTORIZAÇÃO

Implementar integralmente a **Etapa 3 — Condutores Físicos, Cabos, Bornes e Conectividade Física** do CAD-CRAFT.

**Projeto:** CAD-CRAFT
**Diretório:** `C:\Dev\CAD-CRAFT`
**Repositório remoto:** `https://github.com/Vandecao/CAD-CRAFT.git`
**Branch:** `main`

Contexto:

* Etapa 1: Core e modelo de domínio elétrico.
* Etapa 2: Conectividade lógica e redes derivadas.
* Etapa 3: implementação autorizada neste prompt.

A Etapa 2 foi concluída tecnicamente, com 70 testes automatizados aprovados, incluindo 55 testes Rust.

O último commit informado pelo usuário é:

`9844571`

O usuário confirmou que os commits das Etapas 1 e 2 foram enviados ao GitHub.

Essas informações são referências históricas. Verificar o estado real do repositório antes de qualquer alteração.

**Executar exclusivamente a Etapa 3. Não iniciar a Etapa 4.**

Utilizar os recursos de planejamento, implementação, testes e revisão técnica disponíveis no Codex, respeitando integralmente a política de Git do projeto.

---

### 2. RECUPERAÇÃO OBRIGATÓRIA DE CONTEXTO

Antes de implementar:

1. Ler `AGENTS.md`.
2. Ler `docs/internal/memory/HANDOFF.md`.
3. Ler `docs/internal/memory/CURRENT_STATE.md`.
4. Consultar `docs/internal/memory/ARCHITECTURE.md`.
5. Consultar `docs/internal/memory/DECISIONS.md`.
6. Consultar `docs/internal/memory/ROADMAP.md`.
7. Consultar os ADRs existentes.
8. Examinar `docs/internal/reports/ETAPA_01_CORE_DOMINIO.md`.
9. Examinar `docs/internal/reports/ETAPA_02_CONECTIVIDADE_REDES.md`.
10. Inspecionar o código-fonte Rust existente.
11. Examinar os testes automatizados.
12. Consultar os contratos públicos do Core.
13. Verificar o estado do Git utilizando somente comandos de leitura.

Não presumir APIs inexistentes.

Não reconstruir funcionalidades já implementadas.

Não modificar decisões arquiteturais anteriores sem justificativa documentada.

Quando este prompt utilizar nomes conceituais diferentes daqueles existentes no código, priorizar os contratos reais e registrar o mapeamento adotado.

---

### 3. OBJETIVO PRINCIPAL

Implementar no Core a infraestrutura necessária para representar:

* Condutores elétricos individuais.
* Cabos multipolares.
* Veias de cabos.
* Bornes físicos.
* Réguas de bornes.
* Ligações físicas entre pontos.
* Identificação de condutores.
* Características técnicas de cabos.
* Associações entre cabos e condutores.
* Rastreabilidade de conexões físicas.
* Integridade referencial.
* Serialização e reconstrução do modelo.

O resultado deverá permitir documentar ligações físicas de instalações elétricas industriais.

Exemplos de aplicação:

* Circuitos de comando.
* Circuitos de potência.
* Interligações entre painéis.
* Cabos de sensores.
* Cabos de atuadores.
* Ligações de CLPs.
* Conexões em réguas de bornes.
* Cabos multipolares utilizados em máquinas industriais.

Esta etapa não deverá implementar esquemas gráficos nem roteamento visual.

---

### 4. PRINCÍPIO ARQUITETURAL FUNDAMENTAL

Preservar a distinção entre três camadas.

#### 4.1. Conexão lógica

Representa a conectividade elétrica estabelecida explicitamente entre pontos de conexão.

Implementada na Etapa 2.

#### 4.2. Condutor físico

Representa um elemento real de interligação elétrica, com identidade e características físicas próprias.

Exemplos:

* Fio individual.
* Veia de cabo multipolar.
* Condutor de proteção.
* Condutor de alimentação.
* Condutor de sinal.

#### 4.3. Representação gráfica

Representa visualmente símbolos, linhas e rotas no desenho técnico.

Será implementada em etapas posteriores.

**Regra obrigatória:**

Uma `Connection` não deverá criar automaticamente um `Conductor`.

Da mesma forma, uma representação gráfica futura não deverá criar implicitamente um condutor físico.

A associação entre essas entidades deverá ser explícita, validada e rastreável.

---

### 5. ORGANIZAÇÃO DO CÓDIGO

Estender o crate `cad-craft`.

Estrutura conceitual sugerida:

```text
crates/cad-craft/src/
├── domain/
│   ├── conductor.rs
│   ├── cable.rs
│   ├── cable_core.rs
│   ├── terminal_strip.rs
│   ├── terminal.rs
│   └── project/
│       ├── physical_wiring.rs
│       └── physical_integrity.rs
│
├── electrical/
│   ├── mod.rs
│   └── physical.rs
│
└── error.rs
```

A estrutura poderá ser adaptada à organização real.

Evitar duplicação de conceitos existentes.

Não introduzir bibliotecas externas sem justificativa técnica.

Manter o Core independente de egui/eframe e APIs gráficas.

---

### 6. IDENTIFICADORES TIPADOS

Estender o mecanismo UUID v7 existente.

Considerar:

```rust
ConductorId
CableId
CableCoreId
TerminalStripId
```

Cada identificador deverá:

* Possuir tipagem forte.
* Utilizar UUID v7.
* Suportar serialização.
* Suportar desserialização.
* Suportar comparação e ordenação.
* Preservar identidade persistente.
* Impedir conversões implícitas entre categorias.

Reutilizar os mecanismos existentes da Etapa 1.

Não duplicar implementações de UUID.

---

### 7. MODELO DE CONDUTOR FÍSICO

Implementar a entidade `Conductor`.

Propriedades essenciais:

* Identificador persistente.
* Projeto proprietário.
* Designação.
* Ponto físico de origem.
* Ponto físico de destino.
* Seção transversal nominal.
* Cor ou identificação.
* Comprimento opcional.
* Referência opcional ao cabo associado.
* Propriedades técnicas aplicáveis.

Requisitos:

1. Possuir identidade independente de `Connection`.
2. Utilizar referências tipadas.
3. Preservar integridade referencial.
4. Permitir associação explícita com pontos de terminação existentes.
5. Não permitir extremos inexistentes.
6. Não permitir condutores entre projetos diferentes.
7. Não permitir condutor terminado no mesmo ponto físico nas duas extremidades.
8. Permitir condutores diferentes entre os mesmos pontos somente quando representarem elementos físicos distintos e explicitamente identificados.
9. Não assumir que o comprimento possa ser calculado a partir do esquema elétrico.
10. Não confundir cor com função elétrica.

A representação dos extremos físicos deverá considerar que um terminal pode possuir mais de uma posição de conexão.

Avaliar se a arquitetura atual necessita de um tipo específico para ponto de terminação física.

Se necessário, implementar uma abstração mínima, evitando confundir `ConnectionPointId` funcional com um terminal físico.

Documentar a decisão no ADR.

---

### 8. CARACTERÍSTICAS TÉCNICAS DOS CONDUTORES

Implementar tipos de valor apropriados para:

* Seção transversal em mm².
* Comprimento em metros ou milímetros, com unidade explícita.
* Cor.
* Identificação alfanumérica.
* Material condutor, quando aplicável.
* Classificação funcional.

Validar:

* Valores finitos.
* Seções estritamente positivas.
* Comprimentos não negativos.
* Identificações válidas.
* Enumerações controladas.

Evitar representar grandezas físicas apenas como `f64` sem unidade documentada.

Não implementar cálculos de dimensionamento elétrico nesta etapa.

Não implementar cálculo de queda de tensão ou capacidade de condução de corrente.

---

### 9. MODELO DE CABO

Implementar a entidade `Cable`.

Responsabilidades:

* Identificação do cabo.
* Projeto proprietário.
* Designação.
* Fabricante opcional.
* Código de catálogo opcional.
* Tipo de cabo.
* Quantidade de veias.
* Características técnicas.
* Relação com as veias pertencentes ao cabo.

Um cabo deverá possuir identidade independente dos seus condutores.

Não presumir que todos os condutores de um cabo estejam eletricamente conectados.

Não presumir continuidade elétrica entre veias diferentes.

---

### 10. VEIAS DE CABOS

Implementar `CableCore` ou estrutura equivalente.

Cada veia deverá possuir:

* Identidade persistente.
* Cabo proprietário.
* Identificação dentro do cabo.
* Cor ou numeração.
* Seção transversal, quando aplicável.
* Referência ao condutor físico correspondente, quando instalado.

Requisitos:

1. Uma veia pertence a um único cabo.
2. Identificações de veias não podem ser duplicadas dentro do mesmo cabo.
3. Veias de um cabo não podem ser associadas a condutores de outro projeto.
4. Uma veia não pode ser vinculada simultaneamente a condutores incompatíveis.
5. Veias não utilizadas deverão ser representáveis.
6. Condutores individuais não precisam pertencer a cabos.
7. A integridade deverá ser verificada após desserialização.

Preferir uma única fonte de verdade para a associação entre veia e condutor.

Não manter relações bidirecionais redundantes sem mecanismo confiável de consistência.

---

### 11. BORNES E RÉGUAS DE BORNES

A Etapa 1 já implementou `Terminal`.

Reutilizar essa entidade.

Não criar um segundo conceito incompatível de terminal físico.

Implementar uma entidade `TerminalStrip`, representando uma régua de bornes.

Propriedades:

* Identificador.
* Projeto proprietário.
* Designação.
* Localização opcional.
* Lista ou associação de bornes.
* Metadados técnicos.

Requisitos:

* Permitir múltiplos bornes.
* Preservar ordenação.
* Impedir duplicações indevidas.
* Permitir consulta dos bornes associados.
* Validar pertencimento ao projeto.
* Preservar designações de cada borne.

Considerar futuras necessidades:

* Bornes multinível.
* Bornes de passagem.
* Bornes PE.
* Bornes seccionáveis.
* Bornes fusíveis.
* Pontes entre bornes.
* Acessórios.
* Separadores.
* Identificação física.

**Não implementar antecipadamente todas essas funcionalidades.**

Preparar somente uma arquitetura extensível.

Não presumir continuidade elétrica entre lados, níveis ou terminais apenas pela classificação de um borne.

---

### 12. ASSOCIAÇÃO ENTRE CONECTIVIDADE LÓGICA E FÍSICA

Implementar associação explícita entre elementos físicos e o modelo lógico existente.

A associação deverá permitir identificar:

* Qual condutor físico documenta determinada ligação.
* Quais pontos funcionais correspondem às terminações físicas.
* Quais cabos e veias estão envolvidos.
* Se uma ligação lógica possui representação física associada.

Requisitos:

1. Não criar conexões lógicas implicitamente.
2. Não criar condutores implicitamente.
3. Não unir redes elétricas por simples associação física.
4. Validar todas as referências.
5. Evitar duplicação de fontes de verdade.
6. Preservar consistência após alterações.
7. Permitir ligações lógicas ainda sem condutor físico atribuído.
8. Permitir elementos físicos ainda não associados a uma conexão lógica, quando isso representar um estado de engenharia válido.

Distinguir estado incompleto de engenharia de corrupção estrutural.

Não tratar ausência de associação como erro estrutural obrigatório em todos os casos.

---

### 13. INTEGRAÇÃO COM PROJECT

Integrar o modelo físico ao agregado `Project`.

Implementar operações para:

* Inserir condutores.
* Consultar condutores.
* Inserir cabos.
* Consultar cabos.
* Inserir veias.
* Consultar veias.
* Criar réguas de bornes.
* Associar bornes a réguas.
* Associar condutores a veias.
* Associar condutores a conexões lógicas.
* Consultar elementos físicos relacionados a uma conexão.
* Consultar elementos físicos associados a um dispositivo.

As operações deverão seguir as convenções reais do Core.

Preferir:

* APIs tipadas.
* Resultados determinísticos.
* Consultas imutáveis.
* Validação antes de mutação.
* Erros estruturados.
* Operações atômicas no agregado.

Não implementar exclusões genéricas em cascata.

Se forem necessárias remoções ou reassociações dentro do escopo, implementar contratos explícitos que não deixem referências pendentes.

---

### 14. INTEGRIDADE REFERENCIAL

Estender `Project::validate` ou mecanismo equivalente.

Verificar:

* IDs duplicados.
* Projetos estrangeiros.
* Cabos inexistentes.
* Veias inexistentes.
* Condutores inexistentes.
* Bornes inexistentes.
* Associações duplicadas.
* Associações incompatíveis.
* Pontos de terminação inválidos.
* Referências lógicas inválidas.
* Designações inválidas.
* Valores físicos inválidos.
* Inconsistências entre cabo, veia e condutor.
* Inconsistências entre régua e borne.

Garantir que operações inválidas não deixem mutações parciais.

Definir erros específicos quando necessário.

Não utilizar `panic!` para erros esperados de domínio.

---

### 15. SERIALIZAÇÃO E RECONSTRUÇÃO

Estender o mecanismo Serde existente.

Requisitos:

1. Serializar condutores.
2. Serializar cabos.
3. Serializar veias.
4. Serializar réguas de bornes.
5. Preservar UUIDs.
6. Preservar associações.
7. Reconstruir corretamente o agregado.
8. Validar referências após carregamento.
9. Rejeitar duplicações.
10. Rejeitar relações inconsistentes.
11. Preservar ordem determinística.
12. Testar round-trip.

**Preservar compatibilidade com snapshots estruturais das Etapas 1 e 2.**

Não implementar ainda o formato físico definitivo `.cadcraft`.

Não implementar ZIP, backup versionado, migração de schemas ou salvamento atômico.

Esses recursos pertencem às etapas posteriores.

---

### 16. CENÁRIOS DE TESTE

Criar testes automatizados abrangendo, no mínimo:

#### Cenário A — Condutor individual

Criar dois pontos de terminação válidos e associá-los a um condutor.

Verificar identidade, propriedades e extremos.

#### Cenário B — Cabo multipolar

Criar cabo com quatro veias independentes.

Associar cada veia a um condutor distinto.

Verificar associações.

#### Cenário C — Veia não utilizada

Criar cabo com veias disponíveis e não utilizadas.

Validar representação e serialização.

#### Cenário D — Régua de bornes

Criar régua contendo múltiplos bornes.

Verificar ordenação e identificação.

#### Cenário E — Condutor sem cabo

Criar fio individual independente de cabo multipolar.

Validar integridade.

#### Cenário F — Associação lógica

Criar uma conexão lógica existente e associar explicitamente um condutor físico.

Verificar rastreabilidade sem duplicação de conectividade.

#### Cenário G — Condutor sem conexão lógica

Criar um condutor físico ainda não atribuído a uma conexão lógica, quando permitido pelo modelo.

Verificar que isso não altera redes derivadas.

#### Cenário H — Associações inválidas

Tentar associar veia de outro cabo, outro projeto ou condutor inexistente.

Verificar erros e ausência de mutação parcial.

#### Cenário I — Serialização

Serializar e reconstruir projeto contendo cabos, veias, condutores e réguas.

Verificar identidade e integridade.

#### Cenário J — Compatibilidade anterior

Desserializar snapshots das Etapas 1 e 2 sem os novos campos físicos.

Garantir compatibilidade.

---

### 17. TESTES DE REGRESSÃO

Preservar integralmente os testes das Etapas 1 e 2.

Cobrir especialmente:

* Redes derivadas.
* Conexões lógicas.
* Integridade de Project.
* Identificadores UUID v7.
* Serialização.
* Dados adulterados.
* Falhas sem mutação.
* Determinismo.
* Independência da GUI.

Testar explicitamente que adicionar ou modificar elementos físicos **não altera automaticamente as redes lógicas**.

O baseline informado ao início desta etapa é:

* 55 testes Rust.
* 15 testes PowerShell.
* Total: 70 testes automatizados.

A contagem final deverá ser obtida da execução real, não estimada.

---

### 18. DESEMPENHO E ESCALABILIDADE

Preparar a arquitetura para projetos industriais com milhares de componentes.

Evitar:

* Buscas quadráticas desnecessárias.
* Duplicação de registros.
* Reconstrução completa do agregado em operações simples.
* Clonagem de grandes estruturas para consultas.
* Relações bidirecionais inconsistentes.

Criar testes proporcionais para cenários com:

* 100 condutores.
* 1.000 condutores.
* 10.000 condutores.

Considerar também cabos com múltiplas veias e réguas com muitos bornes.

Não estabelecer tempos máximos arbitrários.

Registrar medições reais, quando executadas.

---

### 19. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_03_CONDUTORES_CABOS_BORNES.md`

Documentar:

1. Objetivos.
2. Arquitetura implementada.
3. Modelo de condutores.
4. Modelo de cabos.
5. Modelo de veias.
6. Réguas de bornes.
7. Pontos de terminação.
8. Associações lógicas e físicas.
9. Integridade referencial.
10. APIs públicas.
11. Serialização.
12. Testes.
13. Desempenho.
14. Limitações.
15. Dependências futuras.

Incluir diagramas Mermaid quando agregarem valor.

Registrar as decisões arquiteturais relevantes.

Não declarar conformidade normativa sem validação específica.

---

### 20. MEMÓRIA PERSISTENTE

Atualizar obrigatoriamente:

* `docs/internal/memory/CURRENT_STATE.md`
* `docs/internal/memory/HANDOFF.md`
* `docs/internal/memory/ARCHITECTURE.md`
* `docs/internal/memory/ROADMAP.md`

Atualizar quando necessário:

* `docs/internal/memory/KNOWN_ISSUES.md`
* `docs/internal/memory/DECISIONS.md`
* `docs/internal/memory/PROJECT_CONTEXT.md`

Criar ADR para decisões significativas.

Registrar:

**Etapa 3 — Implementada e testada automaticamente; homologação manual pendente.**

Registrar esse estado somente se as validações forem aprovadas.

Não registrar a Etapa 4 como iniciada.

---

### 21. VALIDAÇÕES OBRIGATÓRIAS

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

Quando o script já executar as validações, evitar duplicação desnecessária.

Executar os testes PowerShell existentes.

Não ignorar falhas.

Corrigir warnings.

Não remover testes anteriores.

---

### 22. REVISÃO TÉCNICA INDEPENDENTE

Utilizar um agente de revisão independente, quando disponível.

A revisão deverá examinar:

* Integridade física e lógica.
* Separação entre entidades.
* Associações.
* Tratamento de erros.
* Serialização.
* Compatibilidade com snapshots anteriores.
* Desempenho.
* Escalabilidade.
* Determinismo.
* Regressões.
* Independência da GUI.

Classificar problemas por severidade:

* Critical.
* Important.
* Minor.

Corrigir problemas Critical e Important dentro do escopo.

Não deixar problemas Minor sem avaliação.

Não afirmar que houve revisão independente caso ela não tenha sido executada.

---

### 23. POLÍTICA GIT — OBRIGATÓRIA

**Todas as operações Git modificadoras são exclusivas do usuário.**

Respeitar integralmente:

* `AGENTS.md`
* ADR-0002.
* Política de GitHub e backup.

O Codex está proibido de executar automaticamente:

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

Utilizar apenas consultas permitidas.

Ao concluir, informar:

* Branch.
* HEAD.
* Arquivos modificados.
* Arquivos novos.
* Arquivos excluídos.
* Necessidade de commit.

**Não executar staging, commit ou push.**

O usuário realizará o commit manual da Etapa 3.

O push será realizado após a Etapa 4, conforme a política de sincronização a cada duas etapas.

---

### 24. CRITÉRIOS DE ACEITAÇÃO

A Etapa 3 estará tecnicamente concluída quando:

* O modelo de condutores estiver implementado.
* Cabos e veias estiverem funcionais.
* Réguas de bornes estiverem implementadas.
* As terminações físicas estiverem representadas.
* A associação lógica/física for explícita.
* A integridade referencial estiver preservada.
* A serialização estiver funcional.
* A compatibilidade anterior estiver preservada.
* Os testes automatizados estiverem aprovados.
* As regressões das Etapas 1 e 2 estiverem aprovadas.
* A documentação estiver atualizada.
* A memória persistente estiver atualizada.
* O Git permanecer sob controle manual.

Não realizar homologação manual nesta etapa.

---

### 25. RELATÓRIO FINAL

Apresentar:

# CAD-CRAFT — ETAPA 3 CONCLUÍDA

**Estado:** concluída / parcialmente concluída / bloqueada.

**Arquitetura:** descrição da implementação.

**Arquivos:** criados, modificados e excluídos.

**Testes:** quantidade total e resultados.

**Validações:** Cargo fmt, check, test, Clippy, Git diff e scripts.

**Desempenho:** resultados medidos, quando disponíveis.

**Memória persistente:** documentos atualizados.

**Decisões:** ADRs criados ou modificados.

**Pendências:** limitações e problemas conhecidos.

**Git:** branch, HEAD e alterações pendentes.

**Próxima etapa:** Etapa 4, aguardando autorização.

Encerrar com:

**ETAPA 3 IMPLEMENTADA — AGUARDANDO REVISÃO E COMMIT MANUAL DO USUÁRIO.**

---

## RESTRIÇÕES FINAIS

Não iniciar a Etapa 4.

Não implementar editor gráfico.

Não implementar WireRoute ou WireJunction.

Não implementar renderização.

Não implementar simulação elétrica.

Não implementar ERC completo.

Não implementar exportação DXF.

Não implementar persistência física definitiva.

Não executar operações Git modificadoras.

Não declarar homologação manual.

**Executar integralmente a Etapa 3, validar, atualizar a memória persistente, produzir o relatório técnico e encerrar.**
