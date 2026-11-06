// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Frame image assets such as logos; start with ImageAsset. The core recognizes PNG and JPEG
//! signatures and dimensions and enforces size limits. Full decoding and texture creation belong to
//! the application. Bytes are stored outside project JSON as `assets/<id>.<ext>` in `.cadcraft`
//! files.
use super::*;
use crate::error::{DomainError, Result};
use serde::{Deserialize, Serialize};

/// Image resource limits enforced by the model.
pub const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
/// Maximum supported image width or height, in pixels.
pub const MAX_IMAGE_SIDE: u32 = 4096;

fn invalid(reason: &'static str) -> DomainError {
    DomainError::InvalidRelation(reason)
}

/// Image formats recognized from their header signatures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    /// Portable Network Graphics image.
    Png,
    /// JPEG image.
    Jpeg,
}
impl ImageFormat {
    /// Return the conventional filename extension without a dot.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

/// Read format and pixel dimensions from the header; reject unknown formats, truncated headers and
/// images exceeding limits.
pub fn inspect_image(bytes: &[u8]) -> Result<(ImageFormat, u32, u32)> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(invalid("imagem acima de 10 MB"));
    }
    let (format, w, h) = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        png_size(bytes)?
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        jpeg_size(bytes)?
    } else {
        return Err(invalid("imagem deve ser PNG ou JPEG"));
    };
    if w == 0 || h == 0 || w > MAX_IMAGE_SIDE || h > MAX_IMAGE_SIDE {
        return Err(invalid("imagem fora do limite de 4096 × 4096 px"));
    }
    Ok((format, w, h))
}
fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}
fn be16(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}
fn png_size(b: &[u8]) -> Result<(ImageFormat, u32, u32)> {
    // Signature (8) + length (4) + IHDR (4) + width (4) + height (4).
    if b.len() < 24 || &b[12..16] != b"IHDR" {
        return Err(invalid("cabeçalho PNG inválido"));
    }
    Ok((ImageFormat::Png, be32(&b[16..20]), be32(&b[20..24])))
}
fn jpeg_size(b: &[u8]) -> Result<(ImageFormat, u32, u32)> {
    let mut i = 2;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            return Err(invalid("cabeçalho JPEG inválido"));
        }
        let marker = b[i + 1];
        if marker == 0xFF {
            i += 1;
            continue;
        }
        if matches!(marker, 0x01 | 0xD0..=0xD8) {
            i += 2;
            continue;
        }
        if marker == 0xD9 || marker == 0xDA {
            break;
        }
        let len = be16(&b[i + 2..i + 4]) as usize;
        if len < 2 || i + 2 + len > b.len() {
            break;
        }
        let sof = matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC);
        if sof {
            if len < 7 {
                break;
            }
            let h = be16(&b[i + 5..i + 7]) as u32;
            let w = be16(&b[i + 7..i + 9]) as u32;
            return Ok((ImageFormat::Jpeg, w, h));
        }
        i += 2 + len;
    }
    Err(invalid("cabeçalho JPEG sem dimensões"))
}

