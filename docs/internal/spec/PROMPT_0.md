# CAD-CRAFT — PROMPT 0

## RECONSTRUÇÃO DO AMBIENTE, BOOTSTRAP E MEMÓRIA PERSISTENTE

### 1. AUTORIZAÇÃO

Você é o agente principal de desenvolvimento do CAD-CRAFT, um software profissional de engenharia elétrica industrial para Windows.

Este projeto está sendo reconstruído após a perda do ambiente anterior.

Sua missão inicial é preparar integralmente o ambiente de desenvolvimento e criar a infraestrutura documental e operacional necessária para executar as Etapas 1 a 15.

**NÃO implemente ainda as funcionalidades da Etapa 1.**

Execute autonomamente as tarefas autorizadas neste prompt, respeitando as permissões do ambiente.

Utilize o maior nível de raciocínio disponível na sessão.

### 2. IDENTIDADE DO PROJETO

Nome: CAD-CRAFT

Diretório principal:

`C:\Dev\cad-craft`

Plataforma-alvo: Windows desktop.

Tecnologias principais:

* Rust.
* Cargo.
* egui/eframe.
* Git.
* GitHub.
* PowerShell.
* Visual Studio Code.

Arquitetura desejada:

* Workspace Rust.
* Core independente da interface gráfica.
* Interface desktop nativa.
* Modularidade.
* Tipagem forte.
* Testabilidade.
* Persistência versionada.
* Baixo acoplamento.
* Separação entre dados elétricos e representações gráficas.

O software deverá oferecer funcionalidades profissionais de ECAD industrial inspiradas funcionalmente no EPLAN Electric P8.

Não copiar código proprietário ou utilizar bibliotecas comerciais sem autorização.

### 3. OBJETIVO DO BOOTSTRAP

Preparar um ambiente de desenvolvimento reproduzível, verificável e documentado.

Esta fase deverá:

1. Auditar o ambiente Windows.
2. Identificar ferramentas instaladas.
3. Instalar dependências ausentes, quando autorizado.
4. Verificar a toolchain Rust.
5. Configurar o workspace inicial.
6. Preparar Git e integração com GitHub.
7. Criar o AGENTS.md.
8. Criar a infraestrutura de memória persistente.
9. Registrar decisões arquiteturais.
10. Preparar o roadmap das Etapas 1 a 15.
11. Documentar o procedimento de backup e recuperação.
12. Validar a infraestrutura criada.

### 4. AUDITORIA DO AMBIENTE

Verificar:

* Sistema operacional.
* Arquitetura x64.
* PowerShell.
* Rust e Cargo.
* Rustup e toolchains.
* Git.
* GitHub CLI.
* VS Code.
* Microsoft C++ Build Tools.
* Windows SDK.
* Compilador MSVC.
* Acesso ao GitHub.
* Espaço disponível em disco.
* Variáveis de ambiente necessárias.

Instalar somente o que estiver ausente.

Preferir ferramentas oficiais.

Não modificar configurações de segurança do Windows sem necessidade e autorização.

Registrar versões instaladas.

Criar relatório:

`docs/internal/reports/BOOTSTRAP_AMBIENTE.md`

### 5. CONFIGURAÇÃO DO WORKSPACE

Criar a estrutura inicial:

```text
C:\Dev\cad-craft\
│
├── AGENTS.md
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── README.md
│
├── crates\
│   ├── cad-craft\
│   └── cad-gui\
│
├── docs\
│   ├── spec\
│   ├── reports\
│   ├── plans\
│   ├── memory\
│   │   └── adr\
│   └── references\
│
├── scripts\
│
└── tests\
```

Os diretórios poderão ser refinados conforme as necessidades técnicas das etapas.

Criar um workspace mínimo compilável.

Não implementar funcionalidades do editor nesta fase.

### 6. ARQUITETURA PRINCIPAL

Registrar as seguintes decisões:

**Core**

O Core deverá conter a lógica de engenharia elétrica, sem dependência da interface gráfica.

Módulos planejados:

* domain
* graph
* document
* library
* history
* xref
* persistence
* reports
* electrical
* connectivity
* validation

**GUI**

A interface será construída com egui/eframe.

Módulos planejados:

* ribbon
* canvas
* navigation
* properties
* dialogs
* editors
* interaction
* tools

