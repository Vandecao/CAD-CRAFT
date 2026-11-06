// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Consistency checks, connector continuity, documentary disconnects and configurable rule
//! schemes.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

mod output_common;
mod physical_common;

use cad_craft::domain::*;
use cad_craft::persistence;
use output_common::*;
use physical_common::part;
use std::collections::BTreeSet;
use std::time::Instant;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn codes(project: &Project) -> BTreeSet<&'static str> {
    project
        .check(CheckScope::Project)
        .messages
        .iter()
        .map(|m| m.rule)
        .collect()
}
fn messages(project: &Project, rule: &str) -> Vec<CheckMessage> {
    project
        .check(CheckScope::Project)
        .messages
        .into_iter()
        .filter(|m| m.rule == rule)
        .collect()
}

fn base() -> (Project, DocumentId, PageId) {
    let mut p = Project::new("Verificação").unwrap();
    let doc = p
        .insert_document(Document::new(p.id(), "Esquema").unwrap())
        .unwrap();
    let page = p
        .insert_page(
            Page::new(
                p.id(),
                doc,
                "Força",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    (p, doc, page)
}

/// Create a device with one function and n points, without a representation.
fn device(
    p: &mut Project,
    name: &str,
    role: FunctionRole,
    n: usize,
) -> (DeviceId, Vec<ConnectionPointId>) {
    let device = p
        .insert_device(Device::new(p.id(), Designation::new(name).unwrap(), "teste").unwrap())
        .unwrap();
    let function = p
        .insert_function(
            Function::new(
                p.id(),
                device,
                Designation::new(name).unwrap(),
                FunctionKind::AuxiliaryContact,
            )
            .with_role(role),
        )
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

fn reload(value: serde_json::Value) -> Project {
    serde_json::from_value(value).unwrap()
}

/// Duplicate device/page designations are detected; equal connector names never join networks.
#[test]
fn designation_page_and_connector_duplicates_are_reported() {
    let (mut p, doc, _) = base();
    device(&mut p, "K1", FunctionRole::Main, 0);
    device(&mut p, "K2", FunctionRole::Main, 0);
    p.insert_page(
        Page::new(
            p.id(),
            doc,
            "Comando",
            "2",
            PageKind::MultiLine,
            PaperFormat::A3.size(true),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!codes(&p).contains("VER-001"));
    assert!(!codes(&p).contains("VER-003"));
    let mut value = serde_json::to_value(&p).unwrap();
    value["devices"][1]["designation"] = value["devices"][0]["designation"].clone();
    value["pages"][1]["number"] = value["pages"][0]["number"].clone();
    let old = reload(value);
    let duplicate = messages(&old, "VER-001");
    assert_eq!(duplicate.len(), 1);
    assert_eq!(duplicate[0].severity, CheckSeverity::Error);
    assert_eq!(duplicate[0].targets.len(), 2);
    let pages = messages(&old, "VER-003");
    assert_eq!(pages.len(), 1);
    assert!(
        pages[0]
            .targets
            .iter()
            .all(|t| matches!(t, CheckTarget::Page(_)))
    );

    // Two connectors named -X1 have no continuity merely because their names match.
    let (a, pa) = device(&mut p, "X1", FunctionRole::Main, 1);
    let (b, pb) = device(&mut p, "X9", FunctionRole::Main, 1);
    let mut edits = vec![];
    let mut contacts = vec![];
    for owner in [a, b] {
        let t = Terminal::new(p.id(), TerminalOwner::Device(owner), part("1"))
            .with_positions(vec![part("c")])
            .unwrap();
        contacts.push(t.id());
        edits.push(PhysicalEdit::UpsertTerminal(t.clone()));
        edits.push(PhysicalEdit::UpsertConnector(
            Connector::new(p.id(), part("X1"))
                .with_device(Some(owner))
                .with_contacts(vec![t.id()]),
        ));
    }
    p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
    for (point, contact) in [(pa[0], contacts[0]), (pb[0], contacts[1])] {
        p.edit_physical(PhysicalEdit::SetPointTerminal {
            point,
            terminal: Some(contact),
            level: None,
        })
        .unwrap();
    }
    let con = messages(&p, "VER-002");
    assert_eq!(con.len(), 1);
    assert_eq!(con[0].severity, CheckSeverity::Warning);
    assert!(
        !p.are_connected(pa[0], pb[0]).unwrap(),
        "nomes iguais nunca unem redes"
    );
}

/// Check auxiliaries without a principal, unrepresented part functions and missing parts.
#[test]
fn function_and_part_rules() {
    let (mut p, _, page) = base();
    let (aux, _) = device(&mut p, "K5", FunctionRole::Auxiliary, 0);
    let codes_now = codes(&p);
    assert!(codes_now.contains("VER-004"));
    assert!(codes_now.contains("VER-018"));
    let main = p
        .insert_function(
            Function::new(
                p.id(),
                aux,
                Designation::new("K5").unwrap(),
                FunctionKind::Coil,
            )
            .with_role(FunctionRole::Main),
        )
        .unwrap();
    let _ = main;
    assert!(!codes(&p).contains("VER-004"));
    // The two unrepresented part functions produce informational diagnostics.
    let part_value = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
        .unwrap()
        .with_functions(vec![
            PartFunction::new(FunctionKind::Coil, FunctionRole::Main, ["A1", "A2"]).unwrap(),
            PartFunction::new(
                FunctionKind::AuxiliaryContact,
                FunctionRole::Auxiliary,
                ["13", "14"],
            )
            .unwrap(),
        ])
        .unwrap();
    let k1 = place(
        &mut p,
        page,
        coil(),
        new_device("K1"),
        FunctionKind::Coil,
        (100.0, 100.0),
    );
    p.assign_part(k1.device_id(), part_value, 1).unwrap();
    let unrepresented = messages(&p, "VER-006");
    assert!(!unrepresented.is_empty());
    assert!(
        unrepresented
            .iter()
            .all(|m| m.severity == CheckSeverity::Info)
    );
    assert!(
        !messages(&p, "VER-018")
            .iter()
            .any(|m| m.targets.contains(&CheckTarget::Device(k1.device_id())))
    );
}

/// Check unconnected ports, missing assigned wire numbers and conflicting network potentials.
#[test]
fn connection_rules() {
    let (mut p, _, page) = base();
    let k1 = place(
        &mut p,
        page,
        coil(),
        new_device("K1"),
        FunctionKind::Coil,
        (100.0, 150.0),
    );
    let open = messages(&p, "VER-007");
    assert_eq!(open.len(), 2, "duas portas da bobina sem fio");
    assert!(open.iter().all(|m| {
        m.targets
            .contains(&CheckTarget::Representation(k1.representation_id()))
    }));
    let k2 = place(
        &mut p,
        page,
        coil(),
        new_device("K2"),
        FunctionKind::Coil,
        (100.0, 100.0),
    );
    p.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
        source: PortRef::new(k1.representation_id(), "1").unwrap(),
        target: PortRef::new(k2.representation_id(), "2").unwrap(),
        points: vec![p_(100.0, 140.0), p_(100.0, 110.0)],
    })
    .unwrap();
    assert_eq!(
        messages(&p, "VER-007").len(),
        2,
        "sobram as portas externas"
    );
    // Number only one of two wires in separate networks.
    let (_, pts) = device(&mut p, "K9", FunctionRole::Main, 4);
    let a = p
        .insert_connection(Connection::new(p.id(), pts[0], pts[1]).unwrap())
        .unwrap();
    let b = p
        .insert_connection(Connection::new(p.id(), pts[2], pts[3]).unwrap())
        .unwrap();
    assert!(!codes(&p).contains("VER-008"));
    p.set_connection_number(a, Some(Text::new("1").unwrap()), false)
        .unwrap();
    let unnumbered = messages(&p, "VER-008");
    assert!(
        unnumbered
            .iter()
            .any(|m| m.targets.contains(&CheckTarget::Connection(b)))
    );
    let preview = p.preview_connection_numbering().unwrap();
    p.apply_connection_numbering(&preview).unwrap();
    assert!(!codes(&p).contains("VER-008"));
    // Two different potentials belong to the same network.
    for (name, point) in [("L1", pts[0]), ("L2", pts[1])] {
        let def = SymbolDefinition::new(
            SymbolDefinitionId::new("teste.potencial").unwrap(),
            1,
            "Potencial",
            "Referências",
            vec![SymbolPrimitive::Line {
                start: p_(0., 0.),
                end: p_(5., 0.),
            }],
            vec![SymbolPort::new("p", p_(0., 0.), 0., "P", None).unwrap()],
        )
        .unwrap();
        let r = Representation::new_symbol(
            p.id(),
            page,
            &def,
            Some(RepresentationTarget::ConnectionPoint(point)),
            at(300.0, 100.0),
        )
        .unwrap();
        let id = r.id();
        p.edit_marker(MarkerEdit::Insert {
            marker: ElectricalMarker::new(
                id,
                Text::new(name).unwrap(),
                MarkerKind::Potential {
                    kind: None,
                    description: String::new(),
                },
                Some(point),
            ),
            representation: Box::new(r),
            definition: Box::new(def),
        })
        .unwrap();
    }
    let conflict = messages(&p, "VER-009");
    assert_eq!(conflict.len(), 1);
    assert!(conflict[0].text.contains("L1") && conflict[0].text.contains("L2"));
}
fn p_(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}

/// Grouping resolves an unpaired interruption; a potential without a point remains invalid.
#[test]
fn marker_rules() {
    let f = fixture("Marcadores");
    assert!(
        !codes(&f.project).contains("VER-010"),
        "o cenário comum agrupa as interrupções"
    );
    let (mut p, _, page) = base();
    let lonely = marker(&mut p, page, "X5");
    let single = messages(&p, "VER-010");
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].targets, vec![CheckTarget::Representation(lonely)]);
    assert_eq!(
        p.check_target_location(single[0].targets[0]),
        Some((page, Some(lonely)))
    );
    let def = SymbolDefinition::new(
        SymbolDefinitionId::new("teste.potencial2").unwrap(),
        1,
        "Potencial",
        "Referências",
        vec![SymbolPrimitive::Line {
            start: p_(0., 0.),
            end: p_(5., 0.),
        }],
        vec![SymbolPort::new("p", p_(0., 0.), 0., "P", None).unwrap()],
    )
    .unwrap();
    let r = Representation::new_symbol(p.id(), page, &def, None, at(50.0, 50.0)).unwrap();
    let id = r.id();
    p.edit_marker(MarkerEdit::Insert {
        marker: ElectricalMarker::new(
            id,
            Text::new("N").unwrap(),
            MarkerKind::Potential {
                kind: None,
                description: String::new(),
            },
            None,
        ),
        representation: Box::new(r),
        definition: Box::new(def),
    })
    .unwrap();
    assert_eq!(messages(&p, "VER-011").len(), 1);
}

