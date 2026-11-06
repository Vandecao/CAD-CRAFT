// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Converts raw QElectroTech symbol XML; start at `import_definition`.
use super::*;

fn num(e: &Element, k: &str) -> Option<f64> {
    e.attr(k)?
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
}

/// `line-style:dashed;line-weight:hight;filling:black;…`
fn style_of(e: &Element, s: f64, report: &mut ImportReport) -> PrimitiveStyle {
    let mut style = PrimitiveStyle::default();
    for part in e.attr("style").unwrap_or("").split(';') {
        let Some((k, v)) = part.split_once(':') else {
            continue;
        };
        match (k.trim(), v.trim()) {
            ("line-style", "dashed") => style.dash = LineDash::Dashed,
            ("line-style", "dotted") => style.dash = LineDash::Dotted,
            ("line-style", "dashdotted") => style.dash = LineDash::DashDot,
            ("line-weight", w) => {
                style.width_mm = match w {
                    "none" | "thin" => Some(0.13),
                    "hight" => Some(0.5),
                    "eleve" => Some(0.7),
                    _ => None,
                }
                .map(|mm: f64| (mm * s / DEFAULT_MM_PER_PX).clamp(0.05, 5.0))
            }
            ("filling", "none") | ("filling", "white") => {}
            ("filling", _) => style.fill = true,
            ("color", c) if c != "black" => {
                report.approximated(format!("cor '{c}' desenhada na cor padrão"))
            }
            _ => {}
        }
    }
    style
}

/// Altura em mm a partir de `font="Família,tamanho,…"`.
pub(crate) fn font_height(e: &Element, s: f64) -> f64 {
    e.attr("font")
        .and_then(|f| f.split(',').nth(1))
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| *v > 0.0)
        .map(|pt| pt * s * 0.8)
        .unwrap_or(2.5)
        .max(0.5)
}

pub(crate) fn align_of(e: &Element) -> TextAlign {
    use TextAlign::*;
    let h = e.attr("Halignment").unwrap_or("AlignLeft");
    let v = e.attr("Valignment").unwrap_or("AlignTop");
    match (v, h) {
        ("AlignTop", "AlignHCenter") => TopCenter,
        ("AlignTop", "AlignRight") => TopRight,
        ("AlignTop", _) => TopLeft,
        ("AlignVCenter", "AlignHCenter") => Center,
        ("AlignVCenter", "AlignRight") => MiddleRight,
        ("AlignVCenter", _) => MiddleLeft,
        (_, "AlignHCenter") => BottomCenter,
        (_, "AlignRight") => BottomRight,
        _ => BottomLeft,
    }
}

/// Nome preferido: pt-BR, pt, en, primeiro.
fn name_of(root: &Element) -> Option<String> {
    let names = root.child("names")?;
    let all: Vec<(&str, &str)> = names
        .children_named("name")
        .filter(|n| !n.text.trim().is_empty())
        .map(|n| (n.attr("lang").unwrap_or(""), n.text.trim()))
        .collect();
    ["pt_BR", "pt", "en"]
        .iter()
        .find_map(|l| all.iter().find(|(k, _)| k == l))
        .or(all.first())
        .map(|(_, v)| (*v).to_owned())
}

