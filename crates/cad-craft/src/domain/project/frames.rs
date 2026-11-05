// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Etapa 13: molduras do projeto, atribuição por página, textos especiais e
//! referências por zona. Edições produzem deltas reversíveis que conferem o
//! estado esperado antes de escrever.
use super::*;

pub(super) fn is_default_format(format: &ReferenceFormat) -> bool {
    *format == ReferenceFormat::default()
}
fn frame_error(reason: &'static str) -> DomainError {
    DomainError::InvalidRelation(reason)
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameEdit {
    /// Insere ou substitui (mesmo ID) uma moldura.
    UpsertFrame(FrameTemplate),
    /// Recusado enquanto o projeto ou alguma página usar a moldura.
    RemoveFrame(FrameTemplateId),
    SetDefaultFrame(Option<FrameTemplateId>),
    SetPageFrame {
        page: PageId,
        frame: Option<FrameTemplateId>,
    },
    SetReferenceFormat(ReferenceFormat),
    /// `None` remove a chave.
    SetProjectMetadata {
        key: Text,
        value: Option<String>,
    },
    SetPageMetadata {
        page: PageId,
        key: Text,
        value: Option<String>,
    },
    /// Etapa 14: imagem do projeto (logotipo).
    AddAsset(ImageAsset),
    /// Recusado enquanto alguma moldura usar a imagem.
    RemoveAsset(AssetId),
    /// Várias edições numa transação; falha desfaz as já aplicadas.
    Batch(Vec<FrameEdit>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameDelta {
    Frame {
        id: FrameTemplateId,
        before: Option<Box<FrameTemplate>>,
        after: Option<Box<FrameTemplate>>,
    },
    DefaultFrame {
        before: Option<FrameTemplateId>,
        after: Option<FrameTemplateId>,
    },
    PageFrame {
        page: PageId,
        before: Option<FrameTemplateId>,
        after: Option<FrameTemplateId>,
    },
    ReferenceFormat {
        before: ReferenceFormat,
        after: ReferenceFormat,
    },
    ProjectMetadata {
        key: Text,
        before: Option<String>,
        after: Option<String>,
    },
    PageMetadata {
        page: PageId,
        key: Text,
        before: Option<String>,
        after: Option<String>,
    },
    Asset {
        id: AssetId,
        before: Option<Box<ImageAsset>>,
        after: Option<Box<ImageAsset>>,
    },
    Batch(Vec<FrameDelta>),
}
impl FrameDelta {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Frame { before, after, .. } => before == after,
            Self::DefaultFrame { before, after } | Self::PageFrame { before, after, .. } => {
                before == after
            }
            Self::ReferenceFormat { before, after } => before == after,
            Self::ProjectMetadata { before, after, .. }
            | Self::PageMetadata { before, after, .. } => before == after,
            Self::Asset { before, after, .. } => before == after,
            Self::Batch(items) => items.iter().all(FrameDelta::is_empty),
        }
    }
}

impl Project {
    pub fn frame(&self, id: FrameTemplateId) -> Option<&FrameTemplate> {
        self.frames.get(&id)
    }
    pub fn frames(&self) -> impl Iterator<Item = &FrameTemplate> {
        self.frames.values()
    }
    pub fn default_frame(&self) -> Option<FrameTemplateId> {
        self.default_frame
    }
    pub fn reference_format(&self) -> ReferenceFormat {
        self.reference_format
    }
    /// Moldura efetiva: a da página ou, sem ela, a padrão do projeto.
    pub fn page_frame(&self, page: PageId) -> Option<&FrameTemplate> {
        let page = self.page(page)?;
        page.frame()
            .or(self.default_frame)
            .and_then(|id| self.frame(id))
    }
    pub fn zone_at(&self, page: PageId, point: Point2D) -> Option<Zone> {
        self.page_frame(page)?.zone_at(point)
    }
    /// Referência cruzada legível. Com moldura e zona: `/página.coluna`
    /// (ou `/página.colunaLinha`); caso contrário, página e coordenadas.
    pub fn reference_label(&self, page: PageId, point: Point2D) -> String {
        let number = self
            .page(page)
            .map(|p| p.number().as_str().to_owned())
            .unwrap_or_else(|| "?".into());
        if let Some(zone) = self.zone_at(page, point) {
            let column = zone.column.unwrap_or_default();
            let row = zone.row.unwrap_or_default();
            let zone_text = match self.reference_format {
                ReferenceFormat::PageColumn if !column.is_empty() => column,
                ReferenceFormat::PageColumn => row,
                ReferenceFormat::PageColumnRow => format!("{column}{row}"),
            };
            if !zone_text.is_empty() {
                return format!("/{number}.{zone_text}");
            }
        }
        format!(
            "Página {number} · X {:.3} · Y {:.3} mm",
            point.x(),
            point.y()
        )
    }
    /// Valor de um texto especial para a página (texto vazio quando ausente).
    pub fn resolve_special(
        &self,
        page: PageId,
        field: &SpecialField,
        index: Option<u32>,
    ) -> String {
        let Some(p) = self.page(page) else {
            return String::new();
        };
        let ordered = self.ordered_pages(p.document_id()).unwrap_or_default();
        let position = ordered.iter().position(|o| o.id() == page);
        let number_at = |offset: isize| {
            position
                .and_then(|i| i.checked_add_signed(offset))
                .and_then(|i| ordered.get(i))
                .map(|o| o.number().as_str().to_owned())
                .unwrap_or_default()
        };
        match field {
            SpecialField::ProjectName => self.name.as_str().to_owned(),
            SpecialField::ProjectDescription => self.description.clone(),
            SpecialField::ProjectIdentification => self
                .identification
                .as_ref()
                .map(|t| t.as_str().to_owned())
                .unwrap_or_default(),
            SpecialField::ProjectProperty(key) => lookup(&self.metadata, key, index),
            SpecialField::DocumentName => self
                .document(p.document_id())
                .map(|d| d.name().as_str().to_owned())
                .unwrap_or_default(),
            SpecialField::PageNumber => p.number().as_str().to_owned(),
            SpecialField::PageName => p.name().as_str().to_owned(),
            SpecialField::PageFunctionalPath => path(p.structure().functional_path(), '='),
            SpecialField::PageLocationPath => path(p.structure().location_path(), '+'),
            SpecialField::PagePosition => position.map(|i| (i + 1).to_string()).unwrap_or_default(),
            SpecialField::PageCountInDocument => ordered.len().to_string(),
            SpecialField::PageCountInProject => self.pages.len().to_string(),
            SpecialField::PreviousPageNumber => number_at(-1),
            SpecialField::NextPageNumber => number_at(1),
            SpecialField::PageProperty(key) => lookup(p.metadata(), key, index),
            SpecialField::FrameName => self
                .page_frame(page)
                .map(|f| f.name().as_str().to_owned())
                .unwrap_or_default(),
        }
    }

    fn frame_in_use(&self, id: FrameTemplateId) -> bool {
        self.default_frame == Some(id) || self.pages.values().any(|p| p.frame() == Some(id))
    }
    pub(super) fn require_frame(&self, id: Option<FrameTemplateId>) -> Result<()> {
        match id {
            Some(id) if !self.frames.contains_key(&id) => Err(frame_error("moldura inexistente")),
            _ => Ok(()),
        }
    }
    pub(super) fn validate_frames(&self) -> Result<()> {
        for (id, frame) in &self.frames {
            if *id != frame.id() {
                return Err(frame_error("moldura com ID inconsistente"));
            }
            frame.validate()?;
            self.require_assets(frame)?;
        }
        for (id, asset) in &self.assets {
            if *id != asset.id() {
                return Err(frame_error("imagem com ID inconsistente"));
            }
        }
        self.require_frame(self.default_frame)?;
        for page in self.pages.values() {
            self.require_frame(page.frame())?;
        }
        Ok(())
    }

    fn require_assets(&self, frame: &FrameTemplate) -> Result<()> {
        if frame.assets().any(|a| !self.assets.contains_key(&a)) {
            return Err(frame_error("imagem da moldura inexistente no projeto"));
        }
        Ok(())
    }
    fn asset_in_use(&self, id: AssetId) -> bool {
        self.frames.values().any(|f| f.assets().any(|a| a == id))
    }
    pub fn asset(&self, id: AssetId) -> Option<&ImageAsset> {
        self.assets.get(&id)
    }
    pub fn assets(&self) -> impl Iterator<Item = &ImageAsset> {
        self.assets.values()
    }
    /// Liga o conteúdo lido do `.cadcraft` aos metadados da imagem.
    pub fn attach_asset_bytes(&mut self, id: AssetId, bytes: Vec<u8>) -> Result<()> {
        self.assets
            .get_mut(&id)
            .ok_or(frame_error("imagem inexistente no projeto"))?
            .attach(bytes)
    }

    pub fn edit_frame(&mut self, edit: FrameEdit) -> Result<FrameDelta> {
        let delta = match edit {
            FrameEdit::Batch(edits) => {
                let mut done: Vec<FrameDelta> = Vec::with_capacity(edits.len());
                for edit in edits {
                    match self.edit_frame(edit) {
                        Ok(delta) => done.push(delta),
                        Err(error) => {
                            for delta in done.iter().rev() {
                                self.restore_frame_edit(delta, false)?;
                            }
                            return Err(error);
                        }
                    }
                }
                return Ok(FrameDelta::Batch(done));
            }
            FrameEdit::AddAsset(asset) => {
                if self.assets.contains_key(&asset.id()) {
                    return Err(frame_error("imagem duplicada"));
                }
                FrameDelta::Asset {
                    id: asset.id(),
                    before: None,
                    after: Some(Box::new(asset)),
                }
            }
            FrameEdit::RemoveAsset(id) => {
                if self.asset_in_use(id) {
                    return Err(frame_error("imagem em uso por moldura"));
                }
                FrameDelta::Asset {
                    id,
                    before: Some(Box::new(
                        self.assets
                            .get(&id)
                            .cloned()
                            .ok_or(frame_error("imagem inexistente no projeto"))?,
                    )),
                    after: None,
                }
            }
            FrameEdit::UpsertFrame(frame) => {
                frame.validate()?;
                self.require_assets(&frame)?;
                FrameDelta::Frame {
                    id: frame.id(),
                    before: self.frames.get(&frame.id()).cloned().map(Box::new),
                    after: Some(Box::new(frame)),
                }
            }
            FrameEdit::RemoveFrame(id) => {
                if !self.frames.contains_key(&id) {
                    return Err(frame_error("moldura inexistente"));
                }
                if self.frame_in_use(id) {
                    return Err(frame_error("moldura em uso pelo projeto ou por páginas"));
                }
                FrameDelta::Frame {
                    id,
                    before: self.frames.get(&id).cloned().map(Box::new),
                    after: None,
                }
            }
            FrameEdit::SetDefaultFrame(frame) => {
                self.require_frame(frame)?;
                FrameDelta::DefaultFrame {
                    before: self.default_frame,
                    after: frame,
                }
            }
            FrameEdit::SetPageFrame { page, frame } => {
                self.require_frame(frame)?;
                let before = self
                    .page(page)
                    .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?
                    .frame();
                FrameDelta::PageFrame {
                    page,
                    before,
                    after: frame,
                }
            }
            FrameEdit::SetReferenceFormat(format) => FrameDelta::ReferenceFormat {
                before: self.reference_format,
                after: format,
            },
            FrameEdit::SetProjectMetadata { key, value } => FrameDelta::ProjectMetadata {
                before: self.metadata.get(&key).cloned(),
                after: normalize(value),
                key,
            },
            FrameEdit::SetPageMetadata { page, key, value } => FrameDelta::PageMetadata {
                before: self
                    .page(page)
                    .ok_or(DomainError::MissingEntity(EntityRef::Page(page)))?
                    .metadata()
                    .get(&key)
                    .cloned(),
                after: normalize(value),
                page,
                key,
            },
        };
        if !delta.is_empty() {
            self.restore_frame_edit(&delta, true)?;
        }
        Ok(delta)
    }

    pub fn restore_frame_edit(&mut self, delta: &FrameDelta, forward: bool) -> Result<()> {
        let mismatch = || frame_error("histórico de moldura não corresponde ao documento atual");
        match delta {
            FrameDelta::Frame { id, before, after } => {
                let (expected, target) = pick(forward, before, after);
                if self.frames.get(id) != expected.as_deref() {
                    return Err(mismatch());
                }
                match target {
                    Some(frame) => {
                        self.require_assets(frame)?;
                        self.frames.insert(*id, (**frame).clone());
                    }
                    None => {
                        if self.frame_in_use(*id) {
                            return Err(frame_error("moldura em uso pelo projeto ou por páginas"));
                        }
                        self.frames.remove(id);
                    }
                }
            }
            FrameDelta::DefaultFrame { before, after } => {
                let (expected, target) = pick(forward, before, after);
                if self.default_frame != *expected {
                    return Err(mismatch());
                }
                self.require_frame(*target)?;
                self.default_frame = *target;
            }
            FrameDelta::PageFrame {
                page,
                before,
                after,
            } => {
                let (expected, target) = pick(forward, before, after);
                self.require_frame(*target)?;
                let p = self.page_mut(*page)?;
                if p.frame != *expected {
                    return Err(mismatch());
                }
                p.frame = *target;
            }
            FrameDelta::ReferenceFormat { before, after } => {
                let (expected, target) = pick(forward, before, after);
                if self.reference_format != *expected {
                    return Err(mismatch());
                }
                self.reference_format = *target;
            }
            FrameDelta::ProjectMetadata { key, before, after } => {
                let (expected, target) = pick(forward, before, after);
                if self.metadata.get(key) != expected.as_ref() {
                    return Err(mismatch());
                }
                set(&mut self.metadata, key, target.clone());
            }
            FrameDelta::PageMetadata {
                page,
                key,
                before,
                after,
            } => {
                let (expected, target) = pick(forward, before, after);
                let p = self.page_mut(*page)?;
                if p.metadata.get(key) != expected.as_ref() {
                    return Err(mismatch());
                }
                set(&mut p.metadata, key, target.clone());
            }
            FrameDelta::Asset { id, before, after } => {
                let (expected, target) = pick(forward, before, after);
                if self.assets.get(id) != expected.as_deref() {
                    return Err(mismatch());
                }
                match target {
                    Some(asset) => {
                        self.assets.insert(*id, (**asset).clone());
                    }
                    None => {
                        if self.asset_in_use(*id) {
                            return Err(frame_error("imagem em uso por moldura"));
                        }
                        self.assets.remove(id);
                    }
                }
            }
            FrameDelta::Batch(items) => {
                let ordered: Vec<&FrameDelta> = if forward {
                    items.iter().collect()
                } else {
                    items.iter().rev().collect()
                };
                for (done, item) in ordered.iter().enumerate() {
                    if let Err(error) = self.restore_frame_edit(item, forward) {
                        for undo in ordered[..done].iter().rev() {
                            self.restore_frame_edit(undo, !forward)?;
                        }
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }
}

fn normalize(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}
fn set(metadata: &mut Metadata, key: &Text, value: Option<String>) {
    match value {
        Some(v) => {
            metadata.insert(key.clone(), v);
        }
        None => {
            metadata.remove(key);
        }
    }
}
fn lookup(metadata: &Metadata, key: &Text, index: Option<u32>) -> String {
    let wanted = indexed_key(key, index);
    metadata
        .iter()
        .find(|(k, _)| k.as_str() == wanted)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}
fn path(parts: &[DesignationPart], prefix: char) -> String {
    parts
        .iter()
        .map(|p| format!("{prefix}{}", p.as_str()))
        .collect()
}
fn pick<'a, T>(forward: bool, before: &'a T, after: &'a T) -> (&'a T, &'a T) {
    if forward {
        (before, after)
    } else {
        (after, before)
    }
}

/// Etapa 14: moldura exportável com as imagens que usa (`.cadframe`).
#[derive(Clone, Debug, PartialEq)]
pub struct FramePackage {
    pub frame: FrameTemplate,
    pub assets: Vec<ImageAsset>,
}

impl Project {
    /// Pacote da moldura com suas imagens (as que tiverem conteúdo).
    pub fn frame_package(&self, id: FrameTemplateId) -> Result<FramePackage> {
        let frame = self
            .frame(id)
            .ok_or(frame_error("moldura inexistente"))?
            .clone();
        let mut ids: Vec<AssetId> = frame.assets().collect();
        ids.sort();
        ids.dedup();
        let mut assets = Vec::with_capacity(ids.len());
        for id in ids {
            let asset = self
                .asset(id)
                .ok_or(frame_error("imagem da moldura inexistente no projeto"))?;
            if asset.bytes().is_none() {
                return Err(frame_error(
                    "imagem sem conteúdo (projeto lido de snapshot)",
                ));
            }
            assets.push(asset.clone());
        }
        Ok(FramePackage { frame, assets })
    }
    /// Edições que importam o pacote: imagens iguais são reaproveitadas,
    /// IDs já usados são trocados. Retorna também o ID da moldura importada.
    pub fn frame_import_edits(
        &self,
        package: &FramePackage,
    ) -> Result<(Vec<FrameEdit>, FrameTemplateId)> {
        let mut map = BTreeMap::new();
        let mut edits = Vec::new();
        for asset in &package.assets {
            let bytes = asset
                .bytes()
                .ok_or(frame_error("imagem do pacote sem conteúdo"))?;
            if let Some(existing) = self.assets().find(|a| a.same_content(bytes)) {
                map.insert(asset.id(), existing.id());
                continue;
            }
            let fresh = if self.assets.contains_key(&asset.id()) {
                asset.clone().with_id(AssetId::new())
            } else {
                asset.clone()
            };
            map.insert(asset.id(), fresh.id());
            edits.push(FrameEdit::AddAsset(fresh));
        }
        let id = if self.frames.contains_key(&package.frame.id()) {
            FrameTemplateId::new()
        } else {
            package.frame.id()
        };
        let frame = package.frame.with_assets_remapped(&map, id);
        frame.validate()?;
        edits.push(FrameEdit::UpsertFrame(frame));
        Ok((edits, id))
    }
}
