//! Phase 5: FFmpeg video generation (PROCESSING_PIPELINE.md §10, ADR-017, ADR-021, ADR-022).
//!
//! A saved sampling run becomes an MP4 in `<root>/.timelapse/exports/`:
//!
//! 1. refuse a tier whose encoder the bundled FFmpeg lacks (never substitute one);
//! 2. decode the selected photographs in Rust and pipe them as raw RGB (paths never
//!    reach FFmpeg);
//! 3. FFmpeg writes `<name>.mp4.part` with `-n` (no overwrite);
//! 4. ffprobe checks codec, size and frame count;
//! 5. a no-clobber rename gives `<name>.mp4`.
//!
//! Every export is recorded in `video_exports` with its settings, FFmpeg version, argv and
//! result. One export runs at a time and can be cancelled.

pub mod ffmpeg;
pub mod frames;
pub mod record;
pub mod settings;
pub mod store;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

use rayon::prelude::*;

use crate::import::open_root;
use crate::sampling::store as sampling_store;
use crate::{Error, Result};

pub use ffmpeg::{FfmpegInfo, FfmpegPaths, Probed};
pub use settings::{Codec, ExportSettings, Quality, duration_seconds, frame_plan, output_frames};
pub use store::ExportRecord;

static EXPORT_LOCK: Mutex<()> = Mutex::new(());
/// The token of the export the user asked to stop (0 = none). Each export carries its own
/// token, so a cancel can never be lost to, or leak into, another export.
static CANCEL_TOKEN: AtomicU64 = AtomicU64::new(0);
/// Frames decoded ahead in parallel.
const DECODE_AHEAD: usize = 6;

pub use crate::layout::exports_dir;

fn fail(reason: &'static str) -> Error {
    Error::Export { reason }
}

/// The export lock, or `None` while an export runs. A panic in an earlier export only
/// poisons the lock; the lock guards no data, so it is taken back.
fn try_export_lock() -> Option<MutexGuard<'static, ()>> {
    match EXPORT_LOCK.try_lock() {
        Ok(g) => Some(g),
        Err(TryLockError::Poisoned(p)) => Some(p.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ExportProgress {
    pub done: usize,
    pub total: usize,
}

/// What the bundled FFmpeg offers (used by the UI to enable tiers).
pub fn info(paths: &FfmpegPaths) -> FfmpegInfo {
    ffmpeg::probe(paths)
}

/// Ask the export started with `token` to stop; it removes its partial file.
pub fn cancel(token: u64) {
    CANCEL_TOKEN.store(token, Ordering::SeqCst);
}

fn cancelled(token: u64) -> bool {
    token != 0 && CANCEL_TOKEN.load(Ordering::SeqCst) == token
}

pub fn exports_for_root(root: &Path) -> Result<Vec<ExportRecord>> {
    let conn = open_root(root)?;
    // Only while no export runs (the guard is held across the update): otherwise a live
    // `running` row is not abandoned.
    if let Some(_idle) = try_export_lock() {
        store::mark_interrupted(&conn)?;
    }
    store::list(&conn)
}

/// If anything unwinds out of an export, the partial file goes and the row says so.
struct Cleanup<'a> {
    conn: &'a rusqlite::Connection,
    id: i64,
    part: PathBuf,
    armed: bool,
}

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.part);
            let _ = store::failed(self.conn, self.id, "interrupted", "");
        }
    }
}

