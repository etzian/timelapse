//! Text for the standard Helvetica font (ADR-024): WinAnsi encoding and widths.
//!
//! English and French fit WinAnsi completely. A character outside it becomes `?`;
//! Unicode isolation and direction marks (which Fluent puts around placeables) are dropped.

use unicode_normalization::UnicodeNormalization;

/// WinAnsi code of `c`, `None` for characters that are dropped, `?` when it has none.
fn winansi(c: char) -> Option<u8> {
    let code = c as u32;
    Some(match c {
        '\u{2066}'..='\u{2069}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' => return None,
        '\u{feff}' | '\u{200b}'..='\u{200d}' => return None,
        '\t' | '\n' | '\r' => b' ',
        '€' => 0x80,
        '‚' => 0x82,
        'ƒ' => 0x83,
        '„' => 0x84,
        '…' => 0x85,
        '†' => 0x86,
        '‡' => 0x87,
        'ˆ' => 0x88,
        '‰' => 0x89,
        'Š' => 0x8a,
        '‹' => 0x8b,
        'Œ' => 0x8c,
        'Ž' => 0x8e,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '˜' => 0x98,
        '™' => 0x99,
        'š' => 0x9a,
        '›' => 0x9b,
        'œ' => 0x9c,
        'ž' => 0x9e,
        'Ÿ' => 0x9f,
        '\u{202f}' => 0xa0, // narrow no-break space (fr-CA before « : ; ! ? »)
        _ if (0x20..0x7f).contains(&code) || (0xa0..=0xff).contains(&code) => code as u8,
        _ => b'?',
    })
}

/// `s` NFC-normalised (so a decomposed é is one WinAnsi byte) and encoded.
pub fn encode(s: &str) -> Vec<u8> {
    s.nfc().filter_map(winansi).collect()
}

/// Helvetica advance widths (per 1000 em) of the printable ASCII range 0x20..=0x7e.
const ASCII: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // 0x20
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, // 0x30
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, // 0x40
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, // 0x50
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, // 0x60
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584, // 0x70
];

/// Width of one WinAnsi byte, per 1000 em.
fn byte_width(b: u8) -> u16 {
    match b {
        0x20..=0x7e => ASCII[usize::from(b - 0x20)],
        0x85 | 0x89 | 0x97 | 0x99 => 1000,
        0x8c | 0xc6 => 1000,
        0x9c | 0xe6 => 889,
        0x91 | 0x92 | 0x82 => 222,
        0x93 | 0x94 | 0x84 => 333,
        0xa0 => 278,
        0xb0 => 400,
        // Latin-1 letters take the width of their base letter (é as e, Ç as C).
        0xc0..=0xff => char::from(b)
            .nfd()
            .next()
            .filter(char::is_ascii_graphic)
            .map_or(556, |c| ASCII[c as usize - 0x20]),
        _ => 556,
    }
}

/// Width of encoded text at `size` points.
pub fn width(text: &[u8], size: f32) -> f32 {
    text.iter().map(|&b| f32::from(byte_width(b))).sum::<f32>() * size / 1000.0
}

const ELLIPSIS: u8 = 0x85;

/// `text` shortened with `…` so it fits `max` points at `size`. `keep_end` keeps the end
/// (a path's file name) and cuts the start; otherwise the end is cut.
pub fn fit(text: &[u8], max: f32, size: f32, keep_end: bool) -> Vec<u8> {
    if width(text, size) <= max {
        return text.to_vec();
    }
    let room = max - width(&[ELLIPSIS], size);
    let mut used = 0.0;
    let mut kept = Vec::new();
    let mut take = |b: u8| {
        let w = f32::from(byte_width(b)) * size / 1000.0;
        if used + w > room {
            return false;
        }
        used += w;
        kept.push(b);
        true
    };
    if keep_end {
        for &b in text.iter().rev() {
            if !take(b) {
                break;
            }
        }
        kept.push(ELLIPSIS);
        kept.reverse();
    } else {
        for &b in text {
            if !take(b) {
                break;
            }
        }
        kept.push(ELLIPSIS);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn french_and_english_encode_completely() {
        assert_eq!(encode("Été à Québec, ça va"), b"\xc9t\xe9 \xe0 Qu\xe9bec, \xe7a va");
        assert_eq!(encode("l’œuvre — 5 €"), b"l\x92\x9cuvre \x97 5 \x80");
        assert_eq!(encode("Page\u{202f}:"), b"Page\xa0:");
        // Decomposed é (as macOS may store a file name) is one byte too.
        assert_eq!(encode("cafe\u{301}.jpg"), b"caf\xe9.jpg");
    }

    #[test]
    fn other_scripts_become_question_marks_and_marks_vanish() {
        assert_eq!(encode("写真.jpg"), b"??.jpg");
        assert_eq!(encode("Page \u{2068}3\u{2069} of \u{2068}9\u{2069}"), b"Page 3 of 9");
    }

    #[test]
    fn widths_follow_helvetica() {
        assert_eq!(width(b"i", 10.0), 2.22);
        assert_eq!(width(b"MM", 1.0), 1.666);
        assert_eq!(width(&encode("é"), 1.0), width(b"e", 1.0));
    }

    #[test]
    fn fit_shortens_from_the_chosen_end() {
        let t = encode("157GOPRO/G0012345.JPG");
        assert_eq!(fit(&t, 1000.0, 8.0, true), t);
        let max = width(b"G0012345.JPG", 8.0) + width(&[ELLIPSIS], 8.0);
        let end = fit(&t, max, 8.0, true);
        assert_eq!(end, [&[ELLIPSIS][..], b"G0012345.JPG"].concat());
        let start = fit(&t, max, 8.0, false);
        assert_eq!(*start.last().unwrap(), ELLIPSIS);
        assert!(width(&start, 8.0) <= max + 0.001 && width(&end, 8.0) <= max + 0.001);
    }
}
