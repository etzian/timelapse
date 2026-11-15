//! Recoverable file operations (ADR-011, ADR-020, ADR-023): quarantine and restore, and
//! the copy / move / undo steps of a consolidation.
//!
//! Every step never overwrites anything and never loses the source before the target is
//! verified:
//!
//! 1. verify the source has the import hash (when the import could hash it);
//! 2. check the target is free, then journal a `pending` row (source, target, the SHA-256);
//! 3. create the target *atomically without replacing*. Both placements fail if anything
//!    already exists at the target, so there is no check-then-act window:
//!    - quarantine / restore (inside the project, ADR-020): a hard link, or a `create_new`
//!      copy where the filesystem has none;
//!    - the working collection (ADR-023): an APFS clone or a `create_new` copy, **never** a
//!      link, so an edit of the target can never reach the original;
//! 4. verify the target hash; on a mismatch, remove the target we just created;
//! 5. for a relocation, remove the source name; if that fails (file in use), remove the
//!    target instead;
//! 6. in one transaction, complete the row and update `images.current_path` and the status.
//!
//! All operations of this process run under one lock, and [`reconcile_if_idle`] (run when
//! a project is opened) only settles `pending` rows while no operation is running, so it
//! only ever sees rows a crashed process left behind. Journal updates only touch rows
//! that are still `pending`. No byte of an original is ever written.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension, params};

use crate::import::{dedup_hash, project_dir};
use crate::{Error, Result};

/// Serialises every file operation and reconciliation in this process.
static FILE_OPS: Mutex<()> = Mutex::new(());

pub(crate) fn lock() -> MutexGuard<'static, ()> {
    // A panic mid-operation leaves a `pending` row, which reconcile handles; the lock
    // itself carries no data, so a poisoned lock is safe to reuse.
    FILE_OPS.lock().unwrap_or_else(|e| e.into_inner())
}

/// `<root>/.timelapse/quarantine`.
pub fn quarantine_dir(root: &Path) -> PathBuf {
    project_dir(root).join("quarantine")
}

/// What happened to one requested image. `failure` is a stable kind, mapped to the Fluent
/// key `files-failure-<kind>` in the UI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OpOutcome {
    pub image_id: i64,
    pub operation_id: Option<i64>,
    pub failure: Option<&'static str>,
}

impl OpOutcome {
    pub fn ok(&self) -> bool {
        self.failure.is_none()
    }
}

/// The journaled operation types (`file_operations.operation_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Quarantine,
    Restore,
    /// Consolidation: a clone or copy into the working collection.
    Copy,
    /// Consolidation: the original itself goes to the working collection.
    Move,
    /// Undo of a `Move`.
    MoveBack,
    /// Undo of a `Copy`: the copy we made is removed.
    RemoveCopy,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Quarantine => "quarantine",
            Self::Restore => "restore",
            Self::Copy => "copy",
            Self::Move => "move",
            Self::MoveBack => "move-back",
            Self::RemoveCopy => "remove-copy",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "quarantine" => Self::Quarantine,
            "restore" => Self::Restore,
            "copy" => Self::Copy,
            "move" => Self::Move,
            "move-back" => Self::MoveBack,
            "remove-copy" => Self::RemoveCopy,
            _ => return None,
        })
    }
    /// The source name goes once the target is verified.
    fn relocates(self) -> bool {
        matches!(self, Self::Quarantine | Self::Restore | Self::Move | Self::MoveBack)
    }
    /// Placed by hard link where possible (only inside the project, for an instant).
    fn may_link(self) -> bool {
        matches!(self, Self::Quarantine | Self::Restore)
    }
}

struct Located {
    /// `COALESCE(current_path, original_path)`: where the file is now.
    location: PathBuf,
    relative: String,
    hash: Option<String>,
    status: Option<String>,
}

