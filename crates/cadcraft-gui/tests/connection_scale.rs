// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Debug workload measurement, without a timing acceptance threshold.
//!
//! The fixture deliberately overlaps 500 copies of the two-route demo on one
//! page. This is a dense adversarial workload, not a realistic industrial page.
//! The batch JSON import avoids 1,000 individual index-rebuilding route edits.
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp, camera::Camera, commands::CommandId, connection_demo,
    connection_editor::ConnectionScene, connection_geometry::move_segment, editing::GraphicEdit,
};
use eframe::egui::{self, Context, Pos2, Rect, vec2};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

fn remap(value: &mut Value, ids: &BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(replacement) = ids.get(text) {
                *text = replacement.clone();
            }
        }
        Value::Array(values) => {
            for value in values {
                remap(value, ids);
            }
        }
        Value::Object(fields) => {
            for value in fields.values_mut() {
                remap(value, ids);
            }
        }
        _ => {}
    }
}

fn dense_snapshot(copies: usize) -> String {
    let template = serde_json::to_value(connection_demo::project().unwrap()).unwrap();
    let mut combined = template.clone();
    let collections = [
        "devices",
        "functions",
        "connection_points",
        "connections",
        "representations",
        "connection_routes",
        "port_bindings",
    ];
    for collection in collections {
        combined[collection] = Value::Array(Vec::new());
    }
    for _ in 0..copies {
        let mut ids = BTreeMap::new();
        for collection in [
            "devices",
            "functions",
            "connection_points",
            "connections",
            "representations",
        ] {
            for value in template[collection].as_array().unwrap() {
                let old = value["id"].as_str().unwrap().to_owned();
                let new = match collection {
                    "devices" => DeviceId::new().to_string(),
                    "functions" => FunctionId::new().to_string(),
                    "connection_points" => ConnectionPointId::new().to_string(),
                    "connections" => ConnectionId::new().to_string(),
                    "representations" => RepresentationId::new().to_string(),
                    _ => unreachable!(),
                };
                assert!(ids.insert(old, new).is_none());
            }
        }
        for collection in collections {
            let mut values = template[collection].as_array().unwrap().clone();
            for value in &mut values {
                remap(value, &ids);
            }
            combined[collection].as_array_mut().unwrap().extend(values);
        }
    }
    combined["name"] = Value::String("Escala de conexões — sobreposição adversarial".into());
    serde_json::to_string(&combined).unwrap()
}

