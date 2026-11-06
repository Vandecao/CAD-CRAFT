// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Synthetic EDZ package import with real egui events.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_gui::{app::CadApp, commands::CommandId, project_file::ScriptedPicker};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};
use std::{io::Write, path::Path};
fn frame(app: &mut CadApp, ctx: &Context, events: Vec<Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 900.))),
            events,
            ..Default::default()
        },
        |ctx| app.show(ctx),
    )
}
fn labels(out: &egui::FullOutput) -> Vec<(String, egui::accesskit::Role, Option<Pos2>)> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, n)| {
            (
                n.label()
                    .or_else(|| n.value())
                    .unwrap_or_default()
                    .to_owned(),
                n.role(),
                n.bounds()
                    .map(|b| Pos2::new(((b.x0 + b.x1) / 2.) as f32, ((b.y0 + b.y1) / 2.) as f32)),
            )
        })
        .collect()
}
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    use egui::accesskit::Role;
    frame(app, ctx, vec![]);
    let out = frame(app, ctx, vec![]);
    let pos = labels(&out)
        .into_iter()
        .filter(|(l, role, _)| {
            l == label && !matches!(role, Role::Label | Role::TextRun | Role::Window)
        })
        .filter_map(|(_, _, p)| p)
        .max_by(|a, b| a.y.total_cmp(&b.y))
        .unwrap_or_else(|| panic!("controle {label} ausente"));
    let event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(app, ctx, vec![Event::PointerMoved(pos)]);
    frame(app, ctx, vec![event(true)]);
    frame(app, ctx, vec![event(false)]);
    frame(app, ctx, vec![]);
}
fn shows(app: &mut CadApp, ctx: &Context, text: &str) -> bool {
    let out = frame(app, ctx, vec![]);
    labels(&out).iter().any(|(l, _, _)| l.contains(text))
}
fn wait_analysis(app: &mut CadApp, ctx: &Context) {
    for _ in 0..1000 {
        frame(app, ctx, vec![]);
        if !app.edz.busy() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("análise não terminou");
}

#[test]
fn invalid_batch_preview_never_writes_libraries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("invalido.edz");
    std::fs::write(&path, b"invalid").unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.files.picker = Box::new(ScriptedPicker {
        edz_open: vec![vec![path]].into(),
        ..Default::default()
    });
    app.execute(CommandId::ImportEdz, &ctx);
    click(&mut app, &ctx, "Escolher arquivos .edz…");
    click(&mut app, &ctx, "Analisar pacotes");
    wait_analysis(&mut app, &ctx);
    assert!(shows(&mut app, &ctx, "Erro:"));
    assert_eq!(app.edz.packages.len(), 1);
    assert!(!dir.path().join("parts").exists());
}

#[test]
fn multiple_file_picker_batch_reports_success_and_error_per_package() {
    let dir = tempfile::tempdir().unwrap();
    let valid = dir.path().join("valid.edz");
    package(&valid);
    let invalid = dir.path().join("invalid.edz");
    std::fs::write(&invalid, b"invalid").unwrap();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.preferences.part_library = dir.path().join("parts").display().to_string();
    app.preferences.macro_library = dir.path().join("macros").display().to_string();
    app.files.picker = Box::new(ScriptedPicker {
        edz_open: vec![vec![valid, invalid]].into(),
        ..Default::default()
    });
    app.execute(CommandId::ImportEdz, &ctx);
    click(&mut app, &ctx, "Escolher arquivos .edz…");
    assert_eq!(app.edz.paths.len(), 2);
    click(&mut app, &ctx, "Analisar pacotes");
    wait_analysis(&mut app, &ctx);
    assert_eq!(app.edz.packages.len(), 2);
    click(&mut app, &ctx, "Confirmar importação nas bibliotecas");
    wait_analysis(&mut app, &ctx);
    assert_eq!(app.part_editor.catalog().count(), 1);
    assert!(app.edz.packages.iter().any(|p| p.outcome == "Importado"));
    assert!(
        app.edz
            .packages
            .iter()
            .any(|p| p.outcome.starts_with("Erro:"))
    );
}

