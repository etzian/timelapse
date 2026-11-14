//! Phase 2 — EXIF & timeline: capture-time candidates and their provenance.
//!
//! For each image we collect every capture-time *candidate* we can find, in
//! priority order (PROCESSING_PIPELINE.md §3, ADR-010):
//!
//! 1. EXIF `DateTimeOriginal` ([`exif`]), confidence **high**
//! 2. EXIF `DateTimeDigitized` ([`exif`]), confidence **medium**
//! 3. a date embedded in the filename ([`filename`]), confidence **medium**
//! 4. the filesystem modification time captured at import (Phase 1), confidence **low**
//!
//! [`resolver`] picks the winner and normalises it. Every result carries an
//! explicit [`TimestampSource`] and [`Confidence`]; nothing here invents a
//! default or corrects a value (ADR-010 / ADR-011: camera-clock drift stays
//! visible, never reconciled).

pub mod exif;
pub mod filename;
pub mod resolver;

#[cfg(test)]
pub(crate) mod fixture;

use std::path::Path;

/// Where the winning timestamp came from. The string form is stored verbatim in
/// `images.timestamp_source` (data, not UI copy — the UI maps it to a Fluent key).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimestampSource {
    ExifDatetimeOriginal,
    ExifDatetimeDigitized,
    Filename,
    Filesystem,
}

impl TimestampSource {
    pub const ALL: [Self; 4] =
        [Self::ExifDatetimeOriginal, Self::ExifDatetimeDigitized, Self::Filename, Self::Filesystem];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExifDatetimeOriginal => "exif_datetime_original",
            Self::ExifDatetimeDigitized => "exif_datetime_digitized",
            Self::Filename => "filename",
            Self::Filesystem => "filesystem",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.as_str() == s)
    }

    /// Confidence is a function of the source alone: a source never upgrades itself.
    pub const fn confidence(self) -> Confidence {
        match self {
            Self::ExifDatetimeOriginal => Confidence::High,
            Self::ExifDatetimeDigitized | Self::Filename => Confidence::Medium,
            Self::Filesystem => Confidence::Low,
        }
    }
}

/// How much weight a resolved timestamp deserves. Stored verbatim in
/// `images.timestamp_confidence`; shown as a badge, never used to reorder or correct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// Wall-clock digits exactly as a source recorded them, plus the UTC offset when
/// the source stated one (EXIF `OffsetTime*`). `offset_minutes == None` means the
/// zone is unknown; the resolver then assumes UTC and says so (ADR-019).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanosecond: u32,
    pub offset_minutes: Option<i16>,
}

impl WallTime {
    pub const fn date(year: i32, month: u8, day: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
            offset_minutes: None,
        }
    }

    pub const fn at(mut self, hour: u8, minute: u8, second: u8) -> Self {
        self.hour = hour;
        self.minute = minute;
        self.second = second;
        self
    }
}

/// A candidate value: the original text (preserved verbatim in the database) and its
/// decoded wall time, or `None` when the text is present but not a usable date
/// (e.g. the common `0000:00:00 00:00:00` placeholder).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawStamp {
    pub raw: String,
    pub wall: Option<WallTime>,
}

/// Every capture-time candidate found for one image.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Candidates {
    pub exif_original: Option<RawStamp>,
    pub exif_digitized: Option<RawStamp>,
    pub filename: Option<RawStamp>,
    /// Seconds since the UNIX epoch; `None` when the import could not read it.
    pub filesystem_unix: Option<i64>,
}

/// Collect the candidates for the file at `path`. EXIF is read only for the formats
/// ADR-019 covers (JPEG/TIFF); `filesystem_unix` is the value Phase 1 recorded at import
/// (`images.modified_at`) — a non-positive value is the scan's "unknown" and is dropped.
/// A missing or unreadable file is not an error: its EXIF candidates are simply absent.
pub fn collect(path: &Path, filesystem_unix: Option<i64>) -> Candidates {
    let (exif_original, exif_digitized) = exif::read(path);
    let filename = path.file_stem().and_then(|s| s.to_str()).and_then(filename::parse);
    Candidates {
        exif_original,
        exif_digitized,
        filename,
        filesystem_unix: filesystem_unix.filter(|v| *v > 0),
    }
}

#[cfg(test)]
mod tests {
    use super::fixture::{ExifSpec, TempDir, write_jpeg};
    use super::*;

    #[test]
    fn confidence_follows_the_source() {
        assert_eq!(TimestampSource::ExifDatetimeOriginal.confidence(), Confidence::High);
        assert_eq!(TimestampSource::ExifDatetimeDigitized.confidence(), Confidence::Medium);
        assert_eq!(TimestampSource::Filename.confidence(), Confidence::Medium);
        assert_eq!(TimestampSource::Filesystem.confidence(), Confidence::Low);
    }

    #[test]
    fn source_strings_round_trip() {
        for s in TimestampSource::ALL {
            assert_eq!(TimestampSource::parse(s.as_str()), Some(s));
        }
        assert_eq!(TimestampSource::parse("nope"), None);
    }

    #[test]
    fn collect_reads_exif_filename_and_filesystem_together() {
        let dir = TempDir::new("collect-all");
        let path = dir.path().join("IMG_20190704_120000.jpg");
        write_jpeg(&path, &ExifSpec::original("2024:01:15 10:30:00"));
        let c = collect(&path, Some(1_600_000_000));
        assert_eq!(c.exif_original.unwrap().raw, "2024:01:15 10:30:00");
        assert!(c.exif_digitized.is_none());
        assert_eq!(c.filename.unwrap().raw, "20190704_120000");
        assert_eq!(c.filesystem_unix, Some(1_600_000_000));
    }

    #[test]
    fn collect_on_a_file_without_exif_keeps_the_other_candidates() {
        let dir = TempDir::new("collect-noexif");
        let path = dir.path().join("scan.jpg");
        write_jpeg(&path, &ExifSpec::none());
        let c = collect(&path, Some(1_600_000_000));
        assert!(c.exif_original.is_none() && c.exif_digitized.is_none() && c.filename.is_none());
        assert_eq!(c.filesystem_unix, Some(1_600_000_000));
    }

    #[test]
    fn collect_drops_the_scans_unknown_mtime() {
        let dir = TempDir::new("collect-zero");
        let c = collect(&dir.path().join("missing.jpg"), Some(0));
        assert_eq!(c, Candidates::default());
    }
}
