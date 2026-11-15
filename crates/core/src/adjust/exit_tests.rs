//! Phase 8 exit criterion (DEVELOPMENT_ROADMAP.md): flicker/stabilization results are
//! traceable in SQLite; re-running with identical config reproduces them; the encode
//! consumes them without a manifest file. Full path on real files: import → selection →
//! deflicker / stabilization → stored frames → (export, see the export tests).

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::*;
use crate::import::metadata::fixture::TempDir;
use crate::import::{backfill_for_root, import_folder};
use crate::sampling::{Config, Method, create_run_for_root};

/// The analyses share the process-wide lock; run these one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

pub(crate) fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) const W: u32 = 320;
pub(crate) const H: u32 = 240;
pub(crate) const FRAMES: usize = 24;

/// Camera shake in pixels of the 320×240 photographs, and exposure flicker in stops.
pub(crate) fn shake(i: usize) -> (f64, f64) {
    let s = [(0.0, 0.0), (5.0, -3.0), (-4.0, 2.0), (6.0, 4.0), (-2.0, -5.0), (3.0, 1.0)];
    s[i % s.len()]
}

pub(crate) fn flicker(i: usize) -> f64 {
    [0.0, 0.35, -0.3, 0.25, -0.2, 0.3, -0.35, 0.1][i % 8]
}

/// A smooth, aperiodic scene seen through a camera at offset `(ox, oy)`, at exposure
/// `2^stops`.
pub(crate) fn scene(ox: f64, oy: f64, stops: f64) -> RgbImage {
    let gain = 2f64.powf(stops);
    let blobs: Vec<(f64, f64, f64)> = (0..30)
        .map(|i| {
            let t = i as f64;
            ((t * 61.7) % f64::from(W), (t * 37.3 + 17.0) % f64::from(H), 4.0 + (t * 2.3) % 9.0)
        })
        .collect();
    RgbImage::from_fn(W, H, |x, y| {
        let (fx, fy) = (f64::from(x) - ox, f64::from(y) - oy);
        let mut v =
            0.5 + 0.15 * (0.09 * fx + 0.061 * fy).sin() + 0.1 * (0.043 * fx - 0.087 * fy).cos();
        for (bx, by, r) in &blobs {
            let d2 = (fx - bx).powi(2) + (fy - by).powi(2);
            v += 0.25 * (-d2 / (2.0 * r * r)).exp();
        }
        let lin = (v * 0.35 * gain).clamp(0.0, 1.0);
        let srgb = |l: f64| {
            ((if l <= 0.003_130_8 { l * 12.92 } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 }) * 255.0)
                .round() as u8
        };
        image::Rgb([srgb(lin * 1.05), srgb(lin), srgb(lin * 0.9)])
    })
}

pub(crate) struct Fixture {
    pub dir: TempDir,
    pub run: i64,
}

impl Fixture {
    pub(crate) fn new(tag: &str) -> Self {
        let dir = TempDir::new(&format!("p8-{tag}"));
        let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        for i in 0..FRAMES {
            let (dx, dy) = shake(i);
            let p = dir.path().join(format!("f{i:03}.png"));
            scene(dx, dy, flicker(i)).save(&p).unwrap();
            let f = fs::File::options().write(true).open(&p).unwrap();
            f.set_modified(t0 + Duration::from_secs(60 * i as u64)).unwrap();
        }
        import_folder(dir.path()).unwrap();
        backfill_for_root(dir.path()).unwrap();
        let run = create_run_for_root(
            dir.path(),
            &Config {
                method: Method::EveryNth { n: 1, offset: 0 },
                from: None,
                to: None,
                include_flagged: true,
            },
        )
        .unwrap()
        .id;
        Self { dir, run }
    }

    pub(crate) fn root(&self) -> &Path {
        self.dir.path()
    }
}

fn stdev(v: &[f64]) -> f64 {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
}

