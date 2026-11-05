// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Core de engenharia do CAD-CRAFT, independente da interface.
//!
//! Modelo elétrico em milímetros. O agregado [`domain::Project`] protege relações
//! entre entidades e conectividade lógica; não contém renderização. O formato
//! `.cadcraft` ([`persistence`]) trabalha sobre bytes, sem trava ou recuperação.

/// Etapa 21: lista de exibição da página (fonte do PDF e do DXF).
pub mod display;
mod document_index;
pub mod domain;
/// Etapa 22: DXF R12 de página (lista de exibição).
pub mod dxf_export;
pub mod error;
/// Etapa 21: métricas da Helvetica (WinAnsi).
pub mod font;
mod graph;
/// Etapa 15: molduras de outros sistemas (.fn1, .titleblock, DXF).
pub mod interchange;
/// Etapa 21: PDF vetorial multipágina com links e marcadores.
pub mod pdf;
pub mod persistence;
mod physical_index;
pub mod report_export;
