// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Symbol catalog and factory definitions; start with SymbolLibrary::builtin.
//!
//! Version 2 of the factory symbols draws contacts, coils and buttons in the
//! IEC style, shows the device designation and names the ports by IEC 60947
//! (A1-A2, 13-14, 21-22). Port IDs and positions stay those of version 1
//! ("1" at the bottom, "2" at the top), so "Atualizar símbolos do projeto"
//! accepts the new version in projects that already use the old one.
use super::*;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Ten original industrial reference drawings, without normative certification.
#[derive(Debug)]
pub struct SymbolLibrary {
    definitions: BTreeMap<SymbolDefinitionId, SymbolDefinition>,
}
impl SymbolLibrary {
    /// Return the shared built-in symbol library.
    pub fn builtin() -> &'static Self {
        static BUILTIN: OnceLock<SymbolLibrary> = OnceLock::new();
        BUILTIN.get_or_init(build_builtin)
    }
    /// Iterate definitions without exposing mutable access.
    pub fn definitions(&self) -> impl Iterator<Item = &SymbolDefinition> {
        self.definitions.values()
    }
    /// Find a symbol definition by its textual library ID.
    pub fn get(&self, id: &SymbolDefinitionId) -> Option<&SymbolDefinition> {
        self.definitions.get(id)
    }
}
fn p(x: f64, y: f64) -> Point2D {
    Point2D::new(x, y).expect("static point")
}
fn line(x: f64, y: f64, a: f64, b: f64) -> SymbolPrimitive {
    SymbolPrimitive::Line {
        start: p(x, y),
        end: p(a, b),
    }
}
fn rect(x: f64, y: f64, w: f64, h: f64) -> SymbolPrimitive {
    SymbolPrimitive::Rectangle {
        rect: Rect2D::new(p(x, y), Size2D::new(w, h).expect("static size")).expect("static rect"),
    }
}
fn text(value: &str) -> SymbolPrimitive {
    SymbolPrimitive::Text {
        origin: p(-1.8, -1.5),
        text: value.into(),
        height: 3.,
    }
}
/// Device designation (-K1) to the left of the symbol, ending at `x`.
fn designation(x: f64) -> SymbolPrimitive {
    SymbolPrimitive::DynamicText {
        origin: p(x, 0.),
        field: SymbolTextField::Designation,
        height: 2.5,
        align: TextAlign::MiddleRight,
        rotation_degrees: 0.,
    }
}
/// Point label of `port`, beside its lead on the right.
fn port_label(port: &str, y: f64) -> SymbolPrimitive {
    SymbolPrimitive::DynamicText {
        origin: p(1., y),
        field: SymbolTextField::PortLabel(port.into()),
        height: 2.,
        align: TextAlign::MiddleLeft,
        rotation_degrees: 0.,
    }
}
/// Leads from the ports to the body edges at `top` and `bottom`.
fn leads(top: f64, bottom: f64) -> [SymbolPrimitive; 2] {
    [line(0., 10., 0., top), line(0., bottom, 0., -10.)]
}
/// Designation on the left and point labels beside both leads.
fn with_labels(mut primitives: Vec<SymbolPrimitive>, x: f64) -> Vec<SymbolPrimitive> {
    primitives.extend([designation(x), port_label("2", 7.), port_label("1", -7.)]);
    primitives
}
/// One factory symbol before it becomes a definition.
struct Factory {
    key: &'static str,
    version: u32,
    name: &'static str,
    category: &'static str,
    primitives: Vec<SymbolPrimitive>,
    /// Indexes of primitives drawn dashed (actuator stems).
    dashed: Vec<usize>,
    /// Top and bottom port identifications; `None` keeps "2" and "1".
    ports: Option<(&'static str, &'static str)>,
    kind: Option<&'static str>,
    prefix: &'static str,
}
fn factories() -> Vec<Factory> {
    // IEC normally open contact: blade from the lower lead, open to the left.
    let no_contact = || {
        let [a, b] = leads(4., -4.);
        vec![a, b, line(0., -4., -4., 4.5)]
    };
    let mut push_button = no_contact();
    // Push actuator: dashed stem to the blade and the button head.
    push_button.extend([
        line(-2., 0.25, -8., 0.25),
        line(-8., -2., -8., 2.5),
        line(-8., -2., -7., -2.),
        line(-8., 2.5, -7., 2.5),
    ]);
    let mut switch = no_contact();
    // Rotary actuator.
    switch.extend([
        line(-2., 0.25, -8., 0.25),
        line(-8., -2., -8., 2.5),
        line(-8., 2.5, -6.5, 2.5),
        line(-8., -2., -9.5, -2.),
    ]);
    // IEC normally closed contact: blade resting on a hook.
    let nc_contact = {
        let [a, b] = leads(4., -4.);
        vec![a, b, line(0., 4., 3.5, 4.), line(0., -4., 3.5, 5.)]
    };
    let coil = {
        let [a, b] = leads(3., -3.);
        vec![a, rect(-5., -3., 10., 6.), b]
    };
    vec![
        Factory {
            key: "contact-no",
            version: 2,
            name: "Contato normalmente aberto",
            category: "Contatos",
            primitives: with_labels(no_contact(), -5.5),
            dashed: vec![],
            ports: Some(("13", "14")),
            kind: Some(KIND_AUXILIARY_CONTACT),
            prefix: "K",
        },
        Factory {
            key: "contact-nc",
            version: 2,
            name: "Contato normalmente fechado",
            category: "Contatos",
            primitives: with_labels(nc_contact, -2.),
            dashed: vec![],
            ports: Some(("21", "22")),
            kind: Some(KIND_AUXILIARY_CONTACT),
            prefix: "K",
        },
        Factory {
            key: "coil",
            version: 2,
            name: "Bobina de relé ou contator",
            category: "Comando",
            primitives: with_labels(coil, -6.5),
            dashed: vec![],
            ports: Some(("A1", "A2")),
            kind: Some(KIND_COIL),
            prefix: "K",
        },
        Factory {
            key: "motor",
            version: 2,
            name: "Motor elétrico",
            category: "Máquinas",
            primitives: vec![
                SymbolPrimitive::Circle {
                    center: p(0., 0.),
                    radius: 6.,
                },
                text("M"),
                line(0., -10., 0., -6.),
                line(0., 6., 0., 10.),
                designation(-7.5),
            ],
            dashed: vec![],
            ports: None,
            kind: None,
            prefix: "M",
        },
        Factory {
            key: "terminal",
            version: 2,
            name: "Borne",
            category: "Bornes",
            primitives: vec![
                SymbolPrimitive::Circle {
                    center: p(0., 0.),
                    radius: 2.,
                },
                line(0., -10., 0., -2.),
                line(0., 2., 0., 10.),
                designation(-3.5),
            ],
            dashed: vec![],
            ports: None,
            kind: None,
            prefix: "X",
        },
        Factory {
            key: "push-button",
            version: 2,
            name: "Botão de comando",
            category: "Comando",
            primitives: with_labels(push_button, -9.5),
            dashed: vec![3],
            ports: Some(("13", "14")),
            kind: Some(KIND_AUXILIARY_CONTACT),
            prefix: "S",
        },
        Factory {
            key: "switch",
            version: 2,
            name: "Chave ou interruptor",
            category: "Comando",
            primitives: with_labels(switch, -10.5),
            dashed: vec![3],
            ports: Some(("13", "14")),
            kind: Some(KIND_AUXILIARY_CONTACT),
            prefix: "S",
        },
        Factory {
            key: "sensor",
            version: 2,
            name: "Sensor",
            category: "Sensores",
            primitives: vec![
                rect(-5., -5., 10., 10.),
                line(-5., -5., 5., 5.),
                line(0., -10., 0., -5.),
                line(0., 5., 0., 10.),
                SymbolPrimitive::Arc {
                    center: p(-2., 2.),
                    radius: 1.5,
                    start_degrees: 0.,
                    sweep_degrees: 180.,
                },
                designation(-6.5),
            ],
            dashed: vec![],
            ports: None,
            kind: None,
            prefix: "B",
        },
        Factory {
            key: "protective-earth",
            version: 1,
            name: "Terra de proteção",
            category: "Potenciais",
            primitives: vec![
                line(0., 10., 0., 0.),
                line(-5., 0., 5., 0.),
                line(-3.5, -2., 3.5, -2.),
                line(-1.5, -4., 1.5, -4.),
            ],
            dashed: vec![],
            ports: None,
            kind: None,
            prefix: "PE",
        },
        Factory {
            key: "supply",
            version: 2,
            name: "Elemento de alimentação",
            category: "Potenciais",
            primitives: vec![
                rect(-5., -5., 10., 10.),
                line(0., -10., 0., -5.),
                line(0., 5., 0., 10.),
                line(-3., 1., -1., 1.),
                line(-2., 0., -2., 2.),
                line(1., -1., 3., -1.),
                designation(-6.5),
            ],
            dashed: vec![],
            ports: None,
            kind: None,
            prefix: "G",
        },
    ]
}
fn build_builtin() -> SymbolLibrary {
    let mut definitions = BTreeMap::new();
    for f in factories() {
        let ports = if f.key == "protective-earth" {
            vec![
                SymbolPort::new("pe", p(0., 10.), 90., "PE", Some("protective-earth".into()))
                    .expect("static port"),
            ]
        } else {
            // Top port first, so points are created in IEC order (13 before 14).
            let (top, bottom) = f.ports.unwrap_or(("2", "1"));
            vec![
                SymbolPort::new("2", p(0., 10.), 90., top, None).expect("static port"),
                SymbolPort::new("1", p(0., -10.), 270., bottom, None).expect("static port"),
            ]
        };
        let id = SymbolDefinitionId::new(format!("cad-craft.{}", f.key)).expect("static id");
        let mut metadata = Metadata::new();
        metadata.insert(
            Text::new("source").expect("static key"),
            "CAD-CRAFT original; referência industrial, sem certificação IEC".into(),
        );
        if f.version > 1 {
            metadata.insert(
                Text::new(METADATA_PREFIX).expect("static key"),
                f.prefix.into(),
            );
        }
        if let Some(kind) = f.kind {
            metadata.insert(
                Text::new(METADATA_FUNCTION_KIND).expect("static key"),
                kind.into(),
            );
        }
        let styles = f
            .dashed
            .iter()
            .map(|&i| {
                (
                    i,
                    PrimitiveStyle {
                        dash: LineDash::Dashed,
                        ..PrimitiveStyle::default()
                    },
                )
            })
            .collect();
        let definition = SymbolDefinition::new(
            id.clone(),
            f.version,
            f.name,
            f.category,
            f.primitives,
            ports,
        )
        .expect("valid builtin definition")
        .with_metadata(metadata)
        .with_styles(styles)
        .expect("valid builtin styles");
        definitions.insert(id, definition);
    }
    SymbolLibrary { definitions }
}
