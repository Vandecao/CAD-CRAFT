// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_craft::interchange::{export_parts_csv, import_parts_csv};
use cad_craft::persistence;

#[test]
fn contactor_part_validates_function_model_and_round_trips() {
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
        .unwrap()
        .with_functions(vec![
            PartFunction::new(FunctionKind::Coil, FunctionRole::Main, ["A1", "A2"]).unwrap(),
            PartFunction::new(
                FunctionKind::MainContact,
                FunctionRole::Auxiliary,
                ["1", "2"],
            )
            .unwrap(),
            PartFunction::new(
                FunctionKind::MainContact,
                FunctionRole::Auxiliary,
                ["3", "4"],
            )
            .unwrap(),
            PartFunction::new(
                FunctionKind::MainContact,
                FunctionRole::Auxiliary,
                ["5", "6"],
            )
            .unwrap(),
            PartFunction::new(
                FunctionKind::AuxiliaryContact,
                FunctionRole::Auxiliary,
                ["13", "14"],
            )
            .unwrap(),
        ])
        .unwrap();
    assert_eq!(part.functions().len(), 5);
    let restored: Part = serde_json::from_str(&serde_json::to_string(&part).unwrap()).unwrap();
    assert_eq!(restored, part);
    assert!(PartFunction::new(FunctionKind::Coil, FunctionRole::Main, ["A1", "A1"]).is_err());
    assert!(Part::new(" ", "ACME", "K1", "Contator", "Contator").is_err());
}

#[test]
fn assigning_part_creates_unrepresented_functions_and_replays_exactly() {
    let mut project = Project::new("P").unwrap();
    let device = Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap();
    let device_id = project.insert_device(device).unwrap();
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
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
    let before = serde_json::to_string(&project).unwrap();
    let delta = project.assign_part(device_id, part.clone(), 2).unwrap();
    assert_eq!(project.parts().count(), 1);
    assert_eq!(
        project
            .components()
            .filter(|c| c.device_id() == Some(device_id))
            .count(),
        1
    );
    assert_eq!(
        project
            .functions()
            .filter(|f| f.device_id() == device_id)
            .count(),
        2
    );
    assert_eq!(project.connection_points().count(), 4);
    assert_eq!(project.representations().count(), 0);
    assert_eq!(project.bom_rows()[0].quantity, 2);
    let after = serde_json::to_string(&project).unwrap();
    project.restore_part_assignment(&delta, false).unwrap();
    assert_eq!(serde_json::to_string(&project).unwrap(), before);
    project.restore_part_assignment(&delta, true).unwrap();
    assert_eq!(serde_json::to_string(&project).unwrap(), after);
    assert!(project.assign_part(device_id, part, 0).is_err());
}

#[test]
fn part_update_preserves_assignment_and_removal_is_reversible() {
    let mut project = Project::new("P").unwrap();
    let device = Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap();
    let device_id = project.insert_device(device).unwrap();
    let mut initial_properties = TechnicalProperties::default();
    initial_properties
        .set_known(KnownProperty::RatedVoltage, 24.0)
        .unwrap();
    let original = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
        .unwrap()
        .with_properties(initial_properties);
    let assigned = project.assign_part(device_id, original.clone(), 2).unwrap();
    assert_eq!(
        project
            .component(assigned.component_id())
            .unwrap()
            .properties(),
        original.properties()
    );
    let mut next_properties = TechnicalProperties::default();
    next_properties
        .set_known(KnownProperty::RatedVoltage, 48.0)
        .unwrap();
    let updated = Part::new("ACME.K1", "ACME", "K1", "Contator novo", "Contator")
        .unwrap()
        .with_properties(next_properties);
    let update = project.update_part(updated.clone()).unwrap();
    assert_eq!(project.part(updated.id()), Some(&updated));
    assert_eq!(
        project
            .component(assigned.component_id())
            .unwrap()
            .properties(),
        updated.properties()
    );
    assert_eq!(project.bom_rows()[0].quantity, 2);
    project.restore_part_update(&update, false).unwrap();
    assert_eq!(project.part(original.id()), Some(&original));
    project.restore_part_update(&update, true).unwrap();
    let removal = project
        .remove_part_assignment(assigned.component_id())
        .unwrap();
    assert!(project.bom_rows().is_empty());
    project.restore_part_removal(&removal, false).unwrap();
    assert_eq!(project.bom_rows()[0].quantity, 2);
}

