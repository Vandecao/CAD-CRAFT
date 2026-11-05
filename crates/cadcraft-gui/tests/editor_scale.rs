// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{
    camera::Camera, editing::GraphicEdit, hit_test::hit_candidates, interaction::Interaction,
    state::Session,
};
use eframe::egui::{Pos2, Rect, vec2};
use std::time::Instant;
#[test]
fn ten_thousand_representations_support_queries_preview_and_one_atomic_undo() {
    let mut p = cad_gui::demo::project().unwrap();
    let first = p.representations().next().unwrap();
    let page = first.page_id();
    let target = first.target();
    let mut ids = Vec::new();
    for n in 0..10_000 {
        let t = Transform2D::new(
            Point2D::new((n % 100) as f64 * 20.0, (n / 100) as f64 * 20.0).unwrap(),
            (n % 4) as f64 * 90.0 + 30.0,
            false,
            false,
        )
        .unwrap();
        let mut rep = Representation::new(p.id(), page, target, t);
        if n % 2 == 0 {
            rep = rep
                .with_local_bounds(
                    Rect2D::new(
                        Point2D::new(0.0, 0.0).unwrap(),
                        Size2D::new(10.0, 5.0).unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap()
                .with_visual(VisualProperties {
                    label: Some(Text::new(format!("R{n}")).unwrap()),
                    ..Default::default()
                });
        }
        ids.push(p.insert_representation(rep).unwrap());
    }
    let view = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
    let camera = Camera::new([0.0, 0.0], 2.0).unwrap();
    let started = Instant::now();
    for _ in 0..40 {
        assert!(hit_candidates(&p, page, camera, view, view.center()).contains(&ids[0]));
    }
    let queries = started.elapsed();
    let mut interaction = Interaction::default();
    interaction.selection.replace(ids.iter().copied());
    let start = Point2D::new(0.0, 0.0).unwrap();
    interaction.begin_move(&p, page, ids[0], start).unwrap();
    let started = Instant::now();
    for n in 0..20 {
        interaction
            .update_move(Point2D::new(n as f64, 13.0).unwrap(), 4.0, true)
            .unwrap();
    }
    let preview = started.elapsed();
    assert_eq!(
        p.representation(ids[0]).unwrap().transform().translation(),
        start
    );
    let batch = interaction.finish_move().unwrap();
    assert_eq!(batch.len(), 10_000);
    let mut s = Session::default();
    s.replace(p, false).unwrap();
    let started = Instant::now();
    s.dispatch_graphic(GraphicEdit::Set(batch)).unwrap();
    let commit = started.elapsed();
    assert_eq!(s.history.undo_len(), 1);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(
        s.project()
            .unwrap()
            .representation(ids[0])
            .unwrap()
            .transform()
            .translation(),
        start
    );
    s.project().unwrap().validate().unwrap();
    eprintln!(
        "10.000 reps: 40 hit queries {queries:?}; 20 group previews {preview:?}; commit {commit:?}"
    );
}