fn package(path: &Path) {
    write_package(path, false);
}
fn write_package(path: &Path, with_macro: bool) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    let mut files = vec![
        (
            "manifest.xml",
            r#"<manifest version="2.0"><package type="part" key="TEST.1"><item type="partxml" name="part" locator="test.part.xml"/></package></manifest>"#,
        ),
        (
            "items/partxml/test.part.xml",
            r#"<partsmanagement><part P_ARTICLE_PARTNR="TEST.1" P_ARTICLE_MANUFACTURER="T" P_ARTICLE_DESCR1="en_US@Test;" P_ARTICLE_PRODUCTGROUP="Relay"/></partsmanagement>"#,
        ),
    ];
    if with_macro {
        files[0].1 = r#"<manifest version="2.0"><package type="part" key="TEST.1"><item type="partxml" name="part" locator="test.part.xml"/><item type="macro" name="Synthetic" locator="synthetic.ema"/></package></manifest>"#;
        files.push(("items/macro/synthetic.ema",r#"<EplanPxfRoot Type="WindowMacro" Name="Synthetic"><MacroVariant MacroFuncType="1" ReferencePoint="10/20/0"><ESymLib Name="Test" ID="0"/><O4 A1="4/1"><O17 A1="17/2" A185="A"><S61x183 A1061="1"/><S40x1201 A762="10/20"><S39x761 A751="1" A752="0" A753="0" A754="1"/></S40x1201><P11 P19008="Missing"/><S89x5 A1651="-2/-2" A1652="2/2"/></O17></O4></MacroVariant></EplanPxfRoot>"#));
    }
    for (name, text) in files {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn imported_macro_is_found_in_navigator_inserted_and_undone_by_real_events() {
    use cad_craft::domain::*;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("own-macro.edz");
    write_package(&path, true);
    let (mut app, ctx) = setup(&path, dir.path());
    assert!(shows(&mut app, &ctx, "1 macro(s), 1 símbolo(s)"));
    click(&mut app, &ctx, "Confirmar importação nas bibliotecas");
    wait_analysis(&mut app, &ctx);
    assert_eq!(app.part_editor.catalog().count(), 1);
    let definition = app
        .macro_library
        .catalog()
        .next()
        .expect("macro na biblioteca");
    assert!(
        app.part_editor
            .catalog()
            .next()
            .unwrap()
            .properties()
            .custom()
            .iter()
            .any(|(key, value)| key.as_str() == "CADCRAFT.MacroIds"
                && matches!(value,PropertyValue::Text(id) if id==definition.id().as_str()))
    );
    let label = format!(
        "{} · {} · v{}",
        definition.name().as_str(),
        definition.id().as_str(),
        definition.version()
    );
    let mut project = Project::new("Insertion").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Scheme").unwrap())
        .unwrap();
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "One",
                "1",
                PageKind::MultiLine,
                Size2D::new(420., 297.).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    app.session.replace(project, false).unwrap();
    app.session.open_page(page).unwrap();
    app.edz.open = false;
    app.execute(CommandId::MacroNavigator, &ctx);
    click(&mut app, &ctx, &label);
    click(&mut app, &ctx, "Inserir na página");
    app.macro_library.open = false;
    frame(&mut app, &ctx, vec![]);
    let pos = app.last_viewport.unwrap().center();
    frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.session.project().unwrap().devices().count(), 1);
    for (key, modifiers) in [
        (egui::Key::Escape, Modifiers::NONE),
        (egui::Key::Z, Modifiers::CTRL),
    ] {
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                }],
            );
        }
    }
    assert_eq!(app.session.project().unwrap().devices().count(), 0);
}

fn setup(path: &Path, dir: &Path) -> (CadApp, Context) {
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.preferences.part_library = dir.join("parts").display().to_string();
    app.preferences.macro_library = dir.join("macros").display().to_string();
    app.files.picker = Box::new(ScriptedPicker {
        edz_open: vec![vec![path.to_owned()]].into(),
        ..Default::default()
    });
    app.execute(CommandId::ImportEdz, &ctx);
    click(&mut app, &ctx, "Escolher arquivos .edz…");
    click(&mut app, &ctx, "Analisar pacotes");
    wait_analysis(&mut app, &ctx);
    (app, ctx)
}

#[test]
fn preview_confirmation_and_explicit_keep_both_and_skip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("own.edz");
    package(&path);
    let (mut app, ctx) = setup(&path, dir.path());
    assert!(shows(&mut app, &ctx, "1 peça(s)"));
    assert!(
        !dir.path().join("parts").exists(),
        "análise é somente leitura"
    );
    click(&mut app, &ctx, "Confirmar importação nas bibliotecas");
    wait_analysis(&mut app, &ctx);
    assert!(shows(&mut app, &ctx, "Importado"), "{}", app.edz.status);
    assert_eq!(app.part_editor.catalog().count(), 1);
    assert_eq!(
        app.part_editor.catalog().next().unwrap().code().as_str(),
        "TEST.1"
    );
    let original_id = app
        .part_editor
        .catalog()
        .next()
        .unwrap()
        .id()
        .as_str()
        .to_owned();
    click(&mut app, &ctx, "Analisar pacotes");
    wait_analysis(&mut app, &ctx);
    assert!(shows(&mut app, &ctx, "Conflito:"));
    click(&mut app, &ctx, "Manter as duas");
    click(&mut app, &ctx, "Confirmar importação nas bibliotecas");
    wait_analysis(&mut app, &ctx);
    assert_eq!(app.part_editor.catalog().count(), 2, "{}", app.edz.status);
    assert!(
        app.part_editor
            .catalog()
            .all(|p| p.code().as_str() == "TEST.1"),
        "as duas peças preservam o código comercial do fabricante"
    );
    assert!(
        app.part_editor
            .catalog()
            .any(|p| p.id().as_str() == format!("{original_id}-2")),
        "a cópia é diferenciada pelo ID da biblioteca"
    );
    click(&mut app, &ctx, "Analisar pacotes");
    wait_analysis(&mut app, &ctx);
    click(&mut app, &ctx, "Pular");
    click(&mut app, &ctx, "Confirmar importação nas bibliotecas");
    wait_analysis(&mut app, &ctx);
    assert_eq!(app.part_editor.catalog().count(), 2);
    assert!(shows(&mut app, &ctx, "Ignorado"));
}

#[test]
fn confirmation_requires_configured_library_and_folder_picker_analyzes_batch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("own.edz");
    package(&path);
    std::fs::write(dir.path().join("other.txt"), "not a package").unwrap();
    let (mut app, ctx) = setup(&path, dir.path());
    app.preferences.part_library.clear();
    click(&mut app, &ctx, "Confirmar importação nas bibliotecas");
    wait_analysis(&mut app, &ctx);
    assert!(shows(&mut app, &ctx, "Configure as pastas"));
    assert!(!dir.path().join("parts").exists());
    app.files.picker = Box::new(ScriptedPicker {
        folders: vec![dir.path().to_owned()].into(),
        ..Default::default()
    });
    click(&mut app, &ctx, "Escolher pasta…");
    assert_eq!(app.edz.paths, vec![path]);
    click(&mut app, &ctx, "Analisar pacotes");
    wait_analysis(&mut app, &ctx);
    assert_eq!(app.edz.packages.len(), 1);
}
