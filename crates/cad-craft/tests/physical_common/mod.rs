// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]
#![allow(dead_code)]
use cad_craft::domain::*;

pub fn part(s: impl Into<String>) -> DesignationPart {
    DesignationPart::new(s).unwrap()
}
pub fn fixture(
    count: usize,
) -> (
    Project,
    DeviceId,
    Vec<PhysicalTermination>,
    Vec<ConnectionPointId>,
) {
    let mut p = Project::new("Fiação física").unwrap();
    let device = p
        .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "device").unwrap())
        .unwrap();
    let function = p
        .insert_function(Function::new(
            p.id(),
            device,
            Designation::new("K1").unwrap(),
            FunctionKind::Coil,
        ))
        .unwrap();
    let mut ends = Vec::new();
    let mut points = Vec::new();
    for i in 0..count {
        let terminal = p
            .insert_terminal(
                Terminal::new(p.id(), TerminalOwner::Device(device), part(format!("T{i}")))
                    .with_positions(vec![part("A"), part("B")])
                    .unwrap(),
            )
            .unwrap();
        ends.push(PhysicalTermination::new(terminal, part("A")));
        points.push(
            p.insert_connection_point(
                ConnectionPoint::new(p.id(), function, part(format!("P{i}")))
                    .with_terminal(terminal),
            )
            .unwrap(),
        );
    }
    (p, device, ends, points)
}
pub fn wire(
    p: &Project,
    name: impl Into<String>,
    a: &PhysicalTermination,
    b: &PhysicalTermination,
) -> Conductor {
    Conductor::new(
        p.id(),
        part(name),
        a.clone(),
        b.clone(),
        CrossSectionMm2::new(1.5).unwrap(),
        ConductorMarking::color(ConductorColor::Black),
    )
    .unwrap()
}
pub fn cable(p: &mut Project, count: u32) -> CableId {
    p.insert_cable(Cable::new(p.id(), part("C1"), CableKind::Control, count).unwrap())
        .unwrap()
}
pub fn core(p: &mut Project, cable: CableId, name: impl Into<String>) -> CableCoreId {
    p.insert_cable_core(
        CableCore::new(
            p.id(),
            cable,
            part(name),
            ConductorMarking::color(ConductorColor::Black),
        )
        .with_cross_section(CrossSectionMm2::new(1.5).unwrap()),
    )
    .unwrap()
}
pub fn connection(p: &mut Project, a: ConnectionPointId, b: ConnectionPointId) -> ConnectionId {
    p.insert_connection(Connection::new(p.id(), a, b).unwrap())
        .unwrap()
}
pub fn populated() -> Project {
    let (mut p, _, ends, points) = fixture(4);
    let cable = cable(&mut p, 4);
    let strip = p
        .insert_terminal_strip(TerminalStrip::new(p.id(), part("X1")))
        .unwrap();
    for i in 0..4 {
        let core = core(&mut p, cable, format!("{i}"));
        p.attach_terminal(strip, ends[i].terminal_id()).unwrap();
        if i < 3 {
            let wire = p
                .insert_conductor(wire(&p, format!("W{i}"), &ends[i], &ends[i + 1]))
                .unwrap();
            p.assign_core_conductor(core, wire).unwrap();
            let edge = connection(&mut p, points[i], points[i + 1]);
            p.assign_conductor_connection(wire, ConductorConnection::new(edge, points[i]))
                .unwrap();
        }
    }
    p
}
