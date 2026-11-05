// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 11 — connectors and terminal specializations driven by real egui
//! interactions, plus window stability across resolutions and DPI scales.
use cad_craft::domain::*;
use cad_gui::{
    app::CadApp,
    commands::{CommandId, registry},
    editing::GraphicEdit,
    preferences::RibbonTab,
    state::Selection,
};
use eframe::egui::{
    self, Context, Event, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, vec2,
};

struct Ui {
    app: CadApp,
    ctx: Context,
    size: egui::Vec2,
}
impl Ui {
    fn new(project: Project) -> Self {
        Self::sized(project, vec2(1600., 900.), 1.0)
    }
    fn sized(project: Project, size: egui::Vec2, dpi: f32) -> Self {
        let mut app = CadApp::default();
        let ctx = Context::default();
        ctx.set_pixels_per_point(dpi);
        ctx.enable_accesskit();
        app.session.replace(project, false).unwrap();
        let mut ui = Self { app, ctx, size };
        ui.settle(2);
        ui
    }
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let app = &mut self.app;
        self.ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
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
        // The body scroll area may be scrolled horizontally (wide rows): a
        // user clicks the visible part of a widget whose center is clipped.
        let at = match self
            .ctx
            .memory(|m| m.area_rect(egui::Id::new("Engenharia física")))
        {
            Some(w)
                if (w.top()..w.bottom()).contains(&at.y)
                    && at.x < w.left() + 10.
                    && at.x > w.left() - 60. =>
            {
                Pos2::new(w.left() + 12., at.y)
            }
            _ => at,
        };
        let press = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        self.frame(vec![Event::PointerMoved(at), press(true)]);
        self.frame(vec![press(false)]);
        // Let scroll animations (e.g. scroll-to-focus) finish before the
        // next interaction reads positions.
        self.settle(15)
    }
    fn type_text(&mut self, at: Pos2, text: &str) -> egui::FullOutput {
        self.click(at);
        self.frame(vec![Event::Text(text.into())]);
        self.settle(2)
    }
    fn execute(&mut self, id: CommandId) -> egui::FullOutput {
        self.app.execute(id, &self.ctx);
        self.settle(3)
    }
    fn project(&self) -> &Project {
        self.app.session.project().unwrap()
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
fn labels(output: &egui::FullOutput) -> Vec<String> {
    nodes(output).into_iter().map(|n| n.0).collect()
}
fn find_where(output: &egui::FullOutput, what: &str, pred: impl Fn(&str) -> bool) -> Pos2 {
    nodes(output)
        .into_iter()
        .find(|(l, _, _)| pred(l))
        .map(|(_, p, _)| p)
        .unwrap_or_else(|| panic!("widget {what:?} not rendered; labels: {:?}", labels(output)))
}
fn find(output: &egui::FullOutput, label: &str) -> Pos2 {
    find_where(output, label, |l| l == label)
}
/// Text fields on the same row as `anchor`, ordered left to right.
fn fields_on_row(output: &egui::FullOutput, anchor: Pos2) -> Vec<Pos2> {
    let mut fields: Vec<_> = nodes(output)
        .into_iter()
        .filter(|(_, p, role)| *role == Role::TextInput && (p.y - anchor.y).abs() < 8.)
        .map(|(_, p, _)| p)
        .collect();
    fields.sort_by(|a, b| a.x.total_cmp(&b.x));
    fields
}
/// Combo boxes have no accessible name: the n-th one on the anchor's row.
fn combo(output: &egui::FullOutput, anchor: Pos2, index: usize) -> Pos2 {
    let mut combos: Vec<_> = nodes(output)
        .into_iter()
        .filter(|(_, p, role)| *role == Role::ComboBox && (p.y - anchor.y).abs() < 8.)
        .map(|(_, p, _)| p)
        .collect();
    combos.sort_by(|a, b| a.x.total_cmp(&b.x));
    *combos
        .get(index)
        .unwrap_or_else(|| panic!("combo {index} missing on row {anchor:?}: {combos:?}"))
}
/// First combo box laid out below the anchor row.
fn combo_below(output: &egui::FullOutput, anchor: Pos2) -> Pos2 {
    nodes(output)
        .into_iter()
        .filter(|(_, p, role)| *role == Role::ComboBox && p.y > anchor.y + 8.)
        .map(|(_, p, _)| p)
        .min_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)))
        .expect("combo below anchor")
}
/// Opens the combo at `at` and clicks the popup option `option` (the node
/// with that label that appeared only after opening), then checks that the
/// combo shows it. The scroll area may still move between frames, so a miss
/// is retried on the combo re-located by its previous text.
fn choose(ui: &mut Ui, at: Pos2, option: &str) -> egui::FullOutput {
    let shown = option.split(" (").next().unwrap_or(option);
    let mut at = at;
    for _ in 0..4 {
        let closed = ui.settle(1);
        let combo_text = nodes(&closed)
            .into_iter()
            .filter(|(_, p, r)| *r == Role::ComboBox && (*p - at).length() < 4.)
            .map(|(l, _, _)| l)
            .next();
        let before: Vec<_> = nodes(&closed)
            .into_iter()
            .filter(|(l, _, _)| l == option)
            .map(|(_, p, _)| p)
            .collect();
        let opened = ui.click(at);
        let target = nodes(&opened)
            .into_iter()
            .filter(|(l, p, _)| l == option && !before.iter().any(|b| (*b - *p).length() < 1.))
            .map(|(_, p, _)| p)
            .next()
            .unwrap_or_else(|| panic!("option {option:?} missing: {:?}", labels(&opened)));
        let out = ui.click(target);
        if nodes(&out)
            .iter()
            .any(|(l, _, r)| *r == Role::ComboBox && l.starts_with(shown))
        {
            return out;
        }
        // Close any popup the miss may have opened, then re-locate the combo.
        ui.frame(vec![Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }]);
        let out = ui.settle(15);
        at = nodes(&out)
            .into_iter()
            .filter(|(l, _, r)| *r == Role::ComboBox && Some(l) == combo_text.as_ref())
            .map(|(_, p, _)| p)
            .min_by(|a, b| (*a - at).length().total_cmp(&(*b - at).length()))
            .unwrap_or(at);
    }
    panic!("combo did not keep option {option:?}");
}

