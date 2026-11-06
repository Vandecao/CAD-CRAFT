// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! files support for the frame editor; start at the parent module.

use super::*;

/// Read and validate image limits, the Core header and complete decoding.
pub fn load_image_file(path: &std::path::Path) -> GuiResult<ImageAsset> {
    let meta = std::fs::metadata(path).map_err(GuiError::from)?;
    if meta.len() > MAX_IMAGE_BYTES as u64 {
        return Err("imagem acima de 10 MB".into());
    }
    let bytes = std::fs::read(path).map_err(GuiError::from)?;
    frame_view::decode_image(&bytes)?;
    ImageAsset::from_bytes(bytes).map_err(GuiError::from)
}
pub(crate) fn read_external_frame(
    path: &std::path::Path,
    page_size: Option<Size2D>,
    dxf_unit_mm: Option<f64>,
    dxf_sheet: Option<Size2D>,
) -> GuiResult<cad_craft::interchange::ImportedFrame> {
    use cad_craft::interchange::{self, SourceFormat};
    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(SourceFormat::from_extension)
        .ok_or_else(|| "Extensão de moldura não suportada".to_owned())?;
    let metadata = std::fs::metadata(path).map_err(GuiError::from)?;
    if metadata.len() > interchange::MAX_IMPORT_BYTES as u64 {
        return Err("arquivo acima de 64 MB".into());
    }
    let bytes = std::fs::read(path).map_err(GuiError::from)?;
    let imported = match format {
        SourceFormat::EplanFn1 => {
            let mut resolve = |name: &str| -> Option<Vec<u8>> {
                let filename = std::path::Path::new(name.replace('\\', "/").as_str())
                    .file_name()?
                    .to_owned();
                let company = path.parent()?.file_name()?;
                let data = path.parent()?.parent()?.parent()?;
                let image = data.join("Imagens").join(company).join(filename);
                (image.metadata().ok()?.len() <= MAX_IMAGE_BYTES as u64)
                    .then(|| std::fs::read(image).ok())
                    .flatten()
            };
            interchange::fn1::import(&bytes, &mut resolve)
        }
        SourceFormat::QetTitleblock => interchange::titleblock::import(
            &bytes,
            interchange::titleblock::TitleblockOptions::new(
                page_size.unwrap_or_else(|| PaperFormat::A3.size(true)),
            ),
        ),
        SourceFormat::Dxf => {
            let mut options = interchange::dxf::DxfImportOptions::new(
                path.file_stem().unwrap_or_default().to_string_lossy(),
            );
            options.unit_mm = dxf_unit_mm;
            options.sheet = dxf_sheet;
            interchange::dxf::import(&bytes, &options)
        }
    }
    .map_err(GuiError::from)?;
    for asset in &imported.package.assets {
        frame_view::decode_image(asset.bytes().unwrap_or_default())?;
    }
    Ok(imported)
}

pub(in crate::frame_editor) fn with_frame_extension(
    path: std::path::PathBuf,
) -> std::path::PathBuf {
    let ext = cad_craft::persistence::FRAME_EXTENSION;
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(".");
        name.push(ext);
        name.into()
    }
}
pub(in crate::frame_editor) fn read_frame_file(path: &std::path::Path) -> GuiResult<FramePackage> {
    let meta = std::fs::metadata(path).map_err(GuiError::from)?;
    if meta.len() > 256 * 1024 * 1024 {
        return Err("arquivo de moldura acima de 256 MiB".into());
    }
    let bytes = std::fs::read(path).map_err(GuiError::from)?;
    cad_craft::persistence::read_frame_package(&bytes).map_err(GuiError::from)
}
