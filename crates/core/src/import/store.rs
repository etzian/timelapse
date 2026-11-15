//! SQLite adapter: one database per project directory (ADR-007). Keeps the
//! `projects` and `images` tables current, applies migrations once, upserts an
//! import batch in a single transaction (idempotent by original path), and
//! surfaces integrity warnings for UI.
//!
//! Rust-side errors are mapped to `crate::Error::DbMessage { .. }`. The UI
//! renders them through Fluent so no prose is hardcoded here.

use rusqlite::{Connection, params};
use std::fs;
use std::path::Path;

use super::metadata::{Candidates, Confidence, TimestampSource, resolver};
use super::scan::Candidate;
use crate::Error;

/// Current schema version, tracked in `PRAGMA user_version` (docs/DATABASE_SCHEMA.md §6).
pub const SCHEMA_VERSION: i64 = 4;

/// v1: the Phase 1 tables. Phase 1 databases were created with `user_version = 0` and
/// these exact statements, so `IF NOT EXISTS` makes the step a no-op for them.
const MIGRATION_V1: &str = r#"
    CREATE TABLE IF NOT EXISTS projects (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        project_path TEXT NOT NULL UNIQUE,
        source_root TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        schema_version INTEGER NOT NULL,
        settings_json TEXT
    );

    CREATE TABLE IF NOT EXISTS images (
        id INTEGER PRIMARY KEY,
        original_path TEXT NOT NULL UNIQUE COLLATE NOCASE,
        relative_path TEXT NOT NULL,
        filename TEXT NOT NULL,
        extension TEXT NOT NULL,
        file_size INTEGER NOT NULL,
        modified_at INTEGER,
        file_hash TEXT,
        width INTEGER,
        height INTEGER,
        orientation INTEGER,
        camera_make TEXT,
        camera_model TEXT,
        exif_datetime_original TEXT,
        exif_create_date TEXT,
        filename_datetime TEXT,
        filesystem_datetime TEXT,
        normalized_datetime TEXT,
        timestamp_source TEXT,
        timestamp_confidence TEXT,
        gps_latitude REAL,
        gps_longitude REAL,
        import_status TEXT NOT NULL,
        created_at INTEGER NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_images_hash ON images (file_hash);
    CREATE INDEX IF NOT EXISTS idx_images_relative ON images (relative_path);
"#;

/// v2 (Phase 2 timeline): the chronology index, and the UTC offset that was applied when
/// `normalized_datetime` was resolved (`NULL` = the source had no zone, UTC was assumed).
const MIGRATION_V2: &str = r#"
    CREATE INDEX IF NOT EXISTS idx_images_normalized_datetime ON images (normalized_datetime);
    ALTER TABLE images ADD COLUMN timestamp_offset_minutes INTEGER;
"#;

/// v3 (Phase 3 cleaning, ADR-020): luma measurements, one status row per image, the
/// file-operation journal, and where an original currently is (`NULL` = at `original_path`).
const MIGRATION_V3: &str = r#"
    CREATE TABLE IF NOT EXISTS image_analysis (
        image_id INTEGER PRIMARY KEY REFERENCES images (id) ON DELETE CASCADE,
        algorithm_version INTEGER NOT NULL,
        source_hash TEXT,
        width INTEGER,
        height INTEGER,
        mean_luma REAL,
        median_luma INTEGER,
        histogram_json TEXT,
        failure TEXT,
        xmp_rating INTEGER,
        analyzed_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS image_status (
        id INTEGER PRIMARY KEY,
        image_id INTEGER NOT NULL UNIQUE REFERENCES images (id) ON DELETE CASCADE,
        status TEXT NOT NULL CHECK (status IN ('eligible', 'flagged', 'rejected', 'quarantined')),
        reason TEXT,
        user_override INTEGER NOT NULL DEFAULT 0,
        updated_at INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_image_status_status ON image_status (status);

    CREATE TABLE IF NOT EXISTS file_operations (
        id INTEGER PRIMARY KEY,
        job_id INTEGER,
        image_id INTEGER REFERENCES images (id),
        operation_type TEXT NOT NULL CHECK (operation_type IN ('quarantine', 'restore')),
        source_path TEXT NOT NULL,
        destination_path TEXT NOT NULL,
        status TEXT NOT NULL CHECK (status IN ('pending', 'completed', 'failed')),
        checksum TEXT,
        failure TEXT,
        created_at INTEGER NOT NULL,
        completed_at INTEGER
    );
    CREATE INDEX IF NOT EXISTS idx_file_operations_job ON file_operations (job_id);
    CREATE INDEX IF NOT EXISTS idx_file_operations_status ON file_operations (status);

    ALTER TABLE images ADD COLUMN current_path TEXT;
"#;

/// v4 (Phase 4 sampling): each run's configuration and inputs, and its ordered selection.
const MIGRATION_V4: &str = r#"
    CREATE TABLE IF NOT EXISTS sampling_runs (
        id INTEGER PRIMARY KEY,
        method TEXT NOT NULL,
        configuration_json TEXT NOT NULL,
        algorithm_version INTEGER NOT NULL,
        input_count INTEGER NOT NULL,
        input_fingerprint TEXT NOT NULL,
        excluded_json TEXT NOT NULL,
        target_count INTEGER NOT NULL,
        selected_count INTEGER NOT NULL,
        unmatched_count INTEGER NOT NULL,
        unmatched_json TEXT NOT NULL,
        status TEXT NOT NULL CHECK (status IN ('completed')),
        created_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS sampling_items (
        id INTEGER PRIMARY KEY,
        sampling_run_id INTEGER NOT NULL REFERENCES sampling_runs (id) ON DELETE CASCADE,
        image_id INTEGER NOT NULL REFERENCES images (id),
        target_datetime TEXT,
        actual_datetime TEXT NOT NULL,
        sequence_index INTEGER NOT NULL,
        UNIQUE (sampling_run_id, sequence_index),
        UNIQUE (sampling_run_id, image_id)
    );
    CREATE INDEX IF NOT EXISTS idx_sampling_items_run ON sampling_items (sampling_run_id, sequence_index);
"#;

/// Apply every outstanding step in order, each in its own transaction together with its
/// `user_version` bump, so a failed step leaves the database at the previous version.
/// A database from a newer build (`user_version > SCHEMA_VERSION`) is refused, not touched.
///
/// Two commands may open a fresh database at once: each step takes the write lock up front
/// (`BEGIN IMMEDIATE`, waiting up to the busy timeout) and re-reads the version inside the
/// transaction, so a step another connection already applied is skipped, not re-run.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let read = |c: &Connection| c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0));
    let version = read(conn)?;
    if version > SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidQuery);
    }
    for (target, sql) in
        [(1, MIGRATION_V1), (2, MIGRATION_V2), (3, MIGRATION_V3), (4, MIGRATION_V4)]
    {
        if version < target {
            let tx = rusqlite::Transaction::new_unchecked(
                conn,
                rusqlite::TransactionBehavior::Immediate,
            )?;
            if read(&tx)? < target {
                tx.execute_batch(sql)?;
                tx.pragma_update(None, "user_version", target)?;
            }
            tx.commit()?;
        }
    }
    Ok(())
}

fn db_path(db_dir: &Path) -> std::path::PathBuf {
    db_dir.join("timelapse.db")
}

/// Open (creating if needed) the project database at `db_dir/timelapse.db` and run
/// any outstanding migrations. `name`, `source_root` and `project_path` describe
/// the project row — they are upserted on every open so a rename of the source
/// tree is reflected without a migration.
pub fn open(
    db_dir: &Path,
    name: &str,
    project_path: &Path,
    source_root: Option<&Path>,
) -> Result<Connection, Error> {
    fs::create_dir_all(db_dir).map_err(|_| db_err(db_dir))?;
    let conn = Connection::open(db_path(db_dir)).map_err(|_| db_err(db_dir))?;
    // Before anything that takes a lock (switching to WAL does): concurrent commands wait
    // for each other instead of failing with SQLITE_BUSY.
    conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(|_| db_err(db_dir))?;
    // Switching a brand-new database to WAL can report SQLITE_BUSY without consulting the
    // busy handler when another connection is opening it at the same moment: retry briefly.
    let mut attempt = 0;
    loop {
        match conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;") {
            Ok(()) => break,
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::DatabaseBusy && attempt < 100 =>
            {
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(_) => return Err(db_err(db_dir)),
        }
    }

    migrate(&conn).map_err(|_| db_err(db_dir))?;
    // Settle any file operation a crash left half-done before anything reads locations.
    crate::files::reconcile_if_idle(&conn).map_err(|_| db_err(db_dir))?;

    let now = now_unix();
    let _ = conn
        .execute(
            r#"
                INSERT INTO projects (id, name, project_path, source_root, created_at, updated_at, schema_version, settings_json)
                VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, NULL)
                ON CONFLICT(project_path) DO UPDATE SET name = excluded.name,
                    source_root = excluded.source_root,
                    updated_at = excluded.updated_at,
                    schema_version = max(projects.schema_version, excluded.schema_version)
            "#,
            params![name, project_path.to_string_lossy(), source_root.map(|p| p.to_string_lossy().into_owned()), now, now, SCHEMA_VERSION],
        )
        .map_err(|_| db_err(db_dir))?;
    Ok(conn)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpsertCounts {
    pub total: usize,
    pub new_rows: usize,
    /// Rows already present that were left untouched (same path).
    pub unchanged: usize,
}

/// Upsert a batch of candidates into the current project's `images` table.
/// Idempotent: re-running on the same root produces zero new rows — `ON CONFLICT
/// DO NOTHING` leaves existing rows untouched, and re-import is therefore a no-op
/// for known paths (ADR-006 originals never change).
///
/// `hashes[i]` is the SHA-256 of `candidates[i]`; `None` means "unknown / hashing failed".
pub fn upsert_images(
    conn: &Connection,
    candidates: &[Candidate],
    hashes: &[Option<String>],
    dup_flags: &[bool],
) -> Result<UpsertCounts, Error> {
    debug_assert_eq!(candidates.len(), hashes.len());
    debug_assert_eq!(candidates.len(), dup_flags.len());
    let now = now_unix();
    let tx = conn.unchecked_transaction().map_err(|_| Error::StoreTransaction)?;

    let mut new_rows = 0usize;
    let mut unchanged = 0usize;

    for i in 0..candidates.len() {
        let c = &candidates[i];
        let status = if dup_flags[i] { "duplicate-suspect" } else { "imported" };
        let n = tx
            .execute(
                r#"
                    INSERT INTO images (original_path, relative_path, filename, extension, file_size,
                                        modified_at, file_hash, import_status, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    ON CONFLICT(original_path) DO NOTHING
                "#,
                params![
                    c.path.to_string_lossy(),
                    c.relative_path.to_string_lossy(),
                    c.filename,
                    c.extension,
                    c.file_size,
                    c.modified_at_unix,
                    hashes[i].as_deref(),
                    status,
                    now
                ],
            )
            .map_err(|_| Error::StoreTransaction)?;
        if n > 0 {
            new_rows += 1;
        } else {
            unchanged += 1;
        }
    }

    tx.commit().map_err(|_| Error::StoreTransaction)?;
    Ok(UpsertCounts { total: candidates.len(), new_rows, unchanged })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageRow {
    pub id: i64,
    pub original_path: String,
    pub relative_path: String,
    pub filename: String,
    pub extension: String,
    pub file_size: i64,
    pub modified_at: Option<i64>,
    pub import_status: String,
}

pub fn list_images(conn: &Connection) -> Result<Vec<ImageRow>, Error> {
    let mut stmt = conn
        .prepare("SELECT id, original_path, relative_path, filename, extension, file_size, modified_at, import_status FROM images ORDER BY relative_path")
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ImageRow {
                id: r.get(0)?,
                original_path: r.get(1)?,
                relative_path: r.get(2)?,
                filename: r.get(3)?,
                extension: r.get(4)?,
                file_size: r.get(5)?,
                modified_at: r.get(6)?,
                import_status: r.get::<_, String>(7)?,
            })
        })
        .map_err(|_| Error::StoreTransaction)?;
    rows.collect::<Result<_, _>>().map_err(|_| Error::StoreTransaction)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrityResult {
    pub missing_files: Vec<String>,
    /// Hash groups with two or more images, each item is the list of paths that share it.
    pub duplicates: Vec<Vec<String>>,
    pub total_images: usize,
}

/// Cheaply check the inventory against the filesystem and surface warnings to the
/// UI without touching the database (ADR-011 + ADR-006).
pub fn integrity_check(conn: &Connection) -> Result<IntegrityResult, Error> {
    let total = conn
        .query_row("SELECT COUNT(*) FROM images", params![], |r| r.get::<_, i64>(0))
        .map_err(|_| Error::StoreTransaction)? as usize;

    // Missing files: rows whose current location (the original path, or the quarantine
    // path while quarantined) no longer exists on disk.
    let mut stmt = conn
        .prepare("SELECT COALESCE(current_path, original_path) FROM images")
        .map_err(|_| Error::StoreTransaction)?;
    let paths: Vec<String> = {
        let it =
            stmt.query_map([], |r| r.get::<_, String>(0)).map_err(|_| Error::StoreTransaction)?;
        it.flatten().collect()
    };
    let missing_files = paths.iter().filter(|p| !Path::new(p).exists()).cloned().collect();

    // Duplicate content groups: rows sharing a non-NULL file_hash, 2+.
    let mut stmt2 = conn
        .prepare("SELECT original_path, file_hash FROM images WHERE file_hash IS NOT NULL")
        .map_err(|_| Error::StoreTransaction)?;
    let mut dup_map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    {
        let it = stmt2
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|_| Error::StoreTransaction)?;
        for row in it.flatten() {
            dup_map.entry(row.1).or_default().push(row.0);
        }
    }
    let duplicates = dup_map.into_values().filter(|v| v.len() >= 2).collect();

    Ok(IntegrityResult { missing_files, duplicates, total_images: total })
}

