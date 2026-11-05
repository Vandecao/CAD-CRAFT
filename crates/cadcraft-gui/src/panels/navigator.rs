// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use crate::{
    app::CadApp,
    commands::{CommandId, Icon},
    icons,
    navigation::{PageFilter, PageIndex, kind_label, page_kinds},
    preferences::PanelId,
    state::Selection,
    theme,
};
use cad_craft::domain::*;
use eframe::egui::{self, Key, Ui};
use std::collections::HashSet;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum NavigatorTab {
    #[default]
    Pages,
    Devices,
    Structures,
    Connections,
    Cables,
    Terminals,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(app: &mut CadApp, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.show(ctx),
        )
    }
    fn app() -> CadApp {
        let mut app = CadApp::default();
        app.session
            .replace(crate::demo::project().unwrap(), true)
            .unwrap();
        app
    }
    #[test]
    fn tree_focus_survives_idle_frames_and_enter_opens_selected_page() {
        let ctx = egui::Context::default();
        let mut app = app();
        let page = app.session.project().unwrap().pages().next().unwrap().id();
        frame(&mut app, &ctx, vec![]);
        app.session.select(Selection::Page(page));
        app.navigator.focused = Some(page.to_string());
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("page_tree")));
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert_eq!(app.session.active, Some(page));
    }
    fn contains_text(shape: &egui::Shape, needle: &str) -> bool {
        match shape {
            egui::Shape::Text(text) => text.galley.text().contains(needle),
            egui::Shape::Vec(shapes) => shapes.iter().any(|s| contains_text(s, needle)),
            _ => false,
        }
    }
    #[test]
    fn synchronize_clears_hidden_context_and_scrolls_to_virtualized_page() {
        let ctx = egui::Context::default();
        let mut app = app();
        let doc = app.session.selected_document().unwrap();
        let mut last = None;
        for i in 0..250 {
            last = Some(
                app.session
                    .new_page(
                        doc,
                        &format!("REGRESSION_ROW_{i:03}"),
                        &i.to_string(),
                        PageKind::MultiLine,
                        Size2D::new(420.0, 297.0).unwrap(),
                    )
                    .unwrap(),
            );
        }
        let last = last.unwrap();
        app.open_page(last);
        frame(&mut app, &ctx, vec![]);
        app.navigator.filter.text = "does-not-match".into();
        app.navigator.tab = NavigatorTab::Devices;
        app.preferences.panel_mut(PanelId::Navigator).collapsed = true;
        app.execute(CommandId::SyncSelection, &ctx);
        for _ in 0..2 {
            frame(&mut app, &ctx, vec![]);
        }
        let output = frame(&mut app, &ctx, vec![]);
        assert!(
            output
                .shapes
                .iter()
                .any(|s| s.clip_rect.right() < 320.0
                    && contains_text(&s.shape, "REGRESSION_ROW_249"))
        );
        assert!(app.navigator.tab == NavigatorTab::Pages);
        assert!(!app.preferences.panel(PanelId::Navigator).collapsed);
        assert!(app.navigator.filter.text.is_empty());
    }
}
impl NavigatorTab {
    const ALL: [Self; 6] = [
        Self::Pages,
        Self::Devices,
        Self::Structures,
        Self::Connections,
        Self::Cables,
        Self::Terminals,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Pages => "Páginas",
            Self::Devices => "Dispositivos",
            Self::Structures => "Estruturas",
            Self::Connections => "Conexões",
            Self::Cables => "Cabos",
            Self::Terminals => "Bornes",
        }
    }
}
#[derive(Clone)]
struct Entry {
    key: String,
    label: String,
    selection: Option<Selection>,
    kind: Option<PageKind>,
    children: Vec<Entry>,
}
impl Entry {
    fn insert(&mut self, path: &[String], page: Entry) {
        if let Some((part, rest)) = path.split_first() {
            let key = format!("{}/{}", self.key, part);
            let pos = if let Some(pos) = self.children.iter().position(|c| c.key == key) {
                pos
            } else {
                self.children.push(Entry {
                    key: key.clone(),
                    label: part.clone(),
                    selection: None,
                    kind: None,
                    children: vec![],
                });
                self.children.len() - 1
            };
            self.children[pos].insert(rest, page);
        } else {
            self.children.push(page);
        }
    }
}
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
pub(crate) struct NavigatorState {
    pub tab: NavigatorTab,
    pub focus_filter: bool,
    pub reveal: Option<PageId>,
    filter: PageFilter,
    last_filter: Option<PageFilter>,
    tree: bool,
    entries: Vec<Entry>,
    reference_cache_key: Option<(ProjectId, u64, NavigatorTab)>,
    reference_cache: Vec<(String, RepresentationId)>,
    physical_key: Option<(ProjectId, u64, NavigatorTab)>,
    physical_rows: Vec<PhysicalTreeRow>,
    collapsed: HashSet<String>,
    focused: Option<String>,
    pub(crate) selected_structure: Option<String>,
}
impl Default for NavigatorState {
    fn default() -> Self {
        Self {
            tab: NavigatorTab::Pages,
            focus_filter: false,
            reveal: None,
            filter: PageFilter::default(),
            last_filter: None,
            tree: true,
            entries: vec![],
            reference_cache_key: None,
            reference_cache: vec![],
            physical_key: None,
            physical_rows: vec![],
            collapsed: HashSet::new(),
            focused: None,
            selected_structure: None,
        }
    }
}
impl NavigatorState {
    pub fn reveal_page(&mut self, page: PageId) {
        self.tab = NavigatorTab::Pages;
        self.filter = PageFilter::default();
        self.last_filter = None;
        self.reveal = Some(page);
        self.focused = Some(page.to_string());
        self.collapsed.clear();
    }
    pub fn invalidate(&mut self) {
        self.last_filter = None;
    }
    fn rebuild(&mut self, index: &PageIndex, project: &Project) {
        let rows = index.filtered(&self.filter);
        let mut root = Entry {
            key: project.id().to_string(),
            label: project.name().as_str().into(),
            selection: Some(Selection::Project),
            kind: None,
            children: vec![],
        };
        for doc in project.documents() {
            if self.filter.document.is_some_and(|id| id != doc.id()) {
                continue;
            }
            let mut branch = Entry {
                key: doc.id().to_string(),
                label: doc.name().as_str().into(),
                selection: Some(Selection::Document(doc.id())),
                kind: None,
                children: vec![],
            };
            for row in rows.iter().filter(|r| r.document == doc.id()) {
                let page = Entry {
                    key: row.id.to_string(),
                    label: format!("{}  {}", row.number, row.name),
                    selection: Some(Selection::Page(row.id)),
                    kind: Some(row.kind.clone()),
                    children: vec![],
                };
                if self.tree {
                    branch.insert(&row.path, page);
                } else {
                    branch.children.push(page);
                }
            }
            if !branch.children.is_empty() || (self.filter == PageFilter::default()) {
                root.children.push(branch);
            }
        }
        self.entries = vec![root];
        self.last_filter = Some(self.filter.clone());
    }
}
fn flatten<'a>(
    entries: &'a [Entry],
    collapsed: &HashSet<String>,
    depth: usize,
    out: &mut Vec<(&'a Entry, usize)>,
) {
    for entry in entries {
        out.push((entry, depth));
        if !collapsed.contains(&entry.key) {
            flatten(&entry.children, collapsed, depth + 1, out);
        }
    }
}
impl CadApp {
    pub(crate) fn show_navigator(&mut self, ui: &mut Ui) {
        let title = self
            .session
            .project()
            .map(|p| format!("Páginas — {}", p.name().as_str()))
            .unwrap_or_else(|| "Páginas — Nenhum projeto".into());
        if !super::header(ui, &mut self.preferences, PanelId::Navigator, &title) {
            return;
        }
        if self.preferences.panel(PanelId::Preview).visible {
            let compact = ui.available_height() < 300.0;
            let collapsed = self.preferences.panel(PanelId::Preview).collapsed || compact;
            let height = if collapsed {
                28.0
            } else {
                self.preferences
                    .preview_height
                    .min((ui.available_height() * 0.45).max(70.0))
            };
            let output =
                egui::TopBottomPanel::bottom(egui::Id::new(("preview", self.layout_generation)))
                    .default_height(height)
                    .min_height(if collapsed { 28.0 } else { 70.0 })
                    .max_height((ui.available_height() * 0.7).max(70.0))
                    .resizable(!collapsed)
                    .show_inside(ui, |ui| {
                        if collapsed {
                            if ui.add_enabled(!compact, egui::Button::new("Expandir pré-visualização"))
                                .on_disabled_hover_text("Aumente a altura da janela ou recolha a Ribbon para exibir miniaturas.")
                                .clicked() {
                                self.preferences.panel_mut(PanelId::Preview).collapsed = false;
                            }
                        } else {
                            self.show_preview(ui);
                        }
                    });
            if !collapsed {
                self.preferences.preview_height = output.response.rect.height();
            }
        }
        egui::ScrollArea::horizontal()
            .id_salt("navigator_tabs")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for tab in NavigatorTab::ALL {
                        ui.selectable_value(&mut self.navigator.tab, tab, tab.label());
                    }
                });
            });
        if self.navigator.tab != NavigatorTab::Pages {
            ui.set_clip_rect(ui.clip_rect().intersect(ui.available_rect_before_wrap()));
            self.entity_navigator(ui);
            return;
        }
        let mut nav = std::mem::take(&mut self.navigator);
        let response = ui.add(
            egui::TextEdit::singleline(&mut nav.filter.text)
                .hint_text("Filtrar páginas ou documentos")
                .desired_width(f32::INFINITY),
        );
        if nav.focus_filter {
            response.request_focus();
            nav.focus_filter = false;
        }
        ui.horizontal(|ui| {
            ui.menu_button("Filtros", |ui| {
                egui::ComboBox::from_id_salt("kind_filter")
                    .selected_text(
                        nav.filter
                            .kind
                            .as_ref()
                            .map(kind_label)
                            .unwrap_or("Todos os tipos"),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut nav.filter.kind, None, "Todos os tipos");
                        for kind in page_kinds() {
                            let label = kind_label(&kind).to_owned();
                            ui.selectable_value(&mut nav.filter.kind, Some(kind), label);
                        }
                    });
                egui::ComboBox::from_id_salt("document_filter")
                    .selected_text(
                        nav.filter
                            .document
                            .and_then(|id| self.session.project()?.document(id))
                            .map(|d| d.name().as_str())
                            .unwrap_or("Todos os documentos"),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut nav.filter.document, None, "Todos os documentos");
                        if let Some(p) = self.session.project() {
                            for doc in p.documents() {
                                ui.selectable_value(
                                    &mut nav.filter.document,
                                    Some(doc.id()),
                                    doc.name().as_str(),
                                );
                            }
                        }
                    });
                ui.label("Estrutura funcional ou localização");
                ui.text_edit_singleline(&mut nav.filter.structure);
                if ui.button("Limpar filtros").clicked() {
                    nav.filter = PageFilter::default();
                }
            });
            if ui.selectable_label(nav.tree, "Árvore").clicked() {
                nav.tree = true;
                nav.invalidate();
            }
            if ui.selectable_label(!nav.tree, "Lista").clicked() {
                nav.tree = false;
                nav.invalidate();
            }
            if icons::tiny(ui, Icon::Close, "Limpar filtros").clicked() {
                nav.filter = PageFilter::default();
            }
        });
        let Some(project) = self.session.project() else {
            ui.weak("Crie uma sessão para adicionar documentos e páginas.");
            self.navigator = nav;
            return;
        };
        if nav.last_filter.as_ref() != Some(&nav.filter) {
            nav.rebuild(&self.index, project);
        }
        if nav.reveal.is_some() {
            nav.collapsed.clear();
        }
        let mut visible = Vec::new();
        flatten(&nav.entries, &nav.collapsed, 0, &mut visible);
        let mut toggle = None;
        let mut select = None;
        let mut open = None;
        let mut command = None;
        let mut focus = None;
        let tree_id = egui::Id::new("page_tree");
        // Register the persistent keyboard target on every frame; egui drops
        // focus for IDs which are not backed by a live widget.
        ui.interact(
            ui.available_rect_before_wrap(),
            tree_id,
            egui::Sense::focusable_noninteractive(),
        );
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                tree_id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    ..Default::default()
                },
            )
        });
        let focused = ui.memory(|m| m.has_focus(tree_id));
        if focused && self.dialog.is_none() {
            let current = visible
                .iter()
                .position(|(e, _)| Some(&e.key) == nav.focused.as_ref())
                .unwrap_or(0);
            for (key, delta) in [(Key::ArrowUp, -1isize), (Key::ArrowDown, 1)] {
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                    let next = current
                        .saturating_add_signed(delta)
                        .min(visible.len().saturating_sub(1));
                    if let Some((e, _)) = visible.get(next) {
                        focus = Some(e.key.clone());
                        select = e.selection;
                    }
                }
            }
            if let Some((entry, depth)) = visible.get(current) {
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowRight)) {
                    if nav.collapsed.contains(&entry.key) {
                        toggle = Some(entry.key.clone());
                    } else if !entry.children.is_empty() {
                        focus = Some(entry.children[0].key.clone());
                        select = entry.children[0].selection;
                    }
                }
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft)) {
                    if !entry.children.is_empty() && !nav.collapsed.contains(&entry.key) {
                        toggle = Some(entry.key.clone());
                    } else if let Some((parent, _)) =
                        visible[..current].iter().rev().find(|(_, d)| d < depth)
                    {
                        focus = Some(parent.key.clone());
                        select = parent.selection;
                    }
                }
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter)) {
                    if let Some(Selection::Page(id)) = entry.selection {
                        open = Some(id);
                    } else if !entry.children.is_empty() {
                        toggle = Some(entry.key.clone());
                    }
                }
            }
        }
        let row_height = 22.0;
        // show_inside reserves the thumbnail region; restrict the tree's paint
        // clip as well, including when controls use most of a small panel.
        ui.set_clip_rect(ui.clip_rect().intersect(ui.available_rect_before_wrap()));
        let mut scroll = egui::ScrollArea::both()
            .id_salt("page_tree_scroll")
            .auto_shrink([false, false]);
        let reveal_index = nav
            .reveal
            .and_then(|page| {
                visible
                    .iter()
                    .position(|(entry, _)| entry.selection == Some(Selection::Page(page)))
            })
            .or_else(|| {
                focus
                    .as_ref()
                    .and_then(|key| visible.iter().position(|(entry, _)| &entry.key == key))
            });
        if let Some(index) = reveal_index {
            let row_stride = row_height + ui.spacing().item_spacing.y;
            scroll = scroll.vertical_scroll_offset(
                (index as f32 * row_stride - ui.available_height() * 0.4).max(0.0),
            );
        }
        scroll.show_rows(ui, row_height, visible.len(), |ui, range| {
            for index in range {
                let (entry, depth) = visible[index];
                ui.push_id(&entry.key, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(depth as f32 * 14.0);
                        if !entry.children.is_empty() {
                            if icons::disclosure(ui, nav.collapsed.contains(&entry.key)).clicked() {
                                toggle = Some(entry.key.clone());
                            }
                        } else {
                            ui.add_space(17.0);
                        }
                        if let Some(kind) = &entry.kind {
                            icons::page_icon(ui, kind);
                        } else {
                            let (r, _) = ui
                                .allocate_exact_size(egui::vec2(16.0, 18.0), egui::Sense::hover());
                            icons::paint(
                                ui.painter(),
                                r,
                                if entry.selection.is_some() {
                                    Icon::Folder
                                } else {
                                    Icon::Panels
                                },
                                theme::MUTED,
                            );
                        }
                        let selected = entry.selection.is_some_and(|s| s == self.session.selection)
                            || nav.focused.as_ref() == Some(&entry.key);
                        let response = ui.selectable_label(selected, &entry.label);
                        if response.clicked() {
                            focus = Some(entry.key.clone());
                            select = entry.selection;
                            ui.memory_mut(|m| m.request_focus(egui::Id::new("page_tree")));
                        }
                        if response.double_clicked() {
                            if let Some(Selection::Page(id)) = entry.selection {
                                open = Some(id);
                            } else if !entry.children.is_empty() {
                                toggle = Some(entry.key.clone());
                            }
                        }
                        if let Some(Selection::Page(id)) = entry.selection {
                            response.context_menu(|ui| {
                                for (cmd, label) in [
                                    (CommandId::OpenPage, "Abrir"),
                                    (CommandId::NewPage, "Nova página"),
                                    (CommandId::RenamePage, "Renomear"),
                                    (CommandId::IdentifyPage, "Alterar identificação"),
                                    (CommandId::Properties, "Propriedades"),
                                    (CommandId::PageUp, "Reordenar acima"),
                                    (CommandId::PageDown, "Reordenar abaixo"),
                                    (CommandId::DeletePage, "Excluir"),
                                ] {
                                    if ui.button(label).clicked() {
                                        select = Some(Selection::Page(id));
                                        command = Some(cmd);
                                        ui.close();
                                    }
                                }
                            });
                            if nav.reveal == Some(id) {
                                response.scroll_to_me(Some(egui::Align::Center));
                                nav.reveal = None;
                            }
                        }
                    });
                });
            }
        });
        if let Some(key) = toggle
            && !nav.collapsed.remove(&key)
        {
            nav.collapsed.insert(key);
        }
        if let Some(key) = focus {
            nav.focused = Some(key);
        }
        self.navigator = nav;
        if let Some(selection) = select {
            self.session.select(selection);
        }
        if let Some(id) = open {
            self.open_page(id);
        }
        if let Some(cmd) = command {
            if cmd == CommandId::Properties {
                self.preferences.panel_mut(PanelId::Properties).visible = true;
                self.preferences.panel_mut(PanelId::Properties).collapsed = false;
            } else {
                self.execute(cmd, ui.ctx());
            }
        }
    }
    fn physical_navigator(&mut self, ui: &mut Ui) {
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
    fn entity_navigator(&mut self, ui: &mut Ui) {
        if matches!(
            self.navigator.tab,
            NavigatorTab::Cables | NavigatorTab::Terminals
        ) {
            self.physical_navigator(ui);
            return;
        }
        let Some(p) = self.session.project() else {
            ui.weak("Nenhum projeto aberto.");
            return;
        };
        let mut entries: Vec<(String, Option<RepresentationTarget>)> = vec![];
        let cache_key = (p.id(), self.session.revision, self.navigator.tab);
        let rebuild_references = self.navigator.reference_cache_key != Some(cache_key);
        let mut references: Vec<(String, RepresentationId)> = vec![];
        match self.navigator.tab {
            NavigatorTab::Devices => {
                if rebuild_references {
                    for device in p.devices() {
                        for r in p.device_references(device.id()).unwrap_or_default() {
                            references.push((
                                format!(
                                    "{} · {:?} · {}",
                                    device.designation(),
                                    r.role,
                                    r.reference
                                ),
                                r.representation,
                            ));
                        }
                    }
                }
                entries.extend(p.devices().map(|d| {
                    (
                        format!("{} · {}", d.designation(), d.classification().as_str()),
                        Some(RepresentationTarget::Device(d.id())),
                    )
                }));
            }
            NavigatorTab::Connections => {
                if rebuild_references {
                    for marker in p.markers() {
                        if let Some(rep) = p.representation(marker.id()) {
                            let page = p
                                .page(rep.page_id())
                                .map(|p| p.number().as_str())
                                .unwrap_or("?");
                            references.push((
                                format!(
                                    "{} · página {} · X {:.3} · Y {:.3} mm",
                                    marker.name().as_str(),
                                    page,
                                    rep.transform().translation().x(),
                                    rep.transform().translation().y()
                                ),
                                marker.id(),
                            ));
                        }
                    }
                }
                entries.extend(p.connections().map(|c| {
                    (
                        format!("Conexão {}", c.id()),
                        Some(RepresentationTarget::Connection(c.id())),
                    )
                }));
            }
            // Rendered by physical_navigator from the revision cache.
            NavigatorTab::Cables | NavigatorTab::Terminals => {}
            NavigatorTab::Structures => {
                for doc in p.documents() {
                    for axis in [StructureAxis::Function, StructureAxis::Location] {
                        for group in p.page_tree(doc.id(), axis).unwrap_or_default() {
                            if !group.path.is_empty() {
                                entries.push((
                                    format!(
                                        "{} · {}{}",
                                        doc.name().as_str(),
                                        if axis == StructureAxis::Function {
                                            "="
                                        } else {
                                            "+"
                                        },
                                        crate::navigation::path_text(&group.path)
                                    ),
                                    None,
                                ));
                            }
                        }
                    }
                }
            }
            NavigatorTab::Pages => {}
        }
        if rebuild_references {
            self.navigator.reference_cache_key = Some(cache_key);
            self.navigator.reference_cache = references.clone();
        } else {
            references = self.navigator.reference_cache.clone();
        }
        let mut target = None;
        let mut reference_target = None;
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if entries.is_empty() && references.is_empty() {
                    ui.weak("Nenhum registro disponível neste navegador.");
                }
                for (label, id) in entries {
                    if ui
                        .selectable_label(
                            id.is_some_and(|id| {
                                self.session.selection == Selection::Electrical(id)
                            }) || (id.is_none()
                                && self.navigator.selected_structure.as_ref() == Some(&label)),
                            &label,
                        )
                        .clicked()
                    {
                        if let Some(id) = id {
                            target = Some(id);
                        } else {
                            self.navigator.selected_structure = Some(label.clone());
                        }
                    }
                }
                for (label, id) in references {
                    if ui
                        .selectable_label(
                            self.session.selection == Selection::Representation(id),
                            label,
                        )
                        .clicked()
                    {
                        reference_target = Some(id);
                    }
                }
            });
        if let Some(target) = target {
            self.session.select(Selection::Electrical(target));
        }
        if let Some(id) = reference_target {
            self.navigate_to_reference(id);
        }
    }
}
