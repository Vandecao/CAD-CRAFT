# Plano de implementação do bootstrap

Objetivo: reconstruir ambiente reproduzível e memória Git sem iniciar Etapa 1.
Especificação: `docs/internal/spec/PROMPT_0.md` e instrução de usar origin existente.
Execução local pelo agente principal, conforme autorização autônoma do usuário.

## Decisões de execução

Utilizar o clone vazio existente em main. Workspace mínimo sem bibliotecas
externas; Core documental e binário diagnóstico permitem validar linker sem
antecipar editor. Instalar somente MSVC/SDK ausentes, por fonte oficial.
Não há implementação de domínio que justifique testes artificiais.

## Tarefas e critérios

- [x] Auditar OS, ferramentas, disco, Git e autenticação existente.
- [x] Instalar dependências ausentes e confirmar versões efetivas.
- [x] Criar manifests, toolchain fixa, Core e entrada diagnóstica.
- [x] Criar AGENTS, memória, ADR, roadmap e recuperação.
- [x] Executar fmt, check, test, clippy, diff e diagnóstico linkado.
- [x] Revisar documentos, segredos e alterações; registrar evidências.
- [x] Commit e primeira sincronização íntegra com origin; confirmar SHA remoto.

Evidência: f902d4784be6072a333e919da66af1150aa7884b publicado em origin/main.
Revisão final pelo agente principal: estrutura, manifests, lockfile, contratos,
memória e padrões de credenciais inspecionados; sem agentes externos.
Autorização autônoma e estrutura expressa do Prompt 0 determinaram a execução.

## Foco da revisão

Clone vazio sem identidade Git; SDK/linker ausente; resolução reproduzível;
diferença entre planejamento e implementação; falha de autenticação/sincronização.
Confirmar autoria pública local; compilar executável; usar lockfile/toolchain;
revisar estados da memória; consultar remoto antes e depois do push.