/// Encode saved sampling run `run` of `root` into an MP4. `token` identifies this export
/// for [`cancel`].
pub fn export_run(
    root: &Path,
    paths: &FfmpegPaths,
    run: i64,
    settings: ExportSettings,
    token: u64,
    mut on_progress: impl FnMut(ExportProgress),
) -> Result<ExportRecord> {
    let _guard = try_export_lock().ok_or_else(|| fail("busy"))?;
    let _work = crate::erase::job(root);
    settings.check().map_err(fail)?;

    let conn = open_root(root)?;
    store::mark_interrupted(&conn)?;
    let items = sampling_store::run_items(&conn, run)?;
    if items.is_empty() {
        return Err(fail("empty-run"));
    }
    // A run of another selection is a settings error: say so before looking for FFmpeg.
    let adjust =
        adjustments(&conn, run, settings.flicker_run, settings.stabilization_run, items.len())?;
    let probe = ffmpeg::probe(paths);
    if !probe.available {
        return Err(fail("ffmpeg-missing"));
    }
    if !probe.supports(settings.codec) {
        return Err(fail("encoder-unavailable"));
    }
    let sources: Vec<PathBuf> = items
        .iter()
        .map(|i| crate::clean::store::location(&conn, i.image_id).map(|l| l.map(|(p, _)| p)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|p| p.ok_or(Error::StoreTransaction))
        .collect::<Result<_>>()?;
    let plan = frame_plan(sources.len(), settings.speed);

    let dir = exports_dir(root);
    fs::create_dir_all(&dir).map_err(|_| fail("write-failed"))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let name = format!("selection-{run}-{}-{stamp}.mp4", settings.codec.probe_name());
    let output = dir.join(&name);
    // The partial file is temporary data (ADR-026); only the finished video goes to the exports.
    let temp = crate::layout::temp_dir(root);
    fs::create_dir_all(&temp).map_err(|_| fail("write-failed"))?;
    let part = temp.join(format!("{name}.part"));
    let args = ffmpeg::encode_args(&settings, &part);
    let id =
        store::begin(&conn, run, &settings, &probe, &args, &output, sources.len(), plan.len())?;
    // A `.part` already there is someone else's file: refuse before arming the cleanup.
    if part.exists() {
        store::failed(&conn, id, "output-exists", "")?;
        return store::get(&conn, id)?.ok_or(Error::StoreTransaction);
    }
    let mut cleanup = Cleanup { conn: &conn, id, part: part.clone(), armed: true };

    let result =
        encode(paths, &args, &sources, &adjust, &plan, &settings, &part, token, &mut on_progress);
    let outcome = match result {
        Ok(()) => finish(paths, &settings, &part, &output, plan.len()),
        Err(e) => Err(e),
    };
    cleanup.armed = false;
    match outcome {
        Ok(probed) => {
            let size = fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
            store::complete(&conn, id, &probed, size)?;
        }
        Err((reason, tail)) => {
            // `output-exists` means the file was there before us: it is not ours to remove.
            if reason != "output-exists" {
                let _ = fs::remove_file(&part);
            }
            store::failed(&conn, id, reason, &tail)?;
        }
    }
    store::get(&conn, id)?.ok_or(Error::StoreTransaction)
}

type Failure = (&'static str, String);

/// A deflicker or stabilization run may be applied to selection `run` only if it was
/// measured on it (`adjustment-mismatch` otherwise, also when it does not cover every
/// photograph) and on the same bytes: the selection's sequence fingerprint must be the
/// one the run recorded (`adjustment-stale` when an original changed since).
fn check_adjustment(
    conn: &rusqlite::Connection,
    run: i64,
    measured_on: i64,
    fingerprint: &str,
    frames: usize,
    photos: usize,
) -> Result<()> {
    if measured_on != run || frames != photos {
        return Err(fail("adjustment-mismatch"));
    }
    if sampling_store::sequence_fingerprint(conn, run)?.0 != fingerprint {
        return Err(fail("adjustment-stale"));
    }
    Ok(())
}

fn frame_count(conn: &rusqlite::Connection, table: &str, id: i64) -> Result<usize> {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table} WHERE run_id = ?1"), [id], |r| {
        r.get::<_, i64>(0)
    })
    .map(|n| n as usize)
    .map_err(|_| Error::StoreTransaction)
}

/// Per-photograph adjustments of the chosen deflicker and stabilization runs, read from
/// SQLite (D5), after [`check_adjustment`].
pub(crate) fn adjustments(
    conn: &rusqlite::Connection,
    run: i64,
    flicker_run: Option<i64>,
    stabilization_run: Option<i64>,
    photos: usize,
) -> Result<Vec<frames::FrameAdjust>> {
    use crate::adjust::{deflicker, store as adjust_store};
    let mut out = vec![frames::FrameAdjust::NONE; photos];
    if let Some(id) = flicker_run {
        let r = adjust_store::flicker_run(conn, id)?.ok_or_else(|| fail("adjustment-mismatch"))?;
        let frames = adjust_store::flicker_frames(conn, id)?;
        check_adjustment(
            conn,
            run,
            r.sampling_run_id,
            &r.sequence_fingerprint,
            frames.len(),
            photos,
        )?;
        for (slot, f) in out.iter_mut().zip(&frames) {
            slot.luts = (f.gains != [1.0; 3]).then(|| deflicker::luts(f.gains));
        }
    }
    if let Some(id) = stabilization_run {
        let r = adjust_store::stabilization_run(conn, id)?
            .ok_or_else(|| fail("adjustment-mismatch"))?;
        let frames = adjust_store::stabilization_frames(conn, id)?;
        check_adjustment(
            conn,
            run,
            r.sampling_run_id,
            &r.sequence_fingerprint,
            frames.len(),
            photos,
        )?;
        for (slot, f) in out.iter_mut().zip(&frames) {
            slot.shift = (r.crop > 0.0).then_some((f.shift.0, f.shift.1, r.crop));
        }
    }
    Ok(out)
}

