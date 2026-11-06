// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Converts conductors, sheet-report markers and their groups; start at `convert`.
use super::*;

pub(super) fn convert(
    importer: &mut Importer<'_>,
    state: &DiagramState,
    placement: &PlacementState,
) -> ImportResult<()> {
    let DiagramState {
        pages,
        instances,
        wires,
        ..
    } = state;
    let PlacementState {
        by_uuid,
        placed,
        reports,
    } = placement;
    let s = importer.s;
    let world = |page, x, y| super::world(pages, s, page, x, y);
    // Terminals per sheet: id → (instance, terminal id).
    let mut terminal_index: BTreeMap<(usize, String), usize> = BTreeMap::new();
    for (n, inst) in instances.iter().enumerate() {
        for (id, _, _) in &inst.terminals {
            terminal_index.insert((inst.page, id.clone()), n);
        }
    }
    let port_ref = |importer: &Importer, n: usize, terminal: &str| -> Option<PortRef> {
        let rep = *placed.get(&n)?;
        let port = importer.port_for(&instances[n], terminal)?;
        PortRef::new(rep, port).ok()
    };

    // Sheet reports: endpoint on the other side of the conductor.
    let report_set: BTreeSet<usize> = reports.iter().copied().collect();
    let mut report_point: BTreeMap<usize, ConnectionPointId> = BTreeMap::new();
    let mut report_lines: Vec<(usize, Vec<Point2D>)> = vec![];
    let terminal_world = |importer: &Importer, n: usize, terminal: &str| -> Option<Point2D> {
        let inst = &instances[n];
        let (x, y) = inst
            .terminals
            .iter()
            .find(|(id, _, _)| id == terminal)
            .map(|(_, x, y)| (*x, *y))
            .or_else(|| {
                importer
                    .definitions
                    .get(&inst.def)?
                    .terminals
                    .get(terminal)
                    .copied()
            })?;
        let t = Transform2D::new(
            world(inst.page, inst.x, inst.y).ok()?,
            rotation(inst.orientation),
            false,
            false,
        )
        .ok()?;
        t.apply(Point2D::new(x * s, -y * s).ok()?).ok()
    };
    for wire in wires {
        let end_of = |e: &Option<String>, t: &String| match e {
            Some(uuid) => by_uuid.get(uuid).copied(),
            None => terminal_index.get(&(wire.page, t.clone())).copied(),
        };
        let ends = [end_of(&wire.e1, &wire.t1), end_of(&wire.e2, &wire.t2)];
        let (Some(a), Some(b)) = (ends[0], ends[1]) else {
            importer.report.ignored("condutor com terminal inexistente");
            continue;
        };
        if report_set.contains(&a) || report_set.contains(&b) {
            let (report, other, terminal) = if report_set.contains(&a) {
                (a, b, &wire.t2)
            } else {
                (b, a, &wire.t1)
            };
            if let Some(port) = port_ref(importer, other, terminal)
                && let Some(point) = importer.project.port_binding(&port)
            {
                report_point.insert(report, point);
            }
            // Keep the conductor stroke as drawing content with no electrical effect.
            let report_terminal = if report == a { &wire.t1 } else { &wire.t2 };
            let other_pos = port_ref(importer, other, terminal)
                .and_then(|p| importer.project.port_position(&p).ok())
                .or_else(|| terminal_world(importer, other, terminal));
            if let (Some(rp), Some(op)) =
                (terminal_world(importer, report, report_terminal), other_pos)
            {
                let (start, end) = if report == a { (rp, op) } else { (op, rp) };
                let path = qet_path(start, end, &wire.segments, s);
                if path.len() >= 2 {
                    report_lines.push((wire.page, path));
                }
            }
            continue;
        }
        let (Some(source), Some(target)) = (
            port_ref(importer, a, &wire.t1),
            port_ref(importer, b, &wire.t2),
        ) else {
            importer
                .report
                .ignored("condutor ligado a elemento não importado ou terminal sem porta");
            continue;
        };
        let (Ok(start), Ok(end)) = (
            importer.project.port_position(&source),
            importer.project.port_position(&target),
        ) else {
            importer.report.ignored("condutor sem posição de porta");
            continue;
        };
        let points = qet_path(start, end, &wire.segments, s);
        let result = importer
            .project
            .edit_connection_drawing(ConnectionDrawingEdit::CreateRouteExact {
                source: source.clone(),
                target: target.clone(),
                points: points.clone(),
            })
            .or_else(|_| {
                // Invalid original path: fall back to a simple L-shaped route.
                let mut simple = vec![start];
                if start.x() != end.x()
                    && start.y() != end.y()
                    && let Ok(corner) = Point2D::new(end.x(), start.y())
                {
                    simple.push(corner);
                }
                simple.push(end);
                let simple = simplify(simple);
                importer
                    .project
                    .edit_connection_drawing(ConnectionDrawingEdit::CreateRouteExact {
                        source: source.clone(),
                        target: target.clone(),
                        points: simple,
                    })
                    .inspect(|_| {
                        importer
                            .report
                            .approximated("caminho do condutor refeito em L");
                    })
            });
        match result {
            Ok(_) => {
                importer.stats.conductors += 1;
                let connection = importer
                    .project
                    .connection_routes()
                    .find(|r| r.source() == &source && r.target() == &target)
                    .map(ConnectionRoute::connection_id);
                if let (Some(connection), false) = (connection, wire.num.is_empty()) {
                    match Text::new(wire.num.clone()) {
                        Ok(number) => {
                            let _ = importer.project.set_connection_number(
                                connection,
                                Some(number),
                                true,
                            );
                        }
                        Err(_) => importer.report.ignored("número de fio inválido"),
                    }
                }
            }
            Err(e) => importer.report.ignored(format!("condutor: {e}")),
        }
    }

    // Interruption markers and groups.
    let mut marker_of: BTreeMap<usize, RepresentationId> = BTreeMap::new();
    for n in reports {
        let inst = &instances[*n];
        let Some(def) = importer.definitions.get(&inst.def) else {
            importer.report.ignored("relatório de folha sem definição");
            continue;
        };
        let Some(first) = def.symbol.ports().first() else {
            importer.report.ignored("relatório de folha sem terminal");
            continue;
        };
        // The marker requires its sole port "p" at the origin: shift the drawing
        // and compensate with the occurrence position.
        let offset = first.position();
        let port = SymbolPort::new(
            "p",
            Point2D::new(0.0, 0.0).map_err(domain)?,
            first.orientation_degrees(),
            "P",
            None,
        )
        .map_err(domain)?;
        // Without a device, designation would show only "-K?"; the marker name
        // already identifies the interruption. Styles follow their indices.
        let mut primitives = vec![];
        let mut styles = BTreeMap::new();
        for (i, p) in def.symbol.primitives().iter().enumerate() {
            if matches!(
                p,
                SymbolPrimitive::DynamicText {
                    field: SymbolTextField::Designation,
                    ..
                }
            ) {
                continue;
            }
            if let Some(style) = def.symbol.styles().get(&i) {
                styles.insert(primitives.len(), *style);
            }
            primitives.push(p.translated(-offset.x(), -offset.y()).map_err(domain)?);
        }
        let marker_def = SymbolDefinition::new(
            SymbolDefinitionId::new(format!("{}.relatorio", def.symbol.id().as_str()))
                .map_err(domain)?,
            1,
            def.symbol.name(),
            "QElectroTech (relatório de folha)",
            primitives,
            vec![port],
        )
        .and_then(|d| d.with_styles(styles))
        .map_err(domain)?;
        let point = report_point.get(n).copied();
        let rotate = Transform2D::new(
            world(inst.page, inst.x, inst.y)?,
            rotation(inst.orientation),
            false,
            false,
        )
        .map_err(domain)?;
        let transform = Transform2D::new(
            rotate.apply(offset).map_err(domain)?,
            rotation(inst.orientation),
            false,
            false,
        )
        .map_err(domain)?;
        let rep = Representation::new_symbol(
            importer.project.id(),
            pages[inst.page].0,
            &marker_def,
            point.map(RepresentationTarget::ConnectionPoint),
            transform,
        )
        .map_err(domain)?;
        let id = rep.id();
        let name = if inst.label.trim().is_empty() {
            format!("R{}", marker_of.len() + 1)
        } else {
            inst.label.trim().to_owned()
        };
        let insert = importer.project.edit_marker(MarkerEdit::Insert {
            marker: ElectricalMarker::new(
                id,
                Text::new(name).map_err(domain)?,
                MarkerKind::Interruption,
                point,
            ),
            representation: Box::new(rep),
            definition: Box::new(marker_def),
        });
        match insert {
            Ok(_) => {
                marker_of.insert(*n, id);
            }
            Err(e) => importer.report.ignored(format!("relatório de folha: {e}")),
        }
    }
    let mut grouped = BTreeSet::new();
    for (n, rep) in &marker_of {
        if grouped.contains(n) {
            continue;
        }
        let partners: Vec<usize> = instances[*n]
            .links
            .iter()
            .filter_map(|u| by_uuid.get(u).copied())
            .filter(|p| marker_of.contains_key(p) && !grouped.contains(p))
            .collect();
        if partners.is_empty() {
            importer.report.approximated("relatório de folha sem par");
            continue;
        }
        let mut members = vec![*rep];
        members.extend(partners.iter().map(|p| marker_of[p]));
        grouped.insert(*n);
        grouped.extend(partners);
        match importer.project.edit_marker(MarkerEdit::SetGroup {
            id: ReferenceGroupId::new(),
            mode: ReferenceMode::Star,
            members,
            continuity: false,
        }) {
            Ok(_) => importer.stats.sheet_reports += 1,
            Err(e) => importer
                .report
                .ignored(format!("grupo de relatório de folha: {e}")),
        }
    }

    // Sheet-report conductor strokes are drawing-only, with one stroke
    // per segment because free drawing content has no polyline.
    for (page, path) in report_lines {
        let mut failed = None;
        for w in path.windows(2) {
            let (a, b) = (w[0], w[1]);
            let rep = Point2D::new(b.x() - a.x(), b.y() - a.y())
                .and_then(|end| {
                    PageGraphic::new(
                        SymbolPrimitive::Line {
                            start: Point2D::new(0.0, 0.0)?,
                            end,
                        },
                        0.25,
                        TextAlignment::Left,
                    )
                })
                .and_then(|g| {
                    Representation::new_graphic(
                        importer.project.id(),
                        pages[page].0,
                        g,
                        Transform2D::new(a, 0.0, false, false)?,
                    )
                })
                .and_then(|r| importer.project.insert_representation(r));
            if let Err(e) = rep {
                failed = Some(e.to_string());
            }
        }
        if let Some(e) = failed {
            importer
                .report
                .ignored(format!("traço de condutor de relatório de folha: {e}"));
        }
    }

    Ok(())
}

