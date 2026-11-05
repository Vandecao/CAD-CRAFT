// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod document_common;
mod physical_common;
use cad_craft::domain::*;

#[test]
fn new_device_cannot_reuse_complete_designation_but_structure_distinguishes_it() {
    let (mut p, _, _, _, _) = document_common::fixture();
    let before = p.clone();
    assert!(
        p.insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "Other").unwrap())
            .is_err()
    );
    assert_eq!(p, before);
    let tag = Designation::new("K1")
        .unwrap()
        .with_location_path(vec![DesignationPart::new("P2").unwrap()]);
    assert!(
        p.insert_device(Device::new(p.id(), tag, "Other").unwrap())
            .is_ok()
    );
}

fn functions(p: &mut Project, d: DeviceId) -> (FunctionId, FunctionId) {
    let coil = p.functions_for_device(d).next().unwrap().id();
    let contact = p
        .insert_function(Function::new(
            p.id(),
            d,
            Designation::new("CONTACT").unwrap(),
            FunctionKind::AuxiliaryContact,
        ))
        .unwrap();
    for label in ["13", "14"] {
        p.insert_connection_point(ConnectionPoint::new(
            p.id(),
            contact,
            DesignationPart::new(label).unwrap(),
        ))
        .unwrap();
    }
    (coil, contact)
}

#[test]
fn rename_is_structured_atomic_reversible_and_keeps_electrical_and_physical_ids() {
    let mut p = physical_common::populated();
    let d = p.devices().next().unwrap().id();
    let before = p.clone();
    let networks = p.networks();
    let tag = Designation::new("K8")
        .unwrap()
        .with_functional_path(vec![DesignationPart::new("BOMBEAMENTO").unwrap()])
        .with_location_path(vec![DesignationPart::new("PAINEL_01").unwrap()]);
    let delta = p
        .edit_device(DeviceEdit::Rename {
            device: d,
            designation: tag,
        })
        .unwrap();
    assert_eq!(
        p.device(d).unwrap().designation().to_string(),
        "=BOMBEAMENTO+PAINEL_01-K8"
    );
    assert_eq!(p.networks(), networks);
    assert_eq!(
        p.conductors().collect::<Vec<_>>(),
        before.conductors().collect::<Vec<_>>()
    );
    assert_eq!(delta.len(), 1);
    let after = p.clone();
    p.restore_device_edit(&delta, false).unwrap();
    assert_eq!(p, before);
    p.restore_device_edit(&delta, true).unwrap();
    assert_eq!(p, after);
    assert!(p.restore_device_edit(&delta, true).is_err());
    assert_eq!(p, after);
}

#[test]
fn duplicate_rename_rejects_and_noop_keeps_empty_delta() {
    let (mut p, _, _, d, _) = document_common::fixture();
    let other = p
        .insert_device(Device::new(p.id(), Designation::new("K2").unwrap(), "Contator").unwrap())
        .unwrap();
    let before = p.clone();
    assert!(
        p.edit_device(DeviceEdit::Rename {
            device: other,
            designation: Designation::new("K1").unwrap()
        })
        .is_err()
    );
    assert_eq!(p, before);
    assert!(
        p.edit_device(DeviceEdit::Rename {
            device: d,
            designation: Designation::new("K1").unwrap()
        })
        .unwrap()
        .is_empty()
    );
}

#[test]
fn main_function_is_explicit_unique_and_replay_restores_roles() {
    let (mut p, _, _, d, _) = document_common::fixture();
    let (coil, contact) = functions(&mut p, d);
    assert_eq!(p.main_function(d).unwrap(), None);
    p.edit_device(DeviceEdit::SetMainFunction {
        device: d,
        function: Some(coil),
    })
    .unwrap();
    assert_eq!(p.main_function(d).unwrap().unwrap().id(), coil);
    let delta = p
        .edit_device(DeviceEdit::SetMainFunction {
            device: d,
            function: Some(contact),
        })
        .unwrap();
    assert_eq!(p.function(coil).unwrap().role(), FunctionRole::Auxiliary);
    assert_eq!(p.function(contact).unwrap().role(), FunctionRole::Main);
    assert_eq!(p.functions().count(), 2);
    p.restore_device_edit(&delta, false).unwrap();
    assert_eq!(p.function(coil).unwrap().role(), FunctionRole::Main);
    assert_eq!(
        p.function(contact).unwrap().role(),
        FunctionRole::Unspecified
    );
}

#[test]
fn functional_reassociation_changes_derived_references_without_connecting_points() {
    let (mut p, a, b, d, _) = document_common::fixture();
    let (coil, contact) = functions(&mut p, d);
    let other = p
        .insert_device(Device::new(p.id(), Designation::new("K2").unwrap(), "Contator").unwrap())
        .unwrap();
    for (f, page) in [(coil, a), (contact, b)] {
        p.insert_representation(document_common::rep(
            &p,
            page,
            RepresentationTarget::Function(f),
        ))
        .unwrap();
    }
    let nets = p.networks();
    let delta = p
        .edit_device(DeviceEdit::AssociateFunction {
            function: contact,
            device: other,
            role: FunctionRole::Auxiliary,
        })
        .unwrap();
    assert_eq!(p.device_references(d).unwrap().len(), 1);
    assert_eq!(p.device_references(other).unwrap().len(), 1);
    assert_eq!(p.connections().count(), 1);
    assert_eq!(
        p.networks()
            .iter()
            .map(|n| n.points().to_vec())
            .collect::<Vec<_>>(),
        nets.iter().map(|n| n.points().to_vec()).collect::<Vec<_>>()
    );
    p.restore_device_edit(&delta, false).unwrap();
    assert_eq!(p.device_references(d).unwrap().len(), 2);
}