/// The adjustment of one photograph (position `index`) only, with the same checks: what a
/// preview needs, without reading or building every frame's.
pub(crate) fn adjustment_at(
    conn: &rusqlite::Connection,
    run: i64,
    index: usize,
    flicker_run: Option<i64>,
    stabilization_run: Option<i64>,
    photos: usize,
) -> Result<frames::FrameAdjust> {
    use crate::adjust::{deflicker, store as adjust_store};
    let mut out = frames::FrameAdjust::NONE;
    if let Some(id) = flicker_run {
        let r = adjust_store::flicker_run(conn, id)?.ok_or_else(|| fail("adjustment-mismatch"))?;
        let count = frame_count(conn, "flicker_frames", id)?;
        check_adjustment(conn, run, r.sampling_run_id, &r.sequence_fingerprint, count, photos)?;
        let f = adjust_store::flicker_frame(conn, id, index)?
            .ok_or_else(|| fail("adjustment-mismatch"))?;
        out.luts = (f.gains != [1.0; 3]).then(|| deflicker::luts(f.gains));
    }
    if let Some(id) = stabilization_run {
        let r = adjust_store::stabilization_run(conn, id)?
            .ok_or_else(|| fail("adjustment-mismatch"))?;
        let count = frame_count(conn, "stabilization_frames", id)?;
        check_adjustment(conn, run, r.sampling_run_id, &r.sequence_fingerprint, count, photos)?;
        let f = adjust_store::stabilization_frame(conn, id, index)?
            .ok_or_else(|| fail("adjustment-mismatch"))?;
        out.shift = (r.crop > 0.0).then_some((f.shift.0, f.shift.1, r.crop));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn encode(
    paths: &FfmpegPaths,
    args: &[std::ffi::OsString],
    sources: &[PathBuf],
    adjust: &[frames::FrameAdjust],
    plan: &[usize],
    s: &ExportSettings,
    part: &Path,
    token: u64,
    on_progress: &mut impl FnMut(ExportProgress),
) -> std::result::Result<(), Failure> {
    if part.exists() {
        return Err(("output-exists", String::new()));
    }
    let mut enc =
        ffmpeg::Encoder::start(paths, args).map_err(|_| ("ffmpeg-failed", String::new()))?;
    let total = plan.len();
    on_progress(ExportProgress { done: 0, total });
    let mut last: Option<(usize, Vec<u8>)> = None;
    let mut k = 0;
    while k < total {
        // Decode the next distinct photographs in parallel, then write in order.
        let mut wanted: Vec<usize> = Vec::new();
        for &i in &plan[k..] {
            if wanted.last() != Some(&i) && last.as_ref().is_none_or(|(l, _)| *l != i) {
                wanted.push(i);
            }
            if wanted.len() == DECODE_AHEAD {
                break;
            }
        }
        let decoded: Vec<(usize, std::result::Result<Vec<u8>, Error>)> = wanted
            .par_iter()
            .map(|&i| (i, frames::prepare_adjusted(&sources[i], s.width, s.height, &adjust[i])))
            .collect();
        let mut decoded = decoded.into_iter().peekable();
        while k < total {
            if cancelled(token) {
                return Err(("cancelled", enc.kill()));
            }
            let i = plan[k];
            if last.as_ref().is_none_or(|(l, _)| *l != i) {
                match decoded.next_if(|(j, _)| *j == i) {
                    Some((_, Ok(rgb))) => last = Some((i, rgb)),
                    Some((_, Err(_))) => return Err(("image-unreadable", enc.kill())),
                    None => break, // refill the decode-ahead batch
                }
            }
            let rgb = &last.as_ref().map(|(_, b)| b).ok_or(("ffmpeg-failed", String::new()))?;
            if enc.write_frame(rgb).is_err() {
                let (_, tail) = enc.finish();
                return Err(("ffmpeg-failed", tail));
            }
            k += 1;
            if k % 8 == 0 || k == total {
                on_progress(ExportProgress { done: k, total });
            }
        }
    }
    let (ok, tail) = enc.finish();
    if ok { Ok(()) } else { Err(("ffmpeg-failed", tail)) }
}

/// Validate the `.part` file and move it into place without replacing anything.
fn finish(
    paths: &FfmpegPaths,
    s: &ExportSettings,
    part: &Path,
    output: &Path,
    frames: usize,
) -> std::result::Result<Probed, Failure> {
    let probed = ffmpeg::validate(paths, part).ok_or(("invalid-output", String::new()))?;
    let valid = probed.codec_name == s.codec.probe_name()
        && (probed.width, probed.height) == (s.width, s.height)
        && probed.packets == frames as u64;
    if !valid {
        return Err(("invalid-output", format!("{probed:?}")));
    }
    // No-clobber placement: a hard link, or (no links, e.g. exFAT) a `create_new` copy.
    // Both fail if a file already has that name; `rename` (which replaces) is never used.
    match fs::hard_link(part, output) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(("output-exists", String::new()));
        }
        Err(_) => copy_new(part, output)?,
    }
    let _ = fs::remove_file(part);
    Ok(probed)
}

fn copy_new(from: &Path, to: &Path) -> std::result::Result<(), Failure> {
    let mut out = match fs::OpenOptions::new().write(true).create_new(true).open(to) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(("output-exists", String::new()));
        }
        Err(_) => return Err(("write-failed", String::new())),
    };
    let copied = fs::File::open(from)
        .and_then(|mut i| std::io::copy(&mut i, &mut out))
        .and_then(|_| out.sync_all());
    if copied.is_err() {
        drop(out);
        let _ = fs::remove_file(to); // the copy we just created
        return Err(("write-failed", String::new()));
    }
    Ok(())
}

#[cfg(test)]
mod exit_tests;
