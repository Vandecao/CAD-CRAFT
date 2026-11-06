// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Project lifecycle through real GUI interactions: create, save, close, reopen, edit and save again; locking, recovery, v0 migration, unsafe files and undo/redo for pages and insertions.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp,
    camera::Camera,
    commands::CommandId,
    editing::GraphicEdit,
    interaction::DevicePlan,
    project_file::{self, Recovery, ScriptedPicker},
};
use eframe::egui::{
    self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, vec2,
};
use std::path::{Path, PathBuf};

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
    fn picker(&mut self, open: &[&Path], save: &[&Path]) {
        self.app.files.picker = Box::new(ScriptedPicker {
            open: open.iter().map(|p| p.to_path_buf()).collect(),
            save: save.iter().map(|p| p.to_path_buf()).collect(),
            ..Default::default()
        });
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
    fn settle(&mut self, frames: usize) -> egui::FullOutput {
        let mut output = self.frame(vec![]);
        for _ in 1..frames {
            output = self.frame(vec![]);
        }
        output
    }
    fn click(&mut self, at: Pos2) -> egui::FullOutput {
        let press = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(at), press(true)]);
        self.frame(vec![press(false)]);
        self.settle(3)
    }
    fn key(&mut self, key: Key, modifiers: Modifiers) -> egui::FullOutput {
        self.frame(vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }]);
        self.settle(3)
    }
    /// Type into the dialog field immediately below the given label. Side panels repeat labels, so search in the modal confirm button's column; ribbon buttons with the same text are not field labels.
    fn type_below(&mut self, label: &str, text: &str) -> egui::FullOutput {
        let output = self.settle(1);
        let confirm = find(&output, "Confirmar");
        let near = |p: &Pos2| (p.x - confirm.x).abs() < 260. && p.y < confirm.y;
        let anchor = nodes(&output)
            .into_iter()
            .filter(|(l, p, role)| l == label && near(p) && *role != Role::Button)
            .map(|(_, p, _)| p)
            .next()
            .unwrap_or_else(|| panic!("rótulo {label} no diálogo"));
        let field = nodes(&output)
            .into_iter()
            .filter(|(_, p, role)| *role == Role::TextInput && p.y > anchor.y && near(p))
            .min_by(|a, b| (a.1.y - anchor.y).total_cmp(&(b.1.y - anchor.y)))
            .map(|(_, p, _)| p)
            .unwrap_or_else(|| panic!("campo abaixo de {label}"));
        self.click(field);
        self.frame(vec![Event::Text(text.into())]);
        self.settle(2)
    }
    fn press(&mut self, label: &str) -> egui::FullOutput {
        let output = self.settle(1);
        self.click(find(&output, label))
    }
    fn project(&self) -> &Project {
        self.app.session.project().expect("projeto aberto")
    }
    fn fix_camera(&mut self) {
        self.app.preferences.grid_visible = false;
        self.app.preferences.grid_mm = 4.;
        self.app.preferences.snap_enabled = true;
        let tab = self.app.session.active_tab_mut().unwrap();
        tab.camera = Camera::new([150., 130.], 2.).unwrap();
        tab.needs_fit = false;
        self.settle(2);
    }
    fn click_world(&mut self, point: Point2D) {
        let viewport = self.app.last_viewport.unwrap();
        let at = self
            .app
            .session
            .active_tab_mut()
            .unwrap()
            .camera
            .world_to_screen(point, viewport)
            .unwrap();
        self.click(at);
    }
    fn place(&mut self, key: &str, plan: DevicePlan, at: Point2D) -> RepresentationId {
        let t = Transform2D::new(p(0., 0.), 0., false, false).unwrap();
        self.app
            .session
            .begin_device_insertion(
                SymbolDefinitionId::new(format!("cad-craft.{key}")).unwrap(),
                plan,
                t,
            )
            .unwrap();
        self.settle(1);
        self.click_world(at);
        self.key(Key::Escape, Modifiers::NONE);
        self.app.session.graphics.selection.primary().unwrap()
    }
    fn connect(&mut self, a: Point2D, b: Point2D) {
        self.app.execute(CommandId::DrawConnection, &self.ctx);
        self.settle(1);
        self.click_world(a);
        self.click_world(b);
        self.key(Key::Escape, Modifiers::NONE);
    }
}

