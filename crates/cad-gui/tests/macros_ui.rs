// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 19 — macros com eventos egui reais: criar da seleção, salvar na
//! biblioteca, inserir duas vezes por clique na folha, Desfazer/Refazer,
//! girar na inserção, macro de página, colar/duplicar com fios e arquivo
//! inválido recusado.
use cad_craft::{domain::*, persistence};
use cad_gui::{
    app::CadApp,
    commands::{CommandId, registry},
    editing::GraphicEdit,
    interaction::Tool,
    preferences::RibbonTab,
};
use eframe::egui::{
    self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, vec2,
};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
}
fn at(x: f64, y: f64) -> Transform2D {
    Transform2D::new(p(x, y), 0.0, false, false).unwrap()
}
fn symbol(key: &str) -> SymbolDefinition {
    SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap())
        .unwrap()
        .clone()
}
fn place(
    project: &mut Project,
    page: PageId,
    key: &str,
    device: PlacementDevice,
    kind: FunctionKind,
    x: f64,
    y: f64,
) -> RepresentationId {
    project
        .place_symbol(SymbolPlacement {
            page,
            definition: symbol(key),
            transform: at(x, y),
            device,
            function_kind: kind,
        })
        .unwrap()
        .representation_id()
}
fn new_device(project: &Project, prefix: &str) -> PlacementDevice {
    PlacementDevice::New {
        designation: project.next_device_designation(prefix).unwrap(),
        classification: prefix.into(),
    }
}
fn port(r: RepresentationId, id: &str) -> PortRef {
    PortRef::new(r, id).unwrap()
}
fn route(project: &mut Project, a: PortRef, b: PortRef, points: Vec<Point2D>) -> RepresentationId {
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source: a.clone(),
            target: b.clone(),
            points,
        })
        .unwrap();
    project
        .connection_routes()
        .find(|r| r.source() == &a && r.target() == &b)
        .unwrap()
        .id()
}