fn locate(conn: &Connection, id: i64) -> Result<Option<Located>> {
    conn.query_row(
        r#"
            SELECT COALESCE(i.current_path, i.original_path), i.relative_path, i.file_hash,
                   s.status
            FROM images i LEFT JOIN image_status s ON s.image_id = i.id
            WHERE i.id = ?1
        "#,
        [id],
        |r| {
            Ok(Located {
                location: PathBuf::from(r.get::<_, String>(0)?),
                relative: r.get(1)?,
                hash: r.get(2)?,
                status: r.get(3)?,
            })
        },
    )
    .optional()
    .map_err(|_| Error::StoreTransaction)
}

/// Move each *rejected* image's original, from wherever it is now, into
/// `<root>/.timelapse/quarantine/<relative path>`. Only images the user explicitly
/// rejected qualify (ADR-011).
pub fn quarantine(conn: &Connection, root: &Path, ids: &[i64]) -> Result<Vec<OpOutcome>> {
    let _guard = lock();
    ids.iter()
        .map(|&id| {
            let Some(img) = locate(conn, id)? else {
                return Ok(fail(id, None, "unknown-image"));
            };
            if img.status.as_deref() != Some("rejected") {
                return Ok(fail(id, None, "not-rejected"));
            }
            let Some(dest) = safe_join(&quarantine_dir(root), &img.relative) else {
                return Ok(fail(id, None, "unsafe-path"));
            };
            run(conn, &Step::new(id, Kind::Quarantine, &img.location, &dest, img.hash))
        })
        .collect()
}

/// Move each quarantined image back to where its quarantine started (its original path,
/// or the working folder a consolidation moved it to); never overwrites.
pub fn restore(conn: &Connection, ids: &[i64]) -> Result<Vec<OpOutcome>> {
    let _guard = lock();
    ids.iter()
        .map(|&id| {
            let Some(img) = locate(conn, id)? else {
                return Ok(fail(id, None, "unknown-image"));
            };
            if img.status.as_deref() != Some("quarantined") {
                return Ok(fail(id, None, "not-quarantined"));
            }
            let Some(back) = quarantined_from(conn, id)? else {
                return Ok(fail(id, None, "not-quarantined"));
            };
            run(conn, &Step::new(id, Kind::Restore, &img.location, &back, img.hash))
        })
        .collect()
}

/// Where the image's last completed quarantine took it from.
fn quarantined_from(conn: &Connection, id: i64) -> Result<Option<PathBuf>> {
    conn.query_row(
        "SELECT source_path FROM file_operations
         WHERE image_id = ?1 AND operation_type = 'quarantine' AND status = 'completed'
         ORDER BY id DESC LIMIT 1",
        [id],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .map(|p| p.map(PathBuf::from))
    .map_err(|_| Error::StoreTransaction)
}

/// Join a stored relative path under `base`, refusing anything that could escape it.
fn safe_join(base: &Path, relative: &str) -> Option<PathBuf> {
    let rel = Path::new(relative);
    let normal = rel.components().all(|c| matches!(c, std::path::Component::Normal(_)));
    (normal && !relative.is_empty()).then(|| base.join(rel))
}

pub(crate) fn fail(image_id: i64, operation_id: Option<i64>, failure: &'static str) -> OpOutcome {
    OpOutcome { image_id, operation_id, failure: Some(failure) }
}

pub(crate) fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Create `dst` with the content of `src`, failing if `dst` exists. A hard link when the
/// filesystem supports it (instant, shares the bytes and the mtime), otherwise a
/// `create_new` copy that keeps the source's modification time. Only for a relocation
/// inside the project, where the source name goes right after.
fn place_no_clobber(src: &Path, dst: &Path) -> std::result::Result<(), &'static str> {
    match fs::hard_link(src, dst) {
        Ok(()) => return Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => return Err("destination-exists"),
        Err(_) => {}
    }
    copy_new(src, dst)
}

/// Create `dst` as an independent copy of `src`, failing if `dst` exists: an APFS clone
/// (copy-on-write, instant, no space until edited) where the volume supports it, otherwise
/// a `create_new` byte copy keeping the mtime. Never a link (ADR-023).
pub(crate) fn clone_or_copy_no_clobber(
    src: &Path,
    dst: &Path,
) -> std::result::Result<(), &'static str> {
    #[cfg(target_os = "macos")]
    {
        if let Some(done) = clone(src, dst) {
            return done;
        }
    }
    copy_new(src, dst)
}

