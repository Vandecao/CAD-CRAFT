// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 11 — conectores industriais: contatos como bornes existentes,
//! associação explícita a pontos funcionais e acoplamento documental.
mod physical_common;
use cad_craft::{domain::*, error::DomainError};
use physical_common::part;

/// Device + function + `n` connection points without physical terminal.
fn device(p: &mut Project, name: &str, n: usize) -> (DeviceId, Vec<ConnectionPointId>) {
    let device = p
        .insert_device(Device::new(p.id(), Designation::new(name).unwrap(), "conector").unwrap())
        .unwrap();
    let function = p
        .insert_function(Function::new(
            p.id(),
            device,
            Designation::new(name).unwrap(),
            FunctionKind::Other(Text::new("conector").unwrap()),
        ))
        .unwrap();
    let points = (0..n)
        .map(|i| {
            p.insert_connection_point(ConnectionPoint::new(
                p.id(),
                function,
                part(format!("{}", i + 1)),
            ))
            .unwrap()
        })
        .collect();
    (device, points)
}

/// Builds a connector with `n` contacts numbered 1..=n in one atomic batch.
fn connector_edits(
    p: &Project,
    device: DeviceId,
    name: &str,
    gender: ConnectorGender,
    n: usize,
) -> (Vec<PhysicalEdit>, ConnectorId, Vec<TerminalId>) {
    let mut edits = Vec::new();
    let mut contacts = Vec::new();
    for i in 1..=n {
        let t = Terminal::new(p.id(), TerminalOwner::Device(device), part(i.to_string()))
            .with_positions(vec![part("c")])
            .unwrap();
        contacts.push(t.id());
        edits.push(PhysicalEdit::UpsertTerminal(t));
    }
    let connector = Connector::new(p.id(), part(name))
        .with_gender(gender)
        .with_device(Some(device))
        .with_contacts(contacts.clone());
    let id = connector.id();
    edits.push(PhysicalEdit::UpsertConnector(connector));
    (edits, id, contacts)
}

fn make(
    p: &mut Project,
    device: DeviceId,
    name: &str,
    gender: ConnectorGender,
    n: usize,
) -> (ConnectorId, Vec<TerminalId>) {
    let (edits, id, contacts) = connector_edits(p, device, name, gender, n);
    p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
    (id, contacts)
}

fn associate(p: &mut Project, point: ConnectionPointId, terminal: TerminalId) -> PhysicalDelta {
    p.edit_physical(PhysicalEdit::SetPointTerminal {
        point,
        terminal: Some(terminal),
        level: None,
    })
    .unwrap()
}

#[test]
fn four_position_connector_is_created_atomically_and_undone_exactly() {
    let mut p = Project::new("Conectores").unwrap();
    let (dev, _) = device(&mut p, "X1", 4);
    let before = p.clone();
    let (edits, id, contacts) = connector_edits(&p, dev, "X1", ConnectorGender::Male, 4);
    let delta = p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
    let connector = p.connector(id).unwrap();
    assert_eq!(connector.contacts(), contacts.as_slice());
    assert_eq!(connector.gender(), ConnectorGender::Male);
    assert_eq!(connector.device(), Some(dev));
    assert_eq!(p.terminal_label(contacts[2]), "X1:3");
    assert_eq!(p.connector_for_terminal(contacts[0]).unwrap().id(), id);
    let row = &p.physical_lists().connectors[0];
    assert_eq!(row.contacts.len(), 4);
    assert!(row.contacts.iter().all(ConnectorContactRow::is_free));
    let after = p.clone();
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();
    assert_eq!(p, after);
}

#[test]
fn contacts_are_added_and_removed_without_partial_state() {
    let mut p = Project::new("Conectores").unwrap();
    let (dev, points) = device(&mut p, "X1", 4);
    let (id, contacts) = make(&mut p, dev, "X1", ConnectorGender::Female, 4);
    // Add a fifth contact.
    let extra = Terminal::new(p.id(), TerminalOwner::Device(dev), part("5"));
    let extra_id = extra.id();
    let mut order = contacts.clone();
    order.push(extra_id);
    let grown = p.connector(id).unwrap().clone().with_contacts(order);
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(extra),
        PhysicalEdit::UpsertConnector(grown),
    ]))
    .unwrap();
    assert_eq!(p.connector(id).unwrap().contacts().len(), 5);
    // A contact still in the connector cannot be deleted alone.
    assert_eq!(
        p.edit_physical(PhysicalEdit::DeleteTerminal(extra_id)),
        Err(DomainError::EntityInUse(EntityRef::Terminal(extra_id)))
    );
    // Removing a free contact is one batch: release from connector + delete.
    let shrink = |p: &Project, t: TerminalId| {
        let c = p.connector(id).unwrap();
        let order = c.contacts().iter().copied().filter(|x| *x != t).collect();
        PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertConnector(c.clone().with_contacts(order)),
            PhysicalEdit::DeleteTerminal(t),
        ])
    };
    p.edit_physical(shrink(&p, extra_id)).unwrap();
    assert!(p.terminal(extra_id).is_none());
    assert_eq!(p.connector(id).unwrap().contacts(), contacts.as_slice());
    // A contact associated with a point cannot disappear: whole batch rejected.
    associate(&mut p, points[0], contacts[0]);
    let snapshot = p.clone();
    assert!(p.edit_physical(shrink(&p, contacts[0])).is_err());
    assert_eq!(p, snapshot);
}