/// Demo project plus two functional points (A1, A2) on the demo device K1.
fn project() -> Project {
    let mut p = cad_gui::demo::project().unwrap();
    let function = p.functions().next().unwrap().id();
    for name in ["A1", "A2"] {
        p.insert_connection_point(ConnectionPoint::new(
            p.id(),
            function,
            DesignationPart::new(name).unwrap(),
        ))
        .unwrap();
    }
    p
}
fn device(p: &Project) -> DeviceId {
    p.devices().next().unwrap().id()
}

#[test]
fn ribbon_registers_real_connector_commands_without_placeholders() {
    let commands = registry();
    let of = |id: CommandId| {
        commands
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.tab, c.group))
    };
    assert_eq!(
        of(CommandId::ConnectorEditor),
        Some((RibbonTab::Devices, "Conectores"))
    );
    assert_eq!(
        of(CommandId::ConnectorNavigator),
        Some((RibbonTab::Devices, "Conectores"))
    );
    let planned = commands
        .iter()
        .filter(|c| {
            matches!(
                c.id,
                CommandId::Planned(RibbonTab::Devices, "Conectores", _)
            )
        })
        .count();
    assert_eq!(planned, 0, "connector group is implemented, not planned");
}

#[test]
fn connector_is_created_edited_associated_and_located_by_real_clicks() {
    let mut ui = Ui::new(project());
    let before = ui.app.session.snapshot().unwrap();
    let dev = device(ui.project());
    let output = ui.execute(CommandId::ConnectorEditor);
    let create = find(&output, "Criar conector");
    let output = choose(&mut ui, combo(&output, create, 0), "-K1");
    let output = choose(&mut ui, combo(&output, create, 1), "macho");
    let create = find(&output, "Criar conector");
    let fields = fields_on_row(&output, create);
    assert_eq!(fields.len(), 3, "identification, contact count, position");
    ui.type_text(fields[0], "X5");
    ui.type_text(fields[1], "4");
    ui.type_text(fields[2], "c");
    let output = ui.click(create);
    let p = ui.project();
    let connector = p
        .connectors()
        .find(|c| c.designation().as_str() == "X5")
        .unwrap_or_else(|| panic!("connector not created: {:?}", labels(&output)))
        .clone();
    assert_eq!(connector.gender(), ConnectorGender::Male);
    assert_eq!(connector.device(), Some(dev));
    assert_eq!(connector.contacts().len(), 4);
    let first = connector.contacts()[0];
    assert_eq!(p.terminal_label(first), "X5:1");
    assert!(
        p.terminal(first)
            .unwrap()
            .has_position(&DesignationPart::new("c").unwrap())
    );
    // The editor shows the new connector, its properties and contacts.
    let all = labels(&output);
    assert!(
        all.iter()
            .any(|l| l.starts_with("Dispositivo -K1 · gênero macho · 4 contatos (4 livres"))
    );

    // Select contact 1 and rename it: identity is preserved.
    let row = find_where(&output, "contact 1", |l| {
        l.starts_with("1. 1 · posições c · livre")
    });
    let output = ui.click(row);
    assert_eq!(
        ui.app.session.selection,
        Selection::Electrical(RepresentationTarget::Terminal(first))
    );
    let rename = find(&output, "Renomear contato");
    ui.type_text(fields_on_row(&output, rename)[0], "A");
    let output = ui.click(rename);
    // egui may select the prefilled text on focus: either way the typed
    // text renames the same contact.
    let renamed = ui
        .project()
        .terminal(first)
        .unwrap()
        .designation()
        .as_str()
        .to_owned();
    assert!(matches!(renamed.as_str(), "A" | "1A"), "{renamed}");
    assert_eq!(ui.project().terminal_label(first), format!("X5:{renamed}"));

    // Explicit association with a functional point of K1.
    let associate = find(&output, "Associar ponto");
    let output = choose(&mut ui, combo(&output, associate, 0), "-K1:A1");
    let networks = ui.project().networks();
    let output = ui.click(find(&output, "Associar ponto"));
    let status = ui.project().contact_status(first).unwrap();
    assert_eq!(status.points.len(), 1);
    assert_eq!(
        ui.project().networks(),
        networks,
        "association is not continuity"
    );
    let occupied = format!("1. {renamed} · posições c · ocupado");
    assert!(labels(&output).iter().any(|l| l.starts_with(&occupied)));

    // Locate: the connector resolves to an explicit graphic occurrence.
    let expected = ui.project().connector_occurrence(connector.id()).unwrap();
    ui.click(find(&output, "Localizar conector"));
    assert_eq!(
        ui.app.session.selection,
        Selection::Representation(expected)
    );

    // Every step is one history entry; Undo restores the original snapshot.
    for _ in 0..3 {
        ui.app.session.dispatch_graphic(GraphicEdit::Undo).unwrap();
    }
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
    ui.app.session.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert!(ui.project().connector(connector.id()).is_some());
}

