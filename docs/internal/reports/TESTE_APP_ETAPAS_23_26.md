# Teste no app — Etapas 23 a 26 (2026-11-05)

O Claude Code testou o executável Release (`target/release/cad-gui.exe`) com autorização do usuário para usar o computador ("Etapa 26 concluida. Use o computador e faça os testes"). Isto não é a homologação manual geral, que é feita na Etapa 28.

Dados usados, todos fora do repositório:
- `industrial.qet` (50 folhas), dos exemplos do QElectroTech;
- quatro pacotes `.edz` dos fabricantes: BAN.85069 (ZIP), 1734-IB8 e SE.RM17TG00 (7z) e RIT.2383210 (incompleto);
- bibliotecas de peças e macros numa pasta temporária. As preferências do app voltaram ao estado anterior ao fim do teste.

## Roteiro e resultado

| Etapa | Verificado no app | Resultado |
|---|---|---|
| 25 | Arquivo > Importar > Projeto QElectroTech: Procurar (diálogo nativo), Analisar em segundo plano, prévia (50 folhas, 290 dispositivos, 345 fios, 105 relatórios de folha), Criar projeto importado, 1ª folha aberta, projeto pendente de salvar | Ok, depois das correções 1 e 2 |
| 25 | Folha 5 com moldura, carimbo, zonas, designações, números de fio, textos e referências cruzadas das setas | Ok, depois das correções 1 e 2 |
| 24 | Verificações abertas sozinhas após a importação (0 erros, 589 avisos, 290 informações); filtro Aviso; duplo clique navega até a folha e seleciona a ocorrência | Ok |
| 23 | Ferramentas > Gerar relatório > Lista de dispositivos: prévia (7 páginas, 290 linhas), página R51 com designação, classificação, funções e ocorrências | Ok |
| 26 | Dados-mestre > Importar .edz: 4 pacotes, análise com prévia por pacote, conflitos com Substituir / Manter as duas / Pular, confirmação nas bibliotecas | Ok, depois da correção 3 |
| 26 | Navegador de macros com as 3 macros na categoria EPLAN; prévia; Inserir na página; Ctrl+Z remove a inserção inteira (19 símbolos) | Ok |
| 26 | Peças e dispositivos: as 3 peças com fabricante, código, descrição e funções | Ok, depois da correção 5 |
| 12 | Recuperação: restaurar a cópia; sair descartando; reabrir sem a cópia | Ok, depois da correção 4 |

## Defeitos encontrados e corrigidos

1. **Etapa 25 — designações e textos das ocorrências ausentes.**
   - **Defeito:** no QElectroTech 0.7 e posteriores, o rótulo e os textos de usuário ficam na ocorrência (`dynamic_texts` e `texts_groups`), não na definição. Por isso as designações (-5F31, -5G1…) e textos como "50HZ 250VA" não apareciam.
   - **Correção** em `interchange/qet.rs`:
     - o rótulo vira o texto dinâmico de designação do símbolo, na posição da 1ª ocorrência, e acompanha renumerações;
     - os textos de usuário viram texto livre na posição transformada da ocorrência.
   - **Efeito em `industrial.qet`:** de 1 para 295 textos.
2. **Etapa 25 — setas de relatório de folha soltas e com "-K?".**
   - **Defeito:** o condutor entre o circuito e a seta virava só o ponto elétrico da interrupção, e o traço sumia da folha. Além disso, a seta mostrava a designação de um dispositivo inexistente ("-K?").
   - **Correção:**
     - o traço é mantido como linhas livres (só desenho, sem efeito elétrico; o conteúdo livre não tem polilinha);
     - o símbolo do marcador perde o texto de designação, e os estilos são reindexados.
3. **Etapa 26 (exposto pelo `.edz`; defeito de persistência mais antigo) — macro gravada que não podia ser relida.**
   - **Defeito:** a confirmação terminava com "Biblioteca: projeto inconsistente: referência ou limites de símbolo inconsistentes". Duas das três macros (BAN.85069 e SE.RM17TG00) não eram relidas.
   - **Causa:** o leitor padrão do `serde_json` pode errar o último bit de números longos. O Core compara exatamente os limites guardados na ocorrência com os recalculados da definição, então a releitura falhava. O defeito afeta qualquer arquivo com coordenadas não "redondas"; os `.edz` só o expuseram.
   - **Correção:** recurso `float_roundtrip` do `serde_json` em `crates/cad-craft/Cargo.toml`. Não é dependência nova, só um recurso da já aprovada; o `Cargo.lock` não muda por ele.
4. **Etapa 12 (encontrado neste teste) — cópia de recuperação regravada depois de "Sair" descartando.**
   - **Causa:** os quadros finais antes do fechamento ainda executavam o salvamento automático.
   - **Correção:** `CadApp::autosave` não grava depois de `exit_confirmed` (`cad-gui/src/app.rs`).
5. **Etapa 18 (exposto pelo `.edz`) — tipo de função mostrado em formato interno.**
   - **Defeito:** o editor de peças mostrava `Other(Text("EPLAN.1302/1/1"))`.
   - **Correção:** passa a mostrar o texto (`part_editor.rs`, `kind_text`).

## Testes acrescentados

- `cad-craft/tests/qet_import.rs::instance_texts_and_report_traces_are_kept`: rótulo e texto da ocorrência; traços dos relatórios de folha.
- `cad-craft/tests/persistence.rs::json_floats_round_trip_bit_exactly`: falha sem `float_roundtrip`, conferido.
- `cad-gui/tests/project_lifecycle.rs::confirmed_exit_discards_the_recovery_copy_for_good`: falha sem a correção, conferido.
- `cad-craft/tests/edz_import.rs` (teste opcional das amostras): grava e relê cada peça e macro. Com `CAD_CRAFT_EDZ_SAMPLES`, os 45 pacotes convertidos passam; RIT.2383210 continua recusado (EDZ-L08).
- Amostras `.qet` (27/27) importam; `industrial.qet` leva de 2 s a 4,5 s em Release, contra a meta de 10 s.

## Observações sem correção nesta entrega

- **Prévia do `.edz` longa demais:** uma linha por código PXF e por propriedade preservada (centenas por pacote). Funciona, mas é cansativo de ler; agrupar fica como sugestão (EDZ-L09).
- **Cópias de recuperação antigas:** "Validacao Etapa 13" e "Macros", de testes anteriores, continuam em `%LOCALAPPDATA%\CAD-CRAFT\recovery` e aparecem ao abrir o app. A cópia criada neste teste foi removida.

## Validação

`scripts/validate.ps1` (2026-11-05) terminou com exit 0: **669 testes Rust** (666 da Etapa 26 + 3 novos), zero falhas, e **15 testes PowerShell**. Release: `target/release/cad-gui.exe` com 20.088.832 bytes.
