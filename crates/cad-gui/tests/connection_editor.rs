// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{
    camera::Camera,
    connection_editor::{ConnectionDraft, ConnectionScene, SnapKind},
};
use eframe::egui::{Pos2, Rect, vec2};
fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

#[test]
fn grid_snap_is_four_mm_with_hidden_grid_at_every_zoom_and_pan() {
    let scene = ConnectionScene::default();
    for scale in [0.25, 1., 2.5, 20.] {
        let camera = Camera::new([50., -30.], scale).unwrap();
        let view = Rect::from_min_size(Pos2::new(100., 50.), vec2(800., 500.));
        let cursor = camera.world_to_screen(p(-5.1, 6.3), view).unwrap();
        let result = scene.snap(
            camera.screen_to_world(cursor, view).unwrap(),
            camera.scale(),
            4.,
            true,
        );
        assert_eq!(result.position, p(-4., 8.));
        assert!(matches!(result.kind, SnapKind::Grid));
    }
}
#[test]
fn orthogonal_draft_has_no_persistent_identity_and_preserves_waypoints() {
    let page = PageId::new();
    let mut d = ConnectionDraft::new(page);
    d.anchors = vec![p(-8., -4.), p(12., -4.)];
    d.update(p(20., 12.));
    assert_eq!(d.preview.first(), Some(&p(-8., -4.)));
    assert_eq!(d.preview.last(), Some(&p(20., 12.)));
    assert!(
        d.preview
            .windows(2)
            .all(|w| w[0] != w[1] && (w[0].x() == w[1].x() || w[0].y() == w[1].y()))
    );
    let a = d.preview.clone();
    d.horizontal_first = false;
    d.update(p(20., 12.));
    assert_ne!(a, d.preview);
}
#[test]
fn escape_cancel_clears_connection_and_route_previews_only() {
    let mut s = cad_gui::state::Session::default();
    s.replace(cad_gui::demo::project().unwrap(), true).unwrap();
    let page = s.project().unwrap().pages().next().unwrap().id();
    s.open_page(page).unwrap();
    let before = s.snapshot().unwrap();
    s.begin_connection().unwrap();
    s.graphics
        .connection
        .as_mut()
        .unwrap()
        .anchors
        .push(p(-8., 12.));
    s.graphics.connection.as_mut().unwrap().update(p(45., -5.));
    s.graphics.cancel();
    assert!(!s.graphics.is_active());
    assert_eq!(s.snapshot().unwrap(), before);
    assert_eq!(s.history.undo_len(), 0);
}

#[test]
fn ordinary_selection_clears_previous_junction_selection() {
    let mut interaction = cad_gui::interaction::Interaction {
        junction_selection: Some(ConnectionJunctionId::new()),
        ..Default::default()
    };
    interaction.click(Some(RepresentationId::new()), false);
    assert!(interaction.junction_selection.is_none());
    interaction.junction_selection = Some(ConnectionJunctionId::new());
    interaction.click(None, false);
    assert!(interaction.junction_selection.is_none());
}

#[test]
fn bound_port_snap_beats_grid_and_is_visually_consistent_at_different_scales() {
    let project = cad_gui::connection_demo::project().unwrap();
    let page = project.pages().next().unwrap().id();
    let mut scene = ConnectionScene::default();
    scene.sync(&project, page, 1);
    for scale in [0.5, 2., 12.] {
        let result = scene.snap(p(80. + 3. / scale, 100.), scale, 4., true);
        assert_eq!(result.position, p(80., 100.));
        assert!(matches!(result.kind, SnapKind::Port { bound: true, .. }));
    }
}

#[test]
fn explicit_junction_snap_has_priority_over_crossing_segments() {
    let mut project = cad_gui::connection_demo::project().unwrap();
    let page = project.pages().next().unwrap().id();
    let routes = project
        .connection_routes()
        .map(ConnectionRoute::id)
        .collect();
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
            page,
            position: p(160., 140.),
            routes,
        })
        .unwrap();
    let mut scene = ConnectionScene::default();
    scene.sync(&project, page, 1);
    assert!(matches!(
        scene.snap(p(160., 140.), 2., 4., true).kind,
        SnapKind::Junction(_)
    ));
}

#[test]
fn alignment_suggestion_is_deterministic_and_has_no_project_mutation() {
    let project = cad_gui::connection_demo::project().unwrap();
    let before = serde_json::to_string(&project).unwrap();
    let page = project.pages().next().unwrap().id();
    let mut scene = ConnectionScene::default();
    scene.sync(&project, page, 1);
    let source = scene
        .ports
        .iter()
        .find(|(_, pos, _)| *pos == p(160., 60.))
        .unwrap()
        .0
        .clone();
    assert_eq!(
        scene
            .aligned_port(&source, p(160., 60.), p(162., 220.), 2.)
            .unwrap()
            .1,
        p(160., 220.)
    );
    assert!(
        scene
            .aligned_port(&source, p(160., 60.), p(300., 300.), 2.)
            .is_none()
    );
    assert_eq!(serde_json::to_string(&project).unwrap(), before);
}
