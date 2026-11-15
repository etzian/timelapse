//! Stabilization (ADR-025): translation only. Frame-to-frame shifts are estimated on
//! normalised grayscale images by coarse-to-fine block matching with a sub-pixel fit;
//! the camera path is smoothed, and each frame gets the correction that puts it on the
//! smoothed path, within a fixed crop.
//!
//! Pure and deterministic. Shifts and corrections are fractions of the frame's width and
//! height, so they apply at any resolution.

use image::RgbImage;
use image::imageops::{self, FilterType};

/// User settings, clamped by [`StabilizeConfig::clamped`].
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StabilizeConfig {
    /// Frames in the path-smoothing window (odd; 3–301). Larger = steadier, but slow
    /// intended moves are followed less closely.
    pub smoothing: usize,
    /// Largest crop per side, as a fraction of the frame (0.01–0.05).
    pub max_crop: f64,
}

impl Default for StabilizeConfig {
    fn default() -> Self {
        Self { smoothing: 31, max_crop: 0.05 }
    }
}

pub const MIN_SMOOTHING: usize = 3;
pub const MAX_SMOOTHING: usize = 301;
pub const MIN_CROP: f64 = 0.01;
pub const MAX_CROP: f64 = 0.05;

impl StabilizeConfig {
    pub fn clamped(self) -> Self {
        let mut smoothing = self.smoothing.clamp(MIN_SMOOTHING, MAX_SMOOTHING);
        if smoothing.is_multiple_of(2) {
            smoothing += 1;
        }
        let max_crop = if self.max_crop.is_finite() {
            self.max_crop.clamp(MIN_CROP, MAX_CROP)
        } else {
            MAX_CROP
        };
        Self { smoothing: smoothing.min(MAX_SMOOTHING), max_crop }
    }
}

/// Width of the working image: the review thumbnail is reduced to this before matching.
pub const WORK_WIDTH: u32 = 512;
/// Shifts larger than this fraction of the frame between two frames are a cut, not shake.
pub const MAX_PAIR_SHIFT: f64 = 0.10;
/// Below this confidence a pair is treated as not moving.
pub const MIN_CONFIDENCE: f64 = 0.15;

const LEVELS: usize = 4;
const COARSE_RADIUS: i32 = 12;
const REFINE_RADIUS: i32 = 2;
/// Fraction of each side left out of the comparison (vignetting, borders, overlays).
const BORDER: f64 = 0.10;

/// A grayscale image normalised to zero mean and unit standard deviation, so a change of
/// brightness or contrast does not read as motion.
#[derive(Debug, Clone, PartialEq)]
pub struct Gray {
    pub w: usize,
    pub h: usize,
    pub px: Vec<f32>,
}

impl Gray {
    pub fn from_luma(w: usize, h: usize, mut px: Vec<f32>) -> Self {
        let n = px.len().max(1) as f64;
        let mean = px.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
        let var = px.iter().map(|&v| (f64::from(v) - mean).powi(2)).sum::<f64>() / n;
        let sd = var.sqrt();
        for v in &mut px {
            *v = if sd > 1e-9 { ((f64::from(*v) - mean) / sd) as f32 } else { 0.0 };
        }
        Self { w, h, px }
    }

    fn at(&self, x: usize, y: usize) -> f32 {
        self.px[y * self.w + x]
    }

    /// Half the size, each pixel the mean of a 2×2 block.
    fn half(&self) -> Self {
        let (w, h) = (self.w / 2, self.h / 2);
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let s = self.at(2 * x, 2 * y)
                    + self.at(2 * x + 1, 2 * y)
                    + self.at(2 * x, 2 * y + 1)
                    + self.at(2 * x + 1, 2 * y + 1);
                px.push(s / 4.0);
            }
        }
        Self { w, h, px }
    }
}

/// `img` reduced to [`WORK_WIDTH`] and normalised.
pub fn gray(img: &RgbImage) -> Gray {
    let (w, h) = img.dimensions();
    let small = if w > WORK_WIDTH {
        let nh = ((f64::from(h) * f64::from(WORK_WIDTH) / f64::from(w)).round() as u32).max(1);
        imageops::resize(img, WORK_WIDTH, nh, FilterType::Triangle)
    } else {
        img.clone()
    };
    let (w, h) = small.dimensions();
    let px = small
        .pixels()
        .map(|p| {
            0.2126 * f32::from(p.0[0]) + 0.7152 * f32::from(p.0[1]) + 0.0722 * f32::from(p.0[2])
        })
        .collect();
    Gray::from_luma(w as usize, h as usize, px)
}

/// The estimated motion of the content from one frame to the next, in pixels of the
/// image measured, and how distinct the best match was (0–1).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Shift {
    pub dx: f64,
    pub dy: f64,
    pub confidence: f64,
}

