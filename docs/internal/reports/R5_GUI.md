# R5 — Organização da GUI

Especificação aprovada: ../spec/R5_GUI.md. Plano: ../plans/2026-11-05-r5.md. ADR-0032. Implementação no checkout privado C:/Dev/CAD-CRAFT, main, origin Vandecao/CAD-CRAFT1. O autor autorizou R4 e R5 e publicação final nos dois repositórios; autorizou expressamente o Codex a executar o roteiro no app.

## Entrega

Onze movimentos com git mv antes das divisões, registrados em .local/r5/moves.json. App separado em coordenação, UI, arquivos e execução; Session em domínio, documentos, gráficos e clipboard. Editores de moldura, símbolo, peça e físico separados em estado, entrada, desenho e assuntos específicos. Conectores, navegador, propriedades e importação EDZ divididos. Catálogo por aba com ordem global preservada (111 comandos operacionais, 12 de clipboard livre e 53 grupos planejados).

CadApp agrupa 32 campos privados em cinco estruturas, com 190 acessos ajustados. Os campos públicos usados pelos testes e navigator usado por teste interno permanecem acessíveis. Defaults conservados; CadApp e PanelState agora derivam Default quando todos os campos usam seus defaults. Oito módulos públicos não usados pelos testes passaram a pub(crate); error é público e testado.

GuiError/GuiResult preservam categorias e fontes nas operações de projeto/recuperação, símbolos, macros, molduras, peças, QET, EDZ, PDF/DXF/DWG e relatórios. Mensagens e prefixos anteriores preservados. Resultados de apresentação clonáveis e SystemShell mantêm String na fronteira. NativeShell abre arquivos com opener, substituindo cmd /C start e os comandos equivalentes das outras plataformas. Impressão preservada.

Auditoria de pacotes nativos: os quatro codecs já estão em cadcraft-format desde R1; GUI chama read/write_*_package e mantém somente política de disco, limites e seletores. Formato 9, arquitetura, undo/redo e rótulos não foram alterados.

