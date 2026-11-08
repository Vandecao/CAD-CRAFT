// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Dynamic symbol occurrence text; start with Project::symbol_text. Values are derived on
//! every drawing and never stored in the occurrence: device designation/properties, function kind,
//! bound port label and `/page.column` reference.
use super::*;

fn property_text(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Text(t) => t.clone(),
        PropertyValue::Integer(i) => i.to_string(),
        PropertyValue::Decimal(d) => d.get().to_string(),
        PropertyValue::Boolean(b) => if *b { "sim" } else { "não" }.to_owned(),
        PropertyValue::Enumeration(e) => e.selected().as_str().to_owned(),
        PropertyValue::Reference(r) => format!("{r:?}"),
    }
}

fn kind_text(kind: &FunctionKind) -> String {
    kind.label().to_owned()
}

impl Project {
    fn representation_device(&self, r: &Representation) -> Option<(&Device, Option<&Function>)> {
        match r.target() {
            RepresentationTarget::Function(f) => {
                let function = self.function(f)?;
                Some((self.device(function.device_id())?, Some(function)))
            }
            RepresentationTarget::Device(d) => Some((self.device(d)?, None)),
            _ => None,
        }
    }

    /// Field text for occurrence `id`, or `None` when absent so rendering shows the field placeholder.
    pub fn symbol_text(&self, id: RepresentationId, field: &SymbolTextField) -> Option<String> {
        let r = self.representation(id)?;
        match field {
            // A strip terminal shows strip and number: -X1:1.
            SymbolTextField::Designation => match r.target() {
                RepresentationTarget::Terminal(t) => {
                    let terminal = self.terminal(t)?;
                    let owner = match terminal.owner() {
                        TerminalOwner::Device(d) => self.device(d)?.designation().to_string(),
                        TerminalOwner::Component(c) => self.component(c)?.designation().to_string(),
                    };
                    Some(format!("{owner}:{}", terminal.designation().as_str()))
                }
                _ => self
                    .representation_device(r)
                    .map(|(d, _)| d.designation().to_string()),
            },
            SymbolTextField::FunctionKind => self
                .representation_device(r)
                .and_then(|(_, f)| f)
                .map(|f| kind_text(f.kind())),
            SymbolTextField::PortLabel(port) => {
                let port = PortRef::new(id, port.as_str()).ok()?;
                let point = self.port_binding(&port)?;
                Some(
                    self.connection_point(point)?
                        .designation()
                        .as_str()
                        .to_owned(),
                )
            }
            SymbolTextField::Reference => {
                let p = r.transform().translation();
                Some(self.reference_label(r.page_id(), p))
            }
            SymbolTextField::DeviceProperty(key) => {
                let (device, _) = self.representation_device(r)?;
                let key = Text::new(key.as_str()).ok()?;
                device.properties().custom().get(&key).map(property_text)
            }
        }
    }
}

impl Project {
    /// Readable function label for selectors: device designation, kind and
    /// point labels, such as "-K1 · Bobina (A1, A2)"; points are listed in
    /// natural order and a long run is shortened to "1…6".
    pub fn function_label(&self, id: FunctionId) -> Option<String> {
        let f = self.function(id)?;
        let device = self.device(f.device_id()).map_or_else(
            || f.designation().to_string(),
            |d| d.designation().to_string(),
        );
        let mut points: Vec<&str> = self
            .connection_points()
            .filter(|p| p.function_id() == id)
            .map(|p| p.designation().as_str())
            .collect();
        points.sort_by_key(|a| natural_key(a));
        let points = match points.as_slice() {
            [] => String::new(),
            list if list.len() > 4 => {
                format!(" ({}…{})", list[0], list[list.len() - 1])
            }
            list => format!(" ({})", list.join(", ")),
        };
        Some(format!("{device} · {}{points}", f.kind().label()))
    }
}

/// Sort key comparing the numeric part of point labels by value ("2" < "13").
fn natural_key(label: &str) -> (String, u64, String) {
    let letters: String = label.chars().take_while(|c| !c.is_ascii_digit()).collect();
    let rest = &label[letters.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let number = digits.parse().unwrap_or(0);
    (letters, number, rest[digits.len()..].to_owned())
}
