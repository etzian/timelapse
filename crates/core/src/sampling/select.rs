//! Temporal sampling algorithms (PROCESSING_PIPELINE.md §6, PRD §5.6, ADR-010).
//!
//! Pure functions over frames already in chronological order, `(epoch_ms, path)`. The same
//! frames and the same method always give the same selection. Two rules hold for every
//! method:
//!
//! - **no duplicates, monotone:** each pick comes later in the sequence than the previous
//!   one, so a photograph is never selected twice and the output stays chronological;
//! - **ties go to the earlier frame**, so the result never depends on iteration order.

/// Bumped whenever a selection rule changes; stored with every run.
pub const ALGORITHM_VERSION: i64 = 1;
/// Upper bound on generated targets (a 1 s interval over ~58 days).
pub const MAX_TARGETS: u64 = 5_000_000;

/// One candidate photograph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub image_id: i64,
    pub at_ms: i64,
}

/// How a fixed-interval target picks its photograph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Policy {
    Closest,
    FirstAfter,
    LastBefore,
}

/// A sampling method and its parameters.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "method", rename_all = "kebab-case")]
pub enum Method {
    /// A target every `interval_seconds`, from the first photograph on.
    FixedInterval {
        interval_seconds: f64,
        policy: Policy,
        /// Furthest a match may be from its target; defaults to half the interval for
        /// `closest` and to one interval otherwise.
        #[serde(default)]
        tolerance_seconds: Option<f64>,
    },
    /// Every `n`th photograph in chronological order, starting at index `offset`.
    EveryNth {
        n: u32,
        #[serde(default)]
        offset: u32,
    },
    /// `count` photographs spread evenly over the first-to-last span, matched closest.
    Uniform { count: u32 },
}

impl Method {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FixedInterval { .. } => "fixed-interval",
            Self::EveryNth { .. } => "every-nth",
            Self::Uniform { .. } => "uniform",
        }
    }
}

/// Why a configuration cannot run. The UI maps it to `error-invalid-sampling` with this
/// reason as a selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    NonPositive,
    TooManyTargets,
}

impl Invalid {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NonPositive => "non-positive",
            Self::TooManyTargets => "too-many-targets",
        }
    }
}

/// One selected photograph, in output order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Pick {
    pub image_id: i64,
    /// The target this photograph answers (`None` for every-Nth).
    pub target_ms: Option<i64>,
    pub actual_ms: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Selection {
    pub picks: Vec<Pick>,
    /// Targets no photograph matched (within tolerance, without reusing one).
    pub unmatched_targets: Vec<i64>,
    pub target_count: usize,
}

pub fn select(frames: &[Frame], method: &Method) -> Result<Selection, Invalid> {
    match *method {
        Method::FixedInterval { interval_seconds, policy, tolerance_seconds } => {
            fixed_interval(frames, interval_seconds, policy, tolerance_seconds)
        }
        Method::EveryNth { n, offset } => every_nth(frames, n, offset),
        Method::Uniform { count } => uniform(frames, count),
    }
}

/// How far from its target a match may lie.
#[derive(Clone, Copy)]
enum Reach {
    /// `|frame - target| <= ms` (an explicit tolerance, or closest's half interval).
    AtMost(i64),
    /// `|frame - target| < ms`: first-after / last-before by default stay strictly within one
    /// interval, so a step never takes the photograph that sits exactly on the next step.
    Below(i64),
    Any,
}

impl Reach {
    fn admits(self, distance: i128) -> bool {
        match self {
            Self::AtMost(ms) => distance <= i128::from(ms),
            Self::Below(ms) => distance < i128::from(ms),
            Self::Any => true,
        }
    }
}

/// Seconds → whole milliseconds; everything below is integer arithmetic, so an interval
/// typed as 1.1 h is exactly 3 960 000 ms and no step is lost to rounding.
fn to_ms(seconds: f64) -> Result<i64, Invalid> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(Invalid::NonPositive);
    }
    let ms = (seconds * 1000.0).round();
    if ms > i64::MAX as f64 { Ok(i64::MAX) } else { Ok(ms as i64) }
}

