// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Symbol primitives and representations; start with Builder::representations.

use super::*;

impl Builder<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn primitive(
        &mut self,
        primitive: &SymbolPrimitive,
        style: PrimitiveStyle,
        t: Transform2D,
        base_width: f64,
        color: DisplayColor,
        text: &dyn Fn(&SymbolTextField) -> String,
    ) {
        use SymbolPrimitive as P;
        let chars = |s: &str| s.chars().count() as f64;
        let (points, closed) = match primitive {
            P::Text {
                origin,
                text,
                height,
            } => {
                let origin = *origin;
                let map = move |x: f64, y: f64| {
                    origin.translated(x, y).ok().and_then(|p| t.apply(p).ok())
                };
                self.mapped_text(text, (*height, height * 0.6 * chars(text)), &map, color);
                return;
            }
            P::DynamicText {
                origin,
                field,
                height,
                align,
                rotation_degrees,
            } => {
                let value = text(field);
                let width = height * 0.6 * chars(&value);
                let (fx, fy) = align.fractions();
                let (s, c) = rotation_degrees.to_radians().sin_cos();
                let (origin, height) = (*origin, *height);
                let map = move |x: f64, y: f64| {
                    let (lx, ly) = (x - fx * width, y - fy * height);
                    origin
                        .translated(lx * c - ly * s, lx * s + ly * c)
                        .ok()
                        .and_then(|p| t.apply(p).ok())
                };
                self.mapped_text(&value, (height, width), &map, color);
                return;
            }
            P::Anchor { .. } => return,
            P::Circle { .. } | P::Arc { .. } => {
                let Some((center, radius, start, sweep)) = world_curve(primitive, t) else {
                    return;
                };
                let mut points = arc_points(center, radius, start, sweep);
                let closed = matches!(primitive, P::Circle { .. });
                if closed {
                    points.pop();
                }
                (points, closed)
            }
            other => {
                let (local, closed) = match other {
                    P::Line { start, end } => (vec![*start, *end], false),
                    P::Polyline { points, closed } => (points.clone(), *closed),
                    P::Rectangle { rect } => (rect_corners(*rect), true),
                    P::Ellipse {
                        center,
                        radius_x,
                        radius_y,
                        rotation_degrees,
                    } => (
                        ellipse_points(*center, *radius_x, *radius_y, *rotation_degrees),
                        true,
                    ),
                    _ => return,
                };
                let Some(world) = local
                    .iter()
                    .map(|p| t.apply(*p).ok())
                    .collect::<Option<Vec<_>>>()
                else {
                    return;
                };
                (world, closed)
            }
        };
        let width = style.width_mm.map_or(base_width, |w| w.max(base_width));
        let fill = (style.fill && closed && points.len() > 2).then_some(color);
        self.path(
            points,
            closed,
            Some(DisplayStroke {
                width,
                dash: style.dash.pattern_mm().to_vec(),
                color,
            }),
            fill,
        );
    }

    pub(super) fn representations(&mut self, page: PageId) {
        let project = self.project;
        let mut reps = project.representations_on_page(page).unwrap_or_default();
        reps.sort_by_key(|r| {
            (
                r.visual().layer,
                u8::from(project.connection_route(r.id()).is_none()),
                r.id(),
            )
        });
        for rep in reps {
            if !rep.visual().visible {
                continue;
            }
            let t = rep.transform();
            if let Some(route) = project.connection_route(rep.id()) {
                self.layer = DisplayLayer::Texts;
                let points = route.points().to_vec();
                if let Some(number) = project
                    .connection(route.connection_id())
                    .and_then(Connection::number)
                    && let Some((origin, align)) =
                        wire_number_placement(&points, route.number_label_position())
                {
                    self.text(
                        origin,
                        number.as_str(),
                        FrameTextStyle::new(LABEL_HEIGHT, align),
                        DisplayColor::Ink,
                    );
                }
                self.layer = DisplayLayer::Wires;
                self.stroke(points, false, ROUTE_STROKE, DisplayColor::Ink);
                continue;
            }
            if let Some(graphic) = rep.graphic() {
                self.layer = DisplayLayer::Texts;
                if let Ok(primitive) = graphic.drawable() {
                    self.primitive(
                        &primitive,
                        graphic.style(),
                        t,
                        graphic.stroke_mm(),
                        DisplayColor::Ink,
                        &|field| field.placeholder(),
                    );
                }
            } else if let Some(d) = rep.symbol().and_then(|id| project.symbol_definition(id)) {
                self.layer = DisplayLayer::Symbols;
                let rid = rep.id();
                let text = |field: &SymbolTextField| {
                    project
                        .symbol_text(rid, field)
                        .unwrap_or_else(|| field.placeholder())
                };
                for (i, primitive) in d.primitives().iter().enumerate() {
                    self.primitive(
                        primitive,
                        d.style(i),
                        t,
                        SYMBOL_STROKE,
                        DisplayColor::Ink,
                        &text,
                    );
                }
            } else if let Some(local) = rep.local_bounds() {
                self.layer = DisplayLayer::Guides;
                let corners: Option<Vec<_>> = rect_corners(local)
                    .into_iter()
                    .map(|p| t.apply(p).ok())
                    .collect();
                if let Some(corners) = corners {
                    self.stroke(corners, true, 0.18, DisplayColor::Guide);
                }
            }
            self.layer = DisplayLayer::Texts;
            if let Some(label) = &rep.visual().label
                && let Ok(origin) = t.translation().translated(1.5, -1.5)
            {
                self.text(
                    origin,
                    label.as_str(),
                    FrameTextStyle::new(LABEL_HEIGHT, TextAlign::TopLeft),
                    DisplayColor::Ink,
                );
            }
        }
    }
}
