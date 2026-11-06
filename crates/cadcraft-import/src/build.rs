// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Shared imported-frame assembly: elements, images, sheet and usable area; start at `Builder`.
use super::{ImportReport, ImportResult, ImportedFrame, fail};
use cadcraft_model::domain::{
    FrameElement, FramePackage, FrameTemplate, FrameTextStyle, ImageAsset, LocalizedText,
    PaperFormat, Point2D, Rect2D, Size2D, SpecialField, ZoneAxis, ZoneLayout,
};

const EPS: f64 = 1e-9;

pub(crate) fn pt(x: f64, y: f64) -> Option<Point2D> {
    Point2D::new(x, y).ok()
}

/// Arc defined by two points and a bulge (tangent of one quarter of the angle;
/// positive means counterclockwise), as in DXF and EPLAN polylines.
pub(crate) fn bulge_arc(
    a: (f64, f64),
    b: (f64, f64),
    bulge: f64,
) -> Option<(Point2D, f64, f64, f64)> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let chord = dx.hypot(dy);
    if chord < EPS || bulge.abs() < EPS || !bulge.is_finite() {
        return None;
    }
    let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let (nx, ny) = (-dy / chord, dx / chord);
    let offset = chord / 2.0 * (1.0 - bulge * bulge) / (2.0 * bulge);
    let (cx, cy) = (mx + nx * offset, my + ny * offset);
    let radius = (a.0 - cx).hypot(a.1 - cy);
    let start = (a.1 - cy).atan2(a.0 - cx).to_degrees();
    let sweep = (4.0 * bulge.atan()).to_degrees();
    Some((pt(cx, cy)?, radius, start, sweep))
}

#[derive(Default)]
pub(crate) struct Builder {
    pub elements: Vec<FrameElement>,
    pub assets: Vec<ImageAsset>,
    pub report: ImportReport,
}
impl Builder {
    fn push(&mut self, element: Option<FrameElement>, what: &str) {
        match element {
            Some(e) => self.elements.push(e),
            None => self
                .report
                .ignored(format!("{what} com geometria inválida ou degenerada")),
        }
    }
    pub fn line(&mut self, a: (f64, f64), b: (f64, f64), width: f64) {
        let e = (|| {
            let (start, end) = (pt(a.0, a.1)?, pt(b.0, b.1)?);
            ((a.0 - b.0).hypot(a.1 - b.1) > EPS).then_some(FrameElement::Line { start, end, width })
        })();
        self.push(e, "linha");
    }
    pub fn rect(&mut self, a: (f64, f64), b: (f64, f64), width: f64) {
        let e = (|| {
            let rect = rect_from(a, b)?;
            Some(FrameElement::Rectangle { rect, width })
        })();
        self.push(e, "retângulo");
    }
    pub fn circle(&mut self, c: (f64, f64), radius: f64, width: f64) {
        let e = (radius.is_finite() && radius > EPS)
            .then(|| pt(c.0, c.1))
            .flatten()
            .map(|center| FrameElement::Circle {
                center,
                radius,
                width,
            });
        self.push(e, "círculo");
    }
    pub fn arc(&mut self, c: (f64, f64), radius: f64, start: f64, sweep: f64, width: f64) {
        let ok = radius.is_finite() && radius > EPS && start.is_finite() && sweep.is_finite();
        let sweep = sweep.clamp(-360.0, 360.0);
        let e = (ok && sweep.abs() > EPS)
            .then(|| pt(c.0, c.1))
            .flatten()
            .map(|center| FrameElement::Arc {
                center,
                radius,
                start_degrees: start,
                sweep_degrees: sweep,
                width,
            });
        self.push(e, "arco");
    }
    /// Polyline segment: straight line or arc according to its bulge.
    pub fn segment(&mut self, a: (f64, f64), b: (f64, f64), bulge: f64, width: f64) {
        match bulge_arc(a, b, bulge) {
            Some((c, r, s, w)) => self.arc((c.x(), c.y()), r, s, w, width),
            None => self.line(a, b, width),
        }
    }
    pub fn text(&mut self, at: (f64, f64), text: LocalizedText, style: FrameTextStyle) {
        if text.default.trim().is_empty() {
            self.report.ignored("texto vazio");
            return;
        }
        let e = pt(at.0, at.1).map(|origin| FrameElement::Text {
            origin,
            text,
            style,
        });
        self.push(e, "texto");
    }
    pub fn special(
        &mut self,
        at: (f64, f64),
        field: SpecialField,
        index: Option<u32>,
        style: FrameTextStyle,
    ) {
        let e = pt(at.0, at.1).map(|origin| FrameElement::Special {
            origin,
            field,
            index: index.filter(|i| *i > 0),
            style,
        });
        self.push(e, "texto especial");
    }
    /// Builds an image from PNG/JPEG bytes, reusing identical images.
    pub fn image(&mut self, a: (f64, f64), b: (f64, f64), bytes: Vec<u8>, label: &str) {
        let Some(rect) = rect_from(a, b) else {
            self.report
                .ignored(format!("imagem '{label}' com retângulo inválido"));
            return;
        };
        let asset = match self.assets.iter().find(|x| x.same_content(&bytes)) {
            Some(existing) => existing.id(),
            None => match ImageAsset::from_bytes(bytes) {
                Ok(asset) => {
                    let id = asset.id();
                    self.assets.push(asset);
                    id
                }
                Err(e) => {
                    self.report
                        .ignored(format!("imagem '{label}' recusada: {e}"));
                    return;
                }
            },
        };
        self.elements.push(FrameElement::Image { rect, asset });
    }

