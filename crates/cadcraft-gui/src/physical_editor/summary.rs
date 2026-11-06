// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! summary support for the physical editor; start at the parent module.

use super::*;

/// Sequential numbering for the strip order. Identity stays the TerminalId;
/// only the documental designation changes, atomically as one batch.
pub(crate) fn renumber_edits(
    terminals: &[Terminal],
    order: &[TerminalId],
    start: u32,
) -> Result<Vec<PhysicalEdit>> {
    order
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let terminal = terminals
                .iter()
                .find(|t| t.id() == *id)
                .ok_or(DomainError::MissingEntity(EntityRef::Terminal(*id)))?;
            let number = start
                .checked_add(index as u32)
                .ok_or(DomainError::InvalidNumber("numeração de borne"))?;
            Ok(PhysicalEdit::UpsertTerminal(
                terminal
                    .clone()
                    .with_designation(DesignationPart::new(number.to_string())?),
            ))
        })
        .collect()
}

/// Read-only physical rows for the Properties panel. Bounded to one entity;
/// edits stay in the physical editor and go through `PhysicalEdit`.
pub(crate) fn physical_summary(p: &Project, target: RepresentationTarget) -> Vec<(String, String)> {
    let text = |v: Option<&Text>| {
        v.map(|t| t.as_str().to_owned())
            .unwrap_or_else(|| "-".into())
    };
    let mut rows: Vec<(String, String)> = Vec::new();
    match target {
        RepresentationTarget::Terminal(id) => {
            let Some(t) = p.terminal(id) else { return rows };
            rows.push(("Borne".into(), p.terminal_label(id)));
            rows.push(("Numeração".into(), t.designation().as_str().into()));
            let sides: Vec<_> = t.positions().map(|v| v.as_str()).collect();
            rows.push((
                "Lados de conexão".into(),
                if sides.is_empty() {
                    "-".into()
                } else {
                    sides.join("/")
                },
            ));
            rows.push(("Especialização".into(), t.kind().label().into()));
            if !t.sides().is_empty() {
                let sides: Vec<_> = t
                    .sides()
                    .iter()
                    .map(|(pos, side)| format!("{} {}", pos.as_str(), side.label()))
                    .collect();
                rows.push(("Lados declarados".into(), sides.join(", ")));
            }
            for level in t.levels() {
                let positions: Vec<_> = level.positions().map(|v| v.as_str()).collect();
                rows.push((
                    format!("Nível {}", level.designation().as_str()),
                    positions.join("/"),
                ));
            }
            if let Some(state) = t.disconnect_state() {
                rows.push((
                    "Estado de seccionamento (documental)".into(),
                    match state {
                        DisconnectState::Closed => "fechado".into(),
                        DisconnectState::Open => "aberto".into(),
                    },
                ));
            }
            if let Some(connector) = p.connector_for_terminal(id) {
                rows.push((
                    "Conector".into(),
                    format!(
                        "{} · {}",
                        connector.designation().as_str(),
                        connector.gender().label()
                    ),
                ));
                if let Some(mate) = p.mated_contact(id) {
                    rows.push(("Acoplado a (documental)".into(), p.terminal_label(mate)));
                }
            }
            if let Some(strip) = p.strip_for_terminal(id) {
                let order = strip
                    .terminals()
                    .iter()
                    .position(|x| *x == id)
                    .map(|i| i + 1);
                rows.push((
                    "Régua".into(),
                    format!(
                        "{} · posição {}",
                        strip.designation().as_str(),
                        order.unwrap_or_default()
                    ),
                ));
            }
            let wires = p
                .conductors()
                .filter(|w| {
                    w.endpoints().0.terminal_id() == id || w.endpoints().1.terminal_id() == id
                })
                .map(|w| w.designation().as_str().to_owned())
                .collect::<Vec<_>>();
            rows.push((
                "Condutores".into(),
                if wires.is_empty() {
                    "-".into()
                } else {
                    wires.join(", ")
                },
            ));
        }
        RepresentationTarget::TerminalStrip(id) => {
            let Some(s) = p.terminal_strip(id) else {
                return rows;
            };
            rows.push(("Régua".into(), s.designation().as_str().into()));
            rows.push(("Bornes".into(), s.terminals().len().to_string()));
            rows.push((
                "Pontes".into(),
                p.terminal_bridges()
                    .filter(|b| b.strip_id() == id)
                    .count()
                    .to_string(),
            ));
        }
        RepresentationTarget::Cable(id) => {
            let Some(c) = p.cable(id) else { return rows };
            let cores: Vec<_> = p.cores_for_cable(id).collect();
            rows.push(("Cabo".into(), c.designation().as_str().into()));
            rows.push(("Tipo".into(), format!("{:?}", c.kind())));
            rows.push(("Descrição".into(), text(c.description())));
            rows.push(("Fabricante".into(), text(c.manufacturer())));
            rows.push(("Código de produto".into(), text(c.catalog_code())));
            rows.push((
                "Veias nominais/cadastradas/usadas".into(),
                format!(
                    "{}/{}/{}",
                    c.nominal_core_count(),
                    cores.len(),
                    cores.iter().filter(|k| k.conductor_id().is_some()).count()
                ),
            ));
        }
        RepresentationTarget::CableCore(id) => {
            let Some(core) = p.cable_core(id) else {
                return rows;
            };
            rows.push((
                "Veia".into(),
                format!(
                    "{}:{}",
                    p.cable(core.cable_id())
                        .map(|c| c.designation().as_str())
                        .unwrap_or("?"),
                    core.identification().as_str()
                ),
            ));
            rows.push((
                "Seção nominal".into(),
                core.cross_section()
                    .map(|v| format!("{} mm²", v.get()))
                    .unwrap_or_else(|| "não informada".into()),
            ));
            rows.push((
                "Utilização".into(),
                core.conductor_id()
                    .and_then(|w| p.conductor(w))
                    .map(|w| format!("usada por {}", w.designation().as_str()))
                    .unwrap_or_else(|| "livre".into()),
            ));
        }
        RepresentationTarget::Conductor(id) => {
            let Some(w) = p.conductor(id) else {
                return rows;
            };
            let (a, b) = w.endpoints();
            rows.push(("Condutor".into(), w.designation().as_str().into()));
            rows.push((
                "Origem".into(),
                format!(
                    "{}/{}",
                    p.terminal_label(a.terminal_id()),
                    a.position().as_str()
                ),
            ));
            rows.push((
                "Destino".into(),
                format!(
                    "{}/{}",
                    p.terminal_label(b.terminal_id()),
                    b.position().as_str()
                ),
            ));
            rows.push((
                "Seção nominal".into(),
                format!("{} mm²", w.cross_section().get()),
            ));
            rows.push((
                "Cabo/veia".into(),
                p.core_for_conductor(id)
                    .and_then(|core| {
                        p.cable(core.cable_id()).map(|c| {
                            format!(
                                "{}:{}",
                                c.designation().as_str(),
                                core.identification().as_str()
                            )
                        })
                    })
                    .unwrap_or_else(|| "fio individual".into()),
            ));
            rows.push((
                "Conexão lógica".into(),
                w.connection()
                    .and_then(|l| p.connection(l.connection_id()))
                    .map(|c| {
                        let (a, b) = c.endpoints();
                        format!(
                            "{} → {}",
                            p.connection_point_label(a),
                            p.connection_point_label(b)
                        )
                    })
                    .unwrap_or_else(|| "não associada".into()),
            ));
        }
        RepresentationTarget::Connection(id) => {
            rows.extend(crate::connection_properties::physical_rows(p, id));
        }
        _ => {}
    }
    rows
}
