// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Session editor for physical engineering records. Every write goes through
//! Project::edit_physical and the shared bounded history.
use crate::state::{Selection, Session};
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};
use eframe::egui::{self, ComboBox, Context, ScrollArea, Ui};

mod connectors;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PhysicalTab {
    #[default]
    Strips,
    Connectors,
    Cables,
    Wiring,
    Lists,
}

#[derive(Default)]
struct PhysicalCache {
    key: Option<(ProjectId, u64)>,
    devices: Vec<(DeviceId, String)>,
    strips: Vec<TerminalStrip>,
    terminals: Vec<Terminal>,
    points: Vec<ConnectionPoint>,
    cables: Vec<Cable>,
    cores: Vec<CableCore>,
    wires: Vec<Conductor>,
    bridges: Vec<TerminalBridge>,
    connectors: Vec<Connector>,
    matings: Vec<ConnectorMating>,
    lists: PhysicalLists,
}

#[derive(Default)]
pub(crate) struct PhysicalEditor {
    pub open: bool,
    pub tab: PhysicalTab,
    cache: PhysicalCache,
    feedback: String,
    selected_strip: Option<TerminalStripId>,
    selected_terminal: Option<TerminalId>,
    attach_candidate: Option<TerminalId>,
    new_terminal_name: String,
    selected_cable: Option<CableId>,
    selected_core: Option<CableCoreId>,
    selected_wire: Option<ConductorId>,
    selected_device: Option<DeviceId>,
    selected_connection: Option<ConnectionId>,
    bridge_source: Option<ConnectionPointId>,
    bridge_target: Option<ConnectionPointId>,
    bridge_kind: bool,
    new_strip: String,
    strip_name: String,
    terminal_name: String,
    terminal_position: String,
    cable_name: String,
    new_cable_name: String,
    new_cable_count: String,
    cable_kind_index: usize,
    cable_description: String,
    cable_manufacturer: String,
    cable_catalog: String,
    cable_count: String,
    core_name: String,
    core_marking: String,
    core_section: String,
    wire_name: String,
    wire_section: String,
    wire_edit_name: String,
    wire_edit_section: String,
    wire_source: Option<(TerminalId, DesignationPart)>,
    wire_target: Option<(TerminalId, DesignationPart)>,
    wire_marking: String,
    wire_to_link: Option<ConductorId>,
    list_tab: usize,
    renumber_start: String,
    locate: Option<RepresentationId>,
    list_selected: Option<RepresentationTarget>,
    connector: connectors::ConnectorDraft,
    kind: connectors::KindDraft,
}