// ---------------------------------------------------------------------------------------
// Phase 2 — timestamps & timeline (PROCESSING_PIPELINE.md §3).
// ---------------------------------------------------------------------------------------

/// An image whose capture time has not been resolved yet.
#[derive(Debug, Clone)]
pub struct PendingImage {
    pub id: i64,
    pub original_path: String,
    pub modified_at: Option<i64>,
}

/// Rows with no `timestamp_source` yet: never backfilled, or nothing resolved last time.
pub fn pending_timestamps(conn: &Connection) -> Result<Vec<PendingImage>, Error> {
    let mut stmt = conn
        .prepare(
            "SELECT id, original_path, modified_at FROM images WHERE timestamp_source IS NULL ORDER BY id",
        )
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(PendingImage { id: r.get(0)?, original_path: r.get(1)?, modified_at: r.get(2)? })
        })
        .map_err(|_| Error::StoreTransaction)?;
    rows.collect::<Result<_, _>>().map_err(|_| Error::StoreTransaction)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct BackfillCounts {
    /// Rows that received a `normalized_datetime` in this run.
    pub resolved: usize,
    /// Rows examined in this run for which no source held a usable value.
    pub unresolved: usize,
}

/// Write the candidates and the resolved timestamp of each `(image id, candidates)` pair,
/// in one transaction. The original values (EXIF text, filename match, filesystem time)
/// are preserved next to the normalised one. Idempotent: a row that already has a
/// `timestamp_source` is left untouched; an unresolved row keeps a `NULL` source and is
/// examined again next time.
pub fn backfill_timestamps(
    conn: &Connection,
    items: &[(i64, Candidates)],
) -> Result<BackfillCounts, Error> {
    let tx = conn.unchecked_transaction().map_err(|_| Error::StoreTransaction)?;
    let mut counts = BackfillCounts::default();
    {
        let mut stmt = tx
            .prepare(
                r#"
                    UPDATE images SET
                        exif_datetime_original = ?2,
                        exif_create_date = ?3,
                        filename_datetime = ?4,
                        filesystem_datetime = ?5,
                        normalized_datetime = ?6,
                        timestamp_source = ?7,
                        timestamp_confidence = ?8,
                        timestamp_offset_minutes = ?9
                    WHERE id = ?1 AND timestamp_source IS NULL
                "#,
            )
            .map_err(|_| Error::StoreTransaction)?;
        for (id, c) in items {
            let resolved = resolver::resolve(c);
            let changed = stmt
                .execute(params![
                    id,
                    c.exif_original.as_ref().map(|s| s.raw.as_str()),
                    c.exif_digitized.as_ref().map(|s| s.raw.as_str()),
                    c.filename.as_ref().map(|s| s.raw.as_str()),
                    c.filesystem_unix.and_then(resolver::normalize_unix),
                    resolved.as_ref().map(|r| r.normalized.as_str()),
                    resolved.as_ref().map(|r| r.source.as_str()),
                    resolved.as_ref().map(|r| r.confidence.as_str()),
                    resolved.as_ref().and_then(|r| r.offset_minutes),
                ])
                .map_err(|_| Error::StoreTransaction)?;
            if changed > 0 {
                if resolved.is_some() {
                    counts.resolved += 1;
                } else {
                    counts.unresolved += 1;
                }
            }
        }
    }
    tx.commit().map_err(|_| Error::StoreTransaction)?;
    Ok(counts)
}

