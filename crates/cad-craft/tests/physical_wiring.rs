// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod physical_common;
use cad_craft::{domain::*, error::DomainError};
use physical_common::*;

#[test]
fn individual_wire_preserves_physical_data_without_creating_logical_edges() {
    let (mut p, device, ends, _) = fixture(2);
    let networks = p.networks();
    let c = wire(&p, "W1", &ends[0], &ends[1])
        .with_length(LengthMm::from_meters(3.0).unwrap())
        .with_material(ConductorMaterial::Copper)
        .with_function(ConductorFunction::Control);
    let id = c.id();
    p.insert_conductor(c.clone()).unwrap();
    assert_eq!(p.conductor(id), Some(&c));
    assert_eq!(p.conductor(id).unwrap().length().unwrap().get(), 3000.0);
    assert_eq!(p.cable_for_conductor(id), None);
    assert_eq!(p.core_for_conductor(id), None);
    assert_eq!(p.functional_points_for_conductor(id).unwrap(), None);
    assert_eq!(p.physical_for_device(device).unwrap().conductors(), &[id]);
    assert_eq!(p.networks(), networks);
    p.validate().unwrap();
}

#[test]
fn four_independent_cores_and_unused_cores_are_representable() {
    let (mut p, _, ends, _) = fixture(2);
    let networks = p.networks();
    let cable = cable(&mut p, 5);
    for i in 0..4 {
        let core = core(&mut p, cable, format!("{i}"));
        let conductor = p
            .insert_conductor(wire(&p, format!("W{i}"), &ends[0], &ends[1]))
            .unwrap();
        p.assign_core_conductor(core, conductor).unwrap();
        assert_eq!(p.cable_core(core).unwrap().conductor_id(), Some(conductor));
        assert_eq!(p.core_for_conductor(conductor).unwrap().id(), core);
        assert_eq!(p.cable_for_conductor(conductor).unwrap().id(), cable);
    }
    let unused = core(&mut p, cable, "spare");
    assert_eq!(p.cable_core(unused).unwrap().conductor_id(), None);
    assert_eq!(p.cores_for_cable(cable).count(), 5);
    assert_eq!(p.networks(), networks);
    p.validate().unwrap();
}

#[test]
fn strip_preserves_order_designations_and_rejects_multiple_membership() {
    let (mut p, device, ends, _) = fixture(3);
    let strip = p
        .insert_terminal_strip(
            TerminalStrip::new(p.id(), part("X1")).with_location(Text::new("Painel A").unwrap()),
        )
        .unwrap();
    let other = p
        .insert_terminal_strip(TerminalStrip::new(p.id(), part("X2")))
        .unwrap();
    for i in [2, 0, 1] {
        p.attach_terminal(strip, ends[i].terminal_id()).unwrap();
    }
    assert_eq!(
        p.terminal_strip(strip).unwrap().terminals(),
        &[
            ends[2].terminal_id(),
            ends[0].terminal_id(),
            ends[1].terminal_id()
        ]
    );
    assert_eq!(
        p.terminal(ends[2].terminal_id())
            .unwrap()
            .designation()
            .as_str(),
        "T2"
    );
    let before = p.clone();
    assert!(p.attach_terminal(strip, ends[0].terminal_id()).is_err());
    assert!(p.attach_terminal(other, ends[0].terminal_id()).is_err());
    assert!(p.attach_terminal(strip, TerminalId::new()).is_err());
    assert!(
        p.attach_terminal(TerminalStripId::new(), ends[0].terminal_id())
            .is_err()
    );
    assert_eq!(p, before);
    assert_eq!(
        p.strip_for_terminal(ends[0].terminal_id()).unwrap().id(),
        strip
    );
    assert_eq!(
        p.physical_for_device(device).unwrap().terminal_strips(),
        &[strip]
    );
    p.validate().unwrap();
}