    /// Bounding box (minimum, maximum) of the elements.
    fn bounds(&self) -> Option<((f64, f64), (f64, f64))> {
        let mut b: Option<((f64, f64), (f64, f64))> = None;
        let mut add = |x: f64, y: f64| {
            let ((x0, y0), (x1, y1)) = b.unwrap_or(((x, y), (x, y)));
            b = Some(((x0.min(x), y0.min(y)), (x1.max(x), y1.max(y))));
        };
        for e in &self.elements {
            match e {
                FrameElement::Line { start, end, .. } => {
                    add(start.x(), start.y());
                    add(end.x(), end.y());
                }
                FrameElement::Rectangle { rect, .. } | FrameElement::Image { rect, .. } => {
                    add(rect.origin().x(), rect.origin().y());
                    add(rect.max().x(), rect.max().y());
                }
                FrameElement::Circle { center, radius, .. }
                | FrameElement::Arc { center, radius, .. } => {
                    add(center.x() - radius, center.y() - radius);
                    add(center.x() + radius, center.y() + radius);
                }
                FrameElement::Text { origin, .. } | FrameElement::Special { origin, .. } => {
                    add(origin.x(), origin.y())
                }
            }
        }
        b
    }

    /// Finishes the frame. Without an explicit sheet, use the drawing bounds,
    /// shifted to the origin when `to_origin` is set or coordinates are
    /// negative. An omitted usable area defaults to the whole sheet.
    pub fn finish(
        mut self,
        name: &str,
        description: &str,
        sheet: Option<(f64, f64)>,
        usable: Option<(f64, f64, f64, f64)>,
        mut zones: ZoneLayout,
        to_origin: bool,
    ) -> ImportResult<ImportedFrame> {
        if self.elements.is_empty() {
            return Err(fail("nenhum elemento de moldura reconhecido no arquivo"));
        }
        let bounds = self.bounds().expect("elementos existem");
        let mut usable = usable;
        let sheet = match sheet.filter(|(w, h)| *w > EPS && *h > EPS) {
            Some(s) => {
                let (min, max) = bounds;
                if min.0 < -0.5 || min.1 < -0.5 || max.0 > s.0 + 0.5 || max.1 > s.1 + 0.5 {
                    self.report
                        .info("há elementos fora da folha (mantidos na posição original)");
                }
                s
            }
            None => {
                let (min, _) = bounds;
                let shift = if to_origin || min.0 < 0.0 || min.1 < 0.0 {
                    (-min.0, -min.1)
                } else {
                    (0.0, 0.0)
                };
                if shift.0.abs() > EPS || shift.1.abs() > EPS {
                    self.elements = self
                        .elements
                        .iter()
                        .map(|e| e.translated(shift.0, shift.1))
                        .collect::<cadcraft_model::error::Result<_>>()
                        .map_err(|e| fail(format!("deslocamento inválido: {e}")))?;
                    usable = usable.map(|(x, y, w, h)| (x + shift.0, y + shift.1, w, h));
                    self.report.info(format!(
                        "desenho deslocado ({:.3}; {:.3}) mm para a origem da folha",
                        shift.0, shift.1
                    ));
                }
                let (_, max) = self.bounds().expect("elementos existem");
                (round3(max.0.max(EPS)), round3(max.1.max(EPS)))
            }
        };
        let sheet =
            Size2D::new(sheet.0, sheet.1).map_err(|e| fail(format!("folha inválida: {e}")))?;
        let sheet = match PaperFormat::matching(sheet) {
            Some((f, landscape)) => {
                self.report.info(format!(
                    "folha {} {}",
                    f.label(),
                    if landscape { "paisagem" } else { "retrato" }
                ));
                f.size(landscape)
            }
            None => {
                self.report.info(format!(
                    "folha personalizada {:.1} × {:.1} mm",
                    sheet.width(),
                    sheet.height()
                ));
                sheet
            }
        };
        let full = (0.0, 0.0, sheet.width(), sheet.height());
        let (ux, uy, uw, uh) = usable.unwrap_or(full);
        let (x0, y0) = (ux.max(0.0), uy.max(0.0));
        let (x1, y1) = ((ux + uw).min(sheet.width()), (uy + uh).min(sheet.height()));
        let clipped = (x0 - ux).abs() > 0.01
            || (y0 - uy).abs() > 0.01
            || (x1 - (ux + uw)).abs() > 0.01
            || (y1 - (uy + uh)).abs() > 0.01;
        let (ux, uy, uw, uh) = if x1 - x0 > EPS && y1 - y0 > EPS {
            (round3(x0), round3(y0), round3(x1 - x0), round3(y1 - y0))
        } else {
            full
        };
        if clipped {
            self.report
                .approximated("área útil ajustada para caber na folha");
        }
        for (axis, size, what) in [
            (&mut zones.columns, uw, "colunas"),
            (&mut zones.rows, uh, "linhas"),
        ] {
            if axis.total() > size + 0.01 {
                self.report
                    .ignored(format!("divisão em {what} maior que a área útil"));
                *axis = ZoneAxis::none();
            } else if axis.total() > size {
                // Adjust the last zone to compensate for rounding.
                let excess = axis.total() - size;
                if let Some(last) = axis.sizes.last_mut() {
                    *last -= excess;
                }
            }
        }
        let usable = Rect2D::new(
            Point2D::new(ux, uy).map_err(|e| fail(e.to_string()))?,
            Size2D::new(uw, uh).map_err(|e| fail(e.to_string()))?,
        )
        .map_err(|e| fail(e.to_string()))?;
        let name = if name.trim().is_empty() {
            "Moldura importada"
        } else {
            name.trim()
        };
        let frame = FrameTemplate::new(name, sheet, usable, zones, self.elements)
            .map_err(|e| fail(format!("moldura resultante inválida: {e}")))?
            .with_description(description.trim());
        Ok(ImportedFrame {
            package: FramePackage {
                frame,
                assets: self.assets,
            },
            report: self.report,
        })
    }
}

