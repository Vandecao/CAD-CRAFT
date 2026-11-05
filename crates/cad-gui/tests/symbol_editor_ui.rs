// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 17 — editor de símbolos, biblioteca externa e atualização do
//! projeto, com eventos egui reais onde o critério pede.
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp, camera::Camera, commands::CommandId, editing::GraphicEdit,
    interaction::DevicePlan, project_file::ScriptedPicker, symbol_editor::SymbolTool,
};
use eframe::egui::{
    self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, vec2,
};

fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).unwrap()
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
    fn press_at(&mut self, at: Pos2) -> egui::FullOutput {
        let b = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(at)]);
        self.frame(vec![b(true)]);
        self.frame(vec![b(false)]);
        self.settle(2)
    }
    fn press(&mut self, label: &str) {
        let out = self.settle(1);
        let at = nodes(&out)
            .into_iter()
            .find(|n| n.0 == label && n.1.y > 140.0)
            .map(|n| n.1)
            .unwrap_or_else(|| panic!("botão {label:?}"));
        self.press_at(at);
    }
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.frame(vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }]);
        self.settle(2);
    }
    fn screen(&self, w: Point2D) -> Pos2 {
        let vp = self.app.symbol_editor.viewport.unwrap();
        self.app
            .symbol_editor
            .camera
            .world_to_screen(w, vp)
            .unwrap()
    }
    fn click_world(&mut self, w: Point2D) {
        let at = self.screen(w);
        self.press_at(at);
    }
    fn drag_world(&mut self, from: Point2D, to: Point2D) {
        let (a, b) = (self.screen(from), self.screen(to));
        let ev = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(a)]);
        self.frame(vec![ev(a, true)]);
        for i in 1..=6 {
            let t = i as f32 / 6.0;
            self.frame(vec![Event::PointerMoved(a + (b - a) * t)]);
        }
        self.frame(vec![ev(b, false)]);
        self.settle(2);
    }
    fn tool(&mut self, tool: SymbolTool) {
        let out = self.settle(1);
        let vp = self.app.symbol_editor.viewport.unwrap();
        let at = nodes(&out)
            .into_iter()
            .filter(|n| n.0 == tool.label() && n.1.y > 140.0 && n.1.y < vp.top() && n.1.x < 1400.0)
            .map(|n| n.1)
            .next()
            .unwrap_or_else(|| panic!("ferramenta {tool:?}"));
        self.press_at(at);
        assert_eq!(self.app.symbol_editor.tool, tool);
    }
    fn type_in_row(&mut self, label: &str, text: &str) {
        let out = self.settle(1);
        let right = self.app.symbol_editor.viewport.unwrap().right();
        let all = nodes(&out);
        let field = all
            .iter()
            .filter(|n| n.0 == label && n.1.x > right)
            .find_map(|anchor| {
                all.iter()
                    .find(|n| n.2 == Role::TextInput && (n.1.y - anchor.1.y).abs() < 8.0)
                    .map(|n| n.1)
            })
            .unwrap_or_else(|| panic!("campo {label}"));
        self.press_at(field);
        self.key(Key::A, Modifiers::COMMAND);
        self.frame(vec![Event::Text(text.into())]);
        self.settle(2);
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
fn base_project() -> (Project, PageId) {
    let mut project = Project::new("Símbolos").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Comando",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    (project, page)
}
fn sheet_texts(ui: &mut Ui) -> Vec<String> {
    let out = ui.settle(2);
    out.shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
            _ => None,
        })
        .collect()
}
fn place(ui: &mut Ui, page: PageId, id: &SymbolDefinitionId, at: (f64, f64)) -> RepresentationId {
    ui.app.session.open_page(page).unwrap();
    ui.app
        .session
        .dispatch_graphic(GraphicEdit::PlaceSymbol {
            definition: id.clone(),
            plan: DevicePlan::New { prefix: "K".into() },
            transform: Transform2D::new(p(at.0, at.1), 0.0, false, false).unwrap(),
        })
        .unwrap();
    ui.app.session.graphics.selection.primary().unwrap()
}