#[test]
fn physical_positions_require_registration_and_allow_same_terminal_different_positions() {
    let (mut p, _, ends, _) = fixture(1);
    let b = PhysicalTermination::new(ends[0].terminal_id(), part("B"));
    p.insert_conductor(wire(&p, "bridge", &ends[0], &b))
        .unwrap();
    let c = PhysicalTermination::new(ends[0].terminal_id(), part("C"));
    let before = p.clone();
    assert!(p.insert_conductor(wire(&p, "W2", &ends[0], &c)).is_err());
    assert!(
        p.add_terminal_position(ends[0].terminal_id(), part("A"))
            .is_err()
    );
    assert!(
        p.add_terminal_position(TerminalId::new(), part("A"))
            .is_err()
    );
    assert_eq!(p, before);
    p.add_terminal_position(ends[0].terminal_id(), part("C"))
        .unwrap();
    p.insert_conductor(wire(&p, "W2", &ends[0], &c)).unwrap();
    assert_eq!(p.networks().len(), 1);
    p.validate().unwrap();
}

#[test]
fn duplicate_parallel_designation_is_rejected_in_both_directions() {
    let (mut p, _, ends, _) = fixture(2);
    let c = wire(&p, "W1", &ends[0], &ends[1]);
    p.insert_conductor(c.clone()).unwrap();
    let before = p.clone();
    assert!(p.insert_conductor(c).is_err());
    assert!(
        p.insert_conductor(wire(&p, "W1", &ends[1], &ends[0]))
            .is_err()
    );
    assert_eq!(p, before);
    p.insert_conductor(wire(&p, "W2", &ends[1], &ends[0]))
        .unwrap();
    p.validate().unwrap();
}

#[test]
fn invalid_insertions_leave_all_registries_and_indexes_unchanged() {
    let (mut p, _, ends, _) = fixture(2);
    let foreign = ProjectId::new();
    let missing = PhysicalTermination::new(TerminalId::new(), part("A"));
    let before = p.clone();
    assert!(
        p.insert_conductor(wire(&p, "W1", &ends[0], &missing))
            .is_err()
    );
    assert!(
        p.insert_conductor(
            Conductor::new(
                foreign,
                part("W1"),
                ends[0].clone(),
                ends[1].clone(),
                CrossSectionMm2::new(1.5).unwrap(),
                ConductorMarking::color(ConductorColor::Black)
            )
            .unwrap()
        )
        .is_err()
    );
    assert!(
        p.insert_cable(Cable::new(foreign, part("C"), CableKind::Control, 4).unwrap())
            .is_err()
    );
    assert!(
        p.insert_cable_core(CableCore::new(
            p.id(),
            CableId::new(),
            part("1"),
            ConductorMarking::identified(part("1"))
        ))
        .is_err()
    );
    assert!(
        p.insert_terminal_strip(TerminalStrip::new(foreign, part("X")))
            .is_err()
    );
    assert_eq!(p, before);
}

#[test]
fn core_capacity_names_ids_and_projects_are_checked_atomically() {
    let (mut p, _, _, _) = fixture(2);
    let cable = cable(&mut p, 1);
    let first = core(&mut p, cable, "1");
    let before = p.clone();
    assert!(p.insert_cable(p.cable(cable).unwrap().clone()).is_err());
    assert!(
        p.insert_cable_core(p.cable_core(first).unwrap().clone())
            .is_err()
    );
    assert!(
        p.insert_cable_core(CableCore::new(
            p.id(),
            cable,
            part("1"),
            ConductorMarking::identified(part("1"))
        ))
        .is_err()
    );
    assert!(
        p.insert_cable_core(CableCore::new(
            p.id(),
            cable,
            part("2"),
            ConductorMarking::identified(part("2"))
        ))
        .is_err()
    );
    assert!(
        p.insert_cable_core(CableCore::new(
            ProjectId::new(),
            cable,
            part("3"),
            ConductorMarking::identified(part("3"))
        ))
        .is_err()
    );
    assert_eq!(p, before);
}

