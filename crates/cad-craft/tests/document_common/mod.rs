// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]
#![allow(dead_code)]
use cad_craft::domain::*;

pub fn fixture() -> (Project, PageId, PageId, DeviceId, ConnectionId) {
    let mut p = Project::new("Documental").unwrap();
    let doc = p
        .insert_document(Document::new(p.id(), "D").unwrap())
        .unwrap();
    let a = p
        .insert_page(
            Page::new(
                p.id(),
                doc,
                "Potência",
                "1",
                PageKind::MultiLine,
                Size2D::new(297.0, 210.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let b = p
        .insert_page(
            Page::new(
                p.id(),
                doc,
                "Geral",
                "2",
                PageKind::Overview,
                Size2D::new(210.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let d = p
        .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "Contator").unwrap())
        .unwrap();
    let f = p
        .insert_function(Function::new(
            p.id(),
            d,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let points: Vec<_> = (1..=2)
        .map(|i| {
            p.insert_connection_point(ConnectionPoint::new(
                p.id(),
                f,
                DesignationPart::new(format!("A{i}")).unwrap(),
            ))
            .unwrap()
        })
        .collect();
    let c = p
        .insert_connection(Connection::new(p.id(), points[0], points[1]).unwrap())
        .unwrap();
    (p, a, b, d, c)
}
pub fn transform(x: f64, y: f64) -> Transform2D {
    Transform2D::new(Point2D::new(x, y).unwrap(), 0.0, false, false).unwrap()
}
pub fn rep(p: &Project, page: PageId, target: RepresentationTarget) -> Representation {
    Representation::new(p.id(), page, target, transform(10.0, 20.0))
}
