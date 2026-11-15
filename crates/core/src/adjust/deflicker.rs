//! Deflicker (ADR-025): per-frame exposure and colour gains that pull quick brightness
//! and white-balance jumps back toward their slow trend, keeping genuine light changes.
//!
//! Pure functions: measurement of one image, gains for a whole sequence, and the lookup
//! tables that apply a frame's gains. Deterministic (fixed-order arithmetic).

use std::sync::LazyLock;

use image::RgbImage;

/// User settings, clamped by [`DeflickerConfig::clamped`].
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeflickerConfig {
    /// 0 = no correction, 1 = all the way to the trend.
    pub strength: f64,
    /// Frames in the trend window (odd; 3–301).
    pub window: usize,
    /// Also correct each colour channel against its own trend.
    pub colour: bool,
}

impl Default for DeflickerConfig {
    fn default() -> Self {
        Self { strength: 0.8, window: 15, colour: true }
    }
}

pub const MIN_WINDOW: usize = 3;
pub const MAX_WINDOW: usize = 301;

impl DeflickerConfig {
    pub fn clamped(self) -> Self {
        let strength = if self.strength.is_finite() { self.strength.clamp(0.0, 1.0) } else { 0.0 };
        let mut window = self.window.clamp(MIN_WINDOW, MAX_WINDOW);
        if window.is_multiple_of(2) {
            window += 1;
        }
        Self { strength, window: window.min(MAX_WINDOW), colour: self.colour }
    }
}

/// Largest exposure correction: one stop either way.
pub const MAX_EXPOSURE: f64 = std::f64::consts::LN_2;
/// Largest colour correction: a third of a stop either way.
pub const MAX_COLOUR: f64 = std::f64::consts::LN_2 / 3.0;

static TO_LINEAR: LazyLock<[f64; 256]> = LazyLock::new(|| {
    let mut t = [0.0; 256];
    for (i, v) in t.iter_mut().enumerate() {
        let c = i as f64 / 255.0;
        *v = if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    }
    t
});

fn to_srgb(l: f64) -> f64 {
    let l = l.clamp(0.0, 1.0);
    if l <= 0.003_130_8 { l * 12.92 } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 }
}

/// Mean of each channel in linear light (0–1).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChannelMeans {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl ChannelMeans {
    /// Relative luminance (Rec. 709 weights).
    pub fn luma(&self) -> f64 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    fn channel(&self, k: usize) -> f64 {
        [self.r, self.g, self.b][k]
    }
}

/// The channel means of `img`, ignoring clipped (any channel ≥ 250) and near-black (all
/// ≤ 4) pixels, which carry no exposure information. `None` when no pixel is usable.
pub fn measure(img: &RgbImage) -> Option<ChannelMeans> {
    let lin = &*TO_LINEAR;
    let (mut r, mut g, mut b, mut n) = (0.0, 0.0, 0.0, 0u64);
    for p in img.pixels() {
        let [pr, pg, pb] = p.0;
        if pr >= 250 || pg >= 250 || pb >= 250 || (pr <= 4 && pg <= 4 && pb <= 4) {
            continue;
        }
        r += lin[usize::from(pr)];
        g += lin[usize::from(pg)];
        b += lin[usize::from(pb)];
        n += 1;
    }
    (n > 0).then(|| {
        let n = n as f64;
        ChannelMeans { r: r / n, g: g / n, b: b / n }
    })
}

/// What the deflicker decided for one frame.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct FrameGains {
    /// Measured luminance (linear), `None` when the frame could not be measured.
    pub luma: Option<f64>,
    /// The trend's luminance at this frame (linear).
    pub trend: Option<f64>,
    /// Gains for R, G and B (1 = unchanged).
    pub gains: [f64; 3],
}

use super::series::trend;