/// Critério 1: símbolo criado do zero pela interface, salvo na biblioteca,
/// inserido com "Novo dispositivo" e mostrando -K1 no texto dinâmico.
#[test]
fn symbol_created_from_scratch_is_saved_inserted_and_shows_designation() {
    let (project, page) = base_project();
    let mut ui = Ui::new(project);
    ui.app.execute(CommandId::SymbolNew, &ui.ctx.clone());
    ui.settle(3);
    assert!(ui.app.symbol_editor.is_open());
    ui.tool(SymbolTool::Rectangle);
    ui.click_world(p(-4.0, -6.0));
    ui.click_world(p(4.0, 6.0));
    assert_eq!(ui.app.symbol_editor.primitives().len(), 1);
    // Desfazer/Refazer do editor.
    ui.key(Key::Z, Modifiers::COMMAND);
    assert!(ui.app.symbol_editor.primitives().is_empty());
    ui.key(Key::Y, Modifiers::COMMAND);
    assert_eq!(ui.app.symbol_editor.primitives().len(), 1);
    ui.tool(SymbolTool::Port);
    ui.click_world(p(0.0, 10.0));
    ui.click_world(p(0.0, -10.0));
    assert_eq!(ui.app.symbol_editor.ports().len(), 2);
    ui.tool(SymbolTool::Ellipse);
    ui.click_world(p(-3.0, -2.0));
    ui.click_world(p(3.0, 2.0));
    ui.tool(SymbolTool::DynamicText);
    ui.click_world(p(6.0, 0.0));
    assert!(matches!(
        ui.app.symbol_editor.primitives().last(),
        Some(SymbolPrimitive::DynamicText {
            field: SymbolTextField::Designation,
            ..
        })
    ));
    // Selecionar e mover o retângulo pela borda (grade de 1 mm).
    ui.tool(SymbolTool::Select);
    ui.drag_world(p(-4.0, 0.0), p(-3.0, 0.0));
    let SymbolPrimitive::Rectangle { rect } = &ui.app.symbol_editor.primitives()[0] else {
        panic!("retângulo")
    };
    assert!((rect.origin().x() - -3.0).abs() < 1e-9, "{rect:?}");
    ui.type_in_row("Nome", "Bobina teste");
    ui.press("Salvar na biblioteca");
    let def = ui
        .app
        .session
        .symbol_library
        .iter()
        .find(|d| d.name() == "Bobina teste")
        .expect("símbolo na biblioteca da sessão")
        .clone();
    assert_eq!(def.version(), 1);
    assert_eq!(def.ports().len(), 2);
    ui.press("Fechar editor");
    assert!(!ui.app.symbol_editor.is_open());
    // Aparece no catálogo do Centro de inserção.
    assert!(
        ui.app
            .session
            .symbol_catalog()
            .iter()
            .any(|d| d.id() == def.id())
    );
    let id = place(&mut ui, page, def.id(), (100.0, 100.0));
    let tab = ui.app.session.active_tab_mut().unwrap();
    tab.camera = Camera::new([100.0, 100.0], 5.0).unwrap();
    tab.needs_fit = false;
    let texts = sheet_texts(&mut ui);
    assert!(texts.iter().any(|t| t == "-K1"), "{texts:?}");
    assert!(
        ui.project()
            .port_bindings()
            .filter(|(port, _)| port.representation_id() == id)
            .count()
            == 2
    );
}

/// Critério 3: editar símbolo em uso → nova versão; porta removida recusa.
#[test]
fn editing_a_used_symbol_updates_project_or_refuses_missing_ports() {
    let (project, page) = base_project();
    let mut ui = Ui::new(project);
    let coil = SymbolLibrary::builtin()
        .definitions()
        .find(|d| d.ports().len() == 2)
        .unwrap()
        .clone();
    let rid = place(&mut ui, page, coil.id(), (100.0, 100.0));
    let before = ui.app.session.snapshot().unwrap();
    ui.app.open_symbol_editor(coil.clone());
    ui.settle(3);
    ui.tool(SymbolTool::Ellipse);
    ui.click_world(p(-2.0, -1.0));
    ui.click_world(p(2.0, 1.0));
    ui.press("Atualizar no projeto");
    let now = ui.project().symbol_definition(coil.id()).unwrap().clone();
    assert_eq!(now.version(), coil.version() + 1);
    assert_eq!(
        ui.project().representation(rid).unwrap().local_bounds(),
        Some(now.bounds())
    );
    // Porta removida: recusa sem alterar.
    let used = ui.project().clone();
    ui.tool(SymbolTool::Select);
    let port = ui.app.symbol_editor.ports()[0].position();
    ui.click_world(port);
    ui.key(Key::Delete, Modifiers::NONE);
    assert_eq!(ui.app.symbol_editor.ports().len(), 1);
    ui.press("Atualizar no projeto");
    assert_eq!(ui.project(), &used);
    ui.press("Fechar editor");
    // Um passo de Desfazer volta à versão original.
    ui.key(Key::Z, Modifiers::CTRL);
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
    ui.key(Key::Y, Modifiers::CTRL);
    assert_eq!(ui.project().symbol_definition(coil.id()), Some(&now));
}

