//! Phase 6: consolidation (PRD §5.5, PROCESSING_PIPELINE.md §5 and §12, ADR-023).
//!
//! Photographs are copied or moved from the collection into a working folder the user
//! picked, outside the collection:
//!
//! 1. [`preview`] plans the targets (naming, collisions, duplicates, what is already
//!    there, bytes vs free space) without writing anything, and returns a fingerprint;
//! 2. [`execute`] re-plans, refuses a plan that differs from the confirmed one, and places
//!    each photograph through the journaled, no-clobber steps of [`crate::files`];
//! 3. [`undo`] removes the copies we made (only while unmodified) or moves the originals
//!    back (only while still where the run put them);
//! 4. [`verify_run`] and [`verify_originals`] re-hash what was written and the originals.

pub mod plan;
pub mod store;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

use rayon::prelude::*;
use rusqlite::Connection;

use crate::files::{self, Kind, Step};
use crate::import::{dedup_hash, open_root};
use crate::sampling::store as sampling_store;
use crate::{Error, Result};

pub use plan::{Action, Existing, Naming, Operation, Planned};
pub use store::{PlanCounts, RunRecord};

/// One consolidation or undo at a time (they also hold the file-operation lock).
static RUN_LOCK: Mutex<()> = Mutex::new(());
/// The token of the run the user asked to stop (0 = none), as for exports.
static CANCEL_TOKEN: AtomicU64 = AtomicU64::new(0);
/// Items a preview sends to the UI (the counts always cover the whole plan).
pub const PREVIEW_ITEMS: usize = 200;
/// Problem entries a verification report lists (the counts are always exact).
pub const REPORT_ITEMS: usize = 200;

fn fail(reason: &'static str) -> Error {
    Error::Consolidation { reason }
}

fn db(_: rusqlite::Error) -> Error {
    Error::StoreTransaction
}

fn try_run_lock() -> Option<MutexGuard<'static, ()>> {
    match RUN_LOCK.try_lock() {
        Ok(g) => Some(g),
        Err(TryLockError::Poisoned(p)) => Some(p.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

/// Ask the run (or undo) started with `token` to stop after the current photograph.
pub fn cancel(token: u64) {
    CANCEL_TOKEN.store(token, Ordering::SeqCst);
}

fn cancelled(token: u64) -> bool {
    token != 0 && CANCEL_TOKEN.load(Ordering::SeqCst) == token
}

/// What to consolidate, where and how.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// An absolute folder outside the collection (it may not exist yet).
    pub destination: PathBuf,
    pub operation: Operation,
    pub naming: Naming,
    /// A saved sampling selection, in its order; `None` = every photograph except rejected
    /// and quarantined ones, chronologically.
    #[serde(default)]
    pub sampling_run: Option<i64>,
}

/// Photographs left out of the input, by reason.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Excluded {
    pub rejected: usize,
    pub quarantined: usize,
    /// Not at their location of record.
    pub missing: usize,
}

/// The dry run.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Preview {
    pub config: Config,
    pub fingerprint: String,
    pub input_count: usize,
    pub excluded: Excluded,
    pub counts: PlanCounts,
    /// Placed under another name than the natural one.
    pub renamed: usize,
    /// Free bytes at the destination, when the platform tells.
    pub free_bytes: Option<u64>,
    pub enough_space: bool,
    /// The first [`PREVIEW_ITEMS`] of the plan, in order.
    pub items: Vec<Planned>,
    /// Up to [`PREVIEW_ITEMS`] items that are not a plain placement (duplicates, already
    /// present, unsafe names, renamed), so the dry run can list what is skipped.
    pub notable: Vec<Planned>,
}

/// Progress of a run or an undo, after each photograph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub failed: usize,
}

/// Reject a destination that is not absolute, not a folder, or overlaps the collection.
/// Returns the destination as given (lexically clean) for storage.
fn check_destination(root: &Path, dest: &Path) -> Result<PathBuf> {
    let clean = dest.is_absolute()
        && dest.components().all(|c| !matches!(c, Component::ParentDir | Component::CurDir));
    if !clean {
        return Err(fail("destination-invalid"));
    }
    if let Ok(meta) = fs::metadata(dest)
        && !meta.is_dir()
    {
        return Err(fail("destination-invalid"));
    }
    let root_c = fs::canonicalize(root).map_err(|_| fail("destination-invalid"))?;
    let dest_c = canonical_prefix(dest).ok_or_else(|| fail("destination-invalid"))?;
    // Inside the collection, copies would be re-imported; an ancestor of it could receive
    // targets inside it (`preserve` naming).
    if dest_c.starts_with(&root_c) || root_c.starts_with(&dest_c) {
        return Err(fail("destination-overlaps-collection"));
    }
    Ok(dest.to_path_buf())
}