/// Gains for a sequence of frames (in order). Frames that could not be measured keep a
/// gain of 1 and are left out of the trend of the others.
pub fn gains(means: &[Option<ChannelMeans>], cfg: &DeflickerConfig) -> Vec<FrameGains> {
    let cfg = cfg.clamped();
    let measured: Vec<(usize, ChannelMeans)> = means
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            m.filter(|m| m.luma() > 0.0 && m.r > 0.0 && m.g > 0.0 && m.b > 0.0).map(|m| (i, m))
        })
        .collect();
    let mut out: Vec<FrameGains> = means
        .iter()
        .map(|m| FrameGains { luma: m.map(|m| m.luma()), trend: None, gains: [1.0; 3] })
        .collect();
    if measured.is_empty() {
        return out;
    }
    let log_y: Vec<f64> = measured.iter().map(|(_, m)| m.luma().ln()).collect();
    let trend_y = trend(&log_y, cfg.window);
    // Per channel: the log ratio to luminance and its trend.
    let ratios: Vec<(Vec<f64>, Vec<f64>)> = (0..3)
        .map(|k| {
            let series: Vec<f64> =
                measured.iter().map(|(_, m)| (m.channel(k) / m.luma()).ln()).collect();
            let t = trend(&series, cfg.window);
            (series, t)
        })
        .collect();
    const W: [f64; 3] = [0.2126, 0.7152, 0.0722];
    for (j, (i, m)) in measured.iter().enumerate() {
        let exposure = (cfg.strength * (trend_y[j] - log_y[j])).clamp(-MAX_EXPOSURE, MAX_EXPOSURE);
        let mut colour = [1.0; 3];
        if cfg.colour {
            for (k, ck) in colour.iter_mut().enumerate() {
                let (series, t) = &ratios[k];
                *ck = (cfg.strength * (t[j] - series[j])).clamp(-MAX_COLOUR, MAX_COLOUR).exp();
            }
            // Colour moves hue only: renormalise so the frame's luminance is unchanged by
            // it, and exposure alone sets brightness.
            let y = m.luma();
            let after: f64 = (0..3).map(|k| W[k] * m.channel(k) * colour[k]).sum();
            if after > 0.0 {
                let norm = y / after;
                colour.iter_mut().for_each(|c| *c *= norm);
            }
        }
        let g = [0, 1, 2].map(|k| exposure.exp() * colour[k]);
        out[*i].trend = Some(trend_y[j].exp());
        out[*i].gains = g;
    }
    out
}

