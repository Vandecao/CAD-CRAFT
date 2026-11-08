// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Editor session and selection state; start with Session and its dispatch methods.
use crate::{
    camera::Camera,
    editing::{Edit, GraphicEdit},
    history::History,
    interaction::{DevicePlan, Interaction},
};
use cad_craft::{
    domain::*,
    error::{DomainError, Result},
};

/// Session clipboard, never persisted.
#[derive(Clone, Debug)]
pub struct PageClipboard {
    project: ProjectId,
    items: Vec<ClipItem>,
    /// Lower-left corner of the set, used as the paste reference point.
    origin: Point2D,
    /// Paste a symbol as a new function of the same device instead of a new device.
    pub keep_device: bool,
    /// Selection with wires, pasted by the macro engine.
    wired: Option<MacroContent>,
}
impl PageClipboard {
    pub fn is_cut(&self) -> bool {
        self.items.iter().any(ClipItem::is_cut)
    }
    pub fn len(&self) -> usize {
        match &self.wired {
            Some(c) => c.representations().len() + c.routes().len(),
            None => self.items.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The copy includes wires and uses the macro engine for pasting.
    pub fn is_wired(&self) -> bool {
        self.wired.is_some()
    }
}

/// Duplicate (Ctrl+D) offset in millimetres.
pub const DUPLICATE_OFFSET: (f64, f64) = (10.0, -10.0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    #[default]
    None,
    Project,
    Document(DocumentId),
    Page(PageId),
    Representation(RepresentationId),
    Electrical(RepresentationTarget),
}
#[derive(Clone, Debug)]
pub struct PageTab {
    pub page: PageId,
    pub camera: Camera,
    pub previous: Option<Camera>,
    pub needs_fit: bool,
}
#[derive(Default)]
pub struct Session {
    project: Option<Project>,
    pub selection: Selection,
    pub tabs: Vec<PageTab>,
    pub active: Option<PageId>,
    pub dirty: bool,
    pub demonstration: bool,
    pub revision: u64,
    /// Invalidates inspector drafts even if a hidden panel misses a round-trip.
    pub selection_revision: u64,
    pub graphics: Interaction,
    pub history: History,
    pub clipboard: Option<PageClipboard>,
    /// External symbol library from the .cadsym directory.
    pub symbol_library: Vec<SymbolDefinition>,
    /// Changes on each library reload, invalidating filters.
    pub symbol_library_revision: u64,
    /// Associated .cadcraft file with its lock, or None.
    pub file: Option<crate::project_file::ProjectFile>,
}
mod clipboard;
mod documents;
mod domain;
mod graphics;

impl Session {
    /// Graphical selection IDs on the active page, checking visibility.
    fn clipboard_selection(&self) -> Result<Vec<RepresentationId>> {
        if self.graphics.junction_selection.is_some() {
            return Err(DomainError::InvalidRelation(
                "junção não admitida na área de transferência",
            ));
        }
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
        let ids: Vec<_> = self.graphics.selection.ids().iter().copied().collect();
        for id in &ids {
            let r = p
                .representation(*id)
                .ok_or(DomainError::MissingRepresentation(*id))?;
            if r.page_id() != page || !r.visual().visible {
                return Err(DomainError::InvalidRelation(
                    "seleção fora da página ativa ou oculta",
                ));
            }
        }
        Ok(ids)
    }
    fn clipboard_from(&self, items: Vec<ClipItem>) -> Result<PageClipboard> {
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let mut min = (f64::INFINITY, f64::INFINITY);
        for item in &items {
            let b = item.representation().bounds()?;
            min = (min.0.min(b.origin().x()), min.1.min(b.origin().y()));
        }
        Ok(PageClipboard {
            project: p.id(),
            items,
            origin: Point2D::new(min.0, min.1)?,
            keep_device: false,
            wired: None,
        })
    }
    /// Convert a selection with wires to macro content, including internal wires and junctions; other content follows the graphical clipboard rules.
    fn wired_selection(&self, ids: &[RepresentationId]) -> Result<Option<MacroContent>> {
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        if !ids.iter().any(|id| p.connection_route(*id).is_some()) {
            return Ok(None);
        }
        let (content, _) = p.capture_macro(ids)?;
        if content.routes().is_empty() {
            return Err(DomainError::InvalidRelation(
                "fios selecionados sem as duas pontas na seleção",
            ));
        }
        Ok(Some(content))
    }
    /// Paste content with wires at the lower-left corner at.
    fn paste_wired(
        &mut self,
        content: &MacroContent,
        base: Point2D,
        at: Point2D,
    ) -> Result<Vec<RepresentationId>> {
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
        let p = self.project_mut()?;
        let (delta, _) = p.insert_macro_content(
            content,
            MacroPlacement {
                page,
                base,
                transform: Transform2D::new(at, 0.0, false, false)?,
            },
        )?;
        let ids = delta.representation_ids();
        self.history.macro_insert(delta);
        self.select_graphics(ids.iter().copied());
        self.changed();
        Ok(ids)
    }
    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }
    fn project_mut(&mut self) -> Result<&mut Project> {
        self.project.as_mut().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))
    }
    fn changed(&mut self) {
        self.dirty = true;
        self.revision = self.revision.wrapping_add(1);
        // Network membership is derived; discard highlights after any mutation.
        self.graphics.highlighted_routes.clear();
        self.graphics.junction_selection = self.graphics.junction_selection.filter(|id| {
            self.project
                .as_ref()
                .is_some_and(|p| p.connection_junction(*id).is_some())
        });
    }
    fn reset_graphics(&mut self) {
        self.selection_revision = self.selection_revision.wrapping_add(1);
        self.graphics = Interaction::default();
    }
    /// Close tabs for pages removed by deletion or undoing creation.
    fn prune_tabs(&mut self) {
        let missing: Vec<_> = self
            .tabs
            .iter()
            .map(|t| t.page)
            .filter(|id| self.project().and_then(|p| p.page(*id)).is_none())
            .collect();
        for id in missing {
            self.close_tab(id);
        }
    }
}