/// `dest` canonicalised through its nearest existing ancestor, the rest appended.
fn canonical_prefix(dest: &Path) -> Option<PathBuf> {
    let mut existing = dest;
    let mut rest = Vec::new();
    while fs::symlink_metadata(existing).is_err() {
        rest.push(existing.file_name()?);
        existing = existing.parent()?;
    }
    let mut out = fs::canonicalize(existing).ok()?;
    out.extend(rest.into_iter().rev());
    Some(out)
}

/// The ordered input and what was left out.
fn inputs(conn: &Connection, cfg: &Config) -> Result<(Vec<plan::Input>, Excluded)> {
    struct Row {
        id: i64,
        location: String,
        relative: String,
        size: i64,
        hash: Option<String>,
        status: Option<String>,
    }
    let read = |r: &rusqlite::Row| {
        Ok(Row {
            id: r.get(0)?,
            location: r.get(1)?,
            relative: r.get(2)?,
            size: r.get(3)?,
            hash: r.get(4)?,
            status: r.get(5)?,
        })
    };
    let cols = "i.id, COALESCE(i.current_path, i.original_path), i.relative_path, i.file_size,
                i.file_hash, s.status";
    let rows: Vec<Row> = match cfg.sampling_run {
        None => {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {cols} FROM images i LEFT JOIN image_status s ON s.image_id = i.id
                     ORDER BY i.normalized_datetime IS NULL, i.normalized_datetime, i.original_path"
                ))
                .map_err(db)?;
            let it = stmt.query_map([], read).map_err(db)?;
            it.collect::<rusqlite::Result<_>>().map_err(db)?
        }
        Some(run) => {
            // Fails with `unknown-run` if the selection is gone.
            sampling_store::run_items(conn, run)?;
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {cols} FROM sampling_items si JOIN images i ON i.id = si.image_id
                     LEFT JOIN image_status s ON s.image_id = i.id
                     WHERE si.sampling_run_id = ?1 ORDER BY si.sequence_index"
                ))
                .map_err(db)?;
            let it = stmt.query_map([run], read).map_err(db)?;
            it.collect::<rusqlite::Result<_>>().map_err(db)?
        }
    };
    let mut ex = Excluded::default();
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        match r.status.as_deref() {
            Some("rejected") => ex.rejected += 1,
            Some("quarantined") => ex.quarantined += 1,
            _ if !Path::new(&r.location).is_file() => ex.missing += 1,
            _ => out.push(plan::Input {
                image_id: r.id,
                source: PathBuf::from(r.location),
                relative_path: r.relative,
                size: r.size.max(0) as u64,
                hash: r.hash,
            }),
        }
    }
    Ok((out, ex))
}

/// What is at `target` under `dest` for photograph `input`.
fn probe(dest: &Path, target: &str, input: &plan::Input) -> Existing {
    let Some(path) = plan::join_target(dest, target) else {
        return Existing::Other;
    };
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Existing::Absent,
        Ok(m) if m.is_file() && m.len() == input.size && input.hash.is_some() => {
            if dedup_hash(&path).ok() == input.hash { Existing::Identical } else { Existing::Other }
        }
        _ => Existing::Other,
    }
}

struct Computed {
    cfg: Config,
    input_count: usize,
    excluded: Excluded,
    items: Vec<Planned>,
    counts: PlanCounts,
    renamed: usize,
    fingerprint: String,
    free_bytes: Option<u64>,
}

impl Computed {
    fn enough_space(&self) -> bool {
        self.free_bytes.is_none_or(|free| self.counts.bytes <= free)
    }
}

