// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{camera::Camera, editing::GraphicEdit, hit_test::hit_candidates, state::Session};
use eframe::egui::{Pos2, Rect, vec2};
use std::time::Instant;

#[test]
fn ten_thousand_instances_share_definitions_and_preserve_history_and_other_pages() {
    let mut p = Project::new("Escala de símbolos").unwrap();
    let doc = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let mut pages = Vec::new();
    for index in 0..5 {
        pages.push(
            p.insert_page(
                Page::new(
                    p.id(),
                    doc,
                    format!("Página {index}"),
                    format!("{index}"),
                    PageKind::MultiLine,
                    Size2D::new(297.0, 210.0).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
    }
    let definitions: Vec<_> = SymbolLibrary::builtin().definitions().collect();
    let start = Instant::now();
    for index in 0..10_000 {
        let d = definitions[index % definitions.len()];
        let t = Transform2D::new(
            Point2D::new((index % 100) as f64 * 24.0, (index / 100) as f64 * 24.0).unwrap(),
            (index % 4) as f64 * 90.0,
            index % 2 == 0,
            false,
        )
        .unwrap();
        let r = Representation::new_symbol(p.id(), pages[index % 5], d, None, t).unwrap();
        p.insert_symbol_representation(r, d.clone()).unwrap();
    }
    let insert = start.elapsed();
    assert_eq!(p.symbol_definitions().count(), 10);
    p.validate().unwrap();
    let viewport = Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0));
    let camera = Camera::new([0.0, 0.0], 2.0).unwrap();
    let start = Instant::now();
    for _ in 0..40 {
        let _ = hit_candidates(&p, pages[0], camera, viewport, viewport.center());
    }
    let hit = start.elapsed();
    let ids: Vec<_> = p
        .representations_on_page(pages[0])
        .unwrap()
        .iter()
        .map(|r| r.id())
        .collect();
    let other: Vec<_> = p
        .representations_on_page(pages[1])
        .unwrap()
        .into_iter()
        .cloned()
        .collect();
    let mut s = Session::default();
    s.replace(p, false).unwrap();
    s.open_page(pages[0]).unwrap();
    s.select_graphics(ids);
    let start = Instant::now();
    s.dispatch_graphic(GraphicEdit::Translate(4.0, -4.0))
        .unwrap();
    let commit = start.elapsed();
    assert_eq!(s.history.undo_len(), 1);
    assert_eq!(
        s.project()
            .unwrap()
            .representations_on_page(pages[1])
            .unwrap()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        other
    );
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    s.project().unwrap().validate().unwrap();
    eprintln!(
        "symbol scale: insert 10k={insert:?}; hit 40 x 2k={hit:?}; transform 2k={commit:?}; definitions=10"
    );
}
