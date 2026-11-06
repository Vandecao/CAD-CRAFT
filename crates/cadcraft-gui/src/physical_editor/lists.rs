// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! lists support for the physical editor; start at the parent module.

use super::*;

impl PhysicalEditor {
    pub(in crate::physical_editor) fn lists_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        const TABS: [&str; 6] = [
            "Réguas",
            "Bornes",
            "Cabos",
            "Veias",
            "Condutores",
            "Associações",
        ];
        ui.horizontal_wrapped(|ui| {
            for (index, label) in TABS.iter().enumerate() {
                if ui
                    .selectable_label(self.list_tab == index, *label)
                    .clicked()
                {
                    self.list_tab = index;
                }
            }
            ui.separator();
            if ui.button("Localizar seleção").clicked()
                && let Some(target) = self.list_selected
            {
                self.request_locate(session, target);
            }
        });
        ui.weak("Visualizações derivadas do Core a cada revisão; não são cópias editáveis.");
        let lists = &self.cache.lists;
        let (header, rows): (&str, Vec<(String, RepresentationTarget)>) = match self.list_tab {
            0 => (
                "Régua │ bornes │ pontes",
                lists
                    .strips
                    .iter()
                    .map(|s| {
                        (
                            format!(
                                "{:<10} │ {:>6} │ {:>6}",
                                s.designation,
                                s.terminals.len(),
                                s.bridges.len()
                            ),
                            RepresentationTarget::TerminalStrip(s.strip),
                        )
                    })
                    .collect(),
            ),
            1 => (
                "Borne │ ordem │ dono │ posições │ conexões │ pontes │ condutores",
                lists
                    .terminals
                    .iter()
                    .map(|t| {
                        (
                            format!(
                                "{:<12} │ {:>5} │ {:<8} │ {:<8} │ {:>3} │ {:>3} │ {:>3}",
                                terminal_text(t),
                                t.order.map(|o| o.to_string()).unwrap_or_else(|| "-".into()),
                                t.owner,
                                t.positions.join("/"),
                                t.connections.len(),
                                t.bridges.len(),
                                t.conductors.len()
                            ),
                            RepresentationTarget::Terminal(t.terminal),
                        )
                    })
                    .collect(),
            ),
            2 => (
                "Cabo │ tipo │ veias nominais/cadastradas/usadas │ origem/destino",
                lists
                    .cables
                    .iter()
                    .map(|c| {
                        (
                            format!(
                                "{:<8} │ {:?} │ {}/{}/{} │ {}",
                                c.designation,
                                c.kind,
                                c.nominal_cores,
                                c.registered_cores,
                                c.used_cores,
                                c.endpoints.join(", ")
                            ),
                            RepresentationTarget::Cable(c.cable),
                        )
                    })
                    .collect(),
            ),
            3 => (
                "Cabo:veia │ marcação │ seção │ estado │ origem → destino",
                lists
                    .cores
                    .iter()
                    .map(|c| {
                        (
                            format!(
                                "{}:{} │ {} │ {} │ {} │ {} → {}",
                                c.cable_designation,
                                c.identification,
                                c.marking,
                                c.cross_section_mm2
                                    .map(|v| format!("{v} mm²"))
                                    .unwrap_or_else(|| "não informada".into()),
                                if c.conductor.is_some() {
                                    "usada"
                                } else {
                                    "livre"
                                },
                                c.source.as_deref().unwrap_or("-"),
                                c.target.as_deref().unwrap_or("-")
                            ),
                            RepresentationTarget::CableCore(c.core),
                        )
                    })
                    .collect(),
            ),
            4 => (
                "Condutor │ origem → destino │ seção │ cabo │ conexão",
                lists
                    .conductors
                    .iter()
                    .map(|w| {
                        (
                            format!(
                                "{} │ {} → {} │ {} mm² │ {} │ {}",
                                w.designation,
                                w.source,
                                w.target,
                                w.cross_section_mm2,
                                w.cable_label.as_deref().unwrap_or("fio individual"),
                                if w.connection.is_some() {
                                    "associada"
                                } else {
                                    "-"
                                }
                            ),
                            RepresentationTarget::Conductor(w.conductor),
                        )
                    })
                    .collect(),
            ),
            _ => (
                "Conexão lógica │ condutores físicos",
                lists
                    .assignments
                    .iter()
                    .map(|a| {
                        let wires = if a.bridge {
                            "ponte de régua".to_owned()
                        } else {
                            a.conductors
                                .iter()
                                .filter_map(|id| {
                                    lists.conductors.iter().find(|w| w.conductor == *id)
                                })
                                .map(|w| w.designation.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        (
                            format!(
                                "{} → {} │ {}",
                                a.source,
                                a.target,
                                if wires.is_empty() { "-".into() } else { wires }
                            ),
                            RepresentationTarget::Connection(a.connection),
                        )
                    })
                    .collect(),
            ),
        };
        // Truncated: a long header must not set a minimum width for the window.
        ui.add(egui::Label::new(egui::RichText::new(header).monospace().strong()).truncate());
        let height = ui.spacing().interact_size.y;
        let mut picked = None;
        // Both axes: long monospace rows scroll instead of widening the window.
        ScrollArea::both()
            .id_salt(("technical_list", self.list_tab))
            .auto_shrink([false, false])
            .show_rows(ui, height, rows.len(), |ui, range| {
                for (text, target) in &rows[range] {
                    let selected = self.list_selected == Some(*target);
                    if ui
                        .selectable_label(selected, egui::RichText::new(text).monospace())
                        .clicked()
                    {
                        picked = Some(*target);
                    }
                }
            });
        if let Some(target) = picked {
            self.list_selected = Some(target);
            session.select(Selection::Electrical(target));
        }
    }
}
pub(in crate::physical_editor) fn terminal_text(row: &TerminalListRow) -> String {
    match (&row.strip_designation, &row.connector_designation) {
        (Some(group), _) | (None, Some(group)) => format!("{group}:{}", row.designation),
        (None, None) => format!("{}:{}", row.owner, row.designation),
    }
}
