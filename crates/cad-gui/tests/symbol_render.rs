// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 17: desenho de elipse, estilos, preenchimento e texto dinâmico; hit
//! test pelo traço em dois zooms.
use cad_craft::domain::*;
use cad_gui::{app::CadApp, camera::Camera};
use eframe::egui::{self, Context, Pos2, Rect, vec2};
use std::collections::BTreeMap;

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

fn lamp(dash: LineDash) -> SymbolDefinition {
    let mut styles = BTreeMap::new();
    styles.insert(
        0,
        PrimitiveStyle {
            dash,
            ..Default::default()
        },
    );
    styles.insert(
        1,
        PrimitiveStyle {
            fill: true,
            ..Default::default()
        },
    );
    SymbolDefinition::new(
        SymbolDefinitionId::new(format!("teste.render.{dash:?}")).unwrap(),
        1,
        "Sinaleiro",
        "Sinalização",
        vec![
            SymbolPrimitive::Ellipse {
                center: p(0.0, 0.0),
                radius_x: 8.0,
                radius_y: 5.0,
                rotation_degrees: 0.0,
            },
            SymbolPrimitive::Polyline {
                points: vec![p(-2.0, -2.0), p(2.0, -2.0), p(0.0, 2.0)],
                closed: true,
            },
            SymbolPrimitive::DynamicText {
                origin: p(10.0, 0.0),
                field: SymbolTextField::Designation,
                height: 3.0,
                align: TextAlign::MiddleLeft,
                rotation_degrees: 0.0,
            },
        ],
        vec![
            SymbolPort::new("1", p(0.0, 8.0), 90.0, "X1", None).unwrap(),
            SymbolPort::new("2", p(0.0, -8.0), 270.0, "X2", None).unwrap(),
        ],
    )
    .unwrap()
    .with_styles(styles)
    .unwrap()
}

/// Projeto com o sinaleiro inserido como -H1; devolve app, contexto e ocorrência.
fn app(dash: LineDash) -> (CadApp, Context, RepresentationId) {
    let mut project = Project::new("Render").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Página",
                "1",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let delta = project
        .place_symbol(SymbolPlacement {
            page,
            definition: lamp(dash),
            transform: Transform2D::new(p(100.0, 100.0), 0.0, false, false).unwrap(),
            device: PlacementDevice::New {
                designation: Designation::new("H1").unwrap(),
                classification: "Sinaleiro".into(),
            },
            function_kind: FunctionKind::Other(Text::new("Sinalização").unwrap()),
        })
        .unwrap();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.open_page(page).unwrap();
    let tab = app.session.active_tab_mut().unwrap();
    tab.camera = Camera::new([100.0, 100.0], 6.0).unwrap();
    tab.needs_fit = false;
    (app, Context::default(), delta.representation_id())
}

fn run(app: &mut CadApp, ctx: &Context) -> egui::FullOutput {
    let mut out = None;
    for _ in 0..3 {
        out = Some(ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 900.0))),
                ..Default::default()
            },
            |ctx| app.show(ctx),
        ));
    }
    out.unwrap()
}

fn shapes(out: &egui::FullOutput) -> (Vec<String>, f32, usize) {
    let (mut texts, mut segments, mut fills) = (Vec::new(), 0.0_f32, 0);
    for s in &out.shapes {
        match &s.shape {
            egui::Shape::Text(t) => texts.push(t.galley.text().to_owned()),
            egui::Shape::LineSegment { points, .. } => segments += points[0].distance(points[1]),
            egui::Shape::Path(p) if p.closed && p.fill != egui::Color32::TRANSPARENT => fills += 1,
            _ => {}
        }
    }
    (texts, segments, fills)
}

#[test]
fn dynamic_text_dashes_and_fill_are_painted_on_the_sheet() {
    let (mut solid_app, ctx, _) = app(LineDash::Solid);
    let (texts, solid_segments, fills) = shapes(&run(&mut solid_app, &ctx));
    assert!(
        texts.iter().any(|t| t == "-H1"),
        "texto dinâmico resolvido: {texts:?}"
    );
    assert!(fills >= 1, "triângulo preenchido");
    let (mut dashed_app, ctx2, _) = app(LineDash::Dashed);
    let (_, dashed_segments, _) = shapes(&run(&mut dashed_app, &ctx2));
    assert!(
        dashed_segments < solid_segments - 20.0,
        "tracejado desenha menos comprimento: {dashed_segments} × {solid_segments}"
    );
}

#[test]
fn ellipse_is_hit_by_its_outline_at_two_zooms() {
    let d = lamp(LineDash::Solid);
    let ellipse = &d.primitives()[0];
    // Ponto no contorno (8, 0) e no centro (0, 0).
    for tolerance in [0.5, 0.1] {
        assert!(cad_gui::symbol_view::primitive_distance(ellipse, p(8.0, 0.0)) < tolerance);
        assert!(cad_gui::symbol_view::primitive_distance(ellipse, p(0.0, 5.0)) < tolerance);
    }
    assert!(cad_gui::symbol_view::primitive_distance(ellipse, p(0.0, 0.0)) > 4.0);
    let text = &d.primitives()[2];
    assert!(cad_gui::symbol_view::primitive_distance(text, p(11.0, 0.0)) < 1e-9);
}
