// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frame-editor acceptance tests using real egui events: create geometry, text, special text and a logo; edit an assigned frame; undo/redo; export/import .cadframe and reject invalid images.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp, camera::Camera, commands::CommandId, editing::GraphicEdit,
    frame_editor::FrameTool, frame_view, interaction::DevicePlan, project_file::ScriptedPicker,
};
use eframe::egui::{
    self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, vec2,
};
use std::path::Path;

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
        app.preferences.grid_mm = 5.0;
        app.preferences.snap_enabled = true;
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
    fn press(&mut self, label: &str) -> egui::FullOutput {
        let out = self.settle(1);
        let at = find(&out, label);
        self.press_at(at)
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
        let vp = self.app.frame_editor.viewport.unwrap();
        self.app.frame_editor.camera.world_to_screen(w, vp).unwrap()
    }
    fn click_world(&mut self, w: Point2D) {
        let at = self.screen(w);
        self.press_at(at);
    }
    /// Real primary-button drag in the editor.
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
    fn tool(&mut self, tool: FrameTool) {
        // Labels such as the text tool also appear in the ribbon; use the editor toolbar.
        let out = self.settle(1);
        let at = nodes(&out)
            .into_iter()
            .filter(|n| n.0 == tool.label() && n.1.y > 150.0 && n.1.x < 1300.0)
            .map(|n| n.1)
            .next()
            .unwrap_or_else(|| panic!("ferramenta {:?}", tool));
        self.press_at(at);
        assert_eq!(self.app.frame_editor.tool, tool);
    }
    fn tool_try(&mut self, tool: FrameTool) {
        let out = self.settle(1);
        let at = nodes(&out)
            .into_iter()
            .filter(|n| n.0 == tool.label() && n.1.y > 150.0 && n.1.x < 1300.0)
            .map(|n| n.1)
            .next()
            .unwrap();
        self.press_at(at);
    }
    fn frame_now(&self, id: FrameTemplateId) -> FrameTemplate {
        self.project().frame(id).unwrap().clone()
    }
    fn project(&self) -> &Project {
        self.app.session.project().unwrap()
    }
    /// Choose an option from an editor-properties selector.
    fn choose(&mut self, current: &str, option: &str) {
        let out = self.settle(1);
        let combo = nodes(&out)
            .into_iter()
            .find(|n| {
                n.0 == current
                    && n.2 == Role::ComboBox
                    && n.1.x > self.app.frame_editor.viewport.unwrap().right()
            })
            .unwrap_or_else(|| panic!("seletor {current}"))
            .1;
        let out = self.press_at(combo);
        let at = find(&out, option);
        self.press_at(at);
    }
    /// Replace text in the field on the specified label's row.
    fn type_in_row(&mut self, label: &str, text: &str) {
        let out = self.settle(1);
        let right = self.app.frame_editor.viewport.unwrap().right();
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
fn base_project() -> (Project, PageId) {
    let mut project =
        cad_gui::state::with_factory_frames(Project::new("Painel CCM").unwrap()).unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Força",
                "1",
                PageKind::MultiLine,
                PaperFormat::A3.size(true),
            )
            .unwrap(),
        )
        .unwrap();
    project
        .edit_frame(FrameEdit::SetPageMetadata {
            page,
            key: Text::new("Data").unwrap(),
            value: Some("03/11/2026".into()),
        })
        .unwrap();
    (project, page)
}
fn write_png(path: &Path, w: u32, h: u32) -> Vec<u8> {
    let mut img = image::RgbaImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        *px = image::Rgba([(x * 4) as u8, (y * 8) as u8, 120, 255]);
    }
    img.save(path).unwrap();
    std::fs::read(path).unwrap()
}
fn sheet_texts(ui: &mut Ui) -> Vec<String> {
    let out = ui.settle(2);
    let vp = ui.app.last_viewport.unwrap();
    out.shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) if vp.contains(t.pos) => Some(t.galley.text().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn external_frame_preview_confirm_assign_and_undo() {
    let (project, page) = base_project();
    let mut ui = Ui::new(project);
    ui.app.session.open_page(page).unwrap();
    ui.settle(2);
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../cad-craft/tests/fixtures/synthetic/fn1/moldura_origem_deslocada.fn1");
    ui.app.files.picker = Box::new(ScriptedPicker {
        external_open: [fixture].into(),
        ..Default::default()
    });
    let before = ui.app.session.project().unwrap().frames().count();
    ui.app
        .execute(CommandId::FrameImportExternal, &ui.ctx.clone());
    assert_eq!(ui.app.session.project().unwrap().frames().count(), before);
    ui.settle(1);
    ui.press("Confirmar");
    let imported = ui
        .app
        .frame_editor
        .frame
        .expect("editor de moldura importada aberto");
    let project = ui.app.session.project().unwrap();
    assert!(project.frame(imported).is_some());
    assert_eq!(
        project.page_frame(page).map(FrameTemplate::id),
        Some(imported)
    );
    ui.app.graphic_edit(GraphicEdit::Undo);
    assert!(ui.app.session.project().unwrap().frame(imported).is_none());
    ui.app.graphic_edit(GraphicEdit::Redo);
    assert!(ui.app.session.project().unwrap().frame(imported).is_some());
}

#[test]
fn external_frame_cancel_keeps_project_and_dxf_export_writes_file() {
    let dir = tempfile::tempdir().unwrap();
    let dxf = dir.path().join("moldura.dxf");
    let (project, page) = base_project();
    let mut ui = Ui::new(project);
    ui.app.session.open_page(page).unwrap();
    ui.app.files.picker = Box::new(ScriptedPicker {
        external_open: [std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../cad-craft/tests/fixtures/synthetic/fn1/moldura_origem_deslocada.fn1")]
        .into(),
        dxf_save: [dxf.clone()].into(),
        ..Default::default()
    });
    let before = ui.app.session.project().unwrap().frames().count();
    ui.app
        .execute(CommandId::FrameImportExternal, &ui.ctx.clone());
    ui.settle(2);
    ui.press("Cancelar");
    assert_eq!(ui.app.session.project().unwrap().frames().count(), before);
    ui.app.execute(CommandId::FrameEditor, &ui.ctx.clone());
    ui.app.execute(CommandId::FrameExportDxf, &ui.ctx.clone());
    let bytes = std::fs::read(&dxf).unwrap();
    assert!(bytes.ends_with(b"EOF\n"));
    cad_craft::interchange::dxf::import(
        &bytes,
        &cad_craft::interchange::dxf::DxfImportOptions::new("prova"),
    )
    .unwrap();
}

#[test]
fn changing_dxf_options_requires_updated_preview() {
    let dir = tempfile::tempdir().unwrap();
    let dxf = dir.path().join("opcoes.dxf");
    let (project, page) = base_project();
    let frame = project.frames().next().unwrap();
    let (bytes, _) = cad_craft::interchange::dxf::export(frame);
    std::fs::write(&dxf, bytes).unwrap();
    let mut ui = Ui::new(project);
    ui.app.session.open_page(page).unwrap();
    ui.app.files.picker = Box::new(ScriptedPicker {
        external_open: [dxf].into(),
        ..Default::default()
    });
    let before = ui.app.session.project().unwrap().frames().count();
    ui.app
        .execute(CommandId::FrameImportExternal, &ui.ctx.clone());
    ui.settle(2);
    ui.press("Definir mm por unidade (em vez do cabeçalho)");
    ui.press("Confirmar");
    assert_eq!(ui.app.session.project().unwrap().frames().count(), before);
    ui.press("Atualizar prévia");
    ui.press("Confirmar");
    assert_eq!(
        ui.app.session.project().unwrap().frames().count(),
        before + 1
    );
}

/// Create a frame from scratch through the interface, including insertions.
#[test]
fn frame_is_created_from_scratch_with_texts_specials_and_logo() {
    let dir = tempfile::tempdir().unwrap();
    let logo = dir.path().join("logo.png");
    let logo_bytes = write_png(&logo, 64, 32);
    let (project, page) = base_project();
    let mut ui = Ui::new(project);
    ui.app.files.picker = Box::new(ScriptedPicker {
        images: [logo.clone()].into(),
        ..Default::default()
    });
    ui.app.execute(CommandId::FrameNew, &ui.ctx.clone());
    ui.settle(3);
    let id = ui.app.frame_editor.frame.expect("editor aberto");
    assert_eq!(ui.frame_now(id).elements().len(), 1); // border of the blank frame

    ui.tool(FrameTool::Line);
    ui.click_world(p(5.0, 40.0));
    ui.click_world(p(415.0, 40.0));
    ui.tool(FrameTool::Rectangle);
    ui.click_world(p(300.0, 5.0));
    ui.click_world(p(415.0, 40.0));
    ui.tool(FrameTool::Text);
    ui.click_world(p(10.0, 30.0));
    ui.type_in_row("Texto", "ACME LTDA");
    ui.press("Aplicar elemento");
    for (at, field, key) in [
        (p(10.0, 10.0), None, None),
        (p(310.0, 10.0), Some("Número da página"), None),
        (p(200.0, 10.0), Some("Propriedade da página"), Some("Data")),
    ] {
        ui.tool(FrameTool::Special);
        ui.click_world(at);
        if let Some(field) = field {
            ui.choose("Nome do projeto", field);
        }
        if let Some(key) = key {
            ui.type_in_row("Chave", key);
        }
        ui.press("Aplicar elemento");
    }
    ui.tool(FrameTool::Image);
    ui.click_world(p(350.0, 15.0));
    let frame = ui.frame_now(id);
    let kinds: Vec<_> = frame
        .elements()
        .iter()
        .map(FrameElement::kind_label)
        .collect();
    assert_eq!(
        kinds,
        [
            "Retângulo",
            "Linha",
            "Retângulo",
            "Texto",
            "Texto especial",
            "Texto especial",
            "Texto especial",
            "Imagem"
        ]
    );
    assert!(
        matches!(&frame.elements()[3], FrameElement::Text { text, .. } if text.default == "ACME LTDA")
    );
    assert!(matches!(
        &frame.elements()[5],
        FrameElement::Special {
            field: SpecialField::PageNumber,
            ..
        }
    ));
    assert!(
        matches!(&frame.elements()[6], FrameElement::Special { field: SpecialField::PageProperty(k), .. } if k.as_str() == "Data")
    );
    let asset = frame.elements()[7].asset().unwrap();
    assert_eq!(
        ui.project().asset(asset).unwrap().bytes(),
        Some(logo_bytes.as_slice())
    );
    match &frame.elements()[7] {
        FrameElement::Image { rect, .. } => {
            assert_eq!(rect.origin(), p(350.0, 15.0));
            assert!((rect.size().width() / rect.size().height() - 2.0).abs() < 1e-9);
        }
        _ => unreachable!(),
    }

    // Assign it to the page and verify painting with project values.
    ui.app
        .graphic_edit(GraphicEdit::Frame(FrameEdit::SetDefaultFrame(Some(id))));
    ui.press("Fechar editor");
    assert!(!ui.app.frame_editor.is_open());
    ui.app.session.open_page(page).unwrap();
    ui.app.session.active_tab_mut().unwrap().camera = Camera::new([210.0, 20.0], 1.5).unwrap();
    ui.app.session.active_tab_mut().unwrap().needs_fit = false;
    let texts = sheet_texts(&mut ui);
    for expected in ["ACME LTDA", "Painel CCM", "1", "03/11/2026"] {
        assert!(texts.iter().any(|t| t == expected), "{expected}: {texts:?}");
    }
    assert!(frame_view::texture(&ui.ctx, ui.project(), asset).is_some());

    // Undo removes the assignment, then the image and its content together.
    let with_image = ui.project().clone();
    ui.key(Key::Z, Modifiers::CTRL);
    assert_eq!(
        ui.project().default_frame(),
        Some(
            with_image
                .frames()
                .find(|f| f.name().as_str() == "CAD-CRAFT A3 paisagem")
                .unwrap()
                .id()
        )
    );
    ui.key(Key::Z, Modifiers::CTRL);
    assert!(ui.project().asset(asset).is_none());
    assert_eq!(ui.frame_now(id).elements().len(), 7);
    ui.key(Key::Y, Modifiers::CTRL);
    ui.key(Key::Y, Modifiers::CTRL);
    assert_eq!(ui.project(), &with_image);
}

/// Editing an assigned frame updates pages and references.
#[test]
fn editing_a_frame_in_use_updates_pages_and_references() {
    let (project, page) = base_project();
    let mut ui = Ui::new(project);
    ui.app.session.open_page(page).unwrap();
    ui.settle(2);
    // Coil in column 4, x 128..169, of the factory A3 frame.
    ui.app
        .session
        .begin_device_insertion(
            SymbolDefinitionId::new("cad-craft.coil").unwrap(),
            DevicePlan::New {
                prefix: String::new(),
            },
            Transform2D::new(p(0.0, 0.0), 0.0, false, false).unwrap(),
        )
        .unwrap();
    ui.app
        .session
        .graphics
        .update_insertion(p(150.0, 160.0), 5.0, true)
        .unwrap();
    let rep = ui.app.session.confirm_symbol_insertion().unwrap();
    ui.app.session.graphics.cancel();
    let reference = |ui: &Ui| {
        let p = ui.project();
        let r = p.representation(rep).unwrap();
        p.reference_label(r.page_id(), r.transform().translation())
    };
    assert_eq!(reference(&ui), "/1.4");

    ui.app.execute(CommandId::FrameEditor, &ui.ctx.clone());
    ui.settle(3);
    let id = ui.app.frame_editor.frame.unwrap();
    let before = ui.frame_now(id);
    // Select the title-block line at y = 35 with a real click and delete it.
    ui.click_world(p(200.0, 35.0));
    assert_eq!(ui.app.frame_editor.selection.len(), 1);
    let selected = *ui.app.frame_editor.selection.iter().next().unwrap();
    assert!(matches!(
        before.elements()[selected],
        FrameElement::Line { .. }
    ));
    ui.press("Excluir seleção");
    let after_delete = ui.frame_now(id);
    assert_eq!(after_delete.elements().len(), before.elements().len() - 1);
    assert_eq!(after_delete.version(), before.version() + 1);

    // Drag the project text 10 mm to the right.
    let label = after_delete
        .elements()
        .iter()
        .position(|e| matches!(e, FrameElement::Text { text, .. } if text.default == "Projeto"))
        .unwrap();
    let anchor = after_delete.elements()[label].anchor();
    ui.click_world(p(anchor.x() + 2.0, anchor.y() - 1.0));
    assert_eq!(ui.app.frame_editor.selection.iter().next(), Some(&label));
    ui.drag_world(
        p(anchor.x() + 2.0, anchor.y() - 1.0),
        p(anchor.x() + 12.0, anchor.y() - 1.0),
    );
    let moved = ui.frame_now(id).elements()[label].anchor();
    assert!((moved.x() - anchor.x() - 10.0).abs() < 1e-9, "{moved:?}");

    // Changing zones to eight 51.25 mm columns recalculates the reference.
    let frame = ui.frame_now(id);
    let mut draft = frame.draft();
    draft.zones.columns = ZoneAxis::uniform(8, 51.25, ZoneLabelStyle::Numbers { start: 1 });
    assert!(ui.app.commit_frame_draft(draft, vec![]));
    assert_eq!(reference(&ui), "/1.3");

    // Undoing all three edits restores the original frame and reference.
    for _ in 0..3 {
        ui.key(Key::Z, Modifiers::CTRL);
    }
    assert_eq!(ui.frame_now(id), before);
    assert_eq!(reference(&ui), "/1.4");
}

/// Export .cadframe and import into another project.
#[test]
fn cadframe_export_and_import_between_projects() {
    let dir = tempfile::tempdir().unwrap();
    let logo = dir.path().join("logo.png");
    let logo_bytes = write_png(&logo, 40, 40);
    let file = dir.path().join("minha-moldura.cadframe");
    let (project, _) = base_project();
    let mut ui = Ui::new(project);
    ui.app.files.picker = Box::new(ScriptedPicker {
        images: [logo].into(),
        frames_save: [dir.path().join("minha-moldura")].into(),
        ..Default::default()
    });
    ui.app.execute(CommandId::FrameNew, &ui.ctx.clone());
    ui.settle(3);
    let id = ui.app.frame_editor.frame.unwrap();
    ui.tool(FrameTool::Image);
    ui.click_world(p(20.0, 20.0));
    ui.app.execute(CommandId::FrameExport, &ui.ctx.clone());
    assert!(file.is_file());
    let exported = ui.frame_now(id);

    let (other, _) = base_project();
    let mut second = Ui::new(other);
    second.app.files.picker = Box::new(ScriptedPicker {
        frames_open: [file.clone()].into(),
        ..Default::default()
    });
    second
        .app
        .execute(CommandId::FrameImport, &second.ctx.clone());
    second.settle(2);
    let imported = second
        .app
        .frame_editor
        .frame
        .expect("editor na moldura importada");
    let frame = second.frame_now(imported);
    assert_eq!(frame.elements(), exported.elements());
    assert_eq!(frame.name(), exported.name());
    let asset = frame.elements().last().unwrap().asset().unwrap();
    assert_eq!(
        second.project().asset(asset).unwrap().bytes(),
        Some(logo_bytes.as_slice())
    );
    // Import again into the same project: new frame, same image.
    second.app.files.picker = Box::new(ScriptedPicker {
        frames_open: [file].into(),
        ..Default::default()
    });
    second
        .app
        .execute(CommandId::FrameImport, &second.ctx.clone());
    assert_eq!(second.project().assets().count(), 1);
    assert_eq!(second.project().frames().count(), 4);
}

/// Reject invalid images and files without changing anything.
#[test]
fn invalid_images_and_frame_files_are_refused_without_changes() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("corrompido.png");
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(16u32.to_be_bytes());
    bytes.extend(16u32.to_be_bytes());
    bytes.extend([8, 6, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7]);
    std::fs::write(&bad, bytes).unwrap();
    let huge = dir.path().join("enorme.png");
    write_png(&huge, 4097, 1);
    let fake = dir.path().join("falsa.cadframe");
    std::fs::write(&fake, b"PK\x03\x04 nao e zip").unwrap();
    let (project, _) = base_project();
    let mut ui = Ui::new(project);
    ui.app.files.picker = Box::new(ScriptedPicker {
        images: [bad, huge].into(),
        frames_open: [fake].into(),
        ..Default::default()
    });
    ui.app.execute(CommandId::FrameNew, &ui.ctx.clone());
    ui.settle(3);
    let before = ui.project().clone();
    for _ in 0..2 {
        ui.tool_try(FrameTool::Image);
        assert_eq!(ui.app.frame_editor.tool, FrameTool::Select);
    }
    ui.app.execute(CommandId::FrameImport, &ui.ctx.clone());
    assert_eq!(ui.project(), &before);
    assert_eq!(ui.project().assets().count(), 0);
}
