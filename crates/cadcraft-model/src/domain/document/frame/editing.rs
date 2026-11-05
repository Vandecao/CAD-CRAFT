// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frame element transforms and template drafts; start with FrameTemplate::edited.
use super::*;

fn moved(p: Point2D, dx: f64, dy: f64) -> Result<Point2D> {
    p.translated(dx, dy)
}
impl FrameElement {
    /// Mesmo elemento deslocado (mm).
    pub fn translated(&self, dx: f64, dy: f64) -> Result<Self> {
        Ok(match self.clone() {
            Self::Line { start, end, width } => Self::Line {
                start: moved(start, dx, dy)?,
                end: moved(end, dx, dy)?,
                width,
            },
            Self::Rectangle { rect, width } => Self::Rectangle {
                rect: Rect2D::new(moved(rect.origin(), dx, dy)?, rect.size())?,
                width,
            },
            Self::Circle {
                center,
                radius,
                width,
            } => Self::Circle {
                center: moved(center, dx, dy)?,
                radius,
                width,
            },
            Self::Arc {
                center,
                radius,
                start_degrees,
                sweep_degrees,
                width,
            } => Self::Arc {
                center: moved(center, dx, dy)?,
                radius,
                start_degrees,
                sweep_degrees,
                width,
            },
            Self::Text {
                origin,
                text,
                style,
            } => Self::Text {
                origin: moved(origin, dx, dy)?,
                text,
                style,
            },
            Self::Special {
                origin,
                field,
                index,
                style,
            } => Self::Special {
                origin: moved(origin, dx, dy)?,
                field,
                index,
                style,
            },
            Self::Image { rect, asset } => Self::Image {
                rect: Rect2D::new(moved(rect.origin(), dx, dy)?, rect.size())?,
                asset,
            },
        })
    }
    /// Ponto de referência usado para mover, alinhar e selecionar.
    pub fn anchor(&self) -> Point2D {
        match self {
            Self::Line { start, .. } => *start,
            Self::Rectangle { rect, .. } | Self::Image { rect, .. } => rect.origin(),
            Self::Circle { center, .. } | Self::Arc { center, .. } => *center,
            Self::Text { origin, .. } | Self::Special { origin, .. } => *origin,
        }
    }
    pub fn asset(&self) -> Option<AssetId> {
        match self {
            Self::Image { asset, .. } => Some(*asset),
            _ => None,
        }
    }
    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Line { .. } => "Linha",
            Self::Rectangle { .. } => "Retângulo",
            Self::Circle { .. } => "Círculo",
            Self::Arc { .. } => "Arco",
            Self::Text { .. } => "Texto",
            Self::Special { .. } => "Texto especial",
            Self::Image { .. } => "Imagem",
        }
    }
}

/// Conteúdo editável de uma moldura; `FrameTemplate::edited` valida e gera a
/// versão seguinte com o mesmo ID.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameDraft {
    pub name: String,
    pub description: String,
    pub sheet: Size2D,
    pub usable: Rect2D,
    pub zones: ZoneLayout,
    pub elements: Vec<FrameElement>,
}
impl FrameTemplate {
    pub fn draft(&self) -> FrameDraft {
        FrameDraft {
            name: self.name.as_str().to_owned(),
            description: self.description.clone(),
            sheet: self.sheet,
            usable: self.usable,
            zones: self.zones.clone(),
            elements: self.elements.clone(),
        }
    }
    pub fn edited(&self, draft: FrameDraft) -> Result<Self> {
        let frame = Self {
            id: self.id,
            name: Text::new(draft.name)?,
            version: self.version.saturating_add(1),
            description: draft.description,
            sheet: draft.sheet,
            usable: draft.usable,
            zones: draft.zones,
            elements: draft.elements,
        };
        frame.validate()?;
        Ok(frame)
    }
    /// Cópia independente: novo ID e versão 1.
    pub fn duplicate(&self, name: impl Into<String>) -> Result<Self> {
        let frame = Self {
            id: FrameTemplateId::new(),
            name: Text::new(name)?,
            version: 1,
            ..self.clone()
        };
        frame.validate()?;
        Ok(frame)
    }
    /// Moldura em branco: borda a 5 mm e área útil igual à borda, sem zonas.
    pub fn blank(name: impl Into<String>, sheet: Size2D) -> Result<Self> {
        let (w, h) = (sheet.width(), sheet.height());
        let inset = 5.0_f64.min(w / 4.0).min(h / 4.0);
        let usable = Rect2D::new(
            Point2D::new(inset, inset)?,
            Size2D::new(w - 2.0 * inset, h - 2.0 * inset)?,
        )?;
        Self::new(
            name,
            sheet,
            usable,
            ZoneLayout {
                columns: ZoneAxis::none(),
                rows: ZoneAxis::none(),
            },
            vec![FrameElement::Rectangle {
                rect: usable,
                width: 0.5,
            }],
        )
    }
    pub fn assets(&self) -> impl Iterator<Item = AssetId> + '_ {
        self.elements.iter().filter_map(FrameElement::asset)
    }
    /// Mesmo conteúdo com IDs de imagem trocados (importação).
    pub fn with_assets_remapped(
        &self,
        map: &BTreeMap<AssetId, AssetId>,
        id: FrameTemplateId,
    ) -> Self {
        let mut frame = self.clone();
        frame.id = id;
        for e in &mut frame.elements {
            if let FrameElement::Image { asset, .. } = e
                && let Some(new) = map.get(asset)
            {
                *asset = *new;
            }
        }
        frame
    }
}
