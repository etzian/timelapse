//! Dates embedded in camera / phone file names, e.g. `IMG_20240115_103000`,
//! `20240115103000`, `PXL_20231130_141500123`, `photo 2023-11-30 14.15`.
//!
//! Recognised shapes (each must start and end at a digit boundary, so an order
//! number glued to other digits is never read as a date):
//!
//! - compact: `YYYYMMDD`, `YYYYMMDDHHMM`, `YYYYMMDDHHMMSS`, and `YYYYMMDD` + one of
//!   `_ - . T space` + `HHMM` / `HHMMSS`;
//! - separated: `YYYY-MM-DD` (separator one of `- . _ space`, the same twice), optionally
//!   followed by one of `_ - . T space` and `HHMM[SS]` or `HH[:.-]MM[[:.-]SS]`.
//!
//! No time zone is implied: a filename only carries wall-clock digits.

use super::{RawStamp, WallTime};

const DATE_SEPS: &[u8] = b"-._ ";
const DATE_TIME_SEPS: &[u8] = b"_-.T ";
const TIME_SEPS: &[u8] = b":.-";

/// The first date (and time, when present) found in a file stem, or `None`.
pub fn parse(stem: &str) -> Option<RawStamp> {
    let b = stem.as_bytes();
    (0..b.len())
        .filter(|&i| b[i].is_ascii_digit() && (i == 0 || !b[i - 1].is_ascii_digit()))
        .find_map(|i| {
            let (end, wall) = compact(b, i).or_else(|| separated(b, i))?;
            Some(RawStamp { raw: stem[i..end].to_owned(), wall: Some(wall) })
        })
}

fn run_len(b: &[u8], i: usize) -> usize {
    b.get(i..).map_or(0, |s| s.iter().take_while(|c| c.is_ascii_digit()).count())
}

fn num(b: &[u8], i: usize, len: usize) -> u32 {
    b[i..i + len].iter().fold(0, |acc, c| acc * 10 + u32::from(c - b'0'))
}

fn is_in(b: &[u8], i: usize, set: &[u8]) -> bool {
    b.get(i).is_some_and(|c| set.contains(c))
}

fn date_at(b: &[u8], y: usize, m: usize, d: usize) -> Option<WallTime> {
    let (year, month, day) = (num(b, y, 4), num(b, m, 2), num(b, d, 2));
    if !(1970..=2100).contains(&year) {
        return None;
    }
    let month = u8::try_from(month).ok()?;
    let day = u8::try_from(day).ok()?;
    // A full calendar check (rejects 2023-02-30), not just range checks.
    time::Date::from_calendar_date(year as i32, time::Month::try_from(month).ok()?, day).ok()?;
    Some(WallTime::date(year as i32, month, day))
}

fn with_time(w: WallTime, h: u32, m: u32, s: u32) -> Option<WallTime> {
    (h < 24 && m < 60 && s < 60).then(|| w.at(h as u8, m as u8, s as u8))
}

/// `HHMM` or `HHMMSS` as one digit run starting at `i`; returns the end index.
fn compact_time(b: &[u8], i: usize, w: WallTime) -> Option<(usize, WallTime)> {
    match run_len(b, i) {
        4 => Some((i + 4, with_time(w, num(b, i, 2), num(b, i + 2, 2), 0)?)),
        6 => Some((i + 6, with_time(w, num(b, i, 2), num(b, i + 2, 2), num(b, i + 4, 2))?)),
        _ => None,
    }
}

fn compact(b: &[u8], i: usize) -> Option<(usize, WallTime)> {
    let n = run_len(b, i);
    if !matches!(n, 8 | 12 | 14) {
        return None;
    }
    let date = date_at(b, i, i + 4, i + 6);
    match n {
        8 => {
            let date = date?;
            if is_in(b, i + 8, DATE_TIME_SEPS)
                && let Some(t) = compact_time(b, i + 9, date)
            {
                return Some(t);
            }
            Some((i + 8, date))
        }
        // An impossible time keeps the date, as in the other shapes.
        12 => {
            let date = date?;
            Some((i + 12, with_time(date, num(b, i + 8, 2), num(b, i + 10, 2), 0).unwrap_or(date)))
        }
        14 => Some((
            i + 14,
            with_time(date?, num(b, i + 8, 2), num(b, i + 10, 2), num(b, i + 12, 2))
                .unwrap_or(date?),
        )),
        _ => None,
    }
}

