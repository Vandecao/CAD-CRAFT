# CAD-CRAFT — ETAPA 1

## IMPLEMENTAÇÃO DO CORE E MODELO DE DOMÍNIO ELÉTRICO

### 1. AUTORIZAÇÃO

Implementar integralmente a **Etapa 1 — Núcleo de Engenharia Elétrica e Modelo de Domínio** do CAD-CRAFT.

Diretório principal:

`C:\Dev\CAD-CRAFT`

Repositório remoto:

`https://github.com/Vandecao/CAD-CRAFT.git`

Utilizar o workspace existente, criado durante o Prompt 0.

**Não recriar o projeto, não modificar a arquitetura sem justificativa técnica e não iniciar a Etapa 2.**

Executar autonomamente as atividades autorizadas, utilizando as ferramentas de planejamento, implementação, testes e revisão disponíveis no ambiente.

### 2. RECUPERAÇÃO DO CONTEXTO

Antes de modificar qualquer arquivo:

1. Ler `AGENTS.md`.
2. Ler `docs/internal/memory/HANDOFF.md`.
3. Ler `docs/internal/memory/CURRENT_STATE.md`.
4. Ler `docs/internal/memory/PROJECT_CONTEXT.md`.
5. Ler `docs/internal/memory/ARCHITECTURE.md`.
6. Consultar `docs/internal/memory/DECISIONS.md`.
7. Consultar os ADRs existentes.
8. Examinar `docs/internal/spec/`.
9. Examinar `Cargo.toml` e os crates existentes.
10. Verificar `git status`, `git log -1` e `git remote -v`.

Tratar o código existente, os testes e os registros Git como fontes técnicas de verdade.

Não presumir funcionalidades implementadas apenas porque aparecem no roadmap.

Identificar possíveis divergências entre documentação e workspace antes de prosseguir.

### 3. OBJETIVO PRINCIPAL

Implementar um Core elétrico independente da interface gráfica, capaz de representar estruturalmente:

* Projetos.
* Documentos.
* Páginas.
* Componentes elétricos.
* Dispositivos.
* Funções.
* Terminais.
* Pontos de conexão.
* Identificadores.
* Localizações.
* Estruturas funcionais.
* Propriedades técnicas.
* Referências entre entidades.

Nesta etapa, o Core deverá ser funcional, testável e extensível, mas ainda não precisará executar operações avançadas de conectividade elétrica, geração de relatórios ou edição gráfica.

O objetivo é construir uma fundação correta, não antecipar todas as funcionalidades do CAD-CRAFT.

### 4. REQUISITOS ARQUITETURAIS

#### 4.1. Independência da GUI

O crate `cad-craft` não poderá depender de:

* egui.
* eframe.
* Componentes gráficos.
* Estado de janelas.
* Eventos de mouse.
* APIs específicas de renderização.

O Core deverá ser utilizável independentemente da interface desktop.

#### 4.2. Organização modular

Organizar o código por responsabilidade.

Estrutura conceitual sugerida:

```text
crates/cad-craft/src/
├── lib.rs
├── domain/
│   ├── mod.rs
│   ├── ids.rs
│   ├── project.rs
│   ├── document.rs
│   ├── page.rs
│   ├── component.rs
│   ├── device.rs
│   ├── function.rs
│   ├── terminal.rs
│   ├── connection_point.rs
│   ├── properties.rs
│   └── designation.rs
└── error.rs
```

A estrutura poderá ser refinada se houver justificativa técnica.

Evitar módulos vazios sem finalidade.

### 5. IDENTIFICADORES TIPADOS

Implementar identificadores fortemente tipados para as principais entidades.

Utilizar UUID v7 para identificadores persistentes, quando tecnicamente adequado.

Exemplos conceituais:

```rust
ProjectId
DocumentId
PageId
ComponentId
DeviceId
FunctionId
TerminalId
ConnectionPointId
```

Requisitos:

* Identificadores distintos não devem ser intercambiáveis acidentalmente.
* Implementar traits apropriadas.
* Permitir comparação e utilização em coleções.
* Permitir serialização e desserialização, quando aplicável.
* Centralizar a geração e validação.
* Evitar dependência da GUI.

Criar testes de unicidade, tipagem e conversão.

### 6. MODELO DE PROJETO

Implementar a entidade `Project`.

Responsabilidades:

* Identidade do projeto.
* Nome.
* Descrição.
* Metadados.
* Documentos.
* Referências às entidades pertencentes ao projeto.

Garantir que cada projeto possua identidade estável.

Criar métodos seguros para operações básicas de criação e consulta.

Evitar duplicação de entidades.

### 7. DOCUMENTOS E PÁGINAS

Implementar modelos para `Document` e `Page`.

Cada documento poderá conter múltiplas páginas.

As páginas deverão possuir:

* Identificador tipado.
* Nome.
* Número ou designação.
* Tipo.
* Metadados.
* Dimensões físicas.
* Referências aos elementos pertencentes à página.

Preparar suporte conceitual para diferentes tipos de página, como:

* Esquema multifilar.
* Esquema unifilar.
* Visão geral.
* Layout de painel.
* Documentação técnica.