#[test]
fn deflicker_is_stored_reproducible_and_reduces_flicker() {
    let _s = serial();
    let f = Fixture::new("deflicker");
    let cfg = DeflickerConfig { strength: 1.0, window: 9, colour: true };
    let mut seen = Vec::new();
    let a = deflicker_for_root(f.root(), f.run, &cfg, 0, |p| seen.push(p)).unwrap().unwrap();
    assert_eq!(seen.last().map(|p| (p.done, p.total)), Some((FRAMES, FRAMES)));
    assert_eq!((a.results.frames, a.results.measured), (FRAMES, FRAMES));
    let b = deflicker_for_root(f.root(), f.run, &cfg, 0, |_| {}).unwrap().unwrap();
    assert_ne!(a.id, b.id);
    let conn = open_root(f.root()).unwrap();
    let (fa, fb) =
        (store::flicker_frames(&conn, a.id).unwrap(), store::flicker_frames(&conn, b.id).unwrap());
    assert_eq!(fa, fb, "identical configuration, identical stored values");
    assert_eq!(
        a.sequence_fingerprint,
        sampling_store::sequence_fingerprint(&conn, f.run).unwrap().0
    );

    let before: Vec<f64> = fa.iter().map(|x| x.means.unwrap().luma().ln()).collect();
    let after: Vec<f64> = fa
        .iter()
        .map(|x| {
            let m = x.means.unwrap();
            (0.2126 * m.r * x.gains[0] + 0.7152 * m.g * x.gains[1] + 0.0722 * m.b * x.gains[2]).ln()
        })
        .collect();
    assert!(stdev(&after) < stdev(&before) / 3.0, "{} → {}", stdev(&before), stdev(&after));
    // A frame shot 0.35 stop bright is darkened by about that much.
    assert!(
        (fa[1].gains[1].ln() / std::f64::consts::LN_2 + 0.35).abs() < 0.12,
        "{:?}",
        fa[1].gains
    );

    let list = adjustments_for_root(f.root()).unwrap();
    assert_eq!(list.flicker.iter().map(|r| r.id).collect::<Vec<_>>(), [b.id, a.id]);
}

#[test]
fn stabilization_recovers_the_shake_and_is_reproducible() {
    let _s = serial();
    let f = Fixture::new("stabilize");
    let cfg = StabilizeConfig { smoothing: 9, max_crop: 0.05 };
    let a = stabilize_for_root(f.root(), f.run, &cfg, 0, |_| {}).unwrap().unwrap();
    let conn = open_root(f.root()).unwrap();
    let frames = store::stabilization_frames(&conn, a.id).unwrap();
    assert_eq!(frames.len(), FRAMES);
    assert_eq!(frames[0].measured, None);
    // Content moves by the camera offset difference; in fractions of the frame.
    for (i, frame) in frames.iter().enumerate().skip(1) {
        let (x0, y0) = shake(i - 1);
        let (x1, y1) = shake(i);
        let m = frame.measured.unwrap_or_else(|| panic!("frame {i} not measured"));
        let (ex, ey) = ((x1 - x0) / f64::from(W), (y1 - y0) / f64::from(H));
        assert!(
            (m.0 - ex).abs() < 0.3 / f64::from(W) && (m.1 - ey).abs() < 0.3 / f64::from(H),
            "frame {i}: {m:?} vs ({ex}, {ey})"
        );
    }
    // Shifting by the correction puts every frame close to the smoothed path.
    for fr in &frames {
        let out = (fr.path.0 + fr.shift.0, fr.path.1 + fr.shift.1);
        assert!((out.0 - fr.smoothed.0).abs() < 1e-12 || fr.shift.0.abs() == a.crop);
        assert!((out.1 - fr.smoothed.1).abs() < 1e-12 || fr.shift.1.abs() == a.crop);
    }
    assert!(a.crop > 0.0 && a.crop <= 0.05);
    let b = stabilize_for_root(f.root(), f.run, &cfg, 0, |_| {}).unwrap().unwrap();
    assert_eq!(store::stabilization_frames(&conn, b.id).unwrap(), frames, "reproducible");
    assert_eq!(a.crop, b.crop);
}

