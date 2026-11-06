// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! mating support for the physical editor/connectors; start at the parent module.

use super::*;

impl PhysicalEditor {
    pub(in crate::physical_editor) fn mating_ui(
        &mut self,
        session: &mut Session,
        ui: &mut Ui,
        connector: &Connector,
        project_id: ProjectId,
    ) {
        ui.strong("Acoplamento entre metades");
        ui.weak(
            "Documental: o mapa é explícito e validado, mas não cria conexões nem \
             continuidade elétrica (decisão pendente DEC-11-01).",
        );
        if let Some(mating) = self
            .cache
            .matings
            .iter()
            .find(|m| m.involves(connector.id()))
            .cloned()
        {
            let label = |t: TerminalId| {
                self.cache
                    .lists
                    .terminals
                    .iter()
                    .find(|r| r.terminal == t)
                    .map(terminal_text)
                    .unwrap_or_else(|| "?".into())
            };
            for pair in mating.pairs() {
                ui.label(format!(
                    "{} ↔ {}",
                    label(pair.first()),
                    label(pair.second())
                ));
            }
            if ui.button("Excluir acoplamento").clicked() {
                self.apply(session, Ok(PhysicalEdit::DeleteMating(mating.id())));
            }
            return;
        }
        let candidates: Vec<_> = self
            .cache
            .connectors
            .iter()
            .filter(|c| c.id() != connector.id())
            .cloned()
            .collect();
        ui.horizontal_wrapped(|ui| {
            ui.label("Conector par");
            ComboBox::from_id_salt("mating_partner")
                .selected_text(
                    self.connector
                        .partner
                        .and_then(|id| candidates.iter().find(|c| c.id() == id))
                        .map(|c| c.designation().as_str())
                        .unwrap_or("Selecione"),
                )
                .show_ui(ui, |ui| {
                    for c in &candidates {
                        if ui
                            .selectable_value(
                                &mut self.connector.partner,
                                Some(c.id()),
                                format!("{} ({})", c.designation().as_str(), c.gender().label()),
                            )
                            .clicked()
                        {
                            self.connector.pairs.clear();
                        }
                    }
                });
        });
        let Some(partner) = self
            .connector
            .partner
            .and_then(|id| candidates.iter().find(|c| c.id() == id))
            .cloned()
        else {
            return;
        };
        let name = |t: TerminalId| {
            self.cache
                .terminals
                .iter()
                .find(|x| x.id() == t)
                .map(|x| x.designation().as_str().to_owned())
                .unwrap_or_else(|| "?".into())
        };
        ui.horizontal_wrapped(|ui| {
            for (salt, value, list, text) in [
                (
                    "pair_first",
                    &mut self.connector.pair_first,
                    connector.contacts(),
                    "Contato deste",
                ),
                (
                    "pair_second",
                    &mut self.connector.pair_second,
                    partner.contacts(),
                    "Contato do par",
                ),
            ] {
                ComboBox::from_id_salt(salt)
                    .selected_text(value.map(name).unwrap_or_else(|| text.into()))
                    .show_ui(ui, |ui| {
                        for t in list {
                            ui.selectable_value(value, Some(*t), name(*t));
                        }
                    });
            }
            if ui.button("Adicionar par").clicked()
                && let (Some(a), Some(b)) = (self.connector.pair_first, self.connector.pair_second)
            {
                self.connector.pairs.push(ContactPair::new(a, b));
            }
            if ui
                .button("Parear por numeração")
                .on_hover_text(
                    "Preenche o rascunho com pares de mesma numeração; \
                     o mapa só vale após confirmação explícita",
                )
                .clicked()
            {
                self.connector.pairs = connector
                    .contacts()
                    .iter()
                    .filter_map(|a| {
                        let number = name(*a);
                        partner
                            .contacts()
                            .iter()
                            .find(|b| name(**b) == number)
                            .map(|b| ContactPair::new(*a, *b))
                    })
                    .collect();
            }
            if ui.button("Limpar pares").clicked() {
                self.connector.pairs.clear();
            }
        });
        for pair in &self.connector.pairs {
            ui.label(format!(
                "{}:{} ↔ {}:{}",
                connector.designation().as_str(),
                name(pair.first()),
                partner.designation().as_str(),
                name(pair.second())
            ));
        }
        if ui.button("Confirmar acoplamento").clicked() {
            let mating = ConnectorMating::new(
                project_id,
                connector.id(),
                partner.id(),
                self.connector.pairs.clone(),
            );
            self.apply(session, Ok(PhysicalEdit::UpsertMating(mating)));
            if self.cache.key.is_none() {
                self.connector.pairs.clear();
            }
        }
    }
}
