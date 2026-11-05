# Cobertura — importação `.qet` (Etapa 25)

Data: 2026-11-04. Teste opcional `reference_samples_import_when_available` (`crates/cad-craft/tests/qet_import.rs`), executado em Release:

```
CAD_CRAFT_QET_SAMPLES=<pasta de amostras>\qet
```

Os 27 exemplos que acompanham o QElectroTech (versões 0.3 a 0.90) ficam fora do repositório. Esta tabela traz só contagens; nenhum conteúdo foi copiado. **Todos os 27 importam sem erro e passam por `Project::validate()`.**

Notas sobre as colunas:
- **Fios** são os condutores convertidos em conexão + rota. Os condutores ligados a relatórios de folha viram o ponto da interrupção.
- **Relatórios** são os pares de interrupções agrupados.
- **Desenhos** são as ocorrências sem terminais.

| Projeto | Tempo | Folhas | Símbolos | Ocorrências | Dispositivos | Fios | Relatórios | Textos | Desenhos | Ignorados | Aproximados |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2612_ats_singlephase | 0,27 s | 3 | 42 | 126 | 90 | 131 | 0 | 31 | 10 | 32 | 153 |
| 741 | 0,02 s | 1 | 9 | 39 | 39 | 67 | 0 | 0 | 0 | 35 | 0 |
| ArduinoLCD | 0,02 s | 3 | 9 | 11 | 11 | 47 | 0 | 1 | 0 | 3 | 87 |
| Easysolar-16Sbattery | 0,03 s | 1 | 23 | 54 | 54 | 77 | 0 | 43 | 0 | 23 | 1 |
| Habitat-Schemas_developpes | 0,04 s | 5 | 14 | 84 | 36 | 112 | 0 | 5 | 0 | 15 | 50 |
| Habitat-Unifilaire | 0,02 s | 1 | 18 | 35 | 25 | 32 | 0 | 17 | 0 | 8 | 16 |
| Projet_vierge | 0,02 s | 4 | 47 | 8 | 8 | 5 | 5 | 47 | 0 | 317 | 45 |
| ShellyParts | 0,01 s | 3 | 12 | 4 | 4 | 0 | 0 | 0 | 0 | 8 | 242 |
| affuteuse_250h | 0,42 s | 12 | 106 | 222 | 169 | 225 | 17 | 59 | 13 | 500 | 51 |
| cablage-eclairages_sikli-v5 | 0,02 s | 1 | 13 | 25 | 24 | 57 | 0 | 9 | 0 | 18 | 16 |
| conductor_definition_mark | 0,00 s | 1 | 3 | 5 | 5 | 4 | 0 | 0 | 0 | 0 | 0 |
| convertisseur | 0,21 s | 7 | 17 | 160 | 41 | 188 | 0 | 21 | 2 | 20 | 117 |
| câblage-éclairages-sikli-v5 | 0,02 s | 1 | 13 | 25 | 24 | 57 | 0 | 9 | 0 | 18 | 16 |
| grafcet | 0,01 s | 3 | 19 | 37 | 24 | 35 | 0 | 1 | 0 | 34 | 25 |
| industrial | 3,60 s | 50 | 66 | 395 | 290 | 345 | 105 | 1 | 48 | 126 | 620 |
| iso_sfc_example | 0,02 s | 2 | 14 | 44 | 36 | 43 | 2 | 0 | 0 | 31 | 15 |
| items_outside_sheet | 0,00 s | 2 | 1 | 3 | 3 | 0 | 0 | 1 | 0 | 0 | 0 |
| lmdg | 0,03 s | 1 | 26 | 71 | 51 | 70 | 0 | 2 | 0 | 79 | 27 |
| m_000 | 0,96 s | 19 | 91 | 318 | 241 | 301 | 49 | 2 | 35 | 798 | 224 |
| perceuse | 0,70 s | 4 | 67 | 552 | 248 | 156 | 0 | 0 | 301 | 105 | 134 |
| photovoltaique | 0,10 s | 1 | 133 | 63 | 63 | 64 | 0 | 51 | 0 | 103 | 24 |
| pinball_williams_em | 0,01 s | 1 | 61 | 61 | 61 | 0 | 0 | 7 | 0 | 38 | 0 |
| schema_indus | 0,02 s | 1 | 26 | 48 | 38 | 69 | 0 | 2 | 0 | 64 | 13 |
| schema_unifilaire_voltaique2 | 0,07 s | 1 | 32 | 71 | 71 | 76 | 0 | 34 | 0 | 24 | 30 |
| tableau_domestique | 0,14 s | 5 | 41 | 78 | 53 | 106 | 6 | 29 | 0 | 9 | 135 |
| tremie_vibrante | 0,03 s | 3 | 29 | 60 | 56 | 75 | 0 | 2 | 4 | 73 | 34 |
| weneedpolonez-Polonez_MR89 | 0,48 s | 1 | 62 | 189 | 189 | 360 | 0 | 146 | 0 | 166 | 108 |

Os ignorados e aproximados são contados por ocorrência. As causas que mais aparecem:
- **Ignorados:**
  - campos `input` dentro de elementos (sem equivalente);
  - imagens e formas na folha (QET-L03);
  - textos dinâmicos sem conteúdo;
  - condutores que apontam para elemento fora da coleção.
- **Aproximados:**
  - textos girados desenhados na horizontal e arcos elípticos convertidos em circulares (herdados do importador `.elmt`, Etapa 17);
  - cores fora da paleta;
  - rótulos repetidos agrupados no mesmo dispositivo.

**Desempenho:** `industrial.qet` (50 folhas) leva de 1,6 s a 3,6 s em Release, conforme a carga da máquina. A meta é 10 s.