#[test]
fn connector_contacts_are_renumbered_and_mated_from_the_gui() {
    let mut p = project();
    let dev = device(&p);
    let contacts = |p: &mut Project, name: &str, gender| {
        let ids: Vec<_> = (1..=3)
            .map(|n| {
                p.insert_terminal(Terminal::new(
                    p.id(),
                    TerminalOwner::Device(dev),
                    DesignationPart::new((n * 10).to_string()).unwrap(),
                ))
                .unwrap()
            })
            .collect();
        let c = Connector::new(p.id(), DesignationPart::new(name).unwrap())
            .with_gender(gender)
            .with_device(Some(dev))
            .with_contacts(ids.clone());
        p.edit_physical(PhysicalEdit::UpsertConnector(c.clone()))
            .unwrap();
        (c.id(), ids)
    };
    let (plug, a) = contacts(&mut p, "X1", ConnectorGender::Male);
    let (socket, b) = contacts(&mut p, "X2", ConnectorGender::Female);
    let mut ui = Ui::new(p);
    let output = ui.execute(CommandId::ConnectorEditor);
    let output = choose(&mut ui, combo(&output, find(&output, "Conector"), 0), "X1");
    // Sequential renumbering keeps the contact IDs.
    let output = ui.click(find(&output, "Renumerar contatos"));
    let p = ui.project();
    assert_eq!(p.connector(plug).unwrap().contacts(), a.as_slice());
    let names: Vec<_> = a
        .iter()
        .map(|t| p.terminal(*t).unwrap().designation().as_str().to_owned())
        .collect();
    assert_eq!(names, ["1", "2", "3"]);
    // Pairing by numbering only fills a draft; it is confirmed explicitly.
    let output = choose(
        &mut ui,
        combo(&output, find(&output, "Conector par"), 0),
        "X2 (fêmea)",
    );
    let output = ui.click(find(&output, "Parear por numeração"));
    assert!(ui.project().connector_matings().next().is_none());
    // X1 is now 1..3 and X2 still 10..30: no equal numbers, empty draft.
    let output = ui.click(find(&output, "Confirmar acoplamento"));
    assert!(ui.project().connector_matings().next().is_none());
    assert!(
        labels(&output)
            .iter()
            .any(|l| l.contains("mapeamento explícito"))
    );
    // Manual explicit pair.
    let add = find(&output, "Adicionar par");
    let output = choose(&mut ui, combo(&output, add, 0), "2");
    let add = find(&output, "Adicionar par");
    let output = choose(&mut ui, combo(&output, add, 1), "30");
    let output = ui.click(find(&output, "Adicionar par"));
    let networks = ui.project().networks();
    let output = ui.click(find(&output, "Confirmar acoplamento"));
    let p = ui.project();
    assert_eq!(p.mated_contact(a[1]), Some(b[2]));
    assert_eq!(
        p.mating_for_connector(socket).map(|m| m.first()),
        Some(plug)
    );
    assert_eq!(p.networks(), networks, "mating creates no continuity");
    assert!(labels(&output).iter().any(|l| l == "X1:2 ↔ X2:30"));
    // Deleting a mated connector is refused and changes nothing.
    let snapshot = ui.app.session.snapshot().unwrap();
    let output = ui.click(find(&output, "Excluir conector e contatos"));
    assert_eq!(ui.app.session.snapshot().unwrap(), snapshot);
    assert!(labels(&output).iter().any(|l| l.contains("referências")));
}

