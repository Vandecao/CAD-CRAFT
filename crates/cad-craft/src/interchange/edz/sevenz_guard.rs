// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Preflight dos recursos que o leitor 7z aloca antes de expor o catálogo.
use super::EdzLimits;
use crate::interchange::{ImportResult, fail};

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
struct Header<'a>(&'a [u8]);
impl<'a> Header<'a> {
    fn take(&mut self, n: usize) -> ImportResult<&'a [u8]> {
        if n > self.0.len() {
            return Err(fail("cabeçalho 7z truncado"));
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }
    fn byte(&mut self) -> ImportResult<u8> {
        Ok(self.take(1)?[0])
    }
    fn expect(&mut self, value: u8) -> ImportResult<()> {
        if self.byte()? != value {
            return Err(fail("estrutura de cabeçalho 7z não suportada"));
        }
        Ok(())
    }
    fn number(&mut self) -> ImportResult<u64> {
        let first = self.byte()?;
        let mut value = 0u64;
        for i in 0..8 {
            let mask = 0x80 >> i;
            if first & mask == 0 {
                return Ok(value | (((first & (mask - 1)) as u64) << (8 * i)));
            }
            value |= (self.byte()? as u64) << (8 * i);
        }
        Ok(value)
    }
    // PackInfo e UnpackInfo podem conter CRC opcional. O subconjunto
    // admitido tem exatamente um stream/bloco, portanto um único bit/CRC.
    fn end_with_optional_crc(&mut self) -> ImportResult<()> {
        let mut next = self.byte()?;
        if next == 10 {
            let defined = match self.byte()? {
                0 => self.byte()? & 0x80 != 0,
                1 => true,
                _ => return Err(fail("CRC de bloco 7z inválido")),
            };
            if defined {
                self.take(4)?;
            }
            next = self.byte()?;
        }
        if next != 0 {
            return Err(fail("fim de bloco 7z inválido"));
        }
        Ok(())
    }
}
fn dictionary(method: &[u8], props: &[u8]) -> ImportResult<u64> {
    match method {
        [0] => Ok(0),
        [3, 1, 1] if props.len() == 5 && props[0] < 225 => {
            Ok(u32::from_le_bytes(props[1..5].try_into().unwrap()) as u64)
        }
        [0x21] if props.len() == 1 && props[0] <= 40 => {
            let p = props[0] as u32;
            Ok(if p == 40 {
                u32::MAX as u64
            } else {
                ((2 | (p & 1)) as u64) << (p / 2 + 11)
            })
        }
        // Filtros de estado pequeno; não incluem o decodificador BCJ2 multistream.
        [3] if props.len() == 1 => Ok(256),
        [0x0a]
        | [0x0b]
        | [3, 3, 1, 3]
        | [3, 3, 2, 5]
        | [3, 3, 4, 1]
        | [3, 3, 5, 1]
        | [3, 3, 7, 1]
        | [3, 3, 8, 5]
            if props.is_empty() || props.len() == 4 =>
        {
            Ok(4096)
        }
        _ => Err(fail(
            "codec ou propriedades 7z não suportados pelo limite de memória",
        )),
    }
}
fn memory_limit(limits: &EdzLimits) -> u64 {
    limits.max_total_bytes.min(64 * 1024 * 1024)
}