Dependências aprovadas: opener =0.8.5 e thiserror =2.0.21; licença MIT OR Apache-2.0 verificada também para novas dependências transitivas. thiserror já existia no lock e agora é direta da GUI. Fontes: [opener](https://docs.rs/opener/0.8.5/opener/), [thiserror](https://github.com/dtolnay/thiserror).

## Verificação

Três testes novos verificam categoria/fonte, diagnóstico com caminho especial e erro tipado na abertura de projeto. RED registrado por import inexistente de cad_gui::error antes da implementação. Os testes existentes permanecem inalterados. O teste físico antigo recebeu apenas uma anotação Clippy no módulo para manter a preparação original após ampliar a visibilidade ancestral do draft.

Comparação de corpos de editores antes da integração tipada, auditoria de atributos dos tipos e defaults em .local/r5. Uma derive de FrameTool omitida no corte foi detectada e restaurada antes da validação. Não houve mudança de fixtures ou regeneração de baselines.

pwsh -NoProfile -File scripts/validate.ps1 concluído com exit 0: 706 testes Rust do workspace + 6 exemplos executados + 16 testes PowerShell = 728, zero falhas/ignorados. fmt, check --workspace --all-targets --locked, clippy -D warnings e diff checks aprovados. Baselines e fixtures v0–v9 preservados. Revalidação completa em 2026-11-06: scripts/validate.ps1 novamente exit 0, com as mesmas contagens (706 + 6 + 16). Evidências locais: .local/r5/validate.log e validate-final.log. R4 já tinha validação independente com 703 Rust + 6 exemplos + 16 PowerShell.

Release compilado com exit 0 (cargo build -p cadcraft-gui --release --locked), 23.736.832 bytes, executado no roteiro nativo. Evidência .local/r5/release.log.

Revisão independente final: nenhum achado material. Verificados atributos/derives, defaults, sequência exata dos três catálogos, corpos dos editores de símbolo/físico e 68 métodos de Session, fluxo de operações, diagnósticos, opener e testes externos sem diff. Maior arquivo da GUI: viewport.rs, 771 linhas; todos os fontes têm SPDX e introdução de módulo. Revisão estática não substitui as verificações automáticas e o roteiro no app registrados separadamente.

## Roteiro no aplicativo (2026-11-05/06)

Executado pelo Codex por autorização expressa, usando somente cópias sintéticas em .local/r5/manual. Acessibilidade real do Release e teclado foram usados: captura de janela apresentou timeout e cliques dependeram de geometria indisponível. Estados de foco atrasados exigiram nova observação antes de confirmar ações. Esses problemas do helper não foram tratados como defeitos do CAD-CRAFT.

| Fluxo | Resultado observado |
|---|---|
| Abrir projeto | Cópia da fixture nativa v9 aberta; folha Força ativada. |
| Editar página | Nome alterado para Força R5 no painel de propriedades; navegador e aba atualizados; projeto salvo na cópia sintética. |
| Moldura | Nova Moldura 1 v1 criada; editor, ferramentas e formulário disponíveis. |
| Símbolo | Novo símbolo v1 aberto; ferramentas, portas e propriedades disponíveis. |
| Peça | Peças e dispositivos aberto; catálogo, modelo de funções e comandos CSV disponíveis. |
| Físico | Editor de fiação aberto; abas de bornes/conectores/cabos/condutores/listas e formulário presentes. |
| PDF | PDF de uma página gerado (1.562 bytes), incluindo caminho com espaços, & e apóstrofo. Com Abrir ao terminar habilitado, o app confirmou Aberto no leitor de PDF e o Edge abriu uma janela com o nome exato do arquivo. |
| DXF | Verificação_Esquema_1.dxf gravado (1.942 bytes); app confirmou uma página, seis entidades DXF e Pasta aberta. |
| QET | entry.qet sintético analisado: uma folha, um texto, uma forma ignorada e zero aproximações. Criar projeto importado instalou Synthetic entry example e abriu a folha Control; Verificações exibido conforme opção. |
| Assistente | Painel aberto no projeto importado; contexto Synthetic entry example / folha 1 e provedor ChatGPT corretos. Sem sessão autenticada, Entrar com ChatGPT disponível e editor/Enviar desabilitados, como esperado. |

A inspeção adicional do Edge foi bloqueada pelo Computer Use porque não conseguiu verificar a URL atual do navegador. Não se contornou o bloqueio; a retomada ficou no aplicativo nativo. A confirmação de abertura do PDF usa a mensagem do app e o inventário de janelas, sem alegar inspeção visual do conteúdo no Edge. Autenticação OAuth e conversa real não foram repetidas; os fluxos de conversa/propostas continuam cobertos pelos testes sintéticos aprovados. O roteiro não equivale à homologação geral da Etapa 28.

O projeto sintético anterior foi salvo antes da substituição por QET. A recuperação de outra sessão encontrada ao abrir foi cancelada sem apagar a cópia. Nenhuma biblioteca real, credencial ou amostra proprietária foi alterada/versionada. A sessão QET importada permanece em memória no app para inspeção.

## Publicação

A cópia pública usa um manifesto explícito com --no-renames: 171 caminhos (121 criados, 30 modificados, 20 removidos). A primeira checagem de fmt detectou quatro fontes antigos que a listagem padrão de renomes havia omitido; a enumeração foi corrigida e os caminhos antigos removidos antes do staging. Código privado validado não foi alterado por essa correção do procedimento.

Inventário completo de caminhos criados, modificados e removidos: [R4_R5_ARQUIVOS.md](R4_R5_ARQUIVOS.md).

HEAD privado pré-versionamento observado em 2026-11-05: 7f5463a956aaa731c77105a4142719db76ba264a. Público separado: C:/Dev/CAD-CRAFT-public, main, origin Vandecao/CAD-CRAFT, HEAD 35d7074b7bc4a5ccae781359a02ba2cbd2d5f4d5. Estes HEADs antecedem R4/R5; nenhum SHA futuro é atribuído a este documento. Publicação por commits independentes e cópia dos arquivos aprovados, sem enviar ancestralidade privada. Recibo com SHAs efetivos em .local/r4-r5/PUBLICATION.md. R6 e Etapa 28 não iniciados.
