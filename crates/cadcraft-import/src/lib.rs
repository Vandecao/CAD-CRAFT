// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! External file conversion into the neutral model; start with ImportReport,
//! then the qet, elmt, edz or frame reader modules for the source format.
//!
//! Frame readers handle FN1, titleblock and ASCII DXF; dxf also writes frames.
//! QET imports projects, ELMT imports symbols, EDZ imports parts and 2D macros,
//! and the re-exported CSV functions exchange parts and bill-of-materials rows.
//! Conversions report ignored and approximated content. Importers return model
//! data for callers to accept through validated project edits or library storage.
use cadcraft_model::domain::{FramePackage, SpecialField, Text};
use std::fmt;

mod build;
pub mod dxf;
pub mod edz;
pub mod elmt;
mod parts_csv;
pub use parts_csv::{
    CsvPartReport, CsvRejectedRow, export_bom_csv, export_parts_csv, import_parts_csv,
};
pub mod fn1;
/// Etapa 25: projetos do QElectroTech (`.qet`).
pub mod qet;
pub mod titleblock;
mod xml;

/// Limite de tamanho dos arquivos de moldura importados.
pub const MAX_IMPORT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError(pub String);
impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ImportError {}
pub(crate) fn fail(message: impl Into<String>) -> ImportError {
    ImportError(message.into())
}
pub type ImportResult<T> = std::result::Result<T, ImportError>;

/// O que aconteceu com itens que não viraram elementos equivalentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    /// Não convertido (o motivo está na mensagem).
    Ignored,
    /// Convertido de forma aproximada.
    Approximated,
    /// Informação sobre a conversão.
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportNote {
    pub kind: NoteKind,
    pub message: String,
}

/// Resumo da conversão; notas iguais são agrupadas com contagem.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub notes: Vec<(ImportNote, usize)>,
}
impl ImportReport {
    pub fn note(&mut self, kind: NoteKind, message: impl Into<String>) {
        let note = ImportNote {
            kind,
            message: message.into(),
        };
        if let Some((_, n)) = self.notes.iter_mut().find(|(m, _)| *m == note) {
            *n += 1;
        } else {
            self.notes.push((note, 1));
        }
    }
    pub fn ignored(&mut self, message: impl Into<String>) {
        self.note(NoteKind::Ignored, message);
    }
    pub fn approximated(&mut self, message: impl Into<String>) {
        self.note(NoteKind::Approximated, message);
    }
    pub fn info(&mut self, message: impl Into<String>) {
        self.note(NoteKind::Info, message);
    }
    pub fn count(&self, kind: NoteKind) -> usize {
        self.notes
            .iter()
            .filter(|(n, _)| n.kind == kind)
            .map(|(_, c)| c)
            .sum()
    }
    /// Linhas legíveis ("Ignorado (3×): …").
    pub fn lines(&self) -> Vec<String> {
        self.notes
            .iter()
            .map(|(n, c)| {
                let kind = match n.kind {
                    NoteKind::Ignored => "Ignorado",
                    NoteKind::Approximated => "Aproximado",
                    NoteKind::Info => "Informação",
                };
                if *c > 1 {
                    format!("{kind} ({c}×): {}", n.message)
                } else {
                    format!("{kind}: {}", n.message)
                }
            })
            .collect()
    }
}

/// Moldura convertida, pronta para `Project::frame_import_edits`.
#[derive(Debug, Clone)]
pub struct ImportedFrame {
    pub package: FramePackage,
    pub report: ImportReport,
}

/// Formatos de origem reconhecidos pela extensão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    EplanFn1,
    QetTitleblock,
    Dxf,
}
impl SourceFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "fn1" => Some(Self::EplanFn1),
            "titleblock" => Some(Self::QetTitleblock),
            "dxf" => Some(Self::Dxf),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::EplanFn1 => "Moldura EPLAN (.fn1)",
            Self::QetTitleblock => "Carimbo QElectroTech (.titleblock)",
            Self::Dxf => "Desenho DXF",
        }
    }
}

pub(crate) fn check_size(bytes: &[u8]) -> ImportResult<()> {
    if bytes.len() > MAX_IMPORT_BYTES {
        return Err(fail("arquivo acima de 64 MB"));
    }
    if bytes.is_empty() {
        return Err(fail("arquivo vazio"));
    }
    Ok(())
}

