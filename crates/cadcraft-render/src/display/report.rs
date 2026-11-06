// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Report table layout; start with Builder::report and its grid helper.

use super::*;

impl Builder<'_> {
    pub(super) fn report(&mut self, page: &Page, definition: &ReportDefinition) -> Result<()> {
        let mut table = self
            .project
            .report_table(definition.kind, &definition.filter)?;
        if definition.sort == ReportSort::Reverse {
            table.rows.reverse();
        }
        if !definition.columns.is_empty() {
            let selected: Vec<_> = table
                .columns
                .iter()
                .enumerate()
                .filter(|(_, c)| definition.columns.iter().any(|name| name == c.title))
                .map(|(i, _)| i)
                .collect();
            if !selected.is_empty() {
                table.columns = selected.iter().map(|i| table.columns[*i].clone()).collect();
                for row in &mut table.rows {
                    row.cells = selected.iter().map(|i| row.cells[*i].clone()).collect();
                }
            }
        }
        let area = self
            .project
            .page_frame(page.id())
            .map(FrameTemplate::usable)
            .unwrap_or_else(|| {
                Rect2D::new(
                    Point2D::new(10.0, 10.0).expect("finite"),
                    Size2D::new(page.size().width() - 20.0, page.size().height() - 20.0)
                        .expect("finite"),
                )
                .expect("finite")
            });
        let per_page = self
            .project
            .report_rows_per_page(page.size(), page.frame())?;
        let total_pages = table.rows.len().div_ceil(per_page).max(1);
        let page_index = definition.index as usize;
        let rows = table
            .rows
            .iter()
            .skip(page_index.saturating_mul(per_page))
            .take(per_page);
        let width = area.size().width();
        let sum: f64 = table.columns.iter().map(|c| c.width_mm).sum();
        if sum <= 0.0 {
            return Ok(());
        }
        let sizes: Vec<f64> = table
            .columns
            .iter()
            .map(|c| c.width_mm / sum * width)
            .collect();
        let top = area.max().y();
        let left = area.origin().x();
        self.layer = DisplayLayer::Texts;
        let mut x = left;
        for (column, w) in table.columns.iter().zip(&sizes) {
            let title = truncate_report_text(column.title, *w - 2.0, 2.5);
            self.text(
                Point2D::new(x + 1.0, top - 1.0)?,
                &title,
                FrameTextStyle::new(2.5, TextAlign::TopLeft),
                DisplayColor::Ink,
            );
            x += w;
        }
        self.report_grid(left, top, &sizes, 8.0);
        let mut y = top - 8.0;
        for row in rows {
            let mut x = left;
            for (cell, w) in row.cells.iter().zip(&sizes) {
                let value = truncate_report_text(&cell.text(), *w - 2.0, 2.5);
                self.text(
                    Point2D::new(x + 1.0, y - 1.0)?,
                    &value,
                    FrameTextStyle::new(2.5, TextAlign::TopLeft),
                    DisplayColor::Ink,
                );
                if let Some(target) = cell.target
                    && let Ok(rect) = Rect2D::new(Point2D::new(x, y - 6.0)?, Size2D::new(*w, 6.0)?)
                {
                    self.items.push(DisplayItem::Link { rect, target });
                }
                x += w;
            }
            self.report_grid(left, y, &sizes, 6.0);
            y -= 6.0;
        }
        let footer = format!("página {} de {} do relatório", page_index + 1, total_pages);
        self.text(
            Point2D::new(left, area.origin().y() + 4.0)?,
            &footer,
            FrameTextStyle::new(2.5, TextAlign::BottomLeft),
            DisplayColor::Ink,
        );
        if self.project.report_stale(definition.group)? {
            self.text(
                Point2D::new(left, area.origin().y() + 8.0)?,
                "Relatório desatualizado",
                FrameTextStyle::new(2.5, TextAlign::BottomLeft),
                DisplayColor::Reference,
            );
        }
        Ok(())
    }

    pub(super) fn report_grid(&mut self, left: f64, top: f64, widths: &[f64], height: f64) {
        let mut x = left;
        self.stroke(
            vec![
                Point2D::new(x, top).expect("finite"),
                Point2D::new(x, top - height).expect("finite"),
            ],
            false,
            0.15,
            DisplayColor::Ink,
        );
        for width in widths {
            x += width;
            self.stroke(
                vec![
                    Point2D::new(x, top).expect("finite"),
                    Point2D::new(x, top - height).expect("finite"),
                ],
                false,
                0.15,
                DisplayColor::Ink,
            );
        }
        self.stroke(
            vec![
                Point2D::new(left, top).expect("finite"),
                Point2D::new(x, top).expect("finite"),
            ],
            false,
            0.15,
            DisplayColor::Ink,
        );
        self.stroke(
            vec![
                Point2D::new(left, top - height).expect("finite"),
                Point2D::new(x, top - height).expect("finite"),
            ],
            false,
            0.15,
            DisplayColor::Ink,
        );
    }
}

fn truncate_report_text(text: &str, width: f64, size: f64) -> String {
    if width <= 0.0 {
        return String::new();
    }
    if font::text_width(text, size) <= width {
        return text.to_owned();
    }
    if font::text_width("…", size) > width {
        return String::new();
    }
    let mut result = String::new();
    for c in text.chars() {
        let next = format!("{result}{c}…");
        if font::text_width(&next, size) > width {
            break;
        }
        result.push(c);
    }
    result.push('…');
    result
}
