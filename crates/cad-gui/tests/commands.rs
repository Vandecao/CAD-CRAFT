// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use cad_gui::{
    commands::{CommandId, Requirement, registry},
    preferences::RibbonTab,
    state::Session,
};
use std::collections::HashSet;
#[test]
fn command_registry_has_unique_identifiers_and_complete_metadata() {
    let r = registry();
    let mut ids = HashSet::new();
    for c in r {
        assert!(ids.insert(c.id));
        assert!(!c.name.is_empty());
        assert!(!c.description.is_empty());
        assert!(!c.group.is_empty());
    }
}
#[test]
fn future_commands_are_disabled_even_with_a_project() {
    let mut s = Session::default();
    s.replace(cad_gui::demo::project().unwrap(), true).unwrap();
    for c in registry() {
        if c.requirement == Requirement::Future {
            assert!(!c.available(&s));
            assert!(c.reason(&s).is_some());
        }
    }
}
#[test]
fn page_commands_require_real_context() {
    let mut s = Session::default();
    let r = registry();
    let open = r.iter().find(|c| c.id == CommandId::OpenPage).unwrap();
    assert!(!open.available(&s));
    s.replace(cad_gui::demo::project().unwrap(), true).unwrap();
    let id = s.project().unwrap().pages().next().unwrap().id();
    s.select(cad_gui::state::Selection::Page(id));
    assert!(open.available(&s));
}
#[test]
fn command_search_matches_related_words_and_reports_group() {
    let r = registry();
    let results: Vec<_> = r.iter().filter(|c| c.matches("enquadrar")).collect();
    assert!(results.iter().any(|c| c.id == CommandId::FitPage));
    assert!(results.iter().any(|c| c.id == CommandId::FitContent));
    assert!(results.iter().all(|c| !c.tab.label().is_empty()));
}
#[test]
fn every_ribbon_tab_has_grouped_commands() {
    let r = registry();
    for tab in RibbonTab::ALL {
        assert!(r.iter().any(|c| c.tab == tab), "{tab:?}");
    }
}