fn nodes(output: &egui::FullOutput) -> Vec<(String, Pos2, Role)> {
    let Some(update) = &output.platform_output.accesskit_update else {
        return vec![];
    };
    update
        .nodes
        .iter()
        .filter_map(|(_, node)| {
            let b = node.bounds()?;
            Some((
                node.label()
                    .filter(|l| !l.is_empty())
                    .or_else(|| node.value())
                    .unwrap_or_default()
                    .to_owned(),
                Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32),
                node.role(),
            ))
        })
        .collect()
}
fn find(output: &egui::FullOutput, label: &str) -> Pos2 {
    nodes(output)
        .into_iter()
        .find(|(l, _, _)| l == label)
        .map(|(_, p, _)| p)
        .unwrap_or_else(|| {
            let labels: Vec<_> = nodes(output).into_iter().map(|n| n.0).collect();
            panic!("widget {label:?} ausente; rótulos: {labels:?}")
        })
}
fn has(output: &egui::FullOutput, label: &str) -> bool {
    nodes(output).iter().any(|(l, _, _)| l == label)
}
fn ctrl() -> Modifiers {
    Modifiers::CTRL
}
fn dir_entries(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    entries
}

/// Complete workflow with dialogs, sheet clicks and real shortcuts.
#[test]
fn real_project_is_created_saved_reopened_edited_and_saved_again() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("projeto-real");
    let file = dir.path().join("projeto-real.cadcraft");
    let mut ui = Ui::new();

    ui.press("Criar sessão em memória");
    ui.type_below("Nome do projeto", "Projeto real");
    ui.press("Confirmar");
    assert_eq!(ui.project().name().as_str(), "Projeto real");

    ui.app.execute(CommandId::NewDocument, &ui.ctx.clone());
    ui.type_below("Nome do documento", "Esquema");
    ui.press("Confirmar");
    for (name, number) in [("Força", "1"), ("Comando", "2")] {
        ui.app.execute(CommandId::NewPage, &ui.ctx.clone());
        ui.type_below("Nome", name);
        ui.type_below("Identificação", number);
        ui.press("Confirmar");
    }
    assert_eq!(ui.project().pages().count(), 2);
    let comando = ui.app.session.active.unwrap();
    assert_eq!(
        ui.project().page(comando).unwrap().name().as_str(),
        "Comando"
    );
    ui.fix_camera();

    // Four symbols, two devices: coils create K1/K2; contacts become functions.
    let new = || DevicePlan::New {
        prefix: String::new(),
    };
    ui.place("coil", new(), p(100., 160.));
    ui.place("coil", new(), p(200., 160.));
    let k1 = ui
        .project()
        .devices()
        .find(|d| d.designation().to_string() == "-K1")
        .unwrap()
        .id();
    let k2 = ui
        .project()
        .devices()
        .find(|d| d.designation().to_string() == "-K2")
        .unwrap()
        .id();
    ui.place("contact-no", DevicePlan::Existing(k1), p(100., 100.));
    ui.place("contact-no", DevicePlan::Existing(k2), p(200., 100.));
    assert_eq!(ui.project().devices().count(), 2);
    assert_eq!(ui.project().functions().count(), 4);
    assert_eq!(ui.project().device_references(k1).unwrap().len(), 2);

    ui.connect(p(100., 110.), p(100., 150.));
    ui.connect(p(200., 110.), p(200., 150.));
    ui.connect(p(100., 170.), p(200., 170.));
    assert_eq!(ui.project().connections().count(), 3);
    assert_eq!(ui.project().connection_routes().count(), 3);
    let joined = ui
        .project()
        .networks()
        .into_iter()
        .filter(|n| n.points().len() == 2)
        .count();
    assert_eq!(joined, 3);
    assert!(ui.app.session.dirty);

    // Ctrl+S without an associated file asks for a destination and appends the extension.
    ui.picker(&[], &[&target]);
    ui.key(Key::S, ctrl());
    assert!(file.is_file());
    assert!(project_file::lock_path(&file).is_file());
    assert!(!ui.app.session.dirty);
    let saved = ui.project().clone();

    ui.app.execute(CommandId::CloseProject, &ui.ctx.clone());
    ui.settle(2);
    assert!(ui.app.session.project().is_none());
    assert!(!project_file::lock_path(&file).exists());

    ui.picker(&[&file], &[]);
    ui.key(Key::O, ctrl());
    assert_eq!(
        ui.project(),
        &saved,
        "IDs, redes, rotas e referências idênticos"
    );
    assert!(!ui.app.session.file.as_ref().unwrap().read_only);
    assert_eq!(ui.app.files.recent.first(), Some(&file));

    // After reopening, move a symbol and rename a device.
    ui.app.session.open_page(comando).unwrap();
    ui.fix_camera();
    let coil = ui
        .project()
        .representations()
        .find(|r| r.transform().translation() == p(200., 160.))
        .unwrap()
        .id();
    ui.click_world(p(200., 160.));
    assert_eq!(ui.app.session.graphics.selection.primary(), Some(coil));
    ui.app.graphic_edit(GraphicEdit::Translate(8., 0.));
    ui.app.graphic_edit(GraphicEdit::Device(DeviceEdit::Rename {
        device: k2,
        designation: Designation::new("K20").unwrap(),
    }));
    assert!(ui.app.session.dirty);
    ui.key(Key::S, ctrl()); // same file, without a dialog
    assert!(!ui.app.session.dirty);
    let edited = ui.project().clone();
    assert_ne!(edited, saved);

    let reopened = project_file::load(&file).unwrap();
    assert_eq!(reopened.project, edited);
    assert_eq!(
        reopened
            .project
            .device(k2)
            .unwrap()
            .designation()
            .to_string(),
        "-K20"
    );
    assert_eq!(
        reopened
            .project
            .representation(coil)
            .unwrap()
            .transform()
            .translation(),
        p(208., 160.)
    );
    drop(ui);
    assert_eq!(dir_entries(dir.path()), vec![file]);
}

