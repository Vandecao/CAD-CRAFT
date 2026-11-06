// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frame interchange contracts for EPLAN .fn1, QElectroTech .titleblock and DXF.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::{
    FrameElement, FrameTemplate, FrameTextStyle, LocalizedText, Point2D, Rect2D, Size2D,
    SpecialField, TextAlign, ZoneAxis, ZoneLayout,
};
use cad_craft::interchange::{dxf, fn1, titleblock};

fn point(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

fn synthetic_fn1(name: &str) -> cad_craft::interchange::ImportedFrame {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/synthetic/fn1")
        .join(name);
    let bytes = std::fs::read(&path).unwrap();
    fn1::import(&bytes, &mut |_| None).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A synthetic EPLAN-style frame uses displaced internal coordinates; its sheet-size border
/// moves to the origin.
#[test]
fn synthetic_eplan_frame_with_offset_origin_imports() {
    let imported = synthetic_fn1("moldura_origem_deslocada.fn1");
    let frame = &imported.package.frame;
    frame.validate().unwrap();
    assert_eq!(frame.zones().columns.sizes.len(), 10);
    assert_eq!(frame.zones().rows.sizes.len(), 4);
    assert_eq!(frame.sheet().width(), 420.0);
    assert_eq!(frame.sheet().height(), 297.0);
    for kind in ["linha", "texto", "especial"] {
        assert!(
            frame.elements().iter().any(|e| match kind {
                "linha" => matches!(e, FrameElement::Line { .. }),
                "texto" => matches!(e, FrameElement::Text { .. }),
                _ => matches!(e, FrameElement::Special { .. }),
            }),
            "{kind}"
        );
    }
    // Border (-29,31)..(391,328) maps to (0,0)..(420,297).
    assert!(frame.elements().iter().any(|e| match e {
        FrameElement::Rectangle { rect, .. } => {
            let (o, m) = (rect.origin(), rect.max());
            o.x().abs() <= 0.5
                && o.y().abs() <= 0.5
                && (m.x() - 420.0).abs() <= 0.5
                && (m.y() - 297.0).abs() <= 0.5
        }
        _ => false,
    }));
    // Line (371,35)..(391,35) maps to (400,4)..(420,4).
    assert!(frame.elements().iter().any(|e| match e {
        FrameElement::Line { start, end, .. } => {
            let xs = [start.x(), end.x()];
            xs.iter().any(|x| (x - 400.0).abs() <= 0.5)
                && xs.iter().any(|x| (x - 420.0).abs() <= 0.5)
                && (start.y() - 4.0).abs() <= 0.5
                && (end.y() - 4.0).abs() <= 0.5
        }
        _ => false,
    }));
    // Cover the page-number property 11000 and multilingual text with pt-BR as the default.
    assert!(frame.elements().iter().any(|e| matches!(
        e,
        FrameElement::Special {
            field: SpecialField::PageNumber,
            ..
        }
    )));
    assert!(frame.elements().iter().any(|e| matches!(
        e,
        FrameElement::Text { text, .. } if text.default == "Projeto"
            && text.translations.get("en-US").map(String::as_str) == Some("Project")
    )));
    let lines = imported.report.lines().join("\n");
    assert!(lines.contains("origem EPLAN ajustada"), "{lines}");
    assert!(lines.contains("rótulos de coluna/linha"), "{lines}");
}

/// Complete synthetic frame: groups, arc polyline, rotated text, project/structure fields,
/// external image and ignored objects.
#[test]
fn synthetic_eplan_frame_covers_groups_polylines_fields_and_reports() {
    let imported = synthetic_fn1("moldura_completa.fn1");
    let frame = &imported.package.frame;
    frame.validate().unwrap();
    assert_eq!(frame.zones().columns.sizes.len(), 10);
    assert!(frame.elements().iter().any(|e| match e {
        FrameElement::Rectangle { rect, .. } => {
            let (o, m) = (rect.origin(), rect.max());
            o.x().abs() <= 0.5
                && o.y().abs() <= 0.5
                && (m.x() - 420.0).abs() <= 0.5
                && (m.y() - 297.0).abs() <= 0.5
        }
        _ => false,
    }));
    assert!(
        frame
            .elements()
            .iter()
            .any(|e| matches!(e, FrameElement::Arc { .. })),
        "polilinha com bulge vira arco"
    );
    assert!(frame.elements().iter().any(|e| matches!(
        e,
        FrameElement::Text { style, .. } if (style.rotation_degrees - 90.0).abs() < 1e-6
    )));
    for field in [SpecialField::ProjectName, SpecialField::PageFunctionalPath] {
        assert!(
            frame.elements().iter().any(|e| matches!(
                e,
                FrameElement::Special { field: f, .. } if *f == field
            )),
            "{field:?}"
        );
    }
    let lines = imported.report.lines().join("\n");
    for expected in ["imagem externa", "campo de usuário"] {
        assert!(lines.contains(expected), "{expected}: {lines}");
    }
}

/// Optional real EPLAN frames stay outside Git under CADCRAFT_SAMPLES/eplan/fn1. Without the
/// variable, skip sample reading.
#[test]
fn sample_eplan_frames_import_when_available() {
    let Some(root) = std::env::var_os("CADCRAFT_SAMPLES") else {
        return;
    };
    let dir = std::path::Path::new(&root).join("eplan").join("fn1");
    if !dir.is_dir() {
        return;
    }
    let mut count = 0;
    let mut folders = vec![dir];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
                continue;
            }
            if path
                .extension()
                .is_none_or(|e| !e.eq_ignore_ascii_case("fn1"))
            {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            let imported = fn1::import(&bytes, &mut |_| None)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            imported.package.frame.validate().unwrap();
            count += 1;
        }
    }
    assert!(count > 0, "pasta eplan/fn1 sem molduras");
}

