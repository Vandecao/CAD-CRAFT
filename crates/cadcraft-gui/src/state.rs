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

/// Etapa 16: área de transferência da sessão (não persiste).
#[derive(Clone, Debug)]
pub struct PageClipboard {
    project: ProjectId,
    items: Vec<ClipItem>,
    /// Canto inferior esquerdo do conjunto: ponto de referência da colagem.
    origin: Point2D,
    /// Colar símbolo como nova função do mesmo dispositivo (em vez de novo).
    pub keep_device: bool,
    /// Etapa 19: seleção com fios — colada pelo motor de macros.
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
    /// Etapa 19: a cópia leva fios (colagem pelo motor de macros).
    pub fn is_wired(&self) -> bool {
        self.wired.is_some()
    }
}

/// Deslocamento de Duplicar (Ctrl+D), em mm.
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
    /// Etapa 17: símbolos da biblioteca externa (pasta `.cadsym`).
    pub symbol_library: Vec<SymbolDefinition>,
    /// Muda a cada recarga da biblioteca (invalida filtros).
    pub symbol_library_revision: u64,
    /// Etapa 12: arquivo `.cadcraft` associado (com trava) ou `None`.
    pub file: Option<crate::project_file::ProjectFile>,
}
impl Session {
    pub fn apply_report_delta(&mut self, delta: ReportDelta) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.apply_report_delta(p, delta)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    /// Etapa 24: troca o esquema de verificação (com Desfazer).
    pub fn set_check_scheme(&mut self, scheme: CheckScheme) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.check_scheme(p, scheme)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn generate_report(&mut self, request: &ReportRequest) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.generate_report(p, request)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn update_reports(&mut self, groups: &[ReportId]) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.update_reports(p, groups)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn set_numbering_scheme(&mut self, scheme: NumberingScheme) -> Result<()> {
        self.project_mut()?.set_numbering_scheme(scheme)?;
        self.changed();
        Ok(())
    }
    pub fn number_devices(&mut self, preview: &DeviceNumberPreview) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.number_devices(p, preview)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn number_connections(&mut self, preview: &ConnectionNumberPreview) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.number_connections(p, preview)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn set_connection_number(
        &mut self,
        id: ConnectionId,
        number: Option<Text>,
        locked: bool,
    ) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.connection_number(p, id, number, locked)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn number_pages(&mut self, preview: &PageNumberPreview) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.number_pages(p, preview)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    /// Etapa 17: símbolo pelo ID — versão do projeto, depois biblioteca
    /// externa, depois embutidos.
    pub fn find_symbol(&self, id: &SymbolDefinitionId) -> Option<SymbolDefinition> {
        self.project()
            .and_then(|p| p.symbol_definition(id))
            .or_else(|| self.symbol_library.iter().find(|d| d.id() == id))
            .or_else(|| SymbolLibrary::builtin().get(id))
            .cloned()
    }
    /// Etapa 17: catálogo para o Centro de inserção (sem repetir IDs).
    pub fn symbol_catalog(&self) -> Vec<SymbolDefinition> {
        let mut seen = std::collections::BTreeSet::new();
        let project = self
            .project()
            .into_iter()
            .flat_map(|p| p.symbol_definitions());
        project
            .chain(self.symbol_library.iter())
            .chain(SymbolLibrary::builtin().definitions())
            .filter(|d| seen.insert(d.id().clone()))
            .cloned()
            .collect()
    }
    /// IDs da seleção gráfica na página ativa, conferindo visibilidade.
    fn clipboard_selection(&self) -> Result<Vec<RepresentationId>> {
        if self.graphics.junction_selection.is_some() {
            return Err(DomainError::InvalidRelation(
                "junção não admitida na área de transferência",
            ));
        }
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation("abra uma página"))?;
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
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
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
    /// Copiar (`cut = false`) não altera o projeto. Recortar remove a seleção
    /// na hora, numa transação que pode ser desfeita. Em caso de recusa, a
    /// área de transferência e o histórico anteriores ficam intactos.
    pub fn capture_selection(&mut self, cut: bool) -> Result<()> {
        let ids = self.clipboard_selection()?;
        let clipboard = if cut {
            let p = self.project.as_mut().expect("seleção conferida");
            let (items, delta) = p.cut_representations(&ids)?;
            let clipboard = self.clipboard_from(items)?;
            self.history.sheet(delta);
            self.select_graphics([]);
            self.changed();
            clipboard
        } else if let Some(content) = self.wired_selection(&ids)? {
            let p = self.project().expect("conferido");
            PageClipboard {
                project: p.id(),
                items: Vec::new(),
                origin: content.bounds()?.origin(),
                keep_device: false,
                wired: Some(content),
            }
        } else {
            let items = self.project().expect("conferido").clipboard_items(&ids)?;
            self.clipboard_from(items)?
        };
        self.clipboard = Some(clipboard);
        Ok(())
    }
    /// Etapa 19: seleção com fios vira conteúdo de macro (fios e junções
    /// internos entram; o resto segue a regra da Etapa 16).
    fn wired_selection(&self, ids: &[RepresentationId]) -> Result<Option<MacroContent>> {
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
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
    /// Etapa 19: cola o conteúdo com fios em `at` (canto inferior esquerdo).
    fn paste_wired(
        &mut self,
        content: &MacroContent,
        base: Point2D,
        at: Point2D,
    ) -> Result<Vec<RepresentationId>> {
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation("abra uma página"))?;
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
    pub fn clipboard_preview(&self, at: Point2D) -> Result<Vec<Representation>> {
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation("abra uma página"))?;
        let clip = self
            .clipboard
            .as_ref()
            .ok_or(DomainError::InvalidRelation("área de transferência vazia"))?;
        if clip.project != p.id() {
            return Err(DomainError::InvalidRelation(
                "área de transferência de outro projeto",
            ));
        }
        if let Some(content) = &clip.wired {
            let (reps, _) =
                content.preview(clip.origin, Transform2D::new(at, 0.0, false, false)?)?;
            return Ok(reps);
        }
        let (dx, dy) = (at.x() - clip.origin.x(), at.y() - clip.origin.y());
        clip.items
            .iter()
            .map(|item| {
                let r = item.representation();
                let t = r.transform();
                let transform = Transform2D::new(
                    t.translation().translated(dx, dy)?,
                    t.rotation_degrees(),
                    t.mirror_x(),
                    t.mirror_y(),
                )?;
                r.duplicate_on(page, transform)
            })
            .collect()
    }
    /// Cola com o canto inferior esquerdo do conjunto em `at`. Símbolos de
    /// função criam dispositivos novos (ou funções do mesmo dispositivo com
    /// `keep_device`); um recorte volta com os mesmos IDs na primeira colagem.
    pub fn confirm_paste(&mut self, at: Point2D) -> Result<bool> {
        self.clipboard_preview(at)?;
        let page = self.active.expect("conferido na prévia");
        let clip = self.clipboard.as_ref().expect("conferido na prévia");
        if let Some(content) = clip.wired.clone() {
            if clip.keep_device {
                return Err(DomainError::InvalidRelation(
                    "cópia com fios cria dispositivos novos; use Colar",
                ));
            }
            let base = clip.origin;
            self.paste_wired(&content, base, at)?;
            return Ok(true);
        }
        let (dx, dy) = (at.x() - clip.origin.x(), at.y() - clip.origin.y());
        let (items, keep) = (clip.items.clone(), clip.keep_device);
        let p = self.project.as_mut().expect("conferido na prévia");
        let (delta, pasted) = p.paste_items(&items, page, dx, dy, keep)?;
        self.history.sheet(delta);
        if let Some(clip) = &mut self.clipboard {
            clip.items = items
                .iter()
                .zip(&pasted)
                .map(|(item, r)| item.as_copy_of(r.clone()))
                .collect();
            clip.origin = at;
        }
        self.select_graphics(pasted.iter().map(Representation::id));
        self.changed();
        Ok(true)
    }
    /// Duplicar (Ctrl+D): cola a seleção deslocada, sem usar a área de
    /// transferência.
    /// Etapa 17: troca definições do projeto por versões novas (um passo).
    pub fn update_symbols(&mut self, definitions: Vec<SymbolDefinition>) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.symbol_updates(p, definitions)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn assign_part(&mut self, device: DeviceId, part: Part, quantity: u32) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.assign_part(p, device, part, quantity)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn remove_part_assignment(&mut self, component: ComponentId) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.remove_part(p, component)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn update_part(&mut self, part: Part) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.update_part(p, part)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn update_parts(&mut self, parts: Vec<Part>) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.update_parts(p, parts)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn edit_function_point(&mut self, edit: FunctionPointEdit) -> Result<bool> {
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.function_point(p, edit)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    /// Etapa 19: macro de janela a partir da seleção; o ponto base é o canto
    /// inferior esquerdo do conjunto, ajustado à grade.
    pub fn macro_from_selection(
        &self,
        id: &str,
        name: &str,
        category: &str,
        description: &str,
        grid_mm: f64,
    ) -> Result<(MacroDefinition, MacroReport)> {
        let ids = self.clipboard_selection()?;
        let p = self.project().expect("conferido");
        let (content, report) = p.capture_macro(&ids)?;
        let corner = content.bounds()?.origin();
        let base = crate::camera::snap(corner, grid_mm, true);
        let definition = MacroDefinition::new(id, name, MacroKind::Window, base, content)?
            .with_category(category)?
            .with_description(description)?;
        Ok((definition, report))
    }
    /// Etapa 19: macro de página a partir da página ativa (sem a moldura).
    pub fn macro_from_page(
        &self,
        id: &str,
        name: &str,
        category: &str,
        description: &str,
    ) -> Result<(MacroDefinition, MacroReport)> {
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let page = self
            .active
            .and_then(|id| p.page(id))
            .ok_or(DomainError::InvalidRelation("abra uma página"))?;
        let (content, report) = p.capture_page_macro(page.id())?;
        let definition = MacroDefinition::new(
            id,
            name,
            MacroKind::Page {
                name: page.name().clone(),
                kind: page.kind().clone(),
                size: page.size(),
            },
            Point2D::new(0.0, 0.0)?,
            content,
        )?
        .with_category(category)?
        .with_description(description)?;
        Ok((definition, report))
    }
    /// Etapa 19: insere a macro de janela na página ativa com o ponto base em
    /// `transform.translation()`.
    pub fn insert_macro(
        &mut self,
        definition: &MacroDefinition,
        transform: Transform2D,
    ) -> Result<MacroReport> {
        if definition.is_page() {
            return Err(DomainError::InvalidRelation(
                "macro de página: use Inserir como página nova",
            ));
        }
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation("abra uma página"))?;
        let p = self.project_mut()?;
        let (delta, report) = p.insert_macro(definition, page, transform)?;
        let ids = delta.representation_ids();
        self.history.macro_insert(delta);
        self.select_graphics(ids);
        self.changed();
        Ok(report)
    }
    /// Etapa 19: insere a macro de página como página nova no documento da
    /// página ativa (ou no primeiro documento) e a abre.
    pub fn insert_page_macro(
        &mut self,
        definition: &MacroDefinition,
    ) -> Result<(PageId, MacroReport)> {
        let MacroKind::Page { size, .. } = definition.kind() else {
            return Err(DomainError::InvalidRelation("a macro não é de página"));
        };
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let document = self
            .active
            .and_then(|id| p.page(id))
            .map(Page::document_id)
            .or_else(|| p.documents().next().map(Document::id))
            .ok_or(DomainError::InvalidRelation("crie um documento antes"))?;
        let next = p
            .pages()
            .filter(|page| page.document_id() == document)
            .filter_map(|page| page.number().as_str().parse::<u32>().ok())
            .max()
            .unwrap_or(0)
            + 1;
        let default_fits = p
            .default_frame()
            .and_then(|id| p.frame(id))
            .is_none_or(|f| frame_for_size(p, *size).is_none_or(|m| m == f.id()));
        let frame = (!default_fits).then(|| frame_for_size(p, *size)).flatten();
        let p = self.project_mut()?;
        let (delta, report) =
            p.insert_page_macro(definition, document, &next.to_string(), frame)?;
        let page = delta.created_page().expect("macro de página cria a página");
        self.history.macro_insert(delta);
        self.changed();
        self.open_page(page)?;
        Ok((page, report))
    }
    pub fn duplicate_selection(&mut self) -> Result<Vec<RepresentationId>> {
        let ids = self.clipboard_selection()?;
        if let Some(content) = self.wired_selection(&ids)? {
            let base = content.bounds()?.origin();
            let at = base.translated(DUPLICATE_OFFSET.0, DUPLICATE_OFFSET.1)?;
            return self.paste_wired(&content, base, at);
        }
        let page = self.active.expect("conferido");
        let p = self.project.as_mut().expect("conferido");
        let items = p.clipboard_items(&ids)?;
        let (delta, pasted) =
            p.paste_items(&items, page, DUPLICATE_OFFSET.0, DUPLICATE_OFFSET.1, false)?;
        self.history.sheet(delta);
        let ids: Vec<_> = pasted.iter().map(Representation::id).collect();
        self.select_graphics(ids.iter().copied());
        self.changed();
        Ok(ids)
    }
    pub fn delete_graphic_selection(&mut self) -> Result<bool> {
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changes = self
            .graphics
            .selection
            .ids()
            .iter()
            .map(|id| {
                let r = p
                    .representation(*id)
                    .ok_or(DomainError::MissingRepresentation(*id))?;
                if r.graphic().is_none() {
                    return Err(DomainError::InvalidRelation(
                        "exclusão da seleção limitada a conteúdo livre",
                    ));
                }
                Ok(RepresentationChange {
                    before: Some(r.clone()),
                    after: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if changes.is_empty() {
            return Ok(false);
        }
        let p = self.project.as_mut().expect("checked");
        let changed = self.history.representations(p, changes)?;
        if changed {
            self.select_graphics([]);
            self.changed();
        }
        Ok(changed)
    }
    pub fn insert_graphic(
        &mut self,
        graphic: PageGraphic,
        transform: Transform2D,
    ) -> Result<RepresentationId> {
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation("abra uma página"))?;
        let r = Representation::new_graphic(p.id(), page, graphic, transform)?;
        let id = r.id();
        let p = self.project.as_mut().expect("checked");
        self.history.representations(
            p,
            vec![RepresentationChange {
                before: None,
                after: Some(r),
            }],
        )?;
        self.select_graphics([id]);
        self.changed();
        Ok(id)
    }
    pub fn edit_free_graphic(
        &mut self,
        id: RepresentationId,
        graphic: PageGraphic,
    ) -> Result<bool> {
        let before = self
            .project()
            .and_then(|p| p.representation(id))
            .ok_or(DomainError::MissingRepresentation(id))?
            .clone();
        let after = before.clone().with_graphic(graphic)?;
        let p = self.project.as_mut().expect("representation checked");
        let changed = self.history.representations(
            p,
            vec![RepresentationChange {
                before: Some(before),
                after: Some(after),
            }],
        )?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn dispatch_physical(&mut self, edit: PhysicalEdit) -> Result<bool> {
        let project = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let changed = self.history.physical(project, edit)?;
        if changed {
            self.changed();
        }
        Ok(changed)
    }
    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }
    fn project_mut(&mut self) -> Result<&mut Project> {
        self.project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))
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
    pub fn select_graphics(&mut self, ids: impl IntoIterator<Item = RepresentationId>) {
        self.graphics.cancel();
        self.graphics.junction_selection = None;
        let valid: Vec<_> = ids
            .into_iter()
            .filter(|id| {
                self.project()
                    .and_then(|p| p.representation(*id))
                    .is_some_and(|r| Some(r.page_id()) == self.active && r.visual().visible)
            })
            .collect();
        self.graphics.selection.replace(valid);
        self.sync_graphic_selection();
    }
    pub fn sync_graphic_selection(&mut self) {
        self.graphics.junction_selection = None;
        self.selection_revision = self.selection_revision.wrapping_add(1);
        self.selection = self.graphics.selection.primary().map_or_else(
            || self.active.map_or(Selection::None, Selection::Page),
            Selection::Representation,
        );
    }
    pub fn dispatch_graphic(&mut self, edit: GraphicEdit) -> Result<bool> {
        let previous_selection = self.selection;
        // Etapa 13: edições de moldura/metadados vêm do painel e mantêm a seleção.
        let keeps_selection = matches!(edit, GraphicEdit::Frame(_));
        // Etapa 17: símbolo do projeto, da biblioteca externa ou embutido.
        let symbol = match &edit {
            GraphicEdit::InsertSymbol { definition, .. }
            | GraphicEdit::PlaceSymbol { definition, .. } => self.find_symbol(definition),
            _ => None,
        };
        let p = self
            .project
            .as_mut()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let mut inserted = None;
        let changed = match edit {
            GraphicEdit::Device(edit) => self.history.device(p, edit)?,
            GraphicEdit::Marker(edit) => {
                if let MarkerEdit::Insert { ref marker, .. } = edit {
                    inserted = Some(marker.id());
                }
                self.history.marker(p, edit)?
            }
            GraphicEdit::Drawing(edit) => self.history.drawing(p, edit)?,
            GraphicEdit::InsertSymbol {
                definition,
                target,
                transform,
            } => {
                let page = self
                    .active
                    .ok_or(DomainError::InvalidRelation("abra uma página para inserir"))?;
                let _ = definition;
                let definition = symbol.as_ref().ok_or(DomainError::InvalidRelation(
                    "símbolo ausente na biblioteca",
                ))?;
                inserted = Some(
                    self.history
                        .insert_symbol(p, definition, page, target, transform)?,
                );
                true
            }
            GraphicEdit::PlaceSymbol {
                definition,
                plan,
                transform,
            } => {
                let page = self
                    .active
                    .ok_or(DomainError::InvalidRelation("abra uma página para inserir"))?;
                let _ = definition;
                let definition = symbol.as_ref().ok_or(DomainError::InvalidRelation(
                    "símbolo ausente na biblioteca",
                ))?;
                if let DevicePlan::ExistingFunction(function) = plan {
                    inserted = Some(self.history.place_existing_function(
                        p,
                        ExistingFunctionPlacement {
                            page,
                            definition: definition.clone(),
                            transform,
                            function,
                            allow_order_fallback: true,
                        },
                    )?);
                    true
                } else {
                    let device = match plan {
                        DevicePlan::New { prefix } => {
                            let prefix = if prefix.trim().is_empty() {
                                default_prefix(definition)
                            } else {
                                prefix.trim()
                            };
                            PlacementDevice::New {
                                designation: p.next_device_designation_at(
                                    prefix,
                                    page,
                                    transform.translation(),
                                )?,
                                classification: definition.name().to_owned(),
                            }
                        }
                        DevicePlan::Existing(id) => PlacementDevice::Existing(id),
                        DevicePlan::ExistingFunction(_) => unreachable!(),
                    };
                    inserted = Some(self.history.place(
                        p,
                        SymbolPlacement {
                            page,
                            definition: definition.clone(),
                            transform,
                            device,
                            function_kind: function_kind(definition)?,
                        },
                    )?);
                    true
                }
            }
            GraphicEdit::Document(edit) => self.history.document(p, edit)?,
            GraphicEdit::Frame(edit) => self.history.frame(p, edit)?,
            GraphicEdit::Undo => self.history.undo(p)?,
            GraphicEdit::Redo => self.history.redo(p)?,
            GraphicEdit::Set(batch) => self.history.commit(p, batch)?,
            GraphicEdit::Visual(id, visual) => self.history.visual(p, id, visual)?,
            operation => {
                let mut batch = Vec::with_capacity(self.graphics.selection.ids().len());
                for &id in self.graphics.selection.ids() {
                    let r = p
                        .representation(id)
                        .ok_or(DomainError::MissingRepresentation(id))?;
                    if Some(r.page_id()) != self.active || !r.visual().visible {
                        return Err(DomainError::InvalidRelation(
                            "seleção inválida para edição gráfica",
                        ));
                    }
                    let t = r.transform();
                    let new = match operation {
                        GraphicEdit::Translate(x, y) => Transform2D::new(
                            t.translation().translated(x, y)?,
                            t.rotation_degrees(),
                            t.mirror_x(),
                            t.mirror_y(),
                        )?,
                        GraphicEdit::Rotate(a) => Transform2D::new(
                            t.translation(),
                            t.rotation_degrees() + a,
                            t.mirror_x(),
                            t.mirror_y(),
                        )?,
                        GraphicEdit::MirrorHorizontal => Transform2D::new(
                            t.translation(),
                            -t.rotation_degrees(),
                            !t.mirror_x(),
                            t.mirror_y(),
                        )?,
                        GraphicEdit::MirrorVertical => Transform2D::new(
                            t.translation(),
                            -t.rotation_degrees(),
                            t.mirror_x(),
                            !t.mirror_y(),
                        )?,
                        _ => unreachable!(),
                    };
                    batch.push((id, new));
                }
                self.history.commit(p, batch)?
            }
        };
        self.graphics.cancel();
        if changed {
            self.prune_tabs();
            let ids: Vec<_> = inserted
                .map(|id| vec![id])
                .unwrap_or_else(|| self.graphics.selection.ids().iter().copied().collect());
            self.select_graphics(ids);
            if inserted.is_none()
                && (keeps_selection || matches!(previous_selection, Selection::Electrical(_)))
            {
                self.select(previous_selection);
            }
            self.changed();
        }
        Ok(changed)
    }
    pub fn begin_symbol_insertion(
        &mut self,
        definition: SymbolDefinitionId,
        target: Option<RepresentationTarget>,
        transform: Transform2D,
    ) -> Result<()> {
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation("abra uma página para inserir"))?;
        let p = self
            .project()
            .ok_or(DomainError::InvalidRelation("nenhum projeto aberto"))?;
        let symbol = self
            .find_symbol(&definition)
            .ok_or(DomainError::InvalidRelation(
                "símbolo ausente na biblioteca",
            ))?;
        let symbol = &symbol;
        if !symbol.orientations().allows(transform) {
            return Err(DomainError::InvalidRelation(
                "orientação não permitida pela definição",
            ));
        }
        if p.symbol_definition(&definition)
            .is_some_and(|existing| existing != symbol)
        {
            return Err(DomainError::InvalidRelation(
                "o documento contém outra versão desta definição",
            ));
        }
        self.graphics.cancel();
        self.graphics.tool = crate::interaction::Tool::InsertSymbol;
        self.graphics.insertion = Some(crate::interaction::InsertionDraft {
            definition,
            page,
            target,
            transform,
            positioned: false,
            device: None,
        });
        Ok(())
    }
    /// Etapa 12: cada clique cria a ocorrência e o cadastro elétrico dela.
    pub fn begin_device_insertion(
        &mut self,
        definition: SymbolDefinitionId,
        plan: DevicePlan,
        transform: Transform2D,
    ) -> Result<()> {
        if let DevicePlan::Existing(id) = plan
            && self.project().and_then(|p| p.device(id)).is_none()
        {
            return Err(DomainError::MissingEntity(EntityRef::Device(id)));
        }
        if let DevicePlan::ExistingFunction(id) = plan
            && self.project().and_then(|p| p.function(id)).is_none()
        {
            return Err(DomainError::MissingEntity(EntityRef::Function(id)));
        }
        if let DevicePlan::New { prefix } = &plan
            && !prefix.trim().is_empty()
        {
            DesignationPart::new(format!("{}1", prefix.trim()))?;
        }
        self.begin_symbol_insertion(definition, None, transform)?;
        if let Some(draft) = &mut self.graphics.insertion {
            draft.device = Some(plan);
        }
        Ok(())
    }
    pub fn confirm_symbol_insertion(&mut self) -> Result<RepresentationId> {
        let draft = self
            .graphics
            .insertion
            .clone()
            .filter(|d| d.positioned && Some(d.page) == self.active)
            .ok_or(DomainError::InvalidRelation(
                "posicione um símbolo na página ativa",
            ))?;
        self.dispatch_graphic(match draft.device.clone() {
            Some(plan) => GraphicEdit::PlaceSymbol {
                definition: draft.definition.clone(),
                plan,
                transform: draft.transform,
            },
            None => GraphicEdit::InsertSymbol {
                definition: draft.definition.clone(),
                target: draft.target,
                transform: draft.transform,
            },
        })?;
        let id = self
            .graphics
            .selection
            .primary()
            .expect("insertion selects the new representation");
        // Repeated insertion is explicit: only Esc / context changes end the tool.
        self.graphics.tool = crate::interaction::Tool::InsertSymbol;
        self.graphics.insertion = Some(draft);
        Ok(id)
    }
    pub fn replace(&mut self, project: Project, demonstration: bool) -> Result<()> {
        project.validate()?;
        self.history.clear();
        self.reset_graphics();
        self.clipboard = None;
        self.project = Some(project);
        self.selection = Selection::Project;
        self.tabs.clear();
        self.active = None;
        self.dirty = false;
        self.demonstration = demonstration;
        // Releases the previous file lock; callers attach the new file afterwards.
        self.file = None;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
    pub fn close(&mut self) {
        self.history.clear();
        self.reset_graphics();
        self.clipboard = None;
        self.project = None;
        self.selection = Selection::None;
        self.tabs.clear();
        self.active = None;
        self.dirty = false;
        self.demonstration = false;
        self.file = None;
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn select(&mut self, selection: Selection) {
        let valid = self.project.as_ref().is_some_and(|p| match selection {
            Selection::None | Selection::Project => true,
            Selection::Document(id) => p.document(id).is_some(),
            Selection::Page(id) => p.page(id).is_some(),
            Selection::Representation(id) => p.representation(id).is_some(),
            Selection::Electrical(target) => p.representations_for_entity(target).is_ok(),
        });
        if valid || selection == Selection::None {
            self.selection_revision = self.selection_revision.wrapping_add(1);
            self.graphics.cancel();
            self.graphics.selection.replace(match selection {
                Selection::Representation(id) => Some(id),
                _ => None,
            });
            self.selection = selection;
        }
    }
    pub fn selected_page(&self) -> Option<PageId> {
        match self.selection {
            Selection::Page(id) => Some(id),
            Selection::Representation(id) => self
                .project()?
                .representation(id)
                .map(Representation::page_id),
            _ => None,
        }
    }
    pub fn selected_document(&self) -> Option<DocumentId> {
        match self.selection {
            Selection::Document(id) => Some(id),
            _ => self
                .selected_page()
                .and_then(|id| self.project()?.page(id).map(Page::document_id))
                .or_else(|| self.project()?.documents().next().map(Document::id)),
        }
    }
    pub fn open_page(&mut self, id: PageId) -> Result<()> {
        if self.project().and_then(|p| p.page(id)).is_none() {
            return Err(DomainError::MissingEntity(EntityRef::Page(id)));
        }
        if !self.tabs.iter().any(|t| t.page == id) {
            self.tabs.push(PageTab {
                page: id,
                camera: Camera::default(),
                previous: None,
                needs_fit: true,
            });
        }
        self.active = Some(id);
        self.reset_graphics();
        self.selection = Selection::Page(id);
        Ok(())
    }
    pub fn close_tab(&mut self, id: PageId) {
        let index = self.tabs.iter().position(|t| t.page == id);
        self.tabs.retain(|t| t.page != id);
        if self.active == Some(id) {
            self.reset_graphics();
            self.active = index
                .and_then(|i| self.tabs.get(i.min(self.tabs.len().saturating_sub(1))))
                .map(|t| t.page);
            self.selection = self.active.map_or(
                if self.project.is_some() {
                    Selection::Project
                } else {
                    Selection::None
                },
                Selection::Page,
            );
        }
    }
    pub fn active_tab_mut(&mut self) -> Option<&mut PageTab> {
        let id = self.active?;
        self.tabs.iter_mut().find(|t| t.page == id)
    }
    pub fn apply(&mut self, edit: Edit) -> Result<()> {
        let edit = match edit {
            Edit::PageName(page, name) => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::PageName {
                        page,
                        name: Text::new(name)?,
                    }))
                    .map(|_| ());
            }
            Edit::PageNumber(page, number) => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::PageNumber {
                        page,
                        number: Text::new(number)?,
                    }))
                    .map(|_| ());
            }
            Edit::PageStructure(page, structure) => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::PageStructure {
                        page,
                        structure,
                    }))
                    .map(|_| ());
            }
            Edit::RepresentationTransform(representation, page, transform) => {
                return self
                    .dispatch_graphic(GraphicEdit::Device(DeviceEdit::MoveRepresentation {
                        representation,
                        page,
                        transform,
                    }))
                    .map(|_| ());
            }
            other => other,
        };
        if let Edit::RepresentationVisual(id, visual) = edit {
            return self
                .dispatch_graphic(GraphicEdit::Visual(id, visual))
                .map(|_| ());
        }
        if let Edit::RepresentationTransform(id, page, t) = &edit
            && self
                .project()
                .and_then(|p| p.representation(*id))
                .is_some_and(|r| r.page_id() == *page)
        {
            return self
                .dispatch_graphic(GraphicEdit::Set(vec![(*id, *t)]))
                .map(|_| ());
        }
        edit.apply(self.project_mut()?)?;
        self.history.clear();
        self.graphics.cancel();
        let ids: Vec<_> = self.graphics.selection.ids().iter().copied().collect();
        if !ids.is_empty() {
            self.select_graphics(ids);
        }
        self.changed();
        Ok(())
    }
    /// Etapa 12: documentos e páginas são reversíveis pelo histórico comum.
    pub fn new_document(&mut self, name: &str) -> Result<DocumentId> {
        let p = self.project_mut()?;
        let document = Document::new(p.id(), name)?;
        let id = document.id();
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::InsertDocument(
            document,
        )))?;
        self.selection = Selection::Document(id);
        Ok(id)
    }
    pub fn new_page(
        &mut self,
        document: DocumentId,
        name: &str,
        number: &str,
        kind: PageKind,
        size: Size2D,
    ) -> Result<PageId> {
        let p = self.project_mut()?;
        // Etapa 13: sem moldura padrão compatível, usa a moldura do mesmo tamanho.
        let default_fits = p
            .default_frame()
            .and_then(|id| p.frame(id))
            .is_none_or(|f| frame_for_size(p, size).is_none_or(|m| m == f.id()));
        let own = (!default_fits).then(|| frame_for_size(p, size)).flatten();
        let page = Page::new(p.id(), document, name, number, kind, size)?.with_frame(own);
        let id = page.id();
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::InsertPage(Box::new(
            page,
        ))))?;
        self.selection = Selection::Page(id);
        Ok(id)
    }
    pub fn delete_page(&mut self, id: PageId) -> Result<()> {
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::RemovePage(id)))?;
        if self.selection == Selection::Page(id) {
            self.selection = Selection::Project;
        }
        Ok(())
    }
    pub fn reorder(&mut self, document: DocumentId, order: &[PageId]) -> Result<()> {
        self.dispatch_graphic(GraphicEdit::Document(DocumentEdit::ReorderPages {
            document,
            order: order.to_vec(),
        }))?;
        Ok(())
    }
    pub fn move_page(&mut self, id: PageId, direction: i32) -> Result<()> {
        let p = self.project_mut()?;
        let doc = p
            .page(id)
            .ok_or(DomainError::MissingEntity(EntityRef::Page(id)))?
            .document_id();
        let mut order: Vec<_> = p.ordered_pages(doc)?.iter().map(|p| p.id()).collect();
        if let Some(index) = order.iter().position(|p| *p == id) {
            let next = index
                .saturating_add_signed(direction as isize)
                .min(order.len() - 1);
            order.swap(index, next);
            self.reorder(doc, &order)?;
        }
        Ok(())
    }
    /// Abas de páginas removidas (exclusão ou Undo de criação) são fechadas.
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
    pub fn snapshot(&self) -> std::result::Result<String, String> {
        serde_json::to_string_pretty(self.project().ok_or("Nenhum projeto aberto")?)
            .map_err(|e| e.to_string())
    }
    pub fn import_snapshot(&mut self, json: &str) -> std::result::Result<(), String> {
        if json.len() > 8 * 1024 * 1024 {
            return Err("Snapshot técnico excede o limite de 8 MiB".into());
        }
        let project: Project = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.replace(project, false).map_err(|e| e.to_string())
    }
}