/// Three 256-entry tables mapping an sRGB value to the value with `gains` applied in
/// linear light (clamped). Black maps to black.
pub fn luts(gains: [f64; 3]) -> [[u8; 256]; 3] {
    let lin = &*TO_LINEAR;
    let mut out = [[0u8; 256]; 3];
    for (k, table) in out.iter_mut().enumerate() {
        for (v, slot) in table.iter_mut().enumerate() {
            *slot = (to_srgb(lin[v] * gains[k]) * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

/// Apply `luts` to packed RGB bytes in place.
pub fn apply(rgb: &mut [u8], luts: &[[u8; 256]; 3]) {
    for px in rgb.as_chunks_mut::<3>().0 {
        px[0] = luts[0][usize::from(px[0])];
        px[1] = luts[1][usize::from(px[1])];
        px[2] = luts[2][usize::from(px[2])];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic pseudo-random sequence in [-1, 1].
    fn noise(n: usize, seed: u64) -> Vec<f64> {
        let mut x = seed;
        (0..n)
            .map(|_| {
                x = x
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                ((x >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
            })
            .collect()
    }

    fn grey(y: f64) -> Option<ChannelMeans> {
        Some(ChannelMeans { r: y, g: y, b: y })
    }

    fn stdev(v: &[f64]) -> f64 {
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    }

    fn corrected_log_luma(means: &[Option<ChannelMeans>], g: &[FrameGains]) -> Vec<f64> {
        means
            .iter()
            .zip(g)
            .map(|(m, g)| {
                let m = m.unwrap();
                (0.2126 * m.r * g.gains[0] + 0.7152 * m.g * g.gains[1] + 0.0722 * m.b * g.gains[2])
                    .ln()
            })
            .collect()
    }

    #[test]
    fn random_flicker_is_strongly_reduced() {
        let n = 400;
        let means: Vec<_> = noise(n, 7).iter().map(|e| grey(0.2 * (0.25 * e).exp())).collect();
        let before: Vec<f64> = means.iter().map(|m| m.unwrap().luma().ln()).collect();
        let cfg = DeflickerConfig { strength: 1.0, window: 31, colour: false };
        let after = corrected_log_luma(&means, &gains(&means, &cfg));
        assert!(stdev(&after) < stdev(&before) / 4.0, "{} vs {}", stdev(&after), stdev(&before));
    }

    #[test]
    fn a_slow_change_such_as_a_sunset_is_kept() {
        // Brightness falling 4 stops over 300 frames, smoothly (linear in log space).
        let n = 300;
        let means: Vec<_> = (0..n)
            .map(|i| grey(0.5 * (-(i as f64) / n as f64 * 4.0 * std::f64::consts::LN_2).exp()))
            .collect();
        for g in gains(&means, &DeflickerConfig { strength: 1.0, window: 31, colour: true }) {
            for k in 0..3 {
                assert!((g.gains[k] - 1.0).abs() < 1e-9, "{:?}", g.gains);
            }
        }
    }

    #[test]
    fn the_first_and_last_frames_are_corrected_too() {
        let mut means: Vec<_> = (0..60).map(|_| grey(0.2)).collect();
        means[0] = grey(0.3);
        means[59] = grey(0.13);
        let g = gains(&means, &DeflickerConfig { strength: 1.0, window: 15, colour: false });
        assert!((g[0].gains[0] - 0.2 / 0.3).abs() < 0.02, "{:?}", g[0].gains);
        assert!((g[59].gains[0] - 0.2 / 0.13).abs() < 0.03, "{:?}", g[59].gains);
    }

    #[test]
    fn colour_correction_leaves_brightness_to_exposure() {
        let n = 120;
        let e = noise(n, 21);
        let means: Vec<_> = (0..n)
            .map(|i| Some(ChannelMeans { r: 0.3 * (0.15 * e[i]).exp(), g: 0.2, b: 0.15 }))
            .collect();
        let with = gains(&means, &DeflickerConfig { strength: 1.0, window: 31, colour: true });
        let without = gains(&means, &DeflickerConfig { strength: 1.0, window: 31, colour: false });
        for i in 0..n {
            let m = means[i].unwrap();
            let y = |g: &[f64; 3]| 0.2126 * m.r * g[0] + 0.7152 * m.g * g[1] + 0.0722 * m.b * g[2];
            assert!((y(&with[i].gains) - y(&without[i].gains)).abs() < 1e-12, "frame {i}");
        }
    }

    #[test]
    fn strength_zero_changes_nothing_and_the_clamp_holds() {
        let mut means: Vec<_> = (0..50).map(|_| grey(0.2)).collect();
        means[25] = grey(2.0); // ten times brighter for one frame
        let none = gains(&means, &DeflickerConfig { strength: 0.0, ..Default::default() });
        assert!(none.iter().all(|g| g.gains == [1.0; 3]));
        let full = gains(&means, &DeflickerConfig { strength: 1.0, window: 9, colour: false });
        assert!(
            (full[25].gains[0] - 0.5).abs() < 1e-12,
            "capped at one stop: {:?}",
            full[25].gains
        );
        assert!((full[24].gains[0] - 1.0).abs() < 1e-12, "neighbours untouched");
    }

    #[test]
    fn colour_flicker_is_corrected_and_a_steady_cast_kept() {
        let n = 200;
        let e = noise(n, 11);
        // Red jumps around a warm cast; green and blue steady.
        let means: Vec<_> = (0..n)
            .map(|i| Some(ChannelMeans { r: 0.3 * (0.15 * e[i]).exp(), g: 0.2, b: 0.15 }))
            .collect();
        let cfg = DeflickerConfig { strength: 1.0, window: 31, colour: true };
        let g = gains(&means, &cfg);
        let ratio = |i: usize, gg: &FrameGains| {
            (means[i].unwrap().r * gg.gains[0] / (means[i].unwrap().g * gg.gains[1])).ln()
        };
        let before: Vec<f64> =
            (0..n).map(|i| (means[i].unwrap().r / means[i].unwrap().g).ln()).collect();
        let after: Vec<f64> = (0..n).map(|i| ratio(i, &g[i])).collect();
        assert!(stdev(&after) < stdev(&before) / 3.0);
        // A constant cast: nothing to correct.
        let steady: Vec<_> =
            (0..60).map(|_| Some(ChannelMeans { r: 0.3, g: 0.2, b: 0.1 })).collect();
        assert!(
            gains(&steady, &cfg).iter().all(|g| g.gains.iter().all(|v| (v - 1.0).abs() < 1e-12))
        );
        // Without colour, the channels share one gain.
        let mono = gains(&means, &DeflickerConfig { colour: false, ..cfg });
        assert!(mono.iter().all(|g| g.gains[0] == g.gains[1] && g.gains[1] == g.gains[2]));
    }

    #[test]
    fn unmeasured_frames_keep_unit_gain_and_do_not_disturb_the_others() {
        let mut means: Vec<_> = (0..40).map(|_| grey(0.2)).collect();
        means[10] = None;
        let g = gains(&means, &DeflickerConfig::default());
        assert_eq!(g[10].gains, [1.0; 3]);
        assert_eq!(g[10].luma, None);
        assert!(g.iter().all(|g| g.gains.iter().all(|v| (v - 1.0).abs() < 1e-12)));
        assert!(
            gains(&[None, None], &DeflickerConfig::default()).iter().all(|g| g.gains == [1.0; 3])
        );
    }

    #[test]
    fn measurement_ignores_clipped_and_black_pixels() {
        let mut img = RgbImage::from_pixel(10, 10, image::Rgb([128, 128, 128]));
        for x in 0..10 {
            img.put_pixel(x, 0, image::Rgb([255, 255, 255]));
            img.put_pixel(x, 1, image::Rgb([0, 0, 0]));
        }
        let m = measure(&img).unwrap();
        assert!((m.r - TO_LINEAR[128]).abs() < 1e-12 && m.r == m.g && m.g == m.b);
        assert_eq!(measure(&RgbImage::from_pixel(4, 4, image::Rgb([0, 0, 0]))), None);
    }

    #[test]
    fn lookup_tables_apply_gains_in_linear_light() {
        let id = luts([1.0; 3]);
        assert!(id.iter().all(|t| t.iter().enumerate().all(|(i, v)| usize::from(*v) == i)));
        let up = luts([2.0, 1.0, 0.5]);
        assert_eq!((up[0][0], up[2][0]), (0, 0), "black stays black");
        assert_eq!(up[0][255], 255, "clamped");
        assert!(up[0][100] > 100 && up[2][100] < 100);
        // One stop up doubles the linear value.
        let v = up[0][100];
        assert!((TO_LINEAR[usize::from(v)] / TO_LINEAR[100] - 2.0).abs() < 0.05);
        let mut px = vec![100, 100, 100];
        apply(&mut px, &up);
        assert_eq!(px, [up[0][100], 100, up[2][100]]);
    }

    #[test]
    fn the_same_input_gives_the_same_gains() {
        let means: Vec<_> = noise(500, 3).iter().map(|e| grey(0.3 * (0.2 * e).exp())).collect();
        let cfg = DeflickerConfig::default();
        assert_eq!(gains(&means, &cfg), gains(&means, &cfg));
    }

    #[test]
    fn the_configuration_is_clamped() {
        let c = DeflickerConfig { strength: 3.0, window: 4, colour: true }.clamped();
        assert_eq!((c.strength, c.window), (1.0, 5));
        let c = DeflickerConfig { strength: f64::NAN, window: 10_000, colour: false }.clamped();
        assert_eq!((c.strength, c.window), (0.0, MAX_WINDOW));
    }
}