#[test]
fn dxf_round_trip_preserves_supported_geometry_and_fields() {
    let sheet = Size2D::new(420.0, 297.0).unwrap();
    let frame = FrameTemplate::new(
        "Troca",
        sheet,
        Rect2D::new(point(5.0, 5.0), Size2D::new(410.0, 287.0).unwrap()).unwrap(),
        ZoneLayout {
            columns: ZoneAxis::none(),
            rows: ZoneAxis::none(),
        },
        vec![
            FrameElement::Line {
                start: point(10.0, 10.0),
                end: point(100.0, 10.0),
                width: 0.25,
            },
            FrameElement::Circle {
                center: point(30.0, 30.0),
                radius: 4.0,
                width: 0.35,
            },
            FrameElement::Text {
                origin: point(50.0, 50.0),
                text: LocalizedText::new("Ação"),
                style: FrameTextStyle::new(2.5, TextAlign::BottomLeft),
            },
            FrameElement::Special {
                origin: point(60.0, 60.0),
                field: SpecialField::PageNumber,
                index: None,
                style: FrameTextStyle::new(2.5, TextAlign::BottomLeft),
            },
        ],
    )
    .unwrap();
    let (bytes, report) = dxf::export(&frame);
    assert_eq!(report.count(cad_craft::interchange::NoteKind::Ignored), 0);
    let back = dxf::import(&bytes, &dxf::DxfImportOptions::new("reimportada")).unwrap();
    assert_eq!(back.package.frame.sheet(), frame.sheet());
    assert_eq!(back.package.frame.usable(), frame.usable());
    assert_eq!(back.package.frame.elements(), frame.elements());
}

