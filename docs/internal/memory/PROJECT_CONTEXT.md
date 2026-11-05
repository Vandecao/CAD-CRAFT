# Contexto do projeto

CAD-CRAFT é um software profissional Windows de engenharia elétrica industrial,
destinado a projetistas e engenheiros. Reconstrução após perda do ambiente anterior.
Inspiração funcional: EPLAN Electric P8, sem copiar código proprietário.

Tecnologias: Rust, Cargo, PowerShell, Git/GitHub, VS Code e GUI egui/eframe.
Objetivos: modularidade, tipagem forte, testabilidade, baixo acoplamento e
persistência versionada, com dados elétricos separados do desenho.

Etapas 1–4 implementam domínio elétrico, conectividade lógica, condutores físicos,
cabos/veias, réguas de bornes, organização de páginas e representações documentais
independentes da GUI. A Etapa 5 acrescenta aplicação desktop documental, Ribbon,
navegadores, inspetor, miniaturas e câmera sem modificar o Core.
Escopo futuro: símbolos, referências cruzadas mais amplas,
especializações de bornes/conectores, relatórios, ERC e exportação DXF,
com preparação para DWG, PDF e XLSX. Ver estado verificável em CURRENT_STATE.
Prompts das etapas serão recebidos progressivamente. Homologação manual
adiada até o fim da Etapa 15, sem dispensar testes automáticos.

## GitHub, armazenamento e proteção do código-fonte

Diretório principal: `C:\Dev\CAD-CRAFT`. Repositório remoto oficial no GitHub:
`https://github.com/Vandecao/CAD-CRAFT.git`, exclusivamente no origin existente;
branch principal `main`. Não criar outro repositório ou nova conexão.

O usuário controla manualmente todo versionamento e sincronização, conforme
AGENTS e [ADR-0002](../../adr/0002-controle-manual-git.md), inclusive após a Etapa 15.
O Codex somente desenvolve, testa, documenta e comunica a necessidade de operações.

Manter Git preferencialmente abaixo de 1 GB; tratar 10 GB como referência
operacional, não garantia de limite. Avaliar arquivos acima de 50 MB e não
adicionar arquivos acima de 100 MB ao Git convencional. LFS somente com
autorização expressa. Builds, caches, temporários, VMs, imagens de disco,
credenciais, logs extensos e backups completos ficam fora do versionamento.
Código, lockfile, testes, scripts, documentação e recursos legítimos permanecem.

A recuperação de commits pelo GitHub depende de push manual efetivamente
concluído. O remoto não protege arquivos não versionados, alterações não commitadas
ou commits ainda não enviados. Memória técnica versionada e toolchain fixada
permitem reconstruir o ambiente após perda da máquina/VM. Recomenda-se backup
externo independente para complementar a proteção; não foi implementado aqui.
Procedimento em [POLITICA_GITHUB_BACKUP](../reports/POLITICA_GITHUB_BACKUP.md).
