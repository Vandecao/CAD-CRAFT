# CAD-CRAFT

**CAD elétrico de código aberto para painéis industriais, feito para trocar projetos entre plataformas.**

[English](README.md) · Licença: [GPL-3.0-or-later](LICENSE) · Estado: **alfa**

O CAD-CRAFT é um aplicativo para Windows, escrito em Rust, para desenhar e documentar esquemas elétricos industriais: dispositivos, ligações, bornes, cabos, referências cruzadas, relatórios e verificações. O objetivo é que os projetos não fiquem presos a um único fornecedor: ele lê e grava formatos de outras plataformas de engenharia elétrica em um modelo de projeto neutro.

> Uma captura de tela do programa será adicionada aqui.

## Intercâmbio entre plataformas

Tudo entra no modelo neutro `Project` e tudo sai dele. Cada importação relata o que foi perdido ou aproximado.

### Importação

| Origem | Extensão | O que é importado | Estado |
|---|---|---|---|
| QElectroTech | `.qet` | Projeto: folhas, molduras, elementos, condutores com número de fio, relatórios de folha, textos | Implementado |
| QElectroTech | `.qetz` | Projeto compactado | Planejado |
| QElectroTech | `.elmt` | Símbolo (elemento) | Implementado |
| QElectroTech | `.titleblock` | Moldura / carimbo | Implementado |
| EPLAN | `.edz` | Peças e macros 2D de pacotes de fabricantes | Implementado (definições de símbolo ESymLib embutidas dependem de amostra) |
| EPLAN | `.fn1` | Moldura (formulário de folha) | Implementado |
| EPLAN | projeto completo (`.elk`, `.zw1`, `.epj`) | Projeto | **Ainda não**; próximo grande marco do intercâmbio |
| CAD genérico | `.dxf` | Moldura | Implementado |
| Planilha | `.csv` | Catálogo de peças | Implementado |

### Exportação

| Destino | Extensão |
|---|---|
| PDF vetorial multipágina, com links e marcadores; impressão | `.pdf` |
| DXF R12, um arquivo por página | `.dxf` |
| DWG, pelo ODA File Converter instalado pelo usuário | `.dwg` |
| Listas e relatórios | `.csv`, `.xlsx` |
| Moldura | `.dxf` |

### Formatos próprios

| Extensão | Conteúdo |
|---|---|
| `.cadcraft` | Projeto (ZIP com manifesto e JSON; versionado, com migração de todas as versões anteriores) |
| `.cadsym` | Símbolo |
| `.cadmacro` | Macro |
| `.cadpart` | Peça |
| `.cadframe` | Moldura |

*EPLAN é marca registrada da EPLAN GmbH & Co. KG. QElectroTech é um projeto independente. O CAD-CRAFT não é afiliado a nenhum deles e não distribui arquivos, bibliotecas ou dados dessas plataformas: o usuário abre os próprios arquivos.*

## Funcionalidades

- Editor de esquemas com símbolos, portas tipadas e rotas de ligação ortogonais.
- Dispositivos, funções e peças separados das representações gráficas; ligações lógicas separadas dos condutores físicos; redes derivadas da conectividade.
- Bibliotecas e editores de símbolos, peças e macros.
- Molduras com zonas e referências cruzadas por coluna; editor de molduras.
- Numeração de dispositivos, fios e páginas, com números travados.
- Réguas de bornes, pontes, cabos, veias e conectores industriais.
- Relatórios em página (índice, dispositivos, materiais, ligações, bornes, cabos) e listas CSV/XLSX.
- Verificações de consistência com navegação até cada problema.
- Desfazer e refazer transacional em todas as edições.
- Ajuda e assistente de IA integrados (opcional): entre com a sua conta ChatGPT (OAuth no navegador; nenhuma senha ou chave de API é pedida ou salva). Ele responde só sobre o projeto aberto e o uso do CAD-CRAFT, localiza comandos e itens e propõe alterações que você revisa e aplica (com Desfazer). Dados do projeto só vão para a OpenAI quando você faz uma pergunta. Opcionalmente, "Manter conectado" guarda só o token de renovação da sessão no Gerenciador de Credenciais do Windows, para a próxima abertura entrar sozinha; "Sair" apaga.

## Compilar e rodar

Requisitos: Windows 10/11 x64, [Rustup](https://rustup.rs) e Microsoft C++ Build Tools (carga Desktop C++ com o Windows SDK). A versão do Rust está fixada em `rust-toolchain.toml` e o Rustup a instala sozinho.

```powershell
cargo build --release --locked -p cadcraft-gui
.\target\release\cad-gui.exe
```

Validação completa (formatação, compilação, testes, Clippy, espaços e os testes PowerShell):

```powershell
pwsh -NoProfile -File scripts/validate.ps1
```

As crates do Core não dependem da interface e também compilam e passam nos testes no Linux: `cargo test --workspace --exclude cadcraft-gui`.

## Estado do projeto

Alfa. O formato `.cadcraft` ainda pode mudar, sempre com migração das versões anteriores. O desenvolvimento segue etapas numeradas, cada uma com especificação, plano, registro de decisão de arquitetura (ADR) e relatório.

## Organização do repositório

Tudo entra no modelo neutro e tudo sai dele:

```
arquivos de outras plataformas ──► cadcraft-import ──► cadcraft-model ◄──► cadcraft-format (.cadcraft)
                                                            │
                                                            └──► cadcraft-render ──► tela, cadcraft-export (PDF, DXF)
```

- `crates/cadcraft-model`: modelo de projeto neutro (entidades, conectividade, documentos, símbolos, numeração, relatórios, verificações).
- `crates/cadcraft-format`: arquivos próprios e migrações.
- `crates/cadcraft-import`: importadores de QElectroTech, EPLAN, DXF e CSV.
- `crates/cadcraft-render`: lista de exibição comum à tela, ao PDF e ao DXF.
- `crates/cadcraft-export`: exportadores de PDF, DXF e CSV/XLSX.
- `crates/cadcraft-assistant`: ajuda, consultas ao projeto e propostas do assistente (sem rede).
- `crates/cad-craft`: fachada que reexporta as crates acima.
- `crates/cadcraft-gui`: aplicativo (egui/eframe).
- `docs/adr`: registros de decisões de arquitetura.
- `docs/TESTES_COM_AMOSTRAS.md`: testes opcionais com os seus próprios arquivos reais.
- `docs/internal`: especificações, planos, relatórios e memória de cada etapa.

## Documentação para começar

[Arquitetura](docs/ARCHITECTURE.md) · [Tour pelo código](docs/TOUR.md) · [Glossário (inglês / português)](docs/GLOSSARY.md) · [Formatos de arquivo](docs/FORMATS.md)

## Contribuir e licença

Contribuições são bem-vindas. Leia o [CONTRIBUTING.md](CONTRIBUTING.md), o [código de conduta](CODE_OF_CONDUCT.md) e a [política de segurança](SECURITY.md).

O CAD-CRAFT é software livre: você pode redistribuí-lo e/ou modificá-lo nos termos da Licença Pública Geral GNU, publicada pela Free Software Foundation, na versão 3 da Licença ou (a seu critério) em qualquer versão posterior. Veja o arquivo [LICENSE](LICENSE).
