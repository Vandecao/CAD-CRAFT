// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::interchange::edz::{self, EdzContainer, EdzLimits};
use std::io::{Cursor, Write};

fn zip(items: &[(&str, &[u8])]) -> Vec<u8> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in items {
        z.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        z.write_all(bytes).unwrap();
    }
    z.finish().unwrap().into_inner()
}
const MANIFEST: &[u8] = br#"<manifest version="2.0"><packages><package type="part" key="TEST.1"><items><item type="partxml" name="part" locator="test.part.xml"/><item type="partxml" name="manufacturer" locator="test.manufacturer.xml"/><item type="partxml" name="connectionpoints" locator="test.connectionpoints.xml"/><item type="picture" name="picture" locator="test.png"/></items></package></packages></manifest>"#;
const PART: &[u8] = br#"<partsmanagement><part P_ARTICLE_PARTNR="TEST.1" P_ARTICLE_MANUFACTURER="T" P_ARTICLE_DESCR1="de_DE@Deutsch;en_US@English;pt_BR@Portugues;" P_ARTICLE_PRODUCTGROUP="Relay" P_ARTICLE_UNKNOWN="original"><variant><functiontemplate functiondefcategory="999" functiondefgroup="2" functiondefid="3" mainfunction="1" connectionDesignation="A1;A2"/></variant></part></partsmanagement>"#;
const MANUFACTURER: &[u8] = br#"<partsmanagement><address P_PART_ADDRESS_SHORTNAME="T" P_PART_ADDRESS_LONGNAME="Test Manufacturer"/></partsmanagement>"#;
const POINTS: &[u8] = br#"<partsmanagement><terminal P_PART_TERMINAL_NAME="TEST.1"><terminalPosition name="A1" pos="1" xpos="2" ypos="4"/><terminalPosition name="A2" pos="2" xpos="2" ypos="8"/></terminal></partsmanagement>"#;
fn items() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("manifest.xml", MANIFEST),
        ("items/partxml/test.part.xml", PART),
        ("items/partxml/test.manufacturer.xml", MANUFACTURER),
        ("items/partxml/test.connectionpoints.xml", POINTS),
        ("items/picture/test.png", b"ignored"),
    ]
}
#[test]
fn zip_and_sevenz_detect_signature_and_map_parts_equally() {
    let z = edz::import(&zip(&items())).unwrap();
    let mut writer = sevenz_rust2::ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    for (name, bytes) in items() {
        writer
            .push_archive_entry(
                sevenz_rust2::ArchiveEntry::new_file(name),
                Some(Cursor::new(bytes)),
            )
            .unwrap();
    }
    let s = edz::import(&writer.finish().unwrap().into_inner()).unwrap();
    assert_eq!(z.container, EdzContainer::Zip);
    assert_eq!(s.container, EdzContainer::SevenZ);
    assert_eq!(z.parts, s.parts);
    let p = &z.parts[0];
    assert_eq!(p.code().as_str(), "TEST.1");
    assert_eq!(p.manufacturer().as_str(), "Test Manufacturer");
    assert_eq!(p.description().as_str(), "Portugues");
    assert_eq!(
        p.functions()[0]
            .points()
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>(),
        ["A1", "A2"]
    );
    assert_eq!(
        p.functions()[0].role(),
        cad_craft::domain::FunctionRole::Main
    );
    assert!(
        p.properties()
            .custom()
            .keys()
            .any(|k| k.as_str() == "P_ARTICLE_UNKNOWN")
    );
    assert!(
        p.properties()
            .custom()
            .keys()
            .any(|k| k.as_str().contains("terminalPosition"))
    );
    assert!(z.report.lines().iter().any(|l| l.contains("picture")));
    let bytes = cad_craft::persistence::write_part_package(p).unwrap();
    assert_eq!(
        &cad_craft::persistence::read_part_package(&bytes).unwrap(),
        p
    );
}
#[test]
fn rejects_invalid_container_paths_limits_truncation_and_missing_items() {
    assert!(
        edz::import(b"not archive")
            .unwrap_err()
            .to_string()
            .contains("assinatura")
    );
    for path in [
        "../bad",
        "/bad",
        "C:/bad",
        "items/../../bad",
        "\\\\server\\bad",
    ] {
        assert!(
            edz::import(&zip(&[("manifest.xml", MANIFEST), (path, b"bad")])).is_err(),
            "{path}"
        );
    }
    let bytes = zip(&items());
    assert!(edz::import(&bytes[..bytes.len() / 2]).is_err());
    for limits in [
        EdzLimits {
            max_entries: 1,
            ..Default::default()
        },
        EdzLimits {
            max_item_bytes: 8,
            ..Default::default()
        },
        EdzLimits {
            max_total_bytes: 8,
            ..Default::default()
        },
        EdzLimits {
            max_input_bytes: 8,
            ..Default::default()
        },
    ] {
        assert!(edz::import_with_limits(&bytes, &limits).is_err());
    }
    assert!(edz::import(&zip(&[("manifest.xml", MANIFEST)])).is_err());
    assert!(edz::import(&zip(&[("manifest.xml", b"<manifest><bad></manifest>")])).is_err());
    let mut bad = items();
    bad[1].1 = b"<partsmanagement><part></partsmanagement>";
    assert!(edz::import(&zip(&bad)).is_err());
    assert!(
        edz::import(&zip(&[
            ("manifest.xml", b"<manifest/>"),
            ("MANIFEST.XML", b"<manifest/>")
        ]))
        .is_err()
    );
}