#[test]
fn renumbering_contacts_keeps_ids_and_rejects_duplicates() {
    let mut p = Project::new("Conectores").unwrap();
    let (dev, _) = device(&mut p, "X1", 4);
    let (id, contacts) = make(&mut p, dev, "X1", ConnectorGender::Male, 4);
    // Reverse 1..4 → 4..1 atomically; intermediate duplicates are allowed.
    let edits = contacts
        .iter()
        .enumerate()
        .map(|(i, t)| {
            PhysicalEdit::UpsertTerminal(
                p.terminal(*t)
                    .unwrap()
                    .clone()
                    .with_designation(part((4 - i).to_string())),
            )
        })
        .collect();
    p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
    assert_eq!(p.connector(id).unwrap().contacts(), contacts.as_slice());
    assert_eq!(p.terminal(contacts[0]).unwrap().designation().as_str(), "4");
    assert_eq!(p.terminal_label(contacts[3]), "X1:1");
    let snapshot = p.clone();
    let duplicate = p
        .terminal(contacts[1])
        .unwrap()
        .clone()
        .with_designation(part("4"));
    assert!(
        p.edit_physical(PhysicalEdit::UpsertTerminal(duplicate))
            .is_err()
    );
    assert_eq!(p, snapshot);
    // Alphanumeric positions are accepted.
    let alpha = p
        .terminal(contacts[1])
        .unwrap()
        .clone()
        .with_designation(part("PE"));
    p.edit_physical(PhysicalEdit::UpsertTerminal(alpha))
        .unwrap();
    assert_eq!(p.terminal_label(contacts[1]), "X1:PE");
}

#[test]
fn contacts_associate_explicitly_and_incompatible_points_are_rejected() {
    let mut p = Project::new("Conectores").unwrap();
    let (x1, points) = device(&mut p, "X1", 2);
    let (_, foreign) = device(&mut p, "K1", 1);
    let (_, contacts) = make(&mut p, x1, "X1", ConnectorGender::Male, 2);
    let edge = physical_common::connection(&mut p, points[0], points[1]);
    let networks = p.networks();
    let before = p.clone();
    let delta = associate(&mut p, points[0], contacts[0]);
    assert_eq!(p.networks(), networks, "association is not continuity");
    let status = p.contact_status(contacts[0]).unwrap();
    assert_eq!(status.points, vec![points[0]]);
    assert!(!status.is_free());
    assert!(p.contact_status(contacts[1]).unwrap().is_free());
    // A point of another device cannot be associated with this contact.
    let snapshot = p.clone();
    assert!(
        p.edit_physical(PhysicalEdit::SetPointTerminal {
            point: foreign[0],
            terminal: Some(contacts[1]),
            level: None,
        })
        .is_err()
    );
    // A level is meaningless for a generic contact.
    assert!(
        p.edit_physical(PhysicalEdit::SetPointTerminal {
            point: points[1],
            terminal: Some(contacts[1]),
            level: Some(TerminalLevelId::new()),
        })
        .is_err()
    );
    assert_eq!(p, snapshot);
    // Disassociation keeps the logical connection.
    p.edit_physical(PhysicalEdit::SetPointTerminal {
        point: points[0],
        terminal: None,
        level: None,
    })
    .unwrap();
    assert!(p.connection(edge).is_some());
    assert!(p.contact_status(contacts[0]).unwrap().is_free());
    p.restore_physical(
        &p.clone()
            .edit_physical(PhysicalEdit::SetPointTerminal {
                point: points[0],
                terminal: Some(contacts[0]),
                level: None,
            })
            .unwrap(),
        true,
    )
    .unwrap();
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
}

