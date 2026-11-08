// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Factory symbols v2, function kind and prefix in symbol metadata, IEC
//! point numbers for new functions of an existing device, the repeated point
//! check (VER-021) and readable function labels.
// Synthetic test fixtures intentionally fail fast when setup is invalid.
#![allow(clippy::unwrap_used)]

use cad_craft::domain::*;

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn at(x: f64) -> Transform2D {
    Transform2D::new(p(x, 100.0), 0.0, false, false).unwrap()
}
fn symbol(key: &str) -> SymbolDefinition {
    SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap())
        .unwrap()
        .clone()
}
fn base() -> (Project, PageId) {
    let mut project = Project::new("Pontos").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Comando",
                "1",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    (project, page)
}
/// New device `prefix`N drawn with `key`; returns the device.
fn new_device(project: &mut Project, page: PageId, key: &str, prefix: &str) -> DeviceId {
    let definition = symbol(key);
    let kind = definition.function_kind_hint().unwrap();
    project
        .place_symbol(SymbolPlacement {
            page,
            definition,
            transform: at(50.0),
            device: PlacementDevice::New {
                designation: project.next_device_designation(prefix).unwrap(),
                classification: "Relé".into(),
            },
            function_kind: kind,
        })
        .unwrap()
        .device_id()
}
/// New function of `device` drawn with `key`, with the suggested labels
/// (or `labels` when given); returns the point labels created.
fn add_function(
    project: &mut Project,
    page: PageId,
    device: DeviceId,
    key: &str,
    labels: Option<&[&str]>,
) -> Vec<String> {
    let definition = symbol(key);
    let labels: Vec<String> = match labels {
        Some(l) => l.iter().map(|s| s.to_string()).collect(),
        None => project.suggested_point_labels(device, &definition),
    };
    let kind = definition.function_kind_hint().unwrap();
    let delta = project
        .place_symbol_with_point_labels(
            SymbolPlacement {
                page,
                definition,
                transform: at(150.0),
                device: PlacementDevice::Existing(device),
                function_kind: kind,
            },
            &labels,
        )
        .unwrap();
    delta
        .point_ids()
        .iter()
        .map(|id| {
            project
                .connection_point(*id)
                .unwrap()
                .designation()
                .as_str()
                .to_owned()
        })
        .collect()
}
fn identifications(key: &str) -> Vec<String> {
    symbol(key)
        .ports()
        .iter()
        .map(|p| p.identification().to_owned())
        .collect()
}

#[test]
fn factory_symbols_v2_name_ports_by_iec_and_show_the_designation() {
    assert_eq!(identifications("coil"), ["A1", "A2"]);
    assert_eq!(identifications("contact-no"), ["13", "14"]);
    assert_eq!(identifications("contact-nc"), ["21", "22"]);
    assert_eq!(identifications("push-button"), ["13", "14"]);
    for key in ["coil", "contact-no", "contact-nc", "push-button", "motor"] {
        let d = symbol(key);
        assert_eq!(d.version(), 2, "{key}");
        assert!(
            d.primitives().iter().any(|p| matches!(
                p,
                SymbolPrimitive::DynamicText {
                    field: SymbolTextField::Designation,
                    ..
                }
            )),
            "{key} mostra a designação"
        );
        // Port IDs and positions of v1 are kept, so old projects can update.
        let mut ports: Vec<_> = d
            .ports()
            .iter()
            .map(|p| (p.id().to_owned(), p.position()))
            .collect();
        ports.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            ports,
            [("1".to_owned(), p(0., -10.)), ("2".to_owned(), p(0., 10.))]
        );
    }
    assert_eq!(
        symbol("coil").function_kind_hint(),
        Some(FunctionKind::Coil)
    );
    assert_eq!(symbol("contact-no").designation_prefix(), Some("K"));
    assert_eq!(symbol("push-button").designation_prefix(), Some("S"));
}

