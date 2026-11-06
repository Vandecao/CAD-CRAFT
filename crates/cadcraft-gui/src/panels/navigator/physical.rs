// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Physical navigator rows and selection; start with physical_tree.
use super::*;

/// Hierarchical row of the cable/terminal navigators: depth, label and the
/// Core entity it selects. Rebuilt only when the project revision changes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PhysicalTreeRow {
    pub depth: u8,
    pub label: String,
    pub target: Option<RepresentationTarget>,
}
fn row(depth: u8, label: String, target: Option<RepresentationTarget>) -> PhysicalTreeRow {
    PhysicalTreeRow {
        depth,
        label,
        target,
    }
}
fn page_of(p: &Project, targets: impl IntoIterator<Item = RepresentationTarget>) -> String {
    targets
        .into_iter()
        .find_map(|t| {
            p.representations_for_entity(t)
                .ok()
                .and_then(|r| r.first().map(|r| r.page_id()))
        })
        .and_then(|page| p.page(page))
        .map(|page| format!(" · pág. {}", page.number().as_str()))
        .unwrap_or_default()
}
/// Derived tree for the Cables (cable → cores, individual wires) and
/// Terminals (strip → ordered terminals and bridges, loose terminals) tabs.
pub(crate) fn physical_tree(p: &Project, tab: NavigatorTab) -> Vec<PhysicalTreeRow> {
    let lists = p.physical_lists();
    let mut rows = Vec::new();
    match tab {
        NavigatorTab::Cables => {
            let mut cores: std::collections::BTreeMap<CableId, Vec<&CoreListRow>> =
                Default::default();
            for core in &lists.cores {
                cores.entry(core.cable).or_default().push(core);
            }
            for cable in &lists.cables {
                let ends = if cable.endpoints.is_empty() {
                    String::new()
                } else {
                    format!(" · {}", cable.endpoints.join(", "))
                };
                rows.push(row(
                    0,
                    format!(
                        "Cabo {} · {:?} · {}/{} veias usadas ({} nominais){ends}",
                        cable.designation,
                        cable.kind,
                        cable.used_cores,
                        cable.registered_cores,
                        cable.nominal_cores
                    ),
                    Some(RepresentationTarget::Cable(cable.cable)),
                ));
                for core in cores.get(&cable.cable).into_iter().flatten() {
                    let usage = match (&core.source, &core.target) {
                        (Some(a), Some(b)) => format!("usada · {a} → {b}"),
                        _ => "livre".into(),
                    };
                    rows.push(row(
                        1,
                        format!("Veia {} · {} · {usage}", core.identification, core.marking),
                        Some(RepresentationTarget::CableCore(core.core)),
                    ));
                }
            }
            let loose: Vec<_> = lists
                .conductors
                .iter()
                .filter(|w| w.core.is_none())
                .collect();
            if !loose.is_empty() {
                rows.push(row(0, format!("Fios individuais ({})", loose.len()), None));
                for w in loose {
                    rows.push(row(
                        1,
                        format!(
                            "{} · {} → {} · {} mm²{}",
                            w.designation,
                            w.source,
                            w.target,
                            w.cross_section_mm2,
                            if w.connection.is_some() {
                                " · associado"
                            } else {
                                ""
                            }
                        ),
                        Some(RepresentationTarget::Conductor(w.conductor)),
                    ));
                }
            }
        }
        NavigatorTab::Terminals => {
            let by_id: std::collections::BTreeMap<TerminalId, &TerminalListRow> =
                lists.terminals.iter().map(|t| (t.terminal, t)).collect();
            let terminal_row = |t: &TerminalListRow, depth: u8, order: String| {
                let location = page_of(
                    p,
                    std::iter::once(RepresentationTarget::Terminal(t.terminal)).chain(
                        t.points
                            .iter()
                            .map(|id| RepresentationTarget::ConnectionPoint(*id)),
                    ),
                );
                let kind = if t.kind == TerminalKind::Generic {
                    String::new()
                } else {
                    format!(" [{}]", t.kind.label())
                };
                row(
                    depth,
                    format!(
                        "{order}{}{kind} · {} · lados {} · {} conexões · {} pontes · {} condutores{location}",
                        t.designation,
                        if t.owner.is_empty() { "-" } else { &t.owner },
                        if t.positions.is_empty() {
                            "-".into()
                        } else {
                            t.positions.join("/")
                        },
                        t.connections.len(),
                        t.bridges.len(),
                        t.conductors.len()
                    ),
                    Some(RepresentationTarget::Terminal(t.terminal)),
                )
            };
            for strip in &lists.strips {
                rows.push(row(
                    0,
                    format!(
                        "Régua {} · {} bornes · {} pontes",
                        strip.designation,
                        strip.terminals.len(),
                        strip.bridges.len()
                    ),
                    Some(RepresentationTarget::TerminalStrip(strip.strip)),
                ));
                for (index, id) in strip.terminals.iter().enumerate() {
                    if let Some(t) = by_id.get(id) {
                        rows.push(terminal_row(t, 1, format!("{}. ", index + 1)));
                        // Régua → Borne → Níveis/Posições.
                        for level in &t.levels {
                            rows.push(row(
                                2,
                                format!("Nível {level}"),
                                Some(RepresentationTarget::Terminal(t.terminal)),
                            ));
                        }
                    }
                }
                for bridge in &strip.bridges {
                    let kind = p
                        .terminal_bridge(*bridge)
                        .map(|b| match b.kind() {
                            TerminalBridgeKind::Fixed => "fixa",
                            TerminalBridgeKind::Inserted => "inserida",
                        })
                        .unwrap_or("?");
                    let ends = p
                        .connection(*bridge)
                        .map(|c| {
                            let (a, b) = c.endpoints();
                            format!(
                                "{} ↔ {}",
                                p.connection_point_label(a),
                                p.connection_point_label(b)
                            )
                        })
                        .unwrap_or_default();
                    rows.push(row(
                        1,
                        format!("Ponte {kind} · {ends}"),
                        Some(RepresentationTarget::Connection(*bridge)),
                    ));
                }
            }
            // Conector → Contatos/Posições. The connector row targets its
            // documental device; contacts target their own terminal.
            for c in &lists.connectors {
                let free = c.contacts.iter().filter(|x| x.is_free()).count();
                rows.push(row(
                    0,
                    format!(
                        "Conector {} · {} · {} contatos ({free} livres){}",
                        c.designation,
                        c.gender.label(),
                        c.contacts.len(),
                        c.mated_with
                            .as_ref()
                            .map(|m| format!(" · acoplado a {m}"))
                            .unwrap_or_default()
                    ),
                    c.device.map(RepresentationTarget::Device),
                ));
                for (index, contact) in c.contacts.iter().enumerate() {
                    rows.push(row(
                        1,
                        format!(
                            "{}. Contato {} · posições {} · {}{}",
                            index + 1,
                            contact.designation,
                            if contact.positions.is_empty() {
                                "-".into()
                            } else {
                                contact.positions.join("/")
                            },
                            if contact.is_free() {
                                "livre"
                            } else {
                                "ocupado"
                            },
                            contact
                                .mated_label
                                .as_ref()
                                .map(|m| format!(" · ↔ {m}"))
                                .unwrap_or_default()
                        ),
                        Some(RepresentationTarget::Terminal(contact.terminal)),
                    ));
                }
            }
            let loose: Vec<_> = lists
                .terminals
                .iter()
                .filter(|t| t.strip.is_none() && t.connector.is_none())
                .collect();
            if !loose.is_empty() {
                rows.push(row(0, format!("Bornes sem régua ({})", loose.len()), None));
                for t in loose {
                    rows.push(terminal_row(t, 1, String::new()));
                    for level in &t.levels {
                        rows.push(row(
                            2,
                            format!("Nível {level}"),
                            Some(RepresentationTarget::Terminal(t.terminal)),
                        ));
                    }
                }
            }
        }
        _ => {}
    }
    rows
}
impl CadApp {
    pub(super) fn physical_navigator(&mut self, ui: &mut Ui) {
        let Some(p) = self.session.project() else {
            ui.weak("Nenhum projeto aberto.");
            return;
        };
        let key = (p.id(), self.session.revision, self.navigator.tab);
        if self.navigator.physical_key != Some(key) {
            self.navigator.physical_key = Some(key);
            self.navigator.physical_rows = physical_tree(p, self.navigator.tab);
        }
        let rows = std::mem::take(&mut self.navigator.physical_rows);
        ui.weak("Clique: propriedades · duplo clique: abrir página e centralizar ocorrência");
        if rows.is_empty() {
            ui.weak("Nenhum registro disponível neste navegador.");
        }
        let mut select = None;
        let mut locate = None;
        let height = ui.spacing().interact_size.y;
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show_rows(ui, height, rows.len(), |ui, range| {
                for entry in &rows[range] {
                    ui.horizontal(|ui| {
                        ui.add_space(f32::from(entry.depth) * 14.);
                        let Some(target) = entry.target else {
                            ui.strong(&entry.label);
                            return;
                        };
                        let selected = self.session.selection == Selection::Electrical(target);
                        let response = ui.selectable_label(selected, &entry.label);
                        if response.clicked() {
                            select = Some(target);
                        }
                        if response.double_clicked() {
                            locate = Some(target);
                        }
                    });
                }
            });
        self.navigator.physical_rows = rows;
        if let Some(target) = select {
            self.session.select(Selection::Electrical(target));
        }
        if let Some(target) = locate {
            match self
                .session
                .project()
                .and_then(|p| p.physical_occurrence(target))
            {
                Some(id) => self.navigate_to_reference(id),
                None => self.log(
                    crate::app::Level::Warning,
                    "Objeto físico sem ocorrência gráfica explícita.",
                ),
            }
        }
    }
}