/// Mean absolute difference between `a(x, y)` and `b(x + dx, y + dy)` over the inner
/// region, sampling every `step`th pixel.
fn cost(a: &Gray, b: &Gray, dx: i32, dy: i32, margin: usize, step: usize) -> f64 {
    let mut sum = 0.0f64;
    let mut n = 0u64;
    let mut y = margin;
    while y < a.h - margin {
        let by = (y as i32 + dy) as usize;
        let mut x = margin;
        while x < a.w - margin {
            let bx = (x as i32 + dx) as usize;
            sum += f64::from((a.at(x, y) - b.at(bx, by)).abs());
            n += 1;
            x += step;
        }
        y += step;
    }
    if n == 0 { f64::INFINITY } else { sum / n as f64 }
}

/// Lowest-cost shift within `radius` of `(cx, cy)`; returns it, its cost, and the costs.
fn search(
    a: &Gray,
    b: &Gray,
    cx: i32,
    cy: i32,
    radius: i32,
    step: usize,
) -> Option<(i32, i32, f64, Vec<f64>)> {
    let reach = (cx.abs().max(cy.abs()) + radius) as usize;
    let margin = ((a.w.min(a.h) as f64 * BORDER).ceil() as usize).max(reach + 1);
    if 2 * margin + 4 > a.w.min(a.h) || a.w != b.w || a.h != b.h {
        return None;
    }
    let mut best = (cx, cy, f64::INFINITY);
    let mut all = Vec::with_capacity(((2 * radius + 1) * (2 * radius + 1)) as usize);
    for dy in cy - radius..=cy + radius {
        for dx in cx - radius..=cx + radius {
            let c = cost(a, b, dx, dy, margin, step);
            all.push(c);
            // Strictly lower wins, so ties keep the first in scan order (deterministic).
            if c < best.2 {
                best = (dx, dy, c);
            }
        }
    }
    Some((best.0, best.1, best.2, all))
}

/// Sub-pixel offset of a minimum from three costs around it (parabola vertex), in −0.5…0.5.
fn vertex(left: f64, mid: f64, right: f64) -> f64 {
    let denom = left - 2.0 * mid + right;
    if denom.abs() < 1e-12 { 0.0 } else { (0.5 * (left - right) / denom).clamp(-0.5, 0.5) }
}

/// The shift of `b`'s content relative to `a` (`b(x + d) ≈ a(x)`), in pixels of `a`.
pub fn estimate(a: &Gray, b: &Gray) -> Shift {
    let none = Shift { dx: 0.0, dy: 0.0, confidence: 0.0 };
    if a.w != b.w || a.h != b.h {
        return none;
    }
    let mut pa = vec![a.clone()];
    let mut pb = vec![b.clone()];
    while pa.len() < LEVELS && pa.last().is_some_and(|g| g.w.min(g.h) >= 64) {
        let (na, nb) = (pa.last().unwrap().half(), pb.last().unwrap().half());
        pa.push(na);
        pb.push(nb);
    }
    let coarse = pa.len() - 1;
    let Some((mut dx, mut dy, _, costs)) = search(&pa[coarse], &pb[coarse], 0, 0, COARSE_RADIUS, 1)
    else {
        return none;
    };
    // Confidence: how far the best match stands below the typical one.
    let mut sorted = costs.clone();
    sorted.sort_by(f64::total_cmp);
    let (best, median) = (sorted[0], sorted[sorted.len() / 2]);
    let confidence = if median > 1e-9 { ((median - best) / median).clamp(0.0, 1.0) } else { 0.0 };
    // No distinct match (a featureless or unrelated frame): no shift, rather than the
    // arbitrary first of equal candidates.
    if confidence < MIN_CONFIDENCE {
        return Shift { dx: 0.0, dy: 0.0, confidence };
    }
    for level in (0..coarse).rev() {
        let step = if level == 0 { 2 } else { 1 };
        match search(&pa[level], &pb[level], dx * 2, dy * 2, REFINE_RADIUS, step) {
            Some((x, y, _, _)) => (dx, dy) = (x, y),
            None => return none,
        }
    }
    let margin =
        ((a.w.min(a.h) as f64 * BORDER).ceil() as usize).max((dx.abs().max(dy.abs()) + 2) as usize);
    if 2 * margin + 4 > a.w.min(a.h) {
        return none;
    }
    let c = |x: i32, y: i32| cost(a, b, x, y, margin, 2);
    let mid = c(dx, dy);
    let fx = vertex(c(dx - 1, dy), mid, c(dx + 1, dy));
    let fy = vertex(c(dx, dy - 1), mid, c(dx, dy + 1));
    Shift { dx: f64::from(dx) + fx, dy: f64::from(dy) + fy, confidence }
}

