// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{editing::GraphicEdit, state::Session};

fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn fixture() -> (Session, PortRef, PortRef) {
    let mut p = Project::new("Circuito").unwrap();
    let doc = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let page = p
        .insert_page(
            Page::new(
                p.id(),
                doc,
                "P",
                "1",
                PageKind::MultiLine,
                Size2D::new(420., 297.).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let d = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.contact-no").unwrap())
        .unwrap();
    let mut ports = Vec::new();
    for x in [40., 120.] {
        let dev = p
            .insert_device(
                Device::new(
                    p.id(),
                    Designation::new(format!("K{x}")).unwrap(),
                    "Contato",
                )
                .unwrap(),
            )
            .unwrap();
        let f = p
            .insert_function(Function::new(
                p.id(),
                dev,
                Designation::new(format!("K{x}")).unwrap(),
                FunctionKind::Coil,
            ))
            .unwrap();
        let point = p
            .insert_connection_point(ConnectionPoint::new(
                p.id(),
                f,
                DesignationPart::new("1").unwrap(),
            ))
            .unwrap();
        let r = Representation::new_symbol(
            p.id(),
            page,
            d,
            Some(RepresentationTarget::Function(f)),
            Transform2D::new(pt(x, 80.), 0., false, false).unwrap(),
        )
        .unwrap();
        let id = p.insert_symbol_representation(r, d.clone()).unwrap();
        let port = PortRef::new(id, d.ports()[0].id()).unwrap();
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: port.clone(),
            point: Some(point),
        })
        .unwrap();
        ports.push(port);
    }
    let mut s = Session::default();
    s.replace(p, false).unwrap();
    s.open_page(page).unwrap();
    (s, ports.remove(0), ports.remove(0))
}
fn create(s: &mut Session, a: &PortRef, b: &PortRef) -> RepresentationId {
    let p = s.project().unwrap();
    let points = vec![p.port_position(a).unwrap(), p.port_position(b).unwrap()];
    s.dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::CreateRoute {
        source: a.clone(),
        target: b.clone(),
        points,
    }))
    .unwrap();
    s.project()
        .unwrap()
        .connection_routes()
        .next()
        .unwrap()
        .id()
}
#[test]
fn connection_creation_deletion_and_replay_restore_exact_snapshot_and_ids() {
    let (mut s, a, b) = fixture();
    let before = s.snapshot().unwrap();
    let id = create(&mut s, &a, &b);
    let after = s.snapshot().unwrap();
    assert_eq!(s.history.undo_len(), 1);
    assert_eq!(s.project().unwrap().conductors().count(), 0);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(s.snapshot().unwrap(), after);
    s.dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::DeleteRoute {
        id,
    }))
    .unwrap();
    assert_eq!(s.project().unwrap().connections().count(), 0);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), after);
}
#[test]
fn connected_symbol_group_move_and_undo_preserve_route_and_topology() {
    let (mut s, a, b) = fixture();
    let id = create(&mut s, &a, &b);
    let before = s.snapshot().unwrap();
    let points = s
        .project()
        .unwrap()
        .connection_route(id)
        .unwrap()
        .points()
        .to_vec();
    s.select_graphics([a.representation_id(), b.representation_id()]);
    s.dispatch_graphic(GraphicEdit::Translate(-12., 8.))
        .unwrap();
    let p = s.project().unwrap();
    let route = p.connection_route(id).unwrap();
    assert_eq!(
        route.points(),
        points
            .iter()
            .map(|p| p.translated(-12., 8.).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(p.connections().count(), 1);
    p.validate().unwrap();
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
}
#[test]
fn invalid_or_duplicate_drawing_operation_keeps_history_revision_and_redo() {
    let (mut s, a, b) = fixture();
    create(&mut s, &a, &b);
    let before = s.snapshot().unwrap();
    let revision = s.revision;
    let p = s.project().unwrap();
    let points = vec![p.port_position(&a).unwrap(), p.port_position(&b).unwrap()];
    assert!(
        s.dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::CreateRoute {
            source: a,
            target: b,
            points
        }))
        .is_err()
    );
    assert_eq!(s.snapshot().unwrap(), before);
    assert_eq!(s.revision, revision);
    assert_eq!(s.history.undo_len(), 1);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert!(s.history.can_redo());
}
#[test]
fn route_change_is_one_reversible_transaction_and_noop_preserves_redo() {
    let (mut s, a, b) = fixture();
    let id = create(&mut s, &a, &b);
    let before = s.snapshot().unwrap();
    let p = s.project().unwrap();
    let a = p.port_position(&a).unwrap();
    let b = p.port_position(&b).unwrap();
    s.dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::SetRoute {
        id,
        points: vec![a, pt(a.x(), 40.), pt(b.x(), 40.), b],
    }))
    .unwrap();
    assert_eq!(s.history.undo_len(), 2);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    let points = s
        .project()
        .unwrap()
        .connection_route(id)
        .unwrap()
        .points()
        .to_vec();
    assert!(
        !s.dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::SetRoute {
            id,
            points
        }))
        .unwrap()
    );
    assert!(s.history.can_redo());
}

