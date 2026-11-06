// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Import QElectroTech .qet projects. Optional external examples run only with CADCRAFT_SAMPLES
//! pointing to a local collection containing qet/.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;
use cad_craft::interchange::NoteKind;
use cad_craft::interchange::qet::{QetOptions, import};
use cad_craft::pdf::{self, PdfOptions, PdfScope};
use cad_craft::persistence;
use std::time::Instant;

const STYLE: &str = "line-style:normal;line-weight:normal;filling:none;color:black";

fn element(name: &str, link: &str, kind: &str, terminals: &[(&str, i32, i32, &str)]) -> String {
    let terms: String = terminals
        .iter()
        .map(|(n, x, y, o)| {
            format!(r#"<terminal name="{n}" x="{x}" y="{y}" orientation="{o}" type="Generic"/>"#)
        })
        .collect();
    let kind = if kind.is_empty() {
        String::new()
    } else {
        format!(
            r#"<kindInformations><kindInformation name="type" show="1">{kind}</kindInformation></kindInformations>"#
        )
    };
    format!(
        r#"<element name="{name}.elmt"><definition version="0.80" type="element" link_type="{link}" width="20" height="40" hotspot_x="10" hotspot_y="20">{kind}<names><name lang="en">{name}</name></names><description><rect x="-5" y="-10" width="10" height="20" style="{STYLE}"/>{terms}</description></definition></element>"#
    )
}

fn instance(
    def: &str,
    uuid: &str,
    (x, y): (i32, i32),
    orientation: u32,
    label: &str,
    terminals: &[(&str, i32, i32)],
    links: &[&str],
) -> String {
    let terms: String = terminals
        .iter()
        .map(|(id, tx, ty)| format!(r#"<terminal id="{id}" x="{tx}" y="{ty}" orientation="0"/>"#))
        .collect();
    let links: String = links
        .iter()
        .map(|u| format!(r#"<link_uuid uuid="{{{u}}}"/>"#))
        .collect();
    let info = if label.is_empty() {
        String::new()
    } else {
        format!(
            r#"<elementInformations><elementInformation name="label" show="1">{label}</elementInformation></elementInformations>"#
        )
    };
    format!(
        r#"<element type="embed://import/demo/{def}.elmt" uuid="{{{uuid}}}" x="{x}" y="{y}" orientation="{orientation}" prefix="">
<terminals>{terms}</terminals>{info}<links_uuids>{links}</links_uuids></element>"#
    )
}

const UC: &str = "00000000-0000-0000-0000-00000000000c";
const UK: &str = "00000000-0000-0000-0000-00000000000a";
const UR1: &str = "00000000-0000-0000-0000-0000000000a1";
const UR2: &str = "00000000-0000-0000-0000-0000000000a2";
const UL: &str = "00000000-0000-0000-0000-0000000000b1";

fn synthetic(embedded_template: bool) -> String {
    let collection = [
        element(
            "coil",
            "master",
            "coil",
            &[("A1", 0, -20, "n"), ("A2", 0, 20, "s")],
        ),
        element(
            "contact",
            "slave",
            "power",
            &[("1", 0, -20, "n"), ("2", 0, 20, "s")],
        ),
        element("report_out", "next_report", "", &[("p", 0, 0, "w")]),
        element("report_in", "previous_report", "", &[("p", 0, 0, "e")]),
        element(
            "lamp",
            "simple",
            "",
            &[("X1", 0, -20, "n"), ("X2", 0, 20, "s")],
        ),
        element("plate", "simple", "", &[]),
    ]
    .concat();
    let template = if embedded_template {
        r#"<titleblocktemplates><titleblocktemplate name="demo"><grid cols="100px;100px" rows="30px;30px"><field row="0" col="0"><value><translation lang="en">%{title}</translation></value></field><field row="1" col="1"><value><translation lang="en">%{folio}</translation></value></field></grid></titleblocktemplate></titleblocktemplates>"#
    } else {
        ""
    };
    // Sheet 1 contains master coil K1, its slave contact and a report to sheet 2.
    let page1 = [
        instance(
            "coil",
            UK,
            (100, 200),
            0,
            "-K1",
            &[("1", 0, -20), ("2", 0, 20)],
            &[UC],
        ),
        instance(
            "contact",
            UC,
            (300, 200),
            1,
            "",
            &[("3", 0, -20), ("4", 0, 20)],
            &[UK],
        ),
        instance("report_out", UR1, (500, 300), 0, "", &[("5", 0, 0)], &[UR2]),
    ]
    .concat();
    let page2 = [
        instance("report_in", UR2, (100, 100), 0, "", &[("1", 0, 0)], &[UR1]),
        instance(
            "lamp",
            UL,
            (300, 100),
            2,
            "-H1",
            &[("2", 0, -20), ("3", 0, 20)],
            &[],
        ),
        instance(
            "plate",
            "00000000-0000-0000-0000-0000000000d1",
            (400, 300),
            0,
            "",
            &[],
            &[],
        ),
    ]
    .concat();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project title="Demo QET" version="0.80" folioSheetQuantity="0">
<properties/>
<newdiagrams><border cols="10" colsize="60" rows="6" rowsize="80" displaycols="true" displayrows="true"/></newdiagrams>
{template}
<diagram order="1" title="Comando" cols="10" colsize="60" rows="6" rowsize="80" height="500" displaycols="true" displayrows="false" titleblocktemplate="demo" author="Ana" date="2026-11-04" indexrev="B" folio="%id/%total">
<elements>{page1}</elements>
<conductors>
<conductor terminal1="2" terminal2="3" num="12" type="multi"><segment orientation="vertical" length="20"/><segment orientation="horizontal" length="200"/><segment orientation="vertical" length="-60"/></conductor>
<conductor terminal1="4" terminal2="5" num="13" type="multi"><segment orientation="vertical" length="20"/></conductor>
</conductors>
<inputs><input x="40" y="60" text="&lt;html&gt;&lt;body&gt;&lt;p&gt;Painel &amp;amp; comando&lt;/p&gt;&lt;/body&gt;&lt;/html&gt;" size="9" rotation="0"/></inputs>
<shapes><shape type="Rectangle"/></shapes>
</diagram>
<diagram order="2" title="Sinal" cols="10" colsize="60" rows="6" rowsize="80" displaycols="true" displayrows="true" titleblocktemplate="demo">
<elements>{page2}</elements>
<conductors><conductor terminal1="1" terminal2="2" num="" type="multi"/></conductors>
</diagram>
<collection><category name="import"><category name="demo">{collection}</category></category></collection>
</project>"#
    )
}

fn opts() -> QetOptions {
    QetOptions::default()
}

#[test]
fn synthetic_project_converts_pages_devices_wires_reports_and_texts() {
    let imported = import(synthetic(true).as_bytes(), &opts()).unwrap();
    let p = &imported.project;
    let st = &imported.stats;
    assert_eq!(p.name().as_str(), "Demo QET");
    assert_eq!(st.pages, 2);
    assert_eq!(st.symbols, 6);
    assert_eq!(st.occurrences, 4, "bobina, contato, lâmpada e placa");
    assert_eq!(st.drawings, 1, "placa sem terminais, sem dispositivo");
    assert_eq!(st.devices, 2, "o escravo fica no dispositivo do mestre");
    assert_eq!(st.sheet_reports, 1);
    assert_eq!(st.texts, 1);

    // Sheet dimensions are (2*5+20+10*60) by (2*5+20+6*80+60) pixels at 0.5 mm/px.
    let pages = page_ids(p);
    assert_eq!(pages.len(), 2);
    let page = p.page(pages[0]).unwrap();
    assert_eq!(page.name().as_str(), "Comando");
    assert_eq!((page.size().width(), page.size().height()), (315.0, 285.0));
    let frame = p.page_frame(pages[0]).unwrap();
    assert_eq!(frame.zones().columns.sizes.len(), 10);
    assert_eq!(frame.zones().rows.sizes.len(), 6);
    assert!(frame.zones().columns.labels_start);
    assert!(!frame.zones().rows.labels_start, "displayrows=false");
    assert!(frame.elements().iter().any(|e| matches!(
        e,
        FrameElement::Special {
            field: SpecialField::PageNumber,
            ..
        }
    )));
    let meta = page.metadata();
    assert_eq!(
        meta.get(&Text::new("Autor").unwrap()).map(String::as_str),
        Some("Ana")
    );
    assert_eq!(
        meta.get(&Text::new("Revisão").unwrap()).map(String::as_str),
        Some("B")
    );
    // The two sheets have different displayrows values, so they require separate frames.
    assert_ne!(page.frame(), p.page(pages[1]).unwrap().frame());

    // Master and slave share one device with a principal coil and a contact.
    let k1 = p
        .devices()
        .find(|d| d.designation().to_string() == "-K1")
        .expect("K1");
    let functions: Vec<_> = p.functions_for_device(k1.id()).collect();
    assert_eq!(functions.len(), 2);
    assert!(functions.iter().any(|f| f.kind() == &FunctionKind::Coil));
    assert!(
        functions
            .iter()
            .any(|f| f.kind() == &FunctionKind::MainContact)
    );
    assert_eq!(
        p.main_function(k1.id()).unwrap().map(|f| f.kind().clone()),
        Some(FunctionKind::Coil)
    );
    assert!(p.devices().any(|d| d.designation().to_string() == "-H1"));

    // Locked wire 12 connects K1:A2 to the contact; the report wire is omitted as a logical
    // connection.
    let numbered: Vec<_> = p
        .connections()
        .filter_map(|c| {
            c.number()
                .map(|n| (n.as_str().to_owned(), c.number_locked()))
        })
        .collect();
    assert_eq!(numbered, vec![("12".to_owned(), true)]);
    assert_eq!(
        st.conductors, 1,
        "os fios dos relatórios viram o ponto da interrupção"
    );

    // The sheet report becomes two grouped interruptions with opposite-side points.
    let markers: Vec<_> = p.markers().collect();
    assert_eq!(markers.len(), 2);
    assert!(
        markers
            .iter()
            .all(|m| m.kind() == &MarkerKind::Interruption)
    );
    assert!(markers.iter().all(|m| m.point().is_some()));
    assert_eq!(p.marker_groups().count(), 1);

    // Remove HTML markup from free text.
    let texts: Vec<String> = p
        .page_display_list(pages[0])
        .unwrap()
        .texts()
        .map(|t| t.text.clone())
        .collect();
    assert!(texts.iter().any(|t| t == "Painel & comando"), "{texts:?}");

    // Unsupported shapes are reported.
    assert!(imported.report.count(NoteKind::Ignored) >= 1);
    assert!(imported.report.lines().iter().any(|l| l.contains("forma")));
    p.validate().unwrap();
}

#[test]
fn coordinates_orientation_and_ports_follow_the_qet_sheet() {
    let imported = import(synthetic(true).as_bytes(), &opts()).unwrap();
    let p = &imported.project;
    let pages = page_ids(p);
    let h = p.page(pages[0]).unwrap().size().height();
    let reps = p.representations_on_page(pages[0]).unwrap();
    let placed: Vec<_> = reps
        .iter()
        .filter(|r| r.symbol().is_some())
        .map(|r| {
            let t = r.transform();
            (
                t.translation().x(),
                t.translation().y(),
                t.rotation_degrees(),
            )
        })
        .collect();
    // The coil is at (100,200) pixels without rotation; the contact at (300,200) has clockwise
    // orientation 1, equivalent to 270 degrees counterclockwise.
    assert!(placed.contains(&(50.0, h - 100.0, 0.0)), "{placed:?}");
    assert!(placed.contains(&(150.0, h - 100.0, 270.0)), "{placed:?}");
    // Coil port A2 at (0,20) pixels is 10 mm below the anchor.
    let coil = reps
        .iter()
        .find(|r| r.transform().translation().x() == 50.0 && r.symbol().is_some())
        .unwrap();
    let a2 = PortRef::new(coil.id(), "A2").unwrap();
    let pos = p.port_position(&a2).unwrap();
    assert_eq!((pos.x(), pos.y()), (50.0, h - 110.0));
    // Wire 12 starts at this port and follows an orthogonal route.
    let route = p
        .connection_routes()
        .find(|r| r.source() == &a2 || r.target() == &a2)
        .expect("rota do fio 12");
    let pts = route.points();
    assert!(pts.len() >= 3);
    assert!(
        pts.windows(2)
            .all(|w| w[0].x() == w[1].x() || w[0].y() == w[1].y())
    );
}

#[test]
fn missing_template_label_conflicts_and_unknown_elements_are_reported() {
    let source = synthetic(false).replace("-H1", "-K1").replace(
        "embed://import/demo/lamp.elmt",
        "embed://import/demo/sumiu.elmt",
    );
    let imported = import(source.as_bytes(), &opts()).unwrap();
    let lines = imported.report.lines().join("\n");
    assert!(lines.contains("não embutido"), "{lines}");
    assert!(lines.contains("ausente da coleção"), "{lines}");
    assert_eq!(imported.stats.occurrences, 3);
    // The page has a simple 50-pixel title block.
    let p = &imported.project;
    let page = p.page(page_ids(p)[0]).unwrap();
    assert_eq!(
        page.size().height(),
        (2.0 * 5.0 + 20.0 + 480.0 + 50.0) * 0.5
    );

    // A repeated label outside a master/slave pair still identifies the same device.
    let dup = synthetic(true).replace("-H1", "-K1");
    let imported = import(dup.as_bytes(), &opts()).unwrap();
    let names: Vec<String> = imported
        .project
        .devices()
        .map(|d| d.designation().to_string())
        .collect();
    assert_eq!(names, vec!["-K1".to_owned()]);
    let k1 = imported.project.devices().next().unwrap().id();
    assert_eq!(imported.project.functions_for_device(k1).count(), 3);
    assert!(
        imported
            .report
            .lines()
            .iter()
            .any(|l| l.contains("agrupada"))
    );
}

#[test]
fn malformed_and_hostile_input_is_rejected() {
    let o = opts();
    assert!(import(b"nao e xml", &o).is_err());
    assert!(import(b"<definition/>", &o).is_err());
    assert!(
        import(br#"<project title="x"></project>"#, &o).is_err(),
        "sem folhas"
    );
    let xxe = r#"<?xml version="1.0"?><!DOCTYPE p [<!ENTITY e SYSTEM "file:///x">]><project>&e;</project>"#;
    assert!(import(xxe.as_bytes(), &o).is_err());
    assert!(import(synthetic(true).as_bytes(), &QetOptions { mm_per_px: 0.0 }).is_err());
    // Missing or unknown wire terminals are reported without failing the import.
    let broken = synthetic(true).replace(
        r#"<conductor terminal1="4" terminal2="5""#,
        r#"<conductor terminal1="4" terminal2="99""#,
    );
    let imported = import(broken.as_bytes(), &o).unwrap();
    assert!(
        imported
            .report
            .lines()
            .iter()
            .any(|l| l.contains("terminal inexistente"))
    );
}

#[test]
fn imported_project_saves_reloads_and_exports() {
    let imported = import(synthetic(true).as_bytes(), &opts()).unwrap();
    let bytes = persistence::write_project_to_vec(&imported.project).unwrap();
    let loaded = persistence::read_project_from_slice(&bytes)
        .unwrap()
        .project;
    assert_eq!(page_ids(&loaded).len(), 2);
    assert_eq!(loaded.devices().count(), 2);
    assert_eq!(loaded.markers().count(), 2);
    let pdf = pdf::export_pdf(&loaded, &PdfOptions::default(), &mut |_, _| true).unwrap();
    assert_eq!(pdf.report.pages, 2);
    let (files, report) =
        cad_craft::dxf_export::export_dxf_pages(&loaded, &PdfScope::Project, &mut |_, _| true)
            .unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(report.pages, 2);
}

/// Read optional external QElectroTech project examples when configured.
#[test]
fn reference_samples_import_when_available() {
    let Some(dir) = std::env::var_os("CADCRAFT_SAMPLES")
        .map(|root| std::path::Path::new(&root).join("qet"))
        .filter(|d| d.is_dir())
    else {
        return;
    };
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "qet"))
        .collect();
    entries.sort();
    assert!(!entries.is_empty());
    let mut failures = vec![];
    for path in entries {
        let bytes = std::fs::read(&path).unwrap();
        let start = Instant::now();
        match import(&bytes, &opts()) {
            Ok(i) => {
                let st = &i.stats;
                eprintln!(
                    "{}\t{:.2}s\t{} folhas\t{} símbolos\t{} ocorrências\t{} dispositivos\t{} fios\t{} relatórios\t{} textos\t{} desenhos\t{} ignorados\t{} aproximados",
                    path.file_name().unwrap().to_string_lossy(),
                    start.elapsed().as_secs_f64(),
                    st.pages,
                    st.symbols,
                    st.occurrences,
                    st.devices,
                    st.conductors,
                    st.sheet_reports,
                    st.texts,
                    st.drawings,
                    i.report.count(NoteKind::Ignored),
                    i.report.count(NoteKind::Approximated),
                );
                if std::env::var_os("CAD_CRAFT_QET_VERBOSE").is_some() {
                    for line in i.report.lines() {
                        eprintln!("    {line}");
                    }
                }
            }
            Err(e) => failures.push(format!("{}: {e}", path.display())),
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

fn page_ids(p: &Project) -> Vec<PageId> {
    p.documents()
        .flat_map(|d| p.ordered_pages(d.id()).unwrap())
        .map(|page| page.id())
        .collect()
}

/// QElectroTech 0.9+ identifies conductor endpoints by element UUID and definition-terminal
/// identifier.
#[test]
fn conductors_by_element_and_terminal_uuid_are_resolved() {
    let lamp = format!(
        r#"<element name="lamp.elmt"><definition version="0.90" type="element" link_type="simple" width="20" height="40" hotspot_x="10" hotspot_y="20"><names><name lang="en">lamp</name></names><description><rect x="-5" y="-10" width="10" height="20" style="{STYLE}"/><terminal uuid="{{aaaaaaaa-0000-0000-0000-000000000001}}" name="X1" x="0" y="-20" orientation="n" type="Generic"/><terminal uuid="{{aaaaaaaa-0000-0000-0000-000000000002}}" name="X2" x="0" y="20" orientation="s" type="Generic"/></description></definition></element>"#
    );
    let inst = |uuid: &str, x: i32, label: &str| {
        format!(
            r#"<element type="embed://import/lamp.elmt" uuid="{{{uuid}}}" x="{x}" y="200" orientation="0"><terminals><terminal id="0" x="0" y="-20" orientation="0"/><terminal id="1" x="0" y="20" orientation="2"/></terminals><elementInformations><elementInformation name="label">{label}</elementInformation></elementInformations></element>"#
        )
    };
    let source = format!(
        r#"<project title="UUID" version="0.90"><diagram order="1" cols="10" colsize="60" rows="6" rowsize="80"><elements>{}{}</elements><conductors><conductor element1="{{bbbbbbbb-0000-0000-0000-000000000001}}" terminal1="{{aaaaaaaa-0000-0000-0000-000000000002}}" element2="{{bbbbbbbb-0000-0000-0000-000000000002}}" terminal2="{{aaaaaaaa-0000-0000-0000-000000000001}}" num="7"/></conductors></diagram><collection><category name="import">{lamp}</category></collection></project>"#,
        inst("bbbbbbbb-0000-0000-0000-000000000001", 100, "-H1"),
        inst("bbbbbbbb-0000-0000-0000-000000000002", 300, "-H2"),
    );
    let imported = import(source.as_bytes(), &opts()).unwrap();
    assert_eq!(
        imported.stats.conductors,
        1,
        "{:?}",
        imported.report.lines()
    );
    let p = &imported.project;
    let route = p.connection_routes().next().unwrap();
    assert_eq!(
        (route.source().port_id(), route.target().port_id()),
        ("X2", "X1")
    );
    let numbers: Vec<_> = p
        .connections()
        .filter_map(|c| c.number().map(|n| n.as_str().to_owned()))
        .collect();
    assert_eq!(numbers, vec!["7".to_owned()]);
}

/// QET 0.7+ stores labels/user text on occurrences; retain sheet-report conductor traces as free
/// drawing.
#[test]
fn instance_texts_and_report_traces_are_kept() {
    let dynamic = r#"<dynamic_texts><dynamic_elmt_text x="12" y="-8" Halignment="AlignLeft" Valignment="AlignTop" rotation="0" font="Sans Serif,9,-1,5,50,0,0,0,0,0,Regular" text_from="ElementInfo"><text></text><info_name>label</info_name></dynamic_elmt_text><dynamic_elmt_text x="12" y="10" font="Sans Serif,6,-1,5,50,0,0,0,0,0,Regular" text_from="UserText"><text>Painel 3</text></dynamic_elmt_text></dynamic_texts>"#;
    let source = synthetic(true).replace(
        r#"<elementInformation name="label" show="1">-H1</elementInformation></elementInformations>"#,
        &format!(
            r#"<elementInformation name="label" show="1">-H1</elementInformation></elementInformations>{dynamic}"#
        ),
    );
    assert!(source.contains("Painel 3"));
    let imported = import(source.as_bytes(), &opts()).unwrap();
    let p = &imported.project;
    let pages = page_ids(p);
    // The added dynamic text shows the lamp designation.
    let texts: Vec<String> = p
        .page_display_list(pages[1])
        .unwrap()
        .texts()
        .map(|t| t.text.clone())
        .collect();
    assert!(texts.iter().any(|t| t == "-H1"), "{texts:?}");
    assert!(texts.iter().any(|t| t == "Painel 3"), "{texts:?}");
    // Sheet-report conductor traces on pages 1 and 2 become free graphics.
    let traces = pages
        .iter()
        .flat_map(|page| p.representations_on_page(*page).unwrap())
        .filter(|r| {
            r.graphic()
                .is_some_and(|g| matches!(g.primitive(), SymbolPrimitive::Line { .. }))
        })
        .count();
    assert!(
        traces >= 2,
        "um traço por segmento, nas duas folhas: {traces}"
    );
    p.validate().unwrap();
}

#[test]
fn tutorial_fixture_reports_unsupported_shapes() {
    let imported = import(include_bytes!("fixtures/synthetic/entry.qet"), &opts()).unwrap();
    imported.project.validate().unwrap();
    assert_eq!(imported.project.name().as_str(), "Synthetic entry example");
    assert_eq!(imported.stats.pages, 1);
    assert_eq!(imported.stats.texts, 1);
    assert!(imported.report.count(NoteKind::Ignored) > 0);
    assert!(
        imported
            .report
            .lines()
            .iter()
            .any(|line| line.contains("shape") || line.contains("forma"))
    );
}