/// What the stabilization decided for one frame (fractions of the frame).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct FrameShift {
    /// Motion from the previous frame as measured (`None` for the first frame, a cut, or a
    /// match with too little confidence: counted as no motion).
    pub measured: Option<(f64, f64)>,
    pub confidence: f64,
    /// Cumulative camera path.
    pub path: (f64, f64),
    /// The smoothed path.
    pub smoothed: (f64, f64),
    /// Where the content must move so the frame lies on the smoothed path.
    pub correction: (f64, f64),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Stabilization {
    pub frames: Vec<FrameShift>,
    /// Crop per side used for every frame (fraction of the frame).
    pub crop: f64,
}

/// Centred running mean with a window shrinking symmetrically at the ends.
fn smooth(values: &[f64], window: usize) -> Vec<f64> {
    let half = window / 2;
    let n = values.len();
    (0..n)
        .map(|i| {
            let h = half.min(i).min(n - 1 - i);
            values[i - h..=i + h].iter().sum::<f64>() / (2 * h + 1) as f64
        })
        .collect()
}

/// The stabilization of a sequence from its pairwise shifts: `pairs[i]` is the shift from
/// frame `i` to frame `i + 1` in fractions of the frame (so `pairs.len() == frames - 1`).
pub fn stabilize(pairs: &[Shift], cfg: &StabilizeConfig) -> Stabilization {
    let cfg = cfg.clamped();
    let n = pairs.len() + 1;
    let usable = |s: &Shift| {
        s.confidence >= MIN_CONFIDENCE
            && s.dx.abs() <= MAX_PAIR_SHIFT
            && s.dy.abs() <= MAX_PAIR_SHIFT
    };
    let mut measured = vec![None; n];
    let mut confidence = vec![0.0; n];
    let (mut px, mut py) = (vec![0.0; n], vec![0.0; n]);
    for (i, s) in pairs.iter().enumerate() {
        confidence[i + 1] = s.confidence;
        let (dx, dy) = if usable(s) { (s.dx, s.dy) } else { (0.0, 0.0) };
        if usable(s) {
            measured[i + 1] = Some((s.dx, s.dy));
        }
        px[i + 1] = px[i] + dx;
        py[i + 1] = py[i] + dy;
    }
    let (sx, sy) = (smooth(&px, cfg.smoothing), smooth(&py, cfg.smoothing));
    let raw: Vec<(f64, f64)> = (0..n).map(|i| (sx[i] - px[i], sy[i] - py[i])).collect();
    let largest = raw.iter().fold(0.0f64, |m, (x, y)| m.max(x.abs()).max(y.abs()));
    let crop = largest.min(cfg.max_crop);
    let frames = (0..n)
        .map(|i| FrameShift {
            measured: measured[i],
            confidence: confidence[i],
            path: (px[i], py[i]),
            smoothed: (sx[i], sy[i]),
            correction: (raw[i].0.clamp(-crop, crop), raw[i].1.clamp(-crop, crop)),
        })
        .collect();
    Stabilization { frames, crop }
}