/// Prefixo de designação sugerido por símbolo (editável no centro de inserção).
pub fn default_prefix(definition: &SymbolDefinition) -> &'static str {
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
/// Tipo funcional da nova função; símbolos sem tipo próprio usam o nome.
pub fn function_kind(definition: &SymbolDefinition) -> Result<FunctionKind> {
    Ok(match definition.id().as_str() {
        "cad-craft.coil" => FunctionKind::Coil,
        "cad-craft.contact-no" | "cad-craft.contact-nc" => FunctionKind::AuxiliaryContact,
        _ => FunctionKind::Other(Text::new(definition.name())?),
    })
}

/// Etapa 13: projeto novo recebe as molduras de fábrica; A3 é a padrão.
pub fn with_factory_frames(mut project: Project) -> Result<Project> {
    let a3 = FrameTemplate::factory_a3();
    let id = a3.id();
    project.edit_frame(FrameEdit::UpsertFrame(a3))?;
    project.edit_frame(FrameEdit::UpsertFrame(FrameTemplate::factory_a4()))?;
    project.edit_frame(FrameEdit::SetDefaultFrame(Some(id)))?;
    Ok(project)
}
/// Moldura do projeto cuja folha coincide com o tamanho (tolerância 0,5 mm).
pub fn frame_for_size(project: &Project, size: Size2D) -> Option<FrameTemplateId> {
    project
        .frames()
        .find(|f| {
            (f.sheet().width() - size.width()).abs() < 0.5
                && (f.sheet().height() - size.height()).abs() < 0.5
        })
        .map(FrameTemplate::id)
}
