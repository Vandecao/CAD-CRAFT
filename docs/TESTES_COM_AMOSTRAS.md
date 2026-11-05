# Testes com amostras reais

Os testes automáticos do CAD-CRAFT usam só arquivos **sintéticos**, escritos à mão para cada formato (`crates/cad-craft/tests/fixtures/synthetic/`), ou gerados pelo próprio teste. Arquivos reais de outras plataformas (molduras do EPLAN, pacotes `.edz` de fabricantes, projetos do QElectroTech) podem ser proprietários ou conter dados pessoais. Por isso **nunca entram no repositório**.

Quem tiver os próprios arquivos pode rodar também os testes opcionais, que importam cada amostra e conferem que nada falha.

## Como montar a pasta

Crie uma pasta em qualquer lugar fora do repositório, com esta estrutura:

```
<pasta de amostras>/
├── eplan/
│   ├── fn1/        molduras do EPLAN (.fn1), em qualquer subpasta
│   └── edz/        pacotes de fabricantes do EPLAN Data Portal (.edz)
├── qet/            projetos do QElectroTech (.qet)
├── elmt/           coleção de elementos do QElectroTech (.elmt), em subpastas
└── titleblock/     modelos de carimbo do QElectroTech (.titleblock)
```

As subpastas ausentes são ignoradas. Atalhos (junções do Windows ou links simbólicos) para pastas que você já tem também funcionam, por exemplo:

```powershell
New-Item -ItemType Junction -Path <pasta de amostras>\qet -Target <sua pasta de projetos .qet>
```

Se preferir deixar a pasta dentro do repositório, use `samples/` na raiz (aponte `CADCRAFT_SAMPLES` para ela): ela está no `.gitignore`.

## Como rodar

```powershell
$env:CADCRAFT_SAMPLES = "<pasta de amostras>"
cargo test --release -p cad-craft --test interchange --test elmt_import --test qet_import --test edz_import
```

Sem a variável, os testes opcionais passam sem rodar. Os testes de `.qet` e `.edz` imprimem uma tabela de cobertura (contagens apenas) com `-- --nocapture`. O de `.edz` grava `docs/internal/reports/ETAPA_26_COBERTURA_EDZ.md`, que traz só nomes de pacotes, contagens e códigos, sem conteúdo dos fabricantes.

## Encontrou um arquivo que não importa?

Abra uma issue com a versão do programa de origem, o tipo de arquivo e o relatório de importação. Se puder, anexe um arquivo **mínimo** que reproduza o problema, sem conteúdo proprietário.
