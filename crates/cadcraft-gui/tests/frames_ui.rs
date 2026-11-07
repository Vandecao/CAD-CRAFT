// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! GUI frame acceptance tests: factory frames in new projects, A3/A4 pages, special text and zone labels, column references on the sheet and inspector, page-frame replacement with undo/redo and persistence.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp,
    camera::Camera,
    commands::CommandId,
    editing::{Edit, GraphicEdit},
    interaction::DevicePlan,
    project_file,
    state::Selection,
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
    fn new() -> Self {
        let ctx = Context::default();
        ctx.enable_accesskit();
        let mut ui = Self {
            app: CadApp::default(),
            ctx,
        };
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
    fn click(&mut self, at: Pos2) -> egui::FullOutput {
        let b = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(at), b(true)]);
        self.frame(vec![b(false)]);
        self.settle(3)
    }
    fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.frame(vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }]);
        self.settle(3);
    }
    fn press(&mut self, label: &str) {
        let out = self.settle(1);
        let at = find(&out, label);
        self.click(at);
    }
    fn type_below(&mut self, label: &str, text: &str) {
        let out = self.settle(1);
        let confirm = find(&out, "Confirmar");
        let near = |q: &Pos2| (q.x - confirm.x).abs() < 260. && q.y < confirm.y;
        let anchor = nodes(&out)
            .into_iter()
            .find(|(l, q, _)| l == label && near(q))
            .map(|n| n.1)
            .unwrap();
        let field = nodes(&out)
            .into_iter()
            .filter(|(_, q, r)| *r == Role::TextInput && q.y > anchor.y && near(q))
            .min_by(|a, b| (a.1.y - anchor.y).total_cmp(&(b.1.y - anchor.y)))
            .map(|n| n.1)
            .unwrap();
        self.click(field);
        self.frame(vec![Event::Text(text.into())]);
        self.settle(2);
    }
    fn project(&self) -> &Project {
        self.app.session.project().unwrap()
    }
    fn view(&mut self, page: PageId, center: [f64; 2], scale: f64) {
        self.app.session.open_page(page).unwrap();
        self.app.preferences.grid_visible = false;
        self.app.preferences.grid_mm = 4.;
        self.app.preferences.snap_enabled = true;
        let tab = self.app.session.active_tab_mut().unwrap();
        tab.camera = Camera::new(center, scale).unwrap();
        tab.needs_fit = false;
        self.settle(2);
    }
    fn click_world(&mut self, point: Point2D) {
        let vp = self.app.last_viewport.unwrap();
        let at = self
            .app
            .session
            .active_tab_mut()
            .unwrap()
            .camera
            .world_to_screen(point, vp)
            .unwrap();
        self.click(at);
    }
    fn place(&mut self, key: &str, plan: DevicePlan, at: Point2D) -> RepresentationId {
        self.app
            .session
            .begin_device_insertion(
                SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap(),
                plan,
                Transform2D::new(p(0., 0.), 0., false, false).unwrap(),
            )
            .unwrap();
        self.settle(1);
        self.click_world(at);
        self.key(Key::Escape, Modifiers::NONE);
        self.app.session.graphics.selection.primary().unwrap()
    }
    /// Text painted within the drawing area.
    fn sheet_texts(&mut self) -> Vec<String> {
        let out = self.settle(1);
        let vp = self.app.last_viewport.unwrap();
        out.shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) if vp.contains(t.pos) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }
    fn panel_texts(&mut self) -> Vec<String> {
        let out = self.settle(2);
        out.shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect()
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
fn find(out: &egui::FullOutput, label: &str) -> Pos2 {
    nodes(out)
        .into_iter()
        .find(|n| n.0 == label)
        .map(|n| n.1)
        .unwrap_or_else(|| panic!("widget {label:?} ausente"))
}

/// Factory frames, special text, references, assignment and persistence.
#[test]
fn new_project_frames_zones_special_texts_and_column_references() {
    let mut ui = Ui::new();
    ui.press("Criar sessão em memória");
    ui.type_below("Nome do projeto", "Painel CCM");
    ui.press("Confirmar");
    let a3 = ui
        .project()
        .frames()
        .find(|f| f.name().as_str() == "CAD-CRAFT A3 paisagem")
        .unwrap()
        .id();
    let a4 = ui
        .project()
        .frames()
        .find(|f| f.name().as_str() == "CAD-CRAFT A4 retrato")
        .unwrap()
        .id();
    assert_eq!(ui.project().default_frame(), Some(a3));
    let doc = ui.app.session.new_document("Esquema").unwrap();
    let force = ui
        .app
        .session
        .new_page(
            doc,
            "Força",
            "1",
            PageKind::MultiLine,
            PaperFormat::A3.size(true),
        )
        .unwrap();
    let cmd = ui
        .app
        .session
        .new_page(
            doc,
            "Comando",
            "2",
            PageKind::MultiLine,
            PaperFormat::A4.size(false),
        )
        .unwrap();
    // A3 inherits the default; A4 receives the matching frame.
    assert_eq!(ui.project().page(force).unwrap().frame(), None);
    assert_eq!(ui.project().page_frame(force).unwrap().id(), a3);
    assert_eq!(ui.project().page(cmd).unwrap().frame(), Some(a4));

    // K1 coil on page 1, column 4 (x 128..169), and contact on page 2, column 2.
    ui.view(force, [150., 150.], 2.);
    let coil = ui.place(
        "coil",
        DevicePlan::New {
            prefix: String::new(),
        },
        p(148., 160.),
    );
    let k1 = ui
        .project()
        .device_for_target(ui.project().representation(coil).unwrap().target())
        .unwrap();
    ui.view(cmd, [100., 150.], 2.);
    ui.place("contact-no", DevicePlan::Existing(k1), p(60., 160.));
    let refs = ui.project().device_references(k1).unwrap();
    let labels: Vec<_> = refs.iter().map(|r| r.reference.clone()).collect();
    assert_eq!(labels, vec!["/1.4".to_string(), "/2.2".to_string()]);

    ui.view(cmd, [105., 148.5], 1.5);
    // Page 2 shows column labels, special text and the main coil reference below the contact.
    let texts = ui.sheet_texts();
    for expected in [
        "1",
        "5",
        "Painel CCM",
        "Comando",
        "Esquema",
        "/1.4",
        "CAD-CRAFT A4 retrato",
    ] {
        assert!(texts.iter().any(|t| t == expected), "{expected}: {texts:?}");
    }
    // On page 1 the coil lists the contact.
    ui.view(force, [210., 148.5], 1.5);
    assert!(ui.sheet_texts().iter().any(|t| t == "/2.2"));
    // The device-properties panel shows the column reference.
    ui.app
        .session
        .select(Selection::Electrical(RepresentationTarget::Device(k1)));
    let panel = ui.panel_texts();
    assert!(panel.iter().any(|t| t.contains("/2.2")), "{panel:?}");

    // Renumbering the page updates special text and references.
    ui.app
        .session
        .apply(Edit::PageNumber(cmd, "20".into()))
        .unwrap();
    assert_eq!(
        ui.project().device_references(k1).unwrap()[1].reference,
        "/20.2"
    );
    assert!(ui.sheet_texts().iter().any(|t| t == "/20.2"));

    // Changing the page frame recalculates the reference; Ctrl+Z/Ctrl+Y restore exact states.
    let before = ui.project().clone();
    ui.app
        .graphic_edit(GraphicEdit::Frame(FrameEdit::SetPageFrame {
            page: cmd,
            frame: Some(a3),
        }));
    assert_eq!(
        ui.project().device_references(k1).unwrap()[1].reference,
        "/20.2"
    );
    ui.app
        .graphic_edit(GraphicEdit::Frame(FrameEdit::SetReferenceFormat(
            ReferenceFormat::PageColumnRow,
        )));
    let after = ui.project().clone();
    ui.app.execute(CommandId::Undo, &ui.ctx.clone());
    ui.app.execute(CommandId::Undo, &ui.ctx.clone());
    assert_eq!(ui.project(), &before);
    ui.app.execute(CommandId::Redo, &ui.ctx.clone());
    ui.app.execute(CommandId::Redo, &ui.ctx.clone());
    assert_eq!(ui.project(), &after);

    // Saving and reopening preserves frames and assignments.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ccm.cadcraft");
    project_file::save(ui.project(), &path).unwrap();
    let loaded = project_file::load(&path).unwrap();
    assert_eq!(loaded.source_format, cad_craft::persistence::FORMAT);
    assert_eq!(&loaded.project, ui.project());
}

/// Properties-panel selectors for the page frame and project metadata.
#[test]
fn properties_panel_assigns_page_frame_and_edits_project_metadata() {
    let mut ui = Ui::new();
    let project = cad_gui::state::with_factory_frames(Project::new("Meta").unwrap()).unwrap();
    ui.app.session.replace(project, false).unwrap();
    let doc = ui.app.session.new_document("Esquema").unwrap();
    let page = ui
        .app
        .session
        .new_page(
            doc,
            "Força",
            "1",
            PageKind::MultiLine,
            PaperFormat::A3.size(true),
        )
        .unwrap();
    // Select a page, open the frame selector and choose A4.
    ui.app.session.select(Selection::Page(page));
    let out = ui.settle(3);
    let label = find(&out, "Moldura");
    let combo = nodes(&out)
        .into_iter()
        .filter(|n| (n.1.y - label.y).abs() < 8. && n.1.x > label.x + 10.)
        .min_by(|a, b| a.1.x.total_cmp(&b.1.x))
        .map(|n| n.1)
        .unwrap();
    let out = ui.click(combo);
    let option = find(&out, "CAD-CRAFT A4 retrato");
    ui.click(option);
    let a4 = ui
        .project()
        .frames()
        .find(|f| f.name().as_str() == "CAD-CRAFT A4 retrato")
        .unwrap()
        .id();
    assert_eq!(ui.project().page(page).unwrap().frame(), Some(a4));
    // The page adopts the frame size, so there is no size warning.
    assert_eq!(
        ui.project().page(page).unwrap().size(),
        PaperFormat::A4.size(false)
    );
    assert!(
        !ui.panel_texts()
            .iter()
            .any(|t| t == "Moldura com tamanho diferente da folha")
    );
    // One Ctrl+Z restores both the frame and the size.
    ui.key(Key::Z, Modifiers::CTRL);
    assert_eq!(ui.project().page(page).unwrap().frame(), None);
    assert_eq!(
        ui.project().page(page).unwrap().size(),
        PaperFormat::A3.size(true)
    );

    // The selected project's author field supplies the title block's special text.
    ui.app.session.select(Selection::Project);
    let out = ui.settle(3);
    let author = find(&out, "Autor");
    let field = nodes(&out)
        .into_iter()
        .filter(|n| n.2 == Role::TextInput && (n.1.y - author.y).abs() < 8.)
        .map(|n| n.1)
        .next()
        .unwrap();
    ui.click(field);
    ui.frame(vec![Event::Text("Ana".into())]);
    ui.key(Key::Enter, Modifiers::NONE);
    let key = Text::new("Autor").unwrap();
    assert_eq!(
        ui.project().metadata().get(&key).map(String::as_str),
        Some("Ana")
    );
    ui.view(page, [210., 40.], 3.);
    assert!(ui.sheet_texts().iter().any(|t| t == "Ana"));
    ui.key(Key::Z, Modifiers::CTRL);
    assert!(ui.project().metadata().get(&key).is_none());
}