#[test]
fn the_export_takes_its_frames_from_the_stored_runs() {
    use crate::export::frames::prepare_adjusted;
    use crate::export::{Codec, ExportSettings, Quality};
    let _s = serial();
    let f = Fixture::new("export-input");
    let fl = deflicker_for_root(
        f.root(),
        f.run,
        &DeflickerConfig { strength: 1.0, window: 9, colour: true },
        0,
        |_| {},
    )
    .unwrap()
    .unwrap();
    let st = stabilize_for_root(
        f.root(),
        f.run,
        &StabilizeConfig { smoothing: 9, max_crop: 0.05 },
        0,
        |_| {},
    )
    .unwrap()
    .unwrap();
    let conn = open_root(f.root()).unwrap();
    let settings = ExportSettings {
        codec: Codec::Av1,
        fps: 24.0,
        speed: 1.0,
        width: 320,
        height: 240,
        quality: Quality::Small,
        flicker_run: Some(fl.id),
        stabilization_run: Some(st.id),
    };
    let adjust = crate::export::adjustments(&conn, f.run, &settings, FRAMES).unwrap();
    let gains = store::flicker_frames(&conn, fl.id).unwrap();
    let shifts = store::stabilization_frames(&conn, st.id).unwrap();
    for i in 0..FRAMES {
        let expect = (gains[i].gains != [1.0; 3]).then(|| deflicker::luts(gains[i].gains));
        assert_eq!(adjust[i].luts, expect, "frame {i}");
        assert_eq!(adjust[i].shift, Some((shifts[i].shift.0, shifts[i].shift.1, st.crop)));
    }

    // The prepared frames are steadier and more even than the photographs.
    let items = crate::sampling::store::run_items(&conn, f.run).unwrap();
    let path =
        |i: usize| crate::clean::store::location(&conn, items[i].image_id).unwrap().unwrap().0;
    let frame = |i: usize, adjusted: bool| {
        let a = if adjusted { adjust[i] } else { crate::export::frames::FrameAdjust::NONE };
        RgbImage::from_raw(320, 240, prepare_adjusted(&path(i), 320, 240, &a).unwrap()).unwrap()
    };
    let luma = |img: &RgbImage| deflicker::measure(img).unwrap().luma().ln();
    let spread =
        |adjusted: bool| stdev(&(0..8).map(|i| luma(&frame(i, adjusted))).collect::<Vec<_>>());
    assert!(spread(true) < spread(false) / 2.0, "{} → {}", spread(false), spread(true));
    let motion = |adjusted: bool| {
        (1..8)
            .map(|i| {
                let s = stabilize::estimate(
                    &stabilize::gray(&frame(i - 1, adjusted)),
                    &stabilize::gray(&frame(i, adjusted)),
                );
                s.dx.abs() + s.dy.abs()
            })
            .sum::<f64>()
    };
    assert!(motion(true) < motion(false) / 2.0, "{} → {}", motion(false), motion(true));

    // Runs of another selection are refused.
    let other = create_run_for_root(
        f.root(),
        &Config {
            method: Method::EveryNth { n: 2, offset: 0 },
            from: None,
            to: None,
            include_flagged: true,
        },
    )
    .unwrap()
    .id;
    assert!(matches!(
        crate::export::adjustments(&conn, other, &settings, FRAMES / 2),
        Err(Error::Export { reason: "adjustment-mismatch" })
    ));
    let unknown = ExportSettings { flicker_run: Some(999), stabilization_run: None, ..settings };
    assert!(matches!(
        crate::export::adjustments(&conn, f.run, &unknown, FRAMES),
        Err(Error::Export { reason: "adjustment-mismatch" })
    ));
}

#[test]
fn cancel_stores_nothing_and_one_analysis_runs_at_a_time() {
    let _s = serial();
    let f = Fixture::new("cancel");
    let token = 8_080;
    cancel(token);
    assert_eq!(
        deflicker_for_root(f.root(), f.run, &DeflickerConfig::default(), token, |_| {}).unwrap(),
        None
    );
    assert_eq!(
        stabilize_for_root(f.root(), f.run, &StabilizeConfig::default(), token, |_| {}).unwrap(),
        None
    );
    cancel(0);
    let list = adjustments_for_root(f.root()).unwrap();
    assert!(list.flicker.is_empty() && list.stabilization.is_empty());

    let _held = try_lock().unwrap();
    assert!(matches!(
        deflicker_for_root(f.root(), f.run, &DeflickerConfig::default(), 0, |_| {}),
        Err(Error::Adjust { reason: "busy" })
    ));
    drop(_held);
    assert!(matches!(
        stabilize_for_root(f.root(), 999, &StabilizeConfig::default(), 0, |_| {}),
        Err(Error::UnknownRun { id: 999 })
    ));
}