Não criar antecipadamente implementações vazias desnecessárias.

Estabelecer somente os contratos arquiteturais iniciais.

### 7. DIRETRIZES TÉCNICAS OBRIGATÓRIAS

Preservar as decisões do desenvolvimento anterior:

1. Coordenadas internas em milímetros.
2. Origem do desenho no canto inferior esquerdo.
3. Eixo X positivo para a direita.
4. Eixo Y positivo para cima.
5. Identificadores tipados utilizando UUID v7, conforme aplicável.
6. Separação entre componentes, dispositivos e representações.
7. Separação entre conexões lógicas e condutores físicos.
8. Redes elétricas derivadas da conectividade.
9. Modelo de dados independente da GUI.
10. Histórico de operações com Undo/Redo transacional.
11. Persistência versionada.
12. Migração de schemas.
13. Testes automatizados.
14. Exportação DXF.
15. Preparação arquitetural para DWG, PDF e XLSX.
16. Referências cruzadas.
17. Biblioteca de símbolos elétricos.
18. Engenharia de bornes, cabos e conectores.
19. Relatórios elétricos.
20. Verificação de regras elétricas — ERC.

Não implementar essas funcionalidades no bootstrap.

### 8. MEMÓRIA PERSISTENTE

Criar:

```text
docs/internal/memory/
├── README.md
├── PROJECT_CONTEXT.md
├── ARCHITECTURE.md
├── CURRENT_STATE.md
├── ROADMAP.md
├── KNOWN_ISSUES.md
├── DECISIONS.md
├── HANDOFF.md
└── adr/
```

A memória será armazenada dentro do repositório Git.

Não depender do histórico de conversas do Codex.

Não depender de APIs externas, bancos vetoriais ou serviços de memória de terceiros.

Cada documento deverá possuir responsabilidade específica.

#### PROJECT_CONTEXT.md

Registrar visão geral, objetivos, público-alvo, tecnologias e escopo funcional.

#### ARCHITECTURE.md

Documentar módulos, separação Core/GUI, contratos, dependências e decisões estruturais.

#### CURRENT_STATE.md

Registrar:

* Etapa atual.
* Última etapa implementada.
* Última etapa testada.
* Última etapa homologada.
* Branch.
* Último commit.
* Estado do build.
* Quantidade de testes.
* Pendências.

O estado inicial deverá indicar:

**BOOTSTRAP EM EXECUÇÃO — ETAPA 1 NÃO INICIADA.**

#### ROADMAP.md

Registrar o planejamento das Etapas 1 a 15.

Não inventar detalhes históricos ainda não fornecidos.

O roadmap deverá permitir atualizações posteriores, conforme forem recebidos os prompts de cada etapa.

#### KNOWN_ISSUES.md

Criar registro estruturado de problemas.

Não marcar como corrigidos problemas que ainda não foram testados.

#### DECISIONS.md

Criar índice das decisões arquiteturais.

#### HANDOFF.md

Criar documento resumido contendo tudo o que outro agente precisa saber para continuar o desenvolvimento.

#### adr/

Registrar decisões arquiteturais significativas no formato ADR.

### 9. AGENTS.md

Criar um arquivo AGENTS.md completo na raiz do projeto.

Ele deverá instruir qualquer nova sessão do Codex a:

1. Ler AGENTS.md.
2. Ler docs/internal/memory/HANDOFF.md.
3. Ler docs/internal/memory/CURRENT_STATE.md.
4. Consultar ARCHITECTURE.md quando necessário.
5. Verificar Git.
6. Identificar a etapa autorizada.
7. Inspecionar código existente.
8. Planejar a implementação.
9. Executar somente a etapa autorizada.
10. Executar validações.
11. Atualizar a memória persistente.
12. Gerar relatório técnico.
13. Encerrar a sessão sem iniciar etapas adicionais.

Evitar carregar todos os documentos em cada sessão.

Recuperar contexto seletivamente.

### 10. POLÍTICA DE DESENVOLVIMENTO

As Etapas 1 a 15 serão executadas sequencialmente.

Cada etapa deverá:

* Respeitar a arquitetura existente.
* Preservar funcionalidades anteriores.
* Implementar integralmente o escopo autorizado.
* Criar testes automatizados.
* Executar validações.
* Atualizar a documentação.
* Atualizar a memória persistente.
* Gerar relatório técnico.

