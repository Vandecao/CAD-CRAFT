// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 17: primitivas ampliadas, estilos, textos dinâmicos e atualização de
//! símbolos do projeto.
mod document_common;
use cad_craft::domain::*;
use document_common::fixture;
use std::collections::BTreeMap;

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

fn lamp(version: u32, port_two: &str) -> SymbolDefinition {
    let mut styles = BTreeMap::new();
    styles.insert(
        0,
        PrimitiveStyle {
            dash: LineDash::Dashed,
            width_mm: Some(0.5),
            fill: false,
        },
    );
    SymbolDefinition::new(
        SymbolDefinitionId::new("teste.sinaleiro").unwrap(),
        version,
        "Sinaleiro",
        "Sinalização",
        vec![
            SymbolPrimitive::Ellipse {
                center: p(0.0, 0.0),
                radius_x: 4.0,
                radius_y: 2.0,
                rotation_degrees: 90.0,
            },
            SymbolPrimitive::DynamicText {
                origin: p(5.0, 0.0),
                field: SymbolTextField::Designation,
                height: 2.5,
                align: TextAlign::MiddleLeft,
                rotation_degrees: 0.0,
            },
            SymbolPrimitive::DynamicText {
                origin: p(1.0, 6.0),
                field: SymbolTextField::PortLabel("1".into()),
                height: 2.0,
                align: TextAlign::BottomLeft,
                rotation_degrees: 0.0,
            },
        ],
        vec![
            SymbolPort::new("1", p(0.0, 6.0), 90.0, "X1", None).unwrap(),
            SymbolPort::new(port_two, p(0.0, -6.0), 270.0, "X2", None).unwrap(),
        ],
    )
    .unwrap()
    .with_styles(styles)
    .unwrap()
}

fn placed() -> (Project, PageId, RepresentationId) {
    let (mut project, page, _, _, _) = fixture();
    let delta = project
        .place_symbol(SymbolPlacement {
            page,
            definition: lamp(1, "2"),
            transform: Transform2D::new(p(100.0, 100.0), 0.0, false, false).unwrap(),
            device: PlacementDevice::New {
                designation: Designation::new("H1").unwrap(),
                classification: "Sinaleiro".into(),
            },
            function_kind: FunctionKind::Other(Text::new("Sinalização").unwrap()),
        })
        .unwrap();
    (project, page, delta.representation_id())
}

#[test]
fn ellipse_bounds_follow_rotation_and_styles_are_validated() {
    let e = SymbolPrimitive::Ellipse {
        center: p(10.0, 10.0),
        radius_x: 4.0,
        radius_y: 2.0,
        rotation_degrees: 90.0,
    };
    let b = e.bounds().unwrap();
    assert!((b.size().width() - 4.0).abs() < 1e-9 && (b.size().height() - 8.0).abs() < 1e-9);
    assert!(e.is_closed());
    let line = SymbolPrimitive::Line {
        start: p(0.0, 0.0),
        end: p(5.0, 0.0),
    };
    let fill = PrimitiveStyle {
        fill: true,
        ..Default::default()
    };
    assert!(fill.check(&line).is_err(), "linha não tem preenchimento");
    assert!(fill.check(&e).is_ok());
    let bad = PrimitiveStyle {
        width_mm: Some(-1.0),
        ..Default::default()
    };
    assert!(bad.check(&e).is_err());
    // Estilo apontando para primitiva inexistente é recusado.
    let mut styles = BTreeMap::new();
    styles.insert(9, PrimitiveStyle::default());
    styles.insert(
        10,
        PrimitiveStyle {
            dash: LineDash::Dotted,
            ..Default::default()
        },
    );
    assert!(lamp(1, "2").with_styles(styles).is_err());
    // Estilos padrão não são guardados.
    let mut only_default = BTreeMap::new();
    only_default.insert(0, PrimitiveStyle::default());
    assert!(
        lamp(1, "2")
            .with_styles(only_default)
            .unwrap()
            .styles()
            .is_empty()
    );
    // Conteúdo livre: elipse arrastada e preenchida.
    let g = PageGraphic::from_drag(GraphicKind::Ellipse, p(0.0, 0.0), p(20.0, 10.0), 0.25)
        .unwrap()
        .with_style(fill)
        .unwrap();
    assert!(g.style().fill);
    assert!(
        PageGraphic::from_drag(GraphicKind::Line, p(0.0, 0.0), p(5.0, 0.0), 0.25)
            .unwrap()
            .with_style(fill)
            .is_err()
    );
}

