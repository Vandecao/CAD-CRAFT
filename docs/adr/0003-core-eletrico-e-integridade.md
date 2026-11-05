# ADR-0003 — Core elétrico e integridade do agregado

Data: 2026-10-27. Estado: implementado no escopo autorizado da Etapa 1;
decisões técnicas do agente sujeitas à revisão do usuário, sem homologação manual.

## Contexto

O anexo da Etapa 1 exige domínio elétrico utilizável sem GUI, IDs persistentes,
relações consistentes e Serde. O bootstrap não continha APIs de domínio.
ADR-0001 e ADR-0002 permanecem válidos.

## Decisão

- `Project` é o agregado: campos privados, registros `BTreeMap` por ID tipado,
  consultas imutáveis e operações de inserção/associação com validação prévia.
- Documentos, páginas, componentes, dispositivos, funções, terminais e pontos
  de conexão armazenam `ProjectId`. Relações usam IDs, não cópias de entidades.
  Páginas por documento e funções por dispositivo são consultas derivadas.
- Componente pode ser independente ou referenciar dispositivo e função; função
  exige dispositivo e ambos devem coincidir. Terminal físico pertence a dispositivo
  ou componente. Ponto funcional pertence a função, podendo mapear terminal físico
  compatível. Sem relações gráficas nem algoritmos de conectividade.
- UUID v7 gerado por `uuid`, envolvido em oito tipos distintos. Conversão e Serde
  validam versão/variante; duplicação é verificada por tipo de entidade no projeto.
- Valores validados (`Text`, `DesignationPart`, `Decimal`, geometria, enumeração)
  repetem as barreiras na desserialização. Não se expõem campos mutáveis do agregado.
- Descrição e metadados são descritivos. Estruturas funcionais/localizações são
  caminhos tipados da designação, sem cadastro hierárquico separado nesta etapa.
- Propriedades conhecidas possuem chaves enumeradas e unidades fixas, separadas
  de propriedades customizadas com valores tipados e `EntityRef`.
- Serialização estrutural via Serde; DTO interno representa registros como listas.
  Importação reúne todas as entidades, rejeita IDs repetidos e valida o agregado.
  Assim, ordem dos dados e ciclos de referências em propriedades não quebram importação.
  JSON é somente um backend de teste (`serde_json` em dev-dependencies).

## Alternativas consideradas e consequências

Objetos aninhados duplicariam dispositivos/funções usados em páginas diferentes.
Referências mutáveis compartilhadas complicariam invariantes e Serde. O agregado
foi escolhido pelo agente para manter fonte única e resolução explícita.

Inserções individuais exigem referências existentes. Para ciclos em propriedades,
inserir entidades primeiro e usar `Project::set_properties` depois. Erros não deixam
inserção parcial. Entidades destacadas validam valores locais; somente inserção no
Project/importação do Project garante validade das relações no contexto completo.

Consultas inversas são lineares no registro correspondente. Índices auxiliares e
benchmarks ficam para necessidade comprovada. Sem limite de tamanho de importação
definido; a futura persistência deverá tratar recursos e evolução de schema.

Designações possuem partes alfanuméricas/`_`/`.` sem separadores; isto é uma política
básica do aplicativo, não conformidade IEC. Igualdade de designações ou números de
página não significa identidade: escopos normativos de unicidade ficam para etapas
apropriadas. IDs e vínculos repetidos em uma página são rejeitados.

Coordenadas são `f64` finitos em mm; tamanhos não negativos, limites de retângulo
finitos. Decimais técnicos não são aritmética decimal exata. Undo/Redo, persistência
física versionada, migrações, representações gráficas, condutores e redes continuam
contratos futuros. Nenhuma operação Git modificadora decorre desta decisão.
