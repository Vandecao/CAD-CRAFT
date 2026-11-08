// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! T-junctions as drawn in real schematics: a route that ends on another
//! route already sharing its port (a feed taken from a contactor output) and
//! ports whose positions come from rotated or mirrored symbols.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod physical_common;
use cad_craft::domain::*;

fn pt(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

/// Page with one-port symbols placed by the given transforms, each bound to
/// its own connection point.
fn setup(
    transforms: &[(f64, f64, f64, bool)],
) -> (Project, PageId, Vec<PortRef>, Vec<ConnectionPointId>) {
    let (mut p, device, _, points) = physical_common::fixture(transforms.len());
    let d = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let page = p
        .insert_page(
            Page::new(
                p.id(),
                d,
                "P",
                "1",
                PageKind::MultiLine,
                Size2D::new(400., 300.).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    // The port sits 10 mm from the symbol origin, so rotation moves it.
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("test.lead").unwrap(),
        1,
        "Lead",
        "Test",
        vec![SymbolPrimitive::Line {
            start: pt(0., 0.),
            end: pt(0., 10.),
        }],
        vec![SymbolPort::new("p", pt(0., 10.), 90., "P", None).unwrap()],
    )
    .unwrap();
    let mut ports = vec![];
    for (i, (x, y, angle, mirror)) in transforms.iter().enumerate() {
        let t = Transform2D::new(pt(*x, *y), *angle, *mirror, false).unwrap();
        let r = Representation::new_symbol(
            p.id(),
            page,
            &def,
            Some(RepresentationTarget::Device(device)),
            t,
        )
        .unwrap();
        let port = PortRef::new(r.id(), "p").unwrap();
        p.insert_symbol_representation(r, def.clone()).unwrap();
        p.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
            port: port.clone(),
            point: Some(points[i]),
        })
        .unwrap();
        ports.push(port);
    }
    (p, page, ports, points)
}

fn route(p: &mut Project, a: &PortRef, b: &PortRef, inner: &[(f64, f64)]) {
    let mut points = vec![p.port_position(a).unwrap()];
    points.extend(inner.iter().map(|(x, y)| pt(*x, *y)));
    points.push(p.port_position(b).unwrap());
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
        source: a.clone(),
        target: b.clone(),
        points,
    })
    .unwrap();
}

fn junction(
    p: &mut Project,
    page: PageId,
    at: Point2D,
) -> Result<ConnectionDrawingDelta, cad_craft::error::DomainError> {
    let routes = p
        .connection_routes()
        .filter(|r| r.contains(at))
        .map(ConnectionRoute::id)
        .collect();
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
        page,
        position: at,
        routes,
    })
}

#[test]
fn a_feed_taken_from_a_shared_port_gets_a_junction_dot() {
    // Port A at (60, 200) feeds port B at (60, 160) and, through a branch that
    // runs back along the same wire, port C at (140, 180): the K1 -> F1 / K3
    // case of a star-delta starter.
    let (mut p, page, ports, points) = setup(&[
        (60., 190., 0., false),
        (60., 150., 0., false),
        (140., 170., 0., false),
    ]);
    route(&mut p, &ports[0], &ports[1], &[]);
    route(&mut p, &ports[2], &ports[0], &[(140., 190.), (60., 190.)]);
    let before = p.clone();
    let delta = junction(&mut p, page, pt(60., 190.)).expect("junção no ponto em T");
    assert_eq!(p.connection_junctions().count(), 1);
    assert!(p.are_connected(points[1], points[2]).unwrap());
    p.validate().unwrap();
    p.restore_connection_drawing(&delta, false).unwrap();
    assert_eq!(p, before);
}

#[test]
fn a_junction_on_a_single_wire_explains_what_is_missing() {
    let (mut p, page, ports, _) = setup(&[(60., 190., 0., false), (60., 150., 0., false)]);
    route(&mut p, &ports[0], &ports[1], &[]);
    let error = junction(&mut p, page, pt(60., 180.))
        .unwrap_err()
        .to_string();
    assert!(error.contains("pelo menos duas conexões"), "{error}");
}

#[test]
fn a_port_a_hair_off_the_grid_still_accepts_the_tee_junction() {
    // 0.1 + 0.2 = 0.30000000000000004: the ports sit a hair off the grid
    // value 0.3 where the junction is clicked.
    let x = 0.1 + 0.2;
    let (mut p, page, ports, points) = setup(&[
        (x, 150., 0., false),
        (x, 110., 0., false),
        (60., 120., 0., false),
    ]);
    assert_ne!(p.port_position(&ports[0]).unwrap().x(), 0.3);
    route(&mut p, &ports[0], &ports[1], &[]);
    route(&mut p, &ports[2], &ports[1], &[(60., 140.), (x, 140.)]);
    junction(&mut p, page, pt(0.3, 140.)).expect("junção com tolerância");
    assert!(p.are_connected(points[0], points[2]).unwrap());
    p.validate().unwrap();
}