/// A pixel shift between two working images as fractions of the frame.
pub fn as_fraction(s: Shift, g: &Gray) -> Shift {
    Shift { dx: s.dx / g.w as f64, dy: s.dy / g.h as f64, confidence: s.confidence }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A smooth, aperiodic texture sampled at a sub-pixel offset.
    fn texture(w: usize, h: usize, ox: f64, oy: f64, gain: f32, bias: f32) -> Gray {
        let blobs: Vec<(f64, f64, f64)> = (0..40)
            .map(|i| {
                let t = i as f64;
                ((t * 37.7) % w as f64, (t * 23.3 + 11.0) % h as f64, 4.0 + (t * 1.7) % 9.0)
            })
            .collect();
        let mut px = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f64 - ox, y as f64 - oy);
                let mut v = (0.11 * fx + 0.07 * fy).sin() + 0.6 * (0.043 * fx - 0.091 * fy).cos();
                for (bx, by, r) in &blobs {
                    let d2 = (fx - bx).powi(2) + (fy - by).powi(2);
                    v += 2.0 * (-d2 / (2.0 * r * r)).exp();
                }
                px.push(gain * (v as f32) + bias);
            }
        }
        Gray::from_luma(w, h, px)
    }

    #[test]
    fn known_shifts_are_recovered_to_a_quarter_pixel() {
        let a = texture(320, 240, 0.0, 0.0, 1.0, 0.0);
        for (dx, dy) in [(0.0, 0.0), (3.4, -1.7), (-7.25, 5.5), (15.6, 9.1), (-22.0, -18.3)] {
            let b = texture(320, 240, dx, dy, 1.0, 0.0);
            let s = estimate(&a, &b);
            assert!((s.dx - dx).abs() < 0.25 && (s.dy - dy).abs() < 0.25, "{dx},{dy} → {s:?}");
            assert!(s.confidence > 0.3, "{s:?}");
        }
    }

    #[test]
    fn a_brightness_change_is_not_motion() {
        let a = texture(320, 240, 0.0, 0.0, 1.0, 0.0);
        let b = texture(320, 240, 2.5, -1.25, 0.4, 3.0);
        let s = estimate(&a, &b);
        assert!((s.dx - 2.5).abs() < 0.25 && (s.dy + 1.25).abs() < 0.25, "{s:?}");
    }

    #[test]
    fn a_featureless_frame_gives_no_shift() {
        let flat = Gray::from_luma(200, 150, vec![7.0; 200 * 150]);
        let s = estimate(&flat, &flat);
        assert_eq!((s.dx, s.dy, s.confidence), (0.0, 0.0, 0.0));
        let st = stabilize(&[s, s], &StabilizeConfig::default());
        assert!(st.frames.iter().all(|f| f.measured.is_none() && f.correction == (0.0, 0.0)));
    }

    fn shift(dx: f64, dy: f64) -> Shift {
        Shift { dx, dy, confidence: 1.0 }
    }

    /// Deterministic jitter in [-a, a].
    fn jitter(n: usize, a: f64, seed: u64) -> Vec<f64> {
        let mut x = seed;
        (0..n)
            .map(|_| {
                x = x
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                (((x >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0) * a
            })
            .collect()
    }

    #[test]
    fn jitter_is_removed_and_a_slow_pan_kept() {
        let n = 300;
        // Camera position: a pan of 0.001 frame per frame plus shake of ±0.004.
        let shake = jitter(n, 0.004, 5);
        let pos: Vec<f64> = (0..n).map(|i| 0.001 * i as f64 + shake[i]).collect();
        let pairs: Vec<Shift> = (1..n).map(|i| shift(pos[i] - pos[i - 1], 0.0)).collect();
        let st = stabilize(&pairs, &StabilizeConfig { smoothing: 31, max_crop: 0.05 });
        // The output position (path + correction) follows the pan, without the shake.
        let mut worst = 0.0f64;
        for (i, f) in st.frames.iter().enumerate().skip(20).take(n - 40) {
            let out = f.path.0 + f.correction.0;
            // The path starts at 0, i.e. at the first frame's position.
            worst = worst.max((out - (0.001 * i as f64 - pos[0])).abs());
        }
        assert!(worst < 0.0015, "residual {worst}");
        assert!(st.crop > 0.0 && st.crop <= 0.05);
    }

    #[test]
    fn the_crop_limit_holds_and_cuts_are_ignored() {
        let big: Vec<Shift> = jitter(100, 0.09, 9).into_iter().map(|d| shift(d, -d)).collect();
        let st = stabilize(&big, &StabilizeConfig { smoothing: 31, max_crop: 0.02 });
        assert_eq!(st.crop, 0.02);
        assert!(
            st.frames.iter().all(|f| f.correction.0.abs() <= 0.02 && f.correction.1.abs() <= 0.02)
        );
        let cut = stabilize(&[shift(0.3, 0.0), shift(0.0, 0.0)], &StabilizeConfig::default());
        assert_eq!(cut.frames[1].measured, None, "a jump of 30% is a cut");
        assert_eq!(cut.frames[1].path, (0.0, 0.0));
        let unsure = stabilize(
            &[Shift { dx: 0.01, dy: 0.0, confidence: 0.05 }],
            &StabilizeConfig::default(),
        );
        assert_eq!(unsure.frames[1].measured, None);
    }

    #[test]
    fn the_same_input_gives_the_same_result() {
        let a = texture(256, 192, 0.0, 0.0, 1.0, 0.0);
        let b = texture(256, 192, 1.3, 0.7, 1.0, 0.0);
        assert_eq!(estimate(&a, &b), estimate(&a, &b));
        let pairs: Vec<Shift> =
            jitter(80, 0.01, 2).into_iter().map(|d| shift(d, d / 2.0)).collect();
        assert_eq!(
            stabilize(&pairs, &StabilizeConfig::default()),
            stabilize(&pairs, &StabilizeConfig::default())
        );
    }

    #[test]
    fn the_configuration_is_clamped_and_images_are_reduced() {
        let c = StabilizeConfig { smoothing: 2, max_crop: 0.5 }.clamped();
        assert_eq!((c.smoothing, c.max_crop), (3, MAX_CROP));
        let c = StabilizeConfig { smoothing: 1000, max_crop: f64::NAN }.clamped();
        assert_eq!((c.smoothing, c.max_crop), (MAX_SMOOTHING, MAX_CROP));
        let g = gray(&RgbImage::from_pixel(1024, 768, image::Rgb([10, 20, 30])));
        assert_eq!((g.w, g.h), (512, 384));
    }
}