Não implementar os editores correspondentes nesta etapa.

### 8. SISTEMA DE COORDENADAS

Adotar obrigatoriamente:

* Unidade interna: milímetros.
* Origem do desenho: canto inferior esquerdo.
* Eixo X positivo: direita.
* Eixo Y positivo: cima.

Criar tipos para representar:

```rust
Point2D
Size2D
Rect2D
```

Requisitos:

* Coordenadas finitas.
* Dimensões não negativas, quando aplicável.
* Operações geométricas básicas.
* Precisão adequada para desenho técnico.
* Sem dependência de renderização.

Não implementar zoom, pan ou transformação de viewport nesta etapa.

### 9. COMPONENTES ELÉTRICOS

Implementar a entidade `Component`.

Um componente deverá possuir:

* Identificador.
* Designação.
* Tipo ou classificação.
* Propriedades técnicas.
* Referência opcional ao fabricante.
* Referência opcional ao código de catálogo.
* Relação com dispositivo e função, quando aplicável.

Separar claramente a identidade do componente de sua futura representação gráfica.

Não acoplar o modelo elétrico a símbolos visuais.

### 10. DISPOSITIVOS E FUNÇÕES

Implementar entidades para representar dispositivos e funções elétricas.

Um dispositivo poderá possuir múltiplas funções.

Exemplo conceitual:

Um contator poderá possuir:

* Bobina.
* Contatos auxiliares.
* Contatos principais.

Essas funções poderão futuramente aparecer em páginas diferentes.

Requisitos:

* Identidade persistente.
* Relação explícita entre dispositivo e funções.
* Designação técnica.
* Classificação funcional.
* Propriedades específicas.
* Integridade referencial.

Preparar a arquitetura para referências cruzadas futuras.

Não implementar ainda o mecanismo completo de navegação entre referências.

### 11. TERMINAIS E PONTOS DE CONEXÃO

Implementar modelos básicos para terminais e pontos de conexão.

Distinguir:

* Terminal físico.
* Ponto de conexão funcional.
* Representação gráfica futura.

Requisitos:

* Identificadores tipados.
* Designação.
* Associação com entidades proprietárias.
* Propriedades técnicas.
* Estrutura extensível para conectividade.

**Não implementar nesta etapa:**

* Roteamento gráfico.
* WireRoute.
* WireJunction.
* Cálculo de redes.
* Condutores físicos.
* Algoritmos de conectividade.
* ERC.

Esses mecanismos serão introduzidos nas etapas apropriadas.

### 12. PROPRIEDADES TÉCNICAS

Implementar um mecanismo de propriedades tipadas.

Considerar tipos como:

* Texto.
* Inteiro.
* Decimal.
* Booleano.
* Enumerações controladas.
* Referências a entidades.

Evitar utilizar exclusivamente mapas genéricos de strings para representar informações estruturais importantes.

Separar propriedades conhecidas do domínio de propriedades personalizadas.

Preparar a arquitetura para extensibilidade sem comprometer a tipagem forte.

### 13. DESIGNAÇÕES ELÉTRICAS

Preparar um modelo para designações estruturadas.

Considerar:

* Identificação funcional.
* Localização.
* Designação do dispositivo.
* Designação do terminal.

A arquitetura deverá permitir evolução para convenções industriais compatíveis com a família IEC 81346.

Não implementar um interpretador completo da norma nesta etapa.

Não assumir conformidade normativa sem testes específicos.

### 14. INTEGRIDADE DO MODELO

Implementar verificações básicas para:

* Identificadores duplicados.
* Referências inexistentes.
* Relações inválidas.
* Entidades pertencentes a projetos diferentes.
* Designações inválidas.
* Dimensões inválidas.

Definir tratamento de erros consistente.

Evitar panics para erros esperados de entrada.

Utilizar `Result` e tipos de erro apropriados.

Não criar um sistema ERC completo nesta etapa.

### 15. SERIALIZAÇÃO

Preparar as entidades do domínio para serialização estruturada.

Utilizar Serde, quando adequado.

Requisitos:

* Tipos serializáveis.
* Identificadores preservados.
* Estruturas consistentes.
* Testes de round-trip.
* Separação entre modelo de domínio e formato físico de armazenamento.

**Não implementar ainda o formato definitivo `.cadcraft`.**

Não implementar arquivos ZIP, migração de schemas ou salvamento atômico nesta etapa.

Esses mecanismos pertencem à fase de persistência.

### 16. TESTES AUTOMATIZADOS

Criar uma suíte de testes abrangente.

Cobrir, no mínimo:

1. Criação de projetos.
2. Criação de documentos.
3. Criação de páginas.
4. Integridade de identificadores.
5. Unicidade.
6. Associação entre entidades.
7. Dispositivos com múltiplas funções.
8. Terminais e pontos de conexão.
9. Propriedades tipadas.
10. Coordenadas e dimensões.
11. Integridade referencial.
12. Serialização.
13. Desserialização.
14. Entradas inválidas.
15. Tratamento de erros.

Testar também cenários negativos.

