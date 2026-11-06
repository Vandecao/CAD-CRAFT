// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Exercise each reader with corrupt synthetic input; start with the per-format tests.
use cad_craft::{domain::Size2D, interchange::*};

#[test]
fn malformed_qet_returns_an_error_without_panicking() {
    let result =
        std::panic::catch_unwind(|| qet::import(b"<project><diagram", &qet::QetOptions::default()));
    assert!(result.expect("QET reader must not panic").is_err());
}

#[test]
fn malformed_elmt_returns_an_error_without_panicking() {
    let result = std::panic::catch_unwind(|| {
        elmt::import(b"<definition", &elmt::ElmtOptions::new("synthetic"))
    });
    assert!(result.expect("ELMT reader must not panic").is_err());
}

#[test]
fn malformed_titleblock_returns_an_error_without_panicking() {
    let result = std::panic::catch_unwind(|| {
        titleblock::import(
            b"<titleblocktemplate",
            titleblock::TitleblockOptions::new(Size2D::new(420.0, 297.0).unwrap()),
        )
    });
    assert!(result.expect("titleblock reader must not panic").is_err());
}

#[test]
fn malformed_fn1_returns_an_error_without_panicking() {
    let result = std::panic::catch_unwind(|| fn1::import(b"<Eplan", &mut |_| None));
    assert!(result.expect("FN1 reader must not panic").is_err());
}

#[test]
fn malformed_edz_returns_an_error_without_panicking() {
    let result = std::panic::catch_unwind(|| edz::import(b"PK\x03\x04\x00"));
    assert!(result.expect("EDZ reader must not panic").is_err());
}

#[test]
fn truncated_dxf_returns_an_error_without_panicking() {
    let result = std::panic::catch_unwind(|| {
        dxf::import(
            b"0\nSECTION\n2\nENTITIES\n0\nLINE\n10",
            &dxf::DxfImportOptions::new("synthetic"),
        )
    });
    assert!(result.expect("DXF reader must not panic").is_err());
}

#[test]
fn corrupt_csv_returns_an_error_without_panicking() {
    let result = std::panic::catch_unwind(|| import_parts_csv(b"invalid,header\n\xff,\0"));
    assert!(result.expect("CSV reader must not panic").is_err());
}
