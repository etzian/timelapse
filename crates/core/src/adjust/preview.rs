//! What the Adjust page shows (ADR-025): chart series downsampled from the stored runs,
//! and a before/after still of one frame. Read-only; everything comes from SQLite and the
//! cached review thumbnails.

use std::io::Cursor;
use std::path::Path;

use image::DynamicImage;
use image::imageops::FilterType;

use super::store;
use crate::export::frames::{FrameAdjust, shift_crop};
use crate::import::open_root;
use crate::{Error, Result};

/// Largest number of points a chart asks for.
pub const MAX_POINTS: usize = 2000;

/// Indices to keep so at most `max` points remain: in each bucket, the point where
/// `weight` is largest (so a spike is never dropped), always in order.
fn pick(n: usize, max: usize, weight: impl Fn(usize) -> f64) -> Vec<usize> {
    let max = max.clamp(2, MAX_POINTS);
    if n <= max {
        return (0..n).collect();
    }
    (0..max)
        .map(|b| {
            let (lo, hi) = (b * n / max, ((b + 1) * n / max).max(b * n / max + 1));
            (lo..hi).max_by(|&x, &y| weight(x).total_cmp(&weight(y)).then(y.cmp(&x))).unwrap_or(lo)
        })
        .collect()
}

/// One point of the deflicker chart: brightness in stops (log2 of linear luminance), and
/// each colour's balance against green in stops, before and after.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct FlickerPoint {
    pub sequence_index: usize,
    pub before: Option<f64>,
    pub trend: Option<f64>,
    pub after: Option<f64>,
    pub red_before: Option<f64>,
    pub red_after: Option<f64>,
    pub blue_before: Option<f64>,
    pub blue_after: Option<f64>,
}

pub fn flicker_series(root: &Path, run: i64, max_points: usize) -> Result<Vec<FlickerPoint>> {
    let conn = open_root(root)?;
    store::flicker_run(&conn, run)?.ok_or(Error::Adjust { reason: "unknown-adjustment" })?;
    let frames = store::flicker_frames(&conn, run)?;
    let log2 = |v: f64| v.log2();
    let points: Vec<FlickerPoint> = frames
        .iter()
        .map(|f| {
            let g = f.gains;
            let m = f.means;
            FlickerPoint {
                sequence_index: f.sequence_index,
                before: m.map(|m| log2(m.luma())),
                trend: f.trend.map(log2),
                after: m
                    .map(|m| log2(0.2126 * m.r * g[0] + 0.7152 * m.g * g[1] + 0.0722 * m.b * g[2])),
                red_before: m.map(|m| log2(m.r / m.g)),
                red_after: m.map(|m| log2(m.r * g[0] / (m.g * g[1]))),
                blue_before: m.map(|m| log2(m.b / m.g)),
                blue_after: m.map(|m| log2(m.b * g[2] / (m.g * g[1]))),
            }
        })
        .collect();
    let keep = pick(points.len(), max_points, |i| {
        let p = &points[i];
        p.before.zip(p.trend).map_or(0.0, |(b, t)| (b - t).abs())
    });
    Ok(keep.into_iter().map(|i| points[i]).collect())
}

/// One point of the stabilization chart, in fractions of the frame.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct StabilizationPoint {
    pub sequence_index: usize,
    pub path: (f64, f64),
    pub smoothed: (f64, f64),
    /// Where the frame ends up: path + correction.
    pub output: (f64, f64),
    pub measured: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct StabilizationSeries {
    pub crop: f64,
    pub points: Vec<StabilizationPoint>,
}

pub fn stabilization_series(
    root: &Path,
    run: i64,
    max_points: usize,
) -> Result<StabilizationSeries> {
    let conn = open_root(root)?;
    let r = store::stabilization_run(&conn, run)?
        .ok_or(Error::Adjust { reason: "unknown-adjustment" })?;
    let frames = store::stabilization_frames(&conn, run)?;
    let points: Vec<StabilizationPoint> = frames
        .iter()
        .map(|f| StabilizationPoint {
            sequence_index: f.sequence_index,
            path: f.path,
            smoothed: f.smoothed,
            output: (f.path.0 + f.shift.0, f.path.1 + f.shift.1),
            measured: f.measured.is_some(),
        })
        .collect();
    let keep = pick(points.len(), max_points, |i| {
        let p = &points[i];
        (p.path.0 - p.smoothed.0).abs().max((p.path.1 - p.smoothed.1).abs())
    });
    Ok(StabilizationSeries { crop: r.crop, points: keep.into_iter().map(|i| points[i]).collect() })
}

/// One frame of selection `run` (by its position), as JPEG bytes at review size: as shot
/// (`after == false`), or with the chosen runs applied, cropped and scaled back to the same
/// size so the two compare directly.
pub fn frame(
    root: &Path,
    run: i64,
    index: usize,
    flicker: Option<i64>,
    stabilization: Option<i64>,
    after: bool,
) -> Result<Vec<u8>> {
    let conn = open_root(root)?;
    let items = crate::sampling::store::run_items(&conn, run)?;
    let item = items.get(index).ok_or(Error::Adjust { reason: "unknown-frame" })?;
    let (location, hash) =
        crate::clean::store::location(&conn, item.image_id)?.ok_or(Error::StoreTransaction)?;
    let jpeg =
        crate::clean::thumbs::thumbnail(root, item.image_id, 1024, &location, hash.as_deref())?;
    if !after {
        return Ok(jpeg);
    }
    let adjust: FrameAdjust =
        crate::export::adjustments(&conn, run, flicker, stabilization, items.len()).map_err(
            |e| match e {
                Error::Export { reason } => Error::Adjust { reason },
                other => other,
            },
        )?[index];
    let undecodable = || Error::ImageUndecodable { path: location.to_string_lossy().into_owned() };
    let img = image::load_from_memory(&jpeg).map_err(|_| undecodable())?;
    let (w, h) = (img.width(), img.height());
    let shifted = match adjust.shift {
        Some((sx, sy, crop)) => DynamicImage::ImageRgb8(
            shift_crop(&img, sx, sy, crop).resize_exact(w, h, FilterType::Triangle).to_rgb8(),
        ),
        None => img,
    };
    let mut rgb = shifted.to_rgb8();
    if let Some(luts) = &adjust.luts {
        super::deflicker::apply(&mut rgb, luts);
    }
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(Cursor::new(&mut out), 85)
        .encode_image(&rgb)
        .map_err(|_| undecodable())?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picking_keeps_order_count_and_spikes() {
        assert_eq!(pick(5, 10, |_| 0.0), [0, 1, 2, 3, 4]);
        let spike = 731;
        let kept = pick(10_000, 100, |i| if i == spike { 9.0 } else { 0.0 });
        assert_eq!(kept.len(), 100);
        assert!(kept.windows(2).all(|w| w[0] < w[1]));
        assert!(kept.contains(&spike));
    }
}
