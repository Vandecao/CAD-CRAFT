// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
use super::*;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Dez desenhos originais de referência industrial; sem certificação normativa.
#[derive(Debug)]
pub struct SymbolLibrary {
    definitions: BTreeMap<SymbolDefinitionId, SymbolDefinition>,
}
impl SymbolLibrary {
    pub fn builtin() -> &'static Self {
        static BUILTIN: OnceLock<SymbolLibrary> = OnceLock::new();
        BUILTIN.get_or_init(build_builtin)
    }
    pub fn definitions(&self) -> impl Iterator<Item = &SymbolDefinition> {
        self.definitions.values()
    }
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
fn build_builtin() -> SymbolLibrary {
    let drawings = vec![
        (
            "contact-no",
            "Contato normalmente aberto",
            "Contatos",
            vec![
                line(0., -10., 0., -3.),
                line(-3., -3., 3., -3.),
                line(-3., 3., 3., 3.),
                line(0., 3., 0., 10.),
            ],
        ),
        (
            "contact-nc",
            "Contato normalmente fechado",
            "Contatos",
            vec![
                line(0., -10., 0., -3.),
                line(-3., -3., 3., -3.),
                line(-3., 3., 3., 3.),
                line(-4., -5., 4., 5.),
                line(0., 3., 0., 10.),
            ],
        ),
        (
            "coil",
            "Bobina de relé ou contator",
            "Comando",
            vec![
                line(0., -10., 0., -4.),
                rect(-4., -4., 8., 8.),
                line(0., 4., 0., 10.),
            ],
        ),
        (
            "motor",
            "Motor elétrico",
            "Máquinas",
            vec![
                SymbolPrimitive::Circle {
                    center: p(0., 0.),
                    radius: 6.,
                },
                text("M"),
                line(0., -10., 0., -6.),
                line(0., 6., 0., 10.),
            ],
        ),
        (
            "terminal",
            "Borne",
            "Bornes",
            vec![
                SymbolPrimitive::Circle {
                    center: p(0., 0.),
                    radius: 2.,
                },
                line(0., -10., 0., -2.),
                line(0., 2., 0., 10.),
            ],
        ),
        (
            "push-button",
            "Botão de comando",
            "Comando",
            vec![
                line(0., -10., 0., -3.),
                line(0., 3., 0., 10.),
                line(-3., -3., 3., 3.),
                line(-5., 4., -5., 8.),
                line(-8., 8., -2., 8.),
                line(-5., 4., -2., 1.),
            ],
        ),
        (
            "switch",
            "Chave ou interruptor",
            "Comando",
            vec![
                line(0., -10., 0., -3.),
                line(0., 3., 0., 10.),
                line(0., -3., 5., 4.),
                SymbolPrimitive::Circle {
                    center: p(0., -3.),
                    radius: 0.7,
                },
            ],
        ),
        (
            "sensor",
            "Sensor",
            "Sensores",
            vec![
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
            ],
        ),
        (
            "protective-earth",
            "Terra de proteção",
            "Potenciais",
            vec![
                line(0., 10., 0., 0.),
                line(-5., 0., 5., 0.),
                line(-3.5, -2., 3.5, -2.),
                line(-1.5, -4., 1.5, -4.),
            ],
        ),
        (
            "supply",
            "Elemento de alimentação",
            "Potenciais",
            vec![
                rect(-5., -5., 10., 10.),
                line(0., -10., 0., -5.),
                line(0., 5., 0., 10.),
                line(-3., 1., -1., 1.),
                line(-2., 0., -2., 2.),
                line(1., -1., 3., -1.),
            ],
        ),
    ];
    let mut definitions = BTreeMap::new();
    for (key, name, category, primitives) in drawings {
        let ports = if key == "protective-earth" {
            vec![
                SymbolPort::new("pe", p(0., 10.), 90., "PE", Some("protective-earth".into()))
                    .expect("static port"),
            ]
        } else {
            vec![
                SymbolPort::new("1", p(0., -10.), 270., "1", None).expect("static port"),
                SymbolPort::new("2", p(0., 10.), 90., "2", None).expect("static port"),
            ]
        };
        let id = SymbolDefinitionId::new(format!("cad-craft.{key}")).expect("static id");
        let mut metadata = Metadata::new();
        metadata.insert(
            Text::new("source").expect("static key"),
            "CAD-CRAFT original; referência industrial, sem certificação IEC".into(),
        );
        let definition = SymbolDefinition::new(id.clone(), 1, name, category, primitives, ports)
            .expect("valid builtin definition")
            .with_metadata(metadata);
        definitions.insert(id, definition);
    }
    SymbolLibrary { definitions }
}