#[test]
fn junction_history_restores_networks_and_rejects_invalid_geometry_atomically() {
    let mut s = Session::default();
    s.replace(cad_gui::connection_demo::project().unwrap(), false)
        .unwrap();
    let page = s.project().unwrap().pages().next().unwrap().id();
    s.open_page(page).unwrap();
    let routes = s
        .project()
        .unwrap()
        .connection_routes()
        .map(ConnectionRoute::id)
        .collect::<Vec<_>>();
    let before = s.snapshot().unwrap();
    s.dispatch_graphic(GraphicEdit::Drawing(
        ConnectionDrawingEdit::CreateJunction {
            page,
            position: pt(160., 140.),
            routes: routes.clone(),
        },
    ))
    .unwrap();
    let after = s.snapshot().unwrap();
    assert_eq!(s.history.undo_len(), 1);
    let p = s.project().unwrap();
    let a = p.connection_route(routes[0]).unwrap();
    let b = p.connection_route(routes[1]).unwrap();
    assert!(
        p.are_connected(
            p.port_binding(a.source()).unwrap(),
            p.port_binding(b.source()).unwrap()
        )
        .unwrap()
    );
    let r = p
        .connection_routes()
        .find(|r| r.points().len() == 2)
        .unwrap();
    let a = r.points()[0];
    let b = r.points()[1];
    let id = r.id();
    assert!(
        s.dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::SetRoute {
            id,
            points: vec![a, pt(300., a.y()), pt(300., b.y()), b]
        }))
        .is_err()
    );
    assert_eq!(s.snapshot().unwrap(), after);
    assert_eq!(s.history.undo_len(), 1);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(s.snapshot().unwrap(), after);
    let id = s
        .project()
        .unwrap()
        .connection_junctions()
        .next()
        .unwrap()
        .id();
    s.dispatch_graphic(GraphicEdit::Drawing(
        ConnectionDrawingEdit::DeleteJunction { id },
    ))
    .unwrap();
    assert_eq!(s.snapshot().unwrap(), before);
    s.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(s.snapshot().unwrap(), after);
}

#[test]
fn corrupt_drawing_import_preserves_valid_session_history_and_selection() {
    let (mut s, a, b) = fixture();
    let id = create(&mut s, &a, &b);
    s.select_graphics([id]);
    let before = s.snapshot().unwrap();
    let revision = s.revision;
    let mut data: serde_json::Value = serde_json::from_str(&before).unwrap();
    data["connection_routes"][0]["source"]["port"] = serde_json::json!("absent");
    assert!(s.import_snapshot(&data.to_string()).is_err());
    assert_eq!(s.snapshot().unwrap(), before);
    assert_eq!(s.revision, revision);
    assert_eq!(s.history.undo_len(), 1);
    assert_eq!(s.graphics.selection.primary(), Some(id));
}

#[test]
fn network_highlight_is_derived_and_invalidated_after_topology_edit() {
    let (mut s, a, b) = fixture();
    let id = create(&mut s, &a, &b);
    s.select_graphics([id]);
    s.highlight_network();
    assert_eq!(s.graphics.highlighted_routes.len(), 1);
    assert!(s.graphics.highlighted_routes.contains(&id));
    s.dispatch_graphic(GraphicEdit::Drawing(ConnectionDrawingEdit::DeleteRoute {
        id,
    }))
    .unwrap();
    assert!(s.graphics.highlighted_routes.is_empty());
}
