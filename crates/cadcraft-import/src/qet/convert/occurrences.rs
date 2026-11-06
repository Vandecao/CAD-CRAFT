// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Converts labels and places master, slave and simple occurrences in order; start at `convert`.
use super::*;

pub(super) fn convert(
    importer: &mut Importer<'_>,
    state: &DiagramState,
) -> ImportResult<PlacementState> {
    let DiagramState {
        pages, instances, ..
    } = state;
    let s = importer.s;
    let world = |page, x, y| super::world(pages, s, page, x, y);
    // QET 0.7+ stores label text on the occurrence. Without a designation in
    // the definition, add dynamic text at the first occurrence's position.
    for inst in instances {
        let Some(label) = &inst.label_text else {
            continue;
        };
        let Some(def) = importer.definitions.get_mut(&inst.def) else {
            continue;
        };
        let has = def.symbol.primitives().iter().any(|p| {
            matches!(
                p,
                SymbolPrimitive::DynamicText {
                    field: SymbolTextField::Designation,
                    ..
                }
            )
        });
        if has {
            continue;
        }
        let mut primitives = def.symbol.primitives().to_vec();
        primitives.push(label.clone());
        let rebuilt = SymbolDefinition::new(
            def.symbol.id().clone(),
            def.symbol.version(),
            def.symbol.name(),
            def.symbol.category(),
            primitives,
            def.symbol.ports().to_vec(),
        )
        .and_then(|d| d.with_styles(def.symbol.styles().clone()));
        match rebuilt {
            Ok(symbol) => def.symbol = symbol,
            Err(e) => importer
                .report
                .ignored(format!("rótulo da ocorrência: {e}")),
        }
    }

    // Place master occurrences first, followed by slaves and simple occurrences.
    let by_uuid: BTreeMap<String, usize> = instances
        .iter()
        .enumerate()
        .filter(|(_, i)| !i.uuid.is_empty())
        .map(|(n, i)| (i.uuid.clone(), n))
        .collect();
    let link_of = |i: &Instance| {
        importer
            .definitions
            .get(&i.def)
            .map_or("simple".to_owned(), |d| d.link.clone())
    };
    let mut order: Vec<usize> = (0..instances.len()).collect();
    order.sort_by_key(|n| match link_of(&instances[*n]).as_str() {
        "master" => 0,
        "slave" => 1,
        _ => 2,
    });
    let mut placed: BTreeMap<usize, RepresentationId> = BTreeMap::new();
    let mut device_of: BTreeMap<usize, DeviceId> = BTreeMap::new();
    let mut reports: Vec<usize> = vec![];
    for n in order {
        let inst = &instances[n];
        let link = link_of(inst);
        if link.ends_with("_report") {
            reports.push(n);
            continue;
        }
        let Some(def) = importer.definitions.get(&inst.def) else {
            importer.report.ignored(format!(
                "elemento '{}' ausente da coleção do projeto",
                inst.def
            ));
            continue;
        };
        // Symbols without terminals (drawings or mounting plans) have occurrences
        // without a device.
        if def.symbol.ports().is_empty() {
            let symbol = def.symbol.clone();
            let placed_graphic = Transform2D::new(
                world(inst.page, inst.x, inst.y)?,
                rotation(inst.orientation),
                false,
                false,
            )
            .and_then(|t| {
                importer.project.add_symbol_definition(symbol.clone())?;
                Representation::new_symbol(
                    importer.project.id(),
                    pages[inst.page].0,
                    &symbol,
                    None,
                    t,
                )
            })
            .and_then(|r| importer.project.insert_representation(r));
            match placed_graphic {
                Ok(_) => {
                    importer.stats.occurrences += 1;
                    importer.stats.drawings += 1;
                }
                Err(e) => importer
                    .report
                    .ignored(format!("elemento '{}': {e}", inst.def)),
            }
            continue;
        }
        let master = if link == "slave" {
            inst.links
                .iter()
                .filter_map(|u| by_uuid.get(u))
                .find_map(|m| device_of.get(m).copied())
        } else {
            None
        };
        // Repeated labels outside master/slave relations (terminals or components
        // on multiple sheets) share QElectroTech identity and therefore the same
        // device.
        let repeated = if master.is_none() && link != "master" {
            device_part(&inst.label)
                .and_then(|d| Designation::new(d).ok())
                .and_then(|c| {
                    importer
                        .project
                        .devices()
                        .find(|x| x.designation() == &c)
                        .map(Device::id)
                })
        } else {
            None
        };
        if repeated.is_some() {
            importer
                .report
                .approximated("rótulo repetido: ocorrência agrupada no dispositivo existente");
        }
        let device = match master.or(repeated) {
            Some(id) => PlacementDevice::Existing(id),
            None => {
                let wanted = device_part(&inst.label).filter(|d| {
                    let candidate = Designation::new(d.as_str());
                    candidate
                        .is_ok_and(|c| importer.project.devices().all(|x| x.designation() != &c))
                });
                let designation = match wanted {
                    Some(d) => Designation::new(d).map_err(domain)?,
                    None => {
                        let prefix = if inst.prefix.trim().is_empty() {
                            prefix_of(&inst.label)
                        } else {
                            prefix_of(&inst.prefix)
                        };
                        if !inst.label.trim().is_empty() {
                            importer.report.approximated(
                                "rótulo inválido ou repetido de mestre: próxima designação livre",
                            );
                        }
                        importer
                            .project
                            .next_device_designation(&prefix)
                            .map_err(domain)?
                    }
                };
                PlacementDevice::New {
                    designation,
                    classification: def.symbol.name().to_owned(),
                }
            }
        };
        let kind = match (link.as_str(), def.kind.as_str()) {
            ("master", "coil") => FunctionKind::Coil,
            ("slave", "power") => FunctionKind::MainContact,
            ("slave", _) => FunctionKind::AuxiliaryContact,
            ("master", k) | (_, k) => FunctionKind::Other(
                Text::new(if k.is_empty() { "componente" } else { k })
                    .unwrap_or_else(|_| Text::new("componente").expect("constante")),
            ),
        };
        let transform = Transform2D::new(
            world(inst.page, inst.x, inst.y)?,
            rotation(inst.orientation),
            false,
            false,
        )
        .map_err(domain)?;
        let result = importer.project.place_symbol(SymbolPlacement {
            page: pages[inst.page].0,
            definition: def.symbol.clone(),
            transform,
            device,
            function_kind: kind,
        });
        match result {
            Ok(delta) => {
                placed.insert(n, delta.representation_id());
                device_of.insert(n, delta.device_id());
                if delta.created_device() {
                    importer.stats.devices += 1;
                }
                if link == "master" {
                    let _ = importer.project.edit_device(DeviceEdit::SetMainFunction {
                        device: delta.device_id(),
                        function: Some(delta.function_id()),
                    });
                }
                importer.stats.occurrences += 1;
            }
            Err(e) => importer
                .report
                .ignored(format!("elemento '{}': {e}", inst.def)),
        }
    }

    Ok(PlacementState {
        by_uuid,
        placed,
        reports,
    })
}
