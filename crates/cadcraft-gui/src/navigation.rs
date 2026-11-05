// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Rebuildable presentation index, never serialized as a second document model.
use cad_craft::domain::*;
#[derive(Clone, Debug)]
pub struct PageRow {
    pub id: PageId,
    pub document: DocumentId,
    pub document_name: String,
    pub name: String,
    pub number: String,
    pub kind: PageKind,
    pub functional: String,
    pub location: String,
    pub path: Vec<String>,
    pub search: String,
}
#[derive(Clone, Default, PartialEq)]
pub struct PageFilter {
    pub text: String,
    pub kind: Option<PageKind>,
    pub document: Option<DocumentId>,
    pub structure: String,
}
#[derive(Default)]
pub struct PageIndex {
    pub rows: Vec<PageRow>,
}
pub fn path_text(path: &[DesignationPart]) -> String {
    path.iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("/")
}
pub fn kind_label(kind: &PageKind) -> &str {
    match kind {
        PageKind::MultiLine => "Multifilar",
        PageKind::SingleLine => "Unifilar",
        PageKind::Overview => "Visão geral",
        PageKind::PanelLayout => "Layout de painel",
        PageKind::TechnicalDocumentation => "Documentação técnica",
        PageKind::Other(text) => text.as_str(),
    }
}
pub fn page_kinds() -> [PageKind; 5] {
    [
        PageKind::MultiLine,
        PageKind::SingleLine,
        PageKind::Overview,
        PageKind::PanelLayout,
        PageKind::TechnicalDocumentation,
    ]
}
impl PageIndex {
    pub fn build(project: &Project) -> Self {
        let mut rows = Vec::new();
        for doc in project.documents() {
            // Core query validates ownership and establishes the document order.
            for page in project.ordered_pages(doc.id()).unwrap_or_default() {
                let structure = page.structure();
                let functional = path_text(structure.functional_path());
                let location = path_text(structure.location_path());
                let mut path: Vec<_> = structure
                    .functional_path()
                    .iter()
                    .map(|p| format!("={}", p.as_str()))
                    .collect();
                path.extend(
                    structure
                        .location_path()
                        .iter()
                        .map(|p| format!("+{}", p.as_str())),
                );
                if path.is_empty() {
                    path.push("Sem classificação".into());
                }
                let search = format!(
                    "{} {} {} {} {} {}",
                    doc.name().as_str(),
                    page.number().as_str(),
                    page.name().as_str(),
                    kind_label(page.kind()),
                    functional,
                    location
                )
                .to_lowercase();
                rows.push(PageRow {
                    id: page.id(),
                    document: doc.id(),
                    document_name: doc.name().as_str().into(),
                    name: if page
                        .report()
                        .is_some_and(|r| project.report_stale(r.group).unwrap_or(false))
                    {
                        format!("{} [desatualizado]", page.name().as_str())
                    } else {
                        page.name().as_str().into()
                    },
                    number: page.number().as_str().into(),
                    kind: page.kind().clone(),
                    functional,
                    location,
                    path,
                    search,
                });
            }
        }
        Self { rows }
    }
    pub fn filtered(&self, filter: &PageFilter) -> Vec<&PageRow> {
        let text = filter.text.to_lowercase();
        let structure = filter.structure.to_lowercase();
        self.rows
            .iter()
            .filter(|p| {
                filter.document.is_none_or(|id| id == p.document)
                    && filter.kind.as_ref().is_none_or(|k| *k == p.kind)
                    && p.search.contains(&text)
                    && (p.functional.to_lowercase().contains(&structure)
                        || p.location.to_lowercase().contains(&structure))
            })
            .collect()
    }
}
