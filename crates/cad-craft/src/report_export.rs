// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Exportação tabular da Etapa 23. O mesmo `ReportTable` alimenta páginas e arquivos.
use crate::domain::{ReportTable, ReportValue};
use rust_xlsxwriter::{Format, Workbook};

pub fn export_csv(table: &ReportTable) -> Result<Vec<u8>, String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(vec![0xEF, 0xBB, 0xBF]);
    writer
        .write_record(table.columns.iter().map(|c| c.title))
        .map_err(|e| e.to_string())?;
    for row in &table.rows {
        writer
            .write_record(row.cells.iter().map(|c| c.text()))
            .map_err(|e| e.to_string())?;
    }
    writer.into_inner().map_err(|e| e.to_string())
}

pub fn export_xlsx(tables: &[ReportTable]) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();
    let bold = Format::new().set_bold();
    for table in tables {
        let sheet = workbook.add_worksheet();
        sheet
            .set_name(table.kind.label())
            .map_err(|e| e.to_string())?;
        for (col, header) in table.columns.iter().enumerate() {
            let col = u16::try_from(col).map_err(|e| e.to_string())?;
            sheet
                .write_string_with_format(0, col, header.title, &bold)
                .map_err(|e| e.to_string())?;
            sheet
                .set_column_width(col, (header.width_mm / 2.5).clamp(8.0, 80.0))
                .map_err(|e| e.to_string())?;
        }
        for (row_index, row) in table.rows.iter().enumerate() {
            let row_num = u32::try_from(row_index + 1).map_err(|e| e.to_string())?;
            for (col_index, cell) in row.cells.iter().enumerate() {
                let col_num = u16::try_from(col_index).map_err(|e| e.to_string())?;
                match &cell.value {
                    ReportValue::Text(value) => {
                        sheet
                            .write_string(row_num, col_num, value)
                            .map_err(|e| e.to_string())?;
                    }
                    ReportValue::Number(value) => {
                        sheet
                            .write_number(row_num, col_num, *value)
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        if !table.columns.is_empty() {
            sheet
                .autofilter(
                    0,
                    0,
                    table.rows.len() as u32,
                    (table.columns.len() - 1) as u16,
                )
                .map_err(|e| e.to_string())?;
        }
    }
    workbook.save_to_buffer().map_err(|e| e.to_string())
}

/// Etapa 24: CSV genérico (cabeçalho + linhas), mesmo formato das listas.
pub fn export_rows_csv(headers: &[&str], rows: &[Vec<String>]) -> Result<Vec<u8>, String> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(vec![0xEF, 0xBB, 0xBF]);
    writer.write_record(headers).map_err(|e| e.to_string())?;
    for row in rows {
        writer.write_record(row).map_err(|e| e.to_string())?;
    }
    writer.into_inner().map_err(|e| e.to_string())
}

/// Etapa 24: XLSX genérico de uma aba, com cabeçalho em negrito e filtro.
pub fn export_rows_xlsx(
    sheet_name: &str,
    headers: &[&str],
    rows: &[Vec<String>],
) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();
    let bold = Format::new().set_bold();
    let sheet = workbook.add_worksheet();
    sheet.set_name(sheet_name).map_err(|e| e.to_string())?;
    for (col, header) in headers.iter().enumerate() {
        let col = u16::try_from(col).map_err(|e| e.to_string())?;
        sheet
            .write_string_with_format(0, col, *header, &bold)
            .map_err(|e| e.to_string())?;
        sheet
            .set_column_width(col, 24.0)
            .map_err(|e| e.to_string())?;
    }
    for (r, row) in rows.iter().enumerate() {
        let r = u32::try_from(r + 1).map_err(|e| e.to_string())?;
        for (c, value) in row.iter().enumerate() {
            let c = u16::try_from(c).map_err(|e| e.to_string())?;
            sheet.write_string(r, c, value).map_err(|e| e.to_string())?;
        }
    }
    if !headers.is_empty() {
        let last = u16::try_from(headers.len() - 1).map_err(|e| e.to_string())?;
        sheet
            .autofilter(0, 0, rows.len() as u32, last)
            .map_err(|e| e.to_string())?;
    }
    workbook.save_to_buffer().map_err(|e| e.to_string())
}