/// `clonefile(2)`: `Some(result)` when it settled the matter, `None` to fall back to a copy
/// (another filesystem, another volume, or a path it cannot take).
#[cfg(target_os = "macos")]
fn clone(src: &Path, dst: &Path) -> Option<std::result::Result<(), &'static str>> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let s = CString::new(src.as_os_str().as_bytes()).ok()?;
    let d = CString::new(dst.as_os_str().as_bytes()).ok()?;
    // SAFETY: both pointers are valid NUL-terminated strings for the duration of the call;
    // `clonefile` reads them and writes nothing through them. Flags 0: follow a symlinked
    // source to its file (the clone is always a regular file); it never replaces `dst`.
    let rc = unsafe { libc::clonefile(s.as_ptr(), d.as_ptr(), 0) };
    if rc == 0 {
        return Some(Ok(()));
    }
    match io::Error::last_os_error().raw_os_error() {
        Some(libc::EEXIST) => Some(Err("destination-exists")),
        _ => None,
    }
}

fn copy_new(src: &Path, dst: &Path) -> std::result::Result<(), &'static str> {
    let mut out = match OpenOptions::new().write(true).create_new(true).open(dst) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::AlreadyExists => return Err("destination-exists"),
        Err(_) => return Err("move-failed"),
    };
    let copied = (|| -> io::Result<()> {
        let mut input = File::open(src)?;
        io::copy(&mut input, &mut out)?;
        if let Ok(modified) = input.metadata().and_then(|m| m.modified()) {
            out.set_modified(modified)?;
        }
        out.sync_all()
    })();
    if copied.is_err() {
        drop(out);
        let _ = fs::remove_file(dst); // the partial copy we created; nothing else is there
        return Err("move-failed");
    }
    Ok(())
}

/// Bytes free for an unprivileged user on the volume holding `path` (or its nearest
/// existing ancestor). `None` where it cannot be told (non-Unix, or an error).
pub fn free_space(path: &Path) -> Option<u64> {
    let mut p = path;
    while !p.exists() {
        p = p.parent()?;
    }
    statvfs_free(p)
}

#[cfg(unix)]
#[allow(clippy::unnecessary_cast)] // the statvfs field widths differ between platforms
fn statvfs_free(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: `c` is a valid NUL-terminated path and `st` a properly sized, writable
    // `statvfs` that the call fills in; it is only read after a successful return.
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c.as_ptr(), &mut st) };
    (rc == 0).then(|| st.f_bavail as u64 * st.f_frsize as u64)
}

#[cfg(not(unix))]
fn statvfs_free(_path: &Path) -> Option<u64> {
    None
}

/// One journaled relocation or copy of one image.
pub(crate) struct Step<'a> {
    pub image_id: i64,
    pub kind: Kind,
    pub source: &'a Path,
    pub dest: &'a Path,
    pub import_hash: Option<String>,
    pub job: Option<i64>,
}

impl<'a> Step<'a> {
    pub(crate) fn new(
        image_id: i64,
        kind: Kind,
        source: &'a Path,
        dest: &'a Path,
        import_hash: Option<String>,
    ) -> Self {
        Self { image_id, kind, source, dest, import_hash, job: None }
    }
}

