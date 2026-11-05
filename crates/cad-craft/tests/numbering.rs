// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
mod document_common;
use cad_craft::domain::*;

#[test]
fn scheme_defaults_to_sequential_and_survives_snapshot() {
    let (project, _, _, _, _) = document_common::fixture();
    assert_eq!(
        project.numbering_scheme().device_format,
        DeviceNumberFormat::Sequential
    );
    assert_eq!(project.numbering_scheme().start, 1);
    assert_eq!(project.numbering_scheme().step, 1);
    let restored: Project =
        serde_json::from_value(serde_json::to_value(&project).unwrap()).unwrap();
    assert_eq!(restored.numbering_scheme(), project.numbering_scheme());
}

#[test]
fn connection_number_is_a_label_and_does_not_change_networks() {
    let (mut project, _, _, _, _) = document_common::fixture();
    let id = project.connections().next().unwrap().id();
    let before = project.networks();
    let delta = project
        .set_connection_number(id, Some(Text::new("42").unwrap()), true)
        .unwrap();
    assert_eq!(
        project.connection(id).unwrap().number().unwrap().as_str(),
        "42"
    );
    assert!(project.connection(id).unwrap().number_locked());
    assert_eq!(project.networks(), before);
    project.restore_connection_number(&delta, false).unwrap();
    assert_eq!(project.networks(), before);
    assert!(project.connection(id).unwrap().number().is_none());
    project.restore_connection_number(&delta, true).unwrap();
    assert_eq!(
        project.connection(id).unwrap().number().unwrap().as_str(),
        "42"
    );
}

