# Etapa 4 — Plano de implementação

Execução direta solicitada pelo usuário em 2026-10-27. Usar execução nativa,
testes antes da implementação e uma revisão independente interna ao final.
Especificação: `docs/internal/spec/ETAPA_04_PROJETOS_PAGINAS_REPRESENTACOES.md`.

## Objetivo e restrições

Ampliar Project/Document/Page existentes e acrescentar representações elétricas
independentes da GUI. Rust + Serde + UUID existentes; nenhuma dependência nova.
Sem comandos Git modificadores, worktrees, commit/push, GUI ou início da Etapa 5.
Base observada: main, df6b482a14b91aad571ae38d7ef7e9c1c26a2e3d (Etapa 3), árvore limpa.

## Decisões de implementação

- Reutilizar Project (nome, descrição, metadados), Document e Page. Acrescentar
  identificação opcional ao Project; snapshots antigos permanecem sem identificação.
- Page conserva number como identificação textual, independente de PageId. Orientação
  é derivada das dimensões (Portrait/Landscape/Square), evitando duas fontes conflitantes.
  Dimensões zero preexistentes continuam válidas como geometria documental degenerada.
- Ordem opcional por página: ausência preserva snapshots antigos; consulta ordena
  por ordem explícita e ID, páginas sem ordem vêm depois. Reordenação exige permutação
  completa das páginas do documento e atribui posições contíguas. Remoção pode deixar lacunas.
- Organização por caminhos funcionais e de localização independentes, usando
  DesignationPart. Árvore derivada por documento/eixo, com caminhos de grupos e páginas
  diretas; não criar cadastro de funções elétricas ou dispositivos para agrupar folhas.
  Caminhos vazios significam sem classificação, sem inventar grupos nos snapshots.
- RepresentationId UUID v7; RepresentationTarget enum estritamente elétrico, incluindo
  Connection, sem estender EntityRef para conexão nem criar propriedades elétricas nela.
  Múltiplas representações por entidade/página são permitidas: ID distingue ocorrências.
- Transform2D: espelhamento dos componentes locais X/Y, rotação em graus anti-horária,
  depois translação em mm. Inversa e AABB verificadas; estouro numérico é erro.
- Representação guarda transform, limites locais opcionais e estilo essencial (visível,
  camada inteira, rótulo opcional). Sem biblioteca de símbolos ou roteamento.
  Sem limites locais, apenas a âncora é conhecida. Folha não recorta conteúdo nem viewport.
- Registros BTreeMap e índices derivados por página/alvo, reconstruídos na importação
  e conferidos por validate. Serialização determinística por IDs, ordem documental separada.
- Remover página exige ausência de representações e de referências de propriedades
  externas à própria página. Remover representação não altera entidades elétricas.
  Remover conexão exige também ausência de representações; demais entidades elétricas
  continuam sem API de exclusão. Sem cascatas; falhas previstas são atômicas.
- Navegação relacionada: outras representações do mesmo alvo, explicitamente; relações
  elétricas adicionais continuam consultáveis pelas APIs anteriores, sem novo grafo.

Alternativas: aninhar entidades elétricas nas páginas duplicaria identidade; cadastro
hierárquico separado exigiria CRUD/IDs desnecessários para caminhos já adotados no Core.
Escolhas acima são decisões técnicas do agente, sujeitas à revisão do usuário.

## Tarefas e evidências

- [x] 1. Valores e páginas: testes document_values/document_pages; estender page.rs,
  criar document_structure.rs e transform.rs; IDs/exports. RED por APIs ausentes;
  GREEN via cargo test -p cad-craft --test document_values --test document_pages --locked.
- [x] 2. Representações e integridade: testes document_representations; criar
  representation.rs, document_index.rs e project/documentation.rs; integrar Project,
  remoção de Connection e validação. Testar ausência, duplicação, projeto estrangeiro,
  falhas atômicas, transformações extremas, exclusão e invariância elétrica.
- [x] 3. Snapshot/consultas: testes document_serialization; defaults vazios/opcionais,
  índices reconstruídos, duplicados e corrupção rejeitados. Testar formatos das Etapas
  1–3, permutações das listas, todos os alvos e consultas sem classificação.
- [x] 4. Revisão independente, validação oficial e diagnóstico offline. Corrigir
  regressões e registrar contagens reais. Atualizar ADR/arquitetura/roadmap/memória,
  relatório, contrato da Etapa 5 e roteiro de homologação futura.

## Foco de revisão

Referências de propriedades que impedem exclusão; inversa de espelhamento+rotação;
overflow mesmo com entradas finitas; snapshots antigos sem metadados fabricados;
consistência dos índices após mover/excluir/importar e invariância elétrica.
Testes específicos pertencem às tarefas 1–3, sem remover testes anteriores.

## Registro de execução

Preflight: tarefas 1–3 compartilham Page/Transform2D/RepresentationTarget e o DTO
ProjectData; executar sequencialmente para manter assinaturas e defaults consistentes.
Política local prevalece sobre sugestões de commits/worktrees dos skills.

Tarefas 1–3 implementadas. Testes iniciais falharam por APIs/tipos ainda ausentes
(logs red-pages/red-representations); primeira suíte completa GREEN. Cobertura
ampliada para todos os alvos e proprietários de propriedades; validação oficial
registrou 113 Rust + 15 PowerShell aprovados, sem falhas/ignorados.
Logs locais ignorados em `.local/etapa-04/`, não são artefatos versionáveis.
Os snapshots legados dos testes removem campos aditivos do formato atual;
não são fixtures históricas congeladas. Essa precisão substitui “legado real”.

Revisão independente interna por subagente somente leitura: sem Critical/Important/
Minor no código. A primeira solicitação não retornou resultado antes da retomada;
uma revisão nova foi concluída após a retomada. Nenhum agente externo utilizado.
Aspectos fora do julgamento: documentação final conferida pelo principal; GUI,
símbolos, roteamento, persistência física e homologação pertencem a etapas futuras;
escala documental não medida, sem promessa de capacidade. Riscos/limites no ADR-0006
e no relatório. Não houve alteração arquitetural elétrica nem dependência nova.

Tarefa 4 concluída: revisão recebida, documentação/memória/ADR/contratos/relatório
produzidos; execução final do script oficial registrada em final-validation.log,
exit 0, 128 aprovados. Verificação final de diff após normalizar LF nos dois
documentos de memória reescritos. Sem commits, push, worktrees ou Etapa 5.
