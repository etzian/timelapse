//! Phase 8 (ADR-025): deflicker and stabilization. Per-frame values are computed for a
//! saved selection from the cached review thumbnails, stored in SQLite as a complete run,
//! and applied by the export while it prepares frames (D5); no manifest file exists.

pub mod deflicker;
pub mod preview;
pub mod series;
pub mod stabilize;
pub mod store;

#[cfg(test)]
pub(crate) mod exit_tests;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

use image::RgbImage;
use rayon::prelude::*;

use crate::clean::thumbs;
use crate::import::open_root;
use crate::sampling::store as sampling_store;
use crate::{Error, Result};

pub use deflicker::DeflickerConfig;
pub use stabilize::StabilizeConfig;
pub use store::{FlickerRun, FrameRef, StabilizationRun};

static ADJUST_LOCK: Mutex<()> = Mutex::new(());
static CANCEL_TOKEN: AtomicU64 = AtomicU64::new(0);

/// Thumbnails decoded at once: a missing one is made from the original (a full-size
/// decode), so the pool stays small (as in the cleaning analysis).
const MAX_THREADS: usize = 3;
/// Frames per batch (and per progress report).
const BATCH: usize = 32;
/// Thumbnail sizes: deflicker needs averages only; stabilization needs detail.
const DEFLICKER_THUMB: u32 = 256;
const STABILIZE_THUMB: u32 = 1024;

fn fail(reason: &'static str) -> Error {
    Error::Adjust { reason }
}