/// Complete demo round trips through the interface.
#[test]
fn demonstration_projects_round_trip_through_cadcraft() {
    let dir = tempfile::tempdir().unwrap();
    for (index, project) in [
        cad_gui::demo::project().unwrap(),
        cad_gui::connection_demo::project().unwrap(),
    ]
    .into_iter()
    .enumerate()
    {
        let path = dir.path().join(format!("demo{index}.cadcraft"));
        let mut ui = Ui::new();
        ui.app.session.replace(project.clone(), true).unwrap();
        ui.picker(&[&path], &[&path]);
        ui.app.execute(CommandId::SaveProject, &ui.ctx.clone());
        ui.app.execute(CommandId::CloseProject, &ui.ctx.clone());
        ui.app.execute(CommandId::OpenProject, &ui.ctx.clone());
        ui.settle(2);
        assert_eq!(ui.project(), &project);
    }
}

/// A second session opens read-only and saves a copy.
#[test]
fn second_session_gets_lock_warning_and_read_only_copy() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("compartilhado.cadcraft");
    let copy = dir.path().join("copia.cadcraft");
    project_file::save(&cad_gui::demo::project().unwrap(), &file).unwrap();
    let mut first = Ui::new();
    first.picker(&[&file], &[]);
    first.key(Key::O, ctrl());
    assert!(first.app.session.file.as_ref().unwrap().locked());

    let mut second = Ui::new();
    second.picker(&[&file], &[&copy]);
    second.key(Key::O, ctrl());
    assert!(second.app.session.project().is_none());
    let output = second.settle(2);
    assert!(has(&output, "Confirmar"));
    second.press("Confirmar");
    let state = second.app.session.file.as_ref().unwrap();
    assert!(state.read_only && !state.locked());
    // Save in a read-only session becomes Save As.
    second.key(Key::S, ctrl());
    assert!(copy.is_file());
    assert!(!second.app.session.file.as_ref().unwrap().read_only);
    drop(first);
    assert!(!project_file::lock_path(&file).exists());
    drop(second);
    assert!(!project_file::lock_path(&copy).exists());
}