/// Critérios 4 e 5 na interface: pasta da biblioteca, `.cadsym`, `.elmt` e
/// "Atualizar símbolos do projeto".
#[test]
fn library_folder_imports_exports_and_updates_project_symbols() {
    let dir = tempfile::tempdir().unwrap();
    let lib = dir.path().join("biblioteca");
    let (project, page) = base_project();
    let mut ui = Ui::new(project);
    ui.app.preferences.symbol_library = lib.display().to_string();
    ui.settle(2);
    // Novo símbolo salvo na pasta.
    ui.app.open_symbol_editor_new();
    ui.settle(2);
    ui.tool(SymbolTool::Circle);
    ui.click_world(p(0.0, 0.0));
    ui.click_world(p(3.0, 0.0));
    ui.tool(SymbolTool::Port);
    ui.click_world(p(0.0, 5.0));
    ui.app.symbol_editor.set_name("Lâmpada");
    ui.press("Salvar na biblioteca");
    let files: Vec<_> = std::fs::read_dir(&lib).unwrap().flatten().collect();
    assert_eq!(files.len(), 1);
    let lamp = ui
        .app
        .session
        .symbol_library
        .iter()
        .find(|d| d.name() == "Lâmpada")
        .unwrap()
        .clone();
    ui.press("Fechar editor");
    // Usa no projeto, depois a biblioteca recebe v2 compatível.
    place(&mut ui, page, lamp.id(), (50.0, 50.0));
    let v2 = lamp
        .edited(
            "Lâmpada",
            "Usuário",
            [
                lamp.primitives().to_vec(),
                vec![SymbolPrimitive::Line {
                    start: p(-3.0, 0.0),
                    end: p(3.0, 0.0),
                }],
            ]
            .concat(),
            lamp.ports().to_vec(),
            lamp.styles().clone(),
        )
        .unwrap();
    std::fs::write(
        lib.join("Lâmpada.cadsym"),
        cad_craft::persistence::write_symbol_package(&v2).unwrap(),
    )
    .unwrap();
    ui.app.execute(CommandId::SymbolReload, &ui.ctx.clone());
    ui.app
        .execute(CommandId::SymbolUpdateProject, &ui.ctx.clone());
    assert_eq!(
        ui.project().symbol_definition(lamp.id()).unwrap().version(),
        2
    );
    // Importar .elmt (arquivo sintético) para a pasta.
    let elmt = dir.path().join("sinaleiro.elmt");
    std::fs::write(
        &elmt,
        r#"<definition type="element"><names><name lang="pt">Sinaleiro QET</name></names><description>
            <ellipse x="-5" y="-5" width="10" height="10" style="line-style:normal;line-weight:normal;filling:none;color:black"/>
            <terminal name="1" x="0" y="-5" orientation="n"/><terminal name="2" x="0" y="5" orientation="s"/>
        </description></definition>"#,
    )
    .unwrap();
    let export = dir.path().join("exportado.cadsym");
    let copy = dir.path().join("copia.cadsym");
    std::fs::write(
        &copy,
        cad_craft::persistence::write_symbol_package(
            &v2.with_identity(SymbolDefinitionId::new("teste.copia").unwrap(), "Cópia")
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    ui.app.files.picker = Box::new(ScriptedPicker {
        elmt_open: [vec![elmt]].into(),
        symbols_save: [export.clone()].into(),
        symbols_open: [copy].into(),
        ..Default::default()
    });
    ui.app.execute(CommandId::SymbolImportElmt, &ui.ctx.clone());
    assert!(
        ui.app
            .session
            .symbol_library
            .iter()
            .any(|d| d.name() == "Sinaleiro QET")
    );
    ui.app.execute(CommandId::SymbolImport, &ui.ctx.clone());
    assert!(
        ui.app
            .session
            .symbol_library
            .iter()
            .any(|d| d.name() == "Cópia")
    );
    // Exportar o símbolo selecionado na página.
    ui.app.execute(CommandId::SymbolExport, &ui.ctx.clone());
    let exported =
        cad_craft::persistence::read_symbol_package(&std::fs::read(&export).unwrap()).unwrap();
    assert_eq!(exported.id(), lamp.id());
    assert_eq!(std::fs::read_dir(&lib).unwrap().count(), 3);
}
