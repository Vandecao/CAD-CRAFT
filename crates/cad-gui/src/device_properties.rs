// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::state::{Selection, Session};
use cad_craft::{domain::*, error::Result};
use eframe::egui::{self, Ui};
#[derive(Default)]
pub(crate) struct DeviceInspector {
    key: Option<(Selection, u64, u64)>,
    pub device_tag: String,
    pub functional: String,
    pub location: String,
    classification: String,
    function: Option<FunctionId>,
    role: FunctionRole,
    function_tag: String,
    function_functional: String,
    function_location: String,
    designation_conflict: bool,
}
fn path(s: &str) -> Result<Vec<DesignationPart>> {
    s.split('/')
        .filter(|p| !p.is_empty())
        .map(DesignationPart::new)
        .collect()
}
pub(crate) fn show(
    ui: &mut Ui,
    s: &Session,
    state: &mut DeviceInspector,
) -> (Option<Result<DeviceEdit>>, Option<RepresentationId>) {
    let Some(p) = s.project() else {
        return (None, None);
    };
    let Some(id) = s.selected_reference_device() else {
        return (None, None);
    };
    let Some(d) = p.device(id) else {
        return (None, None);
    };
    let key = (s.selection, s.revision, s.selection_revision);
    if state.key != Some(key) {
        state.key = Some(key);
        state.device_tag = d.designation().device().as_str().into();
        state.functional = d
            .designation()
            .functional_path()
            .iter()
            .map(DesignationPart::as_str)
            .collect::<Vec<_>>()
            .join("/");
        state.location = d
            .designation()
            .location_path()
            .iter()
            .map(DesignationPart::as_str)
            .collect::<Vec<_>>()
            .join("/");
        state.classification = d.classification().as_str().into();
        state.function = match s.selection {
            Selection::Representation(r) => p
                .representation(r)
                .and_then(|r| p.function_for_target(r.target())),
            Selection::Electrical(t) => p.function_for_target(t),
            _ => None,
        };
        state.role = state
            .function
            .and_then(|f| p.function(f))
            .map(|f| f.role())
            .unwrap_or_default();
        state.designation_conflict = p
            .designation_conflicts()
            .iter()
            .any(|ids| ids.contains(&id));
        if let Some(f) = state.function.and_then(|id| p.function(id)) {
            state.function_tag = f.designation().device().as_str().into();
            state.function_functional = f
                .designation()
                .functional_path()
                .iter()
                .map(DesignationPart::as_str)
                .collect::<Vec<_>>()
                .join("/");
            state.function_location = f
                .designation()
                .location_path()
                .iter()
                .map(DesignationPart::as_str)
                .collect::<Vec<_>>()
                .join("/");
        }
    }
    let mut edit = None;
    let mut nav = None;
    ui.heading("Dispositivo e referências");
    if state.designation_conflict {
        ui.colored_label(
            ui.visuals().error_fg_color,
            "Conflito: identificação completa duplicada no projeto.",
        );
    }
    ui.label(format!("ID: {}", id));
    ui.label(format!("Identificação completa: {}", d.designation()));
    ui.label(format!("Exibida: -{}", d.designation().device().as_str()));
    let mut locked = d.designation_locked();
    if ui.checkbox(&mut locked, "Travar designação").changed() {
        edit = Some(Ok(DeviceEdit::SetDesignationLocked { device: id, locked }));
    }
    ui.label("Função (=)");
    ui.text_edit_singleline(&mut state.functional);
    ui.label("Localização (+)");
    ui.text_edit_singleline(&mut state.location);
    ui.label("Dispositivo (-)");
    ui.text_edit_singleline(&mut state.device_tag);
    if ui.button("Aplicar identificação").clicked() {
        edit = Some(
            Designation::new(&state.device_tag)
                .and_then(|v| {
                    Ok(v.with_functional_path(path(&state.functional)?)
                        .with_location_path(path(&state.location)?))
                })
                .map(|designation| DeviceEdit::Rename {
                    device: id,
                    designation,
                }),
        );
    }
    ui.label("Classificação");
    ui.text_edit_singleline(&mut state.classification);
    if ui.button("Aplicar classificação").clicked() {
        edit = Some(Text::new(&state.classification).map(|classification| {
            DeviceEdit::SetClassification {
                device: id,
                classification,
            }
        }));
    }
    ui.separator();
    ui.strong("Funções existentes");
    let selected_function = state.function;
    egui::ComboBox::from_id_salt(("function", id))
        .selected_text(
            state
                .function
                .and_then(|f| p.function(f))
                .map(|f| format!("{} · {}", f.id(), f.designation()))
                .unwrap_or_else(|| "Escolher função".into()),
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.function, None, "Escolher função");
            for f in p.functions() {
                ui.selectable_value(
                    &mut state.function,
                    Some(f.id()),
                    format!("{} · {}", f.id(), f.designation()),
                );
            }
        });
    if state.function != selected_function
        && let Some(f) = state.function.and_then(|id| p.function(id))
    {
        state.function_tag = f.designation().device().as_str().into();
        state.function_functional = f
            .designation()
            .functional_path()
            .iter()
            .map(DesignationPart::as_str)
            .collect::<Vec<_>>()
            .join("/");
        state.function_location = f
            .designation()
            .location_path()
            .iter()
            .map(DesignationPart::as_str)
            .collect::<Vec<_>>()
            .join("/");
        state.role = f.role();
    }
    if let Some(f) = state.function {
        if let Some(function) = p.function(f) {
            ui.label(format!(
                "Tipo: {:?} · papel: {:?}",
                function.kind(),
                function.role()
            ));
            ui.label("Identificação da função");
            ui.text_edit_singleline(&mut state.function_tag);
            ui.label("Estrutura funcional da função (=)");
            ui.text_edit_singleline(&mut state.function_functional);
            ui.label("Localização da função (+)");
            ui.text_edit_singleline(&mut state.function_location);
            if ui.button("Aplicar identificação da função").clicked() {
                edit = Some(
                    Designation::new(&state.function_tag)
                        .and_then(|v| {
                            Ok(v.with_functional_path(path(&state.function_functional)?)
                                .with_location_path(path(&state.function_location)?))
                        })
                        .map(|designation| DeviceEdit::RenameFunction {
                            function: f,
                            designation,
                        }),
                );
            }
        }
        egui::ComboBox::from_id_salt(("role", id))
            .selected_text(format!("{:?}", state.role))
            .show_ui(ui, |ui| {
                for role in [
                    FunctionRole::Unspecified,
                    FunctionRole::Main,
                    FunctionRole::Auxiliary,
                ] {
                    ui.selectable_value(&mut state.role, role, format!("{role:?}"));
                }
            });
        if ui.button("Associar função").clicked() {
            edit = Some(Ok(DeviceEdit::AssociateFunction {
                function: f,
                device: id,
                role: state.role,
            }));
        }
        if ui.button("Definir principal").clicked() {
            edit = Some(Ok(DeviceEdit::SetMainFunction {
                device: id,
                function: Some(f),
            }));
        }
    }
    ui.separator();
    ui.strong("Ocorrências e contatos");
    for r in p.device_references(id).unwrap_or_default() {
        let label = r.reference.clone();
        ui.label(format!(
            "{} · {:?} · {}",
            r.function_designation
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            r.role,
            r.terminals.join(" / ")
        ));
        if ui.link(label).clicked() {
            nav = Some(r.representation);
        }
    }
    let mirror = p.contact_mirror(id).unwrap_or_default();
    if !mirror.is_empty() {
        ui.separator();
        ui.strong("Espelho de contatos");
        for row in mirror {
            let kind = match &row.kind {
                FunctionKind::MainContact => "Principal",
                FunctionKind::AuxiliaryContact
                    if row.terminals.iter().any(|s| s.ends_with('3'))
                        && row.terminals.iter().any(|s| s.ends_with('4')) =>
                {
                    "NA"
                }
                FunctionKind::AuxiliaryContact
                    if row.terminals.iter().any(|s| s.ends_with('1'))
                        && row.terminals.iter().any(|s| s.ends_with('2')) =>
                {
                    "NF"
                }
                _ => "Contato",
            };
            ui.label(format!("{kind} {}", row.terminals.join("-")));
            if row.references.is_empty() {
                ui.weak("Não representado");
            }
            for reference in row.references {
                if ui.link(&reference.reference).clicked() {
                    nav = Some(reference.representation);
                }
            }
        }
    }
    (edit, nav)
}
