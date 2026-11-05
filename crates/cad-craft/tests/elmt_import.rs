// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 17: importação de elementos `.elmt` do QElectroTech.
//! A coleção de referência (CC-BY 3.0) não é versionada: o teste da amostra
//! real só roda quando a pasta local existe. Não é usada como dado de IA.
use cad_craft::domain::*;
use cad_craft::interchange::elmt::{DEFAULT_MM_PER_PX, ElmtOptions, import};

const SYNTHETIC: &str = r##"<definition version="0.100.0" type="element" link_type="master" width="40" height="60" hotspot_x="20" hotspot_y="32">
    <uuid uuid="{11111111-2222-3333-4444-555555555555}"/>
    <names><name lang="en">Coil</name><name lang="pt_BR">Bobina</name></names>
    <description>
        <rect x="-14" y="-8" width="28" height="16" rx="0" ry="0" style="line-style:normal;line-weight:normal;filling:none;color:black"/>
        <line x1="0" y1="-20" x2="0" y2="-8" end1="none" end2="none" style="line-style:dashed;line-weight:hight;filling:none;color:black"/>
        <ellipse x="-2" y="-2" width="4" height="8" style="line-style:normal;line-weight:normal;filling:black;color:black"/>
        <polygon x1="-1" y1="-4" x2="-1" y2="4" x3="9" y3="0" style="line-style:normal;line-weight:normal;filling:none;color:black"/>
        <arc x="-3" y="-3" width="6" height="6" start="270" angle="180" style="line-style:normal;line-weight:normal;filling:none;color:black"/>
        <text text="PE" x="7" y="5" rotation="0" font="Liberation Sans,5,-1,5,50,0,0,0,0,0,Regular" color="#000000"/>
        <dynamic_text x="25" y="-9" Halignment="AlignLeft" Valignment="AlignTop" rotation="0" text_from="ElementInfo" font="Liberation Sans,9,-1,5,50,0,0,0,0,0,Regular"><text></text><info_name>label</info_name></dynamic_text>
        <dynamic_text x="2" y="11" Halignment="AlignLeft" Valignment="AlignTop" rotation="0" text_from="UserText" font="Liberation Sans,4,-1,5,25,0,0,0,0,0,Regular"><text>A2</text></dynamic_text>
        <terminal name="A1" x="0" y="-20" orientation="n" type="Generic"/>
        <terminal name="A2" x="0" y="20" orientation="s" type="Generic"/>
        <input x="0" y="0" text="_"/>
    </description>
</definition>"##;

#[test]
fn synthetic_element_converts_geometry_ports_texts_and_styles() {
    let imported = import(SYNTHETIC.as_bytes(), &ElmtOptions::new("bobina")).unwrap();
    let d = imported.definition;
    assert_eq!(d.name(), "Bobina");
    assert_eq!(d.id().as_str(), "qet.11111111-2222-3333-4444-555555555555");
    let s = DEFAULT_MM_PER_PX;
    // Retângulo: y = -8 (para baixo) → topo em +4 mm, base em -4 mm.
    let SymbolPrimitive::Rectangle { rect } = &d.primitives()[0] else {
        panic!("retângulo")
    };
    assert_eq!(
        (rect.origin().x(), rect.origin().y()),
        (-14.0 * s, -8.0 * s)
    );
    assert_eq!(
        (rect.size().width(), rect.size().height()),
        (28.0 * s, 16.0 * s)
    );
    // Linha tracejada e espessa; elipse preenchida; polígono fechado.
    assert_eq!(d.style(1).dash, LineDash::Dashed);
    assert!(d.style(1).width_mm.is_some());
    assert!(matches!(d.primitives()[2], SymbolPrimitive::Ellipse { .. }));
    assert!(d.style(2).fill);
    assert!(matches!(
        d.primitives()[3],
        SymbolPrimitive::Polyline { closed: true, .. }
    ));
    assert!(d.primitives().iter().any(|p| matches!(
        p,
        SymbolPrimitive::DynamicText {
            field: SymbolTextField::Designation,
            ..
        }
    )));
    assert!(
        d.primitives()
            .iter()
            .any(|p| matches!(p, SymbolPrimitive::Text { text, .. } if text == "A2"))
    );
    // Terminais: A1 em cima (Y+) apontando para cima, A2 embaixo.
    let a1 = d.ports().iter().find(|p| p.id() == "A1").unwrap();
    assert_eq!(a1.position().y(), 20.0 * s);
    assert_eq!(a1.orientation_degrees(), 90.0);
    assert_eq!(
        d.ports()
            .iter()
            .find(|p| p.id() == "A2")
            .unwrap()
            .orientation_degrees(),
        270.0
    );
    // Entrada QET desconhecida vai para o relatório.
    assert!(imported.report.lines().iter().any(|l| l.contains("input")));
    // Escala ajustável.
    let mut options = ElmtOptions::new("bobina");
    options.mm_per_px = 1.0;
    let big = import(SYNTHETIC.as_bytes(), &options).unwrap().definition;
    assert!((big.bounds().size().width() - d.bounds().size().width() * 2.0).abs() < 1e-6);
}

#[test]
fn invalid_elements_are_refused() {
    assert!(import(b"<titleblocktemplate/>", &ElmtOptions::new("x")).is_err());
    assert!(import(b"nao e xml", &ElmtOptions::new("x")).is_err());
    let empty = r#"<definition type="element"><description></description></definition>"#;
    assert!(import(empty.as_bytes(), &ElmtOptions::new("x")).is_err());
    let xxe = r#"<?xml version="1.0"?><!DOCTYPE d [<!ENTITY e SYSTEM "file:///x">]><definition type="element"><description>&e;</description></definition>"#;
    assert!(import(xxe.as_bytes(), &ElmtOptions::new("x")).is_err());
}

/// Amostra real (≥ 50 elementos) da coleção de referência, quando presente.
#[test]
fn reference_collection_sample_imports_without_errors() {
    // Coleção de elementos fora do Git: `CADCRAFT_SAMPLES/elmt`.
    let Some(root) =
        std::env::var_os("CADCRAFT_SAMPLES").map(|r| std::path::Path::new(&r).join("elmt"))
    else {
        return;
    };
    if !root.exists() {
        eprintln!("coleção de referência ausente: teste pulado");
        return;
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "elmt") {
                files.push(path);
            }
        }
    }
    files.sort();
    let sample: Vec<_> = files
        .iter()
        .step_by((files.len() / 200).max(1))
        .take(200)
        .collect();
    assert!(sample.len() >= 50, "amostra pequena: {}", sample.len());
    let mut failures = Vec::new();
    let mut with_ports = 0;
    for path in &sample {
        let bytes = std::fs::read(path).unwrap();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        match import(&bytes, &ElmtOptions::new(stem)) {
            Ok(s) => {
                s.definition.validate().unwrap();
                with_ports += usize::from(!s.definition.ports().is_empty());
            }
            Err(e) => failures.push(format!("{}: {e}", path.display())),
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
    assert!(
        with_ports * 2 > sample.len(),
        "maioria com portas: {with_ports}/{}",
        sample.len()
    );
}