/// Build male X1 and female X2 with two contacts and associated points; return both.
#[allow(clippy::type_complexity)]
fn connectors(
    p: &mut Project,
) -> (
    (ConnectorId, Vec<TerminalId>, Vec<ConnectionPointId>),
    (ConnectorId, Vec<TerminalId>, Vec<ConnectionPointId>),
) {
    let mut make = |name: &str, gender| {
        let (device, points) = device(p, name, FunctionRole::Main, 2);
        let mut edits = vec![];
        let mut contacts = vec![];
        for i in 1..=2 {
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
        p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
        (id, contacts, points)
    };
    let x1 = make("X1", ConnectorGender::Male);
    let x2 = make("X2", ConnectorGender::Female);
    for (point, contact) in x1.2.iter().zip(&x1.1).chain(x2.2.iter().zip(&x2.1)) {
        p.edit_physical(PhysicalEdit::SetPointTerminal {
            point: *point,
            terminal: Some(*contact),
            level: None,
        })
        .unwrap();
    }
    (x1, x2)
}

/// Mating joins networks through the explicit map; unmating separates them. Undo is exact and
/// old files gain owned connections.
#[test]
fn mating_gives_continuity_and_unmating_separates() {
    let (mut p, _, _) = base();
    let ((x1, c1, p1), (x2, c2, p2)) = connectors(&mut p);
    let before = p.clone();
    let mating = ConnectorMating::new(
        p.id(),
        x1,
        x2,
        vec![
            ContactPair::new(c1[0], c2[0]),
            ContactPair::new(c1[1], c2[1]),
        ],
    );
    let id = mating.id();
    let delta = p
        .edit_physical(PhysicalEdit::UpsertMating(mating.clone()))
        .unwrap();
    assert!(p.are_connected(p1[0], p2[0]).unwrap());
    assert!(p.are_connected(p1[1], p2[1]).unwrap());
    assert!(!p.are_connected(p1[0], p2[1]).unwrap());
    let owned = p.connector_mating(id).unwrap().owned_connections().to_vec();
    assert_eq!(owned.len(), 2);
    // Re-editing the same contact pairs preserves the wire number.
    p.set_connection_number(owned[0], Some(Text::new("7").unwrap()), false)
        .unwrap();
    let again = p.connector_mating(id).unwrap().clone();
    p.edit_physical(PhysicalEdit::UpsertMating(again)).unwrap();
    assert_eq!(
        p.connector_mating(id).unwrap().owned_connections(),
        owned.as_slice()
    );
    assert_eq!(
        p.connection(owned[0]).unwrap().number().unwrap().as_str(),
        "7"
    );
    // Changing a mated contact point is refused until it is unmated.
    let snapshot = p.clone();
    assert!(
        p.edit_physical(PhysicalEdit::SetPointTerminal {
            point: p1[0],
            terminal: None,
            level: None,
        })
        .is_err()
    );
    assert_eq!(p, snapshot);
    // Unmating separates the networks.
    p.edit_physical(PhysicalEdit::DeleteMating(id)).unwrap();
    assert!(!p.are_connected(p1[0], p2[0]).unwrap());
    assert!(owned.iter().all(|c| p.connection(*c).is_none()));
    // Undo the original mating.
    let mut q = before.clone();
    q.restore_physical(&delta, true).unwrap();
    assert!(q.are_connected(p1[0], p2[0]).unwrap());
    q.restore_physical(&delta, false).unwrap();
    assert_eq!(q, before);
    // Loading a v8 project without owned edges creates the owned connections.
    let mut legacy = before.clone();
    legacy
        .edit_physical(PhysicalEdit::UpsertMating(mating))
        .unwrap();
    let mut value = serde_json::to_value(&legacy).unwrap();
    let owned_ids: Vec<String> = value["connector_matings"][0]["owned"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    value["connector_matings"][0]
        .as_object_mut()
        .unwrap()
        .remove("owned");
    let connections = value["connections"].as_array_mut().unwrap();
    connections.retain(|c| !owned_ids.contains(&c["id"].as_str().unwrap().to_owned()));
    let migrated = reload(value);
    assert!(migrated.are_connected(p1[0], p2[0]).unwrap());
    assert_eq!(
        migrated
            .connector_mating(id)
            .unwrap()
            .owned_connections()
            .len(),
        2
    );
}

/// Check unterminated terminals, documentary open disconnects and mated contacts without points.
#[test]
fn physical_rules() {
    let (mut p, _, _) = base();
    let (owner, pts) = device(&mut p, "X5", FunctionRole::Main, 1);
    let open = Terminal::new(p.id(), TerminalOwner::Device(owner), part("1"))
        .with_positions(vec![part("A"), part("B")])
        .unwrap()
        .with_kind(TerminalKind::Disconnect)
        .with_sides(
            [(part("A"), TerminalSide::A), (part("B"), TerminalSide::B)]
                .into_iter()
                .collect(),
        )
        .with_disconnect_state(Some(DisconnectState::Open));
    let open_id = open.id();
    let strip = TerminalStrip::new(p.id(), part("X5")).with_terminals(vec![open_id]);
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(open),
        PhysicalEdit::UpsertStrip(strip),
    ]))
    .unwrap();
    let networks = p.networks();
    let info = messages(&p, "VER-016");
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].severity, CheckSeverity::Info);
    assert_eq!(p.networks(), networks, "estado documental: rede inalterada");
    assert!(
        messages(&p, "VER-014")
            .iter()
            .any(|m| m.targets == vec![CheckTarget::Terminal(open_id)])
    );
    p.edit_physical(PhysicalEdit::SetPointTerminal {
        point: pts[0],
        terminal: Some(open_id),
        level: None,
    })
    .unwrap();
    let other = p
        .insert_connection_point(ConnectionPoint::new(
            p.id(),
            p.connection_point(pts[0]).unwrap().function_id(),
            part("9"),
        ))
        .unwrap();
    p.insert_connection(Connection::new(p.id(), pts[0], other).unwrap())
        .unwrap();
    assert!(
        !messages(&p, "VER-014")
            .iter()
            .any(|m| m.targets == vec![CheckTarget::Terminal(open_id)])
    );
    // A mated contact without a point produces a warning and no continuity.
    let ((x1, c1, _), (x2, c2, p2)) = connectors(&mut p);
    let x1_device = p.connector(x1).unwrap().device().unwrap();
    let unassociated = Terminal::new(p.id(), TerminalOwner::Device(x1_device), part("3"))
        .with_positions(vec![part("c")])
        .unwrap();
    let loose = unassociated.id();
    let mut connector = p.connector(x1).unwrap().clone();
    let mut contacts = connector.contacts().to_vec();
    contacts.push(loose);
    connector = connector.with_contacts(contacts);
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(unassociated),
        PhysicalEdit::UpsertConnector(connector),
    ]))
    .unwrap();
    p.edit_physical(PhysicalEdit::UpsertMating(ConnectorMating::new(
        p.id(),
        x1,
        x2,
        vec![
            ContactPair::new(c1[0], c2[0]),
            ContactPair::new(loose, c2[1]),
        ],
    )))
    .unwrap();
    let loose_msgs = messages(&p, "VER-017");
    assert_eq!(loose_msgs.len(), 1);
    assert_eq!(loose_msgs[0].targets, vec![CheckTarget::Terminal(loose)]);
    assert!(!p.are_connected(p2[1], p2[0]).unwrap());
    let _ = c1[1];
}

