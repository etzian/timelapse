//! Smoothing helpers shared by deflicker and stabilization (ADR-025).
//!
//! A centred window cannot be centred at the ends of a sequence. Shrinking it there would
//! leave the first and last frames uncorrected (a window of one is the value itself), so
//! the series is extended at each end along a straight line fitted (least squares) to its
//! nearest window. A straight trend is kept exactly; a jump at an end is corrected like
//! any other.

/// `values` extended by `pad` points at each end along a robust line through the
/// `fit` values nearest that end (callers use three windows, so the line reflects the
/// local trend rather than the noise of a few frames): the line through the medians of the window's two
/// halves (at their centres). A single outlier at the end cannot tilt it, and a straight
/// series gives its own line exactly.
fn extend(values: &[f64], pad: usize, fit: usize) -> Vec<f64> {
    let n = values.len();
    let line = |start: usize, len: usize| -> (f64, f64) {
        if len < 2 {
            return (0.0, values[start]);
        }
        let h = len / 2;
        let centre = |a: usize, b: usize| (a + b - 1) as f64 / 2.0;
        let (xa, ya) = (centre(start, start + h), median_of(&values[start..start + h]));
        let (xb, yb) = (
            centre(start + len - h, start + len),
            median_of(&values[start + len - h..start + len]),
        );
        let slope = if xb > xa { (yb - ya) / (xb - xa) } else { 0.0 };
        (slope, ya - slope * xa)
    };
    let m = fit.clamp(1, n);
    let (ls, li) = line(0, m);
    let (rs, ri) = line(n - m, m);
    let mut out = Vec::with_capacity(n + 2 * pad);
    out.extend((1..=pad).rev().map(|k| li - ls * k as f64));
    out.extend_from_slice(values);
    out.extend((1..=pad).map(|k| ri + rs * (n - 1 + k) as f64));
    out
}

/// The median (the mean of the two middle values for an even count, so the median of a
/// straight run lies exactly on it).
fn median_of(window: &[f64]) -> f64 {
    let mut w = window.to_vec();
    w.sort_by(f64::total_cmp);
    let n = w.len();
    if n % 2 == 1 { w[n / 2] } else { (w[n / 2 - 1] + w[n / 2]) / 2.0 }
}

/// Centred running median then running mean over `window` (odd) values, full windows
/// everywhere thanks to the linear extension.
pub fn trend(values: &[f64], window: usize) -> Vec<f64> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    let half = window / 2;
    let pad = 2 * half;
    let ext = extend(values, pad, 3 * window);
    let len = ext.len();
    let median: Vec<f64> = (0..len)
        .map(|i| {
            let h = half.min(i).min(len - 1 - i);
            median_of(&ext[i - h..=i + h])
        })
        .collect();
    (pad..pad + n)
        .map(|i| median[i - half..=i + half].iter().sum::<f64>() / (2 * half + 1) as f64)
        .collect()
}

/// Centred running mean over `window` (odd) values, full windows everywhere.
pub fn mean(values: &[f64], window: usize) -> Vec<f64> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    let half = window / 2;
    let ext = extend(values, half, 3 * window);
    (half..half + n)
        .map(|i| ext[i - half..=i + half].iter().sum::<f64>() / (2 * half + 1) as f64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_straight_line_is_kept_exactly_including_the_ends() {
        let line: Vec<f64> = (0..50).map(|i| 3.0 - 0.07 * i as f64).collect();
        for w in [3, 9, 31, 61] {
            for (a, b) in trend(&line, w).iter().zip(&line) {
                assert!((a - b).abs() < 1e-9, "window {w}");
            }
            for (a, b) in mean(&line, w).iter().zip(&line) {
                assert!((a - b).abs() < 1e-9, "window {w}");
            }
        }
    }

    #[test]
    fn a_jump_at_either_end_is_smoothed_like_any_other() {
        let mut v = vec![1.0; 40];
        v[0] = 2.0;
        v[39] = 0.0;
        let t = trend(&v, 9);
        assert!((t[0] - 1.0).abs() < 0.2, "first frame follows its neighbours: {}", t[0]);
        assert!((t[39] - 1.0).abs() < 0.2, "{}", t[39]);
        let m = mean(&v, 9);
        assert!((m[0] - 1.0).abs() < 0.5 && (m[39] - 1.0).abs() < 0.5);
    }

    #[test]
    fn short_and_empty_series() {
        assert!(trend(&[], 9).is_empty());
        assert_eq!(trend(&[0.5], 9), [0.5]);
        assert_eq!(mean(&[0.5, 0.5], 31), [0.5, 0.5]);
    }
}
