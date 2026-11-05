// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Electrical Project operations; start with the subject modules and delta types below.
use super::*;
pub(in crate::domain::project) mod connection_drawing;
pub(in crate::domain::project) mod connectivity;
pub(in crate::domain::project) mod function_edit;
pub(in crate::domain::project) mod identification;
pub(in crate::domain::project) mod markers;
pub(in crate::domain::project) mod numbering;
pub(in crate::domain::project) mod representations;
pub(in crate::domain::project) mod route_clearance;
pub(in crate::domain::project) mod sheet_references;
pub use function_edit::{FunctionPointDelta, FunctionPointEdit};
pub use numbering::{
    ConnectionNumberDelta, ConnectionNumberPreview, ConnectionNumberRow, ConnectionNumberingDelta,
    DeviceNumberDelta, DeviceNumberPreview, DeviceNumberRow,
};
pub use numbering::{PageNumberDelta, PageNumberPreview, PageNumberRow};
pub use representations::{RepresentationChange, RepresentationDelta};
pub use sheet_references::{
    SHEET_REFERENCE_HEIGHT, SheetReference, SheetReferenceKind, SheetReferenceSegment,
    contact_kind_label,
};
