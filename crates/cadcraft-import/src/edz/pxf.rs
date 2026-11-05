// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! EPLAN PXF macros: independently mapped from the user's files (ADR-0028).
use super::super::xml::{self, Element};
use super::super::{ImportReport, ImportResult, fail};
use cadcraft_model::domain::*;
use std::collections::BTreeSet;

fn domain(error: impl std::fmt::Display) -> super::super::ImportError {
    fail(error.to_string())
}
fn number(value: Option<&str>) -> Option<f64> {
    value?
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && v.abs() < 1e9)
}
fn point(value: Option<&str>) -> Option<Point2D> {
    let mut fields = value?.split('/');
    Point2D::new(number(fields.next())?, number(fields.next())?).ok()
}
fn kind(e: &Element) -> Option<u32> {
    e.attr("A1")
        .or_else(|| e.attr("A19"))
        .and_then(|v| v.split('/').next())
        .and_then(|v| v.parse().ok())
        .or_else(|| e.name.strip_prefix('S')?.split('x').next()?.parse().ok())
}
fn text(raw: &str) -> String {
    let localized = super::super::fn1::localized(raw);
    if localized.default.is_empty() {
        raw.to_owned()
    } else {
        localized.default
    }
}
fn unknown(code: impl Into<String>, report: &mut ImportReport, codes: &mut BTreeSet<String>) {
    let code = code.into();
    codes.insert(code.clone());
    report.ignored(format!("PXF não convertido: {code}"));
}
fn rectangle(a: Point2D, b: Point2D) -> Option<SymbolPrimitive> {
    Some(SymbolPrimitive::Rectangle {
        rect: Rect2D::new(
            Point2D::new(a.x().min(b.x()), a.y().min(b.y())).ok()?,
            Size2D::new((a.x() - b.x()).abs(), (a.y() - b.y()).abs()).ok()?,
        )
        .ok()?,
    })
}
fn primitive(
    e: &Element,
    report: &mut ImportReport,
    codes: &mut BTreeSet<String>,
) -> Option<SymbolPrimitive> {
    let p = match kind(e)? {
        31 => Some(SymbolPrimitive::Line {
            start: point(e.attr("A531").or_else(|| e.attr("A1079")))
                .unwrap_or(Point2D::new(0., 0.).ok()?),
            end: point(e.attr("A532").or_else(|| e.attr("A1081")))?,
        }),
        89 => rectangle(
            point(e.attr("A1651").or_else(|| e.attr("A3319")))
                .unwrap_or(Point2D::new(0., 0.).ok()?),
            point(e.attr("A1652").or_else(|| e.attr("A3321")))?,
        ),
        30 => Some(SymbolPrimitive::Text {
            origin: point(e.attr("A501").or_else(|| e.attr("A1019")))
                .unwrap_or(Point2D::new(0., 0.).ok()?),
            text: text(e.attr("A511").or_else(|| e.attr("A1039"))?),
            height: e
                .children
                .iter()
                .find_map(|c| number(c.attr("A961").or_else(|| c.attr("A1939"))))
                .filter(|h| *h > 0.0)
                .unwrap_or(2.5),
        }),
        32 => {
            let center = point(e.attr("A555"))?;
            let axis = point(e.attr("A556"))?;
            let radius = axis.x().hypot(axis.y());
            let ratio = number(e.attr("A557")).unwrap_or(1.0);
            let start = number(e.attr("A553")).unwrap_or(0.0);
            let end = number(e.attr("A554"))?;
            let full = (end - start).abs() >= std::f64::consts::TAU - 1e-6;
            let rotation = axis.y().atan2(axis.x()).to_degrees();
            if (ratio - 1.0).abs() < 1e-6 {
                if full {
                    Some(SymbolPrimitive::Circle { center, radius })
                } else {
                    Some(SymbolPrimitive::Arc {
                        center,
                        radius,
                        start_degrees: start.to_degrees() + rotation,
                        sweep_degrees: (end - start).to_degrees(),
                    })
                }
            } else if full {
                Some(SymbolPrimitive::Ellipse {
                    center,
                    radius_x: radius,
                    radius_y: radius * ratio,
                    rotation_degrees: rotation,
                })
            } else {
                unknown(format!("{}.arco-elíptico", e.name), report, codes);
                None
            }
        }
        34 => {
            let points: Vec<_> = e
                .attr("A621")
                .or_else(|| e.attr("A1259"))?
                .split(';')
                .filter_map(|v| {
                    if v.is_empty() {
                        Point2D::new(0., 0.).ok()
                    } else {
                        point(Some(v))
                    }
                })
                .collect();
            if e.attr("A1261").is_some_and(|v| {
                v.split(';')
                    .any(|n| number(Some(n)).is_some_and(|n| n != 0.0))
            }) {
                report.approximated(
                    "PXF: segmentos curvos da polilinha convertidos em segmentos retos",
                );
            }
            Some(SymbolPrimitive::Polyline {
                points,
                closed: e.attr("A623").or_else(|| e.attr("A1263")) == Some("1"),
            })
        }
        _ => {
            unknown(&e.name, report, codes);
            None
        }
    };
    p.filter(|p| p.bounds().is_ok())
}
fn transform(e: &Element, report: &mut ImportReport) -> ImportResult<Transform2D> {
    let t = e.children.iter().find(|c| c.name.starts_with("S40x"));
    let origin = t
        .and_then(|t| point(t.attr("A762")))
        .unwrap_or(Point2D::new(0.0, 0.0).expect("origin"));
    let matrix = t.and_then(|t| t.children.iter().find(|c| c.name.starts_with("S39x")));
    let (angle, mirror) = if let Some(m) = matrix {
        let a = number(m.attr("A751")).unwrap_or(1.0);
        let b = number(m.attr("A752")).unwrap_or(0.0);
        let c = number(m.attr("A753")).unwrap_or(0.0);
        let d = number(m.attr("A754")).unwrap_or(1.0);
        if ((a * a + b * b) - 1.0).abs() > 1e-6
            || ((c * c + d * d) - 1.0).abs() > 1e-6
            || (a * c + b * d).abs() > 1e-6
        {
            report.approximated(
                "PXF: escala/cisalhamento da ocorrência não suportado; preservada posição",
            );
            (0.0, false)
        } else {
            (b.atan2(a).to_degrees(), a * d - b * c < 0.0)
        }
    } else {
        (0.0, false)
    };
    Transform2D::new(origin, angle, false, mirror).map_err(domain)
}

