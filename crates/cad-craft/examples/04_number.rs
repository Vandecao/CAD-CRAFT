// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Number devices and connections, then undo exactly; start with main.
use cad_craft::domain::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("Connectivity example")?;
    let device =
        project.insert_device(Device::new(project.id(), Designation::new("K9")?, "Relay")?)?;
    let function = project.insert_function(Function::new(
        project.id(),
        device,
        Designation::new("K9")?,
        FunctionKind::Coil,
    ))?;
    let a = project.insert_connection_point(ConnectionPoint::new(
        project.id(),
        function,
        DesignationPart::new("A1")?,
    ))?;
    let b = project.insert_connection_point(ConnectionPoint::new(
        project.id(),
        function,
        DesignationPart::new("A2")?,
    ))?;
    let connection = project.insert_connection(Connection::new(project.id(), a, b)?)?;
    let before = project.clone();
    let devices = project.preview_device_numbering(NumberingFilter::Project)?;
    assert_eq!(devices.rows.len(), 1);
    let device_delta = project.apply_device_numbering(&devices)?;
    assert_eq!(
        project
            .device(device)
            .expect("device exists")
            .designation()
            .to_string(),
        "-K1"
    );
    let connections = project.preview_connection_numbering()?;
    assert_eq!(connections.rows.len(), 1);
    let connection_delta = project.apply_connection_numbering(&connections)?;
    assert!(
        project
            .connection(connection)
            .expect("connection exists")
            .number()
            .is_some()
    );
    println!(
        "{} device(s), {} connection(s) numbered",
        devices.rows.len(),
        connections.rows.len()
    );
    project.restore_connection_numbering(&connection_delta, false)?;
    project.restore_device_numbering(&device_delta, false)?;
    assert_eq!(project, before);
    Ok(())
}

#[test]
fn runs_without_external_data() {
    main().expect("the synthetic example must succeed");
}