/// Removes duplicate points and points inside straight segments, including
/// backtracking along an axis, while preserving the orthogonal path.
fn simplify(points: Vec<Point2D>) -> Vec<Point2D> {
    let mut out: Vec<Point2D> = Vec::with_capacity(points.len());
    for p in points {
        if out.last() == Some(&p) {
            continue;
        }
        if out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            if (a.x() == b.x() && b.x() == p.x()) || (a.y() == b.y() && b.y() == p.y()) {
                out.pop();
                if out.last() == Some(&p) {
                    continue;
                }
            }
        }
        out.push(p);
    }
    out
}

/// Orthogonal QET conductor path: segments from `start`
/// (positive vertical means downward on screen), closed through `end`.
fn qet_path(start: Point2D, end: Point2D, segments: &[(bool, f64)], s: f64) -> Vec<Point2D> {
    let mut points = vec![start];
    let mut last = start;
    for (vertical, length) in segments {
        let next = if *vertical {
            Point2D::new(last.x(), last.y() - length * s)
        } else {
            Point2D::new(last.x() + length * s, last.y())
        };
        if let Ok(next) = next
            && next != last
        {
            points.push(next);
            last = next;
        }
    }
    if last != end {
        if last.x() != end.x()
            && last.y() != end.y()
            && let Ok(corner) = Point2D::new(end.x(), last.y())
        {
            points.push(corner);
        }
        points.push(end);
    }
    simplify(points)
}