/// Amostras locais opcionais: só contagens/códigos no relatório, nunca dados de peça.
#[test]
fn reference_edz_samples_generate_coverage_when_available() {
    use cad_craft::interchange::NoteKind;
    // Amostras reais fora do Git: `CADCRAFT_SAMPLES/eplan/edz`.
    let Some(dir) = std::env::var_os("CADCRAFT_SAMPLES")
        .map(|root| std::path::Path::new(&root).join("eplan").join("edz"))
        .filter(|d| d.is_dir())
    else {
        eprintln!("CADCRAFT_SAMPLES/eplan/edz ausente: amostras opcionais não executadas");
        return;
    };
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("edz")))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    let mut table = String::from(
        "# Etapa 26 — cobertura EDZ\n\nAmostras locais externas ao repositório. Tabela contém somente nomes de pacotes, contagens e códigos; não incorpora conteúdo dos fabricantes. Definições gráficas incluem desenhos livres convertidos, não são definições ESymLib. A pasta é inventariada a cada execução; o total pode diferir do levantamento inicial de 47 amostras.\n\n| Pacote | Contêiner | Peças | Macros | Definições gráficas | Ignorados | Aproximações | Códigos PXF / diagnóstico |\n|---|---|---:|---:|---:|---:|---:|---|\n",
    );
    let mut failures = Vec::new();
    let mut count = 0;
    let mut refused = 0;
    for path in paths {
        let name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace('|', "_");
        let bytes = std::fs::read(&path).unwrap();
        match edz::import(&bytes) {
            Ok(i) => {
                // Teste no app (2026-11-05): o que vai para a biblioteca precisa ser
                // relido (peças e macros).
                for m in &i.macros {
                    let bytes = cad_craft::persistence::write_macro_package(m).unwrap();
                    if let Err(e) = cad_craft::persistence::read_macro_package(&bytes) {
                        failures.push(format!("{name}: macro não relida: {e}"));
                    }
                }
                for p in &i.parts {
                    let bytes = cad_craft::persistence::write_part_package(p).unwrap();
                    if let Err(e) = cad_craft::persistence::read_part_package(&bytes) {
                        failures.push(format!("{name}: peça não relida: {e}"));
                    }
                }
                let symbols: usize = i
                    .macros
                    .iter()
                    .map(|m| m.content().definitions().len())
                    .sum();
                let row = format!(
                    "| {name} | {:?} | {} | {} | {symbols} | {} | {} | {} |\n",
                    i.container,
                    i.parts.len(),
                    i.macros.len(),
                    i.report.count(NoteKind::Ignored),
                    i.report.count(NoteKind::Approximated),
                    i.unknown_codes.into_iter().collect::<Vec<_>>().join(", ")
                );
                println!("{row}");
                table.push_str(&row);
                count += 1;
            }
            Err(e) => {
                // Amostra real comprovadamente incompleta: preservar a recusa, nunca
                // converter parcialmente nem enfraquecer os testes sintéticos de manifesto.
                let known_missing = name == "RIT.2383210.edz"
                    && e.0.starts_with("manifesto aponta item inexistente:");
                if known_missing {
                    refused += 1;
                } else {
                    failures.push(format!("{name}: {e}"));
                }
                let container = if bytes.starts_with(b"PK") {
                    "ZIP"
                } else if bytes.starts_with(b"7z") {
                    "7z"
                } else {
                    "desconhecido"
                };
                table.push_str(&format!("| {name} | {container} / recusado | 0 | 0 | 0 | 0 | 0 | MANIFEST_ITEM_MISSING |\n"));
            }
        }
    }
    table.push_str(&format!(
        "\nResultado: {count} pacotes convertidos; {refused} recusado(s) por referência necessária ausente; {} erros inesperados. Homologação manual pendente.\n",
        failures.len()
    ));
    let report = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/internal/reports/ETAPA_26_COBERTURA_EDZ.md");
    std::fs::write(report, table).unwrap();
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn controls_in_display_text_are_normalized_but_raw_properties_survive() {
    let part=br#"<partsmanagement><part P_ARTICLE_PARTNR="TEST.1" P_ARTICLE_MANUFACTURER="Test" P_ARTICLE_DESCR1="en_US@First&#10;second;" P_ARTICLE_PRODUCTGROUP="Relay"/></partsmanagement>"#;
    let mut content = items();
    content[1].1 = part;
    let imported = edz::import(&zip(&content)).unwrap();
    assert_eq!(imported.parts[0].description().as_str(), "First second");
    let original = imported.parts[0]
        .properties()
        .custom()
        .iter()
        .find(|(k, _)| k.as_str() == "P_ARTICLE_DESCR1")
        .unwrap()
        .1;
    assert!(matches!(original,cad_craft::domain::PropertyValue::Text(v) if v.contains('\n')));
    assert!(
        imported
            .report
            .count(cad_craft::interchange::NoteKind::Approximated)
            > 0
    );
}
#[test]
fn missing_out_of_scope_3d_is_reported_but_required_part_is_rejected() {
    let manifest=br#"<manifest version="2.0"><package type="part" key="T"><item type="macro" name="graphicmacro" locator="test_3D.ema"/><item type="picture" name="empty" locator=""/><item type="partxml" name="part" locator="test.part.xml"/></package></manifest>"#;
    let imported = edz::import(&zip(&[
        ("manifest.xml", manifest),
        ("items/partxml/test.part.xml", PART),
    ]))
    .unwrap();
    assert_eq!(imported.parts.len(), 1);
    assert!(
        imported
            .report
            .lines()
            .iter()
            .any(|l| l.contains("ausente"))
    );
    assert!(imported.report.lines().iter().any(|l| l.contains("vazia")));
    assert!(edz::import(&zip(&[("manifest.xml", manifest)])).is_err());
}
#[test]
fn sevenz_paths_truncation_and_output_limits_are_enforced() {
    fn sevenz(name: &str, bytes: &[u8]) -> Vec<u8> {
        let mut writer = sevenz_rust2::ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
        writer
            .push_archive_entry(
                sevenz_rust2::ArchiveEntry::new_file(name),
                Some(Cursor::new(bytes)),
            )
            .unwrap();
        writer.finish().unwrap().into_inner()
    }
    for name in ["../escape", "/absolute", "C:\\escape"] {
        assert!(edz::import(&sevenz(name, b"test")).is_err());
    }
    let bytes = sevenz("manifest.xml", MANIFEST);
    assert!(edz::import(&bytes[..bytes.len() / 2]).is_err());
    assert!(
        edz::import_with_limits(
            &bytes,
            &EdzLimits {
                max_item_bytes: 16,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        edz::import_with_limits(
            &bytes,
            &EdzLimits {
                max_total_bytes: 16,
                ..Default::default()
            }
        )
        .is_err()
    );
}