impl PhysicalEditor {
    pub fn open(&mut self, tab: PhysicalTab) {
        self.tab = tab;
        self.open = true;
    }
    fn refresh(&mut self, session: &Session) {
        let Some(project) = session.project() else {
            self.cache = PhysicalCache::default();
            return;
        };
        let key = (project.id(), session.revision);
        if self.cache.key == Some(key) {
            return;
        }
        self.cache = PhysicalCache {
            key: Some(key),
            devices: project
                .devices()
                .map(|d| (d.id(), d.designation().to_string()))
                .collect(),
            strips: project.terminal_strips().cloned().collect(),
            terminals: project.terminals().cloned().collect(),
            points: project.connection_points().cloned().collect(),
            cables: project.cables().cloned().collect(),
            cores: project.cable_cores().cloned().collect(),
            wires: project.conductors().cloned().collect(),
            bridges: project.terminal_bridges().cloned().collect(),
            connectors: project.connectors().cloned().collect(),
            matings: project.connector_matings().cloned().collect(),
            lists: project.physical_lists(),
        };
        self.reseed();
    }
    /// Re-seeds edit drafts from the current Core state after any revision
    /// change (edit, Undo, Redo), so a stale text field never re-applies an
    /// undone value. Vanished selections are cleared.
    fn reseed(&mut self) {
        match self
            .selected_strip
            .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
        {
            Some(s) => self.strip_name = s.designation().as_str().into(),
            None => self.selected_strip = None,
        }
        match self
            .selected_terminal
            .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
        {
            Some(t) => self.terminal_name = t.designation().as_str().into(),
            None => self.selected_terminal = None,
        }
        match self
            .selected_cable
            .and_then(|id| self.cache.cables.iter().find(|x| x.id() == id))
            .cloned()
        {
            Some(cable) => self.seed_cable(&cable),
            None => self.selected_cable = None,
        }
        match self
            .selected_core
            .and_then(|id| self.cache.cores.iter().find(|x| x.id() == id))
            .cloned()
        {
            Some(core) => self.seed_core(&core),
            None => self.selected_core = None,
        }
        match self
            .selected_wire
            .and_then(|id| self.cache.wires.iter().find(|w| w.id() == id))
        {
            Some(w) => {
                self.wire_edit_name = w.designation().as_str().into();
                self.wire_edit_section = w.cross_section().get().to_string();
            }
            None => self.selected_wire = None,
        }
        if self.selected_connection.is_some_and(|id| {
            !self
                .cache
                .lists
                .assignments
                .iter()
                .any(|a| a.connection == id)
        }) {
            self.selected_connection = None;
        }
        self.reseed_connectors();
    }
    fn seed_cable(&mut self, cable: &Cable) {
        self.cable_name = cable.designation().as_str().into();
        self.cable_description = cable
            .description()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.cable_manufacturer = cable
            .manufacturer()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.cable_catalog = cable
            .catalog_code()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.cable_count = cable.nominal_core_count().to_string();
        self.cable_kind_index = match cable.kind() {
            CableKind::Unspecified => 0,
            CableKind::Power => 1,
            CableKind::Control => 2,
            CableKind::Instrumentation => 3,
            CableKind::Data => 4,
        };
    }
    fn seed_core(&mut self, core: &CableCore) {
        self.core_name = core.identification().as_str().into();
        self.core_marking = core
            .marking()
            .identification()
            .map(|v| v.as_str().to_owned())
            .unwrap_or_default();
        self.core_section = core
            .cross_section()
            .map(|v| v.get().to_string())
            .unwrap_or_default();
    }
    fn apply(&mut self, session: &mut Session, edit: Result<PhysicalEdit>) {
        match edit.and_then(|edit| session.dispatch_physical(edit)) {
            Ok(true) => {
                self.feedback = "Alteração registrada no histórico.".into();
                self.cache.key = None;
            }
            Ok(false) => self.feedback = "Nenhuma alteração.".into(),
            Err(error) => self.feedback = error.to_string(),
        }
    }
    pub fn show(&mut self, session: &mut Session, ctx: &Context) {
        if !self.open {
            return;
        }
        self.refresh(session);
        let mut open = self.open;
        // On small screens or high DPI scales the window (title bar included)
        // must fit the screen; it is never larger than the available area.
        let screen = ctx.content_rect().size();
        // Sizes are of the content area; title bar and frame add ~50 points.
        let fit = egui::vec2((screen.x - 40.).max(420.), (screen.y - 80.).max(240.));
        egui::Window::new("Engenharia física")
            .open(&mut open)
            .default_size(egui::vec2(850., 610.).min(fit))
            .min_size(egui::vec2(420., 240.))
            .max_size(fit)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (tab, label) in [
                        (PhysicalTab::Strips, "Réguas e bornes"),
                        (PhysicalTab::Connectors, "Conectores"),
                        (PhysicalTab::Cables, "Cabos e veias"),
                        (PhysicalTab::Wiring, "Condutores"),
                        (PhysicalTab::Lists, "Listas técnicas"),
                    ] {
                        if ui.selectable_label(self.tab == tab, label).clicked() {
                            self.tab = tab;
                        }
                    }
                });
                ui.separator();
                // A resizable egui window grows to the content measured on the
                // previous frame, and its frame/background cover only what is
                // allocated in this Ui. So the feedback line is laid out from
                // the bottom and the body fills exactly the remaining space,
                // both allocated here (panels' show_inside would not allocate).
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    if !self.feedback.is_empty() {
                        ui.label(&self.feedback);
                        ui.separator();
                    }
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        match self.tab {
                            // The technical lists are already virtualized scroll areas.
                            PhysicalTab::Lists => self.lists_ui(session, ui),
                            tab => {
                                ScrollArea::both()
                                    .id_salt(("physical_body", tab as u8))
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| match tab {
                                        PhysicalTab::Strips => self.strips_ui(session, ui),
                                        PhysicalTab::Connectors => self.connectors_ui(session, ui),
                                        PhysicalTab::Cables => self.cables_ui(session, ui),
                                        _ => self.wiring_ui(session, ui),
                                    });
                            }
                        }
                    });
                });
            });
        self.open = open;
    }

    fn strips_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        let Some(project_id) = session.project().map(Project::id) else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Nova régua");
            ui.text_edit_singleline(&mut self.new_strip);
            if ui.button("Criar régua").clicked() {
                let edit = DesignationPart::new(&self.new_strip)
                    .map(|name| PhysicalEdit::UpsertStrip(TerminalStrip::new(project_id, name)));
                self.apply(session, edit);
                self.new_strip.clear();
            }
        });
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("strip_choice")
                .selected_text(
                    self.selected_strip
                        .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
                        .map(|s| s.designation().as_str())
                        .unwrap_or("Selecione régua"),
                )
                .show_ui(ui, |ui| {
                    for strip in &self.cache.strips {
                        if ui
                            .selectable_value(
                                &mut self.selected_strip,
                                Some(strip.id()),
                                strip.designation().as_str(),
                            )
                            .clicked()
                        {
                            self.strip_name = strip.designation().as_str().into();
                            self.selected_terminal = None;
                            self.bridge_source = None;
                            self.bridge_target = None;
                        }
                    }
                });
            if let Some(strip) = self
                .selected_strip
                .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
                .cloned()
            {
                ui.text_edit_singleline(&mut self.strip_name);
                if ui.button("Salvar identificação").clicked() {
                    self.apply(
                        session,
                        DesignationPart::new(&self.strip_name).map(|name| {
                            PhysicalEdit::UpsertStrip(strip.clone().with_designation(name))
                        }),
                    );
                }
                if ui.button("Excluir régua").clicked() {
                    self.apply(session, Ok(PhysicalEdit::DeleteStrip(strip.id())));
                }
            }
        });
        ui.separator();
        let selected = self
            .selected_strip
            .and_then(|id| self.cache.strips.iter().find(|s| s.id() == id))
            .cloned();
        if let Some(strip) = selected {
            ui.strong(format!("Bornes da régua {}", strip.designation().as_str()));
            let members = strip.terminals().to_vec();
            ScrollArea::vertical().max_height(160.).show(ui, |ui| {
                for (index, id) in members.iter().enumerate() {
                    let label = self
                        .cache
                        .lists
                        .terminals
                        .iter()
                        .find(|t| t.terminal == *id)
                        .map(|t| {
                            format!(
                                "{}{} · dono {} · lados {} · {} conexões · {} pontes · {} condutores",
                                t.designation,
                                if t.kind == TerminalKind::Generic {
                                    String::new()
                                } else {
                                    format!(" [{}]", t.kind.label())
                                },
                                if t.owner.is_empty() { "-" } else { &t.owner },
                                if t.positions.is_empty() {
                                    "-".into()
                                } else {
                                    t.positions.join("/")
                                },
                                t.connections.len(),
                                t.bridges.len(),
                                t.conductors.len()
                            )
                        })
                        .unwrap_or_else(|| "?".into());
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                self.selected_terminal == Some(*id),
                                format!("{} · {label}", index + 1),
                            )
                            .clicked()
                        {
                            self.selected_terminal = Some(*id);
                            self.terminal_name = self
                                .cache
                                .terminals
                                .iter()
                                .find(|t| t.id() == *id)
                                .map(|t| t.designation().as_str().to_owned())
                                .unwrap_or_default();
                            session
                                .select(Selection::Electrical(RepresentationTarget::Terminal(*id)));
                        }
                        if ui.small_button("Localizar").clicked() {
                            self.request_locate(session, RepresentationTarget::Terminal(*id));
                        }
                        if ui.small_button("↑").clicked() && index > 0 {
                            let mut order = members.clone();
                            order.swap(index, index - 1);
                            self.apply(
                                session,
                                Ok(PhysicalEdit::UpsertStrip(
                                    strip.clone().with_terminals(order),
                                )),
                            );
                        }
                        if ui.small_button("↓").clicked() && index + 1 < members.len() {
                            let mut order = members.clone();
                            order.swap(index, index + 1);
                            self.apply(
                                session,
                                Ok(PhysicalEdit::UpsertStrip(
                                    strip.clone().with_terminals(order),
                                )),
                            );
                        }
                        if ui.small_button("Remover da régua").clicked() {
                            let mut order = members.clone();
                            order.remove(index);
                            self.apply(
                                session,
                                Ok(PhysicalEdit::UpsertStrip(
                                    strip.clone().with_terminals(order),
                                )),
                            );
                        }
                    });
                }
            });
            let available = self
                .cache
                .terminals
                .iter()
                .filter(|t| {
                    !self
                        .cache
                        .strips
                        .iter()
                        .any(|s| s.terminals().contains(&t.id()))
                })
                .cloned()
                .collect::<Vec<_>>();
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("terminal_to_attach")
                    .selected_text(
                        self.attach_candidate
                            .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
                            .map(|t| t.designation().as_str())
                            .unwrap_or("Borne existente"),
                    )
                    .show_ui(ui, |ui| {
                        for terminal in &available {
                            ui.selectable_value(
                                &mut self.attach_candidate,
                                Some(terminal.id()),
                                terminal.designation().as_str(),
                            );
                        }
                    });
                if ui.button("Adicionar borne").clicked()
                    && let Some(id) = self.attach_candidate
                {
                    let mut order = members.clone();
                    order.push(id);
                    self.apply(
                        session,
                        Ok(PhysicalEdit::UpsertStrip(
                            strip.clone().with_terminals(order),
                        )),
                    );
                }
            });
            ui.horizontal(|ui| {
                ui.label("Numerar a partir de");
                ui.add(egui::TextEdit::singleline(&mut self.renumber_start).desired_width(50.));
                if ui
                    .button("Renumerar sequencialmente")
                    .on_hover_text(
                        "Atribui números consecutivos na ordem da régua em uma única transação",
                    )
                    .clicked()
                {
                    let edit = (|| {
                        let start = if self.renumber_start.trim().is_empty() {
                            1
                        } else {
                            self.renumber_start
                                .trim()
                                .parse::<u32>()
                                .map_err(|_| DomainError::InvalidNumber("número inicial"))?
                        };
                        let edits = renumber_edits(&self.cache.terminals, &members, start)?;
                        Ok(PhysicalEdit::Batch(edits))
                    })();
                    self.apply(session, edit);
                }
                if ui.small_button("Localizar régua").clicked() {
                    self.request_locate(session, RepresentationTarget::TerminalStrip(strip.id()));
                }
            });
            if let Some(terminal) = self
                .selected_terminal
                .filter(|id| members.contains(id))
                .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
                .cloned()
            {
                ui.horizontal(|ui| {
                    ui.label("Numeração do borne selecionado");
                    ui.add(egui::TextEdit::singleline(&mut self.terminal_name).desired_width(70.));
                    if ui.button("Renomear borne").clicked() {
                        let edit = DesignationPart::new(&self.terminal_name).map(|name| {
                            PhysicalEdit::UpsertTerminal(terminal.clone().with_designation(name))
                        });
                        self.apply(session, edit);
                    }
                });
                ui.separator();
                self.terminal_kind_ui(session, ui, &terminal);
            }
            ui.separator();
            self.bridge_ui(session, ui, &strip);
        } else {
            ui.weak("Selecione uma régua para ordenar bornes e cadastrar pontes.");
        }
        ui.separator();
        ui.strong("Cadastrar borne individual");
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("terminal_owner")
                .selected_text(
                    self.selected_device
                        .and_then(|id| self.cache.devices.iter().find(|(d, _)| *d == id))
                        .map(|(_, name)| name.as_str())
                        .unwrap_or("Dispositivo proprietário"),
                )
                .show_ui(ui, |ui| {
                    for (id, name) in &self.cache.devices {
                        ui.selectable_value(&mut self.selected_device, Some(*id), name);
                    }
                });
            ui.label("Número");
            ui.text_edit_singleline(&mut self.new_terminal_name);
            ui.label("Lado/posição");
            ui.text_edit_singleline(&mut self.terminal_position);
            if ui.button("Criar borne").clicked() {
                let edit = (|| {
                    let device = self.selected_device.ok_or(DomainError::InvalidRelation(
                        "selecione dispositivo proprietário",
                    ))?;
                    let name = DesignationPart::new(&self.new_terminal_name)?;
                    let mut terminal =
                        Terminal::new(project_id, TerminalOwner::Device(device), name);
                    if !self.terminal_position.trim().is_empty() {
                        terminal = terminal
                            .with_positions(vec![DesignationPart::new(&self.terminal_position)?])?;
                    }
                    Ok(PhysicalEdit::UpsertTerminal(terminal))
                })();
                self.apply(session, edit);
            }
        });
        // Only the member picked in the strip list can be deleted here; the
        // attach combo has its own state, and stale ids are not offered.
        if let Some(row) = self
            .selected_terminal
            .and_then(|id| self.cache.lists.terminals.iter().find(|t| t.terminal == id))
            .cloned()
            && ui
                .button(format!("Excluir borne {}", terminal_text(&row)))
                .clicked()
        {
            self.apply(session, Ok(PhysicalEdit::DeleteTerminal(row.terminal)));
        }
    }

    fn bridge_ui(&mut self, session: &mut Session, ui: &mut Ui, strip: &TerminalStrip) {
        ui.strong("Pontes elétricas explícitas");
        let points = self
            .cache
            .points
            .iter()
            .filter(|p| {
                p.terminal_id()
                    .is_some_and(|id| strip.terminals().contains(&id))
            })
            .cloned()
            .collect::<Vec<_>>();
        ui.horizontal(|ui| {
            for (key, selected, label) in [
                ("bridge_source", &mut self.bridge_source, "Origem"),
                ("bridge_target", &mut self.bridge_target, "Destino"),
            ] {
                ComboBox::from_id_salt(key)
                    .selected_text(
                        selected
                            .and_then(|id| points.iter().find(|p| p.id() == id))
                            .map(|p| p.designation().as_str())
                            .unwrap_or(label),
                    )
                    .show_ui(ui, |ui| {
                        for point in &points {
                            let terminal = point
                                .terminal_id()
                                .and_then(|id| self.cache.terminals.iter().find(|t| t.id() == id))
                                .map(|t| t.designation().as_str())
                                .unwrap_or("?");
                            ui.selectable_value(
                                selected,
                                Some(point.id()),
                                format!("{terminal}:{}", point.designation().as_str()),
                            );
                        }
                    });
            }
            ui.checkbox(&mut self.bridge_kind, "Fixa");
            if ui.button("Criar ponte").clicked() {
                let edit = self
                    .bridge_source
                    .zip(self.bridge_target)
                    .map(|(source, target)| PhysicalEdit::CreateBridge {
                        strip: strip.id(),
                        source,
                        target,
                        kind: if self.bridge_kind {
                            TerminalBridgeKind::Fixed
                        } else {
                            TerminalBridgeKind::Inserted
                        },
                    })
                    .ok_or(DomainError::InvalidRelation(
                        "selecione os dois pontos da ponte",
                    ));
                self.apply(session, edit);
            }
        });
        for bridge in self
            .cache
            .bridges
            .iter()
            .filter(|b| b.strip_id() == strip.id())
            .cloned()
            .collect::<Vec<_>>()
        {
            ui.horizontal(|ui| {
                ui.label(format!("{:?} · {}", bridge.kind(), bridge.id()));
                if ui.small_button("Remover ponte").clicked() {
                    self.apply(session, Ok(PhysicalEdit::DeleteBridge(bridge.id())));
                }
            });
        }
    }

    fn cables_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        let Some(project_id) = session.project().map(Project::id) else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Novo cabo");
            ui.text_edit_singleline(&mut self.new_cable_name);
            ui.label("Veias nominais");
            ui.add(egui::TextEdit::singleline(&mut self.new_cable_count).desired_width(65.));
            if ui.button("Criar cabo").clicked() {
                let edit = (|| {
                    let name = DesignationPart::new(&self.new_cable_name)?;
                    let count = self
                        .new_cable_count
                        .parse::<u32>()
                        .map_err(|_| DomainError::InvalidNumber("quantidade de veias"))?;
                    Ok(PhysicalEdit::UpsertCable(Cable::new(
                        project_id,
                        name,
                        CableKind::Unspecified,
                        count,
                    )?))
                })();
                self.apply(session, edit);
            }
        });
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("cable_choice")
                .selected_text(
                    self.selected_cable
                        .and_then(|id| self.cache.cables.iter().find(|c| c.id() == id))
                        .map(|c| c.designation().as_str())
                        .unwrap_or("Selecione cabo"),
                )
                .show_ui(ui, |ui| {
                    for cable in &self.cache.cables {
                        if ui
                            .selectable_value(
                                &mut self.selected_cable,
                                Some(cable.id()),
                                cable.designation().as_str(),
                            )
                            .clicked()
                        {
                            self.cable_name = cable.designation().as_str().into();
                            self.cable_description = cable
                                .description()
                                .map(|v| v.as_str().to_owned())
                                .unwrap_or_default();
                            self.cable_manufacturer = cable
                                .manufacturer()
                                .map(|v| v.as_str().to_owned())
                                .unwrap_or_default();
                            self.cable_catalog = cable
                                .catalog_code()
                                .map(|v| v.as_str().to_owned())
                                .unwrap_or_default();
                            self.cable_count = cable.nominal_core_count().to_string();
                            self.cable_kind_index = match cable.kind() {
                                CableKind::Unspecified => 0,
                                CableKind::Power => 1,
                                CableKind::Control => 2,
                                CableKind::Instrumentation => 3,
                                CableKind::Data => 4,
                            };
                            self.selected_core = None;
                        }
                    }
                });
        });
        let selected = self
            .selected_cable
            .and_then(|id| self.cache.cables.iter().find(|c| c.id() == id))
            .cloned();
        let Some(cable) = selected else {
            ui.weak("Selecione um cabo para editar as propriedades e veias.");
            return;
        };
        egui::Grid::new("cable_fields")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Identificação");
                ui.text_edit_singleline(&mut self.cable_name);
                ui.end_row();
                ui.label("Descrição");
                ui.text_edit_singleline(&mut self.cable_description);
                ui.end_row();
                ui.label("Fabricante");
                ui.text_edit_singleline(&mut self.cable_manufacturer);
                ui.end_row();
                ui.label("Código de produto");
                ui.text_edit_singleline(&mut self.cable_catalog);
                ui.end_row();
                ui.label("Quantidade de veias");
                ui.text_edit_singleline(&mut self.cable_count);
                ui.end_row();
            });
        ComboBox::from_id_salt("cable_kind")
            .selected_text(
                [
                    "Não especificado",
                    "Potência",
                    "Controle",
                    "Instrumentação",
                    "Dados",
                ][self.cable_kind_index.min(4)],
            )
            .show_ui(ui, |ui| {
                for (index, label) in [
                    "Não especificado",
                    "Potência",
                    "Controle",
                    "Instrumentação",
                    "Dados",
                ]
                .iter()
                .enumerate()
                {
                    ui.selectable_value(&mut self.cable_kind_index, index, *label);
                }
            });
        ui.horizontal(|ui| {
            if ui.button("Salvar cabo").clicked() {
                let edit = (|| {
                    let count = self
                        .cable_count
                        .parse::<u32>()
                        .map_err(|_| DomainError::InvalidNumber("quantidade de veias"))?;
                    let kind = [
                        CableKind::Unspecified,
                        CableKind::Power,
                        CableKind::Control,
                        CableKind::Instrumentation,
                        CableKind::Data,
                    ][self.cable_kind_index.min(4)];
                    let updated = cable
                        .clone()
                        .with_designation(DesignationPart::new(&self.cable_name)?)
                        .with_description_option(optional_text(&self.cable_description)?)
                        .with_manufacturer_option(optional_text(&self.cable_manufacturer)?)
                        .with_catalog_code_option(optional_text(&self.cable_catalog)?)
                        .with_kind(kind)
                        .with_nominal_core_count(count)?;
                    Ok(PhysicalEdit::UpsertCable(updated))
                })();
                self.apply(session, edit);
            }
            if ui.button("Excluir cabo").clicked() {
                self.apply(session, Ok(PhysicalEdit::DeleteCable(cable.id())));
            }
        });
        ui.separator();
        ui.strong("Veias cadastradas");
        let cores = self
            .cache
            .cores
            .iter()
            .filter(|c| c.cable_id() == cable.id())
            .cloned()
            .collect::<Vec<_>>();
        ScrollArea::vertical().max_height(180.).show(ui, |ui| {
            for core in &cores {
                let status = core
                    .conductor_id()
                    .map(|id| format!("usada · {id}"))
                    .unwrap_or_else(|| "livre".into());
                if ui
                    .selectable_label(
                        self.selected_core == Some(core.id()),
                        format!("{} · {status}", core.identification().as_str()),
                    )
                    .clicked()
                {
                    self.selected_core = Some(core.id());
                    self.core_name = core.identification().as_str().into();
                    self.core_marking = core
                        .marking()
                        .identification()
                        .map(|v| v.as_str().to_owned())
                        .unwrap_or_default();
                    self.core_section = core
                        .cross_section()
                        .map(|v| v.get().to_string())
                        .unwrap_or_default();
                    self.selected_wire = core.conductor_id();
                    session.select(Selection::Electrical(RepresentationTarget::CableCore(
                        core.id(),
                    )));
                }
            }
        });
        if let Some(core) = self
            .selected_core
            .and_then(|id| cores.iter().find(|c| c.id() == id))
            .cloned()
        {
            ui.horizontal(|ui| {
                ui.label("Designação");
                ui.text_edit_singleline(&mut self.core_name);
                ui.label("Marcação");
                ui.text_edit_singleline(&mut self.core_marking);
                ui.label("Seção mm²");
                ui.add(egui::TextEdit::singleline(&mut self.core_section).desired_width(65.));
            });
            ui.horizontal(|ui| {
                if ui.button("Salvar veia").clicked() {
                    let edit = (|| {
                        let name = DesignationPart::new(&self.core_name)?;
                        let marking = if self.core_marking.trim().is_empty() {
                            core.marking().clone()
                        } else {
                            ConductorMarking::new(
                                core.marking().color_value(),
                                Some(DesignationPart::new(&self.core_marking)?),
                            )?
                        };
                        let area = optional_section(&self.core_section)?;
                        Ok(PhysicalEdit::UpsertCore(
                            core.clone()
                                .with_identification(name)
                                .with_marking(marking)
                                .with_cross_section_option(area),
                        ))
                    })();
                    self.apply(session, edit);
                }
                if ui.button("Excluir veia").clicked() {
                    self.apply(session, Ok(PhysicalEdit::DeleteCore(core.id())));
                }
            });
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("core_wire")
                    .selected_text(
                        self.selected_wire
                            .and_then(|id| self.cache.wires.iter().find(|w| w.id() == id))
                            .map(|w| w.designation().as_str())
                            .unwrap_or("Condutor físico"),
                    )
                    .show_ui(ui, |ui| {
                        for wire in &self.cache.wires {
                            ui.selectable_value(
                                &mut self.selected_wire,
                                Some(wire.id()),
                                wire.designation().as_str(),
                            );
                        }
                    });
                if ui.button("Atribuir veia").clicked() {
                    self.apply(
                        session,
                        self.selected_wire
                            .map(|id| PhysicalEdit::SetCoreConductor {
                                core: core.id(),
                                conductor: Some(id),
                            })
                            .ok_or(DomainError::InvalidRelation("selecione condutor")),
                    );
                }
                if ui.button("Liberar veia").clicked() {
                    self.apply(
                        session,
                        Ok(PhysicalEdit::SetCoreConductor {
                            core: core.id(),
                            conductor: None,
                        }),
                    );
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Nova veia");
                ui.text_edit_singleline(&mut self.core_name);
                ui.label("Seção mm² (opcional)");
                ui.add(egui::TextEdit::singleline(&mut self.core_section).desired_width(65.));
                if ui.button("Adicionar veia").clicked() {
                    let edit = (|| {
                        let name = DesignationPart::new(&self.core_name)?;
                        let mut core = CableCore::new(
                            project_id,
                            cable.id(),
                            name.clone(),
                            ConductorMarking::identified(name),
                        );
                        if let Some(area) = optional_section(&self.core_section)? {
                            core = core.with_cross_section(area);
                        }
                        Ok(PhysicalEdit::UpsertCore(core))
                    })();
                    self.apply(session, edit);
                }
            });
        }
    }
    /// Preselects a logical connection (for example the selected drawn route).
    pub fn focus_connection(&mut self, id: ConnectionId) {
        if self.selected_connection != Some(id) {
            self.selected_connection = Some(id);
            self.wire_source = None;
            self.wire_target = None;
            self.wire_to_link = None;
        }
    }
    /// Graphic occurrence requested by the editor, consumed by the app navigator.
    pub fn take_locate(&mut self) -> Option<RepresentationId> {
        self.locate.take()
    }
    fn request_locate(&mut self, session: &Session, target: RepresentationTarget) {
        match session
            .project()
            .and_then(|p| p.physical_occurrence(target))
        {
            Some(id) => self.locate = Some(id),
            None => self.feedback = "Objeto sem ocorrência gráfica explícita.".into(),
        }
    }
    fn termination_picker(
        ui: &mut Ui,
        salt: &str,
        terminals: &[TerminalListRow],
        value: &mut Option<(TerminalId, DesignationPart)>,
    ) {
        let label = value
            .as_ref()
            .and_then(|(id, pos)| {
                terminals
                    .iter()
                    .find(|t| t.terminal == *id)
                    .map(|t| format!("{} / {}", terminal_text(t), pos.as_str()))
            })
            .unwrap_or_else(|| "Selecione borne/posição".into());
        ComboBox::from_id_salt(salt)
            .width(220.)
            .selected_text(label)
            .show_ui(ui, |ui| {
                for row in terminals {
                    for position in &row.positions {
                        let Ok(part) = DesignationPart::new(position) else {
                            continue;
                        };
                        let option = Some((row.terminal, part));
                        let text = format!("{} / {position}", terminal_text(row));
                        ui.selectable_value(value, option, text);
                    }
                }
            });
    }
    fn wiring_ui(&mut self, session: &mut Session, ui: &mut Ui) {
        let Some(project) = session.project() else {
            return;
        };
        let project_id = project.id();
        ui.weak(
            "Rota gráfica, conexão lógica e condutor físico são camadas distintas: \
             associar um condutor não altera a topologia nem as redes.",
        );
        let rows = &self.cache.lists.assignments;
        let mut picked = None;
        ui.strong(format!("Conexões lógicas ({})", rows.len()));
        let height = ui.spacing().interact_size.y;
        ScrollArea::vertical()
            .id_salt("wiring_connections")
            .max_height(150.)
            .show_rows(ui, height, rows.len(), |ui, range| {
                for row in &rows[range] {
                    let state = if row.bridge {
                        "ponte de régua".to_owned()
                    } else if row.conductors.is_empty() {
                        "sem condutor".to_owned()
                    } else {
                        format!("{} condutor(es)", row.conductors.len())
                    };
                    let drawn = if row.drawn { " · desenhada" } else { "" };
                    let text = format!("{} → {} · {state}{drawn}", row.source, row.target);
                    let selected = self.selected_connection == Some(row.connection);
                    if ui.selectable_label(selected, text).clicked() {
                        picked = Some(row.connection);
                    }
                }
            });
        if let Some(id) = picked {
            self.focus_connection(id);
        }
        let mut edit: Option<Result<PhysicalEdit>> = None;
        let mut locate = None;
        let selected = self
            .selected_connection
            .and_then(|id| {
                self.cache
                    .lists
                    .assignments
                    .iter()
                    .find(|r| r.connection == id)
            })
            .cloned();
        let mut ends = (None, None);
        if let Some(row) = &selected {
            let terminal_of = |p: ConnectionPointId| {
                project
                    .connection_point(p)
                    .and_then(ConnectionPoint::terminal_id)
            };
            ends = (terminal_of(row.source_point), terminal_of(row.target_point));
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong("Conexão selecionada");
                if ui.small_button("Localizar").clicked() {
                    locate = Some(RepresentationTarget::Connection(row.connection));
                }
            });
            let describe = |t: Option<TerminalId>| {
                t.map(|t| project.terminal_label(t))
                    .unwrap_or_else(|| "sem borne físico".into())
            };
            egui::Grid::new("wiring_endpoints")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Origem");
                    ui.label(format!("{} · borne {}", row.source, describe(ends.0)));
                    ui.end_row();
                    ui.label("Destino");
                    ui.label(format!("{} · borne {}", row.target, describe(ends.1)));
                    ui.end_row();
                });
            if row.bridge {
                ui.weak("Ponte explícita de régua: continuidade interna, sem condutor externo.");
            } else {
                for wire in row.conductors.iter().filter_map(|id| {
                    self.cache
                        .lists
                        .conductors
                        .iter()
                        .find(|w| w.conductor == *id)
                }) {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} · {} → {} · {} mm² · {}",
                            wire.designation,
                            wire.source,
                            wire.target,
                            wire.cross_section_mm2,
                            wire.cable_label.as_deref().unwrap_or("fio individual")
                        ));
                        if ui.small_button("Remover associação").clicked() {
                            edit = Some(Ok(PhysicalEdit::SetConductorConnection {
                                conductor: wire.conductor,
                                link: None,
                            }));
                        }
                    });
                }
                match ends {
                    (Some(a), Some(b)) => {
                        let compatible: Vec<_> = self
                            .cache
                            .lists
                            .conductors
                            .iter()
                            .filter(|w| {
                                w.connection.is_none()
                                    && ((w.source_terminal, w.target_terminal) == (a, b)
                                        || (w.source_terminal, w.target_terminal) == (b, a))
                            })
                            .collect();
                        ui.horizontal(|ui| {
                            ComboBox::from_id_salt("wire_to_link")
                                .selected_text(
                                    self.wire_to_link
                                        .and_then(|id| {
                                            compatible.iter().find(|w| w.conductor == id)
                                        })
                                        .map(|w| w.designation.as_str())
                                        .unwrap_or("Condutor compatível livre"),
                                )
                                .show_ui(ui, |ui| {
                                    for w in &compatible {
                                        ui.selectable_value(
                                            &mut self.wire_to_link,
                                            Some(w.conductor),
                                            format!(
                                                "{} · {} → {}",
                                                w.designation, w.source, w.target
                                            ),
                                        );
                                    }
                                });
                            if ui.button("Associar condutor").clicked() {
                                edit = Some(
                                    self.wire_to_link
                                        .and_then(|id| {
                                            compatible.iter().find(|w| w.conductor == id)
                                        })
                                        .map(|w| PhysicalEdit::SetConductorConnection {
                                            conductor: w.conductor,
                                            link: Some(link_for(row, ends, w.source_terminal)),
                                        })
                                        .ok_or(DomainError::InvalidRelation(
                                            "selecione um condutor compatível",
                                        )),
                                );
                            }
                        });
                    }
                    _ => {
                        ui.weak(
                            "Os dois pontos funcionais precisam de borne físico explícito \
                             para receber um condutor.",
                        );
                    }
                }
            }
        }
        ui.separator();
        ui.strong("Cadastrar condutor físico");
        egui::Grid::new("wire_fields")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Designação");
                ui.text_edit_singleline(&mut self.wire_name);
                ui.end_row();
                ui.label("Seção nominal (mm²)");
                ui.text_edit_singleline(&mut self.wire_section);
                ui.end_row();
                ui.label("Marcação (opcional)");
                ui.text_edit_singleline(&mut self.wire_marking);
                ui.end_row();
                ui.label("Origem");
                Self::termination_picker(
                    ui,
                    "wire_source",
                    &self.cache.lists.terminals,
                    &mut self.wire_source,
                );
                ui.end_row();
                ui.label("Destino");
                Self::termination_picker(
                    ui,
                    "wire_target",
                    &self.cache.lists.terminals,
                    &mut self.wire_target,
                );
                ui.end_row();
            });
        let linkable = selected.as_ref().filter(|r| !r.bridge).zip(match ends {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        });
        ui.horizontal(|ui| {
            if let Some((_, (a, b))) = linkable
                && ui.button("Usar bornes da conexão").clicked()
            {
                let first = |t: TerminalId| {
                    self.cache
                        .lists
                        .terminals
                        .iter()
                        .find(|r| r.terminal == t)
                        .and_then(|r| r.positions.first())
                        .and_then(|p| DesignationPart::new(p).ok())
                        .map(|p| (t, p))
                };
                self.wire_source = first(a);
                self.wire_target = first(b);
                if self.wire_source.is_none() || self.wire_target.is_none() {
                    self.feedback = "Borne sem posição de conexão cadastrada.".into();
                }
            }
            let create = ui.button("Criar condutor").clicked();
            let create_link = linkable.is_some() && ui.button("Criar e associar").clicked();
            if create || create_link {
                let built = self.build_wire(project_id);
                edit = Some(built.map(|wire| match linkable.filter(|_| create_link) {
                    None => PhysicalEdit::UpsertConductor(wire),
                    Some((row, _)) => {
                        let link = link_for(row, ends, wire.endpoints().0.terminal_id());
                        let conductor = wire.id();
                        PhysicalEdit::Batch(vec![
                            PhysicalEdit::UpsertConductor(wire),
                            PhysicalEdit::SetConductorConnection {
                                conductor,
                                link: Some(link),
                            },
                        ])
                    }
                }));
            }
        });
        ui.separator();
        self.conductor_table(ui, &mut edit, &mut locate);
        if let Some(id) = picked {
            session.select(Selection::Electrical(RepresentationTarget::Connection(id)));
        }
        if let Some(target) = locate {
            self.request_locate(session, target);
        }
        if let Some(edit) = edit {
            self.apply(session, edit);
        }
    }
    fn build_wire(&self, project_id: ProjectId) -> Result<Conductor> {
        let name = DesignationPart::new(&self.wire_name)?;
        let area = optional_section(&self.wire_section)?.ok_or(DomainError::InvalidNumber(
            "seção nominal obrigatória em mm²",
        ))?;
        let marking = if self.wire_marking.trim().is_empty() {
            ConductorMarking::identified(name.clone())
        } else {
            ConductorMarking::identified(DesignationPart::new(&self.wire_marking)?)
        };
        let (source, target) = self
            .wire_source
            .clone()
            .zip(self.wire_target.clone())
            .ok_or(DomainError::InvalidRelation(
                "selecione origem e destino físicos",
            ))?;
        Conductor::new(
            project_id,
            name,
            PhysicalTermination::new(source.0, source.1),
            PhysicalTermination::new(target.0, target.1),
            area,
            marking,
        )
    }
    fn conductor_table(
        &mut self,
        ui: &mut Ui,
        edit: &mut Option<Result<PhysicalEdit>>,
        locate: &mut Option<RepresentationTarget>,
    ) {
        let wires = &self.cache.lists.conductors;
        ui.strong(format!("Condutores cadastrados ({})", wires.len()));
        let height = ui.spacing().interact_size.y;
        let mut picked = None;
        ScrollArea::vertical()
            .id_salt("wire_table")
            .max_height(140.)
            .show_rows(ui, height, wires.len(), |ui, range| {
                for w in &wires[range] {
                    let text = format!(
                        "{} · {} → {} · {} mm² · {} · {}",
                        w.designation,
                        w.source,
                        w.target,
                        w.cross_section_mm2,
                        w.cable_label.as_deref().unwrap_or("fio individual"),
                        if w.connection.is_some() {
                            "associado"
                        } else {
                            "sem conexão"
                        }
                    );
                    if ui
                        .selectable_label(self.selected_wire == Some(w.conductor), text)
                        .clicked()
                    {
                        picked = Some(w.conductor);
                    }
                }
            });
        if let Some(id) = picked {
            self.selected_wire = Some(id);
            if let Some(w) = wires.iter().find(|w| w.conductor == id) {
                self.wire_edit_name = w.designation.clone();
                self.wire_edit_section = w.cross_section_mm2.to_string();
            }
        }
        let Some(wire) = self
            .selected_wire
            .and_then(|id| self.cache.wires.iter().find(|w| w.id() == id))
            .cloned()
        else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Designação");
            ui.add(egui::TextEdit::singleline(&mut self.wire_edit_name).desired_width(80.));
            ui.label("Seção mm²");
            ui.add(egui::TextEdit::singleline(&mut self.wire_edit_section).desired_width(60.));
            if ui.button("Salvar designação/seção").clicked() {
                *edit = Some((|| {
                    let area = optional_section(&self.wire_edit_section)?.ok_or(
                        DomainError::InvalidNumber("seção nominal obrigatória em mm²"),
                    )?;
                    Ok(PhysicalEdit::UpsertConductor(
                        wire.clone()
                            .with_designation(DesignationPart::new(&self.wire_edit_name)?)
                            .with_cross_section(area),
                    ))
                })());
            }
            if ui.button("Excluir condutor").clicked() {
                *edit = Some(Ok(PhysicalEdit::DeleteConductor(wire.id())));
            }
            if ui.button("Localizar").clicked() {
                *locate = Some(RepresentationTarget::Conductor(wire.id()));
            }
        });
    }
    fn lists_ui(&mut self, session: &mut Session, ui: &mut Ui) {
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
fn terminal_text(row: &TerminalListRow) -> String {
    match (&row.strip_designation, &row.connector_designation) {
        (Some(group), _) | (None, Some(group)) => format!("{group}:{}", row.designation),
        (None, None) => format!("{}:{}", row.owner, row.designation),
    }
}
/// Orients the conductor link so that its source termination maps to the
/// connection point carrying the same physical terminal.
fn link_for(
    row: &WiringAssignmentRow,
    ends: (Option<TerminalId>, Option<TerminalId>),
    wire_source: TerminalId,
) -> ConductorConnection {
    let point = if ends.0 == Some(wire_source) {
        row.source_point
    } else {
        row.target_point
    };
    ConductorConnection::new(row.connection, point)
}

fn optional_text(value: &str) -> Result<Option<Text>> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Text::new(value).map(Some)
    }
}
fn optional_section(value: &str) -> Result<Option<CrossSectionMm2>> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        let number = value
            .parse::<f64>()
            .map_err(|_| DomainError::InvalidNumber("seção em mm²"))?;
        CrossSectionMm2::new(number).map(Some)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::{NavigatorTab, physical_tree};

    fn project() -> (Project, TerminalStripId, CableId) {
        let mut p = Project::new("Navegador").unwrap();
        let device = p
            .insert_device(Device::new(p.id(), Designation::new("K1").unwrap(), "relé").unwrap())
            .unwrap();
        let strip = TerminalStrip::new(p.id(), DesignationPart::new("X1").unwrap());
        let strip_id = strip.id();
        let mut edits = vec![PhysicalEdit::UpsertStrip(strip.clone())];
        let mut order = vec![];
        for n in ["7", "3", "9"] {
            let t = Terminal::new(
                p.id(),
                TerminalOwner::Device(device),
                DesignationPart::new(n).unwrap(),
            );
            order.push(t.id());
            edits.push(PhysicalEdit::UpsertTerminal(t));
        }
        edits.push(PhysicalEdit::UpsertStrip(strip.with_terminals(order)));
        let cable = Cable::new(
            p.id(),
            DesignationPart::new("W1").unwrap(),
            CableKind::Control,
            2,
        )
        .unwrap();
        let cable_id = cable.id();
        edits.push(PhysicalEdit::UpsertCable(cable));
        for n in ["1", "PE"] {
            edits.push(PhysicalEdit::UpsertCore(CableCore::new(
                p.id(),
                cable_id,
                DesignationPart::new(n).unwrap(),
                ConductorMarking::identified(DesignationPart::new(n).unwrap()),
            )));
        }
        p.edit_physical(PhysicalEdit::Batch(edits)).unwrap();
        (p, strip_id, cable_id)
    }

    #[test]
    fn navigator_tree_is_hierarchical_and_follows_strip_order() {
        let (p, strip, cable) = project();
        let terminals = physical_tree(&p, NavigatorTab::Terminals);
        assert_eq!(
            terminals[0].target,
            Some(RepresentationTarget::TerminalStrip(strip))
        );
        assert_eq!(terminals[0].depth, 0);
        let children: Vec<_> = terminals[1..4].iter().map(|r| r.label.clone()).collect();
        assert!(children[0].starts_with("1. 7") && children[1].starts_with("2. 3"));
        assert!(terminals[1..4].iter().all(|r| r.depth == 1));
        let cables = physical_tree(&p, NavigatorTab::Cables);
        assert_eq!(cables[0].target, Some(RepresentationTarget::Cable(cable)));
        assert_eq!(cables.len(), 3, "cable plus its two cores, no PE invented");
        assert!(
            cables[1..]
                .iter()
                .all(|r| r.depth == 1 && r.label.contains("livre"))
        );
    }

    /// Etapa 11: Régua → Borne → Níveis and Conector → Contatos.
    #[test]
    fn navigator_shows_levels_and_connector_contacts_hierarchically() {
        let (mut p, strip, _) = project();
        let order = p.terminal_strip(strip).unwrap().terminals().to_vec();
        let device = p.devices().next().unwrap().id();
        let multilevel = p
            .terminal(order[0])
            .unwrap()
            .clone()
            .with_kind(TerminalKind::MultiLevel)
            .with_levels(vec![
                TerminalLevel::new(
                    DesignationPart::new("1").unwrap(),
                    vec![DesignationPart::new("a").unwrap()],
                )
                .unwrap(),
                TerminalLevel::new(
                    DesignationPart::new("2").unwrap(),
                    vec![DesignationPart::new("b").unwrap()],
                )
                .unwrap(),
            ]);
        let contact = Terminal::new(
            p.id(),
            TerminalOwner::Device(device),
            DesignationPart::new("1").unwrap(),
        );
        let contact_id = contact.id();
        p.edit_physical(PhysicalEdit::Batch(vec![
            PhysicalEdit::UpsertTerminal(multilevel),
            PhysicalEdit::UpsertTerminal(contact),
            PhysicalEdit::UpsertConnector(
                Connector::new(p.id(), DesignationPart::new("X5").unwrap())
                    .with_gender(ConnectorGender::Male)
                    .with_device(Some(device))
                    .with_contacts(vec![contact_id]),
            ),
        ]))
        .unwrap();
        let rows = physical_tree(&p, NavigatorTab::Terminals);
        let labels: Vec<_> = rows.iter().map(|r| (r.depth, r.label.as_str())).collect();
        assert!(labels[1].1.starts_with("1. 7 [multinível]"), "{labels:?}");
        assert_eq!(labels[2], (2, "Nível 1 (a)"));
        assert_eq!(labels[3], (2, "Nível 2 (b)"));
        let connector = rows
            .iter()
            .position(|r| {
                r.label
                    .starts_with("Conector X5 · macho · 1 contatos (1 livres)")
            })
            .expect("connector row");
        assert_eq!(rows[connector].depth, 0);
        assert_eq!(
            rows[connector].target,
            Some(RepresentationTarget::Device(device))
        );
        assert_eq!(rows[connector + 1].depth, 1);
        assert_eq!(
            rows[connector + 1].target,
            Some(RepresentationTarget::Terminal(contact_id))
        );
        assert!(
            rows[connector + 1]
                .label
                .contains("Contato 1 · posições - · livre")
        );
        assert!(
            rows.iter()
                .all(|r| !r.label.starts_with("Bornes sem régua")),
            "contacts are listed under their connector, not as loose terminals"
        );
        let summary = physical_summary(&p, RepresentationTarget::Terminal(order[0]));
        assert!(
            summary
                .iter()
                .any(|(k, v)| k == "Especialização" && v == "multinível")
        );
        assert!(summary.iter().any(|(k, v)| k == "Nível 2" && v == "b"));
        let summary = physical_summary(&p, RepresentationTarget::Terminal(contact_id));
        assert!(
            summary
                .iter()
                .any(|(k, v)| k == "Conector" && v == "X5 · macho")
        );
    }

    #[test]
    fn specialization_draft_round_trips_and_rejects_conflicting_sides() {
        let (p, strip, _) = project();
        let id = p.terminal_strip(strip).unwrap().terminals()[0];
        let base = p.terminal(id).unwrap().clone();
        let mut draft = connectors::KindDraft::default();
        draft.kind = TerminalKind::FeedThrough;
        draft.positions = "1, 2".into();
        draft.side_a = "1".into();
        draft.side_b = "2".into();
        let t = connectors::terminal_from_draft(&base, &draft).unwrap();
        assert!(t.check_specialization().is_ok());
        assert_eq!(t.id(), id);
        draft.side_b = "1, 2".into();
        assert!(connectors::terminal_from_draft(&base, &draft).is_err());
    }

    #[test]
    fn sequential_renumbering_keeps_ids_and_is_one_undoable_batch() {
        let (mut p, strip, _) = project();
        let order = p.terminal_strip(strip).unwrap().terminals().to_vec();
        let terminals: Vec<_> = p.terminals().cloned().collect();
        let before = p.clone();
        let delta = p
            .edit_physical(PhysicalEdit::Batch(
                renumber_edits(&terminals, &order, 1).unwrap(),
            ))
            .unwrap();
        let names: Vec<_> = order
            .iter()
            .map(|id| p.terminal(*id).unwrap().designation().as_str().to_owned())
            .collect();
        assert_eq!(names, ["1", "2", "3"]);
        assert_eq!(
            p.terminal_strip(strip).unwrap().terminals(),
            order.as_slice()
        );
        p.restore_physical(&delta, false).unwrap();
        assert_eq!(p, before);
        assert!(renumber_edits(&terminals, &order, u32::MAX).is_err());
    }

    #[test]
    fn properties_summary_reports_physical_layers_without_inventing_values() {
        let (p, strip, cable) = project();
        let rows = physical_summary(&p, RepresentationTarget::Cable(cable));
        assert!(
            rows.iter()
                .any(|(k, v)| k.starts_with("Veias") && v == "2/2/0")
        );
        assert!(rows.iter().any(|(k, v)| k == "Fabricante" && v == "-"));
        let core = p.cores_for_cable(cable).next().unwrap().id();
        let rows = physical_summary(&p, RepresentationTarget::CableCore(core));
        assert!(rows.iter().any(|(_, v)| v == "não informada"));
        assert!(rows.iter().any(|(_, v)| v == "livre"));
        let rows = physical_summary(&p, RepresentationTarget::TerminalStrip(strip));
        assert!(rows.iter().any(|(k, v)| k == "Bornes" && v == "3"));
    }
}