pub(crate) fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

pub(crate) fn rect_from(a: (f64, f64), b: (f64, f64)) -> Option<Rect2D> {
    let (x0, x1) = (a.0.min(b.0), a.0.max(b.0));
    let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
    if x1 - x0 < EPS || y1 - y0 < EPS {
        return None;
    }
    Rect2D::new(pt(x0, y0)?, Size2D::new(x1 - x0, y1 - y0).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulge_gives_quarter_and_half_circles() {
        // Bulge 1 is a counterclockwise semicircle from (0,0) to (2,0), centered at (1,0).
        let (c, r, start, sweep) = bulge_arc((0.0, 0.0), (2.0, 0.0), 1.0).unwrap();
        assert!((c.x() - 1.0).abs() < 1e-9 && c.y().abs() < 1e-9);
        assert!((r - 1.0).abs() < 1e-9);
        assert!((start.abs() - 180.0).abs() < 1e-9);
        assert!((sweep - 180.0).abs() < 1e-9);
        // tan(22.5°) gives a quarter circle; a negative value means clockwise.
        let b = (std::f64::consts::PI / 8.0).tan();
        let (c, r, _, sweep) = bulge_arc((1.0, 0.0), (0.0, 1.0), b).unwrap();
        assert!(c.x().abs() < 1e-9 && c.y().abs() < 1e-9 && (r - 1.0).abs() < 1e-9);
        assert!((sweep - 90.0).abs() < 1e-9);
        let (_, _, _, sweep) = bulge_arc((0.0, 1.0), (1.0, 0.0), -b).unwrap();
        assert!((sweep + 90.0).abs() < 1e-9);
        assert!(bulge_arc((0.0, 0.0), (1.0, 0.0), 0.0).is_none());
    }
}
