# Testes

Testes unitários futuros ficam junto ao módulo; integração de cada crate em seu
diretório `tests/`. Este diretório reserva documentação de cenários entre crates.
Bootstrap: zero testes funcionais, sem alegar cobertura elétrica. O binário
diagnóstico e os comandos Cargo verificam compilação e linkedição.

Testes operacionais da política Git: `scripts/tests/check-repository.Tests.ps1`,
incluídos em `scripts/validate.ps1`. Cobrem diagnóstico, limites, nomes sensíveis,
ignore e ausência de escrita no Git. Usam consultas locais reais e respostas Git
simuladas para cenários de índice/histórico; não criam repositórios ou commits.
Não são testes de funcionalidades elétricas ou gráficas.