/// Project image. JSON omits `bytes`, so technical snapshots contain metadata without content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "AssetData")]
pub struct ImageAsset {
    id: AssetId,
    format: ImageFormat,
    width_px: u32,
    height_px: u32,
    byte_len: u64,
    #[serde(skip)]
    bytes: Option<std::sync::Arc<Vec<u8>>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetData {
    id: AssetId,
    format: ImageFormat,
    width_px: u32,
    height_px: u32,
    byte_len: u64,
}
impl TryFrom<AssetData> for ImageAsset {
    type Error = DomainError;
    fn try_from(d: AssetData) -> Result<Self> {
        if d.width_px == 0
            || d.height_px == 0
            || d.width_px > MAX_IMAGE_SIDE
            || d.height_px > MAX_IMAGE_SIDE
            || d.byte_len == 0
            || d.byte_len > MAX_IMAGE_BYTES as u64
        {
            return Err(invalid("metadados de imagem inválidos"));
        }
        Ok(Self {
            id: d.id,
            format: d.format,
            width_px: d.width_px,
            height_px: d.height_px,
            byte_len: d.byte_len,
            bytes: None,
        })
    }
}
impl ImageAsset {
    /// Inspect image bytes, enforce resource limits and create an asset with fresh identity.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let (format, width_px, height_px) = inspect_image(&bytes)?;
        Ok(Self {
            id: AssetId::new(),
            format,
            width_px,
            height_px,
            byte_len: bytes.len() as u64,
            bytes: Some(std::sync::Arc::new(bytes)),
        })
    }
    /// Return this entity or definition's stable identifier.
    pub fn id(&self) -> AssetId {
        self.id
    }
    /// Return the format.
    pub fn format(&self) -> ImageFormat {
        self.format
    }
    /// Return the width px.
    pub fn width_px(&self) -> u32 {
        self.width_px
    }
    /// Return the height px.
    pub fn height_px(&self) -> u32 {
        self.height_px
    }
    /// Return the byte len.
    pub fn byte_len(&self) -> u64 {
        self.byte_len
    }
    /// Return the optional bytes; absence represents an unset value or relation.
    pub fn bytes(&self) -> Option<&[u8]> {
        self.bytes.as_deref().map(Vec::as_slice)
    }
    /// Image width divided by height.
    pub fn aspect(&self) -> f64 {
        self.width_px as f64 / self.height_px as f64
    }
    /// Whether the image content matches, for deduplication.
    pub fn same_content(&self, bytes: &[u8]) -> bool {
        self.bytes().is_some_and(|b| b == bytes)
    }
    /// Copy with a different ID for `.cadframe` import collisions.
    pub fn with_id(mut self, id: AssetId) -> Self {
        self.id = id;
        self
    }
    /// Attach file content after checking that it matches the metadata.
    pub fn attach(&mut self, bytes: Vec<u8>) -> Result<()> {
        let (format, w, h) = inspect_image(&bytes)?;
        if format != self.format
            || w != self.width_px
            || h != self.height_px
            || bytes.len() as u64 != self.byte_len
        {
            return Err(invalid("conteúdo de imagem não corresponde aos metadados"));
        }
        self.bytes = Some(std::sync::Arc::new(bytes));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        b.extend(w.to_be_bytes());
        b.extend(h.to_be_bytes());
        b.extend([8, 6, 0, 0, 0]);
        b
    }
    fn jpeg(w: u16, h: u16) -> Vec<u8> {
        let mut b = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0x4A, 0x46];
        b.extend([0xFF, 0xC0, 0x00, 0x0B, 0x08]);
        b.extend(h.to_be_bytes());
        b.extend(w.to_be_bytes());
        b.extend([0x01, 0x01, 0x11, 0x00, 0xFF, 0xD9]);
        b
    }
    #[test]
    fn headers_give_format_and_size_and_limits_apply() {
        assert_eq!(
            inspect_image(&png(300, 120)).unwrap(),
            (ImageFormat::Png, 300, 120)
        );
        assert_eq!(
            inspect_image(&jpeg(640, 480)).unwrap(),
            (ImageFormat::Jpeg, 640, 480)
        );
        assert!(inspect_image(&png(5000, 10)).is_err());
        assert!(inspect_image(&png(0, 10)).is_err());
        assert!(inspect_image(b"GIF89a....").is_err());
        assert!(inspect_image(&png(10, 10)[..20]).is_err());
        assert!(inspect_image(&[0xFF, 0xD8, 0xFF, 0xD9]).is_err());
        let mut big = png(10, 10);
        big.resize(MAX_IMAGE_BYTES + 1, 0);
        assert!(inspect_image(&big).is_err());
    }
    #[test]
    fn attach_requires_matching_content() {
        let asset = ImageAsset::from_bytes(png(30, 20)).unwrap();
        let json = serde_json::to_value(&asset).unwrap();
        assert!(json.get("bytes").is_none());
        let mut back: ImageAsset = serde_json::from_value(json).unwrap();
        assert!(back.bytes().is_none());
        assert!(back.attach(png(31, 20)).is_err());
        back.attach(png(30, 20)).unwrap();
        assert_eq!(back, asset);
    }
}
