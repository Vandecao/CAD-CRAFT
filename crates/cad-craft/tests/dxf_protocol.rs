// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Pin the shared DXF wire protocol; start with group encoding and malformed input.
use cadcraft_model::dxf;

#[test]
fn group_encoding_and_units_preserve_the_existing_protocol() {
    let mut text = String::new();
    dxf::write_group(&mut text, 9, "$INSUNITS").unwrap();
    dxf::write_group(&mut text, 70, "4").unwrap();
    assert_eq!(text, "  9\n$INSUNITS\n 70\n4\n");
    assert_eq!(
        dxf::parse_groups(&text).unwrap(),
        vec![(9, "$INSUNITS".into()), (70, "4".into())]
    );
    assert_eq!(dxf::unit_mm(1), Some(25.4));
    assert_eq!(dxf::unit_mm(4), Some(1.0));
    assert_eq!(dxf::unit_mm(0), None);
    assert_eq!(dxf::decode(b"\xef\xbb\xbf0\nEOF"), "0\nEOF");
    assert_eq!(dxf::decode(b"\xe9"), "é");
}

#[test]
fn malformed_groups_preserve_diagnostic_messages() {
    assert_eq!(
        dxf::parse_groups("0\n").unwrap_err().to_string(),
        "DXF truncado (valor ausente)"
    );
    assert_eq!(
        dxf::parse_groups("bad\nvalue\n").unwrap_err().to_string(),
        "DXF inválido: código de grupo 'bad'"
    );
}