/// Run one step (the caller holds [`lock`]).
pub(crate) fn run(conn: &Connection, step: &Step) -> Result<OpOutcome> {
    let (id, kind, source, dest) = (step.image_id, step.kind, step.source, step.dest);
    // (1) the bytes we are about to place must be the bytes we imported. With no import
    // hash (unreadable at import) there is no reference; the result is still verified.
    let actual = match dedup_hash(source) {
        Ok(h) => h,
        Err(_) => return journal_failed(conn, step, None, "source-unreadable"),
    };
    if let Some(expected) = step.import_hash.as_deref()
        && expected != actual
    {
        return journal_failed(conn, step, Some(&actual), "changed-since-import");
    }
    // (2) the target must be free *before* the row is journaled, so a `pending` row whose
    // target holds the journaled bytes was written by us (reconcile relies on it).
    if fs::symlink_metadata(dest).is_ok() {
        return journal_failed(conn, step, Some(&actual), "destination-exists");
    }
    conn.execute(
        r#"
            INSERT INTO file_operations (job_id, image_id, operation_type, source_path,
                                         destination_path, status, checksum, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7)
        "#,
        params![
            step.job,
            id,
            kind.as_str(),
            source.to_string_lossy(),
            dest.to_string_lossy(),
            actual,
            now()
        ],
    )
    .map_err(|_| Error::StoreTransaction)?;
    let op = conn.last_insert_rowid();

    if let Some(parent) = dest.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return mark_failed(conn, id, op, "move-failed");
    }
    // (3) place without ever replacing.
    let placed = if kind.may_link() {
        place_no_clobber(source, dest)
    } else {
        clone_or_copy_no_clobber(source, dest)
    };
    if let Err(kind) = placed {
        return mark_failed(conn, id, op, kind);
    }
    // (4) verify; the source is untouched so far.
    if dedup_hash(dest).ok().as_deref() != Some(actual.as_str()) {
        let _ = fs::remove_file(dest); // created by us in (3)
        return mark_failed(conn, id, op, "verify-failed");
    }
    // (5) drop the source name; if it cannot go (in use), undo (3) instead.
    // A failed removal can still have removed the file (network shares, Windows), so the
    // target is only discarded when the source provably still holds the same bytes.
    if kind.relocates() && fs::remove_file(source).is_err() && source.exists() {
        if dedup_hash(source).ok().as_deref() == Some(actual.as_str()) && !same_entry(source, dest)
        {
            let _ = fs::remove_file(dest);
            return mark_failed(conn, id, op, "source-in-use");
        }
        // Something else is at the source now: keep both files and let a person look.
        return mark_failed(conn, id, op, "inconsistent");
    }
    // (6) record. If this fails the row stays `pending` and the next open settles it.
    if complete(conn, op, id, kind, dest).is_err() {
        return Ok(fail(id, Some(op), "record-failed"));
    }
    Ok(OpOutcome { image_id: id, operation_id: Some(op), failure: None })
}

/// Remove a copy that a consolidation made, only while it still holds `checksum` (the
/// bytes we wrote). The caller holds [`lock`].
pub(crate) fn remove_copy(
    conn: &Connection,
    job: i64,
    image_id: i64,
    copy: &Path,
    checksum: &str,
) -> Result<OpOutcome> {
    let step = Step {
        image_id,
        kind: Kind::RemoveCopy,
        source: copy,
        dest: Path::new(""),
        import_hash: None,
        job: Some(job),
    };
    if fs::symlink_metadata(copy).is_err() {
        return journal_failed(conn, &step, None, "missing");
    }
    match dedup_hash(copy) {
        Ok(h) if h == checksum => {}
        Ok(h) => return journal_failed(conn, &step, Some(&h), "modified"),
        Err(_) => return journal_failed(conn, &step, None, "source-unreadable"),
    }
    conn.execute(
        r#"
            INSERT INTO file_operations (job_id, image_id, operation_type, source_path,
                                         destination_path, status, checksum, created_at)
            VALUES (?1, ?2, 'remove-copy', ?3, '', 'pending', ?4, ?5)
        "#,
        params![job, image_id, copy.to_string_lossy(), checksum, now()],
    )
    .map_err(|_| Error::StoreTransaction)?;
    let op = conn.last_insert_rowid();
    if fs::remove_file(copy).is_err() && copy.exists() {
        return mark_failed(conn, image_id, op, "remove-failed");
    }
    if complete(conn, op, image_id, Kind::RemoveCopy, Path::new("")).is_err() {
        return Ok(fail(image_id, Some(op), "record-failed"));
    }
    Ok(OpOutcome { image_id, operation_id: Some(op), failure: None })
}

