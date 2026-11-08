# ADR-0026 — verificações de consistência; DEC-11-01/02 e CON-001

Estado: implementado e testado automaticamente na Etapa 24. Homologação manual geral na Etapa 28.

## Contexto

`Project::validate()` recusa estados inválidos, mas o projeto também precisa apontar o que é **válido e suspeito** (repetições, ligações abertas, relatórios velhos), com navegação até o problema. Três pendências antigas dependiam de decisão do usuário: DEC-11-01 (continuidade pelo acoplamento de conectores), DEC-11-02 (efeito do borne seccionável) e CON-001 (designações de conector repetidas).

## Decisão

### Motor

- `Project::check(CheckScope)` aplica regras puras do cadastro, em ordem determinística.
- Cada mensagem tem um código estável `VER-0xx`, gravidade (erro, aviso, informação), texto e alvos.
- A identidade da mensagem é regra + alvos, usada para ignorar e para reavaliar.
- O alcance "página" mantém as mensagens que têm algum alvo localizado nela.
- `check_target_location` dá a página e a ocorrência de cada alvo.
- Não é ERC normativo nem cálculo elétrico.

### Regras

| Código | Regra | Padrão |
|---|---|---|
| VER-001 | Designação de dispositivo repetida (projetos antigos; novas escritas já recusam) | Erro |
| VER-002 | Designação de conector repetida (CON-001; nomes não unem redes) | Aviso |
| VER-003 | Número de página repetido no documento | Erro |
| VER-004 | Dispositivo com auxiliares sem função principal | Aviso |
| VER-006 | Função de peça não representada | Informação |
| VER-007 | Porta de símbolo sem conexão (marcadores fora) | Aviso |
| VER-008 | Fio sem número quando outros já foram numerados | Aviso |
| VER-009 | Rede com potenciais nomeados diferentes | Erro |
| VER-010 | Interrupção sem par no grupo | Aviso |
| VER-011 | Marcador de potencial sem ponto | Aviso |
| VER-014 | Borne de régua sem destino | Informação |
| VER-016 | Borne seccionável aberto (DEC-11-02) | Informação |
| VER-017 | Contato acoplado sem ponto associado (sem continuidade) | Aviso |
| VER-018 | Dispositivo sem peça | Informação |
| VER-020 | Relatório desatualizado (Etapa 23) | Aviso |
| VER-021 | Designação de ponto repetida no mesmo dispositivo (Etapa 33), por exemplo dois contatos 13-14 em -K1 | Aviso |

**Garantidas pela integridade, sem regra própria:**
- VER-005 (duas funções principais): `validate_function_roles` recusa;
- VER-012 (veia em duas conexões): a veia tem no máximo um condutor, e o condutor no máximo uma conexão;
- VER-013 (veias além da capacidade): o índice físico recusa;
- VER-015 (ponte entre réguas): `CreateBridge` exige a mesma régua;
- VER-019 (peça sem código): o código é um `Text` obrigatório.

### Esquema e execução

- O esquema (ativação e gravidade por regra, mais as mensagens ignoradas com justificativa) fica no projeto, no `.cadcraft` formato 9 (decisão 4). Ele é validado: código desconhecido ou ignorado duplicado são recusados.
- `set_check_scheme` / `restore_check_scheme` são reversíveis (`Transaction::CheckScheme`).
- A verificação é sob demanda, em segundo plano (decisão 5). Depois de cada edição, o painel recalcula em segundo plano e mantém só as mensagens que já estavam listadas: as corrigidas somem e as novas só aparecem ao verificar de novo.

### DEC-11-01 (proposta A)

- `ConnectorMating` passa a possuir uma `Connection` por par de contatos com ponto associado.
- `UpsertMating` cria as conexões e reaproveita as que já ligam os mesmos pontos, o que preserva o número do fio. `DeleteMating` remove as conexões. Tudo acontece no mesmo `PhysicalDelta`.
- As redes atravessam o conector acoplado.
- A validação exige coerência entre pares e conexões: mudar o ponto de um contato acoplado é recusado ("desacople antes").
- Acoplamentos lidos de arquivos anteriores (sem `owned`) recebem as conexões na carga.

### DEC-11-02

O estado do borne seccionável continua documental: a rede é contínua e a VER-016 informa os bornes abertos.

### CON-001

Designações de conector continuam podendo se repetir e nunca unem redes. A VER-002 avisa.

## Consequências

- Acoplar passa a mudar a conectividade. O teste antigo que afirmava o contrário foi ajustado, com o registro no relatório.
- A numeração de fios trata as conexões do acoplamento como qualquer outra.
- A reavaliação automática recalcula o projeto inteiro em segundo plano (0,24 s para 20.040 ocorrências no Release), em vez de só os alvos tocados. O resultado visível é o mesmo.
- O motor fica pronto para as regras elétricas ou normativas futuras e para importar esquemas do EPLAN (Etapa 26).