#[test]
fn thousand_routes_measure_cache_snap_preview_and_cpu_render_without_mutating_topology() {
    let start = Instant::now();
    let json = dense_snapshot(500);
    let prepare = start.elapsed();
    let mut app = CadApp::default();
    let start = Instant::now();
    app.session.import_snapshot(&json).unwrap();
    let import = start.elapsed();
    let project = app.session.project().unwrap();
    assert_eq!(project.documents().count(), 1);
    assert_eq!(project.pages().count(), 1);
    assert_eq!(project.devices().count(), 2_000);
    assert_eq!(project.functions().count(), 2_000);
    assert_eq!(project.connection_points().count(), 4_000);
    assert_eq!(project.connections().count(), 1_000);
    assert_eq!(project.connection_routes().count(), 1_000);
    assert_eq!(project.representations().count(), 3_000);
    assert_eq!(project.port_bindings().count(), 4_000);
    assert_eq!(project.connection_junctions().count(), 0);
    assert_eq!(project.conductors().count(), 0);
    assert_eq!(project.symbol_definitions().count(), 1);
    assert_eq!(
        project
            .representations()
            .map(Representation::id)
            .collect::<BTreeSet<_>>()
            .len(),
        3_000
    );
    let route_ids: BTreeSet<_> = project
        .connection_routes()
        .map(ConnectionRoute::id)
        .collect();
    assert_eq!(route_ids.len(), 1_000);
    for route in project.connection_routes() {
        assert!(project.representation(route.id()).is_some());
        assert!(project.connection(route.connection_id()).is_some());
        assert_eq!(
            project.port_position(route.source()).unwrap(),
            route.points()[0]
        );
        assert_eq!(
            project.port_position(route.target()).unwrap(),
            *route.points().last().unwrap()
        );
    }
    let page = project.pages().next().unwrap().id();
    let mut scene = ConnectionScene::default();
    let revision = app.session.revision;
    let start = Instant::now();
    scene.sync(project, page, revision);
    let build_scene = start.elapsed();
    assert_eq!(scene.ports.len(), 4_000);
    assert_eq!(scene.routes.len(), 1_000);
    let start = Instant::now();
    for _ in 0..100 {
        scene.sync(project, page, revision);
    }
    let cached_sync = start.elapsed();
    let start = Instant::now();
    for index in 0..100 {
        let hit = scene.snap(p(120. + f64::from(index % 4), 140.5), 2., 4., true);
        assert_eq!(hit.position.y(), 140.);
    }
    let snap_queries = start.elapsed();
    let before = app.session.snapshot().unwrap();
    app.session.open_page(page).unwrap();
    let ctx = Context::default();
    app.execute(CommandId::DrawConnection, &ctx);
    app.session
        .connection_click(scene.snap(p(80., 100.), 2., 4., true))
        .unwrap();
    let start = Instant::now();
    for index in 0..20 {
        app.session
            .graphics
            .connection
            .as_mut()
            .unwrap()
            .update(p(120. + f64::from(index), 140.));
    }
    let previews = start.elapsed();
    assert_eq!(app.session.snapshot().unwrap(), before);
    assert_eq!(app.session.history.undo_len(), 0);
    let tab = app.session.active_tab_mut().unwrap();
    tab.camera = Camera::new([160., 140.], 2.).unwrap();
    tab.needs_fit = false;
    let start = Instant::now();
    let mut vertices = 0usize;
    let mut shapes = 0usize;
    for _ in 0..5 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
                ..Default::default()
            },
            |ctx| app.show(ctx),
        );
        assert!(!output.shapes.is_empty());
        shapes += output.shapes.len();
        for primitive in ctx.tessellate(output.shapes, output.pixels_per_point) {
            if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                vertices += mesh.vertices.len();
            }
        }
    }
    let frames = start.elapsed();
    assert!(vertices > 0);
    assert_eq!(app.session.snapshot().unwrap(), before);
    let route = app
        .session
        .project()
        .unwrap()
        .connection_routes()
        .find(|r| r.points().len() == 4)
        .unwrap();
    let id = route.id();
    let original = route.points().to_vec();
    let edited = move_segment(&original, 1, p(120., 144.)).unwrap();
    let start = Instant::now();
    app.session
        .dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::SetRoute {
            id,
            points: edited.clone(),
        }))
        .unwrap();
    let edit = start.elapsed();
    assert_ne!(app.session.revision, revision);
    scene.sync(app.session.project().unwrap(), page, revision);
    assert_eq!(
        scene
            .routes
            .iter()
            .find(|(route, _)| *route == id)
            .unwrap()
            .1,
        original,
        "unchanged revision reuses cached geometry"
    );
    let start = Instant::now();
    scene.sync(app.session.project().unwrap(), page, app.session.revision);
    let rebuild = start.elapsed();
    assert_eq!(
        scene
            .routes
            .iter()
            .find(|(route, _)| *route == id)
            .unwrap()
            .1,
        edited
    );
    let project = app.session.project().unwrap();
    assert_eq!(
        project
            .connection_routes()
            .map(ConnectionRoute::id)
            .collect::<BTreeSet<_>>(),
        route_ids
    );
    assert_eq!(project.connection_junctions().count(), 0);
    assert_eq!(project.connections().count(), 1_000);
    assert_eq!(project.conductors().count(), 0);
    project.validate().unwrap();
    eprintln!(
        "connection scale DEBUG, dense adversarial overlap: routes=1000 symbols=2000 bound_ports=4000; JSON={} bytes; batch_prepare={prepare:?}; snapshot_import={import:?}; scene_initial={build_scene:?}; cached_sync_100={cached_sync:?}; snap_100={snap_queries:?}; preview_20={previews:?}; CadApp_CPU_frames_5_with_tessellation={frames:?}; shapes={shapes}; mesh_vertices={vertices}; committed_edit={edit:?}; scene_revision_rebuild={rebuild:?}. No GPU/presentation measurement or industrial timing guarantee.",
        json.len()
    );
}
