// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Converts embedded symbols and page frames; start with `Importer::collection`.
use super::*;

impl Importer<'_> {
    pub(in crate::qet) fn collection(&mut self, e: &Element, path: &str) {
        for child in &e.children {
            let name = child.attr("name").unwrap_or("").trim();
            let key = format!("{path}/{name}");
            match child.name.as_str() {
                "category" => self.collection(child, &key),
                "element" => {
                    let Some(def) = child.child("definition") else {
                        self.report.ignored("elemento embutido sem definição");
                        continue;
                    };
                    let options = ElmtOptions {
                        mm_per_px: self.s,
                        category: "QElectroTech (importado)".into(),
                        id_hint: safe_id(&key.replace("embed://", "")),
                    };
                    match elmt::import_definition(def, &options) {
                        Ok(imported) => {
                            for (note, count) in imported.report.notes {
                                // Per-symbol counts would only clutter the report.
                                if note.kind == crate::NoteKind::Info {
                                    continue;
                                }
                                for _ in 0..count {
                                    self.report.note(note.kind, note.message.clone());
                                }
                            }
                            let kind = def
                                .child("kindInformations")
                                .into_iter()
                                .flat_map(|k| k.children_named("kindInformation"))
                                .find(|k| k.attr("name") == Some("type"))
                                .map(|k| k.text.trim().to_owned())
                                .unwrap_or_default();
                            let terminals = def
                                .child("description")
                                .into_iter()
                                .flat_map(|d| d.children_named("terminal"))
                                .filter_map(|t| {
                                    Some((trim_uuid(t.attr("uuid")?), (num(t, "x")?, num(t, "y")?)))
                                })
                                .collect();
                            self.definitions.insert(
                                key,
                                Definition {
                                    symbol: imported.definition,
                                    link: def.attr("link_type").unwrap_or("simple").to_owned(),
                                    kind,
                                    terminals,
                                },
                            );
                        }
                        Err(e) => self
                            .report
                            .ignored(format!("elemento embutido {name}: {e}")),
                    }
                }
                _ => {}
            }
        }
    }

    pub(in crate::qet) fn sheet_of(&self, d: &Element, defaults: Option<&Element>) -> Sheet {
        let pick = |k: &str, fallback: f64| {
            num(d, k)
                .or_else(|| defaults.and_then(|b| num(b, k)))
                .unwrap_or(fallback)
        };
        let flag = |k: &str| {
            d.attr(k)
                .or_else(|| defaults.and_then(|b| b.attr(k)))
                .is_none_or(|v| v != "false")
        };
        let titleblock_px = d
            .attr("titleblocktemplate")
            .and_then(|n| self.templates.get(n))
            .and_then(|t| t.child("grid"))
            .and_then(|g| g.attr("rows"))
            .map(|rows| {
                rows.split(';')
                    .filter_map(|r| {
                        let digits: String = r
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        digits.parse::<f64>().ok()
                    })
                    .sum::<f64>()
            })
            .filter(|h| *h > 0.0)
            .unwrap_or(DEFAULT_TITLEBLOCK_PX);
        Sheet {
            cols: pick("cols", 17.0).clamp(1.0, 200.0) as usize,
            rows: pick("rows", 8.0).clamp(1.0, 200.0) as usize,
            colsize: pick("colsize", 60.0).clamp(5.0, 2000.0),
            rowsize: pick("rowsize", 80.0).clamp(5.0, 2000.0),
            show_cols: flag("displaycols"),
            show_rows: flag("displayrows"),
            titleblock_px,
        }
    }

    /// Frame with QElectroTech's zone grid and embedded titleblock.
    pub(in crate::qet) fn frame_for(
        &mut self,
        sheet: Sheet,
        template: Option<&str>,
    ) -> ImportResult<FramePackage> {
        let s = self.s;
        let mut assets = vec![];
        let (w, h) = (sheet.width_px() * s, sheet.height_px() * s);
        let size = Size2D::new(w, h).map_err(domain)?;
        let mut elements = vec![];
        let tpl = template.and_then(|n| self.templates.get(n).copied());
        match tpl.map(|t| {
            titleblock::import_template(
                t,
                TitleblockOptions {
                    sheet: size,
                    margin: MARGIN_PX * s,
                    mm_per_px: s,
                },
            )
        }) {
            Some(Ok(imported)) => {
                elements.extend(imported.package.frame.elements().iter().cloned());
                assets = imported.package.assets;
            }
            Some(Err(e)) => self.report.approximated(format!("carimbo embutido: {e}")),
            None => {
                if let Some(name) = template.filter(|n| !n.is_empty()) {
                    self.report
                        .approximated(format!("carimbo '{name}' não embutido: carimbo simples"));
                }
                let tb = sheet.titleblock_px * s;
                let m = MARGIN_PX * s;
                let rect = Rect2D::new(
                    Point2D::new(m, m).map_err(domain)?,
                    Size2D::new(w - 2.0 * m, tb).map_err(domain)?,
                )
                .map_err(domain)?;
                elements.push(FrameElement::Rectangle { rect, width: 0.35 });
                let style = FrameTextStyle::new(3.0, TextAlign::MiddleLeft);
                for (i, field) in [SpecialField::ProjectName, SpecialField::PageName]
                    .into_iter()
                    .enumerate()
                {
                    elements.push(FrameElement::Special {
                        origin: Point2D::new(m + 3.0, m + tb * (0.75 - 0.5 * i as f64))
                            .map_err(domain)?,
                        field,
                        index: None,
                        style,
                    });
                }
                elements.push(FrameElement::Special {
                    origin: Point2D::new(w - m - 30.0, m + tb / 2.0).map_err(domain)?,
                    field: SpecialField::PageNumber,
                    index: None,
                    style,
                });
            }
        }
        // Drawing border and usable area below the headers.
        let m = MARGIN_PX * s;
        let diagram_h = (HEADER_PX + sheet.rows as f64 * sheet.rowsize) * s;
        let diagram = Rect2D::new(
            Point2D::new(m, h - m - diagram_h).map_err(domain)?,
            Size2D::new(w - 2.0 * m, diagram_h).map_err(domain)?,
        )
        .map_err(domain)?;
        elements.push(FrameElement::Rectangle {
            rect: diagram,
            width: 0.35,
        });
        let usable = Rect2D::new(
            Point2D::new((MARGIN_PX + HEADER_PX) * s, h - m - diagram_h).map_err(domain)?,
            Size2D::new(
                sheet.cols as f64 * sheet.colsize * s,
                sheet.rows as f64 * sheet.rowsize * s,
            )
            .map_err(domain)?,
        )
        .map_err(domain)?;
        elements.push(FrameElement::Rectangle {
            rect: usable,
            width: 0.18,
        });
        let mut columns = ZoneAxis::uniform(
            sheet.cols,
            sheet.colsize * s,
            ZoneLabelStyle::Numbers { start: 1 },
        );
        columns.labels_start = sheet.show_cols;
        let mut rows = ZoneAxis::uniform(
            sheet.rows,
            sheet.rowsize * s,
            ZoneLabelStyle::Letters { start: 0 },
        );
        rows.labels_start = sheet.show_rows;
        let name = format!(
            "QElectroTech {}×{} ({}×{} px){}",
            sheet.cols,
            sheet.rows,
            sheet.colsize,
            sheet.rowsize,
            template.map(|t| format!(" {t}")).unwrap_or_default()
        );
        let frame = FrameTemplate::new(
            name,
            size,
            usable,
            ZoneLayout {
                columns,
                rows,
                generated_marks: false,
            },
            elements,
        )
        .map_err(domain)?;
        Ok(FramePackage { frame, assets })
    }

    pub(in crate::qet) fn port_for(&self, inst: &Instance, terminal: &str) -> Option<String> {
        let def = self.definitions.get(&inst.def)?;
        let (x, y) = inst
            .terminals
            .iter()
            .find(|(id, _, _)| id == terminal)
            .map(|(_, x, y)| (*x, *y))
            .or_else(|| def.terminals.get(terminal).copied())?;
        let (lx, ly) = (x * self.s, -y * self.s);
        def.symbol
            .ports()
            .iter()
            .find(|p| {
                (p.position().x() - lx).abs() < PORT_TOLERANCE
                    && (p.position().y() - ly).abs() < PORT_TOLERANCE
            })
            .or_else(|| {
                let index = inst
                    .terminals
                    .iter()
                    .position(|(id, _, _)| id == terminal)?;
                def.symbol.ports().get(index)
            })
            .map(|p| p.id().to_owned())
    }
}
