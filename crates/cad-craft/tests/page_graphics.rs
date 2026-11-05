// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod document_common;
use cad_craft::{domain::*, persistence};
use document_common::{fixture, transform};

#[test]
fn five_graphics_are_independent_and_survive_format_four() {
    let (mut project, page, _, _, _) = fixture();
    let networks_before = project.networks();
    let zero = Point2D::new(0.0, 0.0).unwrap();
    let far = Point2D::new(8.0, 6.0).unwrap();
    let graphics = [
        PageGraphic::new(
            SymbolPrimitive::Text {
                origin: zero,
                text: "Ação Ω".into(),
                height: 3.0,
            },
            0.25,
            TextAlignment::Center,
        )
        .unwrap(),
        PageGraphic::from_drag(GraphicKind::Line, zero, far, 0.25).unwrap(),
        PageGraphic::from_drag(GraphicKind::Rectangle, zero, far, 0.25).unwrap(),
        PageGraphic::from_drag(GraphicKind::Circle, zero, far, 0.25).unwrap(),
        PageGraphic::from_drag(GraphicKind::Arc, zero, far, 0.25).unwrap(),
    ];
    let ids: Vec<_> = graphics
        .into_iter()
        .map(|g| {
            project
                .insert_representation(
                    Representation::new_graphic(project.id(), page, g, transform(12.0, 20.0))
                        .unwrap(),
                )
                .unwrap()
        })
        .collect();
    assert_eq!(project.representations_on_page(page).unwrap().len(), 5);
    assert_eq!(project.networks(), networks_before);
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    let loaded = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(loaded.source_format, persistence::FORMAT);
    for id in ids {
        assert!(
            loaded
                .project
                .representation(id)
                .unwrap()
                .graphic()
                .is_some()
        );
    }
    assert_eq!(loaded.project, project);
}

#[test]
fn batch_is_atomic_and_reversible_across_pages() {
    let (mut project, first, second, _, _) = fixture();
    let graphic = PageGraphic::from_drag(
        GraphicKind::Rectangle,
        Point2D::new(0.0, 0.0).unwrap(),
        Point2D::new(4.0, 5.0).unwrap(),
        0.3,
    )
    .unwrap();
    let original =
        Representation::new_graphic(project.id(), first, graphic, transform(10.0, 20.0)).unwrap();
    project.insert_representation(original.clone()).unwrap();
    let moved = original.relocated(second, transform(50.0, 60.0)).unwrap();
    let wrong = original.duplicate_on(second, transform(4.0, 4.0)).unwrap();
    let before = project.clone();
    assert!(
        project
            .edit_representations(vec![
                RepresentationChange {
                    before: Some(original.clone()),
                    after: Some(moved.clone())
                },
                RepresentationChange {
                    before: Some(wrong),
                    after: None
                }
            ])
            .is_err()
    );
    assert_eq!(project, before);
    let delta = project
        .edit_representations(vec![RepresentationChange {
            before: Some(original.clone()),
            after: Some(moved.clone()),
        }])
        .unwrap();
    assert_eq!(project.representation(original.id()), Some(&moved));
    project.restore_representations(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_representations(&delta, true).unwrap();
    assert_eq!(project.representation(original.id()), Some(&moved));
}

#[test]
fn invalid_geometry_and_text_are_rejected() {
    let zero = Point2D::new(0.0, 0.0).unwrap();
    assert!(PageGraphic::from_drag(GraphicKind::Line, zero, zero, 0.25).is_err());
    assert!(
        PageGraphic::new(
            SymbolPrimitive::Text {
                origin: zero,
                text: "  ".into(),
                height: 3.0
            },
            0.25,
            TextAlignment::Left
        )
        .is_err()
    );
    assert!(
        PageGraphic::new(
            SymbolPrimitive::Circle {
                center: zero,
                radius: f64::NAN
            },
            0.25,
            TextAlignment::Left
        )
        .is_err()
    );
    assert!(
        PageGraphic::from_drag(
            GraphicKind::Rectangle,
            zero,
            Point2D::new(5.0, 5.0).unwrap(),
            0.0
        )
        .is_err()
    );
}

#[test]
fn clipboard_cut_paste_round_trip_and_refusals_leave_project_unchanged() {
    let (mut project, first, second, _, _) = fixture();
    let zero = Point2D::new(0.0, 0.0).unwrap();
    let graphic = PageGraphic::from_drag(
        GraphicKind::Line,
        zero,
        Point2D::new(10.0, 0.0).unwrap(),
        0.25,
    )
    .unwrap();
    let id = project
        .insert_representation(
            Representation::new_graphic(project.id(), first, graphic, transform(5.0, 5.0)).unwrap(),
        )
        .unwrap();
    let original = project.clone();
    // Seleção vazia e grande demais são recusadas antes de qualquer alteração.
    assert!(project.clipboard_items(&[]).is_err());
    let many = vec![id; MAX_CLIPBOARD_ITEMS + 1];
    assert!(project.cut_representations(&many).is_err());
    assert_eq!(project, original);
    let (items, cut) = project.cut_representations(&[id]).unwrap();
    assert!(items[0].is_cut() && project.representation(id).is_none());
    // Página inexistente: colagem recusada, nada muda.
    let after_cut = project.clone();
    let missing = PageId::new();
    assert!(
        project
            .paste_items(&items, missing, 0.0, 0.0, false)
            .is_err()
    );
    assert_eq!(project, after_cut);
    let (paste, pasted) = project
        .paste_items(&items, second, 20.0, 0.0, false)
        .unwrap();
    assert_eq!(pasted[0].id(), id);
    assert_eq!(project.representation(id).unwrap().page_id(), second);
    project.restore_sheet_edit(&paste, false).unwrap();
    assert_eq!(project, after_cut);
    project.restore_sheet_edit(&cut, false).unwrap();
    assert_eq!(project, original);
    project.restore_sheet_edit(&cut, true).unwrap();
    project.restore_sheet_edit(&paste, true).unwrap();
    assert_eq!(project.representation(id).unwrap().page_id(), second);
    project.validate().unwrap();
}