O usuário optou por adiar as homologações manuais até o final da Etapa 15.

Isso não dispensa testes automatizados.

Nunca declarar homologação manual sem confirmação expressa.

### 11. POLÍTICA GIT

Novo procedimento:

**Commit obrigatório após cada etapa concluída e aprovada automaticamente.**

**Push para o GitHub a cada duas etapas.**

Os commits deverão possuir mensagens descritivas.

Não utilizar comandos destrutivos de Git sem autorização.

Não utilizar `git reset --hard` indiscriminadamente.

Não utilizar `git push --force`.

Não sobrescrever branches remotas.

Antes de sincronizar, verificar o estado do repositório remoto.

Não realizar pull automaticamente sobre alterações locais não consolidadas.

Criar uma branch principal estável.

Configurar integração com GitHub utilizando autenticação oficial.

Não armazenar tokens ou credenciais no repositório.

Se a autenticação exigir interação humana, solicitar exclusivamente essa intervenção.

Na primeira sincronização, garantir que o repositório remoto receba uma cópia íntegra do bootstrap.

### 12. POLÍTICA DE TESTES

Ao final de cada etapa de desenvolvimento, executar:

```powershell
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Executar também testes específicos correspondentes às funcionalidades implementadas.

Não ignorar falhas.

Não remover testes para obter aprovação artificial.

Registrar os resultados reais.

### 13. SEGURANÇA E BACKUP

Como o ambiente anterior foi perdido, implementar medidas preventivas.

Documentar:

* Estratégia de commits.
* Estratégia de push.
* Recuperação a partir do GitHub.
* Reconstrução do ambiente.
* Dependências.
* Toolchains.
* Procedimento de clone.
* Arquivos que não devem ser versionados.
* Procedimento de restauração do projeto.

Não armazenar segredos no Git.

Não criar dependência de arquivos existentes somente em uma máquina virtual.

### 14. PREPARAÇÃO PARA AS ETAPAS 1 A 15

Criar uma estrutura de planejamento capaz de receber os prompts das etapas individuais.

Não presumir que os prompts originais estejam disponíveis no disco.

Os requisitos serão fornecidos progressivamente.

A memória deverá distinguir:

* Planejado.
* Implementado.
* Testado automaticamente.
* Homologado manualmente.
* Pendente.

Não registrar funcionalidades futuras como implementadas.

### 15. VALIDAÇÃO DO BOOTSTRAP

Verificar:

* Ambiente Rust funcional.
* Workspace compilável.
* Cargo configurado.
* Git operacional.
* AGENTS.md acessível.
* Documentação criada.
* Memória persistente íntegra.
* HANDOFF funcional.
* Repositório preparado para sincronização.
* Ausência de credenciais versionadas.

Executar os testes aplicáveis.

Registrar evidências reais.

### 16. RELATÓRIO FINAL

Ao concluir, apresentar:

**CAD-CRAFT — BOOTSTRAP CONCLUÍDO**

Informar:

1. Ferramentas instaladas e versões.
2. Ferramentas ausentes.
3. Estrutura criada.
4. Arquitetura configurada.
5. Memória persistente criada.
6. AGENTS.md criado.
7. Git configurado.
8. Estado da conexão com GitHub.
9. Resultado das validações.
10. Problemas encontrados.
11. Pendências.
12. Próximos passos.

Criar:

`docs/internal/reports/BOOTSTRAP_AMBIENTE.md`

Atualizar:

`docs/internal/memory/CURRENT_STATE.md`

Atualizar:

`docs/internal/memory/HANDOFF.md`

### 17. RESTRIÇÕES FINAIS

NÃO iniciar a Etapa 1.

NÃO implementar funcionalidades do CAD elétrico.

NÃO iniciar etapas futuras.

NÃO declarar testes inexistentes.

NÃO realizar operações destrutivas sobre discos ou repositórios existentes.

NÃO sobrescrever backups anteriores.

NÃO instalar Claude Code.

NÃO utilizar agentes externos sem autorização.

NÃO gerar credenciais.

NÃO assumir que documentos de planejamento representam código implementado.

Encerrar após entregar o relatório de preparação.

**Aguardar o prompt específico da Etapa 1.**