/// Executar antes de ArchiveReader::new: este já descomprime o cabeçalho.
pub(super) fn preflight(bytes: &[u8], limits: &EdzLimits) -> ImportResult<()> {
    if bytes.len() < 32 || !bytes.starts_with(b"7z\xbc\xaf\x27\x1c") {
        return Err(fail("cabeçalho 7z inválido ou truncado"));
    }
    // Impede a recuperação heurística da crate de abrir outro cabeçalho sem preflight.
    if crc32(&bytes[12..32]) != u32::from_le_bytes(bytes[8..12].try_into().unwrap()) {
        return Err(fail("CRC do cabeçalho inicial 7z inválido"));
    }
    let offset = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
    let size = u64::from_le_bytes(bytes[20..28].try_into().unwrap());
    let metadata_limit = limits
        .max_item_bytes
        .min(limits.max_total_bytes)
        .min(4 * 1024 * 1024);
    let start = offset
        .checked_add(32)
        .ok_or_else(|| fail("offset 7z excedido"))?;
    let end = start
        .checked_add(size)
        .filter(|&n| n <= bytes.len() as u64)
        .ok_or_else(|| fail("cabeçalho 7z fora do arquivo"))?;
    if size > metadata_limit {
        return Err(fail("cabeçalho 7z acima do limite"));
    }
    let raw = &bytes[start as usize..end as usize];
    if crc32(raw) != u32::from_le_bytes(bytes[28..32].try_into().unwrap()) {
        return Err(fail("CRC do cabeçalho 7z inválido"));
    }
    let mut h = Header(raw);
    match h.byte()? {
        1 => return Ok(()), // Metadados sem compressão: limitados acima.
        0x17 => {}
        _ => return Err(fail("tipo de cabeçalho 7z não suportado")),
    }
    // Subconjunto observado: um stream e um codec simples no cabeçalho comprimido.
    h.expect(6)?;
    let pack_pos = h.number()?;
    if h.number()? != 1 {
        return Err(fail("streams de cabeçalho 7z não suportados"));
    }
    h.expect(9)?;
    let packed_size = h.number()?;
    if pack_pos
        .checked_add(32)
        .and_then(|n| n.checked_add(packed_size))
        .is_none_or(|n| n > start)
    {
        return Err(fail("stream do cabeçalho 7z fora do arquivo"));
    }
    h.end_with_optional_crc()?;
    h.expect(7)?;
    h.expect(11)?;
    if h.number()? != 1 {
        return Err(fail("blocos de cabeçalho 7z não suportados"));
    }
    h.expect(0)?; // Sem referência externa.
    if h.number()? != 1 {
        return Err(fail("cadeia de codecs de cabeçalho 7z não suportada"));
    }
    let flags = h.byte()?;
    if flags & 0xd0 != 0 || flags & 15 == 0 {
        return Err(fail("codec complexo de cabeçalho 7z recusado"));
    }
    let method = h.take((flags & 15) as usize)?;
    let props = if flags & 0x20 != 0 {
        let n = h.number()?;
        if n > 16 {
            return Err(fail("propriedades de codec 7z acima do limite"));
        }
        h.take(n as usize)?
    } else {
        &[]
    };
    if !matches!(method, [0] | [3, 1, 1] | [0x21])
        || dictionary(method, props)? > memory_limit(limits)
    {
        return Err(fail("memória do codec de cabeçalho 7z acima do limite"));
    }
    h.expect(12)?;
    if h.number()? > metadata_limit {
        return Err(fail("cabeçalho 7z descomprimido acima do limite"));
    }
    h.end_with_optional_crc()?;
    // O writer também pode guardar o CRC do único stream em SubStreamsInfo,
    // em vez de UnpackInfo. Não há tamanhos adicionais com apenas um stream.
    if h.0.first() == Some(&8) {
        h.byte()?;
        if h.0.first() == Some(&13) {
            h.byte()?;
            if h.number()? != 1 {
                return Err(fail("substreams de cabeçalho 7z não suportados"));
            }
        }
        h.end_with_optional_crc()?;
    }
    h.expect(0)?;
    if !h.0.is_empty() {
        return Err(fail("dados extras no cabeçalho 7z"));
    }
    Ok(())
}