struct Converter<'a> {
    project: Project,
    page: PageId,
    id: &'a str,
    report: &'a mut ImportReport,
    codes: &'a mut BTreeSet<String>,
    count: usize,
}
impl Converter<'_> {
    fn insert(&mut self, definition: SymbolDefinition, t: Transform2D) -> ImportResult<()> {
        if definition.ports().is_empty() {
            self.project
                .add_symbol_definition(definition.clone())
                .map_err(domain)?;
            let r = Representation::new_symbol(self.project.id(), self.page, &definition, None, t)
                .map_err(domain)?;
            self.project.insert_representation(r).map_err(domain)?;
        } else {
            let designation = self.project.next_device_designation("D").map_err(domain)?;
            self.project
                .place_symbol(SymbolPlacement {
                    page: self.page,
                    definition,
                    transform: t,
                    device: PlacementDevice::New {
                        designation,
                        classification: "EPLAN".into(),
                    },
                    function_kind: FunctionKind::Other(Text::new("EPLAN").map_err(domain)?),
                })
                .map_err(domain)?;
        }
        Ok(())
    }
    fn walk(&mut self, e: &Element) -> ImportResult<()> {
        for child in &e.children {
            match kind(child) {
                Some(4 | 26) => self.walk(child)?,
                Some(17) => {
                    if let Err(error) = self.occurrence(child) {
                        self.report
                            .ignored(format!("PXF ocorrência não convertida: {error}"));
                    }
                }
                Some(30 | 31 | 32 | 34 | 89) => {
                    if let Some(p) = primitive(child, self.report, self.codes) {
                        self.count += 1;
                        let d = SymbolDefinition::new(
                            SymbolDefinitionId::new(format!("{}.drawing.{}", self.id, self.count))
                                .map_err(domain)?,
                            1,
                            "Desenho PXF",
                            "EPLAN",
                            vec![p],
                            vec![],
                        )
                        .map_err(domain)?;
                        self.insert(
                            d,
                            Transform2D::new(
                                Point2D::new(0.0, 0.0).map_err(domain)?,
                                0.0,
                                false,
                                false,
                            )
                            .map_err(domain)?,
                        )?;
                    } else {
                        self.report
                            .ignored(format!("PXF {}: geometria inválida ou ausente", child.name));
                    }
                }
                Some(6 | 14 | 37 | 76) => {
                    self.report.info(format!(
                        "PXF {}: metadados/configuração sem conteúdo elétrico",
                        child.name
                    ));
                }
                Some(_) => unknown(&child.name, self.report, self.codes),
                None if child.name == "ESymLib" => {
                    if !child.children.is_empty() {
                        unknown(
                            "ESymLib.conteúdo-sem-esquema-conhecido",
                            self.report,
                            self.codes,
                        );
                    }
                }
                None => {
                    if child.name.starts_with(['O', 'P', 'S']) {
                        unknown(&child.name, self.report, self.codes);
                    }
                }
            }
        }
        Ok(())
    }
    fn occurrence(&mut self, e: &Element) -> ImportResult<()> {
        self.count += 1;
        let name = e
            .child("P11")
            .and_then(|p| p.attr("P19008"))
            .unwrap_or("Símbolo EPLAN");
        let mut primitives = Vec::new();
        for c in &e.children {
            if matches!(kind(c), Some(30 | 31 | 32 | 34 | 89)) {
                if let Some(p) = primitive(c, self.report, self.codes) {
                    primitives.push(p);
                } else {
                    self.report
                        .ignored(format!("PXF {}: geometria inválida", c.name));
                }
            } else if c.name.starts_with('S')
                && !c.name.starts_with("S40x")
                && !c.name.starts_with("S61x")
            {
                unknown(&c.name, self.report, self.codes);
            } else if c.name.starts_with('P') {
                for (key, _) in &c.attrs {
                    if key != "P19008" {
                        unknown(key, self.report, self.codes);
                    }
                }
            } else if c.name.starts_with('O') {
                unknown(&c.name, self.report, self.codes);
            }
            if c.name.starts_with("S61x") {
                for nested in &c.children {
                    unknown(&nested.name, self.report, self.codes);
                }
            }
        }
        let has_supported_graphics = !primitives.is_empty();
        let mut ports = Vec::new();
        // In the observed PXF schema A1061 is the number of connection points.
        // A single-port symbol has its connection at the insertion origin.
        let port_count = e
            .children
            .iter()
            .find(|c| c.name.starts_with("S61x"))
            .and_then(|s| s.attr("A1061"))
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(0);
        if port_count == 1 {
            let label = e
                .attr("A185")
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("1");
            let normalized: String = label
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '_' || c == '.' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            if normalized != label {
                self.report.approximated(format!(
                    "PXF: identificação da porta '{label}' normalizada como '{normalized}'"
                ));
                primitives.push(SymbolPrimitive::Text {
                    origin: Point2D::new(0., 1.).map_err(domain)?,
                    text: label.into(),
                    height: 1.5,
                });
            }
            ports.push(
                SymbolPort::new(
                    "1",
                    Point2D::new(0.0, 0.0).map_err(domain)?,
                    0.0,
                    normalized,
                    None,
                )
                .map_err(domain)?,
            );
            self.report
                .approximated("PXF: porta única na origem; direção não disponível");
        } else if port_count > 1 {
            self.report.ignored(format!(
                "PXF: {port_count} portas sem posições reconhecidas; não inventadas"
            ));
            self.codes.insert("S61x183.A1061>1".into());
        }
        if !has_supported_graphics {
            primitives.push(
                rectangle(
                    Point2D::new(-2.5, -2.5).map_err(domain)?,
                    Point2D::new(2.5, 2.5).map_err(domain)?,
                )
                .expect("fixed rectangle"),
            );
            primitives.push(SymbolPrimitive::Text {
                origin: Point2D::new(-2.5, 3.0).map_err(domain)?,
                text: name.into(),
                height: 1.5,
            });
            self.report.approximated(format!(
                "PXF: símbolo '{name}' sem definição gráfica embutida; caixa com portas conhecidas"
            ));
        }
        let d = SymbolDefinition::new(
            SymbolDefinitionId::new(format!("{}.symbol.{}", self.id, self.count))
                .map_err(domain)?,
            1,
            name,
            "EPLAN",
            primitives,
            ports,
        )
        .map_err(domain)?;
        let t = transform(e, self.report)?;
        self.insert(d, t)
    }
}

