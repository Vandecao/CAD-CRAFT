# CAD-CRAFT — ETAPA 4

## ESTRUTURA DOCUMENTAL, PÁGINAS E REPRESENTAÇÕES ELÉTRICAS

Implementar a Etapa 4 do CAD-CRAFT, preparando o Core para a futura interface gráfica egui/eframe.

### 1. Recuperação de contexto

Antes de implementar:

1. Leia `AGENTS.md` e as instruções aplicáveis.
2. Leia `docs/internal/memory/HANDOFF.md`.
3. Leia `docs/internal/memory/CURRENT_STATE.md`.
4. Leia `docs/internal/memory/ARCHITECTURE.md`.
5. Leia `docs/internal/memory/ROADMAP.md`.
6. Consulte os ADRs e especificações relevantes.
7. Examine o código existente.
8. Execute `git log -1 --oneline` e `git status --short`.

Identifique o commit da Etapa 3 e preserve todos os arquivos existentes.

Não altere decisões arquiteturais sem justificativa técnica documentada.

### 2. Objetivo

Implementar um modelo documental que permita organizar projetos elétricos industriais em estruturas, páginas e representações gráficas vinculadas às entidades elétricas do Core.

A estrutura deverá ser independente de egui/eframe.

Não implementar a interface gráfica nesta etapa.

### 3. Estrutura de projetos

Reutilizar o `Project` existente.

Adicionar, conforme necessário:

* Metadados documentais.
* Identificação do projeto.
* Descrição.
* Estrutura hierárquica.
* Organização funcional.
* Organização por localização.
* Índices e consultas determinísticas.

Evitar duplicação das entidades elétricas existentes.

### 4. Páginas

Implementar entidades de página com:

* `PageId` tipado.
* Nome.
* Identificação.
* Tipo de página.
* Dimensões em milímetros.
* Orientação.
* Ordem documental.
* Propriedades.
* Referências ao projeto.

Tipos iniciais sugeridos:

* Esquema multifilar.
* Esquema unifilar.
* Visão geral.
* Layout.
* Documentação.

Permitir criar, consultar, renomear, reordenar e remover páginas por APIs validadas.

### 5. Hierarquia documental

Permitir organizar páginas por função e localização.

Preservar a distinção entre:

* Estrutura funcional.
* Estrutura de localização.
* Identidade do dispositivo.
* Identidade da página.

Não utilizar o número da página como identificador interno.

### 6. Representações

Implementar representações documentais associadas a entidades existentes do Core.

Uma mesma entidade elétrica poderá possuir diferentes representações, quando permitido pelo domínio.

Cada representação deverá possuir:

* ID tipado.
* Página.
* Entidade referenciada.
* Posição em milímetros.
* Rotação.
* Espelhamento.
* Propriedades visuais essenciais.

Não duplicar componentes, terminais ou conexões elétricas para criar uma representação gráfica.

### 7. Coordenadas e transformações

Preservar:

* Origem inferior esquerda.
* X positivo para a direita.
* Y positivo para cima.
* Unidades em milímetros.
* Rotação positiva anti-horária.

Definir estruturas matemáticas independentes da GUI.

A inversão do eixo Y deverá ocorrer exclusivamente na camada de visualização futura.

A folha não deverá definir os limites do viewport interativo.

### 8. Integridade referencial

Validar:

* Página pertencente ao projeto.
* Representação vinculada a entidade existente.
* Identificadores únicos.
* Transformações válidas.
* Coordenadas finitas.
* Referências documentais consistentes.

Estabelecer regras explícitas para remoção de páginas ou entidades referenciadas.

Não remover entidades elétricas automaticamente ao excluir uma representação, salvo regra de domínio explicitamente justificada.

### 9. Consultas para a futura GUI

Criar APIs de consulta para:

* Árvore de páginas.
* Páginas ordenadas.
* Representações por página.
* Representações por entidade.
* Limites documentais.
* Localização de representações.
* Navegação entre representações relacionadas.

Evitar dependência de bibliotecas gráficas.

### 10. Persistência

Integrar as novas entidades ao mecanismo de serialização existente.

Preservar snapshots anteriores.

Não inventar dados documentais para projetos antigos.

Manter serialização determinística e validação de integridade.

Não introduzir prematuramente um formato `.cadcraft` definitivo sem planejamento específico.

### 11. Testes

Preservar os testes anteriores.

Adicionar testes para:

* Criação de páginas.
* Tipos e dimensões.
* Ordenação.
* Renomeação.
* Hierarquia.
* Representações.
* Rotação e espelhamento.
* Coordenadas.
* Referências inválidas.
* Exclusão.
* Serialização.
* Compatibilidade retroativa.
* Consultas.
* Integridade elétrica após alterações documentais.

Executar as validações existentes, preferencialmente pelo script oficial do repositório.

### 12. Preparação para egui

Documentar os contratos que a Etapa 5 utilizará para:

* Navegador de páginas.
* Viewport.
* Seleção.
* Inspetor de propriedades.
* Renderização de representações.
* Comandos do editor.

Não implementar Ribbon, viewport egui ou widgets nesta etapa.

### 13. Documentação e memória

Criar o relatório:

`docs/internal/reports/ETAPA_04_PROJETOS_PAGINAS_REPRESENTACOES.md`

Atualizar os documentos de memória persistente pertinentes.

Registrar novas decisões arquiteturais somente quando necessárias.

Documentar limitações e dependências da Etapa 5.

### 14. Validação final

Executar:

```powershell
pwsh -NoProfile -File scripts/validate.ps1
git diff --check
```

Corrigir todas as regressões.

Solicitar revisão independente quando disponível.

### 15. Restrições

* Não executar commit.
* Não executar push.
* Não iniciar a Etapa 5.
* Não alterar a arquitetura elétrica sem necessidade.
* Não implementar um segundo motor de conectividade.
* Não introduzir egui no Core.
* Não descartar arquivos preexistentes.
* Não apresentar funcionalidades não implementadas como concluídas.

### 16. Entrega

Apresentar:

1. Resumo da implementação.
2. Entidades e APIs criadas.
3. Regras de integridade.
4. Compatibilidade dos snapshots.
5. Quantidade de testes aprovados.
6. Revisão independente.
7. Arquivos criados e modificados.
8. Limitações.
9. Contratos preparados para a Etapa 5.
10. Roteiro de homologação.

Aguardar revisão e commit manual do usuário.
