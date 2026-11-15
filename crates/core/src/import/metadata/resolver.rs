//! Pick the winning candidate and normalise it to one comparable instant.
//!
//! The normalised form is fixed-width UTC ISO-8601 with milliseconds,
//! `YYYY-MM-DDTHH:MM:SS.mmmZ`: it sorts lexicographically in time order (so a plain
//! `ORDER BY` is chronological) and SQLite's `strftime()` / `unixepoch()` read it.
//!
//! Time zones (ADR-019): an explicit EXIF offset is applied; without one the wall
//! clock is *assumed* to be UTC and the result says so (`offset_minutes == None`).
//! No named-zone database is consulted and no value is ever shifted to "fit" its
//! neighbours. A candidate whose text is not a real date (e.g. `0000:00:00`) is
//! skipped in favour of the next source, never repaired.

use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use super::{Candidates, Confidence, RawStamp, TimestampSource, WallTime};

/// The winning timestamp for one image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// `YYYY-MM-DDTHH:MM:SS.mmmZ`.
    pub normalized: String,
    pub source: TimestampSource,
    pub confidence: Confidence,
    /// The offset that was applied, in minutes east of UTC. `None` = the source gave no
    /// zone and UTC was assumed. The filesystem time is a true instant and records `Some(0)`.
    pub offset_minutes: Option<i16>,
}

/// Resolve in priority order: EXIF original → EXIF digitized → filename → filesystem.
/// `None` only when no candidate holds a usable value; the caller keeps the image and
/// marks it unresolved.
pub fn resolve(c: &Candidates) -> Option<Resolved> {
    let walls = [
        (TimestampSource::ExifDatetimeOriginal, &c.exif_original),
        (TimestampSource::ExifDatetimeDigitized, &c.exif_digitized),
        (TimestampSource::Filename, &c.filename),
    ];
    walls
        .into_iter()
        .find_map(|(source, stamp)| {
            let wall = stamp.as_ref().and_then(|s: &RawStamp| s.wall)?;
            let normalized = normalize_wall(&wall)?;
            Some(Resolved {
                normalized,
                source,
                confidence: source.confidence(),
                offset_minutes: wall.offset_minutes,
            })
        })
        .or_else(|| {
            let normalized = normalize_unix(c.filesystem_unix?)?;
            Some(Resolved {
                normalized,
                source: TimestampSource::Filesystem,
                confidence: TimestampSource::Filesystem.confidence(),
                offset_minutes: Some(0),
            })
        })
}

/// Validate a wall time as a calendar instant and express it in UTC. `None` when the
/// digits are not a real date/time or the offset is out of range.
pub fn normalize_wall(w: &WallTime) -> Option<String> {
    let date = Date::from_calendar_date(w.year, Month::try_from(w.month).ok()?, w.day).ok()?;
    let time = Time::from_hms_nano(w.hour, w.minute, w.second, w.nanosecond).ok()?;
    let offset =
        UtcOffset::from_whole_seconds(i32::from(w.offset_minutes.unwrap_or(0)) * 60).ok()?;
    let utc = PrimitiveDateTime::new(date, time).assume_offset(offset).to_offset(UtcOffset::UTC);
    format_utc(utc)
}

/// Seconds since the UNIX epoch, as the normalised string.
pub fn normalize_unix(secs: i64) -> Option<String> {
    format_utc(OffsetDateTime::from_unix_timestamp(secs).ok()?)
}

/// Milliseconds since the UNIX epoch, as the normalised string.
pub fn normalize_unix_ms(ms: i64) -> Option<String> {
    format_utc(OffsetDateTime::from_unix_timestamp_nanos(i128::from(ms) * 1_000_000).ok()?)
}