fn try_lock() -> Option<MutexGuard<'static, ()>> {
    match ADJUST_LOCK.try_lock() {
        Ok(g) => Some(g),
        Err(TryLockError::Poisoned(p)) => Some(p.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

/// Ask the analysis started with `token` to stop after its current batch; nothing of it
/// is stored.
pub fn cancel(token: u64) {
    CANCEL_TOKEN.store(token, Ordering::SeqCst);
}

fn cancelled(token: u64) -> bool {
    token != 0 && CANCEL_TOKEN.load(Ordering::SeqCst) == token
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct AdjustProgress {
    pub done: usize,
    pub total: usize,
}

/// One frame of a selection: what to store, and where its photograph is.
struct Frame {
    r: FrameRef,
    location: PathBuf,
    hash: Option<String>,
}

fn frames_of(conn: &rusqlite::Connection, run: i64) -> Result<Vec<Frame>> {
    let items = sampling_store::run_items(conn, run)?;
    if items.is_empty() {
        return Err(fail("empty-run"));
    }
    items
        .into_iter()
        .map(|i| {
            let (location, hash) =
                crate::clean::store::location(conn, i.image_id)?.ok_or(Error::StoreTransaction)?;
            Ok(Frame {
                r: FrameRef { sequence_index: i.sequence_index, image_id: i.image_id },
                location,
                hash,
            })
        })
        .collect()
}

/// The review thumbnail of a frame, decoded; `None` when it cannot be read.
fn thumb(root: &Path, f: &Frame, size: u32) -> Option<RgbImage> {
    let jpeg = thumbs::thumbnail(root, f.r.image_id, size, &f.location, f.hash.as_deref()).ok()?;
    image::load_from_memory(&jpeg).ok().map(|i| i.to_rgb8())
}

fn pool() -> Result<rayon::ThreadPool> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(MAX_THREADS);
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|_| Error::StoreTransaction)
}

/// Measure every frame of saved selection `run` and store a deflicker run. `Ok(None)`
/// when it was cancelled (nothing is stored).
pub fn deflicker_for_root(
    root: &Path,
    run: i64,
    config: &DeflickerConfig,
    token: u64,
    mut on_progress: impl FnMut(AdjustProgress),
) -> Result<Option<FlickerRun>> {
    let _guard = try_lock().ok_or_else(|| fail("busy"))?;
    let _work = crate::erase::job(root);
    let config = config.clamped();
    let conn = open_root(root)?;
    let frames = frames_of(&conn, run)?;
    let (fingerprint, _) = sampling_store::sequence_fingerprint(&conn, run)?;
    let pool = pool()?;
    let total = frames.len();
    let mut means = Vec::with_capacity(total);
    on_progress(AdjustProgress { done: 0, total });
    for batch in frames.chunks(BATCH) {
        if cancelled(token) {
            return Ok(None);
        }
        let measured: Vec<_> = pool.install(|| {
            batch
                .par_iter()
                .map(|f| thumb(root, f, DEFLICKER_THUMB).and_then(|i| deflicker::measure(&i)))
                .collect()
        });
        means.extend(measured);
        on_progress(AdjustProgress { done: means.len(), total });
    }
    let gains = deflicker::gains(&means, &config);
    let refs: Vec<FrameRef> = frames.iter().map(|f| f.r).collect();
    store::insert_flicker(&conn, run, &config, &fingerprint, &refs, &means, &gains).map(Some)
}

/// Measure the motion between consecutive frames of saved selection `run` and store a
/// stabilization run. `Ok(None)` when it was cancelled (nothing is stored).
pub fn stabilize_for_root(
    root: &Path,
    run: i64,
    config: &StabilizeConfig,
    token: u64,
    mut on_progress: impl FnMut(AdjustProgress),
) -> Result<Option<StabilizationRun>> {
    let _guard = try_lock().ok_or_else(|| fail("busy"))?;
    let _work = crate::erase::job(root);
    let config = config.clamped();
    let conn = open_root(root)?;
    let frames = frames_of(&conn, run)?;
    let (fingerprint, _) = sampling_store::sequence_fingerprint(&conn, run)?;
    let pool = pool()?;
    let total = frames.len();
    let no_motion = stabilize::Shift { dx: 0.0, dy: 0.0, confidence: 0.0 };
    let mut pairs: Vec<stabilize::Shift> = Vec::with_capacity(total.saturating_sub(1));
    let mut previous: Option<stabilize::Gray> = None;
    let mut done = 0;
    on_progress(AdjustProgress { done, total });
    for batch in frames.chunks(BATCH) {
        if cancelled(token) {
            return Ok(None);
        }
        let grays: Vec<Option<stabilize::Gray>> = pool.install(|| {
            batch
                .par_iter()
                .map(|f| thumb(root, f, STABILIZE_THUMB).map(|i| stabilize::gray(&i)))
                .collect()
        });
        // Each frame against the one before it (the last of the previous batch for the
        // first of this one).
        let before: Vec<Option<&stabilize::Gray>> = std::iter::once(previous.as_ref())
            .chain(grays.iter().map(Option::as_ref))
            .take(grays.len())
            .collect();
        let measured: Vec<Option<stabilize::Shift>> = pool.install(|| {
            before
                .par_iter()
                .zip(grays.par_iter())
                .map(|(a, b)| match (a, b) {
                    (Some(a), Some(b)) => {
                        Some(stabilize::as_fraction(stabilize::estimate(a, b), a))
                    }
                    _ => None,
                })
                .collect()
        });
        let first = done == 0;
        for (k, m) in measured.into_iter().enumerate() {
            if first && k == 0 {
                continue; // the very first frame has no predecessor
            }
            pairs.push(m.unwrap_or(no_motion));
        }
        previous = grays.into_iter().last().flatten();
        done += batch.len();
        on_progress(AdjustProgress { done, total });
    }
    let result = stabilize::stabilize(&pairs, &config);
    let refs: Vec<FrameRef> = frames.iter().map(|f| f.r).collect();
    store::insert_stabilization(
        &conn,
        run,
        &config,
        &fingerprint,
        &refs,
        &result.frames,
        result.crop,
    )
    .map(Some)
}

/// Every stored run of both kinds, newest first.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Adjustments {
    pub flicker: Vec<FlickerRun>,
    pub stabilization: Vec<StabilizationRun>,
}

pub fn adjustments_for_root(root: &Path) -> Result<Adjustments> {
    let conn = open_root(root)?;
    Ok(Adjustments {
        flicker: store::flicker_runs(&conn)?,
        stabilization: store::stabilization_runs(&conn)?,
    })
}