/// Partida direta (-Q1, -F1, bobina e contato de -K1, fios e junção) na
/// página 1; página 2 vazia.
fn project() -> (Project, PageId, PageId, Vec<RepresentationId>) {
    let mut project = Project::new("Macros").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let mut pages = Vec::new();
    for i in 1..=2 {
        pages.push(
            project
                .insert_page(
                    Page::new(
                        project.id(),
                        doc,
                        format!("Página {i}"),
                        format!("{i}"),
                        PageKind::MultiLine,
                        Size2D::new(420.0, 297.0).unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap(),
        );
    }
    let page = pages[0];
    let device = new_device(&project, "Q");
    let q1 = place(
        &mut project,
        page,
        "switch",
        device,
        FunctionKind::MainContact,
        100.0,
        200.0,
    );
    let device = new_device(&project, "F");
    let f1 = place(
        &mut project,
        page,
        "contact-nc",
        device,
        FunctionKind::AuxiliaryContact,
        100.0,
        160.0,
    );
    let device = new_device(&project, "K");
    let k1 = place(
        &mut project,
        page,
        "coil",
        device,
        FunctionKind::Coil,
        100.0,
        120.0,
    );
    let RepresentationTarget::Function(f) = project.representation(k1).unwrap().target() else {
        unreachable!()
    };
    let k1_device = project.function(f).unwrap().device_id();
    let aux = place(
        &mut project,
        page,
        "contact-no",
        PlacementDevice::Existing(k1_device),
        FunctionKind::AuxiliaryContact,
        140.0,
        160.0,
    );
    route(
        &mut project,
        port(q1, "1"),
        port(f1, "2"),
        vec![p(100.0, 190.0), p(100.0, 170.0)],
    );
    let r1 = route(
        &mut project,
        port(f1, "1"),
        port(k1, "2"),
        vec![p(100.0, 150.0), p(100.0, 130.0)],
    );
    let r2 = route(
        &mut project,
        port(aux, "1"),
        port(k1, "2"),
        vec![
            p(140.0, 150.0),
            p(140.0, 140.0),
            p(100.0, 140.0),
            p(100.0, 130.0),
        ],
    );
    project
        .edit_connection_drawing(ConnectionDrawingEdit::CreateJunction {
            page,
            position: p(100.0, 140.0),
            routes: vec![r1, r2],
        })
        .unwrap();
    (project, pages[0], pages[1], vec![q1, f1, k1, aux, r1, r2])
}
fn page_ids(project: &Project, page: PageId) -> Vec<RepresentationId> {
    project
        .representations()
        .filter(|r| r.page_id() == page)
        .map(Representation::id)
        .collect()
}
fn designations(project: &Project) -> Vec<String> {
    let mut names: Vec<_> = project
        .devices()
        .map(|d| d.designation().to_string())
        .collect();
    names.sort();
    names
}
fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cadcraft-macros-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

struct Ui {
    app: CadApp,
    ctx: Context,
}
impl Ui {
    fn new(project: Project) -> Self {
        let ctx = Context::default();
        ctx.enable_accesskit();
        let mut app = CadApp::default();
        app.session.replace(project, false).unwrap();
        let mut ui = Self { app, ctx };
        ui.settle(2);
        ui
    }
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let app = &mut self.app;
        self.ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
                events,
                ..Default::default()
            },
            |ctx| app.show(ctx),
        )
    }
    fn settle(&mut self, n: usize) -> egui::FullOutput {
        let mut out = self.frame(vec![]);
        for _ in 1..n {
            out = self.frame(vec![]);
        }
        out
    }
    fn press_at(&mut self, at: Pos2) {
        let ev = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(at)]);
        self.frame(vec![ev(true)]);
        self.frame(vec![ev(false)]);
        self.settle(2);
    }
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.frame(vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }]);
        self.frame(vec![Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers,
        }]);
    }
    fn click(&mut self, label: &str) {
        let out = self.settle(2);
        let at = nodes(&out)
            .into_iter()
            .find(|n| n.0 == label && n.2 != Role::Label)
            .or_else(|| nodes(&out).into_iter().find(|n| n.0 == label))
            .map(|n| n.1)
            .unwrap_or_else(|| {
                panic!(
                    "widget {label:?}; disponíveis: {:?}",
                    nodes(&out).iter().map(|n| &n.0).collect::<Vec<_>>()
                )
            });
        self.press_at(at);
    }
    fn type_in_row(&mut self, label: &str, text: &str) {
        let out = self.settle(2);
        let all = nodes(&out);
        let field = all
            .iter()
            .filter(|n| n.0 == label)
            .find_map(|anchor| {
                all.iter()
                    .filter(|n| {
                        n.2 == Role::TextInput
                            && (n.1.y - anchor.1.y).abs() < 8.0
                            && n.1.x > anchor.1.x
                    })
                    .min_by(|a, b| a.1.x.total_cmp(&b.1.x))
                    .map(|n| n.1)
            })
            .unwrap_or_else(|| panic!("campo {label}"));
        self.press_at(field);
        self.key(Key::A, Modifiers::COMMAND);
        self.frame(vec![Event::Text(text.into())]);
        self.settle(2);
    }
    /// Clique primário na folha, deslocado do centro do editor.
    fn sheet_click(&mut self, dx: f32, dy: f32) {
        self.settle(2);
        let at = self.app.last_viewport.unwrap().center() + vec2(dx, dy);
        self.press_at(at);
    }
    fn project(&self) -> &Project {
        self.app.session.project().unwrap()
    }
}
fn nodes(out: &egui::FullOutput) -> Vec<(String, Pos2, Role)> {
    let Some(update) = &out.platform_output.accesskit_update else {
        return vec![];
    };
    update
        .nodes
        .iter()
        .filter_map(|(_, n)| {
            let b = n.bounds()?;
            Some((
                n.label()
                    .filter(|l| !l.is_empty())
                    .or_else(|| n.value())
                    .unwrap_or_default()
                    .to_owned(),
                Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32),
                n.role(),
            ))
        })
        .collect()
}

