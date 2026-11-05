// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 22: diálogo Exportar DXF / Exportar DWG com eventos egui reais.
//! O conversor DWG é um script `.cmd` de teste (Windows); nada de terceiros.
use cad_craft::domain::*;
use cad_gui::{app::CadApp, commands::CommandId, project_file::ScriptedPicker};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};
use std::path::{Path, PathBuf};

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
fn labels(out: &egui::FullOutput) -> Vec<String> {
    out.platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, n)| n.label().or_else(|| n.value()).map(str::to_owned))
        .collect()
}
/// Clica no controle com o rótulo (o título da janela, mesmo texto, é ignorado).
fn click(app: &mut CadApp, ctx: &Context, label: &str) {
    frame(app, ctx, vec![]);
    let out = frame(app, ctx, vec![]);
    let node = out
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, n)| {
            n.label() == Some(label)
                && !matches!(
                    n.role(),
                    egui::accesskit::Role::Label
                        | egui::accesskit::Role::TextRun
                        | egui::accesskit::Role::Window
                )
        })
        .filter_map(|(_, n)| n.bounds())
        // A faixa de opções (topo) repete os nomes dos comandos; o diálogo fica abaixo.
        .max_by(|a, b| a.y0.total_cmp(&b.y0))
        .unwrap_or_else(|| panic!("widget {label} ausente: {:?}", labels(&out)));
    let pos = Pos2::new(
        ((node.x0 + node.x1) / 2.) as f32,
        ((node.y0 + node.y1) / 2.) as f32,
    );
    let event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(app, ctx, vec![Event::PointerMoved(pos), event(true)]);
    frame(app, ctx, vec![event(false)]);
    frame(app, ctx, vec![]);
}
/// Espera a thread terminar; devolve o relatório ou o erro.
fn finish(app: &mut CadApp, ctx: &Context) -> Result<String, String> {
    for _ in 0..1500 {
        frame(app, ctx, vec![]);
        if let Some(result) = &app.pdf.result {
            return result.clone();
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("exportação não terminou");
}

fn project() -> (Project, [PageId; 2]) {
    let mut project = Project::new("Exportação CAD").unwrap();
    let doc = project
        .insert_document(Document::new(project.id(), "Esquema").unwrap())
        .unwrap();
    let pages = ["1", "2"].map(|number| {
        project
            .insert_page(
                Page::new(
                    project.id(),
                    doc,
                    format!("Folha {number}"),
                    number,
                    PageKind::MultiLine,
                    PaperFormat::A4.size(true),
                )
                .unwrap(),
            )
            .unwrap()
    });
    (project, pages)
}

fn app_with(folders: &[&Path]) -> (CadApp, Context, [PageId; 2]) {
    let (project, pages) = project();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.open_page(pages[1]).unwrap();
    app.files.picker = Box::new(ScriptedPicker {
        folders: folders.iter().map(|p| p.to_path_buf()).collect(),
        ..Default::default()
    });
    frame(&mut app, &ctx, vec![]);
    (app, ctx, pages)
}

fn files_with(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(extension))
        })
        .collect();
    out.sort();
    out
}

/// Os testes de DWG observam a pasta temporária: um de cada vez.
static TEMP_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Pastas temporárias do conversor ainda existentes.
fn leftovers() -> Vec<PathBuf> {
    std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("cad-craft-dwg-"))
        })
        .collect()
}