/// Suggested designation prefix per symbol, editable in the insertion centre:
/// the prefix recorded in the symbol, else the one for the factory ID.
pub fn default_prefix(definition: &SymbolDefinition) -> &str {
    if let Some(prefix) = definition.designation_prefix() {
        return prefix;
    }
    match definition.id().as_str() {
        "cad-craft.contact-no" | "cad-craft.contact-nc" | "cad-craft.coil" => "K",
        "cad-craft.motor" => "M",
        "cad-craft.terminal" => "X",
        "cad-craft.push-button" | "cad-craft.switch" => "S",
        "cad-craft.sensor" => "B",
        "cad-craft.protective-earth" => "PE",
        "cad-craft.supply" => "G",
        _ => "D",
    }
}
/// Function type for a new function: the kind recorded in the symbol, else
/// the one for the factory ID; untyped symbols use their name.
pub fn function_kind(definition: &SymbolDefinition) -> Result<FunctionKind> {
    if let Some(kind) = definition.function_kind_hint() {
        return Ok(kind);
    }
    Ok(match definition.id().as_str() {
        "cad-craft.coil" => FunctionKind::Coil,
        "cad-craft.contact-no" | "cad-craft.contact-nc" => FunctionKind::AuxiliaryContact,
        _ => FunctionKind::Other(Text::new(definition.name())?),
    })
}

/// Add factory frames to a new project, with A3 as the default.
pub fn with_factory_frames(mut project: Project) -> Result<Project> {
    let a3 = FrameTemplate::factory_a3();
    let id = a3.id();
    project.edit_frame(FrameEdit::UpsertFrame(a3))?;
    project.edit_frame(FrameEdit::UpsertFrame(FrameTemplate::factory_a4()))?;
    project.edit_frame(FrameEdit::SetDefaultFrame(Some(id)))?;
    Ok(project)
}
/// Project frame matching the sheet dimensions within 0.5 mm.
pub fn frame_for_size(project: &Project, size: Size2D) -> Option<FrameTemplateId> {
    project
        .frames()
        .find(|f| {
            (f.sheet().width() - size.width()).abs() < 0.5
                && (f.sheet().height() - size.height()).abs() < 0.5
        })
        .map(FrameTemplate::id)
}