fn compute(conn: &Connection, root: &Path, cfg: &Config) -> Result<Computed> {
    let destination = check_destination(root, &cfg.destination)?;
    let cfg = Config { destination, ..cfg.clone() };
    let (ins, excluded) = inputs(conn, &cfg)?;
    let items = plan::plan(&ins, cfg.naming, |t, i| probe(&cfg.destination, t, i));
    let mut counts = PlanCounts::default();
    let mut renamed = 0;
    for p in &items {
        match p.action {
            Action::Place => {
                counts.planned += 1;
                counts.bytes += p.size;
                renamed += usize::from(p.renamed);
            }
            Action::Duplicate { .. } => counts.duplicates += 1,
            Action::AlreadyPresent => counts.already_present += 1,
            Action::Unsafe => counts.unsafe_names += 1,
        }
    }
    let fingerprint = plan::fingerprint(cfg.operation, &cfg.destination, &items);
    let free_bytes = files::free_space(&cfg.destination);
    Ok(Computed {
        input_count: ins.len(),
        excluded,
        items,
        counts,
        renamed,
        fingerprint,
        free_bytes,
        cfg,
    })
}

/// The dry run: nothing is written.
pub fn preview(root: &Path, cfg: &Config) -> Result<Preview> {
    let conn = open_root(root)?;
    let c = compute(&conn, root, cfg)?;
    let notable = c
        .items
        .iter()
        .filter(|p| p.action != Action::Place || p.renamed)
        .take(PREVIEW_ITEMS)
        .cloned()
        .collect();
    Ok(Preview {
        enough_space: c.enough_space(),
        items: c.items.iter().take(PREVIEW_ITEMS).cloned().collect(),
        notable,
        config: c.cfg,
        fingerprint: c.fingerprint,
        input_count: c.input_count,
        excluded: c.excluded,
        counts: c.counts,
        renamed: c.renamed,
        free_bytes: c.free_bytes,
    })
}

/// If anything unwinds out of a run, the row says it was interrupted (the journal rows
/// left `pending` are settled by reconciliation).
struct Interrupted<'a> {
    conn: &'a Connection,
    id: i64,
    armed: bool,
    /// Folders this run created, so an undo removes those and nothing else.
    created: BTreeSet<PathBuf>,
}

impl Drop for Interrupted<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = store::set_created_dirs(self.conn, self.id, &self.created);
            let _ = store::finish(self.conn, self.id, "interrupted");
        }
    }
}

/// Record the folders that do not exist yet and that placing `path` will create: its
/// parents up to `base` (exclusive), or `path` itself and its missing parents when `base`
/// is `None` (the destination folder).
fn note_missing_dirs(path: &Path, base: Option<&Path>, out: &mut BTreeSet<PathBuf>) {
    let mut d = match base {
        Some(_) => path.parent(),
        None => Some(path),
    };
    while let Some(dir) = d {
        if base.is_some_and(|b| dir == b || !dir.starts_with(b)) || dir.exists() {
            break;
        }
        out.insert(dir.to_path_buf());
        d = dir.parent();
    }
}

/// Carry out the plan the user confirmed (`fingerprint` from [`preview`]). `token`
/// identifies this run for [`cancel`].
pub fn execute(
    root: &Path,
    cfg: &Config,
    fingerprint: &str,
    token: u64,
    mut on_progress: impl FnMut(Progress),
) -> Result<RunRecord> {
    let _run = try_run_lock().ok_or_else(|| fail("busy"))?;
    let conn = open_root(root)?;
    let _ops = files::lock();
    // Nothing else runs now: `pending` journal rows and `running` runs are a crash's
    // leftovers (the open may have skipped them while another project held the lock).
    files::settle_locked(&conn)?;
    store::mark_interrupted(&conn)?;
    let c = compute(&conn, root, cfg)?;
    if c.fingerprint != fingerprint {
        return Err(fail("plan-changed"));
    }
    if c.counts.planned == 0 {
        return Err(fail("nothing-to-do"));
    }
    if !c.enough_space() {
        return Err(fail("insufficient-space"));
    }
    let mut created = BTreeSet::new();
    note_missing_dirs(&c.cfg.destination, None, &mut created);
    fs::create_dir_all(&c.cfg.destination).map_err(|_| fail("destination-unwritable"))?;

    let id = store::create(&conn, &c.cfg, &c.fingerprint, c.counts)?;
    let mut guard = Interrupted { conn: &conn, id, armed: true, created };
    let kind = match c.cfg.operation {
        Operation::Copy => Kind::Copy,
        Operation::Move => Kind::Move,
    };
    let total = c.counts.planned;
    let (mut done, mut failed, mut stopped) = (0, 0, false);
    on_progress(Progress { done, total, failed });
    for p in c.items.iter().filter(|p| p.action == Action::Place) {
        if cancelled(token) {
            stopped = true;
            break;
        }
        let Some(dest) = p.target.as_deref().and_then(|t| plan::join_target(&c.cfg.destination, t))
        else {
            failed += 1;
            continue;
        };
        note_missing_dirs(&dest, Some(&c.cfg.destination), &mut guard.created);
        let step = Step {
            image_id: p.image_id,
            kind,
            source: &p.source,
            dest: &dest,
            import_hash: p.hash.clone(),
            job: Some(id),
        };
        if !files::run(&conn, &step)?.ok() {
            failed += 1;
        }
        done += 1;
        on_progress(Progress { done, total, failed });
    }
    let status = if stopped {
        "cancelled"
    } else if failed > 0 {
        "partial"
    } else {
        "completed"
    };
    store::set_created_dirs(&conn, id, &guard.created)?;
    store::finish(&conn, id, status)?;
    guard.armed = false;
    store::get(&conn, id)?.ok_or(Error::StoreTransaction)
}