/// Código textual estável de um campo especial, usado nos marcadores
/// `<<código>>` do DXF (`<<pagina:Revisão#2>>` = chave com índice 2).
pub fn special_code(field: &SpecialField, index: Option<u32>) -> String {
    let base = match field {
        SpecialField::ProjectName => "projeto.nome".to_owned(),
        SpecialField::ProjectDescription => "projeto.descricao".to_owned(),
        SpecialField::ProjectIdentification => "projeto.identificacao".to_owned(),
        SpecialField::ProjectProperty(k) => format!("projeto:{}", k.as_str()),
        SpecialField::DocumentName => "documento".to_owned(),
        SpecialField::PageNumber => "pagina.numero".to_owned(),
        SpecialField::PageName => "pagina.descricao".to_owned(),
        SpecialField::PageFunctionalPath => "pagina.funcao".to_owned(),
        SpecialField::PageLocationPath => "pagina.local".to_owned(),
        SpecialField::PagePosition => "pagina.posicao".to_owned(),
        SpecialField::PageCountInDocument => "documento.paginas".to_owned(),
        SpecialField::PageCountInProject => "projeto.paginas".to_owned(),
        SpecialField::PreviousPageNumber => "pagina.anterior".to_owned(),
        SpecialField::NextPageNumber => "pagina.seguinte".to_owned(),
        SpecialField::PageProperty(k) => format!("pagina:{}", k.as_str()),
        SpecialField::FrameName => "moldura.nome".to_owned(),
    };
    match index {
        Some(i) => format!("{base}#{i}"),
        None => base,
    }
}

/// Inverso de [`special_code`]; `None` para código desconhecido.
pub fn parse_special_code(code: &str) -> Option<(SpecialField, Option<u32>)> {
    let code = code.trim();
    let (base, index) = match code.rsplit_once('#') {
        Some((b, i)) => (b, Some(i.trim().parse::<u32>().ok().filter(|i| *i > 0)?)),
        None => (code, None),
    };
    let key = |k: &str| Text::new(k.trim()).ok();
    let field = if let Some(k) = base.strip_prefix("projeto:") {
        SpecialField::ProjectProperty(key(k)?)
    } else if let Some(k) = base.strip_prefix("pagina:") {
        SpecialField::PageProperty(key(k)?)
    } else {
        match base.to_lowercase().as_str() {
            "projeto.nome" => SpecialField::ProjectName,
            "projeto.descricao" => SpecialField::ProjectDescription,
            "projeto.identificacao" => SpecialField::ProjectIdentification,
            "documento" => SpecialField::DocumentName,
            "pagina.numero" => SpecialField::PageNumber,
            "pagina.descricao" => SpecialField::PageName,
            "pagina.funcao" => SpecialField::PageFunctionalPath,
            "pagina.local" => SpecialField::PageLocationPath,
            "pagina.posicao" => SpecialField::PagePosition,
            "documento.paginas" => SpecialField::PageCountInDocument,
            "projeto.paginas" => SpecialField::PageCountInProject,
            "pagina.anterior" => SpecialField::PreviousPageNumber,
            "pagina.seguinte" => SpecialField::NextPageNumber,
            "moldura.nome" => SpecialField::FrameName,
            _ => return None,
        }
    };
    Some((field, index))
}

/// Variável do QElectroTech (`%{nome}`) → campo especial.
pub(crate) fn qet_variable(name: &str) -> SpecialField {
    let page = |k: &str| SpecialField::PageProperty(Text::new(k).expect("chave fixa"));
    let project = |k: &str| SpecialField::ProjectProperty(Text::new(k).expect("chave fixa"));
    match name.trim() {
        "title" => SpecialField::PageName,
        "projecttitle" => SpecialField::ProjectName,
        "author" => project("Autor"),
        "date" => page("Data"),
        "folio" => SpecialField::PageNumber,
        "folio-id" => SpecialField::PagePosition,
        "folio-total" => SpecialField::PageCountInProject,
        "previous-folio-num" => SpecialField::PreviousPageNumber,
        "next-folio-num" => SpecialField::NextPageNumber,
        "plant" => SpecialField::PageFunctionalPath,
        "locmach" => SpecialField::PageLocationPath,
        "indexrev" => page("Revisão"),
        "filename" => project("Arquivo"),
        other => match Text::new(format!("QET.{other}")) {
            Ok(k) => SpecialField::PageProperty(k),
            Err(_) => page("QET"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_codes_round_trip() {
        let fields = [
            (SpecialField::ProjectName, None),
            (SpecialField::PageNumber, None),
            (
                SpecialField::PageProperty(Text::new("Revisão").unwrap()),
                Some(2),
            ),
            (
                SpecialField::ProjectProperty(Text::new("Autor").unwrap()),
                None,
            ),
            (SpecialField::FrameName, None),
        ];
        for (f, i) in fields {
            let code = special_code(&f, i);
            assert_eq!(parse_special_code(&code), Some((f, i)), "{code}");
        }
        assert_eq!(parse_special_code("desconhecido"), None);
        assert_eq!(parse_special_code("pagina.numero#0"), None);
    }
}

/// Base64 padrão (ignora espaços e quebras de linha); `None` se inválido.
pub(crate) fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let value = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let clean: Vec<u8> = text.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    let body = clean
        .strip_suffix(b"==")
        .or_else(|| clean.strip_suffix(b"="))
        .unwrap_or(&clean);
    let mut out = Vec::with_capacity(body.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for &c in body {
        acc = (acc << 6) | value(c)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

#[cfg(test)]
mod base64_tests {
    #[test]
    fn decodes_standard_base64() {
        assert_eq!(super::base64_decode("TWFu\nTWE=").unwrap(), b"ManMa");
        assert_eq!(super::base64_decode("TQ==").unwrap(), b"M");
        assert!(super::base64_decode("T$==").is_none());
    }
}
