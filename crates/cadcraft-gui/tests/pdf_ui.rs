// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 21: diálogo Exportar PDF / Imprimir com eventos egui reais. O
//! sistema (abrir/imprimir) é substituído por um gravador.
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp, commands::CommandId, pdf_export::SystemShell, project_file::ScriptedPicker,
};
use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, Rect, vec2};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
/// Clica no widget com o rótulo, dentro da janela do PDF (o mais à direita
/// entre homônimos não importa: os rótulos do diálogo são únicos).
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
        .filter(|(_, n)| n.label() == Some(label))
        .filter_map(|(_, n)| n.bounds())
        .next_back()
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
/// Espera a thread de geração terminar (quadros reais até o resultado).
fn finish(app: &mut CadApp, ctx: &Context) -> String {
    for _ in 0..1000 {
        frame(app, ctx, vec![]);
        if let Some(result) = &app.pdf.result {
            return result.clone().unwrap_or_else(|e| panic!("falhou: {e}"));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("geração do PDF não terminou");
}

#[derive(Clone, Default)]
struct Recorder(Arc<Mutex<Vec<(&'static str, PathBuf)>>>);
impl SystemShell for Recorder {
    fn open(&mut self, path: &Path) -> Result<(), String> {
        self.0.lock().unwrap().push(("abrir", path.to_owned()));
        Ok(())
    }
    fn print(&mut self, path: &Path) -> Result<(), String> {
        self.0.lock().unwrap().push(("imprimir", path.to_owned()));
        Ok(())
    }
}

fn project() -> (Project, [PageId; 2]) {
    let mut project = Project::new("Exportação").unwrap();
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

fn pdf_pages(path: &Path) -> usize {
    let bytes = std::fs::read(path).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    bytes
        .windows(b"/Type /Page\n".len())
        .filter(|w| *w == b"/Type /Page\n")
        .count()
}

/// Critério 7: exporta a página ativa e depois o projeto; mostra o relatório;
/// "Abrir ao terminar" chama o sistema (substituído).
#[test]
fn export_dialog_writes_active_page_then_project_and_reports() {
    let dir = tempfile::tempdir().unwrap();
    let (first, second) = (
        dir.path().join("pagina.pdf"),
        dir.path().join("projeto.pdf"),
    );
    let (project, pages) = project();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    app.session.open_page(pages[1]).unwrap();
    app.files.picker = Box::new(ScriptedPicker {
        pdf_save: [first.clone(), second.clone()].into(),
        ..Default::default()
    });
    let recorder = Recorder::default();
    app.pdf.shell = Box::new(recorder.clone());
    frame(&mut app, &ctx, vec![]);
    app.execute(CommandId::ExportPdf, &ctx);
    click(&mut app, &ctx, "Página ativa");
    click(&mut app, &ctx, "Exportar");
    let report = finish(&mut app, &ctx);
    assert!(report.contains("1 página(s)"), "{report}");
    assert_eq!(pdf_pages(&first), 1);
    let out = frame(&mut app, &ctx, vec![]);
    assert!(labels(&out).iter().any(|l| l.starts_with("PDF gerado em")));
    assert!(recorder.0.lock().unwrap().is_empty());

    click(&mut app, &ctx, "Projeto");
    click(&mut app, &ctx, "Abrir ao terminar");
    click(&mut app, &ctx, "Exportar");
    let report = finish(&mut app, &ctx);
    assert!(report.contains("2 página(s)"), "{report}");
    assert_eq!(pdf_pages(&second), 2);
    assert_eq!(*recorder.0.lock().unwrap(), vec![("abrir", second)]);
}

/// Critério 7: Imprimir gera o PDF temporário no papel escolhido e o
/// entrega ao sistema; nada é impresso de verdade.
#[test]
fn print_dialog_builds_temporary_pdf_and_hands_it_to_the_system() {
    let (project, _) = project();
    let ctx = Context::default();
    ctx.enable_accesskit();
    let mut app = CadApp::default();
    app.session.replace(project, false).unwrap();
    let recorder = Recorder::default();
    app.pdf.shell = Box::new(recorder.clone());
    frame(&mut app, &ctx, vec![]);
    app.execute(CommandId::Print, &ctx);
    click(&mut app, &ctx, "A3");
    click(&mut app, &ctx, "Preto e branco");
    click(&mut app, &ctx, "Imprimir agora");
    let report = finish(&mut app, &ctx);
    assert!(report.contains("Enviado à impressão"), "{report}");
    let calls = recorder.0.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    let (action, path) = &calls[0];
    assert_eq!(*action, "imprimir");
    assert!(path.extension().is_some_and(|e| e == "pdf"));
    assert_eq!(pdf_pages(path), 2);
    // A3 deitado (A4 da folha ampliado para caber).
    let bytes = std::fs::read(path).unwrap();
    assert!(bytes.windows(7).any(|w| w == b"1190.55"));
    std::fs::remove_file(path).unwrap();
}