#[test]
fn contact_used_by_conductor_cannot_be_disassociated_from_its_point() {
    let mut p = Project::new("Conectores").unwrap();
    let (x1, points) = device(&mut p, "X1", 2);
    let (_, contacts) = make(&mut p, x1, "X1", ConnectorGender::Male, 2);
    associate(&mut p, points[0], contacts[0]);
    associate(&mut p, points[1], contacts[1]);
    let edge = physical_common::connection(&mut p, points[0], points[1]);
    let a = PhysicalTermination::new(contacts[0], part("c"));
    let b = PhysicalTermination::new(contacts[1], part("c"));
    let wire = physical_common::wire(&p, "W1", &a, &b);
    let wire_id = wire.id();
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertConductor(wire),
        PhysicalEdit::SetConductorConnection {
            conductor: wire_id,
            link: Some(ConductorConnection::new(edge, points[0])),
        },
    ]))
    .unwrap();
    assert_eq!(
        p.contact_status(contacts[0]).unwrap().conductors,
        vec![wire_id]
    );
    let snapshot = p.clone();
    assert!(
        p.edit_physical(PhysicalEdit::SetPointTerminal {
            point: points[0],
            terminal: None,
            level: None,
        })
        .is_err(),
        "the conductor link depends on the association"
    );
    assert_eq!(p, snapshot);
}

#[test]
fn connectors_with_equal_names_never_create_continuity() {
    let mut p = Project::new("Conectores").unwrap();
    let (a, pa) = device(&mut p, "X1", 2);
    let (b, pb) = device(&mut p, "X2", 2);
    let (_, ca) = make(&mut p, a, "X1", ConnectorGender::Male, 2);
    let (_, cb) = make(&mut p, b, "X1", ConnectorGender::Female, 2);
    for i in 0..2 {
        associate(&mut p, pa[i], ca[i]);
        associate(&mut p, pb[i], cb[i]);
    }
    for i in 0..2 {
        assert!(!p.are_connected(pa[i], pb[i]).unwrap());
    }
    assert!(p.networks().iter().all(|n| n.points().len() == 1));
}

#[test]
fn mating_is_explicit_validated_and_gives_continuity() {
    let mut p = Project::new("Acoplamento").unwrap();
    let (a, pa) = device(&mut p, "X1", 2);
    let (b, pb) = device(&mut p, "X2", 2);
    let (plug, ca) = make(&mut p, a, "X1", ConnectorGender::Male, 2);
    let (socket, cb) = make(&mut p, b, "X2", ConnectorGender::Female, 2);
    for i in 0..2 {
        associate(&mut p, pa[i], ca[i]);
        associate(&mut p, pb[i], cb[i]);
    }
    let networks = p.networks();
    let cross = vec![
        ContactPair::new(ca[0], cb[1]),
        ContactPair::new(ca[1], cb[0]),
    ];
    let mating = ConnectorMating::new(p.id(), plug, socket, cross);
    let mating_id = mating.id();
    let before = p.clone();
    let delta = p.edit_physical(PhysicalEdit::UpsertMating(mating)).unwrap();
    // Mapping is the explicit one, not the equal numbering.
    assert_eq!(p.mated_contact(ca[0]), Some(cb[1]));
    assert_eq!(p.mated_contact(cb[0]), Some(ca[1]));
    assert_eq!(p.contact_status(ca[0]).unwrap().mated_with, Some(cb[1]));
    // Etapa 24 (DEC-11-01, proposta A): o acoplamento possui uma conexão por
    // par com ponto e as redes atravessam o conector, pelo mapa explícito.
    assert_ne!(p.networks(), networks);
    assert!(p.are_connected(pa[0], pb[1]).unwrap());
    assert!(p.are_connected(pa[1], pb[0]).unwrap());
    assert!(!p.are_connected(pa[0], pb[0]).unwrap());
    let owned = p
        .connector_mating(mating_id)
        .unwrap()
        .owned_connections()
        .to_vec();
    assert_eq!(owned.len(), 2);
    let row = p
        .physical_lists()
        .connectors
        .into_iter()
        .find(|c| c.connector == plug)
        .unwrap();
    assert_eq!(row.mated_with.as_deref(), Some("X2"));
    // Deleting a mated connector is refused without cascade.
    let snapshot = p.clone();
    assert_eq!(
        p.edit_physical(PhysicalEdit::DeleteConnector(plug)),
        Err(DomainError::EntityInUse(EntityRef::Connector(plug)))
    );
    assert_eq!(p, snapshot);
    // A second mating of the same connector is a conflict.
    let other = ConnectorMating::new(p.id(), socket, plug, vec![ContactPair::new(cb[0], ca[0])]);
    assert!(p.edit_physical(PhysicalEdit::UpsertMating(other)).is_err());
    assert_eq!(p, snapshot);
    p.restore_physical(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_physical(&delta, true).unwrap();
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::DeleteMating(mating_id),
        PhysicalEdit::DeleteConnector(plug),
    ]))
    .unwrap();
    assert!(p.connector(plug).is_none());
    assert!(p.terminal(ca[0]).is_some(), "contacts are not cascaded");
}

