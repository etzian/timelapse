//! EXIF capture-time tags, read with `kamadak-exif` (ADR-019; JPEG/TIFF only).
//!
//! Only the timestamp tags are read: `DateTimeOriginal` / `DateTimeDigitized`, their
//! `SubSecTime*` companions (burst shots often differ only there) and their
//! `OffsetTime*` companions (EXIF 2.31 time-zone offsets). The raw text is kept
//! verbatim; decoding never "fixes" a value.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// `::exif` is the `kamadak-exif` crate (its library name), not this module.
use ::exif::{DateTime, Exif, In, Reader, Tag, Value};

use super::{RawStamp, WallTime};

/// Extensions whose EXIF layout ADR-019 covers. Anything else (PNG, HEIC…) gets no
/// EXIF candidates rather than a reader we have not validated.
const EXIF_EXTENSIONS: &[&str] = &["jpg", "jpeg", "tif", "tiff"];

/// `(DateTimeOriginal, DateTimeDigitized)` for `path`; each is `None` when the file is
/// not a supported format, has no readable EXIF, or lacks the tag.
pub fn read(path: &Path) -> (Option<RawStamp>, Option<RawStamp>) {
    let supported = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXIF_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()));
    if !supported {
        return (None, None);
    }
    let Some(exif) = open(path) else {
        return (None, None);
    };
    (
        stamp(&exif, Tag::DateTimeOriginal, Tag::SubSecTimeOriginal, Tag::OffsetTimeOriginal),
        stamp(&exif, Tag::DateTimeDigitized, Tag::SubSecTimeDigitized, Tag::OffsetTimeDigitized),
    )
}

fn open(path: &Path) -> Option<Exif> {
    let file = File::open(path).ok()?;
    Reader::new().read_from_container(&mut BufReader::new(file)).ok()
}

fn ascii(exif: &Exif, tag: Tag) -> Option<&[u8]> {
    // Exif-IFD tags live under the primary image (IFD0 → ExifIFDPointer).
    match &exif.get_field(tag, In::PRIMARY)?.value {
        Value::Ascii(parts) => parts.first().map(Vec::as_slice),
        _ => None,
    }
}

fn stamp(exif: &Exif, main: Tag, subsec: Tag, offset: Tag) -> Option<RawStamp> {
    let bytes = ascii(exif, main)?;
    let raw = String::from_utf8_lossy(bytes)
        .trim_matches(|c: char| c == '\0' || c.is_whitespace())
        .to_owned();
    if raw.is_empty() {
        return None;
    }
    let wall = DateTime::from_ascii(bytes).ok().map(|mut dt| {
        // Companion tags are optional; a malformed one is ignored, not guessed.
        if let Some(s) = ascii(exif, subsec) {
            let _ = dt.parse_subsec(s);
        }
        if let Some(o) = ascii(exif, offset) {
            let _ = dt.parse_offset(o);
        }
        WallTime {
            year: i32::from(dt.year),
            month: dt.month,
            day: dt.day,
            hour: dt.hour,
            minute: dt.minute,
            second: dt.second,
            nanosecond: dt.nanosecond.unwrap_or(0),
            offset_minutes: dt.offset,
        }
    });
    Some(RawStamp { raw, wall })
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{ExifSpec, TempDir, write_jpeg, write_tiff};
    use super::*;

    #[test]
    fn reads_date_time_original_from_a_real_jpeg() {
        let dir = TempDir::new("exif-original");
        let p = dir.path().join("a.jpg");
        write_jpeg(&p, &ExifSpec::original("2024:01:15 10:30:00"));
        let (orig, dig) = read(&p);
        let orig = orig.expect("DateTimeOriginal");
        assert_eq!(orig.raw, "2024:01:15 10:30:00");
        assert_eq!(orig.wall, Some(WallTime::date(2024, 1, 15).at(10, 30, 0)));
        assert!(dig.is_none());
    }

    #[test]
    fn reads_digitized_subseconds_and_offset() {
        let dir = TempDir::new("exif-full");
        let p = dir.path().join("b.JPG");
        let spec = ExifSpec {
            original: Some("2024:01:15 10:30:00"),
            subsec_original: Some("25"),
            offset_original: Some("-05:00"),
            digitized: Some("2024:01:15 10:30:05"),
            ..ExifSpec::default()
        };
        write_jpeg(&p, &spec);
        let (orig, dig) = read(&p);
        let w = orig.unwrap().wall.unwrap();
        assert_eq!(w.nanosecond, 250_000_000);
        assert_eq!(w.offset_minutes, Some(-300));
        assert_eq!(dig.unwrap().raw, "2024:01:15 10:30:05");
    }

    #[test]
    fn reads_a_tiff_container() {
        let dir = TempDir::new("exif-tiff");
        let p = dir.path().join("c.tiff");
        write_tiff(&p, &ExifSpec::original("2021:06:01 08:00:00"));
        assert_eq!(read(&p).0.unwrap().raw, "2021:06:01 08:00:00");
    }

    #[test]
    fn a_jpeg_without_exif_has_no_candidates() {
        let dir = TempDir::new("exif-none");
        let p = dir.path().join("d.jpg");
        write_jpeg(&p, &ExifSpec::none());
        assert_eq!(read(&p), (None, None));
    }

    #[test]
    fn an_exif_segment_without_date_tags_has_no_candidates() {
        let dir = TempDir::new("exif-nodates");
        let p = dir.path().join("e.jpg");
        write_jpeg(&p, &ExifSpec::default());
        assert_eq!(read(&p), (None, None));
    }

    #[test]
    fn the_zero_placeholder_is_kept_raw_but_not_decoded_into_a_valid_date() {
        let dir = TempDir::new("exif-zero");
        let p = dir.path().join("f.jpg");
        write_jpeg(&p, &ExifSpec::original("0000:00:00 00:00:00"));
        let orig = read(&p).0.expect("raw text preserved");
        assert_eq!(orig.raw, "0000:00:00 00:00:00");
        // Decoded digits are all zero; the resolver rejects them as a calendar date.
        assert!(orig.wall.is_none_or(|w| w.month == 0));
    }

    #[test]
    fn unsupported_formats_and_garbage_are_ignored() {
        let dir = TempDir::new("exif-other");
        let png = dir.path().join("g.png");
        std::fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
        assert_eq!(read(&png), (None, None));
        let bad = dir.path().join("h.jpg");
        std::fs::write(&bad, b"not a jpeg").unwrap();
        assert_eq!(read(&bad), (None, None));
        assert_eq!(read(&dir.path().join("absent.jpg")), (None, None));
    }
}