/// Record a step that was refused before anything was touched (the caller holds [`lock`]).
pub(crate) fn refuse(conn: &Connection, step: &Step, failure: &'static str) -> Result<OpOutcome> {
    journal_failed(conn, step, None, failure)
}

fn journal_failed(
    conn: &Connection,
    step: &Step,
    checksum: Option<&str>,
    failure: &'static str,
) -> Result<OpOutcome> {
    conn.execute(
        r#"
            INSERT INTO file_operations (job_id, image_id, operation_type, source_path,
                                         destination_path, status, checksum, failure,
                                         created_at, completed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 'failed', ?6, ?7, ?8, ?8)
        "#,
        params![
            step.job,
            step.image_id,
            step.kind.as_str(),
            step.source.to_string_lossy(),
            step.dest.to_string_lossy(),
            checksum,
            failure,
            now()
        ],
    )
    .map_err(|_| Error::StoreTransaction)?;
    Ok(fail(step.image_id, Some(conn.last_insert_rowid()), failure))
}

fn mark_failed(conn: &Connection, id: i64, op: i64, failure: &'static str) -> Result<OpOutcome> {
    conn.execute(
        "UPDATE file_operations SET status = 'failed', failure = ?2, completed_at = ?3
         WHERE id = ?1 AND status = 'pending'",
        params![op, failure, now()],
    )
    .map_err(|_| Error::StoreTransaction)?;
    Ok(fail(id, Some(op), failure))
}

/// Record a finished step: the journal row, the image's current location and its status,
/// in one transaction, and only if the row is still `pending`.
fn complete(conn: &Connection, op: i64, id: i64, kind: Kind, dest: &Path) -> Result<()> {
    let tx = conn.unchecked_transaction().map_err(|_| Error::StoreTransaction)?;
    let claimed = tx
        .execute(
            "UPDATE file_operations SET status = 'completed', completed_at = ?2
             WHERE id = ?1 AND status = 'pending'",
            params![op, now()],
        )
        .map_err(|_| Error::StoreTransaction)?;
    if claimed == 0 {
        return Err(Error::StoreTransaction);
    }
    let dest = dest.to_string_lossy();
    // `current_path` is NULL exactly when the file is back at its path of record. The
    // comparison is exact (BINARY): the path we return to is the string we came from.
    let set_location = || {
        tx.execute(
            "UPDATE images SET current_path =
                 CASE WHEN ?2 = original_path COLLATE BINARY THEN NULL ELSE ?2 END
             WHERE id = ?1",
            params![id, dest],
        )
    };
    let status = |s: &str| {
        tx.execute(
            "UPDATE image_status SET status = ?2, user_override = 1, updated_at = ?3 WHERE image_id = ?1",
            params![id, s, now()],
        )
    };
    let result = match kind {
        Kind::Quarantine => set_location().and_then(|_| status("quarantined")),
        // Restoring undoes the quarantine: the image is back where it was, still rejected.
        Kind::Restore => set_location().and_then(|_| status("rejected")),
        Kind::Move | Kind::MoveBack => set_location(),
        Kind::Copy | Kind::RemoveCopy => Ok(0),
    };
    result.map_err(|_| Error::StoreTransaction)?;
    tx.commit().map_err(|_| Error::StoreTransaction)
}

/// Whether two paths name the same directory entry (aliasing through junctions, links or
/// case). When in doubt, they are treated as the same so that nothing is removed.
fn same_entry(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => true,
    }
}

fn is_partial_copy(src: &Path, dst: &Path) -> bool {
    match (fs::metadata(src), fs::metadata(dst)) {
        (Ok(s), Ok(d)) => d.is_file() && d.len() < s.len() && !same_entry(src, dst),
        _ => false,
    }
}

