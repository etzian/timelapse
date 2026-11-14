//! Recoverable file operations: the minimal Phase 6 primitive that quarantine needs
//! (ADR-011, ADR-020). Consolidation, copy and link policy remain Phase 6.
//!
//! A move never overwrites anything and never loses the source before the destination is
//! verified:
//!
//! 1. verify the source has the import hash (when the import could hash it);
//! 2. journal a `pending` row (source, destination, the SHA-256 being moved);
//! 3. create the destination *atomically without replacing*: a hard link, or, where the
//!    filesystem has none (FAT/exFAT), a `create_new` copy with the source's mtime. Both
//!    fail if anything already exists at the destination, so there is no check-then-act
//!    window;
//! 4. verify the destination hash; on a mismatch, remove the destination we just created;
//! 5. remove the source name; if that fails (file in use), remove the destination instead;
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

fn lock() -> MutexGuard<'static, ()> {
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Quarantine,
    Restore,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Quarantine => "quarantine",
            Self::Restore => "restore",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "quarantine" => Some(Self::Quarantine),
            "restore" => Some(Self::Restore),
            _ => None,
        }
    }
}

struct Located {
    original: PathBuf,
    current: Option<PathBuf>,
    relative: String,
    hash: Option<String>,
    status: Option<String>,
}

fn locate(conn: &Connection, id: i64) -> Result<Option<Located>> {
    conn.query_row(
        r#"
            SELECT i.original_path, i.current_path, i.relative_path, i.file_hash, s.status
            FROM images i LEFT JOIN image_status s ON s.image_id = i.id
            WHERE i.id = ?1
        "#,
        [id],
        |r| {
            Ok(Located {
                original: PathBuf::from(r.get::<_, String>(0)?),
                current: r.get::<_, Option<String>>(1)?.map(PathBuf::from),
                relative: r.get(2)?,
                hash: r.get(3)?,
                status: r.get(4)?,
            })
        },
    )
    .optional()
    .map_err(|_| Error::StoreTransaction)
}

/// Move each *rejected* image's original into `<root>/.timelapse/quarantine/<relative path>`.
/// Only images the user explicitly rejected qualify (ADR-011).
pub fn quarantine(conn: &Connection, root: &Path, ids: &[i64]) -> Result<Vec<OpOutcome>> {
    let _guard = lock();
    ids.iter()
        .map(|&id| {
            let Some(img) = locate(conn, id)? else {
                return Ok(fail(id, None, "unknown-image"));
            };
            if img.status.as_deref() != Some("rejected") || img.current.is_some() {
                return Ok(fail(id, None, "not-rejected"));
            }
            let Some(dest) = safe_join(&quarantine_dir(root), &img.relative) else {
                return Ok(fail(id, None, "unsafe-path"));
            };
            run(conn, id, Kind::Quarantine, &img.original, &dest, img.hash)
        })
        .collect()
}

/// Move each quarantined image back to its original path; never overwrites.
pub fn restore(conn: &Connection, ids: &[i64]) -> Result<Vec<OpOutcome>> {
    let _guard = lock();
    ids.iter()
        .map(|&id| {
            let Some(img) = locate(conn, id)? else {
                return Ok(fail(id, None, "unknown-image"));
            };
            let (Some("quarantined"), Some(current)) = (img.status.as_deref(), img.current) else {
                return Ok(fail(id, None, "not-quarantined"));
            };
            run(conn, id, Kind::Restore, &current, &img.original, img.hash)
        })
        .collect()
}

/// Join a stored relative path under `base`, refusing anything that could escape it.
fn safe_join(base: &Path, relative: &str) -> Option<PathBuf> {
    let rel = Path::new(relative);
    let normal = rel.components().all(|c| matches!(c, std::path::Component::Normal(_)));
    (normal && !relative.is_empty()).then(|| base.join(rel))
}