#[test]
fn device_preview_applies_as_one_reversible_transaction() {
    let (mut project, first, second, first_device, _) = document_common::fixture();
    let second_device = project
        .insert_device(
            Device::new(project.id(), Designation::new("K9").unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    project
        .edit_device(DeviceEdit::Rename {
            device: first_device,
            designation: Designation::new("K5").unwrap(),
        })
        .unwrap();
    for (device, page) in [(first_device, first), (second_device, second)] {
        project
            .insert_representation(document_common::rep(
                &project,
                page,
                RepresentationTarget::Device(device),
            ))
            .unwrap();
    }
    let preview = project
        .preview_device_numbering(NumberingFilter::Project)
        .unwrap();
    assert_eq!(preview.rows.len(), 2);
    assert_eq!(preview.rows[0].after.to_string(), "-K1");
    assert_eq!(preview.rows[1].after.to_string(), "-K2");
    let before = project.clone();
    let delta = project.apply_device_numbering(&preview).unwrap();
    let after = project.clone();
    assert_eq!(
        project
            .device(first_device)
            .unwrap()
            .designation()
            .to_string(),
        "-K1"
    );
    project.restore_device_numbering(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_device_numbering(&delta, true).unwrap();
    assert_eq!(project, after);
}

#[test]
fn locked_device_collision_is_previewed_and_rejected() {
    let (mut project, _, _, first, _) = document_common::fixture();
    let second = project
        .insert_device(
            Device::new(project.id(), Designation::new("K2").unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    project.set_designation_locked(first, true).unwrap();
    let before = project.clone();
    let preview = project
        .preview_device_numbering(NumberingFilter::Project)
        .unwrap();
    assert!(preview.rows.iter().any(|r| r.device == second));
    assert!(preview.rows.iter().any(|r| r.conflict.is_some()));
    assert!(project.apply_device_numbering(&preview).is_err());
    assert_eq!(project, before);
}

#[test]
fn network_numbering_keeps_one_label_and_respects_locked_number() {
    let (mut project, _, _, _, first_connection) = document_common::fixture();
    let first_net = project
        .networks()
        .into_iter()
        .find(|n| n.connections().contains(&first_connection))
        .unwrap();
    let point = first_net.points()[0];
    let extra = project
        .insert_connection_point(ConnectionPoint::new(
            project.id(),
            first_net.functions()[0],
            DesignationPart::new("A3").unwrap(),
        ))
        .unwrap();
    let second_connection = project
        .insert_connection(Connection::new(project.id(), point, extra).unwrap())
        .unwrap();
    let preview = project.preview_connection_numbering().unwrap();
    assert_eq!(preview.rows.len(), 2);
    assert_eq!(preview.rows[0].after, preview.rows[1].after);
    let before = project.clone();
    let delta = project.apply_connection_numbering(&preview).unwrap();
    assert_eq!(
        project.connection(first_connection).unwrap().number(),
        project.connection(second_connection).unwrap().number()
    );
    project.restore_connection_numbering(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_connection_numbering(&delta, true).unwrap();
    project
        .set_connection_number(first_connection, Some(Text::new("L1").unwrap()), true)
        .unwrap();
    let locked = project.preview_connection_numbering().unwrap();
    assert!(
        locked
            .rows
            .iter()
            .all(|r| r.after.as_ref().unwrap().as_str() == "L1")
    );
}

#[test]
fn page_numbering_keeps_order_and_recomputes_reference() {
    let (mut project, first, second, device, _) = document_common::fixture();
    let document = project.page(first).unwrap().document_id();
    project.identify_page(first, "8").unwrap();
    project.identify_page(second, "9").unwrap();
    let representation = project
        .insert_representation(document_common::rep(
            &project,
            second,
            RepresentationTarget::Device(device),
        ))
        .unwrap();
    let old_reference = project.reference_label(
        second,
        project
            .representation(representation)
            .unwrap()
            .transform()
            .translation(),
    );
    let preview = project.preview_page_numbering(document, "P", 1).unwrap();
    assert_eq!(
        preview
            .rows
            .iter()
            .map(|r| r.after.as_str())
            .collect::<Vec<_>>(),
        vec!["P1", "P2"]
    );
    let before = project.clone();
    let delta = project.apply_page_numbering(&preview).unwrap();
    assert_eq!(
        project
            .ordered_pages(document)
            .unwrap()
            .iter()
            .map(|p| p.id())
            .collect::<Vec<_>>(),
        vec![first, second]
    );
    assert_ne!(
        project.reference_label(
            second,
            project
                .representation(representation)
                .unwrap()
                .transform()
                .translation()
        ),
        old_reference
    );
    project.restore_page_numbering(&delta, false).unwrap();
    assert_eq!(project, before);
}

#[test]
fn contact_mirror_is_derived_including_unrepresented_contacts() {
    let (mut project, first, second, device, _) = document_common::fixture();
    let main = project.functions_for_device(device).next().unwrap().id();
    project
        .edit_device(DeviceEdit::SetMainFunction {
            device,
            function: Some(main),
        })
        .unwrap();
    let contact = project
        .insert_function(
            Function::new(
                project.id(),
                device,
                Designation::new("K1").unwrap(),
                FunctionKind::AuxiliaryContact,
            )
            .with_role(FunctionRole::Auxiliary),
        )
        .unwrap();
    for label in ["13", "14"] {
        project
            .insert_connection_point(ConnectionPoint::new(
                project.id(),
                contact,
                DesignationPart::new(label).unwrap(),
            ))
            .unwrap();
    }
    let no_rep = project.contact_mirror(device).unwrap();
    assert_eq!(no_rep.len(), 1);
    assert_eq!(no_rep[0].terminals, vec!["13", "14"]);
    assert!(no_rep[0].references.is_empty());
    let rep = project
        .insert_representation(document_common::rep(
            &project,
            first,
            RepresentationTarget::Function(contact),
        ))
        .unwrap();
    let first_reference = project.contact_mirror(device).unwrap()[0].references[0]
        .reference
        .clone();
    project
        .edit_device(DeviceEdit::MoveRepresentation {
            representation: rep,
            page: second,
            transform: document_common::transform(30.0, 40.0),
        })
        .unwrap();
    let mirror = project.contact_mirror(device).unwrap();
    assert_eq!(mirror[0].references[0].representation, rep);
    assert_ne!(mirror[0].references[0].reference, first_reference);
}

#[test]
fn insertion_rule_can_use_page_and_column_without_reusing_names() {
    let (mut project, _, second, _, _) = document_common::fixture();
    let mut scheme = project.numbering_scheme().clone();
    scheme.device_format = DeviceNumberFormat::PageColumn;
    scheme.use_on_insert = true;
    project.set_numbering_scheme(scheme).unwrap();
    let point = Point2D::new(10.0, 20.0).unwrap();
    let first = project
        .next_device_designation_at("K", second, point)
        .unwrap();
    assert_eq!(first.to_string(), "-K2.1");
    project
        .insert_device(Device::new(project.id(), first, "Contator").unwrap())
        .unwrap();
    let next = project
        .next_device_designation_at("K", second, point)
        .unwrap();
    assert_ne!(next.to_string(), "-K2.1");
}

fn page(project: &mut Project, number: &str) -> PageId {
    let document = project.documents().next().unwrap().id();
    project
        .insert_page(
            Page::new(
                project.id(),
                document,
                format!("P{number}"),
                number,
                PageKind::MultiLine,
                Size2D::new(297.0, 210.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
}

fn placed(project: &mut Project, name: &str, page: PageId, x: f64, y: f64) -> DeviceId {
    let device = project
        .insert_device(
            Device::new(project.id(), Designation::new(name).unwrap(), "Contator").unwrap(),
        )
        .unwrap();
    project
        .insert_representation(Representation::new(
            project.id(),
            page,
            RepresentationTarget::Device(device),
            document_common::transform(x, y),
        ))
        .unwrap();
    device
}

fn names(project: &Project, ids: &[DeviceId]) -> Vec<String> {
    ids.iter()
        .map(|id| project.device(*id).unwrap().designation().to_string())
        .collect()
}

/// Critério 1 (Core): três páginas, -K5/-K2/-K9 fora de ordem; funções e
/// peças acompanham; desfazer/refazer exatos.
#[test]
fn three_pages_out_of_order_become_sequential_with_functions_and_parts() {
    let (mut project, first, second, _, _) = document_common::fixture();
    let third = page(&mut project, "3");
    let k9 = placed(&mut project, "K9", first, 50.0, 100.0);
    let k5 = placed(&mut project, "K5", second, 50.0, 100.0);
    let k2 = placed(&mut project, "K2", third, 50.0, 100.0);
    let contact = project
        .insert_function(
            Function::new(
                project.id(),
                k9,
                Designation::new("K9").unwrap(),
                FunctionKind::AuxiliaryContact,
            )
            .with_role(FunctionRole::Auxiliary),
        )
        .unwrap();
    let part = Part::new("ACME.K1", "ACME", "K1", "Contator", "Contator").unwrap();
    project.assign_part(k9, part, 1).unwrap();
    let preview = project
        .preview_device_numbering(NumberingFilter::Project)
        .unwrap();
    assert!(preview.rows.iter().all(|r| r.conflict.is_none()));
    let before = project.clone();
    let delta = project.apply_device_numbering(&preview).unwrap();
    assert_eq!(names(&project, &[k9, k5, k2]), ["-K1", "-K2", "-K3"]);
    assert_eq!(
        project.function(contact).unwrap().designation().to_string(),
        "-K1"
    );
    assert!(
        project
            .bom_rows()
            .iter()
            .any(|row| row.device_designation == "-K1")
    );
    let after = project.clone();
    project.restore_device_numbering(&delta, false).unwrap();
    assert_eq!(project, before);
    project.restore_device_numbering(&delta, true).unwrap();
    assert_eq!(project, after);
}

/// Critério 3: página/coluna gera -K2.4; dois na mesma coluna não colidem.
#[test]
fn page_column_rule_names_by_frame_zone_without_collisions() {
    let (mut project, _, second, _, _) = document_common::fixture();
    let a3 = FrameTemplate::factory_a3();
    let a3_id = a3.id();
    project.edit_frame(FrameEdit::UpsertFrame(a3)).unwrap();
    project
        .edit_frame(FrameEdit::SetDefaultFrame(Some(a3_id)))
        .unwrap();
    // Colunas de 41 mm a partir de x = 5: x = 150 → coluna 4.
    let upper = placed(&mut project, "K7", second, 150.0, 150.0);
    let lower = placed(&mut project, "K8", second, 150.0, 100.0);
    let mut scheme = project.numbering_scheme().clone();
    scheme.device_format = DeviceNumberFormat::PageColumn;
    project.set_numbering_scheme(scheme).unwrap();
    let preview = project
        .preview_device_numbering(NumberingFilter::Selection(vec![upper, lower]))
        .unwrap();
    assert!(preview.rows.iter().all(|r| r.conflict.is_none()));
    project.apply_device_numbering(&preview).unwrap();
    assert_eq!(names(&project, &[upper, lower]), ["-K2.4", "-K2.4.2"]);
}

#[test]
fn order_by_rows_and_unlocking_rule_change_the_result() {
    let (mut project, first, _, _, _) = document_common::fixture();
    // Coluna da esquerda embaixo, coluna da direita em cima.
    let left_low = placed(&mut project, "K5", first, 20.0, 20.0);
    let right_high = placed(&mut project, "K6", first, 200.0, 180.0);
    let ids = vec![left_low, right_high];
    let by_column = project
        .preview_device_numbering(NumberingFilter::Selection(ids.clone()))
        .unwrap();
    assert_eq!(by_column.rows[0].device, left_low);
    let mut scheme = project.numbering_scheme().clone();
    scheme.order = NumberingOrder::PositionXY;
    project.set_numbering_scheme(scheme.clone()).unwrap();
    let by_row = project
        .preview_device_numbering(NumberingFilter::Selection(ids.clone()))
        .unwrap();
    assert_eq!(by_row.rows[0].device, right_high);
    // Trava respeitada por padrão; desligar "manter travadas" renumera.
    project.set_designation_locked(left_low, true).unwrap();
    let kept = project
        .preview_device_numbering(NumberingFilter::Selection(ids.clone()))
        .unwrap();
    let row = |p: &DeviceNumberPreview| {
        p.rows
            .iter()
            .find(|r| r.device == left_low)
            .unwrap()
            .after
            .to_string()
    };
    assert_eq!(row(&kept), "-K5");
    scheme.keep_locked = false;
    project.set_numbering_scheme(scheme).unwrap();
    let renumbered = project
        .preview_device_numbering(NumberingFilter::Selection(ids))
        .unwrap();
    assert_eq!(row(&renumbered), "-K2");
    // -K1 fora da seleção (fixture) já existe: a prévia acusa o conflito.
    assert!(
        renumbered
            .rows
            .iter()
            .any(|r| r.device == right_high && r.conflict.is_some())
    );
    assert!(project.apply_device_numbering(&renumbered).is_err());
}

/// Critério 4: redes distintas recebem números distintos; número travado de
/// outra rede fica reservado.
#[test]
fn distinct_networks_get_distinct_numbers_and_skip_reserved_ones() {
    let (mut project, _, _, device, first_connection) = document_common::fixture();
    let function = project.functions_for_device(device).next().unwrap().id();
    let mut extra = Vec::new();
    for pair in [["B1", "B2"], ["C1", "C2"]] {
        let points: Vec<_> = pair
            .iter()
            .map(|label| {
                project
                    .insert_connection_point(ConnectionPoint::new(
                        project.id(),
                        function,
                        DesignationPart::new(*label).unwrap(),
                    ))
                    .unwrap()
            })
            .collect();
        extra.push(
            project
                .insert_connection(Connection::new(project.id(), points[0], points[1]).unwrap())
                .unwrap(),
        );
    }
    project
        .set_connection_number(first_connection, Some(Text::new("2").unwrap()), true)
        .unwrap();
    let preview = project.preview_connection_numbering().unwrap();
    project.apply_connection_numbering(&preview).unwrap();
    let mut numbers: Vec<_> = std::iter::once(first_connection)
        .chain(extra)
        .map(|id| {
            project
                .connection(id)
                .unwrap()
                .number()
                .unwrap()
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(numbers[0], "2");
    numbers.sort();
    assert_eq!(numbers, ["1", "2", "3"]);
}