fn separated(b: &[u8], i: usize) -> Option<(usize, WallTime)> {
    if run_len(b, i) != 4 || !is_in(b, i + 4, DATE_SEPS) {
        return None;
    }
    let sep = b[i + 4];
    if run_len(b, i + 5) != 2 || b.get(i + 7) != Some(&sep) || run_len(b, i + 8) != 2 {
        return None;
    }
    let date = date_at(b, i, i + 5, i + 8)?;
    let end = i + 10;
    if !is_in(b, end, DATE_TIME_SEPS) {
        return Some((end, date));
    }
    let t = end + 1;
    if let Some(found) = compact_time(b, t, date) {
        return Some(found);
    }
    // HH<sep>MM[<sep>SS]
    if run_len(b, t) == 2 && is_in(b, t + 2, TIME_SEPS) && run_len(b, t + 3) == 2 {
        let (h, m) = (num(b, t, 2), num(b, t + 3, 2));
        if is_in(b, t + 5, TIME_SEPS)
            && run_len(b, t + 6) == 2
            && let Some(w) = with_time(date, h, m, num(b, t + 6, 2))
        {
            return Some((t + 8, w));
        }
        if let Some(w) = with_time(date, h, m, 0) {
            return Some((t + 5, w));
        }
    }
    Some((end, date))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall(stem: &str) -> Option<WallTime> {
        parse(stem).and_then(|s| s.wall)
    }

    fn raw(stem: &str) -> Option<String> {
        parse(stem).map(|s| s.raw)
    }

    #[test]
    fn compact_date_only() {
        assert_eq!(wall("DSC_20240115"), Some(WallTime::date(2024, 1, 15)));
    }

    #[test]
    fn compact_fourteen_and_twelve_digit_runs() {
        assert_eq!(wall("IMG_20240115103000"), Some(WallTime::date(2024, 1, 15).at(10, 30, 0)));
        assert_eq!(wall("IMG_202401151030"), Some(WallTime::date(2024, 1, 15).at(10, 30, 0)));
    }

    #[test]
    fn compact_date_plus_separate_time_run() {
        assert_eq!(wall("IMG_20240115_103005"), Some(WallTime::date(2024, 1, 15).at(10, 30, 5)));
        assert_eq!(raw("IMG_20240115_103005").as_deref(), Some("20240115_103005"));
        assert_eq!(wall("20240115-1030"), Some(WallTime::date(2024, 1, 15).at(10, 30, 0)));
    }

    #[test]
    fn a_trailing_millisecond_run_does_not_produce_a_time() {
        // PXL_…_141500123: the 9-digit tail is not HHMM/HHMMSS, so only the date is used.
        assert_eq!(wall("PXL_20231130_141500123"), Some(WallTime::date(2023, 11, 30)));
    }

    #[test]
    fn separated_dates_and_times() {
        assert_eq!(wall("photo_2023-11-30_1415"), Some(WallTime::date(2023, 11, 30).at(14, 15, 0)));
        assert_eq!(wall("IMG 2023-06-02 14:33"), Some(WallTime::date(2023, 6, 2).at(14, 33, 0)));
        assert_eq!(wall("2023.06.02 14.33.07"), Some(WallTime::date(2023, 6, 2).at(14, 33, 7)));
        assert_eq!(wall("Screenshot 2023-06-02"), Some(WallTime::date(2023, 6, 2)));
    }

    #[test]
    fn mixed_date_separators_are_rejected() {
        assert_eq!(parse("2023-06.02"), None);
    }

    #[test]
    fn names_without_a_date_yield_none() {
        assert_eq!(parse("IMG_0042"), None);
        assert_eq!(parse("scan"), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn implausible_values_are_rejected_not_clamped() {
        assert_eq!(parse("x2600-13-40y"), None);
        assert_eq!(parse("00000000"), None);
        assert_eq!(parse("20230230"), None, "February 30th is not a date");
        assert_eq!(parse("123456789"), None, "a 9-digit run is not a date");
        // A valid date with an invalid time keeps the date only.
        assert_eq!(wall("2023-06-02 25:61"), Some(WallTime::date(2023, 6, 2)));
        assert_eq!(wall("202401152599"), Some(WallTime::date(2024, 1, 15)));
        assert_eq!(wall("20240115259900"), Some(WallTime::date(2024, 1, 15)));
    }

    #[test]
    fn digits_glued_to_other_digits_are_not_a_date() {
        assert_eq!(parse("9920240115"), None);
    }

    #[test]
    fn the_first_valid_date_wins() {
        assert_eq!(wall("trip_0001_20220101"), Some(WallTime::date(2022, 1, 1)));
    }
}