#[test]
fn dynamic_texts_resolve_designation_port_reference_and_property() {
    let (mut project, _, id) = placed();
    assert_eq!(
        project
            .symbol_text(id, &SymbolTextField::Designation)
            .as_deref(),
        Some("-H1")
    );
    assert_eq!(
        project
            .symbol_text(id, &SymbolTextField::PortLabel("1".into()))
            .as_deref(),
        Some("X1")
    );
    assert_eq!(
        project
            .symbol_text(id, &SymbolTextField::FunctionKind)
            .as_deref(),
        Some("Sinalização")
    );
    assert!(
        project
            .symbol_text(id, &SymbolTextField::Reference)
            .is_some()
    );
    assert_eq!(
        project.symbol_text(id, &SymbolTextField::DeviceProperty("Cor".into())),
        None
    );
    assert_eq!(
        SymbolTextField::Designation.placeholder(),
        "-K?",
        "marcador sem ocorrência"
    );
    let _ = &mut project;
}

#[test]
fn symbol_update_keeps_occurrences_and_refuses_missing_ports() {
    let (mut project, _, id) = placed();
    let before = project.clone();
    // Nova versão compatível (mesmas portas, geometria diferente).
    let v1 = project
        .symbol_definition(&SymbolDefinitionId::new("teste.sinaleiro").unwrap())
        .unwrap()
        .clone();
    let mut primitives = v1.primitives().to_vec();
    primitives.push(SymbolPrimitive::Rectangle {
        rect: Rect2D::new(p(-6.0, -6.0), Size2D::new(12.0, 12.0).unwrap()).unwrap(),
    });
    let v2 = v1
        .edited(
            "Sinaleiro",
            "Sinalização",
            primitives,
            v1.ports().to_vec(),
            v1.styles().clone(),
        )
        .unwrap();
    assert_eq!(v2.version(), 2);
    assert!(project.symbol_update_problems(&v2).is_empty());
    let delta = project.update_symbol_definition(v2.clone()).unwrap();
    let r = project.representation(id).unwrap();
    assert_eq!(r.local_bounds(), Some(v2.bounds()));
    assert_eq!(project.symbol_occurrences(v2.id()), vec![id]);
    project.restore_symbol_update(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_symbol_update(&delta, true).unwrap();
    assert_eq!(project.symbol_definition(v2.id()), Some(&v2));
    // Versão sem a porta "2" (em uso): recusada sem alterar nada.
    let after = project.clone();
    let broken = lamp(3, "3");
    let problems = project.symbol_update_problems(&broken);
    assert!(problems.iter().any(|p| p.contains("'2'")), "{problems:?}");
    assert!(project.update_symbol_definition(broken).is_err());
    assert_eq!(project, after);
    // Registrar uma definição nova sem ocorrências; outra versão é recusada.
    let other = lamp(1, "2")
        .with_identity(SymbolDefinitionId::new("teste.outro").unwrap(), "Outro")
        .unwrap();
    assert!(project.add_symbol_definition(other.clone()).unwrap());
    assert!(!project.add_symbol_definition(other).unwrap());
    assert!(project.add_symbol_definition(lamp(9, "2")).is_err());
}

#[test]
fn cadsym_round_trip_and_invalid_files_are_refused() {
    use cad_craft::persistence;
    let def = lamp(2, "2");
    let bytes = persistence::write_symbol_package(&def).unwrap();
    assert_eq!(persistence::read_symbol_package(&bytes).unwrap(), def);
    assert!(persistence::read_symbol_package(b"nao e zip").is_err());
    let mut truncated = bytes.clone();
    truncated.truncate(bytes.len() / 2);
    assert!(persistence::read_symbol_package(&truncated).is_err());
    // Outro tipo de pacote (moldura) não é aceito como símbolo.
    let (project, _, _) = placed();
    let frame = project.frames().next().map(|f| f.id());
    if let Some(id) = frame {
        let package = project.frame_package(id).unwrap();
        let frame_bytes = persistence::write_frame_package(&package).unwrap();
        assert!(persistence::read_symbol_package(&frame_bytes).is_err());
    }
}
