// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Clipboard and macro operations; start with Session::capture_selection.
use super::*;

impl Session {
    /// Copy leaves the project unchanged. Cut removes the selection immediately in one reversible transaction. Rejection preserves the previous clipboard and history.
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
    pub fn clipboard_preview(&self, at: Point2D) -> Result<Vec<Representation>> {
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
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
    /// Paste with the set's lower-left corner at at. Function symbols create new devices or functions of the same device with keep_device. The first paste after cutting restores the same IDs.
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
    /// Window macro from the selection, using the set's grid-snapped lower-left corner as its base point.
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
    /// Page macro from the active page, excluding its frame.
    pub fn macro_from_page(
        &self,
        id: &str,
        name: &str,
        category: &str,
        description: &str,
    ) -> Result<(MacroDefinition, MacroReport)> {
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let page = self
            .active
            .and_then(|id| p.page(id))
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
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
    /// Insert a window macro on the active page with its base point at transform.translation().
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
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
        let p = self.project_mut()?;
        let (delta, report) = p.insert_macro(definition, page, transform)?;
        let ids = delta.representation_ids();
        self.history.macro_insert(delta);
        self.select_graphics(ids);
        self.changed();
        Ok(report)
    }
    /// Insert a page macro as a new page in the active page's document, or the first document, then open it.
    pub fn insert_page_macro(
        &mut self,
        definition: &MacroDefinition,
    ) -> Result<(PageId, MacroReport)> {
        let MacroKind::Page { size, .. } = definition.kind() else {
            return Err(DomainError::InvalidRelation("a macro não é de página"));
        };
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
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
}