/// How many images resolved to each source.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct SourceCounts {
    pub exif_datetime_original: usize,
    pub exif_datetime_digitized: usize,
    pub filename: usize,
    pub filesystem: usize,
}

/// The state of the whole inventory's timeline.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct TimelineSummary {
    pub total: usize,
    pub resolved: usize,
    pub unresolved: usize,
    pub by_source: SourceCounts,
    pub low_confidence: usize,
    /// Resolved from a wall clock with no stated zone, so UTC was assumed (ADR-019).
    pub tz_assumed: usize,
}

pub fn timeline_summary(conn: &Connection) -> Result<TimelineSummary, Error> {
    let mut stmt = conn
        .prepare(
            r#"
                SELECT timestamp_source, timestamp_confidence,
                       normalized_datetime IS NOT NULL AND timestamp_offset_minutes IS NULL,
                       COUNT(*)
                FROM images GROUP BY 1, 2, 3
            "#,
        )
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, bool>(2)?,
                r.get::<_, i64>(3)? as usize,
            ))
        })
        .map_err(|_| Error::StoreTransaction)?;
    let mut s = TimelineSummary::default();
    for row in rows {
        let (source, confidence, tz_assumed, n) = row.map_err(|_| Error::StoreTransaction)?;
        s.total += n;
        match source.as_deref().and_then(TimestampSource::parse) {
            None => s.unresolved += n,
            Some(src) => {
                s.resolved += n;
                let slot = match src {
                    TimestampSource::ExifDatetimeOriginal => {
                        &mut s.by_source.exif_datetime_original
                    }
                    TimestampSource::ExifDatetimeDigitized => {
                        &mut s.by_source.exif_datetime_digitized
                    }
                    TimestampSource::Filename => &mut s.by_source.filename,
                    TimestampSource::Filesystem => &mut s.by_source.filesystem,
                };
                *slot += n;
            }
        }
        if confidence.as_deref() == Some(Confidence::Low.as_str()) {
            s.low_confidence += n;
        }
        if tz_assumed {
            s.tz_assumed += n;
        }
    }
    Ok(s)
}