/// [`reconcile`], unless a file operation of this process is running: then its `pending`
/// rows are live, not abandoned, and are left alone (the next open settles any leftover).
pub fn reconcile_if_idle(conn: &Connection) -> Result<usize> {
    match FILE_OPS.try_lock() {
        Ok(_guard) => settle(conn),
        Err(std::sync::TryLockError::Poisoned(p)) => {
            let _guard = p.into_inner();
            settle(conn)
        }
        Err(std::sync::TryLockError::WouldBlock) => Ok(0),
    }
}

/// Settle every `pending` row left by an interrupted process, from the state on disk.
/// Waits for any running operation of this process first. Returns how many were settled.
///
/// Relocations (quarantine, restore, move, move-back):
/// - target has the journaled hash, source gone → the move finished: complete it;
/// - source and target both have the journaled hash → interrupted between placing and
///   removing the source: remove the target (an identical duplicate we created) and mark
///   it `interrupted`, leaving the original where it was;
/// - source present, target absent → it never happened: `interrupted`.
///
/// Copy: target has the journaled hash → completed; a shorter target → removed,
/// `interrupted`; no target → `interrupted`.
///
/// Remove-copy: the copy is gone → completed; still there with its hash → `interrupted`.
///
/// Anything else → `inconsistent`; no file is touched.
pub fn reconcile(conn: &Connection) -> Result<usize> {
    let _guard = lock();
    settle(conn)
}

fn settle(conn: &Connection) -> Result<usize> {
    let pending: Vec<(i64, i64, String, String, String, Option<String>)> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, image_id, operation_type, source_path, destination_path, checksum
                 FROM file_operations WHERE status = 'pending' ORDER BY id",
            )
            .map_err(|_| Error::StoreTransaction)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
            })
            .map_err(|_| Error::StoreTransaction)?;
        rows.collect::<std::result::Result<_, _>>().map_err(|_| Error::StoreTransaction)?
    };
    for (op, image, kind, source, dest, checksum) in &pending {
        let (src, dst) = (Path::new(source), Path::new(dest));
        let matches = |p: &Path| {
            p.exists()
                && checksum.as_deref().is_some_and(|c| dedup_hash(p).ok().as_deref() == Some(c))
        };
        let interrupted = |remove: Option<&Path>| -> Result<()> {
            if let Some(p) = remove {
                let _ = fs::remove_file(p);
            }
            mark_failed(conn, *image, *op, "interrupted").map(|_| ())
        };
        let inconsistent = || mark_failed(conn, *image, *op, "inconsistent").map(|_| ());
        match Kind::parse(kind) {
            Some(Kind::Copy) => {
                let src_ok = matches(src);
                if matches(dst) {
                    complete(conn, *op, *image, Kind::Copy, dst)?;
                } else if src_ok && is_partial_copy(src, dst) {
                    interrupted(Some(dst))?;
                } else if src_ok && !dst.exists() {
                    interrupted(None)?;
                } else {
                    inconsistent()?;
                }
            }
            Some(Kind::RemoveCopy) => {
                if fs::symlink_metadata(src).is_err() {
                    complete(conn, *op, *image, Kind::RemoveCopy, dst)?;
                } else if matches(src) {
                    interrupted(None)?;
                } else {
                    inconsistent()?;
                }
            }
            Some(k) => {
                let (src_ok, dst_ok) = (matches(src), matches(dst));
                if dst_ok && !src.exists() {
                    complete(conn, *op, *image, k, dst)?;
                } else if src_ok && dst_ok && !same_entry(src, dst) {
                    interrupted(Some(dst))?;
                } else if src_ok && is_partial_copy(src, dst) {
                    // A copy cut short by the crash: smaller than the verified source, and
                    // created by this operation (the target was free when journaled).
                    interrupted(Some(dst))?;
                } else if src_ok && !dst.exists() {
                    interrupted(None)?;
                } else {
                    inconsistent()?;
                }
            }
            None => inconsistent()?,
        }
    }
    Ok(pending.len())
}

