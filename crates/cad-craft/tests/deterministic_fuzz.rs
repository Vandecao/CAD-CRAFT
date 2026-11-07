// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Bounded deterministic reader fuzzing; start with the seven per-format tests.
// Synthetic fixture construction deliberately fails fast on invalid setup.
#![allow(clippy::unwrap_used)]

use cad_craft::{
    domain::{FrameTemplate, Size2D},
    interchange::{dxf, edz, elmt, fn1, qet, titleblock},
    persistence,
};
use std::io::{Cursor, Write};

const SEED: u64 = 0x28_ca_dc_af_70_11_06;
const CASES: usize = 192;
const MAX_RANDOM_BYTES: usize = 2048;

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn index(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
    fn bytes(&mut self, count: usize) -> Vec<u8> {
        (0..count).map(|_| self.next() as u8).collect()
    }
}

/// Exercise valid input first, then binary noise and mutations that keep much
/// of the grammar intact. The seed and case printed on failure reproduce it.
fn exercise(name: &str, valid: &[u8], mut reader: impl FnMut(&[u8]) -> bool) {
    assert!(reader(valid), "{name}: the synthetic seed must be valid");
    let mut random = Random(SEED);
    for case in 0..CASES {
        let mut bytes = valid.to_vec();
        match case % 6 {
            0 => {
                let length = random.index(MAX_RANDOM_BYTES + 1);
                bytes = random.bytes(length);
            }
            1 => bytes.truncate(random.index(bytes.len() + 1)),
            2 => {
                for _ in 0..1 + random.index(8) {
                    let at = random.index(bytes.len());
                    bytes[at] ^= 1 << random.index(8);
                }
            }
            3 => {
                let at = random.index(bytes.len() + 1);
                let tokens: &[&[u8]] = &[
                    b"NaN",
                    b"-inf",
                    b"1e309",
                    b"-2147483649",
                    b"4294967296",
                    b"\0\xff",
                    b"</",
                    b"&unknown;",
                    b"../",
                    b"\r\n",
                ];
                bytes.splice(at..at, tokens[random.index(tokens.len())].iter().copied());
            }
            4 => {
                let start = random.index(bytes.len());
                let end = start + random.index(bytes.len() - start + 1);
                bytes.drain(start..end);
            }
            _ => {
                let at = random.index(bytes.len());
                let end = (at + 1 + random.index(32)).min(bytes.len());
                let repeated = bytes[at..end].repeat(4);
                bytes.splice(at..end, repeated);
            }
        }
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reader(&bytes)));
        assert!(
            outcome.is_ok(),
            "{name}: seed={SEED:#x}, case={case}, bytes={bytes:?}"
        );
    }
}

fn archive(items: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in items {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

#[test]
fn native_project_deterministic_fuzz() {
    // The versioned synthetic fixture pins IDs as well as the mutation seed.
    let project =
        persistence::read_project_from_slice(include_bytes!("fixtures/etapa24-v9.cadcraft"))
            .unwrap()
            .project;
    let valid = persistence::write_project_to_vec(&project).unwrap();
    exercise("cadcraft archive", &valid, |bytes| {
        persistence::read_project(Cursor::new(bytes)).is_ok()
    });
    // Rebuild a valid ZIP after payload mutations to reach JSON and validation,
    // rather than stopping at ZIP checksums in every corrupt case.
    let json = serde_json::to_vec(&project).unwrap();
    exercise("cadcraft content", &json, |bytes| {
        let manifest = serde_json::to_vec(&persistence::Manifest::current()).unwrap();
        let package = archive(&[
            ("mimetype", persistence::MIMETYPE.as_bytes()),
            ("manifest.json", &manifest),
            ("project.json", bytes),
        ]);
        persistence::read_project(Cursor::new(package)).is_ok()
    });
}

#[test]
fn qet_deterministic_fuzz() {
    let valid = br#"<project title="Synthetic" version="0.80"><diagram order="1" title="Sheet" cols="10" colsize="60" rows="6" rowsize="80"><elements/><conductors/></diagram></project>"#;
    exercise("qet", valid, |bytes| {
        qet::import(bytes, &qet::QetOptions::default()).is_ok()
    });
}

#[test]
fn elmt_deterministic_fuzz() {
    let valid = br#"<definition type="element" width="40" height="60" hotspot_x="20" hotspot_y="30"><names><name lang="en">Synthetic</name></names><description><rect x="-10" y="-10" width="20" height="20"/></description></definition>"#;
    exercise("elmt", valid, |bytes| {
        elmt::import(bytes, &elmt::ElmtOptions::new("synthetic")).is_ok()
    });
}

#[test]
fn titleblock_deterministic_fuzz() {
    let valid = br#"<titleblocktemplate name="Synthetic"><grid cols="100px;100px" rows="40px"><field row="0" col="0"><value><translation lang="en">%{folio}</translation></value></field></grid></titleblocktemplate>"#;
    exercise("titleblock", valid, |bytes| {
        titleblock::import(
            bytes,
            titleblock::TitleblockOptions::new(Size2D::new(420.0, 297.0).unwrap()),
        )
        .is_ok()
    });
}

#[test]
fn fn1_deterministic_fuzz() {
    let valid = include_bytes!("fixtures/synthetic/fn1/moldura_completa.fn1");
    exercise("fn1", valid, |bytes| {
        fn1::import(bytes, &mut |_| None).is_ok()
    });
}

#[test]
fn edz_deterministic_fuzz() {
    let manifest = br#"<manifest version="2.0"><packages><package type="part" key="TEST.1"><items><item type="partxml" name="part" locator="test.part.xml"/></items></package></packages></manifest>"#;
    let part = br#"<partsmanagement><part P_ARTICLE_PARTNR="TEST.1" P_ARTICLE_MANUFACTURER="Synthetic" P_ARTICLE_DESCR1="en_US@Test;"/></partsmanagement>"#;
    let items = [
        ("manifest.xml", manifest.as_slice()),
        ("items/partxml/test.part.xml", part.as_slice()),
    ];
    let valid = archive(&items);
    exercise("edz zip", &valid, |bytes| edz::import(bytes).is_ok());
    let mut seven = sevenz_rust2::ArchiveWriter::new(Cursor::new(Vec::new())).unwrap();
    for (name, bytes) in items {
        seven
            .push_archive_entry(
                sevenz_rust2::ArchiveEntry::new_file(name),
                Some(Cursor::new(bytes)),
            )
            .unwrap();
    }
    let seven = seven.finish().unwrap().into_inner();
    exercise("edz sevenz", &seven, |bytes| edz::import(bytes).is_ok());
    exercise("edz part payload", part, |bytes| {
        edz::import(&archive(&[
            ("manifest.xml", manifest),
            ("items/partxml/test.part.xml", bytes),
        ]))
        .is_ok()
    });
}

#[test]
fn dxf_deterministic_fuzz() {
    let valid = dxf::export(&FrameTemplate::factory_a3()).0;
    exercise("dxf", &valid, |bytes| {
        dxf::import(bytes, &dxf::DxfImportOptions::new("Synthetic")).is_ok()
    });
}