pub(super) fn convert(
    bytes: &[u8],
    id: &str,
    report: &mut ImportReport,
    unknown_codes: &mut BTreeSet<String>,
) -> ImportResult<Option<MacroDefinition>> {
    let root = xml::parse(bytes)?;
    if root.name != "EplanPxfRoot" {
        return Err(fail("macro não é XML PXF do EPLAN"));
    }
    if !matches!(root.attr("Type"), Some("WindowMacro" | "PageMacro")) {
        report.ignored("PXF: tipo de macro fora do escopo 2D");
        return Ok(None);
    }
    let variants: Vec<_> = root.children_named("MacroVariant").collect();
    let chosen = variants
        .iter()
        .find(|v| v.attr("MacroFuncType") == Some("1"))
        .or_else(|| {
            variants
                .iter()
                .find(|v| v.attr("MacroFuncType") != Some("13"))
        })
        .copied();
    if !variants.is_empty() && chosen.is_none() {
        report.ignored("PXF: macro contém somente variantes 3D");
        return Ok(None);
    }
    let selected = chosen.unwrap_or(&root);
    report.approximated("PXF: estilos de camada e textos dinâmicos específicos permanecem sem equivalência; geometria usa estilo padrão");
    if variants.len() > 1 {
        report.approximated(format!(
            "PXF: primeira variante 2D convertida; {} variantes adicionais não convertidas",
            variants.len() - 1
        ));
    }
    let mut project = Project::new("Importação PXF").map_err(domain)?;
    let doc = project
        .insert_document(Document::new(project.id(), "Macro").map_err(domain)?)
        .map_err(domain)?;
    let page = project
        .insert_page(
            Page::new(
                project.id(),
                doc,
                "Macro",
                "1",
                PageKind::MultiLine,
                Size2D::new(420.0, 297.0).map_err(domain)?,
            )
            .map_err(domain)?,
        )
        .map_err(domain)?;
    let mut converter = Converter {
        project,
        page,
        id,
        report,
        codes: unknown_codes,
        count: 0,
    };
    converter.walk(selected)?;
    if converter.project.representations().next().is_none() {
        converter
            .report
            .ignored("PXF: macro sem conteúdo 2D convertido");
        return Ok(None);
    }
    let (content, capture_report) = converter.project.capture_page_macro(page).map_err(domain)?;
    if !capture_report.is_clean() {
        converter.report.approximated(capture_report.summary());
    }
    let base =
        point(selected.attr("ReferencePoint")).unwrap_or(Point2D::new(0.0, 0.0).map_err(domain)?);
    let name = root
        .attr("Name")
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(id);
    let definition = MacroDefinition::new(id, name, MacroKind::Window, base, content)
        .map_err(domain)?
        .with_category("EPLAN")
        .map_err(domain)?;
    Ok(Some(definition))
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASIC: &[u8] = br#"<EplanPxfRoot Type="WindowMacro" Name="Synthetic"><MacroVariant MacroFuncType="1" ReferencePoint="10/20/0"><ESymLib Name="Test" ID="0"/><O4 A1="4/1"><O17 A1="17/2" A185="A"><S61x183 A1061="1"/><S40x1201 A762="10/20"><S39x761 A751="0" A752="1" A753="-1" A754="0"/></S40x1201><P11 P19008="Missing"/></O17><O999 A1="999/3"/></O4></MacroVariant></EplanPxfRoot>"#;
    #[test]
    fn fallback_keeps_port_and_macro_inserts_and_undoes() {
        let mut report = ImportReport::default();
        let mut unknown = BTreeSet::new();
        let macro_def = convert(BASIC, "test.pxf", &mut report, &mut unknown)
            .unwrap()
            .unwrap();
        assert_eq!(macro_def.content().definitions().len(), 1);
        let symbol = &macro_def.content().definitions()[0];
        assert_eq!(symbol.ports().len(), 1);
        assert_eq!(symbol.ports()[0].identification(), "A");
        assert!(unknown.contains("O999"));
        let mut p = Project::new("Test").unwrap();
        let d = p
            .insert_document(Document::new(p.id(), "D").unwrap())
            .unwrap();
        let page = p
            .insert_page(
                Page::new(
                    p.id(),
                    d,
                    "P",
                    "1",
                    PageKind::MultiLine,
                    Size2D::new(420., 297.).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        let t = Transform2D::new(Point2D::new(100., 100.).unwrap(), 0., false, false).unwrap();
        let (delta, _) = p.insert_macro(&macro_def, page, t).unwrap();
        assert_eq!(p.port_bindings().count(), 1);
        let port = p.port_bindings().next().unwrap().0.clone();
        assert_eq!(
            p.port_position(&port).unwrap(),
            Point2D::new(100., 100.).unwrap()
        );
        p.restore_macro_insert(&delta, false).unwrap();
        assert_eq!(p.representations().count(), 0);
        p.restore_macro_insert(&delta, true).unwrap();
        p.validate().unwrap();
    }
    #[test]
    fn missing_definition_still_has_box_when_port_label_needs_normalization() {
        let source = std::str::from_utf8(BASIC)
            .unwrap()
            .replace("A185=\"A\"", "A185=\"A-1\"");
        let imported = convert(
            source.as_bytes(),
            "fallback.normalized",
            &mut ImportReport::default(),
            &mut BTreeSet::new(),
        )
        .unwrap()
        .unwrap();
        let symbol = &imported.content().definitions()[0];
        assert!(
            symbol
                .primitives()
                .iter()
                .any(|p| matches!(p, SymbolPrimitive::Rectangle { .. }))
        );
        assert_eq!(symbol.ports()[0].identification(), "A_1");
    }
    #[test]
    fn malformed_and_non_pxf_rejected() {
        for bytes in [b"<bad>".as_slice(), b"<other/>".as_slice()] {
            assert!(
                convert(
                    bytes,
                    "test",
                    &mut ImportReport::default(),
                    &mut BTreeSet::new()
                )
                .is_err()
            );
        }
    }
    #[test]
    fn observed_primitives_and_invalid_port_labels_are_converted() {
        let bytes =
            br#"<EplanPxfRoot Type="WindowMacro"><MacroVariant MacroFuncType="1"><O4 A1="4/1">
        <O31 A1="31/2" A531="1/2" A532="3/4"/>
        <O34 A1="34/3" A621=";5/0;5/5" A623="0"/>
        <O89 A1="89/4" A1651="1/1" A1652="5/5"/>
        <O32 A1="32/5" A553="0" A554="6.283185307179586" A555="10/10" A556="2/0" A557="1"/>
        <O32 A1="32/6" A553="0" A554="1.5707963267948966" A555="10/10" A556="2/0" A557="1"/>
        <O32 A1="32/7" A553="0" A554="6.283185307179586" A555="10/10" A556="2/0" A557="0.5"/>
        <O30 A1="30/8" A501="1/2" A511="en_US@Test;"><S54x505 A961="2"/></O30>
        <O17 A1="17/9" A185="1/L1"><S61x183 A1061="1"/><P11 P19008="Test"/></O17>
        </O4></MacroVariant></EplanPxfRoot>"#;
        let mut report = ImportReport::default();
        let mut codes = BTreeSet::new();
        let d = convert(bytes, "primitives", &mut report, &mut codes)
            .unwrap()
            .unwrap();
        let primitives: Vec<_> = d
            .content()
            .definitions()
            .iter()
            .flat_map(|d| d.primitives())
            .collect();
        assert!(
            primitives
                .iter()
                .any(|p| matches!(p,SymbolPrimitive::Line{start,..} if start.x()==1.))
        );
        assert!(
            primitives
                .iter()
                .any(|p| matches!(p,SymbolPrimitive::Polyline{points,..} if points.len()==3))
        );
        assert!(
            primitives
                .iter()
                .any(|p| matches!(p,SymbolPrimitive::Circle{radius,..} if *radius==2.))
        );
        assert!(primitives.iter().any(|p|matches!(p,SymbolPrimitive::Arc{sweep_degrees,..} if (*sweep_degrees-90.).abs()<1e-8)));
        assert!(
            primitives
                .iter()
                .any(|p| matches!(p,SymbolPrimitive::Ellipse{radius_y,..} if *radius_y==1.))
        );
        assert!(
            primitives
                .iter()
                .any(|p| matches!(p,SymbolPrimitive::Text{text,..} if text=="Test"))
        );
        assert!(
            d.content()
                .definitions()
                .iter()
                .any(|d| d.ports().iter().any(|p| p.identification() == "1_L1"))
        );
        d.validate().unwrap();
    }
}