/// One journal entry, for the UI's operation history.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FileOperation {
    pub id: i64,
    pub job_id: Option<i64>,
    pub image_id: Option<i64>,
    pub operation_type: String,
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    pub failure: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

const OPERATION_COLUMNS: &str = "id, job_id, image_id, operation_type, source_path,
    destination_path, status, failure, created_at, completed_at";

fn operation_row(r: &rusqlite::Row) -> rusqlite::Result<FileOperation> {
    Ok(FileOperation {
        id: r.get(0)?,
        job_id: r.get(1)?,
        image_id: r.get(2)?,
        operation_type: r.get(3)?,
        source_path: r.get(4)?,
        destination_path: r.get(5)?,
        status: r.get(6)?,
        failure: r.get(7)?,
        created_at: r.get(8)?,
        completed_at: r.get(9)?,
    })
}

/// The journal, newest first, at most `limit` entries.
pub fn list_operations(conn: &Connection, limit: usize) -> Result<Vec<FileOperation>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {OPERATION_COLUMNS} FROM file_operations ORDER BY id DESC LIMIT ?1"
        ))
        .map_err(|_| Error::StoreTransaction)?;
    let rows =
        stmt.query_map([limit as i64], operation_row).map_err(|_| Error::StoreTransaction)?;
    rows.collect::<std::result::Result<_, _>>().map_err(|_| Error::StoreTransaction)
}

/// The completed placements (`copy` / `move`) of a consolidation run, in order, with the
/// checksum each one wrote.
pub(crate) fn completed_placements(
    conn: &Connection,
    job: i64,
) -> Result<Vec<(FileOperation, String)>> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {OPERATION_COLUMNS}, checksum FROM file_operations
             WHERE job_id = ?1 AND status = 'completed' AND operation_type IN ('copy', 'move')
             ORDER BY id"
        ))
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map([job], |r| Ok((operation_row(r)?, r.get::<_, String>(10)?)))
        .map_err(|_| Error::StoreTransaction)?;
    rows.collect::<std::result::Result<_, _>>().map_err(|_| Error::StoreTransaction)
}

/// Run `f` while holding the file-operation lock, as a live operation would (tests only).
#[cfg(test)]
pub(crate) fn with_operation_lock_for_tests(f: impl FnOnce()) {
    let _guard = lock();
    f();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_refuses_escapes() {
        let base = Path::new("/q");
        assert!(safe_join(base, "a/b.jpg").is_some());
        assert!(safe_join(base, "../b.jpg").is_none());
        assert!(safe_join(base, "").is_none());
        #[cfg(windows)]
        assert!(safe_join(base, "C:\\x.jpg").is_none());
        #[cfg(not(windows))]
        assert!(safe_join(base, "/x.jpg").is_none());
    }

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tl-files-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn a_clone_or_copy_is_independent_and_never_replaces() {
        let d = temp("clone");
        let (src, dst) = (d.join("a.jpg"), d.join("b.jpg"));
        fs::write(&src, b"original bytes").unwrap();
        clone_or_copy_no_clobber(&src, &dst).unwrap();
        assert_eq!(fs::read(&dst).unwrap(), b"original bytes");
        // Editing the copy never reaches the original (no shared bytes, ADR-006).
        fs::write(&dst, b"edited").unwrap();
        assert_eq!(fs::read(&src).unwrap(), b"original bytes");
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let (a, b) = (fs::metadata(&src).unwrap(), fs::metadata(&dst).unwrap());
            assert_ne!((a.dev(), a.ino()), (b.dev(), b.ino()));
        }
        assert_eq!(clone_or_copy_no_clobber(&src, &dst), Err("destination-exists"));
        assert_eq!(fs::read(&dst).unwrap(), b"edited");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn free_space_walks_up_to_an_existing_folder() {
        let d = temp("space");
        let deep = d.join("not/yet/there");
        #[cfg(unix)]
        assert!(free_space(&deep).is_some_and(|b| b > 0));
        #[cfg(not(unix))]
        assert_eq!(free_space(&deep), None);
        let _ = fs::remove_dir_all(&d);
    }
}
