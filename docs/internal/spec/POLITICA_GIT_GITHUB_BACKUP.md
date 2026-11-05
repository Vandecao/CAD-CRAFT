# CAD-CRAFT — INSTRUÇÃO COMPLEMENTAR

## POLÍTICA PERMANENTE DE GIT, GITHUB, ARMAZENAMENTO E BACKUP

### 1. OBJETIVO

Implementar uma política permanente de gerenciamento do repositório CAD-CRAFT, garantindo:

* Controle do armazenamento.
* Versionamento organizado.
* Segurança dos arquivos.
* Proteção contra perda de dados.
* Continuidade entre sessões do Codex.
* Memória técnica persistente.
* Controle exclusivamente manual das operações Git que alteram o histórico ou sincronizam o repositório.

**Esta instrução complementa o Prompt 0 e deverá ser executada somente após sua conclusão.**

Diretório principal:

`C:\Dev\CAD-CRAFT`

Repositório remoto oficial:

`https://github.com/Vandecao/CAD-CRAFT.git`

Branch principal:

`main`

**Não iniciar a Etapa 1.**

---

### 2. POLÍTICA DE ARMAZENAMENTO DO GITHUB

Estabelecer as seguintes diretrizes permanentes:

1. Manter o repositório Git preferencialmente abaixo de 1 GB.
2. Considerar 10 GB como referência operacional de tamanho, não como limite rígido garantido pelo GitHub.
3. Evitar versionar arquivos binários grandes.
4. Não armazenar artefatos de compilação.
5. Não armazenar caches e arquivos temporários.
6. Não armazenar máquinas virtuais ou imagens de disco.
7. Não armazenar credenciais, tokens ou chaves privadas.
8. Não armazenar logs extensos.
9. Não versionar backups completos do projeto dentro do próprio repositório.
10. Preservar arquivos necessários à compilação, documentação e reprodução do ambiente.

Arquivos individuais superiores a 50 MB deverão ser identificados e avaliados antes do commit.

Arquivos superiores a 100 MB não deverão ser adicionados ao Git convencional.

Git LFS poderá ser considerado futuramente, mediante autorização expressa do usuário.

---

### 3. CONFIGURAÇÃO DO .gitignore

Auditar o `.gitignore` existente.

Garantir que arquivos desnecessários não sejam incluídos no versionamento.

Utilizar as seguintes regras como referência:

```gitignore
# Rust
/target/
**/target/

# Compilação e depuração
*.pdb
*.ilk
*.obj
*.o
*.dmp

# Logs e temporários
*.log
*.tmp
*.bak
*.swp

# Credenciais
.env
.env.*
!.env.example
*.pem
*.key

# IDE
.vs/
.idea/

# Máquinas virtuais
*.vmdk
*.vdi
*.vhd
*.vhdx
*.vmem
*.vmss
```

**Atenção:** antes de adicionar regras, verificar se elas não excluem arquivos legítimos do projeto.

Não excluir indiscriminadamente executáveis, bibliotecas, símbolos, macros ou outros recursos necessários ao funcionamento do CAD-CRAFT.

Os seguintes arquivos deverão permanecer versionáveis:

* `Cargo.toml`
* `Cargo.lock`
* `AGENTS.md`
* `README.md`
* `docs/internal/spec/`
* `docs/internal/memory/`
* `docs/internal/reports/`
* Código-fonte Rust.
* Testes automatizados.
* Scripts de desenvolvimento.
* Configurações de build.
* Recursos necessários à aplicação.

---

### 4. POLÍTICA OBRIGATÓRIA DE VERSIONAMENTO

**DECISÃO PERMANENTE DO PROJETO: TODAS AS OPERAÇÕES GIT DE ALTERAÇÃO DO HISTÓRICO E SINCRONIZAÇÃO SERÃO EXECUTADAS MANUALMENTE PELO USUÁRIO.**

O Codex está expressamente proibido de executar automaticamente:

```bash
git add
git commit
git push
git pull
git fetch
git merge
git rebase
git reset
git revert
git cherry-pick
git checkout
git switch
git branch -d
git branch -D
git clean
git stash
git tag
```

A restrição também se aplica a comandos equivalentes executados por:

* Scripts PowerShell.
* Scripts Python.
* Hooks.
* Subagentes.
* Ferramentas externas.
* Automações.
* Interfaces gráficas.
* Processos de CI/CD.

Não contornar essas restrições utilizando comandos alternativos.

**Exceção:** somente mediante autorização explícita do usuário para uma operação específica.

Essa autorização não deve ser interpretada como permissão permanente para operações futuras.

#### 4.1. Operações permitidas

O Codex poderá utilizar comandos exclusivamente de consulta e diagnóstico, como:

```powershell
git status
git diff
git diff --check
git log
git remote -v
git branch --show-current
git ls-files
```

Outros comandos de leitura poderão ser utilizados desde que não modifiquem arquivos, referências ou o estado do repositório.

#### 4.2. Responsabilidade do Codex

Ao concluir cada etapa, o Codex deverá:

1. Implementar o escopo autorizado.
2. Executar os testes automatizados.
3. Corrigir os problemas encontrados.
4. Atualizar a documentação.
5. Atualizar a memória persistente.
6. Gerar o relatório técnico.
7. Verificar o estado do Git.
8. Apresentar os arquivos criados, modificados e excluídos.
9. Informar que a etapa está pronta para revisão.
10. Encerrar a execução.

**Não executar commit ou push.**

#### 4.3. Responsabilidade do usuário

O usuário executará manualmente:

* Staging.
* Commits.
* Push.
* Pull, quando necessário.
* Merge.
* Resolução de conflitos.
* Operações de manutenção do histórico Git.

Essas operações serão realizadas com orientação externa ao Codex.

---

### 5. FREQUÊNCIA DE COMMITS E PUSH

Estabelecer o seguinte procedimento:

| Evento                      | Procedimento                      |
| --------------------------- | --------------------------------- |
| Conclusão do bootstrap      | Commit + push manual              |
| Conclusão da Etapa 1        | Commit manual                     |
| Conclusão da Etapa 2        | Commit + push manual              |
| Conclusão da Etapa 3        | Commit manual                     |
| Conclusão da Etapa 4        | Commit + push manual              |
| Etapas seguintes            | Repetir o padrão                  |
| Conclusão da Etapa 14       | Commit + push manual              |
| Conclusão da Etapa 15       | Commit + push manual              |
| Homologação após a Etapa 15 | Correções e novos commits manuais |

O Codex deverá apenas informar quando uma operação Git for necessária.

**Nunca executá-la automaticamente.**

---

### 6. MEMÓRIA PERSISTENTE

Registrar integralmente esta política na infraestrutura de memória do CAD-CRAFT.

#### 6.1. AGENTS.md

Adicionar uma seção obrigatória:

`POLÍTICA DE GIT, GITHUB E BACKUP`

Essa seção deverá:

* Declarar a proibição de operações Git modificadoras sem autorização.
* Definir as operações permitidas.
* Registrar a frequência de commits e push.
* Estabelecer as regras de armazenamento.
* Definir o protocolo de encerramento das etapas.
* Instruir novas sessões a consultar a política antes de executar comandos Git.

#### 6.2. docs/internal/memory/DECISIONS.md

Registrar uma decisão arquitetural e operacional:

**ADR — Controle manual do versionamento Git**

Contexto:

O ambiente anterior do CAD-CRAFT sofreu uma falha grave, ocasionando perda do código-fonte e do histórico Git local.

Decisão:

O desenvolvimento será automatizado pelo Codex, mas todas as operações de versionamento e sincronização permanecerão sob controle manual do usuário.

Justificativa:

* Evitar alterações não autorizadas no histórico.
* Preservar rastreabilidade.
* Reduzir riscos de sobrescrita.
* Garantir revisão antes de cada commit.
* Proteger o projeto por meio de sincronização remota periódica.

Consequências:

* O Codex não executará commits.
* O Codex não executará push.
* O Codex informará quando houver alterações prontas.
* O usuário realizará as operações Git manualmente.

Criar o ADR correspondente em:

`docs/adr/`

#### 6.3. docs/internal/memory/PROJECT_CONTEXT.md

Registrar:

* GitHub como repositório remoto oficial.
* Política de armazenamento.
* Estratégia de proteção do código-fonte.
* Dependência de sincronização remota para recuperação de commits.

#### 6.4. docs/internal/memory/CURRENT_STATE.md

Registrar e atualizar:

* Branch atual.
* Último commit local confirmado.
* Último commit remoto confirmado, quando verificável.
* Etapa atual.
* Etapas concluídas.
* Etapas pendentes de commit.
* Commits pendentes de push.
* Estado da sincronização com o GitHub.
* Pendências de backup.

Não inventar estados ou hashes.

#### 6.5. docs/internal/memory/HANDOFF.md

Adicionar instruções para novas sessões:

Antes de iniciar qualquer etapa:

1. Ler `AGENTS.md`.
2. Ler `docs/internal/memory/HANDOFF.md`.
3. Ler `docs/internal/memory/CURRENT_STATE.md`.
4. Consultar a política Git.
5. Verificar o estado do repositório.
6. Confirmar a etapa autorizada.

Ao concluir:

1. Executar validações.
2. Atualizar documentação.
3. Atualizar memória persistente.
4. Apresentar relatório.
5. Informar se há necessidade de commit ou push.
6. Encerrar sem executar operações Git modificadoras.

---

### 7. VERIFICAÇÃO AUTOMÁTICA DO REPOSITÓRIO

Criar:

`scripts/check-repository.ps1`

O script deverá ser exclusivamente de diagnóstico.

Verificações obrigatórias:

1. Identificar branch atual.
2. Identificar último commit local.
3. Identificar referência remota de acompanhamento, se disponível.
4. Informar commits pendentes de push quando os dados locais forem suficientes.
5. Identificar arquivos rastreados superiores a 50 MB.
6. Verificar arquivos temporários indevidamente rastreados.
7. Verificar integridade estrutural de `AGENTS.md`.
8. Verificar existência de `Cargo.lock`.
9. Verificar existência de `docs/internal/memory/`.
10. Verificar configuração do `.gitignore`.
11. Informar possíveis arquivos sensíveis, sem expor seu conteúdo.
12. Apresentar relatório legível no PowerShell.

O script deverá funcionar sem conexão à internet.

Quando não for possível confirmar o estado remoto, informar:

`ESTADO REMOTO NÃO VERIFICADO`

**Proibições:**

* Não executar `git add`.
* Não executar `git commit`.
* Não executar `git push`.
* Não executar `git fetch`.
* Não executar `git pull`.
* Não excluir arquivos.
* Não modificar referências Git.
* Não alterar o histórico.
* Não realizar limpeza automática.

---

### 8. POLÍTICA DE SEGURANÇA E RECUPERAÇÃO

Documentar em:

`docs/internal/reports/POLITICA_GITHUB_BACKUP.md`

Incluir:

* Estratégia de commits.
* Estratégia de push.
* Recuperação por clone.
* Reconstrução do ambiente.
* Instalação de dependências.
* Recuperação da memória persistente.
* Restauração de uma versão anterior.
* Verificação de integridade.
* Procedimentos para continuar o desenvolvimento após perda de uma máquina virtual.

Registrar que o GitHub protege somente o conteúdo efetivamente enviado ao repositório remoto.

Arquivos não versionados e commits não enviados continuam sujeitos a perdas locais.

Recomendar backup externo independente, sem implementá-lo automaticamente nesta etapa.

---

### 9. VALIDAÇÕES

Após implementar a política:

1. Verificar a integridade do `AGENTS.md`.
2. Verificar a memória persistente.
3. Verificar os ADRs.
4. Validar o script PowerShell.
5. Verificar o `.gitignore`.
6. Confirmar que nenhum arquivo necessário foi excluído.
7. Confirmar que nenhuma operação Git modificadora foi executada.
8. Confirmar que nenhuma credencial foi exposta.
9. Executar `git diff --check`.
10. Executar as demais validações aplicáveis ao estado do projeto.

Registrar somente resultados efetivamente observados.

---

### 10. RELATÓRIO FINAL

Criar:

`docs/internal/reports/POLITICA_GITHUB_BACKUP.md`

Apresentar ao usuário:

**CAD-CRAFT — POLÍTICA DE GIT E BACKUP IMPLEMENTADA**

Informar:

* Arquivos criados.
* Arquivos modificados.
* Regras adicionadas ao AGENTS.md.
* Memórias atualizadas.
* ADR criado.
* Script implementado.
* Resultado das validações.
* Estado atual do Git.
* Arquivos pendentes de versionamento.
* Pendências identificadas.

Encerrar com:

**IMPLEMENTAÇÃO CONCLUÍDA — AGUARDANDO COMMIT E PUSH MANUAIS DO USUÁRIO.**

---

### 11. RESTRIÇÕES FINAIS

Não iniciar a Etapa 1.

Não implementar funcionalidades elétricas ou gráficas.

Não instalar Claude Code.

Não criar outro repositório.

Não modificar o remote `origin`.

Não executar Git automaticamente para alterar o histórico ou sincronizar o repositório.

Não executar commit.

Não executar push.

Não executar pull.

Não executar merge.

Não executar reset.

Não executar operações destrutivas.

Não sobrescrever decisões arquiteturais existentes.

Não declarar homologação manual sem confirmação.

**REGRA PERMANENTE: O CODEX DESENVOLVE, TESTA, DOCUMENTA E REGISTRA A MEMÓRIA. O USUÁRIO CONTROLA EXCLUSIVAMENTE O VERSIONAMENTO E A SINCRONIZAÇÃO GIT.**

Essa regra deverá permanecer válida durante todas as etapas futuras do CAD-CRAFT, inclusive após a Etapa 15.