#[test]
fn terminal_specialization_and_levels_are_edited_from_the_strip_editor() {
    let mut p = project();
    let dev = device(&p);
    let terminal = Terminal::new(
        p.id(),
        TerminalOwner::Device(dev),
        DesignationPart::new("5").unwrap(),
    );
    let id = terminal.id();
    let strip =
        TerminalStrip::new(p.id(), DesignationPart::new("XT").unwrap()).with_terminals(vec![id]);
    p.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(terminal),
        PhysicalEdit::UpsertStrip(strip),
    ]))
    .unwrap();
    let mut ui = Ui::new(p);
    let before = ui.app.session.snapshot().unwrap();
    let output = ui.execute(CommandId::TerminalStripEditor);
    let output = choose(
        &mut ui,
        combo_below(&output, find(&output, "Criar régua")),
        "XT",
    );
    let output = ui.click(find_where(&output, "terminal row", |l| {
        l.starts_with("1 · 5 ·")
    }));
    // PE needs a position: the Core rejects it until one is declared.
    let output = choose(
        &mut ui,
        combo(&output, find(&output, "Tipo de borne"), 0),
        "PE (proteção)",
    );
    let output = ui.click(find(&output, "Aplicar especialização"));
    assert_eq!(
        ui.project().terminal(id).unwrap().kind(),
        TerminalKind::Generic
    );
    assert!(
        labels(&output).iter().any(|l| l.contains("borne PE exige")),
        "{:?}",
        labels(&output)
            .into_iter()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
    );
    let positions = find(&output, "Posições do borne");
    let typed = ui.type_text(fields_on_row(&output, positions)[0], "a");
    let output = ui.click(find(&typed, "Aplicar especialização"));
    assert_eq!(
        ui.project().terminal(id).unwrap().kind(),
        TerminalKind::ProtectiveEarth
    );
    // Multilevel with two levels edited in place.
    let output = choose(
        &mut ui,
        combo(&output, find(&output, "Tipo de borne"), 0),
        "multinível",
    );
    let output = ui.click(find(&output, "Adicionar nível"));
    let output = ui.click(find(&output, "Adicionar nível"));
    for (level, position) in [("Nível 1", "a"), ("Nível 2", "b")] {
        let row = fields_on_row(&output, find(&output, level));
        ui.type_text(row[1], position);
    }
    let output = ui.settle(2);
    let output = ui.click(find(&output, "Aplicar especialização"));
    let t = ui.project().terminal(id).unwrap().clone();
    assert_eq!(t.kind(), TerminalKind::MultiLevel, "{:?}", labels(&output));
    let levels: Vec<_> = t
        .levels()
        .iter()
        .map(|l| (l.designation().as_str().to_owned(), l.positions().count()))
        .collect();
    assert_eq!(levels, [("1".to_owned(), 1), ("2".to_owned(), 1)]);
    // Level-aware explicit association of a point.
    let associate = find(&output, "Associar ponto");
    let output = choose(&mut ui, combo(&output, associate, 0), "-K1:A1");
    let associate = find(&output, "Associar ponto");
    let lc = combo(&output, associate, 1);
    let output = choose(&mut ui, lc, "2");
    let output = ui.click(find(&output, "Associar ponto"));
    let feedback: Vec<_> = labels(&output)
        .into_iter()
        .filter(|l| {
            l.contains("relação")
                || l.contains("Alteração")
                || l.contains("selecione")
                || l == "-K1:A1"
                || l == "2"
                || l == "Nível"
        })
        .collect();
    let point = ui
        .project()
        .connection_points()
        .find(|pt| pt.terminal_id() == Some(id))
        .unwrap_or_else(|| panic!("point not associated: {feedback:?}"))
        .clone();
    assert_eq!(point.terminal_level(), Some(t.levels()[1].id()));
    for _ in 0..3 {
        ui.app.session.dispatch_graphic(GraphicEdit::Undo).unwrap();
    }
    assert_eq!(ui.app.session.snapshot().unwrap(), before);
}