/// Abrupt termination offers recovery at the next startup.
#[test]
fn recovery_copy_is_offered_after_abrupt_termination() {
    let dir = tempfile::tempdir().unwrap();
    let rec = dir.path().join("recuperacao");
    let mut ui = Ui::new();
    ui.app.files.recovery = Some(Recovery::new(&rec));
    ui.app
        .session
        .replace(Project::new("Sem salvar").unwrap(), false)
        .unwrap();
    ui.app.session.new_document("Esquema").unwrap();
    ui.settle(1); // immediate autosave of the first modified state
    let expected = ui.project().clone();
    assert_eq!(Recovery::new(&rec).entries().len(), 1);
    // Abrupt termination executes no confirmation or discard.
    drop(ui);

    let mut next = Ui::new();
    next.app.enable_recovery(Recovery::new(&rec));
    let output = next.settle(2);
    assert!(has(&output, "Confirmar"));
    next.press("Confirmar");
    assert_eq!(next.project(), &expected);
    assert!(next.app.session.dirty);
    let target = dir.path().join("recuperado.cadcraft");
    next.picker(&[], &[&target]);
    next.key(Key::S, ctrl());
    assert!(target.is_file());
    assert!(Recovery::new(&rec).entries().is_empty());
}

/// The GUI migrates v0; an unsafe file never touches the disk.
#[test]
fn legacy_snapshot_migrates_and_unsafe_file_is_refused_without_side_effects() {
    let dir = tempfile::tempdir().unwrap();
    let json = dir.path().join("antigo.json");
    let project = cad_gui::connection_demo::project().unwrap();
    std::fs::write(&json, serde_json::to_string(&project).unwrap()).unwrap();
    let mut ui = Ui::new();
    ui.picker(&[&json], &[&dir.path().join("novo")]);
    ui.key(Key::O, ctrl());
    assert_eq!(ui.project(), &project);
    assert!(ui.app.session.dirty && ui.app.session.file.is_none());
    ui.key(Key::S, ctrl());
    let saved = dir.path().join("novo.cadcraft");
    assert_eq!(project_file::load(&saved).unwrap().project, project);

    let evil = dir.path().join("malicioso.cadcraft");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&evil).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    for (name, data) in [
        ("mimetype", cad_craft::persistence::MIMETYPE),
        ("../fora.txt", "x"),
    ] {
        zip.start_file(name, options).unwrap();
        std::io::Write::write_all(&mut zip, data.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
    let before = dir_entries(dir.path());
    let current = ui.project().clone();
    ui.picker(&[&evil], &[]);
    ui.key(Key::O, ctrl());
    assert_eq!(dir_entries(dir.path()), before);
    assert!(!dir.path().parent().unwrap().join("fora.txt").exists());
    assert_eq!(ui.project(), &current);
}

/// Undo/redo for insertion with a device and for pages.
#[test]
fn undo_redo_restore_placements_and_page_operations_exactly() {
    let mut ui = Ui::new();
    ui.app
        .session
        .replace(Project::new("Undo").unwrap(), false)
        .unwrap();
    let empty = ui.project().clone();
    let doc = ui.app.session.new_document("Esquema").unwrap();
    let size = Size2D::new(420., 297.).unwrap();
    let a = ui
        .app
        .session
        .new_page(doc, "A", "1", PageKind::MultiLine, size)
        .unwrap();
    let b = ui
        .app
        .session
        .new_page(doc, "B", "2", PageKind::MultiLine, size)
        .unwrap();
    ui.app.session.open_page(a).unwrap();
    ui.fix_camera();
    let with_pages = ui.project().clone();
    let rep = ui.place(
        "motor",
        DevicePlan::New {
            prefix: String::new(),
        },
        p(120., 120.),
    );
    let placed = ui.project().clone();
    assert_eq!(
        placed.devices().next().unwrap().designation().to_string(),
        "-M1"
    );
    ui.app.session.move_page(b, -1).unwrap();
    ui.app.session.delete_page(b).unwrap();
    let end = ui.project().clone();

    ui.key(Key::Z, ctrl()); // deletion
    ui.key(Key::Z, ctrl()); // reordering
    assert_eq!(ui.project(), &placed);
    ui.key(Key::Z, ctrl()); // insertion with a device
    assert_eq!(ui.project(), &with_pages);
    assert!(ui.project().representation(rep).is_none());
    for _ in 0..3 {
        ui.key(Key::Z, ctrl());
    }
    assert_eq!(ui.project(), &empty);
    assert!(
        ui.app.session.tabs.is_empty(),
        "abas de páginas desfeitas fecham"
    );
    for _ in 0..6 {
        ui.key(Key::Y, ctrl());
    }
    assert_eq!(ui.project(), &end);
}

/// Reopening the active file reuses the session's lock without conflict.
#[test]
fn reopening_the_same_file_keeps_the_session_lock() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("mesmo.cadcraft");
    project_file::save(&cad_gui::demo::project().unwrap(), &file).unwrap();
    let mut ui = Ui::new();
    ui.picker(&[&file, &file], &[]);
    ui.key(Key::O, ctrl());
    ui.key(Key::O, ctrl());
    let state = ui.app.session.file.as_ref().unwrap();
    assert!(state.locked() && !state.read_only);
    assert!(project_file::lock_path(&file).is_file());
    drop(ui);
    assert!(!project_file::lock_path(&file).exists());
}

