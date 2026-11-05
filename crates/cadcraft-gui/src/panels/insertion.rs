// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::{app::CadApp, commands::Icon, icons, preferences::PanelId, state::Selection};
use cad_craft::domain::*;
use eframe::egui::{self, Ui};
const CATEGORIES: [&str; 7] = [
    "Últimos utilizados",
    "Favoritos",
    "Tags",
    "Símbolos",
    "Dispositivos",
    "Macros",
    "Catálogos",
];
pub(crate) struct InsertionState {
    history: Vec<&'static str>,
    position: usize,
    query: String,
    favorites: Vec<RepresentationTarget>,
    recent: Vec<RepresentationTarget>,
    symbols: SymbolBrowserState,
}
/// Etapa 12: o que a inserção cria ou associa.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Association {
    #[default]
    NewDevice,
    ExistingDevice,
    ExistingFunction,
    Entity,
}
#[derive(Default)]
struct SymbolBrowserState {
    category: Option<String>,
    selected: Option<SymbolDefinitionId>,
    association: Association,
    /// Vazio usa o prefixo sugerido pelo símbolo.
    prefix: String,
    device: Option<DeviceId>,
    function: Option<FunctionId>,
    target: Option<RepresentationTarget>,
    angle: f64,
    mirror_x: bool,
    mirror_y: bool,
    cached_filter: Option<(String, Option<String>)>,
    rows: Vec<std::sync::Arc<SymbolDefinition>>,
    /// Etapa 17: projeto + biblioteca externa + embutidos.
    catalog: Vec<std::sync::Arc<SymbolDefinition>>,
    catalog_revision: Option<(u64, u64)>,
}
impl SymbolBrowserState {
    fn filter(&mut self, query: &str) {
        let key = (query.trim().to_lowercase(), self.category.clone());
        if self.cached_filter.as_ref() == Some(&key) {
            return;
        }
        self.rows = self
            .catalog
            .iter()
            .filter(|d| {
                self.category.as_deref().is_none_or(|c| c == d.category())
                    && format!("{} {} {}", d.name(), d.category(), d.id())
                        .to_lowercase()
                        .contains(&key.0)
            })
            .cloned()
            .collect();
        self.rows.sort_by(|a, b| a.name().cmp(b.name()));
        self.cached_filter = Some(key);
    }
}
impl Default for InsertionState {
    fn default() -> Self {
        Self {
            history: vec!["Página inicial"],
            position: 0,
            query: String::new(),
            favorites: vec![],
            recent: vec![],
            symbols: SymbolBrowserState::default(),
        }
    }
}
impl InsertionState {
    pub(crate) fn show_symbols(&mut self) {
        self.navigate("Símbolos");
    }
    fn navigate(&mut self, category: &'static str) {
        if self.history[self.position] == category {
            return;
        }
        self.history.truncate(self.position + 1);
        self.history.push(category);
        self.position = self.history.len() - 1;
    }
}
impl CadApp {
    /// Etapa 17: símbolo escolhido no Centro de inserção.
    pub(crate) fn selected_library_symbol(&self) -> Option<SymbolDefinitionId> {
        self.insertion.symbols.selected.clone()
    }
    pub(crate) fn show_insertion(&mut self, ui: &mut Ui) {
        if !super::header(
            ui,
            &mut self.preferences,
            PanelId::Insertion,
            "Centro de inserção",
        ) {
            return;
        }
        if matches!(
            self.session.graphics.tool,
            crate::interaction::Tool::InsertText | crate::interaction::Tool::InsertGraphic(_)
        ) {
            ui.group(|ui| {
                ui.label("Conteúdo livre · clique na folha para posicionar");
                if self.session.graphics.tool == crate::interaction::Tool::InsertText {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.free_text)
                            .hint_text("Texto fixo Unicode"),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Altura (mm)");
                        ui.add(
                            egui::DragValue::new(&mut self.free_height)
                                .range(0.1..=100.0)
                                .speed(0.1),
                        );
                    });
                    egui::ComboBox::from_id_salt("free_text_align")
                        .selected_text(format!("{:?}", self.free_align))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.free_align,
                                TextAlignment::Left,
                                "Esquerda",
                            );
                            ui.selectable_value(
                                &mut self.free_align,
                                TextAlignment::Center,
                                "Centro",
                            );
                            ui.selectable_value(
                                &mut self.free_align,
                                TextAlignment::Right,
                                "Direita",
                            );
                        });
                }
                ui.horizontal(|ui| {
                    ui.label("Traço (mm)");
                    ui.add(
                        egui::DragValue::new(&mut self.free_stroke)
                            .range(0.01..=10.0)
                            .speed(0.01),
                    );
                });
                ui.small("Clique inicial e final para formas · Esc cancela");
            });
        }
        ui.add(
            egui::TextEdit::singleline(&mut self.insertion.query)
                .hint_text("Pesquisar registros disponíveis")
                .desired_width(f32::INFINITY),
        );
        ui.horizontal(|ui| {
            if icons::button(
                ui,
                Icon::ArrowLeft,
                "",
                "Voltar categoria",
                self.insertion.position > 0,
                false,
                false,
            )
            .clicked()
            {
                self.insertion.position -= 1;
            }
            if icons::button(
                ui,
                Icon::ArrowRight,
                "",
                "Avançar categoria",
                self.insertion.position + 1 < self.insertion.history.len(),
                false,
                false,
            )
            .clicked()
            {
                self.insertion.position += 1;
            }
            if icons::tiny(ui, Icon::Home, "Página inicial do centro de inserção").clicked() {
                self.insertion.navigate("Página inicial");
            }
        });
        let category = self.insertion.history[self.insertion.position];
        ui.small(category);
        ui.separator();
        if category == "Símbolos" {
            egui::ScrollArea::vertical()
                .id_salt("symbol_results")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::ComboBox::from_id_salt("insertion_navigation")
                        .selected_text("Símbolos")
                        .show_ui(ui, |ui| {
                            for name in CATEGORIES {
                                if ui.selectable_label(name == category, name).clicked() {
                                    self.insertion.navigate(name);
                                }
                            }
                        });
                    self.show_symbol_browser(ui);
                });
            return;
        }
        let mut selected = None;
        let mut favorite = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for name in CATEGORIES {
                    if ui.selectable_label(name == category, name).clicked() {
                        self.insertion.navigate(name);
                    }
                }
                ui.separator();
                ui.weak("Registros elétricos da sessão. Abra Símbolos para inserir uma ocorrência gráfica.");
                if ["Símbolos", "Macros", "Catálogos", "Tags"].contains(&category) {
                    ui.label(format!(
                        "Nenhum registro de {} disponível.",
                        category.to_lowercase()
                    ));
                    return;
                }
                let mut count = 0;
                if let Some(project) = self.session.project() {
                    for device in project.devices() {
                        let target = RepresentationTarget::Device(device.id());
                        let text = format!(
                            "{} · {}",
                            device.designation(),
                            device.classification().as_str()
                        );
                        let in_category = match category {
                            "Favoritos" => self.insertion.favorites.contains(&target),
                            "Últimos utilizados" => self.insertion.recent.contains(&target),
                            _ => true,
                        };
                        if !in_category
                            || !text
                                .to_lowercase()
                                .contains(&self.insertion.query.to_lowercase())
                        {
                            continue;
                        }
                        count += 1;
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    self.session.selection == Selection::Electrical(target),
                                    &text,
                                )
                                .clicked()
                            {
                                selected = Some(target);
                            }
                            if icons::button(
                                ui,
                                Icon::Pin,
                                "",
                                "Adicionar/remover dos favoritos desta sessão",
                                true,
                                self.insertion.favorites.contains(&target),
                                false,
                            )
                            .clicked()
                            {
                                favorite = Some(target);
                            }
                        });
                    }
                }
                if count == 0 {
                    ui.label("Nenhum registro disponível para esta categoria e pesquisa.");
                }
                if category == "Últimos utilizados" {
                    ui.small("Histórico das consultas feitas neste centro.");
                }
            });
        if let Some(target) = selected {
            self.session.select(Selection::Electrical(target));
            self.insertion.recent.retain(|id| *id != target);
            self.insertion.recent.insert(0, target);
            self.insertion.recent.truncate(20);
        }
        if let Some(target) = favorite {
            if self.insertion.favorites.contains(&target) {
                self.insertion.favorites.retain(|id| *id != target);
            } else {
                self.insertion.favorites.push(target);
            }
        }
    }
    fn show_symbol_browser(&mut self, ui: &mut Ui) {
        let mut start = None;
        let revision = (self.session.revision, self.session.symbol_library_revision);
        if self.insertion.symbols.catalog_revision != Some(revision) {
            self.insertion.symbols.catalog = self
                .session
                .symbol_catalog()
                .into_iter()
                .map(std::sync::Arc::new)
                .collect();
            self.insertion.symbols.catalog_revision = Some(revision);
            self.insertion.symbols.cached_filter = None;
        }
        let browser = &mut self.insertion.symbols;
        let categories: std::collections::BTreeSet<String> = browser
            .catalog
            .iter()
            .map(|d| d.category().to_owned())
            .collect();
        egui::ComboBox::from_id_salt("symbol_category")
            .selected_text(browser.category.as_deref().unwrap_or("Todas as categorias"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut browser.category, None, "Todas as categorias");
                for category in categories {
                    ui.selectable_value(&mut browser.category, Some(category.clone()), category);
                }
            });
        browser.filter(&self.insertion.query);
        let active = self.session.active.is_some();
        let selected_definition = browser
            .selected
            .as_ref()
            .and_then(|id| browser.catalog.iter().find(|d| d.id() == id).cloned());
        if ui
            .add_enabled(
                active && selected_definition.is_some(),
                egui::Button::new("Inserir na página"),
            )
            .clicked()
        {
            start = browser.selected.clone();
        }
        if !active {
            ui.small("Abra uma página para inserir.");
        }
        // Compact summary keeps the symbol rows in view; details are in the header.
        let suggested = selected_definition
            .as_deref()
            .map(crate::state::default_prefix)
            .unwrap_or("K");
        let prefix = if browser.prefix.trim().is_empty() {
            suggested.to_owned()
        } else {
            browser.prefix.trim().to_owned()
        };
        if let Some(d) = &selected_definition {
            let summary = match browser.association {
                Association::NewDevice => self
                    .session
                    .project()
                    .and_then(|p| p.next_device_designation(&prefix).ok())
                    .map_or_else(
                        || "prefixo inválido".to_owned(),
                        |next| format!("cria {next}"),
                    ),
                Association::ExistingDevice => browser
                    .device
                    .and_then(|id| self.session.project()?.device(id))
                    .map_or_else(
                        || "escolha o dispositivo".to_owned(),
                        |dev| format!("função em {}", dev.designation()),
                    ),
                Association::ExistingFunction => browser
                    .function
                    .and_then(|id| self.session.project()?.function(id))
                    .map_or_else(
                        || "escolha a função não representada".to_owned(),
                        |f| format!("função existente de {}", f.designation()),
                    ),
                Association::Entity => "sem novo cadastro".to_owned(),
            };
            ui.small(format!("{} · {summary}", d.name()));
        }
        egui::CollapsingHeader::new("Orientação e associação").show(ui, |ui| {
            egui::ComboBox::from_id_salt("symbol_association")
                .selected_text(match browser.association {
                    Association::NewDevice => "Novo dispositivo",
                    Association::ExistingDevice => "Função em dispositivo existente",
                    Association::ExistingFunction => "Função não representada existente",
                    Association::Entity => "Entidade existente ou sem associação",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut browser.association,
                        Association::NewDevice,
                        "Novo dispositivo",
                    );
                    ui.selectable_value(
                        &mut browser.association,
                        Association::ExistingDevice,
                        "Função em dispositivo existente",
                    );
                    ui.selectable_value(&mut browser.association, Association::ExistingFunction,
                        "Função não representada existente");
                    ui.selectable_value(
                        &mut browser.association,
                        Association::Entity,
                        "Entidade existente ou sem associação",
                    );
                });
            match browser.association {
                Association::NewDevice => {
                    ui.horizontal(|ui| {
                        ui.label("Prefixo");
                        ui.add(
                            egui::TextEdit::singleline(&mut browser.prefix)
                                .hint_text(suggested)
                                .desired_width(60.0),
                        );
                    });
                }
                Association::ExistingDevice => {
                    let label = browser
                        .device
                        .and_then(|id| self.session.project()?.device(id))
                        .map(|d| d.designation().to_string())
                        .unwrap_or_else(|| "Escolha o dispositivo".into());
                    egui::ComboBox::from_id_salt("symbol_device")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            if let Some(p) = self.session.project() {
                                for d in p.devices() {
                                    ui.selectable_value(
                                        &mut browser.device,
                                        Some(d.id()),
                                        d.designation().to_string(),
                                    );
                                }
                            }
                        });
                }
                Association::ExistingFunction => {
                    let label = browser.function
                        .and_then(|id| self.session.project()?.function(id))
                        .map(|f| format!("{} · {:?}", f.designation(), f.kind()))
                        .unwrap_or_else(|| "Escolha a função".into());
                    egui::ComboBox::from_id_salt("symbol_existing_function")
                        .selected_text(label).show_ui(ui, |ui| {
                            if let Some(p) = self.session.project() {
                                for f in p.functions() {
                                    if p.function_references(f.id()).is_ok_and(|v| v.is_empty()) {
                                        ui.selectable_value(&mut browser.function, Some(f.id()),
                                            format!("{} · {:?}", f.designation(), f.kind()));
                                    }
                                }
                            }
                        });
                    if let (Some(id), Some(definition), Some(p)) = (browser.function, selected_definition.as_deref(), self.session.project()) {
                        let labels: std::collections::BTreeSet<_> = p.connection_points()
                            .filter(|point| point.function_id() == id)
                            .map(|point| point.designation().as_str()).collect();
                        if definition.ports().iter().any(|port| !labels.contains(port.identification())) {
                            ui.colored_label(ui.visuals().warn_fg_color,
                                "Aviso: rótulos diferentes; portas serão ligadas aos pontos por ordem.");
                        }
                    }
                }
                Association::Entity => {}
            }
            ui.horizontal(|ui| {
                ui.label("Ângulo (°)");
                ui.add(egui::DragValue::new(&mut browser.angle).speed(90.0));
            });
            ui.checkbox(&mut browser.mirror_x, "Espelhar X local");
            ui.checkbox(&mut browser.mirror_y, "Espelhar Y local");
            if browser.association != Association::Entity {
                return;
            }
            egui::ComboBox::from_id_salt("symbol_target")
                .selected_text(if browser.target.is_some() {
                    "Entidade existente"
                } else {
                    "Sem associação elétrica"
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut browser.target, None, "Sem associação elétrica");
                    if let Some(p) = self.session.project() {
                        for f in p.functions() {
                            ui.selectable_value(
                                &mut browser.target,
                                Some(RepresentationTarget::Function(f.id())),
                                format!("Função {}", f.id()),
                            );
                        }
                        for d in p.devices() {
                            ui.selectable_value(
                                &mut browser.target,
                                Some(RepresentationTarget::Device(d.id())),
                                format!("Dispositivo {}", d.designation()),
                            );
                        }
                        for t in p.terminals() {
                            ui.selectable_value(
                                &mut browser.target,
                                Some(RepresentationTarget::Terminal(t.id())),
                                format!("Borne {}", t.id()),
                            );
                        }
                    }
                });
            if let Some(target) = browser.target {
                ui.small(format!("{target:?}"));
            }
        });
        ui.small("Clique para selecionar; duplo clique inicia. Inserção repetida até Esc.");
        ui.separator();
        // One scroll surface includes controls and rows, so short panels can
        // scroll past the controls. Reserve the full list but build visible rows only.
        let row_height = 68.0;
        let (_, area) = ui.allocate_space(egui::vec2(
            ui.available_width(),
            browser.rows.len() as f32 * row_height,
        ));
        let first = (((ui.clip_rect().top() - area.top()).max(0.0) / row_height).floor() as usize)
            .min(browser.rows.len());
        let last =
            ((((ui.clip_rect().bottom() - area.top()).max(0.0) / row_height).ceil() as usize) + 1)
                .min(browser.rows.len());
        let visible = egui::Rect::from_min_max(
            area.min + egui::vec2(0.0, first as f32 * row_height),
            egui::pos2(area.right(), area.top() + last as f32 * row_height),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(visible), |ui| {
            for row in first..last {
                let top = ui.cursor().top();
                let d = browser.rows[row].clone();
                let d = &*d;
                ui.push_id(d.id().as_str(), |ui| {
                    ui.horizontal(|ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(52.0, 52.0), egui::Sense::click());
                        let painter = ui.painter_at(rect);
                        painter.rect_filled(rect, 3, crate::theme::PAPER);
                        let mut camera = crate::camera::Camera::default();
                        camera.fit(d.bounds(), rect.shrink(6.0));
                        let transform = Transform2D::new(
                            Point2D::new(0.0, 0.0).expect("zero"),
                            0.0,
                            false,
                            false,
                        )
                        .expect("identity");
                        crate::symbol_view::paint_symbol(
                            &painter,
                            d,
                            transform,
                            camera,
                            rect,
                            egui::Stroke::new(1.2_f32, crate::theme::INK),
                        );
                        let label = ui.add_sized(
                            [ui.available_width(), 52.0],
                            egui::Button::new(d.name())
                                .selected(browser.selected.as_ref() == Some(d.id()))
                                .wrap(),
                        );
                        if response.clicked() || label.clicked() {
                            browser.selected = Some(d.id().clone());
                        }
                        if active && (response.double_clicked() || label.double_clicked()) {
                            start = Some(d.id().clone());
                        }
                        label.on_hover_text(format!(
                            "{} · v{}\n{} ponto(s) de conexão",
                            d.category(),
                            d.version(),
                            d.ports().len()
                        ));
                    });
                    ui.add_space(8.0);
                });
                ui.add_space((row_height - (ui.cursor().top() - top)).max(0.0));
            }
        });
        if browser.rows.is_empty() {
            ui.label("Nenhum símbolo corresponde à pesquisa.");
        }
        if let Some(id) = start {
            use crate::interaction::DevicePlan;
            let result = Point2D::new(0.0, 0.0)
                .and_then(|p| {
                    Transform2D::new(p, browser.angle, browser.mirror_x, browser.mirror_y)
                })
                .and_then(|t| match browser.association {
                    Association::NewDevice => self.session.begin_device_insertion(
                        id,
                        DevicePlan::New {
                            prefix: browser.prefix.clone(),
                        },
                        t,
                    ),
                    Association::ExistingDevice => match browser.device {
                        Some(device) => {
                            self.session
                                .begin_device_insertion(id, DevicePlan::Existing(device), t)
                        }
                        None => Err(cad_craft::error::DomainError::InvalidRelation(
                            "escolha o dispositivo existente",
                        )),
                    },
                    Association::ExistingFunction => match browser.function {
                        Some(function) => self.session.begin_device_insertion(
                            id,
                            DevicePlan::ExistingFunction(function),
                            t,
                        ),
                        None => Err(cad_craft::error::DomainError::InvalidRelation(
                            "escolha a função não representada",
                        )),
                    },
                    Association::Entity => {
                        self.session.begin_symbol_insertion(id, browser.target, t)
                    }
                });
            match result {
                Ok(()) => self.log(
                    crate::app::Level::Info,
                    "Inserção ativa: clique na página; Esc cancela.",
                ),
                Err(e) => self.log(crate::app::Level::Error, e.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn symbol_search_combines_name_and_category_and_caches_unchanged_filter() {
        // Etapa 17: o catálogo vem da sessão (projeto + biblioteca + embutidos).
        let mut b = SymbolBrowserState {
            catalog: SymbolLibrary::builtin()
                .definitions()
                .cloned()
                .map(std::sync::Arc::new)
                .collect(),
            ..Default::default()
        };
        b.filter("normalmente");
        assert_eq!(b.rows.len(), 2);
        b.category = Some("Contatos".into());
        b.filter("ABERTO");
        assert_eq!(b.rows.len(), 1);
        assert_eq!(b.rows[0].id().as_str(), "cad-craft.contact-no");
        let ptr = b.rows.as_ptr();
        b.filter("ABERTO");
        assert_eq!(b.rows.as_ptr(), ptr);
        b.category = Some("Máquinas".into());
        b.filter("ABERTO");
        assert!(b.rows.is_empty());
    }
}
