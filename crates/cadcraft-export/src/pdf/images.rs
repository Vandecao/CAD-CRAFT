// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! PDF image embedding and compression; start with Writer::image.

use super::*;

pub(super) fn deflate(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes).expect("compressão em memória");
    encoder.finish().expect("compressão em memória")
}

/// JPEG color components from the SOF marker; `None` if unreadable.
fn jpeg_components(bytes: &[u8]) -> Option<u8> {
    let mut i = 2;
    while i + 9 < bytes.len() {
        if bytes[i] != 0xFF {
            return None;
        }
        let marker = bytes[i + 1];
        let length = usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
        if matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            return bytes.get(i + 9).copied();
        }
        i += 2 + length;
    }
    None
}

impl Writer<'_> {
    /// Writes an image once; `None` when readable content is unavailable.
    pub(super) fn image(&mut self, asset: AssetId) -> ImageEntry {
        if let Some(entry) = self.images.get(&asset) {
            return entry.clone();
        }
        let entry = self.write_image(asset);
        self.images.insert(asset, entry.clone());
        entry
    }
    pub(super) fn write_image(&mut self, asset: AssetId) -> ImageEntry {
        let image = self.project.asset(asset)?;
        let bytes = image.bytes()?;
        let name = format!("Im{}", self.images.len() + 1);
        let id = self.refs.next();
        match image.format() {
            ImageFormat::Jpeg => {
                let components = jpeg_components(bytes)?;
                let mut x = self.pdf.image_xobject(id, bytes);
                x.filter(Filter::DctDecode);
                x.width(image.width_px() as i32);
                x.height(image.height_px() as i32);
                match components {
                    1 => x.color_space().device_gray(),
                    4 => x.color_space().device_cmyk(),
                    _ => x.color_space().device_rgb(),
                };
                x.bits_per_component(8);
                x.finish();
            }
            ImageFormat::Png => {
                let decoded =
                    image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?;
                let rgba = decoded.to_rgba8();
                let (w, h) = (rgba.width() as i32, rgba.height() as i32);
                let rgb: Vec<u8> = rgba
                    .pixels()
                    .flat_map(|p| [p.0[0], p.0[1], p.0[2]])
                    .collect();
                let alpha: Vec<u8> = rgba.pixels().map(|p| p.0[3]).collect();
                let mask = alpha.iter().any(|a| *a != 255).then(|| {
                    let mask_id = self.refs.next();
                    let data = deflate(&alpha);
                    let mut m = self.pdf.image_xobject(mask_id, &data);
                    m.filter(Filter::FlateDecode);
                    m.width(w);
                    m.height(h);
                    m.color_space().device_gray();
                    m.bits_per_component(8);
                    m.finish();
                    mask_id
                });
                let data = deflate(&rgb);
                let mut x = self.pdf.image_xobject(id, &data);
                x.filter(Filter::FlateDecode);
                x.width(w);
                x.height(h);
                x.color_space().device_rgb();
                x.bits_per_component(8);
                if let Some(mask) = mask {
                    x.s_mask(mask);
                }
                x.finish();
            }
        }
        Some((name, id))
    }
}