fn fail(image_id: i64, operation_id: Option<i64>, failure: &'static str) -> OpOutcome {
    OpOutcome { image_id, operation_id, failure: Some(failure) }
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Create `dst` with the content of `src`, failing if `dst` exists. A hard link when the
/// filesystem supports it (instant, shares the bytes and the mtime), otherwise a
/// `create_new` copy that keeps the source's modification time.
fn place_no_clobber(src: &Path, dst: &Path) -> std::result::Result<(), &'static str> {
    match fs::hard_link(src, dst) {
        Ok(()) => return Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => return Err("destination-exists"),
        Err(_) => {}
    }
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

fn run(
    conn: &Connection,
    id: i64,
    kind: Kind,
    source: &Path,
    dest: &Path,
    import_hash: Option<String>,
) -> Result<OpOutcome> {
    // (1) the bytes we are about to move must be the bytes we imported. With no import
    // hash (unreadable at import) there is no reference; the move is still verified below.
    let actual = match dedup_hash(source) {
        Ok(h) => h,
        Err(_) => return journal_failed(conn, id, kind, source, dest, None, "source-unreadable"),
    };
    if let Some(expected) = import_hash.as_deref()
        && expected != actual
    {
        return journal_failed(conn, id, kind, source, dest, Some(&actual), "changed-since-import");
    }
    // (2) journal before touching anything.
    conn.execute(
        r#"
            INSERT INTO file_operations (image_id, operation_type, source_path, destination_path,
                                         status, checksum, created_at)
            VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6)
        "#,
        params![id, kind.as_str(), source.to_string_lossy(), dest.to_string_lossy(), actual, now()],
    )
    .map_err(|_| Error::StoreTransaction)?;
    let op = conn.last_insert_rowid();

    if let Some(parent) = dest.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return mark_failed(conn, id, op, "move-failed");
    }
    // (3) place without ever replacing.
    if let Err(kind) = place_no_clobber(source, dest) {
        return mark_failed(conn, id, op, kind);
    }
    // (4) verify; the source is untouched so far.
    if dedup_hash(dest).ok().as_deref() != Some(actual.as_str()) {
        let _ = fs::remove_file(dest); // created by us in (3)
        return mark_failed(conn, id, op, "verify-failed");
    }
    // (5) drop the source name; if it cannot go (in use), undo (3) instead.
    // A failed removal can still have removed the file (network shares, Windows), so the
    // destination is only discarded when the source provably still holds the same bytes.
    if fs::remove_file(source).is_err() && source.exists() {
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

fn journal_failed(
    conn: &Connection,
    id: i64,
    kind: Kind,
    source: &Path,
    dest: &Path,
    checksum: Option<&str>,
    failure: &'static str,
) -> Result<OpOutcome> {
    conn.execute(
        r#"
            INSERT INTO file_operations (image_id, operation_type, source_path, destination_path,
                                         status, checksum, failure, created_at, completed_at)
            VALUES (?1, ?2, ?3, ?4, 'failed', ?5, ?6, ?7, ?7)
        "#,
        params![
            id,
            kind.as_str(),
            source.to_string_lossy(),
            dest.to_string_lossy(),
            checksum,
            failure,
            now()
        ],
    )
    .map_err(|_| Error::StoreTransaction)?;
    Ok(fail(id, Some(conn.last_insert_rowid()), failure))
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

/// Record a finished move: journal row, the image's current location and its status, in
/// one transaction, and only if the row is still `pending`.
fn complete(conn: &Connection, op: i64, id: i64, kind: Kind, dest: &Path) -> Result<()> {
    let tx = conn.unchecked_transaction().map_err(|_| Error::StoreTransaction)?;
    let (current, status) = match kind {
        Kind::Quarantine => (Some(dest.to_string_lossy().into_owned()), "quarantined"),
        // Restoring undoes the quarantine: the image is back where it was, still rejected.
        Kind::Restore => (None, "rejected"),
    };
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
    tx.execute("UPDATE images SET current_path = ?2 WHERE id = ?1", params![id, current])
        .and_then(|_| {
            tx.execute(
                "UPDATE image_status SET status = ?2, user_override = 1, updated_at = ?3 WHERE image_id = ?1",
                params![id, status, now()],
            )
        })
        .map_err(|_| Error::StoreTransaction)?;
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
/// - destination has the journaled hash, source gone → the move finished: complete it;
/// - source and destination both have the journaled hash → interrupted between placing and
///   removing the source: remove the destination copy (an identical duplicate we created)
///   and mark it `interrupted`, leaving the original where it was;
/// - source present, destination absent → it never happened: `interrupted`;
/// - anything else → `inconsistent`; no file is touched.
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
        let (src_ok, dst_ok) = (matches(src), matches(dst));
        match Kind::parse(kind) {
            Some(k) if dst_ok && !src.exists() => complete(conn, *op, *image, k, dst)?,
            _ if src_ok && dst_ok && !same_entry(src, dst) => {
                let _ = fs::remove_file(dst);
                mark_failed(conn, *image, *op, "interrupted")?;
            }
            // A copy cut short by the crash: smaller than the verified source, and created by
            // this operation (a pre-existing file would have failed it as `destination-exists`).
            _ if src_ok && is_partial_copy(src, dst) => {
                let _ = fs::remove_file(dst);
                mark_failed(conn, *image, *op, "interrupted")?;
            }
            _ if src_ok && !dst.exists() => {
                mark_failed(conn, *image, *op, "interrupted")?;
            }
            _ => {
                mark_failed(conn, *image, *op, "inconsistent")?;
            }
        }
    }
    Ok(pending.len())
}

/// One journal entry, for the UI's operation history.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FileOperation {
    pub id: i64,
    pub image_id: Option<i64>,
    pub operation_type: String,
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    pub failure: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

/// The journal, newest first, at most `limit` entries.
pub fn list_operations(conn: &Connection, limit: usize) -> Result<Vec<FileOperation>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, image_id, operation_type, source_path, destination_path, status, failure,
                    created_at, completed_at
             FROM file_operations ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map([limit as i64], |r| {
            Ok(FileOperation {
                id: r.get(0)?,
                image_id: r.get(1)?,
                operation_type: r.get(2)?,
                source_path: r.get(3)?,
                destination_path: r.get(4)?,
                status: r.get(5)?,
                failure: r.get(6)?,
                created_at: r.get(7)?,
                completed_at: r.get(8)?,
            })
        })
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
}