/// Critério 6: DXF da página ativa e do projeto, com relatório.
#[test]
fn dxf_dialog_exports_active_page_then_project() {
    let dir = tempfile::tempdir().unwrap();
    let (one, all) = (dir.path().join("pagina"), dir.path().join("projeto"));
    std::fs::create_dir_all(&one).unwrap();
    std::fs::create_dir_all(&all).unwrap();
    let (mut app, ctx, _) = app_with(&[&one, &all]);
    app.execute(CommandId::ExportDxf, &ctx);
    click(&mut app, &ctx, "Página ativa");
    click(&mut app, &ctx, "Exportar DXF");
    let report = finish(&mut app, &ctx).unwrap();
    assert!(report.contains("1 arquivo(s)"), "{report}");
    let files = files_with(&one, "dxf");
    assert_eq!(files.len(), 1);
    assert!(
        files[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with("_Esquema_2.dxf")
    );
    let bytes = std::fs::read(&files[0]).unwrap();
    assert!(bytes.windows(6).any(|w| w == b"AC1009"));
    click(&mut app, &ctx, "Projeto");
    click(&mut app, &ctx, "Exportar DXF");
    let report = finish(&mut app, &ctx).unwrap();
    assert!(report.contains("2 arquivo(s)"), "{report}");
    assert_eq!(files_with(&all, "dxf").len(), 2);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(labels(&out).iter().any(|l| l.starts_with("DXF gerado em")));
}

/// Critério 5/6: sem conversor configurado, Exportar DWG fica desabilitado
/// e explica o motivo.
#[test]
fn dwg_is_disabled_without_converter() {
    let dir = tempfile::tempdir().unwrap();
    let (mut app, ctx, _) = app_with(&[dir.path()]);
    app.execute(CommandId::ExportDwg, &ctx);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(
        labels(&out)
            .iter()
            .any(|l| l.starts_with("Configure o conversor DWG")),
        "{:?}",
        labels(&out)
    );
    click(&mut app, &ctx, "Exportar DWG");
    assert!(!app.pdf.is_running());
    assert!(app.pdf.result.is_none());
    assert!(files_with(dir.path(), "dwg").is_empty());
}

#[cfg(windows)]
fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, format!("@echo off\r\n{body}\r\n")).unwrap();
    path
}

/// Critério 5/6: com o conversor falso, a chamada leva entrada, saída,
/// versão escolhida e filtro; os DWG chegam ao destino; nada fica no temp.
#[cfg(windows)]
#[test]
fn dwg_export_runs_the_configured_converter_and_cleans_up() {
    let _guard = TEMP_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tools = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let converter = script(
        tools.path(),
        "conversor.cmd",
        "echo %3 %4 %5 %6 %7> \"%~dp0argumentos.txt\"\r\n\
         for %%f in (\"%~1\\*.dxf\") do copy /y \"%%f\" \"%~2\\%%~nf.dwg\" >nul",
    );
    let before = leftovers();
    let (mut app, ctx, _) = app_with(&[dest.path()]);
    app.preferences.dwg_converter = converter.display().to_string();
    app.execute(CommandId::ExportDwg, &ctx);
    click(&mut app, &ctx, "2013");
    click(&mut app, &ctx, "Exportar DWG");
    let report = finish(&mut app, &ctx).unwrap();
    assert!(report.contains("2 arquivo(s) DWG"), "{report}");
    assert_eq!(files_with(dest.path(), "dwg").len(), 2);
    let args = std::fs::read_to_string(tools.path().join("argumentos.txt")).unwrap();
    assert_eq!(args.trim(), "ACAD2013 DWG 0 1 *.DXF");
    assert_eq!(leftovers(), before, "pastas temporárias removidas");
}

/// Critério 5: tempo limite encerra o conversor; falha do conversor é
/// relatada; em nenhum caso sobram temporários.
#[cfg(windows)]
#[test]
fn dwg_converter_timeout_and_failure_are_reported() {
    let _guard = TEMP_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tools = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let slow = script(tools.path(), "lento.cmd", "ping -n 6 127.0.0.1 >nul");
    let broken = script(tools.path(), "falha.cmd", "exit /b 3");
    let before = leftovers();
    let (mut app, ctx, _) = app_with(&[dest.path(), dest.path()]);
    app.preferences.dwg_converter = slow.display().to_string();
    app.pdf.dwg_timeout = std::time::Duration::from_millis(500);
    app.execute(CommandId::ExportDwg, &ctx);
    click(&mut app, &ctx, "Exportar DWG");
    let error = finish(&mut app, &ctx).unwrap_err();
    assert!(error.contains("tempo limite"), "{error}");
    app.preferences.dwg_converter = broken.display().to_string();
    click(&mut app, &ctx, "Exportar DWG");
    let error = finish(&mut app, &ctx).unwrap_err();
    assert!(error.contains("terminou com erro"), "{error}");
    assert!(files_with(dest.path(), "dwg").is_empty());
    assert_eq!(leftovers(), before);
}
