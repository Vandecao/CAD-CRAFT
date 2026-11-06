// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Verify typed operation errors preserve user diagnostics; start with the path test.
use cad_gui::{
    error::{GuiError, GuiResult},
    project_file,
};
use std::{io, path::Path};

#[test]
fn io_errors_retain_path_kind_and_legacy_display() {
    let path = Path::new("synthetic & quoted ' file.cadcraft");
    let error = GuiError::at_path(
        path,
        io::Error::new(io::ErrorKind::PermissionDenied, "synthetic denial"),
    );
    assert_eq!(
        error.to_string(),
        format!("{}: synthetic denial", path.display())
    );
    match error {
        GuiError::PathIo {
            path: actual,
            source,
        } => {
            assert_eq!(actual, path);
            assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        }
        _ => panic!("path errors must retain typed context"),
    }
}

#[test]
fn import_and_native_format_failures_keep_their_categories_and_text() {
    let import = GuiError::from(cad_craft::interchange::ImportError(
        "synthetic import".into(),
    ));
    assert!(matches!(&import, GuiError::Import(_)));
    assert_eq!(String::from(import), "synthetic import");
    let source = cad_craft::persistence::PersistenceError::NotAProject;
    let expected = source.to_string();
    let native = GuiError::from(source);
    assert!(matches!(&native, GuiError::Persistence(_)));
    assert_eq!(native.to_string(), expected);
}

#[test]
fn project_file_boundary_returns_a_typed_path_error() {
    let path = std::env::temp_dir()
        .join(format!(
            "cadcraft-missing-{}",
            cad_craft::domain::ProjectId::new()
        ))
        .join("missing.cadcraft");
    let result: GuiResult<_> = project_file::load(&path);
    match result.unwrap_err() {
        GuiError::PathIo {
            path: actual,
            source,
        } => {
            assert_eq!(actual, path);
            assert_eq!(source.kind(), io::ErrorKind::NotFound);
        }
        other => panic!("unexpected error: {other}"),
    }
}
