// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! terminals support for the physical editor/connectors; start at the parent module.

use super::*;

impl PhysicalEditor {
    /// Explicit association between functional points of the owner device and
    /// a terminal/contact (and level). It never creates or removes connections.
    pub(in crate::physical_editor) fn point_association_ui(
        &mut self,
        session: &mut Session,
        ui: &mut Ui,
        terminal: &Terminal,
        salt: &str,
    ) {
        let Some(p) = session.project() else { return };
        let owner = match terminal.owner() {
            TerminalOwner::Device(d) => Some(d),
            TerminalOwner::Component(c) => p.component(c).and_then(Component::device_id),
        };
        let level_name = |level: Option<TerminalLevelId>| {
            level
                .and_then(|id| terminal.level(id))
                .map(|l| format!(" · nível {}", l.designation().as_str()))
                .unwrap_or_default()
        };
        let associated: Vec<(ConnectionPointId, String)> = p
            .connection_points()
            .filter(|pt| pt.terminal_id() == Some(terminal.id()))
            .map(|pt| {
                (
                    pt.id(),
                    format!(
                        "{}{}",
                        p.connection_point_label(pt.id()),
                        level_name(pt.terminal_level())
                    ),
                )
            })
            .collect();
        let candidates: Vec<(ConnectionPointId, String)> = p
            .connection_points()
            .filter(|pt| pt.terminal_id() != Some(terminal.id()))
            .filter(|pt| p.function(pt.function_id()).map(Function::device_id) == owner)
            .map(|pt| {
                let current = pt
                    .terminal_id()
                    .map(|t| format!(" (em {})", p.terminal_label(t)))
                    .unwrap_or_default();
                (
                    pt.id(),
                    format!("{}{current}", p.connection_point_label(pt.id())),
                )
            })
            .collect();
        ui.label("Pontos funcionais associados (explícito, sem criar conexões):");
        if associated.is_empty() {
            ui.weak("Nenhum ponto associado.");
        }
        for (id, label) in associated {
            ui.horizontal(|ui| {
                ui.label(label);
                if ui.small_button("Desassociar").clicked() {
                    self.apply(
                        session,
                        Ok(PhysicalEdit::SetPointTerminal {
                            point: id,
                            terminal: None,
                            level: None,
                        }),
                    );
                }
            });
        }
        if self
            .kind
            .point
            .is_some_and(|id| !candidates.iter().any(|(c, _)| *c == id))
        {
            self.kind.point = None;
        }
        ui.horizontal_wrapped(|ui| {
            ComboBox::from_id_salt(("point_candidate", salt))
                .selected_text(
                    self.kind
                        .point
                        .and_then(|id| candidates.iter().find(|(c, _)| *c == id))
                        .map(|(_, l)| l.as_str())
                        .unwrap_or("Ponto do dispositivo"),
                )
                .show_ui(ui, |ui| {
                    for (id, label) in &candidates {
                        ui.selectable_value(&mut self.kind.point, Some(*id), label);
                    }
                });
            if terminal.kind() == TerminalKind::MultiLevel {
                ComboBox::from_id_salt(("point_level", salt))
                    .selected_text(
                        self.kind
                            .point_level
                            .and_then(|id| terminal.level(id))
                            .map(|l| l.designation().as_str())
                            .unwrap_or("Nível"),
                    )
                    .show_ui(ui, |ui| {
                        for level in terminal.levels() {
                            ui.selectable_value(
                                &mut self.kind.point_level,
                                Some(level.id()),
                                level.designation().as_str(),
                            );
                        }
                    });
            }
            if ui.button("Associar ponto").clicked() {
                let edit = self
                    .kind
                    .point
                    .ok_or(DomainError::InvalidRelation("selecione o ponto funcional"))
                    .map(|point| PhysicalEdit::SetPointTerminal {
                        point,
                        terminal: Some(terminal.id()),
                        level: (terminal.kind() == TerminalKind::MultiLevel)
                            .then_some(self.kind.point_level)
                            .flatten(),
                    });
                self.apply(session, edit);
            }
        });
    }

    /// Specialization editor of the terminal selected in the strip list.
    pub(in crate::physical_editor) fn terminal_kind_ui(
        &mut self,
        session: &mut Session,
        ui: &mut Ui,
        t: &Terminal,
    ) {
        if self.kind.seeded != Some(t.id()) {
            self.kind.seed(t);
        }
        ui.strong("Especialização do borne");
        if self
            .cache
            .lists
            .terminals
            .iter()
            .any(|r| r.terminal == t.id() && r.connector.is_some())
        {
            ui.weak("Contato de conector: permanece borne genérico.");
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("Tipo de borne");
            ComboBox::from_id_salt("terminal_kind")
                .selected_text(self.kind.kind.label())
                .show_ui(ui, |ui| {
                    for kind in TerminalKind::ALL {
                        ui.selectable_value(&mut self.kind.kind, kind, kind.label());
                    }
                });
            if self.kind.kind != TerminalKind::MultiLevel {
                ui.label("Posições do borne");
                ui.add(egui::TextEdit::singleline(&mut self.kind.positions).desired_width(90.));
            }
            ui.label("Lado A");
            ui.add(egui::TextEdit::singleline(&mut self.kind.side_a).desired_width(70.));
            ui.label("Lado B");
            ui.add(egui::TextEdit::singleline(&mut self.kind.side_b).desired_width(70.));
            if self.kind.kind == TerminalKind::Disconnect {
                ui.checkbox(&mut self.kind.disconnect_open, "Seccionado (aberto)")
                    .on_hover_text("Estado estático documental; não altera redes nem simula");
            }
        });
        if self.kind.kind == TerminalKind::MultiLevel {
            ui.label("Níveis (identificação · posições separadas por vírgula):");
            let mut remove = None;
            for (index, (_, name, positions)) in self.kind.levels.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("Nível {}", index + 1));
                    ui.add(egui::TextEdit::singleline(name).desired_width(50.));
                    ui.add(egui::TextEdit::singleline(positions).desired_width(110.));
                    if ui.small_button("Remover nível").clicked() {
                        remove = Some(index);
                    }
                });
            }
            if let Some(index) = remove {
                self.kind.levels.remove(index);
            }
            if ui.button("Adicionar nível").clicked() {
                let n = self.kind.levels.len() + 1;
                self.kind.levels.push((None, n.to_string(), String::new()));
            }
        }
        let note = match self.kind.kind {
            TerminalKind::ProtectiveEarth => {
                "PE: não cria terra nem redes; recusa condutor declarado não PE e ponte com borne não PE."
            }
            TerminalKind::MultiLevel => {
                "Níveis isolados: cada ponto associado indica seu nível; continuidade só por ponte explícita."
            }
            TerminalKind::FeedThrough | TerminalKind::Distribution => {
                "Continuidade somente pelos pontos funcionais associados explicitamente."
            }
            TerminalKind::Disconnect => {
                "Estado documental: não abre nem fecha continuidade no grafo (sem simulação)."
            }
            TerminalKind::Generic => "Borne sem especialização.",
        };
        ui.weak(note);
        if ui.button("Aplicar especialização").clicked() {
            let edit = terminal_from_draft(t, &self.kind).map(PhysicalEdit::UpsertTerminal);
            self.apply(session, edit);
        }
        self.point_association_ui(session, ui, t, "strip_terminal");
    }
}