pub(crate) fn import_definition(
    root: &xml::Element,
    options: &ElmtOptions,
) -> ImportResult<ImportedSymbol> {
    let s = options.mm_per_px;
    if !(s.is_finite() && s > 0.0) {
        return Err(fail("escala inválida"));
    }
    if root.name != "definition" || root.attr("type") != Some("element") {
        return Err(fail("o arquivo não é um elemento do QElectroTech"));
    }
    let description = root
        .child("description")
        .ok_or_else(|| fail("elemento sem descrição gráfica"))?;
    let mut report = ImportReport::default();
    let pt = |x: f64, y: f64| Point2D::new(x * s, -y * s).ok();
    let mut primitives: Vec<SymbolPrimitive> = Vec::new();
    let mut styles: BTreeMap<usize, PrimitiveStyle> = BTreeMap::new();
    let mut ports: Vec<SymbolPort> = Vec::new();
    let mut used_ports = BTreeSet::new();
    let mut push = |p: Option<SymbolPrimitive>,
                    style: PrimitiveStyle,
                    what: &str,
                    report: &mut ImportReport| {
        match p.filter(|p| p.bounds().is_ok()) {
            Some(p) => {
                let style = if style.fill && !p.is_closed() {
                    PrimitiveStyle {
                        fill: false,
                        ..style
                    }
                } else {
                    style
                };
                if !style.is_default() {
                    styles.insert(primitives.len(), style);
                }
                primitives.push(p);
            }
            None => report.ignored(format!("{what} com geometria inválida")),
        }
    };
    for e in &description.children {
        let style = style_of(e, s, &mut report);
        match e.name.as_str() {
            "line" => {
                let p = (|| {
                    Some(SymbolPrimitive::Line {
                        start: pt(num(e, "x1")?, num(e, "y1")?)?,
                        end: pt(num(e, "x2")?, num(e, "y2")?)?,
                    })
                })();
                if e.attr("end1").is_some_and(|v| v != "none")
                    || e.attr("end2").is_some_and(|v| v != "none")
                {
                    report.approximated("extremidades de linha (setas) desenhadas sem marca");
                }
                push(p, style, "linha", &mut report);
            }
            "rect" => {
                let p = (|| {
                    let (x, y, w, h) = (
                        num(e, "x")?,
                        num(e, "y")?,
                        num(e, "width")?,
                        num(e, "height")?,
                    );
                    Some(SymbolPrimitive::Rectangle {
                        rect: Rect2D::new(pt(x, y + h)?, Size2D::new(w * s, h * s).ok()?).ok()?,
                    })
                })();
                if num(e, "rx").unwrap_or(0.0) > 0.0 {
                    report.approximated("retângulo com cantos arredondados desenhado reto");
                }
                push(p, style, "retângulo", &mut report);
            }
            "ellipse" | "circle" => {
                let p = (|| {
                    let (x, y) = (num(e, "x")?, num(e, "y")?);
                    let (w, h) = if e.name == "circle" {
                        let d = num(e, "diameter")?;
                        (d, d)
                    } else {
                        (num(e, "width")?, num(e, "height")?)
                    };
                    let center = pt(x + w / 2.0, y + h / 2.0)?;
                    Some(if (w - h).abs() < 1e-9 {
                        SymbolPrimitive::Circle {
                            center,
                            radius: w * s / 2.0,
                        }
                    } else {
                        SymbolPrimitive::Ellipse {
                            center,
                            radius_x: w * s / 2.0,
                            radius_y: h * s / 2.0,
                            rotation_degrees: 0.0,
                        }
                    })
                })();
                push(p, style, "elipse", &mut report);
            }
            "arc" => {
                let p = (|| {
                    let (x, y, w, h) = (
                        num(e, "x")?,
                        num(e, "y")?,
                        num(e, "width")?,
                        num(e, "height")?,
                    );
                    if (w - h).abs() > 1e-9 {
                        report.approximated("arco elíptico convertido em arco circular");
                    }
                    Some(SymbolPrimitive::Arc {
                        center: pt(x + w / 2.0, y + h / 2.0)?,
                        radius: (w + h) * s / 4.0,
                        start_degrees: num(e, "start")?,
                        sweep_degrees: num(e, "angle")?,
                    })
                })();
                push(p, style, "arco", &mut report);
            }
            "polygon" => {
                let mut points = Vec::new();
                for i in 1.. {
                    match (num(e, &format!("x{i}")), num(e, &format!("y{i}"))) {
                        (Some(x), Some(y)) => match pt(x, y) {
                            Some(p) if points.last() != Some(&p) => points.push(p),
                            _ => {}
                        },
                        _ => break,
                    }
                }
                let closed = e.attr("closed") != Some("false");
                if closed && points.len() > 2 && points.first() == points.last() {
                    points.pop();
                }
                push(
                    Some(SymbolPrimitive::Polyline { points, closed }),
                    style,
                    "polígono",
                    &mut report,
                );
            }
            "text" => {
                let p = (|| {
                    let text = e.attr("text")?.trim().to_owned();
                    if text.is_empty() {
                        return None;
                    }
                    Some(SymbolPrimitive::Text {
                        origin: pt(num(e, "x")?, num(e, "y")?)?,
                        text,
                        height: font_height(e, s),
                    })
                })();
                if num(e, "rotation").unwrap_or(0.0) != 0.0 {
                    report.approximated("texto fixo girado desenhado na horizontal");
                }
                push(p, PrimitiveStyle::default(), "texto", &mut report);
            }
            "dynamic_text" => {
                let from = e.attr("text_from").unwrap_or("");
                let info = e.child("info_name").map(|i| i.text.trim().to_owned());
                let user = e
                    .child("text")
                    .map(|t| t.text.trim().to_owned())
                    .unwrap_or_default();
                let origin = num(e, "x").zip(num(e, "y")).and_then(|(x, y)| pt(x, y));
                let height = font_height(e, s);
                let rotation = -num(e, "rotation").unwrap_or(0.0);
                let p = match (from, info.as_deref()) {
                    ("ElementInfo", Some("label")) => {
                        origin.map(|origin| SymbolPrimitive::DynamicText {
                            origin,
                            field: SymbolTextField::Designation,
                            height,
                            align: align_of(e),
                            rotation_degrees: rotation,
                        })
                    }
                    ("ElementInfo", Some(key)) if !key.is_empty() => {
                        origin.map(|origin| SymbolPrimitive::DynamicText {
                            origin,
                            field: SymbolTextField::DeviceProperty(key.to_owned()),
                            height,
                            align: align_of(e),
                            rotation_degrees: rotation,
                        })
                    }
                    _ if !user.is_empty() => {
                        if rotation != 0.0 {
                            report.approximated("texto fixo girado desenhado na horizontal");
                        }
                        origin.map(|origin| SymbolPrimitive::Text {
                            origin,
                            text: user,
                            height,
                        })
                    }
                    _ => {
                        report.ignored(format!("texto dinâmico sem conteúdo ({from})"));
                        continue;
                    }
                };
                push(p, PrimitiveStyle::default(), "texto dinâmico", &mut report);
            }
            "terminal" => {
                let (Some(x), Some(y)) = (num(e, "x"), num(e, "y")) else {
                    report.ignored("terminal sem posição");
                    continue;
                };
                let orientation = match e.attr("orientation") {
                    Some("n") => 90.0,
                    Some("s") => 270.0,
                    Some("w") => 180.0,
                    _ => 0.0,
                };
                let name = e.attr("name").unwrap_or("").trim();
                let mut id = if name.is_empty() {
                    (ports.len() + 1).to_string()
                } else {
                    name.to_owned()
                };
                if !used_ports.insert(id.clone()) {
                    id = format!("{id}#{}", ports.len() + 1);
                    used_ports.insert(id.clone());
                    report.approximated("terminais com o mesmo nome renomeados");
                }
                let identification = if name.is_empty() {
                    id.clone()
                } else {
                    name.to_owned()
                };
                match pt(x, y)
                    .and_then(|p| SymbolPort::new(id, p, orientation, identification, None).ok())
                {
                    Some(port) => ports.push(port),
                    None => report.ignored("terminal inválido"),
                }
            }
            other => report.ignored(format!("elemento QET '{other}' sem suporte")),
        }
    }
    if primitives.is_empty() {
        return Err(fail("elemento sem desenho reconhecido"));
    }
    let id = root
        .child("uuid")
        .and_then(|u| u.attr("uuid"))
        .map(|u| u.trim_matches(|c| c == '{' || c == '}').to_owned())
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| options.id_hint.clone());
    let id = SymbolDefinitionId::new(format!("qet.{id}"))
        .map_err(|e| fail(format!("identificação inválida: {e}")))?;
    let name = name_of(root).unwrap_or_else(|| options.id_hint.clone());
    let category = if options.category.trim().is_empty() {
        "QElectroTech".to_owned()
    } else {
        options.category.clone()
    };
    report::summarize(&mut report, primitives.len(), ports.len(), s);
    let definition = SymbolDefinition::new(id, 1, name, category, primitives, ports)
        .and_then(|d| d.with_styles(styles))
        .map_err(|e| fail(format!("símbolo resultante inválido: {e}")))?;
    Ok(ImportedSymbol { definition, report })
}