/// After confirming exit and discarding changes, final frames before closing must not recreate the recovery copy.
#[test]
fn confirmed_exit_discards_the_recovery_copy_for_good() {
    let dir = tempfile::tempdir().unwrap();
    let rec = dir.path().join("recuperacao");
    let mut ui = Ui::new();
    ui.app.files.recovery = Some(Recovery::new(&rec));
    ui.app
        .session
        .replace(Project::new("Descartar").unwrap(), false)
        .unwrap();
    ui.app.session.new_document("Esquema").unwrap();
    ui.settle(1);
    assert_eq!(Recovery::new(&rec).entries().len(), 1);
    // Autosave on every frame would recreate the copy if the regression returned.
    ui.app.files.autosave_interval = std::time::Duration::ZERO;
    let ctx = ui.ctx.clone();
    ui.app.execute(cad_gui::commands::CommandId::Exit, &ctx);
    let output = ui.settle(2);
    assert!(has(&output, "Confirmar"));
    ui.press("Confirmar");
    assert!(!ui.app.dialog_open(), "confirmação respondida");
    ui.settle(3);
    assert!(Recovery::new(&rec).entries().is_empty());
}

/// Pin rendered labels independently of the production phrase constants.
#[test]
fn repeated_dialog_labels_remain_brazilian_portuguese() {
    let mut ui = Ui::new();
    ui.app
        .session
        .replace(cad_gui::demo::project().unwrap(), false)
        .unwrap();
    let before = ui.app.session.snapshot().unwrap();
    for (command, title) in [
        (CommandId::NewDocument, "Novo documento"),
        (CommandId::NewPage, "Nova página"),
    ] {
        ui.app.execute(command, &ui.ctx.clone());
        let output = ui.settle(3);
        let labels = nodes(&output);
        assert!(
            labels.iter().any(|(label, _, _)| label == title),
            "missing pinned dialog title: {title}"
        );
        assert!(
            labels
                .iter()
                .any(|(label, _, role)| label == "Cancelar" && *role == Role::Button)
        );
        assert!(
            labels
                .iter()
                .any(|(label, _, role)| label == "Confirmar" && *role == Role::Button)
        );
        ui.press("Cancelar");
        assert!(!ui.app.dialog_open());
        assert_eq!(ui.app.session.snapshot().unwrap(), before);
    }
}