#[test]
fn a_project_with_the_old_contact_keeps_it_until_the_update() {
    let (mut project, page) = base();
    let new = symbol("contact-no");
    // Version 1 as stored in projects made before this stage.
    let old = SymbolDefinition::new(
        new.id().clone(),
        1,
        new.name(),
        new.category(),
        vec![SymbolPrimitive::Line {
            start: p(0., -10.),
            end: p(0., 10.),
        }],
        vec![
            SymbolPort::new("1", p(0., -10.), 270., "1", None).unwrap(),
            SymbolPort::new("2", p(0., 10.), 90., "2", None).unwrap(),
        ],
    )
    .unwrap();
    let delta = project
        .place_symbol(SymbolPlacement {
            page,
            definition: old.clone(),
            transform: at(50.0),
            device: PlacementDevice::New {
                designation: project.next_device_designation("K").unwrap(),
                classification: "Relé".into(),
            },
            function_kind: FunctionKind::AuxiliaryContact,
        })
        .unwrap();
    assert_eq!(project.symbol_definition(new.id()).unwrap().version(), 1);
    assert!(project.symbol_update_problems(&new).is_empty());
    project.update_symbol_definition(new.clone()).unwrap();
    assert_eq!(project.symbol_definition(new.id()).unwrap().version(), 2);
    // The existing points keep their names; the new drawing shows them.
    let rep = delta.representation_id();
    assert_eq!(
        project
            .symbol_text(rep, &SymbolTextField::PortLabel("2".into()))
            .as_deref(),
        Some("2")
    );
    assert_eq!(
        project
            .symbol_text(rep, &SymbolTextField::Designation)
            .as_deref(),
        Some("-K1")
    );
}

#[test]
fn contacts_on_an_existing_device_get_the_next_free_iec_numbers() {
    let (mut project, page) = base();
    let k1 = new_device(&mut project, page, "coil", "K");
    assert_eq!(
        add_function(&mut project, page, k1, "contact-no", None),
        ["13", "14"]
    );
    assert_eq!(
        add_function(&mut project, page, k1, "contact-no", None),
        ["23", "24"]
    );
    assert_eq!(
        add_function(&mut project, page, k1, "contact-nc", None),
        ["31", "32"]
    );
    // Thermal relay and timer numbering.
    let f1 = new_device(&mut project, page, "coil", "F");
    assert_eq!(
        add_function(&mut project, page, f1, "contact-nc", None),
        ["95", "96"]
    );
    assert_eq!(
        add_function(&mut project, page, f1, "contact-no", None),
        ["97", "98"]
    );
    let kt1 = new_device(&mut project, page, "coil", "KT");
    assert_eq!(
        add_function(&mut project, page, kt1, "contact-no", None),
        ["15", "18"]
    );
    // A second coil on a device takes B1-B2.
    assert_eq!(
        add_function(&mut project, page, k1, "coil", None),
        ["B1", "B2"]
    );
}

#[test]
fn a_repeated_point_on_one_device_is_reported() {
    let (mut project, page) = base();
    let k1 = new_device(&mut project, page, "coil", "K");
    add_function(&mut project, page, k1, "contact-no", Some(&["13", "14"]));
    let clean = project.check(CheckScope::Project);
    assert!(!clean.messages.iter().any(|m| m.rule == "VER-021"));
    add_function(&mut project, page, k1, "contact-no", Some(&["13", "14"]));
    let report = project.check(CheckScope::Project);
    let repeated: Vec<_> = report
        .messages
        .iter()
        .filter(|m| m.rule == "VER-021")
        .collect();
    assert_eq!(repeated.len(), 2, "13 e 14: {repeated:?}");
    assert!(repeated[0].text.contains("-K1"), "{}", repeated[0].text);
    assert!(
        repeated[0]
            .targets
            .iter()
            .any(|t| matches!(t, CheckTarget::ConnectionPoint(_)))
    );
}

#[test]
fn function_labels_show_the_tag_kind_and_points() {
    let (mut project, page) = base();
    let k1 = new_device(&mut project, page, "coil", "K");
    add_function(&mut project, page, k1, "contact-no", None);
    let mut labels: Vec<String> = project
        .functions()
        .map(|f| project.function_label(f.id()).unwrap())
        .collect();
    labels.sort();
    assert_eq!(
        labels,
        ["-K1 · Bobina (A1, A2)", "-K1 · Contato auxiliar (13, 14)"]
    );
}

#[test]
fn user_symbols_record_function_kind_and_prefix_in_metadata() {
    let user = symbol("supply")
        .with_identity(
            SymbolDefinitionId::new("usuario.temporizador").unwrap(),
            "Bobina temporizada",
        )
        .unwrap();
    assert_eq!(user.function_kind_hint(), None);
    let metadata = user.metadata_with_hints(SymbolFunctionKind::Coil, " KT ");
    let typed = user.clone().with_metadata(metadata);
    assert_eq!(typed.function_kind_hint(), Some(FunctionKind::Coil));
    assert_eq!(typed.designation_prefix(), Some("KT"));
    assert_eq!(typed.symbol_function_kind(), SymbolFunctionKind::Coil);
    // "Outro" and an empty prefix remove the entries.
    let cleared = typed.metadata_with_hints(SymbolFunctionKind::Other, "");
    let cleared = typed.with_metadata(cleared);
    assert_eq!(cleared.function_kind_hint(), None);
    assert_eq!(cleared.designation_prefix(), None);
}
