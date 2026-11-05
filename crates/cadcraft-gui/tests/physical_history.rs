// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_craft::domain::*;
use cad_gui::{editing::GraphicEdit, state::Session};

#[test]
fn physical_strip_edit_uses_the_existing_undo_redo_history() {
    let mut session = Session::default();
    session
        .replace(Project::new("Engenharia").unwrap(), true)
        .unwrap();
    let before = session.snapshot().unwrap();
    let strip = TerminalStrip::new(
        session.project().unwrap().id(),
        DesignationPart::new("X1").unwrap(),
    );
    let id = strip.id();
    assert!(
        session
            .dispatch_physical(PhysicalEdit::UpsertStrip(strip))
            .unwrap()
    );
    let after = session.snapshot().unwrap();
    assert!(session.project().unwrap().terminal_strip(id).is_some());
    session.dispatch_graphic(GraphicEdit::Undo).unwrap();
    assert_eq!(session.snapshot().unwrap(), before);
    session.dispatch_graphic(GraphicEdit::Redo).unwrap();
    assert_eq!(session.snapshot().unwrap(), after);
}