Evitar testes artificiais que apenas confirmem valores constantes.

Os testes deverão validar comportamento real.

### 17. VALIDAÇÕES OBRIGATÓRIAS

Ao concluir a implementação, executar:

```powershell
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Utilizar `--locked` nos comandos Cargo compatíveis, preservando a política de dependências do projeto.

Se necessário, executar validações adicionais.

Corrigir os problemas identificados.

Não ignorar warnings.

Não remover funcionalidades ou testes para obter aprovação.

### 18. DOCUMENTAÇÃO

Criar:

`docs/internal/reports/ETAPA_01_CORE_DOMINIO.md`

Documentar:

* Objetivos.
* Arquitetura implementada.
* Entidades criadas.
* Relações entre entidades.
* Identificadores.
* Sistema de coordenadas.
* Tipos geométricos.
* Sistema de propriedades.
* Regras de integridade.
* Serialização.
* Testes.
* Limitações.
* Próximas dependências arquiteturais.

Criar diagramas Mermaid, quando úteis.

Não adicionar documentação redundante sem necessidade.

### 19. MEMÓRIA PERSISTENTE

Atualizar obrigatoriamente:

* `docs/internal/memory/CURRENT_STATE.md`
* `docs/internal/memory/HANDOFF.md`
* `docs/internal/memory/ARCHITECTURE.md`
* `docs/internal/memory/ROADMAP.md`
* `docs/internal/memory/KNOWN_ISSUES.md`, se necessário.
* `docs/internal/memory/DECISIONS.md`, se houver decisões novas.

Registrar:

**Etapa 1 — Implementada e testada automaticamente. Homologação manual pendente.**

Somente registrar esse estado se todos os critérios técnicos aplicáveis forem atendidos.

Informar:

* Arquivos implementados.
* Módulos criados.
* Quantidade de testes aprovados.
* Pendências.
* Decisões tomadas.
* Próxima etapa prevista.

Não registrar a Etapa 2 como iniciada.

### 20. POLÍTICA GIT — OBRIGATÓRIA

Respeitar integralmente o `AGENTS.md` e o ADR de controle manual do Git.

**É proibido executar automaticamente:**

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
* Operações equivalentes que alterem estado, histórico ou sincronização.

São permitidas consultas de leitura, conforme definido no `AGENTS.md`.

Ao concluir, apresentar os arquivos modificados e informar que estão prontos para revisão e commit manual.

Não executar nenhuma operação Git modificadora.

### 21. REVISÃO TÉCNICA

Antes de encerrar:

1. Revisar a separação Core/GUI.
2. Procurar acoplamentos indevidos.
3. Verificar integridade referencial.
4. Revisar o tratamento de erros.
5. Revisar consistência dos identificadores.
6. Verificar serialização.
7. Confirmar que os testes cobrem os principais cenários.
8. Identificar possíveis regressões.
9. Confirmar que nenhuma funcionalidade da Etapa 2 foi antecipada desnecessariamente.
10. Confirmar que nenhuma operação Git modificadora foi realizada.

Corrigir problemas encontrados dentro do escopo.

### 22. CRITÉRIOS DE ACEITAÇÃO

A Etapa 1 estará tecnicamente concluída quando:

* O Core estiver implementado.
* As entidades básicas existirem.
* O modelo preservar integridade referencial.
* Os identificadores forem fortemente tipados.
* O sistema de coordenadas estiver definido.
* As propriedades tipadas estiverem funcionais.
* A serialização básica estiver operacional.
* Os testes automatizados estiverem aprovados.
* A compilação ocorrer sem erros.
* O Clippy estiver aprovado.
* A documentação estiver atualizada.
* A memória persistente estiver atualizada.
* O Git permanecer sob controle manual.

Não exigir homologação manual nesta etapa.

### 23. RELATÓRIO FINAL

Apresentar:

# CAD-CRAFT — ETAPA 1 CONCLUÍDA

**Estado:** concluída / parcialmente concluída / bloqueada.

**Arquitetura:** resumo das implementações.

**Arquivos:** criados, modificados e excluídos.

**Testes:** quantidade total e resultados.

**Validações:** Cargo fmt, check, test, Clippy e Git diff.

**Memória persistente:** arquivos atualizados.

**Pendências:** problemas conhecidos e limitações.

**Git:** branch, HEAD e alterações pendentes, sem executar operações modificadoras.

**Próxima etapa:** Etapa 2 — aguardando autorização.

Encerrar com:

**ETAPA 1 IMPLEMENTADA — AGUARDANDO REVISÃO E COMMIT MANUAL DO USUÁRIO.**

---

## RESTRIÇÕES FINAIS

Não iniciar a Etapa 2.

Não executar operações Git modificadoras.

Não substituir a arquitetura do bootstrap.

Não adicionar dependências desnecessárias.

Não implementar funcionalidades gráficas avançadas.

Não implementar roteamento elétrico.

Não implementar ERC completo.

Não implementar o formato definitivo `.cadcraft`.

Não declarar homologação manual.

**Executar integralmente a Etapa 1, validar, atualizar a memória persistente, apresentar o relatório e encerrar.**