fn fixed_interval(
    frames: &[Frame],
    interval_s: f64,
    policy: Policy,
    tolerance_s: Option<f64>,
) -> Result<Selection, Invalid> {
    let interval_ms = to_ms(interval_s)?;
    if interval_ms < 1 {
        return Err(Invalid::NonPositive);
    }
    let reach = match (tolerance_s, policy) {
        (Some(t), _) => Reach::AtMost(to_ms(t)?),
        (None, Policy::Closest) => Reach::AtMost(interval_ms / 2),
        (None, _) => Reach::Below(interval_ms),
    };
    let (Some(first), Some(last)) = (frames.first(), frames.last()) else {
        return Ok(Selection::default());
    };
    let span = i128::from(last.at_ms) - i128::from(first.at_ms);
    let n_targets = span / i128::from(interval_ms) + 1;
    if n_targets > i128::from(MAX_TARGETS) {
        return Err(Invalid::TooManyTargets);
    }
    let start = first.at_ms;
    // k * interval <= span, so every target lies within [first, last] and fits in i64.
    let targets = (0..n_targets as i64).map(move |k| start + k * interval_ms);
    Ok(match_targets(frames, targets, policy, reach, None))
}

fn every_nth(frames: &[Frame], n: u32, offset: u32) -> Result<Selection, Invalid> {
    if n == 0 {
        return Err(Invalid::NonPositive);
    }
    let picks = frames
        .iter()
        .skip(offset as usize)
        .step_by(n as usize)
        .map(|f| Pick { image_id: f.image_id, target_ms: None, actual_ms: f.at_ms })
        .collect();
    Ok(Selection { picks, unmatched_targets: Vec::new(), target_count: 0 })
}

fn uniform(frames: &[Frame], count: u32) -> Result<Selection, Invalid> {
    if count == 0 {
        return Err(Invalid::NonPositive);
    }
    if u64::from(count) > MAX_TARGETS {
        return Err(Invalid::TooManyTargets);
    }
    let (Some(first), Some(last)) = (frames.first(), frames.last()) else {
        return Ok(Selection::default());
    };
    let span = i128::from(last.at_ms) - i128::from(first.at_ms);
    let targets: Vec<i64> = if count == 1 {
        vec![first.at_ms]
    } else {
        let d = i128::from(count - 1);
        // Rounded integer division: first + round(span * k / (count - 1)).
        (0..count)
            .map(|k| first.at_ms + ((span * i128::from(k) * 2 + d) / (2 * d)) as i64)
            .collect()
    };
    // When there are enough photographs, every target gets one: each pick leaves at least
    // as many photographs as targets remain.
    if count as usize >= frames.len() {
        return Ok(every_frame_on_spread_targets(frames, &targets));
    }
    let reserve = Some(count as usize);
    Ok(match_targets(frames, targets.into_iter(), Policy::Closest, Reach::Any, reserve))
}

/// At least as many targets as photographs: every photograph is used, frame `k` answering
/// target `round(k * (targets - 1) / (frames - 1))`. Those indices strictly increase (the
/// step is at least 1), so each target gets at most one photograph; the rest are unmatched.
fn every_frame_on_spread_targets(frames: &[Frame], targets: &[i64]) -> Selection {
    let (n, m) = (frames.len() as u64, targets.len() as u64);
    let mut used = vec![false; targets.len()];
    let picks = frames
        .iter()
        .enumerate()
        .map(|(k, f)| {
            let j = if n <= 1 {
                0
            } else {
                ((k as u64 * (m - 1) * 2 + (n - 1)) / (2 * (n - 1))) as usize
            };
            used[j] = true;
            Pick { image_id: f.image_id, target_ms: Some(targets[j]), actual_ms: f.at_ms }
        })
        .collect();
    Selection {
        picks,
        unmatched_targets: targets
            .iter()
            .zip(&used)
            .filter(|(_, u)| !**u)
            .map(|(t, _)| *t)
            .collect(),
        target_count: targets.len(),
    }
}