fn window(ctx: &Context) -> Rect {
    ctx.memory(|m| m.area_rect(egui::Id::new("Engenharia física")))
        .expect("physical engineering window rendered")
}
fn is_background(shape: &egui::Shape, window: Rect) -> bool {
    match shape {
        egui::Shape::Rect(r) => {
            r.fill.a() > 0
                && (r.rect.min - window.min).length() < 1.5
                && (r.rect.max - window.max).length() < 1.5
        }
        egui::Shape::Vec(shapes) => shapes.iter().any(|s| is_background(s, window)),
        _ => false,
    }
}
/// GUI-PHY-02: content painted only inside the window background.
fn assert_contained(output: &egui::FullOutput, ctx: &Context, what: &str) {
    let rect = window(ctx);
    let start = output
        .shapes
        .iter()
        .position(|c| is_background(&c.shape, rect))
        .unwrap_or_else(|| panic!("{what}: window background not painted"));
    let bounds = rect.expand(1.0);
    for clipped in &output.shapes[start + 1..] {
        let visible = clipped
            .shape
            .visual_bounding_rect()
            .intersect(clipped.clip_rect);
        if visible.is_positive() {
            assert!(
                bounds.contains_rect(visible),
                "{what}: painted at {visible:?} outside {rect:?}"
            );
        }
    }
}

