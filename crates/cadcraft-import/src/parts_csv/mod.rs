// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Part catalog CSV and BOM output; start with import_parts_csv and CsvPartReport.
use cadcraft_model::domain::{BomRow, Part, PartFunction, TechnicalProperties};
use std::collections::{BTreeMap, BTreeSet};

const HEADER: [&str; 8] = [
    "id",
    "manufacturer",
    "code",
    "description",
    "classification",
    "translations",
    "properties",
    "functions",
];
const MAX_CSV_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CsvRejectedRow {
    pub line: usize,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CsvPartReport {
    pub parts: Vec<Part>,
    pub rejected: Vec<CsvRejectedRow>,
}

pub fn export_parts_csv(parts: &[Part]) -> Result<Vec<u8>, String> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer.write_record(HEADER).map_err(|e| e.to_string())?;
    for part in parts {
        let translations = serde_json::to_string(part.translations()).map_err(|e| e.to_string())?;
        let properties = serde_json::to_string(part.properties()).map_err(|e| e.to_string())?;
        let functions = serde_json::to_string(part.functions()).map_err(|e| e.to_string())?;
        writer
            .write_record([
                part.id().as_str(),
                part.manufacturer().as_str(),
                part.code().as_str(),
                part.description().as_str(),
                part.classification().as_str(),
                &translations,
                &properties,
                &functions,
            ])
            .map_err(|e| e.to_string())?;
    }
    writer.into_inner().map_err(|e| e.to_string())
}

pub fn import_parts_csv(bytes: &[u8]) -> Result<CsvPartReport, String> {
    // Read and validate the header before converting individual rows.
    if bytes.len() > MAX_CSV_BYTES {
        return Err("CSV excede limite de 32 MiB".into());
    }
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(bytes);
    let header = reader.headers().map_err(|e| e.to_string())?;
    if header.iter().collect::<Vec<_>>() != HEADER {
        return Err("cabeçalho CSV inválido".into());
    }
    let mut report = CsvPartReport {
        parts: Vec::new(),
        rejected: Vec::new(),
    };
    let mut ids = BTreeSet::new();
    for (index, row) in reader.records().enumerate() {
        let line = index + 2;
        // Convert one raw record through the neutral model validation.
        let parsed = (|| -> Result<Part, String> {
            let row = row.map_err(|e| e.to_string())?;
            if row.len() != HEADER.len() {
                return Err("número de colunas inválido".into());
            }
            let mut part = Part::new(&row[0], &row[1], &row[2], &row[3], &row[4])
                .map_err(|e| e.to_string())?;
            let translations: BTreeMap<String, String> =
                serde_json::from_str(&row[5]).map_err(|e| e.to_string())?;
            for (language, description) in translations {
                part = part
                    .with_translation(language, description)
                    .map_err(|e| e.to_string())?;
            }
            let properties: TechnicalProperties =
                serde_json::from_str(&row[6]).map_err(|e| e.to_string())?;
            let functions: Vec<PartFunction> =
                serde_json::from_str(&row[7]).map_err(|e| e.to_string())?;
            part.with_functions(functions)
                .map(|part| part.with_properties(properties))
                .map_err(|e| e.to_string())
        })();
        // Report accepted rows, duplicate identities and conversion failures in order.
        match parsed {
            Ok(part) if ids.insert(part.id().clone()) => report.parts.push(part),
            Ok(_) => report.rejected.push(CsvRejectedRow {
                line,
                reason: "ID de peça duplicado".into(),
            }),
            Err(reason) => report.rejected.push(CsvRejectedRow { line, reason }),
        }
    }
    Ok(report)
}

pub fn export_bom_csv(rows: &[BomRow]) -> Result<Vec<u8>, String> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer
        .write_record([
            "device",
            "part_id",
            "manufacturer",
            "code",
            "quantity",
            "description",
        ])
        .map_err(|e| e.to_string())?;
    for row in rows {
        writer
            .write_record([
                row.device_designation.clone(),
                row.part_id.as_str().to_owned(),
                row.manufacturer.clone(),
                row.code.clone(),
                row.quantity.to_string(),
                row.description.clone(),
            ])
            .map_err(|e| e.to_string())?;
    }
    writer.into_inner().map_err(|e| e.to_string())
}