/// A stale report produces VER-020.
#[test]
fn stale_report_is_reported() {
    let f = fixture("Relatório");
    let mut p = f.project;
    let doc = p.documents().next().unwrap().id();
    p.generate_report(&ReportRequest::new(
        ReportKind::Devices,
        doc,
        PaperFormat::A3.size(true),
    ))
    .unwrap();
    assert!(!codes(&p).contains("VER-020"));
    for i in 0..60 {
        p.insert_device(
            Device::new(
                p.id(),
                Designation::new(format!("Q{i}")).unwrap(),
                "Disjuntor",
            )
            .unwrap(),
        )
        .unwrap();
    }
    let stale = messages(&p, "VER-020");
    assert_eq!(stale.len(), 1);
    assert!(matches!(stale[0].targets[0], CheckTarget::Page(_)));
}

/// Disable rules, change severity, ignore with a reason, undo, then save/reopen the v9 fixture.
#[test]
fn scheme_disables_changes_severity_and_ignores_with_undo_and_persistence() {
    let (mut p, _, page) = base();
    place(
        &mut p,
        page,
        coil(),
        new_device("K1"),
        FunctionKind::Coil,
        (100.0, 150.0),
    );
    let open = messages(&p, "VER-007");
    assert_eq!(open.len(), 2);
    let original = p.check_scheme().clone();
    let mut scheme = original.clone();
    scheme.rules.insert(
        "VER-018".into(),
        RuleSetting {
            enabled: false,
            severity: CheckSeverity::Info,
        },
    );
    scheme.rules.insert(
        "VER-007".into(),
        RuleSetting {
            enabled: true,
            severity: CheckSeverity::Error,
        },
    );
    scheme.ignored.push(IgnoredCheck {
        key: open[0].key(),
        reason: Text::new("Porta reservada").unwrap(),
    });
    let delta = p.set_check_scheme(scheme.clone()).unwrap();
    let report = p.check(CheckScope::Project);
    assert!(!report.messages.iter().any(|m| m.rule == "VER-018"));
    let still: Vec<_> = report
        .messages
        .iter()
        .filter(|m| m.rule == "VER-007")
        .collect();
    assert_eq!(still.len(), 1);
    assert_eq!(still[0].severity, CheckSeverity::Error);
    assert_eq!(report.ignored, 1);
    assert_eq!(report.count(CheckSeverity::Error), 1);
    // Page-scoped diagnostics belong to the page containing the occurrence.
    assert_eq!(p.check(CheckScope::Page(page)).messages.len(), 1);
    // Unknown rules are refused.
    let mut bad = scheme.clone();
    bad.rules.insert(
        "VER-999".into(),
        RuleSetting {
            enabled: true,
            severity: CheckSeverity::Error,
        },
    );
    assert!(p.set_check_scheme(bad).is_err());
    // Persist in native format 9.
    let bytes = persistence::write_project_to_vec(&p).unwrap();
    let fixture = format!("{FIXTURES}/etapa24-v9.cadcraft");
    if std::env::var_os("CAD_CRAFT_WRITE_ETAPA24_FIXTURE").is_some() {
        std::fs::write(&fixture, &bytes).unwrap();
    }
    let loaded = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(loaded.project.check_scheme(), &scheme);
    let stored = persistence::read_project_from_slice(&std::fs::read(&fixture).unwrap()).unwrap();
    assert_eq!(stored.source_format, 9);
    assert_eq!(stored.manifest.unwrap().min_reader, 9);
    assert_eq!(stored.project.check_scheme().ignored.len(), 1);
    let v8 = persistence::read_project_from_slice(
        &std::fs::read(format!("{FIXTURES}/etapa23-v8.cadcraft")).unwrap(),
    )
    .unwrap();
    assert_eq!(v8.source_format, 8);
    assert!(v8.migrated());
    // Undo restores the rule scheme.
    p.restore_check_scheme(&delta, false).unwrap();
    assert_eq!(p.check_scheme(), &original);
    p.restore_check_scheme(&delta, true).unwrap();
    assert_eq!(p.check_scheme(), &scheme);
}

/// Scale checks cover 10,000 occurrences across 334 pages with coils and wires.
#[test]
fn ten_thousand_occurrences_are_checked_within_budget() {
    let project = scale_project(334);
    assert!(project.representations().count() >= 10_000);
    let start = Instant::now();
    let report = project.check(CheckScope::Project);
    let elapsed = start.elapsed();
    eprintln!(
        "Verificação: {} ocorrências, {} mensagens, {elapsed:?}",
        project.representations().count(),
        report.messages.len()
    );
    let limit = if cfg!(debug_assertions) { 60.0 } else { 2.0 };
    assert!(elapsed.as_secs_f64() < limit, "{elapsed:?}");
}