/// One image on the timeline.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimelineEntry {
    pub id: i64,
    pub relative_path: String,
    /// `YYYY-MM-DDTHH:MM:SS.mmmZ`, or `None` when no source resolved.
    pub normalized_datetime: Option<String>,
    /// The same instant as milliseconds since the UNIX epoch (for `Intl` formatting).
    pub epoch_ms: Option<i64>,
    pub source: Option<String>,
    pub confidence: Option<String>,
    /// UTC offset (minutes) applied when resolving; `None` together with `tz_assumed`.
    pub offset_minutes: Option<i64>,
    pub tz_assumed: bool,
}

/// Every image, in chronological order of `normalized_datetime` (ties broken by path, so
/// the order is total and stable). Unresolved images are returned last with
/// `normalized_datetime == None` — visible, never dropped.
pub fn timeline(conn: &Connection) -> Result<Vec<TimelineEntry>, Error> {
    let mut stmt = conn
        .prepare(
            r#"
                SELECT id, relative_path, normalized_datetime,
                       CAST(round(unixepoch(normalized_datetime, 'subsec') * 1000) AS INTEGER),
                       timestamp_source, timestamp_confidence, timestamp_offset_minutes
                FROM images
                ORDER BY normalized_datetime IS NULL, normalized_datetime, original_path
            "#,
        )
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map([], |r| {
            let normalized: Option<String> = r.get(2)?;
            let offset: Option<i64> = r.get(6)?;
            Ok(TimelineEntry {
                id: r.get(0)?,
                relative_path: r.get(1)?,
                tz_assumed: normalized.is_some() && offset.is_none(),
                normalized_datetime: normalized,
                epoch_ms: r.get(3)?,
                source: r.get(4)?,
                confidence: r.get(5)?,
                offset_minutes: offset,
            })
        })
        .map_err(|_| Error::StoreTransaction)?;
    rows.collect::<Result<_, _>>().map_err(|_| Error::StoreTransaction)
}

