// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{camera::Camera, hit_test::hit_candidates};
use eframe::egui::{Pos2, Rect, vec2};

#[test]
fn line_and_aligned_text_hit_at_two_viewport_scales() {
    let mut project = Project::new("Geometria livre").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "P",
                "1",
                PageKind::MultiLine,
                Size2D::new(210.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let zero = Point2D::new(0.0, 0.0).unwrap();
    let line = PageGraphic::from_drag(
        GraphicKind::Line,
        zero,
        Point2D::new(40.0, 0.0).unwrap(),
        0.5,
    )
    .unwrap();
    let id = project
        .insert_representation(
            Representation::new_graphic(
                project.id(),
                page,
                line,
                Transform2D::new(Point2D::new(20.0, 40.0).unwrap(), 0.0, false, false).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let text = PageGraphic::new(
        SymbolPrimitive::Text {
            origin: zero,
            text: "ABC".into(),
            height: 5.0,
        },
        0.25,
        TextAlignment::Center,
    )
    .unwrap();
    let text_id = project
        .insert_representation(
            Representation::new_graphic(
                project.id(),
                page,
                text,
                Transform2D::new(Point2D::new(100.0, 80.0).unwrap(), 30.0, false, false).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    for (width, height) in [(800.0, 600.0), (1600.0, 1200.0)] {
        let viewport = Rect::from_min_size(Pos2::ZERO, vec2(width, height));
        let mut camera = Camera::default();
        camera.fit(project.document_bounds(page).unwrap().sheet, viewport);
        let line_point = camera
            .world_to_screen(Point2D::new(40.0, 40.0).unwrap(), viewport)
            .unwrap();
        assert!(hit_candidates(&project, page, camera, viewport, line_point).contains(&id));
        let text_point = camera
            .world_to_screen(Point2D::new(100.0, 82.0).unwrap(), viewport)
            .unwrap();
        assert!(hit_candidates(&project, page, camera, viewport, text_point).contains(&text_id));
    }
}
