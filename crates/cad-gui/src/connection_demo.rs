// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Demonstration project for the connection editor.
//!
//! Four function occurrences have explicitly bound ports. Two independent
//! logical connections cross at (160, 140) mm without an electrical junction.
//! This fixture is separate from the previous demonstration and creates no
//! physical conductors.
use cad_craft::{domain::*, error::Result};

pub fn project() -> Result<Project> {
    let mut project = Project::new("Conexões multifilares")?;
    let document = project.insert_document(Document::new(project.id(), "Esquema multifilar")?)?;
    let page = project.insert_page(Page::new(
        project.id(),
        document,
        "Cruzamento sem junção",
        "1",
        PageKind::MultiLine,
        Size2D::new(420.0, 297.0)?,
    )?)?;
    let definition = SymbolLibrary::builtin()
        .get(&SymbolDefinitionId::new("cad-craft.contact-no")?)
        .expect("built-in contact definition");
    let mut endpoints = Vec::new();
    for (index, (x, y)) in [(80.0, 100.0), (240.0, 180.0), (160.0, 60.0), (160.0, 220.0)]
        .into_iter()
        .enumerate()
    {
        let designation = Designation::new(format!("K{}", index + 1))?;
        let device = project.insert_device(Device::new(
            project.id(),
            designation.clone(),
            "Contato de demonstração",
        )?)?;
        let function = project.insert_function(Function::new(
            project.id(),
            device,
            designation,
            FunctionKind::Coil,
        ))?;
        // Port 1 of this definition is (0, -10) mm in local coordinates.
        let representation = Representation::new_symbol(
            project.id(),
            page,
            definition,
            Some(RepresentationTarget::Function(function)),
            Transform2D::new(Point2D::new(x, y + 10.0)?, 0.0, false, false)?,
        )?;
        let id = project.insert_symbol_representation(representation, definition.clone())?;
        for port in definition.ports() {
            let point = project.insert_connection_point(ConnectionPoint::new(
                project.id(),
                function,
                DesignationPart::new(port.id())?,
            ))?;
            project.edit_connection_drawing(ConnectionDrawingEdit::BindPort {
                port: PortRef::new(id, port.id())?,
                point: Some(point),
            })?;
        }
        endpoints.push(PortRef::new(id, "1")?);
    }
    for (source, target, coordinates) in [
        (
            0,
            1,
            vec![(80.0, 100.0), (80.0, 140.0), (240.0, 140.0), (240.0, 180.0)],
        ),
        (2, 3, vec![(160.0, 60.0), (160.0, 220.0)]),
    ] {
        let points = coordinates
            .into_iter()
            .map(|(x, y)| Point2D::new(x, y))
            .collect::<Result<Vec<_>>>()?;
        project.edit_connection_drawing(ConnectionDrawingEdit::CreateRoute {
            source: endpoints[source].clone(),
            target: endpoints[target].clone(),
            points,
        })?;
    }
    project.validate()?;
    Ok(project)
}
