# Etapa 3 — Plano de implementação

> Execução pelo agente principal com superpowers:executing-plans; revisão técnica
> independente ao final, autorizada pelo anexo. Sem operações Git modificadoras.

**Objetivo:** documentar condutores, cabos, veias e réguas no Core, com integridade
e rastreabilidade explícita para a conectividade lógica já existente.
**Arquitetura:** registros BTreeMap no Project; entidades e consultas imutáveis;
relações persistentes unidirecionais e índices privados reconstruíveis.
**Tecnologias:** Rust, std, Serde, uuid existentes; nenhuma dependência nova.
**Especificação:** [anexo integral](../spec/ETAPA_03_CONDUTORES_CABOS_BORNES.md).

## Restrições e decisões de implementação

- Trabalhar no checkout/main autorizado. Não criar worktree, staging ou commit.
- Preservar todos os testes, APIs e snapshots anteriores; sem GUI ou Etapa 4.
- Terminal continua entidade física de Device/Component. Acrescentar posições
  explicitamente cadastradas; `PhysicalTermination` = TerminalId + DesignationPart.
  Não criar outra entidade Terminal nem usar ConnectionPoint como extremo físico.
- Conductor possui dois extremos distintos, designação, seção em mm², marcação
  (cor e/ou identificação), comprimento opcional em mm, material e função controlados.
  Condutores paralelos exigem designações distintas no mesmo par não direcionado.
- Cable declara quantidade nominal positiva; permite cadastro incompleto de veias
  até esse limite. CableCore pertence a um cabo e guarda o único vínculo opcional
  para Conductor; inversas/cabo do condutor são consultas derivadas.
- Veia instalada exige seção compatível quando declarada e cores compatíveis quando
  ambas informadas. Identificação de veia é única por cabo, condutor só em uma veia.
- TerminalStrip contém lista ordenada de TerminalId. Um terminal só pode integrar
  uma régua; seu proprietário físico e sua designação permanecem preservados.
- Conductor guarda vínculo opcional para Connection e qual ponto lógico corresponde
  ao seu extremo source. O outro ponto vem da Connection. Verificar mapeamento dos
  pontos funcionais aos terminais existentes; não inferir continuidade entre posições.
- Desvinculação de veia/conexão é explícita. Remoção de Connection associada é
  bloqueada até desvinculação; nenhuma exclusão em cascata.
- Novas entidades não removíveis integram EntityRef e propriedades. Seção nominal
  de condutor/veia usa exclusivamente o campo tipado, sem CrossSection redundante
  em TechnicalProperties. Índices não entram no snapshot.

Alternativas: extremos puramente funcionais confundiriam camadas; um novo UUID
para cada posição acrescentaria entidade sem necessidade atual. Relações inversas
persistentes duplicariam verdade; varreduras em cada inserção gerariam construção
quadrática. A opção adotada preserva convenções com índices reconstruíveis.
São decisões técnicas do agente dentro da autorização integral, sujeitas à revisão.

## Tarefas e verificações

### 1. Valores e entidades físicas

Arquivos: domain/{physical_values,conductor,cable,cable_core,terminal_strip}.rs;
extensões ids.rs, terminal.rs, mod.rs, error.rs. Teste physical_values.rs.

- [x] Escrever testes de UUID v7, números finitos/unidades, marcação obrigatória,
  extremos distintos, posições duplicadas, enums inválidos e Serde adulterado.
- [x] Executar `cargo test -p cad-craft --test physical_values --locked` e observar
  ausência das APIs; implementar tipos/construtores/validação local e repetir.

### 2. Agregado e rastreabilidade

Arquivos: project/{physical_wiring,physical_integrity,physical_queries}.rs;
physical_index.rs; project.rs, integrity.rs, connectivity.rs e snapshot.rs.
Testes: physical_wiring.rs, physical_serialization.rs e physical_common/mod.rs.

- [x] Escrever cenários A–J, falhas atômicas, associação invertida, proteção de
  remoção, propriedades referenciais, independência das redes e consultas por device.
- [x] Executar testes antes da implementação; acrescentar insert/lookup/iteradores,
  assign/unassign de veias e conexões, associação ordenada de terminais.
- [x] Estender snapshot com defaults vazios, coletar registros antes da validação,
  rejeitar duplicações e reconstruir índices; testar ordem/identidade/compatibilidade.
- [x] Confirmar aprovação das suítes novas e de toda a suíte Rust anterior.

### 3. Escala, revisão e entrega

Arquivos: tests/physical_scale.rs, ADR-0005, relatório da etapa, memória, README.

- [x] Exercitar 100/1.000/10.000 condutores, cabos de quatro veias e uma régua longa;
  medir construção/consulta/validação/importação em release, sem limites arbitrários.
- [x] Revisar independentemente: separação, integridade, Serde, escala e regressões;
  corrigir Critical/Important e avaliar Minor com registro explícito.
- [x] Executar scripts/validate.ps1, diagnóstico offline e rustdoc/consumidor.
- [x] Atualizar memória, ADR, relatório com contagens reais, inventário e SHA observado.
- [x] Encerrar para revisão/commit manual, push após Etapa 4; não iniciar Etapa 4.

## Foco de revisão

1. Mesmo terminal com posições diferentes não é autoextremo: testar cadastro e uso.
2. Mesmos extremos em ordem invertida não permitem designação física duplicada.
3. Trocar vínculo deve falhar sem desfazer o anterior quando destino é incompatível.
4. Snapshot com relações cíclicas em propriedades deve resolver após coleta completa.
5. Remover Connection usada não pode deixar vínculo físico pendente.

## Registro de execução

- 2026-10-27: árvore limpa, main, HEAD 98445717f16b9f464aa45664422b6eec12c35781.
  Baseline Rust: 55 aprovados. ls-remote falhou (DNS github.com); tracking local
  coincide com HEAD, envio confirmado pelo usuário, remoto atual não confirmado.
- Deliberação: autorização integral do anexo permite decisões de implementação e
  execução contínua. Etapas de confirmação, worktree/commit das skills não se aplicam
  diante dessa autorização e da política expressa de Git do projeto.
- Valores: 6 testes aprovados após falha de compilação por APIs ausentes.
- Agregado/Serde: 13 + 8 testes aprovados após falhas por APIs ausentes. Sem alteração
  de testes anteriores. Registro em values-red.log, wiring-red.log e serde-red.log
  dentro de .local/etapa-03, ignorada pelo Git.
- Escala: 2 testes, 100/1.000/10.000 elementos, 3 amostras por caso em release;
  medições registradas no relatório, sem limiares ou promessa de capacidade.
- Revisão independente interna por inspeção: zero Critical/Important; um Minor de
  cobertura de entidades desserializadas isoladamente. Avaliado e tratado com três
  testes adicionais, aprovados. Não foi demonstrado defeito funcional anterior.
- Validação final: scripts/validate.ps1 exit 0; 88 Rust + 15 PowerShell = 103
  aprovados, zero falhas/ignorados. Fmt/check/Clippy/diff aprovados. Diagnóstico
  offline, rustdoc e smoke cad-gui exit 0. Nenhuma homologação manual.
- Documentação: ADR-0005, relatório técnico, README, índice de especificações e
  sete arquivos de memória atualizados. Etapa 4 não iniciada. Inventário: 20 novos,
  18 modificados, nenhum excluído. Sem staging/commit/push; HEAD observado inalterado.