#[test]
fn incompatible_or_inconsistent_matings_are_rejected() {
    let mut p = Project::new("Acoplamento").unwrap();
    let (a, _) = device(&mut p, "X1", 0);
    let (b, _) = device(&mut p, "X2", 0);
    let (c, _) = device(&mut p, "X3", 0);
    let (male, cm) = make(&mut p, a, "X1", ConnectorGender::Male, 2);
    let (female, cf) = make(&mut p, b, "X2", ConnectorGender::Female, 2);
    let (other_male, co) = make(&mut p, c, "X3", ConnectorGender::Male, 2);
    let snapshot = p.clone();
    let id = p.id();
    let attempts = [
        // Male ↔ male.
        ConnectorMating::new(id, male, other_male, vec![ContactPair::new(cm[0], co[0])]),
        // Contact from the wrong connector.
        ConnectorMating::new(id, male, female, vec![ContactPair::new(co[0], cf[0])]),
        // Same contact mapped twice.
        ConnectorMating::new(
            id,
            male,
            female,
            vec![
                ContactPair::new(cm[0], cf[0]),
                ContactPair::new(cm[0], cf[1]),
            ],
        ),
        // Nonexistent contact.
        ConnectorMating::new(
            id,
            male,
            female,
            vec![ContactPair::new(TerminalId::new(), cf[0])],
        ),
        // Empty map.
        ConnectorMating::new(id, male, female, vec![]),
        // Self mating.
        ConnectorMating::new(id, male, male, vec![ContactPair::new(cm[0], cm[1])]),
        // Missing connector.
        ConnectorMating::new(
            id,
            male,
            ConnectorId::new(),
            vec![ContactPair::new(cm[0], cf[0])],
        ),
    ];
    for mating in attempts {
        assert!(
            p.edit_physical(PhysicalEdit::UpsertMating(mating.clone()))
                .is_err(),
            "{mating:?}"
        );
        assert_eq!(p, snapshot);
    }
    // Undeclared gender does not mate.
    let undeclared = p
        .connector(female)
        .unwrap()
        .clone()
        .with_gender(ConnectorGender::Unspecified);
    p.edit_physical(PhysicalEdit::UpsertConnector(undeclared))
        .unwrap();
    assert!(
        p.edit_physical(PhysicalEdit::UpsertMating(ConnectorMating::new(
            id,
            male,
            female,
            vec![ContactPair::new(cm[0], cf[0])],
        )))
        .is_err()
    );
    // Changing the gender of a mated connector is rejected as well.
    let mut q = snapshot.clone();
    q.edit_physical(PhysicalEdit::UpsertMating(ConnectorMating::new(
        id,
        male,
        female,
        vec![ContactPair::new(cm[0], cf[0])],
    )))
    .unwrap();
    let state = q.clone();
    let wrong = q
        .connector(female)
        .unwrap()
        .clone()
        .with_gender(ConnectorGender::Male);
    assert!(
        q.edit_physical(PhysicalEdit::UpsertConnector(wrong))
            .is_err()
    );
    assert_eq!(q, state);
}

#[test]
fn connector_relations_and_membership_are_validated() {
    let mut p = Project::new("Conectores").unwrap();
    let (x1, _) = device(&mut p, "X1", 0);
    let (k1, _) = device(&mut p, "K1", 0);
    let (id, contacts) = make(&mut p, x1, "X1", ConnectorGender::Male, 2);
    let snapshot = p.clone();
    // A contact owned by another device does not belong to a connector of X1.
    let foreign = Terminal::new(p.id(), TerminalOwner::Device(k1), part("3"));
    let mut order = contacts.clone();
    order.push(foreign.id());
    let edit = PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(foreign),
        PhysicalEdit::UpsertConnector(p.connector(id).unwrap().clone().with_contacts(order)),
    ]);
    assert!(p.edit_physical(edit).is_err());
    assert_eq!(p, snapshot);
    // Duplicate numbering inside one connector.
    let renamed = p
        .terminal(contacts[1])
        .unwrap()
        .clone()
        .with_designation(part("1"));
    assert!(
        p.edit_physical(PhysicalEdit::UpsertTerminal(renamed))
            .is_err()
    );
    // A contact cannot also be a strip terminal, nor be in two connectors.
    let strip = TerminalStrip::new(p.id(), part("XT")).with_terminals(vec![contacts[0]]);
    assert!(p.edit_physical(PhysicalEdit::UpsertStrip(strip)).is_err());
    let second = Connector::new(p.id(), part("X9")).with_contacts(vec![contacts[0]]);
    assert!(
        p.edit_physical(PhysicalEdit::UpsertConnector(second))
            .is_err()
    );
    // Contacts are generic terminals.
    let special = p
        .terminal(contacts[0])
        .unwrap()
        .clone()
        .with_kind(TerminalKind::ProtectiveEarth);
    assert!(
        p.edit_physical(PhysicalEdit::UpsertTerminal(special))
            .is_err()
    );
    assert_eq!(p, snapshot);
}