#[test]
fn cadpart_and_project_round_trip_keeps_v6_fixture() {
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator").unwrap();
    let package = persistence::write_part_package(&part).unwrap();
    assert_eq!(persistence::read_part_package(&package).unwrap(), part);
    assert!(persistence::read_part_package(b"not zip").is_err());
    let mut project = Project::new("P").unwrap();
    let device = Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap();
    let device = project.insert_device(device).unwrap();
    project.assign_part(device, part, 1).unwrap();
    let bytes = persistence::write_project_to_vec(&project).unwrap();
    let loaded = persistence::read_project_from_slice(&bytes).unwrap();
    assert_eq!(loaded.source_format, persistence::FORMAT);
    assert_eq!(loaded.project, project);
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/etapa18-v6.cadcraft"
    );
    if std::env::var_os("CAD_CRAFT_WRITE_FIXTURES").is_some() {
        std::fs::write(fixture, &bytes).unwrap();
    }
    let fixed = persistence::read_project_from_slice(&std::fs::read(fixture).unwrap()).unwrap();
    assert_eq!(fixed.source_format, 6);
    assert_eq!(fixed.project.parts().count(), 1);
}

#[test]
fn csv_import_reports_bad_rows_and_round_trips_one_hundred_parts() {
    let parts: Vec<_> = (0..100)
        .map(|i| {
            Part::new(
                format!("ACME.P{i}"),
                "ACME",
                format!("P{i}"),
                "Peça",
                "Elétrica",
            )
            .unwrap()
        })
        .collect();
    let csv = export_parts_csv(&parts).unwrap();
    let mut invalid = csv.clone();
    invalid.extend_from_slice(b"\nBAD,,X,broken,Class,,,\n");
    let report = import_parts_csv(&invalid).unwrap();
    assert_eq!(report.parts, parts);
    assert_eq!(report.rejected.len(), 1);
    let round_trip = import_parts_csv(&csv).unwrap();
    assert_eq!(round_trip.parts, parts);
}

#[test]
fn bom_sums_quantities_and_exports_csv() {
    let mut project = Project::new("P").unwrap();
    let device = Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap();
    let device = project.insert_device(device).unwrap();
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator").unwrap();
    project.assign_part(device, part.clone(), 2).unwrap();
    project.assign_part(device, part, 3).unwrap();
    let rows = project.bom_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].quantity, 5);
    let csv = cad_craft::interchange::export_bom_csv(&rows).unwrap();
    let text = String::from_utf8(csv).unwrap();
    assert!(text.contains("-K1,ACME.K1"));
    assert!(text.contains("ACME.K1,ACME,K1,5,Contator"));
}

#[test]
fn assignment_completes_points_of_matching_existing_function() {
    let mut project = Project::new("P").unwrap();
    let designation = Designation::new("K1").unwrap();
    let device = Device::new(project.id(), designation.clone(), "Contator").unwrap();
    let device = project.insert_device(device).unwrap();
    let function = Function::new(project.id(), device, designation, FunctionKind::Coil)
        .with_role(FunctionRole::Main);
    let function_id = project.insert_function(function).unwrap();
    project
        .insert_connection_point(ConnectionPoint::new(
            project.id(),
            function_id,
            DesignationPart::new("A1").unwrap(),
        ))
        .unwrap();
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
        .unwrap()
        .with_functions(vec![
            PartFunction::new(FunctionKind::Coil, FunctionRole::Main, ["A1", "A2"]).unwrap(),
        ])
        .unwrap();
    project.assign_part(device, part, 1).unwrap();
    assert_eq!(project.functions_for_device(device).count(), 1);
    assert_eq!(
        project
            .connection_points()
            .filter(|p| p.function_id() == function_id)
            .count(),
        2
    );
}