#[test]
fn incompatible_reassociation_preserves_mapped_terminals_and_whole_project() {
    let (mut p, d, _, _) = physical_common::fixture(2);
    let f = p.functions_for_device(d).next().unwrap().id();
    let other = p
        .insert_device(Device::new(p.id(), Designation::new("Q1").unwrap(), "Other").unwrap())
        .unwrap();
    let before = p.clone();
    assert!(
        p.edit_device(DeviceEdit::AssociateFunction {
            function: f,
            device: other,
            role: FunctionRole::Main
        })
        .is_err()
    );
    assert_eq!(p, before);
}

#[test]
fn references_use_real_contact_labels_and_update_after_page_movement_rename_and_undo() {
    let (mut p, a, b, d, _) = document_common::fixture();
    let (coil, contact) = functions(&mut p, d);
    p.edit_device(DeviceEdit::SetMainFunction {
        device: d,
        function: Some(coil),
    })
    .unwrap();
    for (f, page) in [(coil, a), (contact, b)] {
        p.insert_representation(document_common::rep(
            &p,
            page,
            RepresentationTarget::Function(f),
        ))
        .unwrap();
    }
    let refs = p.device_references(d).unwrap();
    assert_eq!(refs.len(), 2);
    let destination = refs.iter().find(|r| r.function == Some(contact)).unwrap();
    assert_eq!(destination.terminals, vec!["13", "14"]);
    assert_eq!(destination.page_number, "2");
    assert_eq!(destination.position, Point2D::new(10., 20.).unwrap());
    let rid = destination.representation;
    let delta = p
        .edit_device(DeviceEdit::MoveRepresentation {
            representation: rid,
            page: a,
            transform: document_common::transform(44., 88.),
        })
        .unwrap();
    let destination = p.function_references(contact).unwrap().remove(0);
    assert_eq!(destination.page, a);
    assert_eq!(destination.position, Point2D::new(44., 88.).unwrap());
    p.edit_device(DeviceEdit::PageNumber {
        page: a,
        number: Text::new("7").unwrap(),
    })
    .unwrap();
    assert_eq!(p.function_references(contact).unwrap()[0].page_number, "7");
    p.restore_device_edit(&delta, false).unwrap();
    assert_eq!(p.function_references(contact).unwrap()[0].page, b);
    p.edit_device(DeviceEdit::Rename {
        device: d,
        designation: Designation::new("K9").unwrap(),
    })
    .unwrap();
    assert!(
        p.device_references(d)
            .unwrap()
            .iter()
            .all(|r| r.designation.to_string() == "-K9")
    );
    p.remove_representation(rid).unwrap();
    assert_eq!(p.device_references(d).unwrap().len(), 1);
}

#[test]
fn legacy_roles_default_and_duplicate_designations_are_diagnosed_without_merging() {
    let (mut p, _, _, d, _) = document_common::fixture();
    let other = p
        .insert_device(Device::new(p.id(), Designation::new("K2").unwrap(), "Contator").unwrap())
        .unwrap();
    let mut json = serde_json::to_value(&p).unwrap();
    for f in json["functions"].as_array_mut().unwrap() {
        f.as_object_mut().unwrap().remove("role");
    }
    json["devices"][1]["designation"] = json["devices"][0]["designation"].clone();
    let loaded: Project = serde_json::from_value(json).unwrap();
    assert_eq!(
        loaded.functions().next().unwrap().role(),
        FunctionRole::Unspecified
    );
    assert_eq!(
        loaded.designation_conflicts(),
        vec![vec![d.min(other), d.max(other)]]
    );
    assert_eq!(loaded.devices().count(), 2);
    assert_eq!(loaded.connections().count(), p.connections().count());
}

#[test]
fn snapshot_rejects_multiple_main_functions_and_preserves_valid_roles_and_references() {
    let (mut p, a, b, d, _) = document_common::fixture();
    let (coil, contact) = functions(&mut p, d);
    for (f, page) in [(coil, a), (contact, b)] {
        p.insert_representation(document_common::rep(
            &p,
            page,
            RepresentationTarget::Function(f),
        ))
        .unwrap();
    }
    p.edit_device(DeviceEdit::SetMainFunction {
        device: d,
        function: Some(coil),
    })
    .unwrap();
    let loaded: Project = serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    assert_eq!(
        loaded.device_references(d).unwrap(),
        p.device_references(d).unwrap()
    );
    let mut json = serde_json::to_value(&p).unwrap();
    for f in json["functions"].as_array_mut().unwrap() {
        f["role"] = serde_json::json!("Main");
    }
    assert!(serde_json::from_value::<Project>(json).is_err());
}

#[test]
fn document_edits_and_function_identification_replay_exactly() {
    let (mut p, a, _, d, _) = document_common::fixture();
    let f = p.functions_for_device(d).next().unwrap().id();
    let before = p.clone();
    let edits = [
        DeviceEdit::SetClassification {
            device: d,
            classification: Text::new("Relé configurado").unwrap(),
        },
        DeviceEdit::RenameFunction {
            function: f,
            designation: Designation::new("BOBINA").unwrap(),
        },
        DeviceEdit::PageName {
            page: a,
            name: Text::new("Comando").unwrap(),
        },
        DeviceEdit::PageStructure {
            page: a,
            structure: DocumentStructure::new(vec![DesignationPart::new("F").unwrap()], vec![]),
        },
    ];
    let deltas = edits
        .into_iter()
        .map(|e| p.edit_device(e).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        p.device(d).unwrap().classification().as_str(),
        "Relé configurado"
    );
    assert_eq!(p.function(f).unwrap().designation().to_string(), "-BOBINA");
    for delta in deltas.iter().rev() {
        p.restore_device_edit(delta, false).unwrap();
    }
    assert_eq!(p, before);
}