/// Bucket size for [`distribution`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Granularity {
    Hour,
    Day,
    Month,
}

impl Granularity {
    /// `strftime` format of the bucket key. Every key is a prefix of the normalised values
    /// it counts, so a key can be passed back as a `from` bound (drill-down).
    fn strftime(self) -> &'static str {
        match self {
            Self::Hour => "%Y-%m-%dT%H",
            Self::Day => "%Y-%m-%d",
            Self::Month => "%Y-%m",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DistributionBucket {
    /// UTC bucket key: `YYYY-MM-DDTHH`, `YYYY-MM-DD` or `YYYY-MM`.
    pub bucket: String,
    pub count: usize,
}

/// The "distribution over a period": image counts per hour / day / month (UTC), in
/// chronological order, optionally limited to `from <= normalized_datetime < to` (both
/// in the normalised format or a prefix of it, e.g. `2024-01`). Empty buckets are omitted.
pub fn distribution(
    conn: &Connection,
    granularity: Granularity,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<DistributionBucket>, Error> {
    let mut stmt = conn
        .prepare(
            r#"
                SELECT strftime(?1, normalized_datetime) AS bucket, COUNT(*)
                FROM images
                WHERE normalized_datetime IS NOT NULL
                  AND (?2 IS NULL OR normalized_datetime >= ?2)
                  AND (?3 IS NULL OR normalized_datetime < ?3)
                GROUP BY bucket ORDER BY bucket
            "#,
        )
        .map_err(|_| Error::StoreTransaction)?;
    let rows = stmt
        .query_map(params![granularity.strftime(), from, to], |r| {
            Ok(DistributionBucket { bucket: r.get(0)?, count: r.get::<_, i64>(1)? as usize })
        })
        .map_err(|_| Error::StoreTransaction)?;
    rows.collect::<Result<_, _>>().map_err(|_| Error::StoreTransaction)
}

/// When is an interval between consecutive images "unusually long"?
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GapOptions {
    /// A gap must exceed `factor ×` the median interval…
    pub factor: f64,
    /// …and never be shorter than this many seconds.
    pub min_seconds: i64,
}

impl Default for GapOptions {
    fn default() -> Self {
        Self { factor: 10.0, min_seconds: 3600 }
    }
}

impl GapOptions {
    /// Upper bounds that keep the millisecond arithmetic far from overflow.
    pub const MAX_FACTOR: f64 = 1_000_000.0;
    pub const MAX_MIN_SECONDS: i64 = 100 * 366 * 86_400;

    /// Options from optional caller values (e.g. over IPC): missing ones take the defaults,
    /// out-of-range ones are clamped to `[0, MAX]` (a NaN factor counts as missing).
    pub fn from_parts(factor: Option<f64>, min_seconds: Option<i64>) -> Self {
        let d = Self::default();
        Self {
            factor: factor.filter(|f| !f.is_nan()).unwrap_or(d.factor).clamp(0.0, Self::MAX_FACTOR),
            min_seconds: min_seconds.unwrap_or(d.min_seconds).clamp(0, Self::MAX_MIN_SECONDS),
        }
    }
}

/// An unusually long interval between two consecutive resolved images.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimelineGap {
    pub before: TimelineEntry,
    pub after: TimelineEntry,
    pub seconds: i64,
    /// The two sides were resolved from different sources, or one side's zone was assumed
    /// and the other's was not: the gap may be a metadata inconsistency rather than a real
    /// pause in shooting. Reported, never corrected (ADR-010 / ADR-011).
    pub source_changed: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct GapReport {
    pub median_interval_seconds: Option<f64>,
    pub threshold_seconds: i64,
    pub gaps: Vec<TimelineGap>,
}

/// Interval boundaries on the resolved timeline longer than
/// `max(min_seconds, factor × median interval)`. Data only: adjacent timestamps are
/// never adjusted to close a gap.
pub fn gaps(conn: &Connection, opts: GapOptions) -> Result<GapReport, Error> {
    let opts = GapOptions::from_parts(Some(opts.factor), Some(opts.min_seconds));
    let resolved: Vec<TimelineEntry> =
        timeline(conn)?.into_iter().filter(|e| e.epoch_ms.is_some()).collect();
    let intervals: Vec<i64> = resolved
        .windows(2)
        .map(|w| w[1].epoch_ms.unwrap_or(0) - w[0].epoch_ms.unwrap_or(0))
        .collect();
    let median_ms = median(&intervals);
    let by_factor = median_ms.map_or(0.0, |m| m * opts.factor / 1000.0);
    let threshold_seconds = opts.min_seconds.max(by_factor.ceil() as i64);
    let gaps = resolved
        .windows(2)
        .zip(&intervals)
        .filter(|(_, ms)| **ms > threshold_seconds.saturating_mul(1000))
        .map(|(w, ms)| TimelineGap {
            source_changed: w[0].source != w[1].source || w[0].tz_assumed != w[1].tz_assumed,
            before: w[0].clone(),
            after: w[1].clone(),
            seconds: ms / 1000,
        })
        .collect();
    Ok(GapReport {
        median_interval_seconds: median_ms.map(|m| m / 1000.0),
        threshold_seconds,
        gaps,
    })
}

fn median(values: &[i64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    let mid = v.len() / 2;
    Some(if v.len().is_multiple_of(2) { (v[mid - 1] + v[mid]) as f64 / 2.0 } else { v[mid] as f64 })
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn db_err(dir: &Path) -> Error {
    Error::DbMessage { path: dir.to_string_lossy().into_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ts-store-{tag}-{}", std::process::id()));
        let _ = fs::create_dir_all(&d);
        d
    }

    fn cand(path: &std::path::Path, relative: &str) -> Candidate {
        Candidate {
            path: path.to_path_buf(),
            relative_path: relative.into(),
            filename: "x.jpg".into(),
            extension: "jpg".into(),
            file_size: 10,
            modified_at_unix: 0,
        }
    }

    #[test]
    fn upsert_is_idempotent_per_original_path() {
        let root = tmp_dir("idem");
        let root_img = root.join("source.jpg");
        fs::write(&root_img, b"jpg").unwrap();
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();

        let conn = open(&root, "p1", &src, Some(&root)).unwrap();
        let cands = vec![cand(&root_img, "source.jpg")];
        let flags = vec![false];
        let hashes = vec![Some("deadbeef".into())];

        let first = super::upsert_images(&conn, &cands, &hashes, &flags).unwrap();
        assert_eq!(first.new_rows, 1);

        let second = super::upsert_images(&conn, &cands, &hashes, &flags).unwrap();
        assert_eq!(second.unchanged, 1, "re-importing the same path must not add rows");
    }

    #[test]
    fn integrity_marks_missing_files_and_grouping_duplicates() {
        let root = tmp_dir("integ");
        fs::create_dir_all(&root).unwrap();

        // Two rows sharing a hash (a.jpg + b.jpg, present on disk) and one
        // row ('gone.jpg') whose file is absent on purpose.
        let a_path = root.join("a.jpg");
        let b_path = root.join("b.jpg");
        fs::write(&a_path, b"dup").unwrap();
        fs::write(&b_path, b"dup").unwrap();
        let gone_path = root.join("gone.jpg"); // intentionally not created

        let conn = open(&root, "p2", &root, Some(&root)).unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO images (original_path, relative_path, filename, extension, file_size, import_status, created_at)
             VALUES (?1, 'gone.jpg', 'gone.jpg', 'jpg', 4, 'imported', 0),
                    (?2, 'a.jpg', 'a.jpg', 'jpg', 8, 'imported', 0),
                    (?3, 'b.jpg', 'b.jpg', 'jpg', 8, 'duplicate-suspect', 0)",
            params![gone_path.to_string_lossy().into_owned(), a_path.to_string_lossy().into_owned(), b_path.to_string_lossy().into_owned()],
        )
        .unwrap();
        let _ = conn
            .execute(
                "UPDATE images SET file_hash = 'abc' WHERE original_path IN (?1, ?2)",
                params![
                    a_path.to_string_lossy().into_owned(),
                    b_path.to_string_lossy().into_owned()
                ],
            )
            .unwrap();

        let res = super::integrity_check(&conn).unwrap();
        assert_eq!(res.missing_files.len(), 1);
        assert_eq!(res.duplicates.len(), 1);
        assert_eq!(res.duplicates[0].len(), 2);
    }

    #[test]
    fn a_phase_1_database_is_migrated_to_v2_in_place() {
        let root = tmp_dir("migrate-v1");
        let _ = fs::remove_file(db_path(&root));
        {
            // Exactly what Phase 1 left behind: v1 tables, user_version 0, one row.
            let conn = Connection::open(db_path(&root)).unwrap();
            conn.execute_batch(MIGRATION_V1).unwrap();
            conn.execute(
                "INSERT INTO images (original_path, relative_path, filename, extension, file_size, import_status, created_at)
                 VALUES ('x', 'x.jpg', 'x.jpg', 'jpg', 1, 'imported', 0)",
                [],
            )
            .unwrap();
        }
        let conn = open(&root, "p", &root, Some(&root)).unwrap();
        let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(v, SCHEMA_VERSION);
        let idx: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_images_normalized_datetime'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(idx, 1);
        let kept: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM images WHERE timestamp_offset_minutes IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kept, 1, "existing rows survive the migration");
        drop(conn);
        // Re-opening is a no-op (the ALTER TABLE must not run twice).
        open(&root, "p", &root, Some(&root)).unwrap();
    }

    #[test]
    fn a_database_from_a_newer_build_is_refused_untouched() {
        let root = tmp_dir("migrate-future");
        let _ = fs::remove_file(db_path(&root));
        Connection::open(db_path(&root)).unwrap().pragma_update(None, "user_version", 99).unwrap();
        assert!(matches!(open(&root, "p", &root, Some(&root)), Err(Error::DbMessage { .. })));
        let v: i64 = Connection::open(db_path(&root))
            .unwrap()
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 99);
    }

    #[test]
    fn gap_options_are_defaulted_and_clamped() {
        assert_eq!(GapOptions::from_parts(None, None), GapOptions::default());
        let o = GapOptions::from_parts(Some(f64::MAX), Some(i64::MAX));
        assert_eq!(o.factor, GapOptions::MAX_FACTOR);
        assert_eq!(o.min_seconds, GapOptions::MAX_MIN_SECONDS);
        let o = GapOptions::from_parts(Some(f64::NAN), Some(-5));
        assert_eq!((o.factor, o.min_seconds), (10.0, 0));
    }

    #[test]
    fn extreme_gap_options_do_not_overflow() {
        let root = tmp_dir("gaps-extreme");
        let _ = fs::remove_file(db_path(&root));
        let conn = open(&root, "p", &root, Some(&root)).unwrap();
        for (p, t) in [("a", "2024-01-01T00:00:00.000Z"), ("b", "2024-06-01T00:00:00.000Z")] {
            conn.execute(
                "INSERT INTO images (original_path, relative_path, filename, extension, file_size, import_status, created_at,
                                     normalized_datetime, timestamp_source, timestamp_confidence, timestamp_offset_minutes)
                 VALUES (?1, ?1, ?1, 'jpg', 1, 'imported', 0, ?2, 'filename', 'medium', NULL)",
                params![p, t],
            )
            .unwrap();
        }
        let r = gaps(&conn, GapOptions { factor: f64::MAX, min_seconds: i64::MAX }).unwrap();
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn concurrent_first_opens_migrate_once() {
        let root = tmp_dir("migrate-race");
        let _ = fs::remove_file(db_path(&root));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let r = root.clone();
                std::thread::spawn(move || open(&r, "p", &r, Some(&r)).map(|_| ()))
            })
            .collect();
        for h in handles {
            if let Err(e) = h.join().unwrap() {
                panic!("a concurrent open failed: {e:?}");
            }
        }
    }