#[test]
fn core_assignment_is_exclusive_compatible_and_explicitly_reversible() {
    let (mut p, _, ends, _) = fixture(2);
    let cable = cable(&mut p, 3);
    let a = core(&mut p, cable, "1");
    let b = core(&mut p, cable, "2");
    let wrong = p
        .insert_cable_core(CableCore::new(
            p.id(),
            cable,
            part("3"),
            ConductorMarking::color(ConductorColor::Blue),
        ))
        .unwrap();
    let w = p
        .insert_conductor(wire(&p, "W1", &ends[0], &ends[1]))
        .unwrap();
    p.assign_core_conductor(a, w).unwrap();
    let before = p.clone();
    for target in [a, b, wrong, CableCoreId::new()] {
        assert!(p.assign_core_conductor(target, w).is_err());
    }
    assert!(p.assign_core_conductor(b, ConductorId::new()).is_err());
    assert!(p.unassign_core_conductor(CableCoreId::new()).is_err());
    assert_eq!(p, before);
    assert_eq!(p.unassign_core_conductor(a).unwrap(), Some(w));
    assert_eq!(p.core_for_conductor(w), None);
    let before = p.clone();
    assert!(p.assign_core_conductor(wrong, w).is_err());
    assert_eq!(p, before);
    p.assign_core_conductor(b, w).unwrap();
    p.validate().unwrap();
}

#[test]
fn section_mismatch_cannot_install_a_core() {
    let (mut p, _, ends, _) = fixture(2);
    let cable = cable(&mut p, 1);
    let core = p
        .insert_cable_core(
            CableCore::new(
                p.id(),
                cable,
                part("1"),
                ConductorMarking::identified(part("1")),
            )
            .with_cross_section(CrossSectionMm2::new(2.5).unwrap()),
        )
        .unwrap();
    let w = p
        .insert_conductor(wire(&p, "W1", &ends[0], &ends[1]))
        .unwrap();
    let before = p.clone();
    assert!(p.assign_core_conductor(core, w).is_err());
    assert_eq!(p, before);
}

#[test]
fn explicit_logical_link_traces_reversed_orientation_without_changing_networks() {
    let (mut p, _, ends, points) = fixture(2);
    let edge = connection(&mut p, points[0], points[1]);
    assert!(
        p.physical_for_connection(edge)
            .unwrap()
            .conductors()
            .is_empty()
    );
    let networks = p.networks();
    let w = p
        .insert_conductor(wire(&p, "W1", &ends[1], &ends[0]))
        .unwrap();
    let cable = cable(&mut p, 1);
    let core = core(&mut p, cable, "1");
    p.assign_core_conductor(core, w).unwrap();
    p.assign_conductor_connection(w, ConductorConnection::new(edge, points[1]))
        .unwrap();
    assert_eq!(
        p.functional_points_for_conductor(w).unwrap(),
        Some((points[1], points[0]))
    );
    let elements = p.physical_for_connection(edge).unwrap();
    assert_eq!(elements.conductors(), &[w]);
    assert_eq!(elements.cables(), &[cable]);
    assert_eq!(elements.cable_cores(), &[core]);
    assert_eq!(p.networks(), networks);
    let before = p.clone();
    assert_eq!(
        p.remove_connection(edge),
        Err(DomainError::ConnectionInUse(edge))
    );
    assert_eq!(p, before);
    p.unassign_conductor_connection(w).unwrap();
    p.remove_connection(edge).unwrap();
    assert!(p.conductor(w).is_some());
    assert_eq!(p.networks().len(), 2);
    p.validate().unwrap();
}

