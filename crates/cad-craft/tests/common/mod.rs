// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;

pub fn fixture(count: usize) -> (Project, Vec<ConnectionPointId>) {
    let mut p = Project::new("Conectividade").unwrap();
    let d = p
        .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "contactor").unwrap())
        .unwrap();
    let f = p
        .insert_function(Function::new(
            p.id(),
            d,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let points = (0..count)
        .map(|i| {
            p.insert_connection_point(ConnectionPoint::new(
                p.id(),
                f,
                DesignationPart::new(format!("P{i}")).unwrap(),
            ))
            .unwrap()
        })
        .collect();
    (p, points)
}

pub fn connect(p: &mut Project, a: ConnectionPointId, b: ConnectionPointId) -> ConnectionId {
    p.insert_connection(Connection::new(p.id(), a, b).unwrap())
        .unwrap()
}