#[test]
fn macro_is_created_saved_and_inserted_twice_by_real_events() {
    let (project, source, target, _) = project();
    let dir = temp_dir("create");
    let mut ui = Ui::new(project);
    ui.app.preferences.macro_library = dir.to_string_lossy().into_owned();
    ui.app.session.open_page(source).unwrap();
    let ids = page_ids(ui.project(), source);
    ui.app.session.select_graphics(ids);
    let ctx = ui.ctx.clone();
    ui.app.execute(CommandId::MacroCreate, &ctx);
    ui.type_in_row("ID da macro", "PARTIDA.DIRETA");
    ui.type_in_row("Nome da macro", "Partida direta");
    ui.type_in_row("Categoria da macro", "Motores");
    ui.click("Criar da seleção");
    let file = dir.join("PARTIDA.DIRETA.cadmacro");
    assert!(file.exists(), "{}", ui.app.macro_library.message);
    let saved = persistence::read_macro_package(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(saved.content().routes().len(), 3);
    assert_eq!(saved.content().junctions().len(), 1);
    // Inserir duas vezes na outra página por cliques reais.
    ui.app.session.open_page(target).unwrap();
    ui.click("Partida direta · PARTIDA.DIRETA · v1");
    ui.click("Inserir na página");
    ui.app.macro_library.open = false;
    assert_eq!(ui.app.session.graphics.tool, Tool::InsertMacro);
    ui.sheet_click(-120.0, 0.0);
    ui.sheet_click(120.0, 0.0);
    assert_eq!(
        designations(ui.project()),
        [
            "-F1", "-F2", "-F3", "-K1", "-K2", "-K3", "-Q1", "-Q2", "-Q3"
        ]
    );
    let on_target = |p: &Project| {
        p.connection_routes()
            .filter(|r| r.page_id() == target)
            .count()
    };
    assert_eq!(on_target(ui.project()), 6);
    assert_eq!(
        ui.project()
            .connection_junctions()
            .filter(|j| j.page_id() == target)
            .count(),
        2
    );
    ui.project().validate().unwrap();
    // Ctrl+Z desfaz a segunda inserção inteira; Refazer a devolve.
    let after = ui.app.session.snapshot().unwrap();
    ui.key(Key::Escape, Modifiers::NONE);
    ui.key(Key::Z, Modifiers::CTRL);
    assert_eq!(on_target(ui.project()), 3);
    assert!(!designations(ui.project()).contains(&"-K3".to_owned()));
    ui.app.execute(CommandId::Redo, &ctx);
    assert_eq!(ui.app.session.snapshot().unwrap(), after);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rotating_during_insertion_keeps_wires_on_ports() {
    let (project, source, target, _) = project();
    let mut ui = Ui::new(project);
    ui.app.session.open_page(source).unwrap();
    let ids = page_ids(ui.project(), source);
    ui.app.session.select_graphics(ids);
    let (definition, report) = ui
        .app
        .session
        .macro_from_selection("M", "M", "", "", 4.0)
        .unwrap();
    assert!(report.is_clean());
    // Ponto base ajustado à grade de 4 mm.
    assert_eq!(definition.base().x() % 4.0, 0.0);
    ui.app.session.open_page(target).unwrap();
    ui.app.start_macro_insertion(definition);
    let ctx = ui.ctx.clone();
    ui.app.execute(CommandId::Rotate90, &ctx);
    ui.app.execute(CommandId::MirrorHorizontal, &ctx);
    let draft = ui.app.session.graphics.macro_draft.clone().unwrap();
    assert_eq!((draft.rotation, draft.mirror), (90.0, true));
    ui.sheet_click(0.0, 0.0);
    let p = ui.project();
    let routes: Vec<_> = p
        .connection_routes()
        .filter(|r| r.page_id() == target)
        .collect();
    assert_eq!(routes.len(), 3);
    for r in routes {
        assert_eq!(r.points()[0], p.port_position(r.source()).unwrap());
        assert_eq!(
            *r.points().last().unwrap(),
            p.port_position(r.target()).unwrap()
        );
    }
    p.validate().unwrap();
}

#[test]
fn copy_paste_and_duplicate_carry_internal_wires() {
    let (project, source, target, ids) = project();
    let mut ui = Ui::new(project);
    ui.app.session.open_page(source).unwrap();
    // -F1, bobina, contato auxiliar e os dois fios entre eles (com junção).
    ui.app
        .session
        .select_graphics([ids[1], ids[2], ids[3], ids[4], ids[5]]);
    ui.app.session.capture_selection(false).unwrap();
    assert!(ui.app.session.clipboard.as_ref().unwrap().is_wired());
    ui.app.session.open_page(target).unwrap();
    let before = ui.app.session.snapshot().unwrap();
    let connections = ui.project().connections().count();
    ui.app.session.confirm_paste(p(200.0, 100.0)).unwrap();
    let p2 = ui.project();
    assert_eq!(
        p2.connection_routes()
            .filter(|r| r.page_id() == target)
            .count(),
        2
    );
    assert_eq!(
        p2.connection_junctions()
            .filter(|j| j.page_id() == target)
            .count(),
        1
    );
    assert!(p2.connections().count() > connections);
    assert!(designations(p2).contains(&"-K2".to_owned()));
    // Um Desfazer remove a colagem inteira.
    ui.app.session.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
    // Colar no mesmo dispositivo com fios é recusado sem alterar nada.
    ui.app.session.clipboard.as_mut().unwrap().keep_device = true;
    assert!(ui.app.session.confirm_paste(p(200.0, 100.0)).is_err());
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
    // Ctrl+D com fios duplica fios e junção na mesma página.
    ui.app.session.open_page(source).unwrap();
    ui.app
        .session
        .select_graphics([ids[1], ids[2], ids[3], ids[4], ids[5]]);
    let routes = ui.project().connection_routes().count();
    ui.app.session.duplicate_selection().unwrap();
    assert_eq!(ui.project().connection_routes().count(), routes + 2);
    ui.project().validate().unwrap();
}

#[test]
fn page_macro_is_created_and_inserted_as_new_page_by_real_events() {
    let (project, source, _, _) = project();
    let mut ui = Ui::new(project);
    ui.app.session.open_page(source).unwrap();
    let ctx = ui.ctx.clone();
    ui.app.execute(CommandId::MacroPage, &ctx);
    ui.type_in_row("ID da macro", "PAGINA.PARTIDA");
    ui.type_in_row("Nome da macro", "Página de partida");
    ui.click("Criar da página ativa");
    assert!(
        ui.app
            .macro_library
            .selected()
            .is_some_and(MacroDefinition::is_page),
        "{}",
        ui.app.macro_library.message
    );
    let pages = ui.project().pages().count();
    ui.click("Inserir como página nova");
    assert_eq!(ui.project().pages().count(), pages + 1);
    let new_page = ui.app.session.active.unwrap();
    assert_ne!(new_page, source);
    assert_eq!(
        page_ids(ui.project(), new_page).len(),
        page_ids(ui.project(), source).len()
    );
    ui.app.execute(CommandId::Undo, &ctx);
    assert_eq!(ui.project().pages().count(), pages);
}

#[test]
fn invalid_macro_file_is_refused_without_changes() {
    let (project, source, _, _) = project();
    let dir = temp_dir("invalid");
    let (content, _) = project.capture_macro(&page_ids(&project, source)).unwrap();
    let good = MacroDefinition::new("BOA", "Boa", MacroKind::Window, p(0.0, 0.0), content).unwrap();
    std::fs::write(
        dir.join("BOA.cadmacro"),
        persistence::write_macro_package(&good).unwrap(),
    )
    .unwrap();
    std::fs::write(dir.join("RUIM.cadmacro"), b"PK\x03\x04 truncado").unwrap();
    let mut ui = Ui::new(project);
    ui.app.preferences.macro_library = dir.to_string_lossy().into_owned();
    let ctx = ui.ctx.clone();
    ui.app.execute(CommandId::MacroNavigator, &ctx);
    ui.settle(3);
    assert!(ui.app.macro_library.message.contains("RUIM"));
    assert_eq!(ui.app.macro_library.catalog().count(), 1);
    let before = ui.app.session.snapshot().unwrap();
    ui.type_in_row("Importar de", &dir.join("RUIM.cadmacro").to_string_lossy());
    ui.click("Importar macro");
    assert!(
        ui.app
            .macro_library
            .message
            .starts_with("Operação recusada")
    );
    assert_eq!(ui.app.macro_library.catalog().count(), 1);
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn macro_commands_live_in_insert_and_master_data() {
    let tab = |id| registry().iter().find(|c| c.id == id).unwrap().tab;
    assert_eq!(tab(CommandId::MacroNavigator), RibbonTab::Insert);
    assert_eq!(tab(CommandId::MacroInsert), RibbonTab::Insert);
    assert_eq!(tab(CommandId::MacroCreate), RibbonTab::MasterData);
    assert_eq!(tab(CommandId::MacroPage), RibbonTab::Home);
}
