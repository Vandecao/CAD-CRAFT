// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Connect two logical points and inspect the derived network; start with main.
// Examples present their results on standard output.
#![allow(clippy::print_stdout)]

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
    assert!(project.are_connected(a, b)?);
    let networks = project.networks();
    let network = networks
        .iter()
        .find(|n| n.connections().contains(&connection))
        .expect("a connection belongs to a derived network");
    assert!(network.points().contains(&a) && network.points().contains(&b));
    println!("{} points in the network", network.points().len());
    project.validate()?;
    Ok(())
}

#[test]
fn runs_without_external_data() {
    main().expect("the synthetic example must succeed");
}