/// Limita memória dos codecs de dados antes de construir os decodificadores.
pub(super) fn post_archive(
    archive: &sevenz_rust2::Archive,
    limits: &EdzLimits,
) -> ImportResult<()> {
    for block in &archive.blocks {
        let mut memory = 0u64;
        for coder in &block.coders {
            let size = dictionary(coder.encoder_method_id(), coder.properties())?;
            memory = memory
                .checked_add(size)
                .ok_or_else(|| fail("memória 7z excedida"))?;
            if memory > memory_limit(limits) {
                return Err(fail("memória dos codecs 7z acima do limite"));
            }
            if block.get_unpack_size_for_coder(coder) > limits.max_total_bytes {
                return Err(fail("stream intermediário 7z acima do limite"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive(header: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 32];
        bytes[..6].copy_from_slice(b"7z\xbc\xaf\x27\x1c");
        bytes[20..28].copy_from_slice(&(header.len() as u64).to_le_bytes());
        bytes[28..32].copy_from_slice(&crc32(header).to_le_bytes());
        let crc = crc32(&bytes[12..32]);
        bytes[8..12].copy_from_slice(&crc.to_le_bytes());
        bytes.extend_from_slice(header);
        bytes
    }
    fn encoded(prop: u8, size: u8) -> Vec<u8> {
        // Single empty packed stream, single simple LZMA2 coder.
        archive(&[
            0x17, 6, 0, 1, 9, 0, 0, 7, 11, 1, 0, 1, 0x21, 0x21, 1, prop, 12, size, 0, 0,
        ])
    }
    #[test]
    fn refuses_encoded_header_size_and_decoder_dictionary_before_reading() {
        let limits = EdzLimits {
            max_item_bytes: 64,
            ..Default::default()
        };
        assert!(preflight(&encoded(0, 65), &limits).is_err());
        assert!(preflight(&encoded(40, 1), &EdzLimits::default()).is_err());
        assert!(preflight(&encoded(0, 1), &EdzLimits::default()).is_ok());
    }
    #[test]
    fn refuses_offsets_truncation_unknown_header_and_packed_overflow() {
        let mut bytes = archive(&[1, 0]);
        bytes[12..20].copy_from_slice(&u64::MAX.to_le_bytes());
        let crc = crc32(&bytes[12..32]);
        bytes[8..12].copy_from_slice(&crc.to_le_bytes());
        assert!(preflight(&bytes, &EdzLimits::default()).is_err());
        assert!(preflight(&[0; 12], &EdzLimits::default()).is_err());
        assert!(preflight(&archive(&[99]), &EdzLimits::default()).is_err());
        let mut bytes = encoded(0, 1);
        bytes[34] = 127; // packPos beyond physical input
        let crc = crc32(&bytes[32..]);
        bytes[28..32].copy_from_slice(&crc.to_le_bytes());
        let crc = crc32(&bytes[12..32]);
        bytes[8..12].copy_from_slice(&crc.to_le_bytes());
        assert!(preflight(&bytes, &EdzLimits::default()).is_err());
    }
    #[test]
    fn checks_lzma_properties_and_dictionary_encoding() {
        assert!(dictionary(&[3, 1, 1], &[0x5d, 0, 0, 0, 64]).is_ok());
        assert!(dictionary(&[3, 1, 1], &[0x5d]).is_err());
        assert!(dictionary(&[0x21], &[41]).is_err());
        assert!(dictionary(&[0x99], &[]).is_err());
        assert_eq!(dictionary(&[0x21], &[24]).unwrap(), 16 * 1024 * 1024);
    }
    #[test]
    fn rejects_data_dictionary_budget_without_decoding_payload() {
        use std::io::Cursor;
        let mut writer = sevenz_rust2::ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
        writer
            .push_archive_entry(
                sevenz_rust2::ArchiveEntry::new_file("test"),
                Some(Cursor::new(b"data")),
            )
            .unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let archive =
            sevenz_rust2::ArchiveReader::new(Cursor::new(&bytes), sevenz_rust2::Password::empty())
                .unwrap();
        assert!(preflight(&bytes, &EdzLimits::default()).is_ok());
        assert!(post_archive(archive.archive(), &EdzLimits::default()).is_ok());
        let limits = EdzLimits {
            max_total_bytes: 1024,
            ..Default::default()
        };
        assert!(post_archive(archive.archive(), &limits).is_err());
    }
    #[test]
    fn rejects_corrupt_crc_and_all_truncated_descriptor_prefixes() {
        let bytes = encoded(0, 1);
        for size in 0..bytes.len() {
            assert!(preflight(&bytes[..size], &EdzLimits::default()).is_err());
        }
        let mut corrupt = bytes;
        corrupt[8] ^= 1;
        assert!(preflight(&corrupt, &EdzLimits::default()).is_err());
    }
    #[test]
    fn accepts_writer_encoded_header_with_packed_stream_crc() {
        use std::io::Cursor;
        let mut writer = sevenz_rust2::ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
        for index in 0..5 {
            writer
                .push_archive_entry(
                    sevenz_rust2::ArchiveEntry::new_file(&format!(
                        "items/partxml/long-filename-{index}.xml"
                    )),
                    Some(Cursor::new(b"synthetic XML data")),
                )
                .unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();
        let offset = u64::from_le_bytes(bytes[12..20].try_into().unwrap()) as usize;
        let header = &bytes[32 + offset..];
        assert_eq!(header[0], 0x17, "fixture must exercise encoded header");
        assert!(
            preflight(&bytes, &EdzLimits::default()).is_ok(),
            "{header:02x?}"
        );
    }
    #[test]
    fn reads_optional_pack_crc_defined_bitset_and_refuses_truncated_crc() {
        let bytes = encoded(0, 1);
        for crc in [
            &[10, 1, 0, 0, 0, 0][..],
            &[10, 0, 0x80, 0, 0, 0, 0],
            &[10, 0, 0],
        ] {
            let mut header = bytes[32..].to_vec();
            header.splice(6..6, crc.iter().copied());
            assert!(preflight(&archive(&header), &EdzLimits::default()).is_ok());
        }
        let mut header = bytes[32..].to_vec();
        header.splice(6.., [10, 1, 0]);
        assert!(preflight(&archive(&header), &EdzLimits::default()).is_err());
    }
}
