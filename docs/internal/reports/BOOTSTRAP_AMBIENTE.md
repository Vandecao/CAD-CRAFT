# CAD-CRAFT — Relatório do bootstrap

Referência: 2026-10-26, America/Sao_Paulo. Prompt 0.
Etapa 1 não iniciada. Nenhuma funcionalidade elétrica implementada.

## Auditoria e ferramentas

| Item | Evidência observada | Ação |
|---|---|---|
| Windows | Windows 10 Pro 10.0.19045, 64 bits | Preservado |
| Disco C na auditoria | 145.835.851.776 bytes livres, aproximadamente 135,8 GiB | Espaço suficiente para bootstrap |
| PowerShell | 7.6.5 | Já disponível |
| Rust | rustc 1.99.1, host x86_64-pc-windows-msvc | Stable já instalado; toolchain nominal 1.99.1 instalada e fixada |
| Cargo | 1.99.1 | Já disponível |
| Rustup | 1.29.1 | Já disponível |
| Componentes Rust | rustfmt e clippy | Disponíveis na toolchain fixada |
| Git | 2.56.0.windows.3 | Já disponível |
| GitHub CLI | 2.103.3 | Já disponível |
| VS Code | 1.141.2 x64 | Já disponível |
| C++ Build Tools | Visual Studio 2022 17.14.42, instalação 17.14.37806.7 | Ausente, instalado por winget oficial |
| MSVC | cl 19.44.35229 x64; diretório de ferramentas 14.44.35207 | Instalado e linkedição comprovada |
| Windows SDK | 10.0.26100.0 | Instalado com workload C++ |

Instalação usada: pacote Microsoft.VisualStudio.2022.BuildTools com
`--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended`, sem reinício
automático. Não foram atualizadas ferramentas já existentes.
Não há dependências obrigatórias conhecidas ausentes após validação.

Rust/Cargo/Git/gh/code encontrados pelo PATH. CARGO_HOME e RUSTUP_HOME não exigem
overrides: Rustup identificou `%USERPROFILE%\.rustup`; executáveis Rust
em `%USERPROFILE%\.cargo\bin`. Variáveis VCToolsInstallDir,
WindowsSdkDir e VSINSTALLDIR não estavam definidas no shell comum; Cargo
descobriu a instalação MSVC automaticamente, conforme build efetivo.

## Estrutura e arquitetura

Workspace com crates cad-craft (Core documental) e cad-gui (diagnóstico de console).
Core não depende da GUI. GUI depende do Core por caminho local.
egui/eframe permanece escolha arquitetural futura, sem código de interface ou
dependência prematura. Módulos futuros e contratos em ARCHITECTURE e ADR-0001.
Manifests, Cargo.lock, rust-toolchain.toml, gitignore, gitattributes e README
versionados; scripts/validate.ps1 interrompe na primeira falha.

AGENTS.md criado com retomada seletiva e política de etapas. Memória contém
README, PROJECT_CONTEXT, ARCHITECTURE, CURRENT_STATE, ROADMAP, KNOWN_ISSUES,
DECISIONS, HANDOFF e adr/0001. Planejamento em docs/plans; especificação original
preservada em docs/internal/spec/PROMPT_0.md. Roadmap enumera 1 a 15 sem inventar escopos.
Procedimento de backup e recuperação em docs/internal/references/BACKUP_RECUPERACAO.md.

## Git e GitHub

Clone existente preservado em C:\Dev\CAD-CRAFT, branch main inicialmente vazia.
Origin: https://github.com/Vandecao/CAD-CRAFT.git.
`git ls-remote origin` funcionou e não retornou refs antes da sincronização.
Autenticação oficial existente da conta Vandecao confirmada por gh auth status.
Nenhum repositório, conexão ou credencial foi criado.
Autoria configurada somente neste repositório: Vanderlei Braga e endereço público
noreply `115331764+Vandecao@users.noreply.github.com`, usando ID público consultado
na API oficial via gh. Não foram gravados tokens no projeto.

Primeiro commit/push deste bootstrap autorizado pelo Prompt 0. SHA final deve
ser consultado no próprio Git, evitando SHA autorreferente neste documento.
Verificação final: HEAD local deve coincidir com origin/main.

Primeira sincronização executada: commit
`f902d4784be6072a333e919da66af1150aa7884b`, SHA idêntico no HEAD e na consulta
`git ls-remote origin refs/heads/main`; árvore limpa e tracking main...origin/main.
O fechamento documental subsequente registra esta evidência.

## Validações executadas

| Comando | Resultado real |
|---|---|
| cargo fmt --check | Aprovado |
| cargo check --workspace --all-targets --locked | Aprovado, 2 crates |
| cargo test --workspace --locked | Aprovado: 0 testes, 0 falhas, incluindo doctests |
| cargo clippy --workspace --all-targets --locked -- -D warnings | Aprovado |
| git diff --check | Aprovado |
| git diff --cached --check | Aprovado; repetido após staging para incluir arquivos novos |
| cargo run -p cad-gui --locked | Compilou, linkou e imprimiu diagnóstico de bootstrap |

Não há testes de domínio nem cobertura funcional; não foram inventados testes
para atribuir aprovação a um editor inexistente. Nenhuma homologação manual.
Estrutura obrigatória conferida por existência de arquivos; cargo metadata
confirmou os dois crates e ausência de dependências externas. Arquivos staged
revisados e busca por padrões de tokens GitHub, AWS e chaves privadas sem
ocorrências. Nenhuma credencial identificada no conteúdo versionado.

## Problemas e limitações

- Terminal restrito falhou na criação do processo. Auditoria, instalação e
  validações foram executadas com aprovações fora do sandbox.
- Build Tools/SDK faltavam; instalação e uso real corrigiram essa ausência.
- winget informou reinicialização necessária; vswhere depois informou instalação
  completa, launchable e isRebootRequired=false. Builds funcionaram sem reinício.
  Registrar observação ENV-003 e repetir validação após próximo reinício oportuno.
- Sem GUI CAD e sem funcionalidades das Etapas 1 a 15 por restrição expressa.

## Próximos passos

Aguardar prompt específico da Etapa 1. A instrução complementar posterior
substituiu as regras de execução Git: commits e push são exclusivamente manuais,
conforme [política vigente](POLITICA_GITHUB_BACKUP.md). Os registros acima
descrevem apenas operações históricas do bootstrap, não autorizações futuras.
Manter memória atualizada e homologações manuais ao final da Etapa 15.
