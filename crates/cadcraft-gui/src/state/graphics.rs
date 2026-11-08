// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Graphic and physical editing; start with Session::dispatch_graphic.
use super::*;

impl Session {
    pub fn delete_graphic_selection(&mut self) -> Result<bool> {
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
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
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let page = self
            .active
            .ok_or(DomainError::InvalidRelation(crate::strings::OPEN_PAGE))?;
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
        let project = self.project.as_mut().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let changed = self.history.physical(project, edit)?;
        if changed {
            self.changed();
        }
        Ok(changed)
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
        // Panel frame and metadata edits preserve selection.
        let keeps_selection = matches!(edit, GraphicEdit::Frame(_));
        // Symbol from the project, external library or built-ins.
        let symbol = match &edit {
            GraphicEdit::InsertSymbol { definition, .. }
            | GraphicEdit::PlaceSymbol { definition, .. } => self.find_symbol(definition),
            _ => None,
        };
        let p = self.project.as_mut().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
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
                let page = self.active.ok_or(DomainError::InvalidRelation(
                    crate::strings::OPEN_PAGE_TO_INSERT,
                ))?;
                let _ = definition;
                let definition = symbol.as_ref().ok_or(DomainError::InvalidRelation(
                    crate::strings::SYMBOL_NOT_IN_LIBRARY,
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
                let page = self.active.ok_or(DomainError::InvalidRelation(
                    crate::strings::OPEN_PAGE_TO_INSERT,
                ))?;
                let _ = definition;
                let definition = symbol.as_ref().ok_or(DomainError::InvalidRelation(
                    crate::strings::SYMBOL_NOT_IN_LIBRARY,
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
            // Inspector edits on a page, document, project or electrical
            // entity keep that entity selected; falling back to the active
            // tab's page sent the next edit to the wrong page.
            if inserted.is_none()
                && (keeps_selection
                    || matches!(
                        previous_selection,
                        Selection::Electrical(_)
                            | Selection::Page(_)
                            | Selection::Document(_)
                            | Selection::Project
                    ))
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
        let page = self.active.ok_or(DomainError::InvalidRelation(
            crate::strings::OPEN_PAGE_TO_INSERT,
        ))?;
        let p = self.project().ok_or(DomainError::InvalidRelation(
            crate::strings::NO_OPEN_PROJECT,
        ))?;
        let symbol = self
            .find_symbol(&definition)
            .ok_or(DomainError::InvalidRelation(
                crate::strings::SYMBOL_NOT_IN_LIBRARY,
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
    /// Each click creates the occurrence and its electrical registration.
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
}