#[test]
fn deleting_connector_with_dependent_contact_is_atomic() {
    let mut p = Project::new("Conectores").unwrap();
    let (x1, points) = device(&mut p, "X1", 4);
    let (id, contacts) = make(&mut p, x1, "X1", ConnectorGender::Male, 4);
    associate(&mut p, points[2], contacts[2]);
    let delete_all = |contacts: &[TerminalId]| {
        let mut edits = vec![PhysicalEdit::DeleteConnector(id)];
        edits.extend(contacts.iter().map(|t| PhysicalEdit::DeleteTerminal(*t)));
        PhysicalEdit::Batch(edits)
    };
    let snapshot = p.clone();
    assert!(p.edit_physical(delete_all(&contacts)).is_err());
    assert_eq!(p, snapshot, "nothing deleted when one contact is in use");
    p.edit_physical(PhysicalEdit::SetPointTerminal {
        point: points[2],
        terminal: None,
        level: None,
    })
    .unwrap();
    p.edit_physical(delete_all(&contacts)).unwrap();
    assert!(p.connector(id).is_none());
    assert!(contacts.iter().all(|t| p.terminal(*t).is_none()));
}

#[test]
fn connectors_and_matings_round_trip_and_bad_snapshots_are_rejected() {
    let mut p = Project::new("Persistência").unwrap();
    let (a, pa) = device(&mut p, "X1", 2);
    let (b, _) = device(&mut p, "X2", 2);
    let (plug, ca) = make(&mut p, a, "X1", ConnectorGender::Male, 2);
    let (socket, cb) = make(&mut p, b, "X2", ConnectorGender::Female, 2);
    associate(&mut p, pa[0], ca[0]);
    p.edit_physical(PhysicalEdit::UpsertMating(ConnectorMating::new(
        p.id(),
        plug,
        socket,
        vec![ContactPair::new(ca[0], cb[0])],
    )))
    .unwrap();
    let json = serde_json::to_value(&p).unwrap();
    let loaded: Project = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(loaded.mated_contact(ca[0]), Some(cb[0]));
    assert_eq!(loaded.connector(plug).unwrap().contacts(), ca.as_slice());

    // A mating pointing at an unknown connector.
    let mut bad = json.clone();
    bad["connector_matings"][0]["second"] = serde_json::json!(ConnectorId::new());
    assert!(serde_json::from_value::<Project>(bad).is_err());
    // A contact listed in two connectors.
    let mut bad = json.clone();
    bad["connectors"][1]["contacts"][0] = serde_json::json!(ca[1]);
    assert!(serde_json::from_value::<Project>(bad).is_err());
    // Duplicate connector ID.
    let mut bad = json.clone();
    let first = bad["connectors"][0].clone();
    bad["connectors"].as_array_mut().unwrap().push(first);
    assert!(serde_json::from_value::<Project>(bad).is_err());
    // Unknown field.
    let mut bad = json;
    bad["connectors"][0]["continuity"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Project>(bad).is_err());
}

#[test]
fn snapshots_without_connectors_load_and_reserialize_unchanged() {
    let p = physical_common::populated();
    let json = serde_json::to_value(&p).unwrap();
    for key in ["connectors", "connector_matings"] {
        assert!(json.get(key).is_none(), "{key} omitted when empty");
    }
    for terminal in json["terminals"].as_array().unwrap() {
        for key in ["kind", "sides", "levels", "disconnect_state"] {
            assert!(terminal.get(key).is_none(), "{key} omitted when default");
        }
    }
    for point in json["connection_points"].as_array().unwrap() {
        assert!(point.get("terminal_level").is_none());
    }
    let loaded: Project = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(loaded, p);
    assert_eq!(serde_json::to_value(&loaded).unwrap(), json);
}