/// Undo run `run`: remove its copies (only while unmodified) or move its originals back
/// (only while still where it put them). Steps already undone are skipped, so an undo
/// that stopped part-way can be repeated.
pub fn undo(
    root: &Path,
    run: i64,
    token: u64,
    mut on_progress: impl FnMut(Progress),
) -> Result<RunRecord> {
    let _run = try_run_lock().ok_or_else(|| fail("busy"))?;
    let conn = open_root(root)?;
    let _ops = files::lock();
    files::settle_locked(&conn)?;
    store::mark_interrupted(&conn)?;
    let rec = store::get(&conn, run)?.ok_or_else(|| fail("unknown-consolidation"))?;
    if rec.undo_status.as_deref() == Some("undone") {
        return Err(fail("already-undone"));
    }
    let done_before = store::undone_images(&conn, run)?;
    let todo: Vec<_> = files::completed_placements(&conn, run)?
        .into_iter()
        .rev()
        .filter(|(op, _)| op.image_id.is_some_and(|i| !done_before.contains(&i)))
        .collect();
    let total = todo.len();
    let (mut done, mut failed, mut stopped) = (0, 0, false);
    on_progress(Progress { done, total, failed });
    for (op, checksum) in &todo {
        if cancelled(token) {
            stopped = true;
            break;
        }
        let image = op.image_id.unwrap_or_default();
        let placed = PathBuf::from(&op.destination_path);
        let outcome = if op.operation_type == "copy" {
            files::remove_copy(&conn, run, image, &placed, checksum)?
        } else {
            let back = PathBuf::from(&op.source_path);
            let step = Step {
                image_id: image,
                kind: Kind::MoveBack,
                source: &placed,
                dest: &back,
                import_hash: store::file_hash(&conn, image)?,
                job: Some(run),
            };
            if store::current_path(&conn, image)?.as_deref() != Some(op.destination_path.as_str()) {
                files::refuse(&conn, &step, "moved-since")?
            } else {
                files::run(&conn, &step)?
            }
        };
        if !outcome.ok() {
            failed += 1;
        }
        done += 1;
        on_progress(Progress { done, total, failed });
    }
    remove_empty_dirs(store::created_dirs(&conn, run)?);
    let complete = !stopped && failed == 0;
    store::set_undo(&conn, run, if complete { "undone" } else { "partially-undone" })?;
    store::get(&conn, run)?.ok_or(Error::StoreTransaction)
}

/// Remove the folders a run created, deepest first, each only if it is empty now
/// (`remove_dir` refuses a folder that is not), so a folder the user made or filled stays.
fn remove_empty_dirs(dirs: Vec<PathBuf>) {
    let mut dirs = dirs;
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for d in dirs {
        let _ = fs::remove_dir(d);
    }
}

/// Every consolidation of `root`, newest first.
pub fn runs_for_root(root: &Path) -> Result<Vec<RunRecord>> {
    let conn = open_root(root)?;
    if let Some(_idle) = try_run_lock() {
        store::mark_interrupted(&conn)?;
    }
    store::list(&conn)
}