#[test]
fn dxf_reads_units_polyline_block_and_text_codes() {
    let source = b"0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nCARIMBO\n10\n0\n20\n0\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n0\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nINSERT\n2\nCARIMBO\n10\n1\n20\n1\n0\nLWPOLYLINE\n90\n2\n10\n0\n20\n0\n10\n1\n20\n1\n0\nCIRCLE\n10\n2\n20\n2\n40\n1\n0\nARC\n10\n3\n20\n3\n40\n1\n50\n0\n51\n90\n0\nMTEXT\n10\n1\n20\n2\n40\n0.1\n1\nLinha\\PDois\n0\nTEXT\n10\n2\n20\n3\n40\n0.1\n1\n<<pagina.numero>>\n0\nENDSEC\n0\nEOF\n";
    let mut options = dxf::DxfImportOptions::new("polegada");
    options.sheet = Some(Size2D::new(420.0, 297.0).unwrap());
    let imported = dxf::import(source, &options).unwrap();
    let elements = imported.package.frame.elements();
    assert!(elements.iter().any(|e| matches!(e, FrameElement::Line { start, end, .. } if (start.x() - 25.4).abs() < 0.01 && (end.x() - 50.8).abs() < 0.01)));
    assert!(
        elements
            .iter()
            .any(|e| matches!(e, FrameElement::Arc { .. }))
    );
    assert!(
        elements
            .iter()
            .any(|e| matches!(e, FrameElement::Circle { .. }))
    );
    assert!(elements.iter().any(|e| matches!(
        e,
        FrameElement::Special {
            field: SpecialField::PageNumber,
            ..
        }
    )));
    assert!(
        elements
            .iter()
            .any(|e| matches!(e, FrameElement::Text { text, .. } if text.default == "Linha\nDois"))
    );
}

#[test]
fn unsafe_and_truncated_inputs_are_rejected() {
    let xml = b"<!DOCTYPE root [<!ENTITY steal SYSTEM 'file:///c:/secret'>]><EplanPxfRoot Type='Frame'>&steal;</EplanPxfRoot>";
    assert!(fn1::import(xml, &mut |_| None).is_err());
    assert!(
        titleblock::import(
            xml,
            titleblock::TitleblockOptions::new(Size2D::new(420.0, 297.0).unwrap())
        )
        .is_err()
    );
    assert!(
        dxf::import(
            b"0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n1\n",
            &dxf::DxfImportOptions::new("truncado")
        )
        .is_err()
    );
}

#[test]
fn qet_titleblock_places_fields_at_bottom_right() {
    let source = br#"<titleblocktemplate name="QET"><grid cols="100px;100px" rows="40px"><field row="0" col="0" displaylabel="true"><label><translation lang="pt_BR">Folha</translation></label><value><translation lang="pt_BR">%{folio}</translation></value></field><field row="0" col="1"><value><translation lang="pt_BR">%{title}</translation></value></field></grid></titleblocktemplate>"#;
    let imported = titleblock::import(
        source,
        titleblock::TitleblockOptions::new(Size2D::new(420.0, 297.0).unwrap()),
    )
    .unwrap();
    let frame = imported.package.frame;
    assert!(frame.elements().iter().any(|e| matches!(
        e,
        FrameElement::Special {
            field: SpecialField::PageNumber,
            ..
        }
    )));
    assert!(frame.elements().iter().any(|e| matches!(
        e,
        FrameElement::Special {
            field: SpecialField::PageName,
            ..
        }
    )));
    assert!(
        frame
            .elements()
            .iter()
            .any(|e| matches!(e, FrameElement::Text { .. }))
    );
    assert!(frame.usable().origin().y() > 5.0);
}

#[test]
fn available_qet_reference_titleblocks_import_without_copying_sources() {
    // External title-block templates are under CADCRAFT_SAMPLES/titleblock.
    let Some(dir) =
        std::env::var_os("CADCRAFT_SAMPLES").map(|r| std::path::Path::new(&r).join("titleblock"))
    else {
        return;
    };
    if !dir.is_dir() {
        return;
    }
    let mut count = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_none_or(|e| !e.eq_ignore_ascii_case("titleblock"))
        {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let imported = titleblock::import(
            &bytes,
            titleblock::TitleblockOptions::new(Size2D::new(420.0, 297.0).unwrap()),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        imported.package.frame.validate().unwrap();
        count += 1;
    }
    assert_eq!(count, 13);
}