/// Match increasing targets to frames: each pick lies after the previous pick in the
/// sequence (`next`), so nothing is reused and the output stays in order. With
/// `reserve = Some(total)`, a pick never consumes frames the remaining targets need.
fn match_targets(
    frames: &[Frame],
    targets: impl Iterator<Item = i64>,
    policy: Policy,
    reach: Reach,
    reserve: Option<usize>,
) -> Selection {
    let mut out = Selection::default();
    let mut next = 0usize;
    for (j, t) in targets.enumerate() {
        out.target_count += 1;
        let rest = &frames[next..];
        // First index (within `rest`) at or after the target, and last index at or before it.
        let after = rest.partition_point(|f| f.at_ms < t);
        let before = rest.partition_point(|f| f.at_ms <= t).checked_sub(1);
        let distance = |i: usize| (i128::from(rest[i].at_ms) - i128::from(t)).abs();
        let mut candidate = match policy {
            Policy::FirstAfter => (after < rest.len()).then_some(after),
            Policy::LastBefore => before,
            Policy::Closest => match (before, (after < rest.len()).then_some(after)) {
                (Some(b), Some(a)) => {
                    let (db, da) = (distance(b), distance(a));
                    // Equal distance (including a burst exactly on the target): the frame
                    // earlier in the sequence.
                    Some(if db < da {
                        b
                    } else if da < db {
                        a
                    } else {
                        b.min(a)
                    })
                }
                (b, a) => b.or(a),
            },
        };
        if let Some(total) = reserve {
            let still_needed = total - j - 1;
            candidate = rest
                .len()
                .checked_sub(1 + still_needed)
                .map(|max| candidate.unwrap_or(max).min(max));
        }
        match candidate.filter(|&i| reach.admits(distance(i))) {
            Some(i) => {
                let f = &rest[i];
                out.picks.push(Pick {
                    image_id: f.image_id,
                    target_ms: Some(t),
                    actual_ms: f.at_ms,
                });
                next += i + 1;
            }
            None => out.unmatched_targets.push(t),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: i64 = 1000;
    const H: i64 = 3600 * S;

    /// Frames at the given times (seconds), ids 1, 2, 3…
    fn frames(secs: &[i64]) -> Vec<Frame> {
        secs.iter()
            .enumerate()
            .map(|(i, s)| Frame { image_id: i as i64 + 1, at_ms: s * S })
            .collect()
    }

    fn ids(s: &Selection) -> Vec<i64> {
        s.picks.iter().map(|p| p.image_id).collect()
    }

    fn fixed(secs: f64, policy: Policy) -> Method {
        Method::FixedInterval { interval_seconds: secs, policy, tolerance_seconds: None }
    }

    #[test]
    fn fixed_interval_closest_picks_the_nearest_frame_per_target() {
        // Targets at 0, 10, 20, 30.
        let f = frames(&[0, 4, 9, 12, 21, 29, 31]);
        let s = select(&f, &fixed(10.0, Policy::Closest)).unwrap();
        assert_eq!(ids(&s), [1, 3, 5, 6]);
        assert_eq!(s.picks[1].target_ms, Some(10 * S));
        assert_eq!(s.picks[1].actual_ms, 9 * S);
        assert!(s.unmatched_targets.is_empty());
        assert_eq!(s.target_count, 4);
    }

    #[test]
    fn first_after_and_last_before_policies() {
        let f = frames(&[0, 4, 9, 12, 21, 29, 31]);
        assert_eq!(ids(&select(&f, &fixed(10.0, Policy::FirstAfter)).unwrap()), [1, 4, 5, 7]);
        assert_eq!(ids(&select(&f, &fixed(10.0, Policy::LastBefore)).unwrap()), [1, 3, 4, 6]);
    }

    #[test]
    fn a_gap_leaves_targets_unmatched_and_reported() {
        // Daily targets over 5 days; nothing on days 2 and 3.
        let f = frames(&[0, 24 * 3600, 4 * 24 * 3600]);
        let s = select(&f, &fixed(24.0 * 3600.0, Policy::Closest)).unwrap();
        assert_eq!(ids(&s), [1, 2, 3]);
        assert_eq!(s.unmatched_targets, [48 * H, 72 * H]);
    }

    #[test]
    fn tolerance_limits_how_far_a_match_may_be() {
        let f = frames(&[0, 7, 20]);
        let strict = Method::FixedInterval {
            interval_seconds: 10.0,
            policy: Policy::Closest,
            tolerance_seconds: Some(2.0),
        };
        let s = select(&f, &strict).unwrap();
        assert_eq!(ids(&s), [1, 3]);
        assert_eq!(s.unmatched_targets, [10 * S]);
        // Default for closest is half the interval: 7 s is 3 s from the 10 s target → ok.
        assert_eq!(ids(&select(&f, &fixed(10.0, Policy::Closest)).unwrap()), [1, 2, 3]);
    }

    #[test]
    fn ties_go_to_the_earlier_frame() {
        let f = frames(&[0, 5, 15, 20]);
        // Target 10: frames at 5 and 15 are equally close.
        let s = select(&f, &fixed(10.0, Policy::Closest)).unwrap();
        assert_eq!(ids(&s), [1, 2, 4]);
    }

    #[test]
    fn a_photograph_is_never_selected_twice() {
        // A burst at 0 and one late frame: dense targets must not reuse frames.
        let f = frames(&[0, 0, 0, 100]);
        let s = select(
            &f,
            &Method::FixedInterval {
                interval_seconds: 1.0,
                policy: Policy::Closest,
                tolerance_seconds: Some(1000.0),
            },
        )
        .unwrap();
        let picked = ids(&s);
        let mut dedup = picked.clone();
        dedup.sort_unstable();
        dedup.dedup();
        assert_eq!(picked.len(), dedup.len());
        // t=0 takes the first of the burst, t=1 the last one at or before it, t=2 the 100 s frame.
        assert_eq!(picked, [1, 3, 4]);
        assert_eq!(s.unmatched_targets.len(), 101 - 3);
        // And the output is chronological.
        assert!(s.picks.windows(2).all(|w| w[0].actual_ms <= w[1].actual_ms));
    }

    #[test]
    fn every_nth_with_offset() {
        let f = frames(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(ids(&select(&f, &Method::EveryNth { n: 3, offset: 0 }).unwrap()), [1, 4, 7, 10]);
        assert_eq!(ids(&select(&f, &Method::EveryNth { n: 4, offset: 1 }).unwrap()), [2, 6, 10]);
        assert_eq!(ids(&select(&f, &Method::EveryNth { n: 1, offset: 0 }).unwrap()).len(), 10);
        assert!(select(&f, &Method::EveryNth { n: 2, offset: 50 }).unwrap().picks.is_empty());
    }

    #[test]
    fn uniform_spreads_over_the_period() {
        let f = frames(&(0..=100).collect::<Vec<_>>());
        let s = select(&f, &Method::Uniform { count: 5 }).unwrap();
        assert_eq!(ids(&s), [1, 26, 51, 76, 101]);
        // More requested than available: every frame once, the rest unmatched.
        let few = frames(&[0, 50, 100]);
        let s = select(&few, &Method::Uniform { count: 5 }).unwrap();
        assert_eq!(ids(&s), [1, 2, 3]);
        assert_eq!(s.unmatched_targets, [25 * S, 75 * S]);
        assert_eq!(s.picks[1].target_ms, Some(50 * S));
        // Even when the photographs are bunched up, all of them are used.
        let bunched = frames(&[0, 1, 2, 3, 100]);
        let s = select(&bunched, &Method::Uniform { count: 9 }).unwrap();
        assert_eq!(ids(&s), [1, 2, 3, 4, 5]);
        assert_eq!(s.picks.len() + s.unmatched_targets.len(), 9);
        assert!(s.picks.windows(2).all(|w| w[0].target_ms < w[1].target_ms));
        assert_eq!(ids(&select(&few, &Method::Uniform { count: 1 }).unwrap()), [1]);
    }

    #[test]
    fn empty_input_and_invalid_configurations() {
        assert_eq!(select(&[], &fixed(10.0, Policy::Closest)).unwrap(), Selection::default());
        assert_eq!(select(&[], &Method::Uniform { count: 3 }).unwrap(), Selection::default());
        let f = frames(&[0, 10]);
        assert_eq!(select(&f, &fixed(0.0, Policy::Closest)), Err(Invalid::NonPositive));
        assert_eq!(select(&f, &fixed(f64::NAN, Policy::Closest)), Err(Invalid::NonPositive));
        assert_eq!(select(&f, &Method::EveryNth { n: 0, offset: 0 }), Err(Invalid::NonPositive));
        assert_eq!(select(&f, &Method::Uniform { count: 0 }), Err(Invalid::NonPositive));
        let neg = Method::FixedInterval {
            interval_seconds: 1.0,
            policy: Policy::Closest,
            tolerance_seconds: Some(-1.0),
        };
        assert_eq!(select(&f, &neg), Err(Invalid::NonPositive));
        let huge = frames(&[0, 365 * 24 * 3600]);
        assert_eq!(select(&huge, &fixed(1.0, Policy::Closest)), Err(Invalid::TooManyTargets));
    }

    #[test]
    fn decimal_intervals_lose_no_step() {
        // 1.1 h = 3960 s (3960.0000000000005 in floating point); frames exactly on each step.
        let f = frames(&[0, 3960, 7920, 11880]);
        let s = select(&f, &fixed(1.1 * 3600.0, Policy::Closest)).unwrap();
        assert_eq!(s.target_count, 4);
        assert_eq!(ids(&s), [1, 2, 3, 4]);
        // A sweep of decimal intervals over exact multiples: the last frame is always a target.
        for tenths in 1..200 {
            let interval = f64::from(tenths) * 0.1;
            let ms = (interval * 1000.0).round() as i64;
            let fr: Vec<Frame> = (0..5).map(|k| Frame { image_id: k + 1, at_ms: k * ms }).collect();
            let s = select(&fr, &fixed(interval, Policy::Closest)).unwrap();
            assert_eq!(ids(&s), [1, 2, 3, 4, 5], "interval {interval}");
        }
        assert_eq!(select(&f, &fixed(0.0004, Policy::Closest)), Err(Invalid::NonPositive));
    }

    #[test]
    fn first_after_and_last_before_never_take_the_next_steps_photo() {
        let f = frames(&[0, 20]);
        let s = select(&f, &fixed(10.0, Policy::FirstAfter)).unwrap();
        assert_eq!(ids(&s), [1, 2]);
        assert_eq!(s.picks[1].target_ms, Some(20 * S), "20 s answers its own step");
        assert_eq!(s.unmatched_targets, [10 * S]);
        let s = select(&f, &fixed(10.0, Policy::LastBefore)).unwrap();
        assert_eq!(s.picks[0].target_ms, Some(0));
        assert_eq!(s.picks[1].target_ms, Some(20 * S));
        // An explicit tolerance is inclusive.
        let wide = Method::FixedInterval {
            interval_seconds: 10.0,
            policy: Policy::FirstAfter,
            tolerance_seconds: Some(10.0),
        };
        assert_eq!(select(&f, &wide).unwrap().picks[1].target_ms, Some(10 * S));
    }

    #[test]
    fn uniform_returns_exactly_n_when_there_are_enough_photographs() {
        let f = frames(&[0, 1, 2, 3, 12]);
        let s = select(&f, &Method::Uniform { count: 4 }).unwrap();
        assert_eq!(s.picks.len(), 4);
        assert!(s.unmatched_targets.is_empty());
        // Random-ish sets: always exactly N, chronological, no duplicates.
        let mut seed = 7u64;
        for _ in 0..500 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let n = (seed >> 33) as usize % 40 + 1;
            let mut secs: Vec<i64> = (0..n).map(|k| ((seed >> (k % 30)) % 1000) as i64).collect();
            secs.sort_unstable();
            let fr = frames(&secs);
            let count = ((seed >> 20) as u32 % n as u32) + 1;
            let s = select(&fr, &Method::Uniform { count }).unwrap();
            assert_eq!(s.picks.len(), count as usize);
            assert!(s.picks.windows(2).all(|w| w[0].image_id < w[1].image_id));
        }
    }

    #[test]
    fn selection_is_deterministic() {
        let f = frames(&[0, 3, 3, 7, 11, 11, 19, 25, 26, 40]);
        for m in [
            fixed(5.0, Policy::Closest),
            Method::EveryNth { n: 2, offset: 1 },
            Method::Uniform { count: 4 },
        ] {
            assert_eq!(select(&f, &m), select(&f, &m));
        }
    }

    #[test]
    fn methods_round_trip_through_json() {
        let m = Method::FixedInterval {
            interval_seconds: 60.0,
            policy: Policy::LastBefore,
            tolerance_seconds: None,
        };
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(v["method"], "fixed-interval");
        assert_eq!(v["policy"], "last-before");
        assert_eq!(serde_json::from_value::<Method>(v).unwrap(), m);
        let m: Method = serde_json::from_str(r#"{"method":"every-nth","n":3}"#).unwrap();
        assert_eq!(m, Method::EveryNth { n: 3, offset: 0 });
    }
}