fn format_utc(t: OffsetDateTime) -> Option<String> {
    // Fixed width is what keeps text order == time order; years outside 0..=9999 cannot be
    // written in four digits, so they are not representable rather than mis-sorted.
    if !(0..=9999).contains(&t.year()) {
        return None;
    }
    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        t.year(),
        u8::from(t.month()),
        t.day(),
        t.hour(),
        t.minute(),
        t.second(),
        t.millisecond()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp(raw: &str, wall: Option<WallTime>) -> Option<RawStamp> {
        Some(RawStamp { raw: raw.into(), wall })
    }

    fn naive(y: i32, mo: u8, d: u8, h: u8, mi: u8, s: u8) -> WallTime {
        WallTime::date(y, mo, d).at(h, mi, s)
    }

    #[test]
    fn exif_original_wins_over_a_filename_on_a_different_day() {
        let c = Candidates {
            exif_original: stamp("2024:01:15 10:30:00", Some(naive(2024, 1, 15, 10, 30, 0))),
            filename: stamp("20190704", Some(WallTime::date(2019, 7, 4))),
            filesystem_unix: Some(1_700_000_000),
            ..Candidates::default()
        };
        let r = resolve(&c).unwrap();
        assert_eq!(r.source, TimestampSource::ExifDatetimeOriginal);
        assert_eq!(r.confidence, Confidence::High);
        assert_eq!(r.normalized, "2024-01-15T10:30:00.000Z");
        assert_eq!(r.offset_minutes, None, "no EXIF offset → UTC assumed, and said so");
    }

    #[test]
    fn digitized_then_filename_then_filesystem() {
        let mut c = Candidates {
            exif_digitized: stamp("x", Some(naive(2019, 6, 1, 8, 0, 0))),
            filename: stamp("20180302", Some(WallTime::date(2018, 3, 2))),
            filesystem_unix: Some(1_600_000_000),
            ..Candidates::default()
        };
        assert_eq!(resolve(&c).unwrap().source, TimestampSource::ExifDatetimeDigitized);
        c.exif_digitized = None;
        let r = resolve(&c).unwrap();
        assert_eq!((r.source, r.confidence), (TimestampSource::Filename, Confidence::Medium));
        assert_eq!(r.normalized, "2018-03-02T00:00:00.000Z");
        c.filename = None;
        let r = resolve(&c).unwrap();
        assert_eq!((r.source, r.confidence), (TimestampSource::Filesystem, Confidence::Low));
        assert_eq!(r.normalized, "2020-09-13T12:26:40.000Z");
        assert_eq!(r.offset_minutes, Some(0));
        c.filesystem_unix = None;
        assert_eq!(resolve(&c), None);
    }

    #[test]
    fn an_invalid_exif_value_falls_through_instead_of_being_repaired() {
        let c = Candidates {
            exif_original: stamp("0000:00:00 00:00:00", Some(naive(0, 0, 0, 0, 0, 0))),
            exif_digitized: stamp("2023:02:30 10:00:00", Some(naive(2023, 2, 30, 10, 0, 0))),
            filesystem_unix: Some(1_600_000_000),
            ..Candidates::default()
        };
        assert_eq!(resolve(&c).unwrap().source, TimestampSource::Filesystem);
    }

    #[test]
    fn an_explicit_offset_is_applied() {
        let mut w = naive(2024, 1, 15, 10, 30, 0);
        w.offset_minutes = Some(-300);
        assert_eq!(normalize_wall(&w).as_deref(), Some("2024-01-15T15:30:00.000Z"));
        w.offset_minutes = Some(330);
        assert_eq!(normalize_wall(&w).as_deref(), Some("2024-01-15T05:00:00.000Z"));
        // Crossing midnight and a year boundary.
        let mut ny = naive(2024, 1, 1, 0, 30, 0);
        ny.offset_minutes = Some(60);
        assert_eq!(normalize_wall(&ny).as_deref(), Some("2023-12-31T23:30:00.000Z"));
    }

    #[test]
    fn subseconds_are_kept_to_the_millisecond() {
        let mut w = naive(2024, 1, 15, 10, 30, 0);
        w.nanosecond = 123_456_789;
        assert_eq!(normalize_wall(&w).as_deref(), Some("2024-01-15T10:30:00.123Z"));
    }

    #[test]
    fn mixed_sources_and_zones_sort_by_the_normalised_instant() {
        let mut east = naive(2024, 1, 15, 18, 0, 0);
        east.offset_minutes = Some(480); // 10:00Z
        let mut west = naive(2024, 1, 15, 5, 30, 0);
        west.offset_minutes = Some(-300); // 10:30Z
        let set = [
            Candidates { exif_original: stamp("e", Some(east)), ..Candidates::default() },
            Candidates { exif_original: stamp("w", Some(west)), ..Candidates::default() },
            Candidates {
                filename: stamp("f", Some(naive(2024, 1, 15, 9, 0, 0))),
                ..Candidates::default()
            },
            Candidates { filesystem_unix: Some(1_705_316_400), ..Candidates::default() }, // 11:00Z
        ];
        let mut out: Vec<String> = set.iter().map(|c| resolve(c).unwrap().normalized).collect();
        let expected = [
            "2024-01-15T09:00:00.000Z",
            "2024-01-15T10:00:00.000Z",
            "2024-01-15T10:30:00.000Z",
            "2024-01-15T11:00:00.000Z",
        ];
        out.sort();
        assert_eq!(out, expected, "text order of the normalised form is time order");
    }

    #[test]
    fn unrepresentable_inputs_are_rejected() {
        let mut w = naive(2024, 1, 15, 10, 0, 0);
        w.offset_minutes = Some(i16::MAX);
        assert_eq!(normalize_wall(&w), None);
        assert_eq!(normalize_unix(i64::MAX), None);
        assert_eq!(normalize_unix(-62_200_000_000), None, "year < 0");
    }
}
