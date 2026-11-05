# Backup e recuperação

A política vigente é [POLITICA_GITHUB_BACKUP](../reports/POLITICA_GITHUB_BACKUP.md),
com instrução integral em [docs/spec](../spec/POLITICA_GIT_GITHUB_BACKUP.md)
e decisão em [ADR-0002](../../adr/0002-controle-manual-git.md).

Todas as operações de versionamento e sincronização são manuais e exclusivas
do usuário. O Codex informa a necessidade após testar, documentar e relatar.
Commit manual por etapa; push manual nas pares, no bootstrap e na Etapa 15.
A política complementar também aguarda commit e push manuais.

Após perda de máquina/VM, o usuário recupera o remoto em pasta nova, preservando
qualquer cópia sobrevivente, e reconstrói a toolchain pelo procedimento do relatório.
Nunca sobrescrever clones/backups existentes para recuperar uma versão anterior.
Não usar pull sobre trabalho não consolidado, force push ou reset destrutivo.

GitHub protege somente conteúdo enviado. Arquivos não versionados, alterações
não commitadas e commits sem push continuam sujeitos a perdas locais. Recomenda-se
cópia externa independente com retenção de versões e teste de restauração;
nenhum backup é criado automaticamente nesta entrega. Backup de arquivos em
trabalho deve incluir a memória e as mudanças locais necessárias, e proteger
credenciais separadamente. Um bundle Git preserva commits/refs, não arquivos
não rastreados nem mudanças ainda não commitadas; sua criação/restauração cabe ao usuário.