    #[test]
    fn median_handles_odd_even_and_empty() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[3, 1, 2]), Some(2.0));
        assert_eq!(median(&[4, 1, 2, 3]), Some(2.5));
    }

    #[test]
    fn gaps_threshold_respects_the_floor_and_flags_source_changes() {
        let root = tmp_dir("gaps-unit");
        let _ = fs::remove_file(db_path(&root));
        let conn = open(&root, "p", &root, Some(&root)).unwrap();
        let rows = [
            ("a", "2024-01-01T00:00:00.000Z", "exif_datetime_original"),
            ("b", "2024-01-01T00:00:01.000Z", "exif_datetime_original"),
            ("c", "2024-01-01T00:00:02.000Z", "exif_datetime_original"),
            ("d", "2024-01-01T00:00:03.000Z", "exif_datetime_original"),
            ("e", "2024-01-01T00:30:00.000Z", "filesystem"),
            ("f", "2024-01-01T02:00:00.000Z", "filesystem"),
        ];
        for (p, t, s) in rows {
            conn.execute(
                "INSERT INTO images (original_path, relative_path, filename, extension, file_size, import_status, created_at,
                                     normalized_datetime, timestamp_source, timestamp_confidence, timestamp_offset_minutes)
                 VALUES (?1, ?1, ?1, 'jpg', 1, 'imported', 0, ?2, ?3, 'high', 0)",
                params![p, t, s],
            )
            .unwrap();
        }
        // Median 1s × 10 = 10s, but the 1h floor wins: only e → f (90 min) is a gap.
        let r = gaps(&conn, GapOptions::default()).unwrap();
        assert_eq!(r.threshold_seconds, 3600);
        assert_eq!(r.gaps.len(), 1);
        assert_eq!(r.gaps[0].before.relative_path, "e");
        // With a lower floor, d → e (filesystem after EXIF) is flagged as a source change.
        let r = gaps(&conn, GapOptions { factor: 10.0, min_seconds: 60 }).unwrap();
        assert_eq!(r.gaps.len(), 2);
        assert!(r.gaps[0].source_changed);
        assert!(!r.gaps[1].source_changed);
    }
}