#[test]
fn invalid_logical_associations_never_replace_an_existing_link() {
    let (mut p, _, ends, points) = fixture(3);
    let edge = connection(&mut p, points[0], points[1]);
    let incompatible = connection(&mut p, points[1], points[2]);
    let w = p
        .insert_conductor(wire(&p, "W1", &ends[0], &ends[1]))
        .unwrap();
    let before = p.clone();
    for link in [
        ConductorConnection::new(edge, points[1]),
        ConductorConnection::new(edge, points[2]),
        ConductorConnection::new(incompatible, points[1]),
        ConductorConnection::new(ConnectionId::new(), points[0]),
    ] {
        assert!(p.assign_conductor_connection(w, link).is_err());
    }
    assert!(
        p.assign_conductor_connection(
            ConductorId::new(),
            ConductorConnection::new(edge, points[0])
        )
        .is_err()
    );
    assert!(p.physical_for_connection(ConnectionId::new()).is_err());
    assert!(p.physical_for_device(DeviceId::new()).is_err());
    assert!(
        p.functional_points_for_conductor(ConductorId::new())
            .is_err()
    );
    assert!(p.unassign_conductor_connection(ConductorId::new()).is_err());
    assert_eq!(p, before);
    p.assign_conductor_connection(w, ConductorConnection::new(edge, points[0]))
        .unwrap();
    let before = p.clone();
    assert!(
        p.assign_conductor_connection(w, ConductorConnection::new(incompatible, points[1]))
            .is_err()
    );
    assert_eq!(p, before);
}

#[test]
fn physical_properties_resolve_references_and_do_not_duplicate_nominal_section() {
    let (mut p, _, ends, _) = fixture(2);
    let cable = cable(&mut p, 1);
    let core = core(&mut p, cable, "1");
    let w = p
        .insert_conductor(wire(&p, "W1", &ends[0], &ends[1]))
        .unwrap();
    let strip = p
        .insert_terminal_strip(TerminalStrip::new(p.id(), part("X1")))
        .unwrap();
    for target in [
        EntityRef::Conductor(w),
        EntityRef::Cable(cable),
        EntityRef::CableCore(core),
        EntityRef::TerminalStrip(strip),
    ] {
        let mut props = TechnicalProperties::default();
        props
            .set_custom("cable", PropertyValue::Reference(EntityRef::Cable(cable)))
            .unwrap();
        p.set_properties(target, props.clone()).unwrap();
        let before = p.clone();
        props
            .set_custom(
                "missing",
                PropertyValue::Reference(EntityRef::Conductor(ConductorId::new())),
            )
            .unwrap();
        assert!(p.set_properties(target, props).is_err());
        assert_eq!(p, before);
    }
    let mut redundant = TechnicalProperties::default();
    redundant
        .set_known(KnownProperty::CrossSection, 1.5)
        .unwrap();
    let before = p.clone();
    assert!(
        p.set_properties(EntityRef::Conductor(w), redundant.clone())
            .is_err()
    );
    assert!(
        p.set_properties(EntityRef::CableCore(core), redundant)
            .is_err()
    );
    assert_eq!(p, before);
    p.validate().unwrap();
}

#[test]
fn device_queries_include_component_owned_terminals_without_implicit_continuity() {
    let (mut p, device, ends, _) = fixture(1);
    let component = Component::new(p.id(), Designation::new("C1").unwrap(), "terminal")
        .unwrap()
        .with_assignment(Some(device), None);
    let component = p.insert_component(component).unwrap();
    let terminal = p
        .insert_terminal(
            Terminal::new(p.id(), TerminalOwner::Component(component), part("1"))
                .with_positions(vec![part("A")])
                .unwrap(),
        )
        .unwrap();
    let w = p
        .insert_conductor(wire(
            &p,
            "W1",
            &ends[0],
            &PhysicalTermination::new(terminal, part("A")),
        ))
        .unwrap();
    let physical = p.physical_for_device(device).unwrap();
    assert_eq!(physical.conductors(), &[w]);
    assert_eq!(physical.terminals().len(), 2);
    assert_eq!(p.connections().count(), 0);
}
