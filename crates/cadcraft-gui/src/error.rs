// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Typed GUI operation failures; start with GuiError and GuiResult.
use std::path::{Path, PathBuf};
use thiserror::Error;

pub type GuiResult<T> = Result<T, GuiError>;

#[derive(Debug, Error)]
pub enum GuiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{}: {source}", path.display())]
    PathIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Persistence(#[from] cad_craft::persistence::PersistenceError),
    #[error(transparent)]
    Import(#[from] cad_craft::interchange::ImportError),
    #[error(transparent)]
    Export(#[from] cad_craft::pdf::PdfError),
    #[error(transparent)]
    Domain(#[from] cad_craft::error::DomainError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    Shell(#[from] opener::OpenError),
    #[error("{0}")]
    Converter(String),
    #[error("{0}")]
    Report(String),
    #[error("{0}")]
    Message(String),
    #[error("{context}: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<GuiError>,
    },
}
impl GuiError {
    pub fn context(context: impl Into<String>, source: impl Into<GuiError>) -> Self {
        Self::Context {
            context: context.into(),
            source: Box::new(source.into()),
        }
    }
    /// Retain both the failing path and I/O source while preserving the old text.
    pub fn at_path(path: &Path, source: std::io::Error) -> Self {
        Self::PathIo {
            path: path.to_owned(),
            source,
        }
    }
    /// Search the rendered diagnostic, as supported by existing UI consumers.
    pub fn contains(&self, pattern: &str) -> bool {
        self.to_string().contains(pattern)
    }
}
impl From<String> for GuiError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}
impl From<&str> for GuiError {
    fn from(message: &str) -> Self {
        Self::Message(message.to_owned())
    }
}
impl From<GuiError> for String {
    fn from(error: GuiError) -> Self {
        error.to_string()
    }
}