/// One written file that is no longer what the run wrote.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Problem {
    pub image_id: i64,
    pub path: String,
    /// `modified` · `missing` · `moved-since` (verify_run); `changed` · `missing`
    /// (verify_originals).
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RunCheck {
    pub run_id: i64,
    pub checked: usize,
    pub ok: usize,
    pub modified: usize,
    pub missing: usize,
    pub moved_since: usize,
    pub problems: Vec<Problem>,
}

/// Re-hash every file run `run` placed (and has not undone) against what it wrote.
pub fn verify_run(root: &Path, run: i64) -> Result<RunCheck> {
    let conn = open_root(root)?;
    store::get(&conn, run)?.ok_or_else(|| fail("unknown-consolidation"))?;
    let undone = store::undone_images(&conn, run)?;
    let mut items = Vec::new();
    for (op, checksum) in files::completed_placements(&conn, run)? {
        let Some(image) = op.image_id.filter(|i| !undone.contains(i)) else {
            continue;
        };
        let here = op.operation_type == "copy"
            || store::current_path(&conn, image)?.as_deref() == Some(op.destination_path.as_str());
        items.push((image, op.destination_path, checksum, here));
    }
    let states: Vec<(i64, String, &'static str)> = items
        .into_par_iter()
        .map(|(image, path, checksum, here)| {
            let state = if !here {
                "moved-since"
            } else if fs::symlink_metadata(&path).is_err() {
                "missing"
            } else if dedup_hash(Path::new(&path)).ok().as_deref() == Some(checksum.as_str()) {
                "ok"
            } else {
                "modified"
            };
            (image, path, state)
        })
        .collect();
    let count = |s: &str| states.iter().filter(|x| x.2 == s).count();
    Ok(RunCheck {
        run_id: run,
        checked: states.len(),
        ok: count("ok"),
        modified: count("modified"),
        missing: count("missing"),
        moved_since: count("moved-since"),
        problems: problems(&states),
    })
}

fn problems(states: &[(i64, String, &'static str)]) -> Vec<Problem> {
    states
        .iter()
        .filter(|s| s.2 != "ok" && s.2 != "unhashed")
        .take(REPORT_ITEMS)
        .map(|(image_id, path, state)| Problem { image_id: *image_id, path: path.clone(), state })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OriginalsCheck {
    pub total: usize,
    pub ok: usize,
    pub changed: usize,
    pub missing: usize,
    /// No import hash to compare with (the file was unreadable at import).
    pub unhashed: usize,
    pub problems: Vec<Problem>,
}

/// "Originals immutable", end to end: re-hash every photograph at its location of record
/// (`COALESCE(current_path, original_path)`) against its import hash.
pub fn verify_originals(
    root: &Path,
    mut on_progress: impl FnMut(Progress),
) -> Result<OriginalsCheck> {
    let conn = open_root(root)?;
    let rows: Vec<(i64, String, Option<String>)> = {
        let mut stmt = conn
            .prepare("SELECT id, COALESCE(current_path, original_path), file_hash FROM images ORDER BY id")
            .map_err(|_| Error::StoreTransaction)?;
        let it = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|_| Error::StoreTransaction)?;
        it.collect::<rusqlite::Result<_>>().map_err(|_| Error::StoreTransaction)?
    };
    let total = rows.len();
    let mut states = Vec::with_capacity(total);
    let mut failed = 0;
    on_progress(Progress { done: 0, total, failed });
    for chunk in rows.chunks(64) {
        let part: Vec<(i64, String, &'static str)> = chunk
            .par_iter()
            .map(|(id, path, hash)| {
                let state = match hash {
                    _ if fs::symlink_metadata(path).is_err() => "missing",
                    None => "unhashed",
                    Some(h) if dedup_hash(Path::new(path)).ok().as_deref() == Some(h.as_str()) => {
                        "ok"
                    }
                    Some(_) => "changed",
                };
                (*id, path.clone(), state)
            })
            .collect();
        failed += part.iter().filter(|s| s.2 == "changed" || s.2 == "missing").count();
        states.extend(part);
        on_progress(Progress { done: states.len(), total, failed });
    }
    let count = |s: &str| states.iter().filter(|x| x.2 == s).count();
    Ok(OriginalsCheck {
        total,
        ok: count("ok"),
        changed: count("changed"),
        missing: count("missing"),
        unhashed: count("unhashed"),
        problems: problems(&states),
    })
}

#[cfg(test)]
mod exit_tests;
