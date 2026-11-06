// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod common;
use cad_craft::{domain::*, error::DomainError};
use common::{connect, fixture};
use std::collections::BTreeSet;

#[test]
fn connection_ids_follow_v7_validation_and_preserve_identity() {
    let ids: BTreeSet<_> = (0..2048).map(|_| ConnectionId::new()).collect();
    assert_eq!(ids.len(), 2048);
    for id in ids {
        assert_eq!(id.as_uuid().get_version_num(), 7);
        assert_eq!(id.as_uuid().get_variant(), uuid::Variant::RFC4122);
        assert_eq!(id.to_string().parse::<ConnectionId>().unwrap(), id);
        assert_eq!(
            serde_json::from_value::<ConnectionId>(serde_json::to_value(id).unwrap()).unwrap(),
            id
        );
    }
    for invalid in [
        "bad",
        "00000000-0000-0000-0000-000000000000",
        "550e8400-e29b-41d4-a716-446655440000",
        "01890abc-1234-7000-0000-000000000001",
    ] {
        assert!(invalid.parse::<ConnectionId>().is_err());
        assert!(serde_json::from_value::<ConnectionId>(serde_json::json!(invalid)).is_err());
    }
}

#[test]
fn canonical_endpoints_are_undirected_and_self_connections_are_rejected() {
    let (p, ids) = fixture(2);
    let mut sorted = ids.clone();
    sorted.sort();
    for (a, b) in [(ids[0], ids[1]), (ids[1], ids[0])] {
        let c = Connection::new(p.id(), a, b).unwrap();
        assert_eq!(c.endpoints(), (sorted[0], sorted[1]));
        assert_eq!(c.project_id(), p.id());
    }
    assert_eq!(
        Connection::new(p.id(), ids[0], ids[0]),
        Err(DomainError::SelfConnection(ids[0]))
    );
}

#[test]
fn insert_query_remove_and_reinsert_preserve_other_entities() {
    let (mut p, ids) = fixture(3);
    let baseline = p.clone();
    let c = Connection::new(p.id(), ids[1], ids[0]).unwrap();
    let id = p.insert_connection(c.clone()).unwrap();
    assert_eq!(p.connection(id), Some(&c));
    assert_eq!(p.connections().count(), 1);
    p.validate().unwrap();
    assert_eq!(p.remove_connection(id).unwrap(), c);
    assert_eq!(p, baseline);
    assert!(p.connection(id).is_none());
    assert_eq!(p.insert_connection(c).unwrap(), id);
    p.validate().unwrap();
}

#[test]
fn duplicate_ids_and_reversed_edges_fail_without_mutation() {
    let (mut p, ids) = fixture(2);
    let id = connect(&mut p, ids[0], ids[1]);
    let before = p.clone();
    assert_eq!(
        p.insert_connection(p.connection(id).unwrap().clone()),
        Err(DomainError::DuplicateConnectionId(id))
    );
    let mut sorted = ids.clone();
    sorted.sort();
    for (a, b) in [(ids[0], ids[1]), (ids[1], ids[0])] {
        assert_eq!(
            p.insert_connection(Connection::new(p.id(), a, b).unwrap()),
            Err(DomainError::DuplicateConnection {
                source: sorted[0],
                target: sorted[1]
            })
        );
        assert_eq!(p, before);
    }
}

#[test]
fn missing_foreign_points_and_foreign_connections_fail_atomically() {
    let (mut p, ids) = fixture(2);
    let (other, foreign) = fixture(1);
    let before = p.clone();
    for absent in [ConnectionPointId::new(), foreign[0]] {
        for (a, b) in [(ids[0], absent), (absent, ids[1])] {
            assert_eq!(
                p.insert_connection(Connection::new(p.id(), a, b).unwrap()),
                Err(DomainError::MissingEntity(EntityRef::ConnectionPoint(
                    absent
                )))
            );
            assert_eq!(p, before);
        }
    }
    assert_eq!(
        p.insert_connection(Connection::new(other.id(), ids[0], ids[1]).unwrap()),
        Err(DomainError::ForeignProject {
            expected: p.id(),
            found: other.id()
        })
    );
    assert_eq!(p, before);
}

#[test]
fn removing_unknown_connection_is_a_structured_error_without_mutation() {
    let (mut p, ids) = fixture(2);
    connect(&mut p, ids[0], ids[1]);
    let before = p.clone();
    let missing = ConnectionId::new();
    assert_eq!(
        p.remove_connection(missing),
        Err(DomainError::MissingConnection(missing))
    );
    assert_eq!(p, before);
}
