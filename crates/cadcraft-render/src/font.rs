// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Vanderlei Braga e colaboradores do CAD-CRAFT
//! Helvetica metrics for the PDF default font with WinAnsi encoding; start
//! at `text_width`. Widths are in thousandths of font size, from public AFM metrics.

/// Capital letter height relative to font size.
pub const CAP_HEIGHT: f64 = 0.718;

/// WinAnsi code for a character; `None` if unavailable in the font.
pub fn win_ansi(c: char) -> Option<u8> {
    let code = c as u32;
    match code {
        0x20..=0x7E | 0xA0..=0xFF => Some(code as u8),
        _ => match c {
            '€' => Some(0x80),
            '‚' => Some(0x82),
            'ƒ' => Some(0x83),
            '„' => Some(0x84),
            '…' => Some(0x85),
            '†' => Some(0x86),
            '‡' => Some(0x87),
            'ˆ' => Some(0x88),
            '‰' => Some(0x89),
            'Š' => Some(0x8A),
            '‹' => Some(0x8B),
            'Œ' => Some(0x8C),
            'Ž' => Some(0x8E),
            '\u{2018}' => Some(0x91),
            '\u{2019}' => Some(0x92),
            '\u{201C}' => Some(0x93),
            '\u{201D}' => Some(0x94),
            '•' => Some(0x95),
            '–' => Some(0x96),
            '—' => Some(0x97),
            '˜' => Some(0x98),
            '™' => Some(0x99),
            'š' => Some(0x9A),
            '›' => Some(0x9B),
            'œ' => Some(0x9C),
            'ž' => Some(0x9E),
            'Ÿ' => Some(0x9F),
            _ => None,
        },
    }
}

/// WinAnsi text bytes; missing characters become `?` and are returned separately.
pub fn encode(text: &str) -> (Vec<u8>, Vec<char>) {
    let mut missing = vec![];
    let bytes = text
        .chars()
        .map(|c| {
            win_ansi(c).unwrap_or_else(|| {
                if !c.is_control() {
                    missing.push(c);
                }
                b'?'
            })
        })
        .collect();
    (bytes, missing)
}

/// WinAnsi code width in thousandths of font size.
pub fn width_of(code: u8) -> u16 {
    match code {
        0x20..=0x7E => ASCII[(code - 0x20) as usize],
        0x80..=0x9F => SPECIAL[(code - 0x80) as usize],
        0xA0..=0xFF => LATIN1[(code - 0xA0) as usize],
        _ => 278,
    }
}

/// Text width in the same units as `size`.
pub fn text_width(text: &str, size: f64) -> f64 {
    let (bytes, _) = encode(text);
    bytes.iter().map(|b| f64::from(width_of(*b))).sum::<f64>() / 1000.0 * size
}

#[rustfmt::skip]
const ASCII: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // space through /
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556,                               // 0 through 9
    278, 278, 584, 584, 584, 556, 1015,                                             // : through @
    667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833,                // A through M
    722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611,                // N through Z
    278, 278, 278, 469, 556, 333,                                                   // [ through `
    556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833,                // a through m
    556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500,                // n through z
    334, 260, 334, 584,                                                             // { through ~
];

#[rustfmt::skip]
const SPECIAL: [u16; 32] = [
    556, 278, 222, 556, 333, 1000, 556, 556, 333, 1000, 667, 333, 1000, 278, 611, 278,
    278, 222, 222, 333, 333, 350, 556, 1000, 333, 1000, 500, 333, 944, 278, 500, 667,
];

#[rustfmt::skip]
const LATIN1: [u16; 96] = [
    278, 333, 556, 556, 556, 556, 260, 556, 333, 737, 370, 556, 584, 333, 737, 333, // A0 through AF
    400, 584, 333, 333, 333, 556, 537, 278, 333, 333, 365, 556, 834, 834, 834, 611, // B0 through BF
    667, 667, 667, 667, 667, 667, 1000, 722, 667, 667, 667, 667, 278, 278, 278, 278, // C0 through CF
    722, 722, 778, 778, 778, 778, 778, 584, 778, 722, 722, 722, 722, 667, 667, 611, // D0 through DF
    556, 556, 556, 556, 556, 556, 889, 500, 556, 556, 556, 556, 278, 278, 278, 278, // E0 through EF
    556, 556, 556, 556, 556, 556, 556, 584, 611, 556, 556, 556, 556, 500, 556, 500, // F0 through FF
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accents_are_encoded_and_missing_characters_reported() {
        let (bytes, missing) = encode("Ação µ° Ω→");
        assert_eq!(&bytes[..4], &[b'A', 0xE7, 0xE3, b'o']);
        assert_eq!(missing, vec!['Ω', '→']);
        assert!((text_width("AB", 10.0) - 13.34).abs() < 1e-9);
        assert_eq!(width_of(win_ansi('—').unwrap()), 1000);
    }
}
