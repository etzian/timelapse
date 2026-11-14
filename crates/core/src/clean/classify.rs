//! Classification: a pure function of the stored measurements, the stored XMP rating and
//! the thresholds (ADR-020). Given the same inputs it always gives the same answer, which
//! makes every cleaning decision reproducible from SQLite alone. It only ever proposes
//! `eligible` or `flagged`: rejection and quarantine are the user's decisions (ADR-011).

use super::analysis::Luma;
use super::xmp;

/// User-configurable thresholds. Stored in `projects.settings_json` and copied into the
/// reason of every decision made with them.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Thresholds {
    /// Flag when the mean luma (0–255) is below this.
    pub min_mean_luma: f64,
    /// A pixel is "dark" when its luma (0–255) is at or below this…
    pub dark_level: u8,
    /// …and the image is flagged when more than this fraction (0–1) of its pixels are dark.
    pub max_dark_fraction: f64,
    /// Flag photos that Lightroom marked as rejected.
    pub flag_xmp_rejected: bool,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            min_mean_luma: 40.0,
            dark_level: 30,
            max_dark_fraction: 0.6,
            flag_xmp_rejected: true,
        }
    }
}

impl Thresholds {
    /// The same thresholds with every value forced into its valid range (a NaN takes the
    /// default), so bad input from the UI can never produce an unreproducible decision.
    pub fn sanitized(self) -> Self {
        let d = Self::default();
        let fix = |v: f64, d: f64, hi: f64| if v.is_nan() { d } else { v.clamp(0.0, hi) };
        Self {
            min_mean_luma: fix(self.min_mean_luma, d.min_mean_luma, 255.0),
            dark_level: self.dark_level,
            max_dark_fraction: fix(self.max_dark_fraction, d.max_dark_fraction, 1.0),
            flag_xmp_rejected: self.flag_xmp_rejected,
        }
    }
}

/// Why an image was flagged. Serialised into `image_status.reason` and mapped to Fluent
/// keys (`clean-reason-<kind>`) in the UI.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Reason {
    MeanBelow { mean: f64, min: f64 },
    TooManyDarkPixels { fraction: f64, max: f64, level: u8 },
    XmpRejected,
    AnalysisFailed { failure: String },
}

/// `eligible` when `reasons` is empty, otherwise `flagged`.
#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    pub reasons: Vec<Reason>,
}

impl Proposal {
    pub fn flagged(&self) -> bool {
        !self.reasons.is_empty()
    }
}

/// Classify one image. `luma` is `None` when the analysis failed with `failure`.
pub fn classify(
    luma: Option<&Luma>,
    failure: Option<&str>,
    xmp_rating: Option<i32>,
    t: &Thresholds,
) -> Proposal {
    let t = t.sanitized();
    let mut reasons = Vec::new();
    match luma {
        Some(l) => {
            let mean = l.mean();
            if mean < t.min_mean_luma {
                reasons.push(Reason::MeanBelow { mean, min: t.min_mean_luma });
            }
            let fraction = l.fraction_at_or_below(t.dark_level);
            if fraction > t.max_dark_fraction {
                reasons.push(Reason::TooManyDarkPixels {
                    fraction,
                    max: t.max_dark_fraction,
                    level: t.dark_level,
                });
            }
        }
        None => reasons
            .push(Reason::AnalysisFailed { failure: failure.unwrap_or("undecodable").to_owned() }),
    }
    if t.flag_xmp_rejected && xmp_rating == Some(xmp::REJECTED) {
        reasons.push(Reason::XmpRejected);
    }
    Proposal { reasons }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(y: u8, n: u64) -> Luma {
        let mut h = vec![0; 256];
        h[usize::from(y)] = n;
        Luma { histogram: h, width: 1, height: 1 }
    }

    #[test]
    fn a_bright_image_is_eligible() {
        let p = classify(Some(&uniform(150, 100)), None, None, &Thresholds::default());
        assert!(!p.flagged());
    }

    #[test]
    fn a_dark_image_is_flagged_for_both_reasons() {
        let p = classify(Some(&uniform(5, 100)), None, None, &Thresholds::default());
        assert_eq!(p.reasons.len(), 2);
        assert!(
            matches!(p.reasons[0], Reason::MeanBelow { mean, min } if mean == 5.0 && min == 40.0)
        );
        assert!(
            matches!(p.reasons[1], Reason::TooManyDarkPixels { fraction, .. } if fraction == 1.0)
        );
    }

    #[test]
    fn dark_fraction_uses_the_configured_level() {
        let mut h = vec![0; 256];
        h[20] = 70;
        h[200] = 30;
        let l = Luma { histogram: h, width: 10, height: 10 };
        let t = Thresholds { min_mean_luma: 0.0, ..Thresholds::default() };
        assert!(classify(Some(&l), None, None, &t).flagged(), "70 % at or below 30");
        let t = Thresholds { dark_level: 10, ..t };
        assert!(!classify(Some(&l), None, None, &t).flagged(), "none at or below 10");
    }

    #[test]
    fn lightroom_rejects_are_flagged_only_when_enabled() {
        let bright = uniform(200, 10);
        let on = Thresholds::default();
        let p = classify(Some(&bright), None, Some(-1), &on);
        assert_eq!(p.reasons, [Reason::XmpRejected]);
        let off = Thresholds { flag_xmp_rejected: false, ..on };
        assert!(!classify(Some(&bright), None, Some(-1), &off).flagged());
        assert!(!classify(Some(&bright), None, Some(0), &on).flagged());
    }

    #[test]
    fn an_analysis_failure_is_flagged_for_review() {
        let p = classify(None, Some("unreadable"), None, &Thresholds::default());
        assert_eq!(p.reasons, [Reason::AnalysisFailed { failure: "unreadable".into() }]);
    }

    #[test]
    fn classification_is_deterministic_and_sanitized() {
        let l = uniform(35, 7);
        let t =
            Thresholds { min_mean_luma: f64::NAN, max_dark_fraction: 7.0, ..Thresholds::default() };
        let a = classify(Some(&l), None, None, &t);
        let b = classify(Some(&l), None, None, &t);
        assert_eq!(a, b);
        assert!(matches!(a.reasons[0], Reason::MeanBelow { min, .. } if min == 40.0));
        assert_eq!(a.reasons.len(), 1, "max_dark_fraction clamped to 1.0 never trips");
    }

    #[test]
    fn reasons_serialize_with_a_stable_kind() {
        let v =
            serde_json::to_value(Reason::TooManyDarkPixels { fraction: 0.5, max: 0.4, level: 30 })
                .unwrap();
        assert_eq!(v["kind"], "too-many-dark-pixels");
        assert_eq!(serde_json::to_value(Reason::XmpRejected).unwrap()["kind"], "xmp-rejected");
        let back: Reason = serde_json::from_value(v).unwrap();
        assert!(matches!(back, Reason::TooManyDarkPixels { level: 30, .. }));
    }
}
