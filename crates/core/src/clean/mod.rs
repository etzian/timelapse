//! Phase 3: brightness analysis and cleaning (PROCESSING_PIPELINE.md §4, ADR-020).
//!
//! [`analysis`] measures luma, [`xmp`] reads Lightroom ratings, [`classify`] turns both
//! into a reproducible proposal, [`store`] persists measurements, proposals and user
//! decisions, and [`thumbs`] serves review thumbnails. Moving files into quarantine is
//! [`crate::files`]' job. Every public function takes the source `root`, like the import.

pub mod analysis;
pub mod classify;
pub mod store;
pub mod thumbs;
pub mod xmp;

use std::path::Path;

use rayon::prelude::*;

use crate::files::{self, FileOperation, OpOutcome};
use crate::import::open_root;
use crate::{Error, Result};

pub use classify::{Reason, Thresholds};
pub use store::{Decision, ReviewItem, StatusCounts};

/// Images measured per transaction (and per progress report).
const CHUNK: usize = 64;
/// At most this many images are decoded at once: a full-size decode can take ~100–300 MiB
/// (bounded per image by `analysis::decode_limits`).
const MAX_THREADS: usize = 3;

/// Reported after each chunk of an analysis run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct AnalyzeProgress {
    pub done: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AnalyzeReport {
    pub analyzed: usize,
    pub failed: usize,
    pub counts: StatusCounts,
}

/// Measure every image that has no current measurement, in parallel, then classify with
/// the stored thresholds. Idempotent: a second run measures nothing new.
pub fn analyze_for_root(
    root: &Path,
    mut on_progress: impl FnMut(AnalyzeProgress),
) -> Result<AnalyzeReport> {
    let conn = open_root(root)?;
    let pending = store::pending_analysis(&conn)?;
    let total = pending.len();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(MAX_THREADS);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|_| Error::StoreTransaction)?;
    let (mut done, mut failed) = (0, 0);
    on_progress(AnalyzeProgress { done, total });
    for chunk in pending.chunks(CHUNK) {
        let measured: Vec<store::Measured> =
            pool.install(|| chunk.par_iter().map(store::measure_file).collect());
        failed += measured.iter().filter(|m| m.luma.is_err()).count();
        store::save_measurements(&conn, &measured)?;
        done += chunk.len();
        on_progress(AnalyzeProgress { done, total });
    }
    Ok(AnalyzeReport { analyzed: total, failed, counts: store::classify_all(&conn)? })
}

pub fn thresholds_for_root(root: &Path) -> Result<Thresholds> {
    store::thresholds(&open_root(root)?)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ThresholdsApplied {
    /// The thresholds as stored (sanitised).
    pub thresholds: Thresholds,
    pub counts: StatusCounts,
}

/// Store new thresholds and re-propose every undecided image with them.
pub fn set_thresholds_for_root(root: &Path, t: Thresholds) -> Result<ThresholdsApplied> {
    let conn = open_root(root)?;
    let thresholds = store::set_thresholds(&conn, t)?;
    Ok(ThresholdsApplied { thresholds, counts: store::classify_all(&conn)? })
}

/// Re-propose with the stored thresholds (from SQLite alone).
pub fn classify_for_root(root: &Path) -> Result<StatusCounts> {
    store::classify_all(&open_root(root)?)
}

pub fn review_for_root(root: &Path, statuses: &[String]) -> Result<Vec<ReviewItem>> {
    store::review(&open_root(root)?, statuses)
}

pub fn decide_for_root(root: &Path, ids: &[i64], decision: Decision) -> Result<StatusCounts> {
    let conn = open_root(root)?;
    store::decide(&conn, ids, decision)?;
    store::status_counts(&conn)
}

pub fn quarantine_for_root(root: &Path, ids: &[i64]) -> Result<Vec<OpOutcome>> {
    files::quarantine(&open_root(root)?, root, ids)
}

pub fn restore_for_root(root: &Path, ids: &[i64]) -> Result<Vec<OpOutcome>> {
    files::restore(&open_root(root)?, ids)
}

pub fn operations_for_root(root: &Path, limit: usize) -> Result<Vec<FileOperation>> {
    files::list_operations(&open_root(root)?, limit.clamp(1, 1000))
}

/// A JPEG thumbnail of image `id` at its current location.
pub fn thumbnail_for_root(root: &Path, id: i64, size: u32) -> Result<Vec<u8>> {
    let conn = open_root(root)?;
    let (path, hash) = store::location(&conn, id)?.ok_or(Error::UnknownImage { id })?;
    drop(conn);
    thumbs::thumbnail(root, id, size, &path, hash.as_deref())
}

#[cfg(test)]
mod exit_tests;