/// GUI-PHY-01/02 on the new tab and on the specialization editor, at several
/// resolutions and DPI scales: no growth, stays on screen, content contained.
#[test]
fn connector_and_specialization_views_are_stable_across_resolutions_and_scales() {
    let mut base = project();
    let dev = device(&base);
    let ids: Vec<_> = (1..=24)
        .map(|n| {
            base.insert_terminal(Terminal::new(
                base.id(),
                TerminalOwner::Device(dev),
                DesignationPart::new(n.to_string()).unwrap(),
            ))
            .unwrap()
        })
        .collect();
    base.edit_physical(PhysicalEdit::UpsertConnector(
        Connector::new(base.id(), DesignationPart::new("X9").unwrap())
            .with_gender(ConnectorGender::Female)
            .with_device(Some(dev))
            .with_contacts(ids),
    ))
    .unwrap();
    let pe = Terminal::new(
        base.id(),
        TerminalOwner::Device(dev),
        DesignationPart::new("PE").unwrap(),
    )
    .with_positions(vec![DesignationPart::new("a").unwrap()])
    .unwrap()
    .with_kind(TerminalKind::ProtectiveEarth);
    let strip = TerminalStrip::new(base.id(), DesignationPart::new("XT").unwrap())
        .with_terminals(vec![pe.id()]);
    base.edit_physical(PhysicalEdit::Batch(vec![
        PhysicalEdit::UpsertTerminal(pe),
        PhysicalEdit::UpsertStrip(strip),
    ]))
    .unwrap();
    for (w, h, dpi) in [
        (1280., 720., 1.0),
        (1366., 768., 1.25),
        (1920., 1080., 1.5),
        (2560., 1440., 2.0),
        (1600., 900., 1.0),
    ] {
        let screen = vec2(w / dpi, h / dpi);
        let what = format!("{w}x{h}@{dpi}");
        let mut ui = Ui::sized(base.clone(), screen, dpi);
        let output = ui.execute(CommandId::ConnectorEditor);
        let output = choose(&mut ui, combo(&output, find(&output, "Conector"), 0), "X9");
        ui.click(find_where(&output, "contact", |l| l.starts_with("1. 1 ·")));
        let output = ui.execute(CommandId::TerminalStripEditor);
        let output = choose(
            &mut ui,
            combo_below(&output, find(&output, "Criar régua")),
            "XT",
        );
        ui.click(find_where(&output, "terminal", |l| {
            l.starts_with("1 · PE [PE (proteção)] ·")
        }));
        for command in [CommandId::ConnectorEditor, CommandId::TerminalStripEditor] {
            ui.execute(command);
            let settled = window(&ui.ctx);
            let output = ui.settle(40);
            let later = window(&ui.ctx);
            assert!(
                (settled.height() - later.height()).abs() < 0.5
                    && (settled.width() - later.width()).abs() < 0.5,
                "{what} {command:?}: {settled:?} grew to {later:?}"
            );
            assert!(
                later.top() >= -0.5 && later.bottom() <= screen.y + 0.5,
                "{what} {command:?}: {later:?} off screen {screen:?}"
            );
            assert!(
                later.width() > 300. && later.height() > 200.,
                "{what}: collapsed"
            );
            assert_contained(&output, &ui.ctx, &format!("{what} {command:?}"));
        }
    }
}
