// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;
use cad_craft::domain::*;
use serde_json::json;
use std::time::Instant;

fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn tr(x: f64, y: f64) -> Transform2D {
    Transform2D::new(pt(x, y), 0., false, false).unwrap()
}

/// Build the DTO in a batch: timings below measure import/edit behavior rather
/// than N separate confirmed edits, each of which deliberately validates state.
fn drawing_snapshot(
    count: usize,
) -> (
    serde_json::Value,
    Vec<RepresentationId>,
    Vec<(RepresentationId, RepresentationId)>,
) {
    let (mut p, device, _, points) = physical_common::fixture((count + 1) * 2);
    let d = p
        .insert_document(Document::new(p.id(), "Scale").unwrap())
        .unwrap();
    let page = p
        .insert_page(
            Page::new(
                p.id(),
                d,
                "Scale",
                "1",
                PageKind::MultiLine,
                Size2D::new(200., 200.).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let definition = SymbolDefinition::new(
        SymbolDefinitionId::new("scale.port").unwrap(),
        1,
        "Port",
        "Scale",
        vec![SymbolPrimitive::Line {
            start: pt(-1., 0.),
            end: pt(1., 0.),
        }],
        vec![SymbolPort::new("p", pt(0., 0.), 0., "P", None).unwrap()],
    )
    .unwrap();
    let mut routes = vec![];
    let mut connections = vec![];
    let mut bindings = vec![];
    let mut representations = vec![];
    let mut ids = vec![];
    let mut endpoints = vec![];
    for i in 0..=count {
        let (a, b) = if i < count {
            (
                pt(0., i as f64 * 2. - 1000.),
                pt(100., i as f64 * 2. - 1000.),
            )
        } else {
            (pt(50., -1001.), pt(50., 1001.))
        };
        let mut ports = vec![];
        for (offset, position) in [a, b].into_iter().enumerate() {
            let r = Representation::new_symbol(
                p.id(),
                page,
                &definition,
                Some(RepresentationTarget::Device(device)),
                tr(position.x(), position.y()),
            )
            .unwrap();
            let port = PortRef::new(r.id(), "p").unwrap();
            bindings.push(json!({"port":port,"point":points[i*2+offset]}));
            ports.push(port);
            p.insert_symbol_representation(r, definition.clone())
                .unwrap();
        }
        endpoints.push((ports[0].representation_id(), ports[1].representation_id()));
        let edge = Connection::new(p.id(), points[i * 2], points[i * 2 + 1]).unwrap();
        let route = ConnectionRoute::new(
            p.id(),
            page,
            edge.id(),
            ports[0].clone(),
            ports[1].clone(),
            vec![a, b],
        )
        .unwrap();
        let bounds = Rect2D::new(
            pt(a.x().min(b.x()), a.y().min(b.y())),
            Size2D::new((a.x() - b.x()).abs(), (a.y() - b.y()).abs()).unwrap(),
        )
        .unwrap();
        let representation = Representation::new(
            p.id(),
            page,
            RepresentationTarget::Connection(edge.id()),
            tr(0., 0.),
        )
        .with_local_bounds(bounds)
        .unwrap();
        let mut representation = json!(representation);
        representation["id"] = json!(route.id());
        representations.push(representation);
        ids.push(route.id());
        connections.push(edge);
        routes.push(route);
    }
    let mut snapshot = json!(p);
    snapshot["connections"] = json!(connections);
    snapshot["connection_routes"] = json!(routes);
    snapshot["port_bindings"] = json!(bindings);
    snapshot["representations"]
        .as_array_mut()
        .unwrap()
        .extend(representations);
    (snapshot, ids, endpoints)
}
#[test]
fn thousand_routes_import_edit_transform_junction_and_validate_keep_integrity() {
    let (snapshot, ids, endpoints) = drawing_snapshot(1000);
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let begin = Instant::now();
    let mut p: Project = serde_json::from_slice(&bytes).unwrap();
    let import = begin.elapsed();
    assert_eq!(p.connection_routes().count(), 1001);
    assert_eq!(p.representations().count(), 3003);
    assert_eq!(p.conductors().count(), 0);
    let before = serde_json::to_vec(&p).unwrap();
    let begin = Instant::now();
    let transform = p
        .transform_representations_with_drawing(&[
            (endpoints[0].0, tr(-3., -993.)),
            (endpoints[0].1, tr(97., -993.)),
        ])
        .unwrap();
    let transform_time = begin.elapsed();
    assert!(transform.len() <= 4);
    assert_eq!(
        p.connection_route(ids[0]).unwrap().points(),
        &[pt(-3., -993.), pt(97., -993.)]
    );
    let begin = Instant::now();
    let edit = p
        .edit_connection_drawing(ConnectionDrawingEdit::SetRoute {
            id: ids[1],
            points: vec![
                pt(0., -998.),
                pt(0., -995.),
                pt(100., -995.),
                pt(100., -998.),
            ],
        })
        .unwrap();
    let edit_time = begin.elapsed();
    assert!(edit.len() <= 2);
    let page = p.connection_route(ids[500]).unwrap().page_id();
    let begin = Instant::now();
    let junction = p
        .edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
            page,
            position: pt(50., 0.),
            routes: vec![ids[500], ids[1000]],
        })
        .unwrap();
    let junction_time = begin.elapsed();
    assert_eq!(junction.len(), 2);
    let begin = Instant::now();
    p.validate().unwrap();
    let validate = begin.elapsed();
    p.restore_connection_drawing(&junction, false).unwrap();
    p.restore_connection_drawing(&edit, false).unwrap();
    p.restore_connection_drawing(&transform, false).unwrap();
    assert_eq!(serde_json::to_vec(&p).unwrap(), before);
    eprintln!(
        "scale: routes=1001 symbols=2002 bytes={} import={import:?} transform={transform_time:?} edit={edit_time:?} junction={junction_time:?} validate={validate:?}",
        bytes.len()
    );
}
