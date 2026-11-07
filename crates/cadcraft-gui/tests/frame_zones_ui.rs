// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frame editor zones with real egui events: changing the number of columns
//! or rows fills the usable area again, the sum is shown, divisions follow
//! the zones, and overflowing zones give a message with the numbers.
// Fixture helpers use validated synthetic inputs and fail immediately on setup errors.
#![allow(clippy::unwrap_used)]
use cad_craft::domain::*;
use cad_gui::{app::CadApp, commands::CommandId};
use eframe::egui::{
    self, Context, Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, vec2,
};

struct Ui {
    app: CadApp,
    ctx: Context,
}
impl Ui {
    /// A4 page with the factory A4 frame open in the frame editor.
    fn a4() -> Self {
        let mut project =
            cad_gui::state::with_factory_frames(Project::new("Zonas").unwrap()).unwrap();
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
                    PaperFormat::A4.size(false),
                )
                .unwrap(),
            )
            .unwrap();
        let a4 = project
            .frames()
            .find(|f| f.name().as_str() == "CAD-CRAFT A4 retrato")
            .unwrap()
            .id();
        project
            .edit_frame(FrameEdit::SetPageFrame {
                page,
                frame: Some(a4),
            })
            .unwrap();
        let ctx = Context::default();
        ctx.enable_accesskit();
        let mut app = CadApp::default();
        app.session.replace(project, false).unwrap();
        app.session.open_page(page).unwrap();
        let mut ui = Self { app, ctx };
        ui.settle(2);
        ui.app.execute(CommandId::FrameEditor, &ui.ctx.clone());
        ui.settle(3);
        ui
    }
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let app = &mut self.app;
        self.ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1600., 1000.))),
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
        let at = nodes(&out)
            .into_iter()
            .filter(|n| n.0 == label)
            .map(|n| n.1)
            .next()
            .unwrap_or_else(|| panic!("controle {label:?} ausente"));
        self.press_at(at)
    }
    fn key(&mut self, key: Key) {
        self.frame(vec![Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }]);
        self.settle(2);
    }
    /// Types a new value in the n-th "Quantidade" field (0 columns, 1 rows).
    fn set_quantity(&mut self, axis: usize, value: &str) {
        // The rows section sits low in the properties panel.
        if axis == 1 {
            self.scroll_panel();
        }
        let out = self.settle(1);
        let all = nodes(&out);
        let anchor = all
            .iter()
            .filter(|n| n.0 == "Quantidade")
            .nth(axis)
            .unwrap_or_else(|| panic!("Quantidade {axis}"))
            .1;
        let field = all
            .iter()
            .filter(|n| n.2 == Role::SpinButton && (n.1.y - anchor.y).abs() < 8.0)
            .map(|n| n.1)
            .next()
            .unwrap_or_else(|| panic!("campo Quantidade {axis}"));
        self.press_at(field);
        self.frame(vec![Event::Key {
            key: Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        }]);
        self.frame(vec![Event::Text(value.into())]);
        self.key(Key::Enter);
    }
    /// Scrolls the frame properties panel to its end.
    fn scroll_panel(&mut self) {
        let right = self.app.frame_editor.viewport.unwrap().right();
        self.frame(vec![
            Event::PointerMoved(Pos2::new(right + 120., 500.)),
            Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: vec2(0., -1000.),
                modifiers: Modifiers::NONE,
            },
        ]);
        self.settle(30);
    }
    /// Presses Aplicar at the end of the properties panel.
    fn apply(&mut self) {
        self.scroll_panel();
        self.press("Aplicar moldura");
    }
    fn editing(&self) -> FrameTemplate {
        let id = self.app.frame_editor.frame.unwrap();
        self.app
            .session
            .project()
            .unwrap()
            .frame(id)
            .unwrap()
            .clone()
    }
    fn labels(&mut self) -> Vec<String> {
        let out = self.settle(1);
        nodes(&out).into_iter().map(|n| n.0).collect()
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
fn removing_a_column_redistributes_and_redraws_the_divisions() {
    let mut ui = Ui::a4();
    let before = ui.editing();
    assert_eq!(before.zones().columns.sizes, vec![40.0; 5]);
    assert_eq!(before.zone_marks().len(), 4);
    ui.set_quantity(0, "4");
    assert!(
        ui.labels()
            .iter()
            .any(|l| l == "4 × 50,0 mm = 200,0 mm de 200,0 mm"),
        "soma das colunas visível"
    );
    ui.apply();
    let after = ui.editing();
    assert_eq!(after.zones().columns.sizes, vec![50.0; 4]);
    assert_eq!(
        after.zone_marks().len(),
        3,
        "três divisões, nenhuma sobrando"
    );
    assert!(after.zones().generated_marks);
}

#[test]
fn eight_rows_fill_the_usable_height_and_apply() {
    let mut ui = Ui::a4();
    ui.set_quantity(1, "8");
    assert!(
        ui.labels()
            .iter()
            .any(|l| l == "8 × 31,5 mm = 252,0 mm de 252,0 mm")
    );
    ui.apply();
    let frame = ui.editing();
    assert_eq!(frame.zones().rows.sizes, vec![31.5; 8]);
}

#[test]
fn uneven_division_puts_the_remainder_in_the_last_zone() {
    let mut ui = Ui::a4();
    ui.set_quantity(0, "3");
    ui.apply();
    let sizes = ui.editing().zones().columns.sizes.clone();
    assert_eq!(sizes.len(), 3);
    assert!((sizes[0] - 66.6).abs() < 1e-9 && (sizes[1] - 66.6).abs() < 1e-9);
    assert!((sizes.iter().sum::<f64>() - 200.0).abs() < 1e-9);
}

#[test]
fn overflowing_zones_are_refused_with_the_numbers() {
    let mut ui = Ui::a4();
    let before = ui.editing();
    // Individual sizes that do not fit: 8 × 40 mm in 252 mm.
    ui.scroll_panel();
    let out = ui.settle(1);
    let all = nodes(&out);
    let anchor = all
        .iter()
        .filter(|n| n.0 == "Individuais (;)")
        .nth(1)
        .unwrap()
        .1;
    let field = all
        .iter()
        .filter(|n| n.2 == Role::TextInput && (n.1.y - anchor.y).abs() < 8.0)
        .map(|n| n.1)
        .next()
        .unwrap();
    ui.press_at(field);
    ui.frame(vec![Event::Text("40; 40; 40; 40; 40; 40; 40; 40".into())]);
    ui.settle(2);
    assert!(
        ui.labels()
            .iter()
            .any(|l| l == "8 zonas = 320,0 mm de 252,0 mm")
    );
    ui.apply();
    assert_eq!(ui.editing(), before, "nada muda");
    let messages = ui.labels();
    assert!(
        messages
            .iter()
            .any(|m| m.contains("As 8 linhas somam 320,0 mm, mas a área útil tem 252,0 mm")),
        "mensagem com os números no painel Mensagens"
    );
}
