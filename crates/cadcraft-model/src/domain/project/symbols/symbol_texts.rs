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
    match kind {
        FunctionKind::Coil => "Bobina".into(),
        FunctionKind::AuxiliaryContact => "Contato auxiliar".into(),
        FunctionKind::MainContact => "Contato principal".into(),
        FunctionKind::Other(t) => t.as_str().to_owned(),
    }
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
            SymbolTextField::Designation => self
                .representation_device(r)
                .map(|(d, _)| d.designation().to_string()),
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