#[test]
fn updating_library_parts_is_atomic_and_one_reversible_batch() {
    let mut project = Project::new("P").unwrap();
    let device = project
        .insert_device(
            Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    let a = Part::new("A", "ACME", "A", "Antiga A", "Peça").unwrap();
    let b = Part::new("B", "ACME", "B", "Antiga B", "Peça").unwrap();
    project.assign_part(device, a.clone(), 1).unwrap();
    project.assign_part(device, b.clone(), 1).unwrap();
    let before = project.clone();
    let a_new = Part::new("A", "ACME", "A", "Nova A", "Peça").unwrap();
    let b_new = Part::new("B", "ACME", "B", "Nova B", "Peça").unwrap();
    let missing = Part::new("C", "ACME", "C", "Nova C", "Peça").unwrap();
    assert!(project.update_parts(vec![a_new.clone(), missing]).is_err());
    assert_eq!(project, before);
    let changes = project
        .update_parts(vec![a_new.clone(), b_new.clone()])
        .unwrap();
    assert_eq!(changes.len(), 2);
    project.restore_part_updates(&changes, false).unwrap();
    assert_eq!(project, before);
    project.restore_part_updates(&changes, true).unwrap();
    assert_eq!(project.part(a_new.id()), Some(&a_new));
    assert_eq!(project.part(b_new.id()), Some(&b_new));
}

#[test]
fn function_and_point_editor_replay_and_dependency_refusal() {
    let mut project = Project::new("P").unwrap();
    let device = Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap();
    let device = project.insert_device(device).unwrap();
    let before = project.clone();
    let created = project
        .edit_function_point(FunctionPointEdit::CreateFunction {
            device,
            designation: Designation::new("K1").unwrap(),
            kind: FunctionKind::Coil,
            role: FunctionRole::Main,
        })
        .unwrap();
    let function = project.functions().next().unwrap().id();
    let point_delta = project
        .edit_function_point(FunctionPointEdit::CreatePoint {
            function,
            designation: DesignationPart::new("A1").unwrap(),
        })
        .unwrap();
    let point = project.connection_points().next().unwrap().id();
    let rename = project
        .edit_function_point(FunctionPointEdit::RenamePoint {
            point,
            designation: DesignationPart::new("A2").unwrap(),
        })
        .unwrap();
    assert_eq!(
        project
            .connection_point(point)
            .unwrap()
            .designation()
            .as_str(),
        "A2"
    );
    project.restore_function_point(&rename, false).unwrap();
    project.restore_function_point(&point_delta, false).unwrap();
    project.restore_function_point(&created, false).unwrap();
    assert_eq!(project, before);
    project.restore_function_point(&created, true).unwrap();
    project.restore_function_point(&point_delta, true).unwrap();
    project.restore_function_point(&rename, true).unwrap();
    assert!(
        project
            .edit_function_point(FunctionPointEdit::RemoveFunction { function })
            .is_ok()
    );
}

#[test]
fn unrepresented_function_uses_existing_points_when_symbol_is_placed() {
    let mut project = Project::new("P").unwrap();
    let document = project
        .insert_document(Document::new(project.id(), "D").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                document,
                "P",
                "1",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let device = Device::new(project.id(), Designation::new("K1").unwrap(), "Contator").unwrap();
    let device = project.insert_device(device).unwrap();
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator")
        .unwrap()
        .with_functions(vec![
            PartFunction::new(FunctionKind::Coil, FunctionRole::Main, ["A1", "A2"]).unwrap(),
        ])
        .unwrap();
    project.assign_part(device, part, 1).unwrap();
    let function = project.functions().next().unwrap().id();
    let points: Vec<_> = project
        .connection_points()
        .map(ConnectionPoint::id)
        .collect();
    let symbol = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.coil").unwrap())
        .unwrap()
        .clone();
    let placement = ExistingFunctionPlacement {
        page,
        definition: symbol,
        transform: Transform2D::new(Point2D::new(100.0, 100.0).unwrap(), 0.0, false, false)
            .unwrap(),
        function,
        allow_order_fallback: true,
    };
    let delta = project.place_existing_function(placement.clone()).unwrap();
    assert_eq!(project.connection_points().count(), 2);
    assert_eq!(project.function_references(function).unwrap().len(), 1);
    assert_eq!(delta.point_ids().len(), 2);
    assert!(delta.point_ids().iter().all(|id| points.contains(id)));
    assert!(project.place_existing_function(placement).is_err());
    assert!(
        project
            .edit_function_point(FunctionPointEdit::RemoveFunction { function })
            .is_err()
    );
    assert!(
        project
            .edit_function_point(FunctionPointEdit::RemovePoint { point: points[0] })
            .is_err()
    );
    let changed_model = Part::new("ACME.K1", "ACME", "K1", "Contator revisto", "Contator").unwrap();
    assert!(project.update_part(changed_model).is_err());
    project
        .restore_existing_function_placement(&delta, false)
        .unwrap();
    assert!(project.function_references(function).unwrap().is_empty());
    project
        .restore_existing_function_placement(&delta, true)
        .unwrap();
}
