//! Public facade for Phase 1: import & inventory.
//!
//! The Tauri shell is a thin adapter over this module; everything heavy (walking,
//! hashing, SQLite persistence, integrity checks) lives here so `crates/core`
//! stays free of any `tauri` dependency (CLAUDE.md "core invariants").
//!
//! Every public function takes only a source `root` and knows where its own
//! database is (via [`default_db_dir`]) — the shell does not have to think about paths.

use std::path::{Path, PathBuf};

mod dedup;
pub mod metadata;
mod scan;
pub mod store;

use crate::{Error, Result};

pub(crate) use dedup::hash_file as dedup_hash;
pub use scan::{Candidate, SUPPORTED_EXTENSIONS as IMAGE_EXTENSIONS};

/// Standard location of the project's database: `<root>/.timelapse/db`. Keeping it inside
/// the source tree means moving the folder moves its data too, and no global app-state
/// dir is required (ADR-011). The only public function with a `db_dir` parameter —
/// internal callers in this crate use `default_db_dir`, so the shell never has to.
pub fn default_db_dir(root: &Path) -> PathBuf {
    project_dir(root).join("db")
}

/// `<root>/.timelapse`: everything the app writes for this project (database, thumbnails,
/// quarantine). The import scan never descends into it.
pub fn project_dir(root: &Path) -> PathBuf {
    root.join(scan::PROJECT_DIR)
}

/// Display name for a source root when no explicit name is given: the folder's own name,
/// e.g. `C:\Photos\2024` → `2024`. Phase 2 will introduce an explicit project-file model
/// where this is renamed by the user; in Phase 1 the path is the identity (ADR-007).
fn derive_name(root: &Path) -> String {
    root.file_name()
        .and_then(|s| s.to_str())
        .map(String::from)
        .unwrap_or_else(|| root.to_string_lossy().into_owned())
}

/// Everything the UI needs to render the import report and inventory.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportReport {
    pub scanned_count: usize,
    pub hash_computed: usize,
    /// Number of files that are content duplicates (appear 2+ times in this tree).
    pub duplicates_found: usize,
    /// Distinct duplicate *groups:* one entry per distinct content appearing 2+ times.
    pub duplicate_groups: Vec<Vec<String>>,
    pub new_rows: usize,
    pub unchanged: usize,
    /// Files present on disk that could not be hashed (permissions / I/O). They are still
    /// imported without a hash so they can be inspected.
    pub skipped_unreadable: usize,
    /// Rows for hidden files (macOS `._*` and the like) left by an earlier import, removed.
    pub hidden_removed: usize,
}

/// Phase 1 "import": scan `root`, SHA-256 the supported files, deduplicate on byte equality
/// and upsert into the project's SQLite database (idempotent by original path). Returns
/// a report. The connection is dropped when this function returns — the shell holds no state.
pub fn import_folder(root: &Path) -> Result<ImportReport> {
    let name = derive_name(root);
    let conn = store::open(default_db_dir(root).as_path(), &name, root, Some(root))?;

    let candidates = scan::scan(root)?;
    let n = candidates.len();
    let hidden_removed = store::remove_hidden_entries(&conn)?;

    let mut hashes: Vec<Option<String>> = Vec::with_capacity(n);
    let mut skipped_unreadable = 0usize;
    for c in &candidates {
        match dedup::hash_file(&c.path) {
            Ok(h) => hashes.push(Some(h)),
            Err(Error::ImageUnreadable { .. }) => {
                hashes.push(None);
                skipped_unreadable += 1;
            }
            Err(e) => return Err(e),
        }
    }

    let groups = dedup::duplicate_groups(&candidates, &hashes);
    let mut dup_flag = vec![false; n];
    for g in &groups {
        for i in &g.indices {
            if let Some(slot) = dup_flag.get_mut(*i) {
                *slot = true;
            }
        }
    }

    let counts = store::upsert_images(&conn, &candidates, &hashes, &dup_flag)?;

    let duplicate_groups: Vec<Vec<String>> = groups
        .iter()
        .map(|g| {
            g.indices
                .iter()
                .map(|i| candidates[*i].relative_path.to_string_lossy().into_owned())
                .collect()
        })
        .collect();

    Ok(ImportReport {
        scanned_count: n,
        hash_computed: hashes.iter().filter(|h| h.is_some()).count(),
        duplicates_found: dup_flag.iter().filter(|v| **v).count(),
        duplicate_groups,
        new_rows: counts.new_rows,
        unchanged: counts.unchanged,
        skipped_unreadable,
        hidden_removed,
    })
}

/// List the inventory for `root`. Opens (and migrates) the project database before reading.
pub fn list_for_root(root: &Path) -> Result<Vec<store::ImageRow>> {
    let name = derive_name(root);
    let conn = store::open(default_db_dir(root).as_path(), &name, root, Some(root))?;
    store::list_images(&conn)
}

/// Integrity snapshot for `root`: missing files + duplicate groups. Read-only against the DB.
pub fn integrity_for_root(root: &Path) -> Result<store::IntegrityResult> {
    let name = derive_name(root);
    let conn = store::open(default_db_dir(root).as_path(), &name, root, Some(root))?;
    store::integrity_check(&conn)
}

pub(crate) fn open_root(root: &Path) -> Result<rusqlite::Connection> {
    store::open(default_db_dir(root).as_path(), &derive_name(root), root, Some(root))
}

/// Result of [`backfill_for_root`]: what this run did, and the state of the whole timeline.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackfillReport {
    /// Images that received a capture time in this run.
    pub newly_resolved: usize,
    /// Images examined in this run for which no source held a usable value.
    pub unresolved_this_run: usize,
    pub summary: store::TimelineSummary,
}

/// Phase 2 backfill: for every imported image without a capture time, read its EXIF /
/// filename / filesystem candidates, resolve and persist them (one transaction). Reads
/// originals only; never writes, moves or renames them. Idempotent.
pub fn backfill_for_root(root: &Path) -> Result<BackfillReport> {
    let conn = open_root(root)?;
    let items: Vec<(i64, metadata::Candidates)> = store::pending_timestamps(&conn)?
        .into_iter()
        .map(|p| (p.id, metadata::collect(Path::new(&p.original_path), p.modified_at)))
        .collect();
    let counts = store::backfill_timestamps(&conn, &items)?;
    Ok(BackfillReport {
        newly_resolved: counts.resolved,
        unresolved_this_run: counts.unresolved,
        summary: store::timeline_summary(&conn)?,
    })
}

/// The chronological timeline of `root` (unresolved images last, never dropped).
pub fn timeline_for_root(root: &Path) -> Result<Vec<store::TimelineEntry>> {
    store::timeline(&open_root(root)?)
}

/// Image counts per hour / day / week / month over an optional `[from, to)` period.
pub fn distribution_for_root(
    root: &Path,
    granularity: store::Granularity,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<store::DistributionBucket>> {
    store::distribution(&open_root(root)?, granularity, from, to)
}

/// One page of the photographs of `[from, to)` on the timeline of `root`.
pub fn timeline_period_for_root(
    root: &Path,
    from: &str,
    to: &str,
    offset: usize,
    limit: usize,
) -> Result<store::TimelinePage> {
    store::timeline_period(&open_root(root)?, from, to, offset, limit.min(1000))
}

/// Groups of photographs of `root` that share a capture time, the largest `limit` listed.
pub fn identical_timestamps_for_root(root: &Path, limit: usize) -> Result<store::IdenticalReport> {
    store::identical_timestamps(&open_root(root)?, limit.min(1000))
}

/// Unusually long intervals on the resolved timeline of `root`.
pub fn gaps_for_root(root: &Path, opts: store::GapOptions) -> Result<store::GapReport> {
    store::gaps(&open_root(root)?, opts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        // Always a fresh folder (process ids repeat between test runs).
        let d = std::env::temp_dir().join(format!("ts-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let _ = std::fs::create_dir_all(&d);
        d
    }

    #[test]
    fn hidden_files_are_skipped_and_old_rows_for_them_removed() {
        let root = tmp("hidden");
        std::fs::write(root.join("G001.JPG"), b"photo").unwrap();
        // What macOS writes on an exFAT/FAT drive, and the system folders it creates.
        std::fs::write(root.join("._G001.JPG"), b"\x00\x05\x16\x07appledouble").unwrap();
        std::fs::create_dir_all(root.join(".Trashes/501")).unwrap();
        std::fs::write(root.join(".Trashes/501/old.jpg"), b"trashed").unwrap();
        let report = import_folder(&root).unwrap();
        assert_eq!((report.scanned_count, report.new_rows, report.hidden_removed), (1, 1, 0));

        // A project imported by an earlier build holds a `._` row (with a status), and
        // another one that a saved selection uses.
        let conn = open_root(&root).unwrap();
        for (i, name) in ["._G001.JPG", "._G002.JPG"].iter().enumerate() {
            conn.execute(
                "INSERT INTO images (original_path, relative_path, filename, extension, file_size,
                                     import_status, created_at)
                 VALUES (?1, ?2, ?2, 'jpg', 4, 'imported', 0)",
                rusqlite::params![root.join(name).to_string_lossy(), name],
            )
            .unwrap();
            let id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO image_status (image_id, status, updated_at) VALUES (?1, 'flagged', 0)",
                [id],
            )
            .unwrap();
            if i == 1 {
                conn.execute(
                    "INSERT INTO sampling_runs (id, method, configuration_json, algorithm_version,
                         input_count, input_fingerprint, excluded_json, target_count,
                         selected_count, unmatched_count, unmatched_json, status, created_at)
                     VALUES (1, 'every-nth', '{}', 1, 1, '', '{}', 1, 1, 0, '[]', 'completed', 0)",
                    [],
                )
                .unwrap();
                conn.execute(
                    "INSERT INTO sampling_items (sampling_run_id, image_id, sequence_index,
                                                 actual_datetime)
                     VALUES (1, ?1, 0, '2024-01-01T00:00:00.000Z')",
                    [id],
                )
                .unwrap();
            }
        }
        let again = import_folder(&root).unwrap();
        assert_eq!(again.hidden_removed, 1, "the one a saved selection uses stays");
        let names: Vec<String> =
            list_for_root(&root).unwrap().into_iter().map(|r| r.filename).collect();
        assert_eq!(names.len(), 2);
        assert!(
            names.contains(&"G001.JPG".to_string()) && names.contains(&"._G002.JPG".to_string())
        );
        let statuses: i64 =
            conn.query_row("SELECT COUNT(*) FROM image_status", [], |r| r.get(0)).unwrap();
        assert_eq!(statuses, 1, "the removed row's status went with it");
    }

    #[test]
    fn empty_folder_is_a_valid_import() {
        let root = tmp("empty");
        let report = import_folder(&root).unwrap();
        assert_eq!(report.scanned_count, 0);
        assert_eq!(report.new_rows, 0);
    }

    #[test]
    fn identical_files_form_one_duplicate_group() {
        let root = tmp("dup");
        std::fs::write(root.join("a.jpg"), b"same").unwrap();
        std::fs::write(root.join("b.jpg"), b"same").unwrap();

        let report = import_folder(&root).unwrap();
        assert_eq!(report.scanned_count, 2);
        assert_eq!(report.duplicates_found, 2);
        assert_eq!(report.duplicate_groups.len(), 1);
        assert_eq!(report.duplicate_groups[0].len(), 2);
    }

    #[test]
    fn unsupported_extensions_are_ignored() {
        let root = tmp("skip");
        std::fs::write(root.join("readme.txt"), b"hi").unwrap();
        std::fs::write(root.join("photo.jpg"), b"jpg").unwrap();

        let report = import_folder(&root).unwrap();
        assert_eq!(report.scanned_count, 1, "only .jpg should be imported");
    }

    #[test]
    fn the_project_directory_is_never_imported() {
        let root = tmp("projdir");
        let _ = std::fs::remove_dir_all(project_dir(&root));
        std::fs::write(root.join("keep.jpg"), b"a").unwrap();
        let q = project_dir(&root).join("quarantine");
        std::fs::create_dir_all(&q).unwrap();
        std::fs::write(q.join("moved.jpg"), b"b").unwrap();
        // A nested project's data (another `.timelapse/`, with its thumbnails) is hidden
        // too: like every hidden folder, it is not part of the collection.
        let nested = root.join("trip").join(".timelapse");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("x.jpg"), b"c").unwrap();
        std::fs::write(root.join("trip").join("y.jpg"), b"d").unwrap();

        let report = import_folder(&root).unwrap();
        assert_eq!(report.scanned_count, 2, "keep.jpg and trip/y.jpg only");
        let rows = list_for_root(&root).unwrap();
        assert!(rows.iter().all(|r| !r.relative_path.contains("moved")));
    }

    #[test]
    fn list_for_root_returns_the_rows_that_were_imported() {
        let root = tmp("list");
        std::fs::write(root.join("photo.jpg"), b"jpg").unwrap();
        import_folder(&root).unwrap();

        let rows = list_for_root(&root).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relative_path, "photo.jpg");
    }

    #[test]
    fn integrity_reports_a_missing_file_when_the_on_disk_copy_is_deleted() {
        let root = tmp("integ-missing");
        std::fs::write(root.join("gone.jpg"), b"jpg").unwrap();
        import_folder(&root).unwrap();
        // Simulate the user deleting a file after import.
        std::fs::remove_file(root.join("gone.jpg")).unwrap();

        let res = integrity_for_root(&root).unwrap();
        assert_eq!(res.missing_files.len(), 1);
    }
}

/// Phase 2 exit criterion (DEVELOPMENT_ROADMAP.md): for an imported set, the timeline is
/// monotonic with one normalised instant per image; mixed time zones, missing EXIF and
/// burst offsets are covered. Full path on real files: import → backfill → queries.
#[cfg(test)]
mod timeline_exit_tests {
    use std::time::{Duration, SystemTime};

    use super::metadata::fixture::{ExifSpec, TempDir, write_jpeg};
    use super::store::{GapOptions, Granularity, TimelineEntry};
    use super::*;

    fn set_mtime(path: &Path, unix: u64) {
        let f = std::fs::File::options().write(true).open(path).unwrap();
        f.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(unix)).unwrap();
    }

    fn by_path<'a>(t: &'a [TimelineEntry], p: &str) -> &'a TimelineEntry {
        t.iter().find(|e| e.relative_path == p).unwrap_or_else(|| panic!("{p} on the timeline"))
    }

    fn assert_monotonic(t: &[TimelineEntry]) {
        let resolved: Vec<i64> = t.iter().map_while(|e| e.epoch_ms).collect();
        assert!(resolved.windows(2).all(|w| w[0] <= w[1]), "timeline must be monotonic: {t:#?}");
        assert!(t[resolved.len()..].iter().all(|e| e.normalized_datetime.is_none()));
    }

    /// 2024-01-15T09:00:00Z
    const FS_0900Z: u64 = 1_705_309_200;

    fn mixed_folder() -> TempDir {
        let dir = TempDir::new("exit-mixed");
        let r = dir.path();
        // EXIF with an explicit +05:00 offset: 10:00 local → 05:00Z.
        let tz = ExifSpec {
            original: Some("2024:01:15 10:00:00"),
            offset_original: Some("+05:00"),
            ..ExifSpec::default()
        };
        write_jpeg(&r.join("tz_east.jpg"), &tz);
        // EXIF with no zone (assumed UTC) whose filename says another day: EXIF must win.
        write_jpeg(&r.join("IMG_20190704_b.jpg"), &ExifSpec::original("2024:01:15 07:00:00"));
        // Only DateTimeDigitized.
        let dig = ExifSpec { digitized: Some("2024:01:15 06:00:00"), ..ExifSpec::default() };
        write_jpeg(&r.join("digitized.jpg"), &dig);
        // No EXIF at all → filename.
        write_jpeg(&r.join("IMG_20240115_080000.jpg"), &ExifSpec::none());
        // A placeholder EXIF value and nothing in the name → filesystem time.
        let png = r.join("scan.png");
        std::fs::write(&png, b"\x89PNG\r\n\x1a\n").unwrap();
        set_mtime(&png, FS_0900Z);
        let zero = r.join("zeroed.jpg");
        write_jpeg(&zero, &ExifSpec::original("0000:00:00 00:00:00"));
        set_mtime(&zero, FS_0900Z + 60);
        dir
    }

    #[test]
    fn mixed_sources_and_zones_resolve_to_one_monotonic_timeline() {
        let dir = mixed_folder();
        let root = dir.path();
        import_folder(root).unwrap();
        let report = backfill_for_root(root).unwrap();
        assert_eq!(report.newly_resolved, 6);
        assert_eq!(report.summary.unresolved, 0);
        assert_eq!(report.summary.by_source.exif_datetime_original, 2);
        assert_eq!(report.summary.by_source.exif_datetime_digitized, 1);
        assert_eq!(report.summary.by_source.filename, 1);
        assert_eq!(report.summary.by_source.filesystem, 2);
        assert_eq!(report.summary.low_confidence, 2);
        assert_eq!(report.summary.tz_assumed, 3, "IMG_…_b, digitized, IMG_…_080000");

        let t = timeline_for_root(root).unwrap();
        assert_monotonic(&t);
        let order: Vec<&str> = t.iter().map(|e| e.relative_path.as_str()).collect();
        assert_eq!(
            order,
            [
                "tz_east.jpg",
                "digitized.jpg",
                "IMG_20190704_b.jpg",
                "IMG_20240115_080000.jpg",
                "scan.png",
                "zeroed.jpg"
            ]
        );
        let east = by_path(&t, "tz_east.jpg");
        assert_eq!(east.normalized_datetime.as_deref(), Some("2024-01-15T05:00:00.000Z"));
        assert_eq!((east.offset_minutes, east.tz_assumed), (Some(300), false));
        let b = by_path(&t, "IMG_20190704_b.jpg");
        assert_eq!(b.source.as_deref(), Some("exif_datetime_original"));
        assert_eq!(b.confidence.as_deref(), Some("high"));
        assert!(b.tz_assumed);
        assert_eq!(by_path(&t, "digitized.jpg").confidence.as_deref(), Some("medium"));
        let zeroed = by_path(&t, "zeroed.jpg");
        assert_eq!(zeroed.source.as_deref(), Some("filesystem"));
        assert_eq!(zeroed.epoch_ms, Some((FS_0900Z as i64 + 60) * 1000));
    }

    #[test]
    fn missing_exif_falls_back_and_the_original_values_are_preserved() {
        let dir = mixed_folder();
        let root = dir.path();
        import_folder(root).unwrap();
        backfill_for_root(root).unwrap();

        let conn = open_root(root).unwrap();
        let row = |name: &str| -> (Option<String>, Option<String>, Option<String>, Option<String>) {
            conn.query_row(
                "SELECT exif_datetime_original, filename_datetime, filesystem_datetime, timestamp_source
                 FROM images WHERE relative_path = ?1",
                [name],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap()
        };
        let (exif, name, fs, src) = row("IMG_20240115_080000.jpg");
        assert_eq!(exif, None);
        assert_eq!(name.as_deref(), Some("20240115_080000"));
        assert!(fs.is_some(), "filesystem candidate recorded even when it loses");
        assert_eq!(src.as_deref(), Some("filename"));

        let (exif, _, fs, src) = row("zeroed.jpg");
        assert_eq!(exif.as_deref(), Some("0000:00:00 00:00:00"), "kept verbatim, not repaired");
        assert_eq!(fs.as_deref(), Some("2024-01-15T09:01:00.000Z"));
        assert_eq!(src.as_deref(), Some("filesystem"));

        let (exif, name, _, src) = row("IMG_20190704_b.jpg");
        assert_eq!(exif.as_deref(), Some("2024:01:15 07:00:00"));
        assert_eq!(name.as_deref(), Some("20190704"), "losing filename value kept");
        assert_eq!(src.as_deref(), Some("exif_datetime_original"));
    }

    #[test]
    fn an_image_with_no_usable_source_stays_on_the_timeline_marked_unresolved() {
        let dir = TempDir::new("exit-unresolved");
        let root = dir.path();
        write_jpeg(&root.join("a.jpg"), &ExifSpec::original("2024:01:15 10:00:00"));
        write_jpeg(&root.join("lost.jpg"), &ExifSpec::none());
        import_folder(root).unwrap();
        // Simulate an unknown filesystem time (the scan's 0) on the file with nothing else.
        open_root(root)
            .unwrap()
            .execute("UPDATE images SET modified_at = 0 WHERE relative_path = 'lost.jpg'", [])
            .unwrap();
        let report = backfill_for_root(root).unwrap();
        assert_eq!((report.newly_resolved, report.unresolved_this_run), (1, 1));
        assert_eq!(report.summary.unresolved, 1);
        let t = timeline_for_root(root).unwrap();
        assert_eq!(t.len(), 2, "never dropped");
        assert_monotonic(&t);
        let lost = &t[1];
        assert_eq!(lost.relative_path, "lost.jpg");
        assert_eq!((lost.normalized_datetime.as_ref(), lost.source.as_ref()), (None, None));
        assert!(!lost.tz_assumed);
    }

    #[test]
    fn burst_offsets_are_recorded_exactly_and_never_reconciled() {
        let dir = TempDir::new("exit-burst");
        let root = dir.path();
        // Six frames in one second; sub-second values are out of order w.r.t. the file
        // names, and two frames share the very same instant.
        let subsecs = ["50", "10", "90", "30", "70", "30"];
        for (i, ss) in subsecs.iter().enumerate() {
            let spec = ExifSpec {
                original: Some("2024:01:15 10:30:00"),
                subsec_original: Some(ss),
                ..ExifSpec::default()
            };
            write_jpeg(&root.join(format!("burst_{}.jpg", i + 1)), &spec);
        }
        import_folder(root).unwrap();
        backfill_for_root(root).unwrap();
        let t = timeline_for_root(root).unwrap();
        assert_monotonic(&t);
        // Each value is exactly what its camera recorded — nothing spread out or snapped.
        for (i, ss) in subsecs.iter().enumerate() {
            let e = by_path(&t, &format!("burst_{}.jpg", i + 1));
            assert_eq!(
                e.normalized_datetime.as_deref(),
                Some(format!("2024-01-15T10:30:00.{ss}0Z").as_str())
            );
            assert_eq!(e.confidence.as_deref(), Some("high"));
        }
        let order: Vec<&str> = t.iter().map(|e| e.relative_path.as_str()).collect();
        // Sorted by recorded instant; the tie (burst_4 / burst_6) is broken by path.
        assert_eq!(
            order,
            [
                "burst_2.jpg",
                "burst_4.jpg",
                "burst_6.jpg",
                "burst_1.jpg",
                "burst_5.jpg",
                "burst_3.jpg"
            ]
        );
        assert!(gaps_for_root(root, GapOptions::default()).unwrap().gaps.is_empty());
    }

    #[test]
    fn backfill_is_idempotent_and_picks_up_newly_imported_files() {
        let dir = TempDir::new("exit-idem");
        let root = dir.path();
        write_jpeg(&root.join("a.jpg"), &ExifSpec::original("2024:01:15 10:00:00"));
        import_folder(root).unwrap();
        assert_eq!(backfill_for_root(root).unwrap().newly_resolved, 1);
        let before = timeline_for_root(root).unwrap();

        write_jpeg(&root.join("b.jpg"), &ExifSpec::original("2024:01:15 11:00:00"));
        import_folder(root).unwrap();
        let again = backfill_for_root(root).unwrap();
        assert_eq!(again.newly_resolved, 1, "only the new file");
        assert_eq!(backfill_for_root(root).unwrap().newly_resolved, 0);
        let after = timeline_for_root(root).unwrap();
        assert_eq!(after[0], before[0], "already-resolved row untouched");
        assert_eq!(after.len(), 2);
    }

    #[test]
    fn weeks_periods_and_identical_times() {
        let dir = TempDir::new("p7-weeks");
        let root = dir.path();
        // 2024-01-14 is a Sunday, 2024-01-15 a Monday.
        for (name, at) in [
            ("sun.jpg", "2024:01:14 23:59:59"),
            ("mon.jpg", "2024:01:15 00:00:00"),
            ("burst1.jpg", "2024:01:17 12:00:00"),
            ("burst2.jpg", "2024:01:17 12:00:00"),
            ("burst3.jpg", "2024:01:17 12:00:00"),
            ("pair1.jpg", "2024:01:20 08:00:00"),
            ("pair2.jpg", "2024:01:20 08:00:00"),
            ("next.jpg", "2024:01:22 08:00:00"),
        ] {
            write_jpeg(&root.join(name), &ExifSpec::original(at));
        }
        import_folder(root).unwrap();
        backfill_for_root(root).unwrap();

        let weeks = distribution_for_root(root, Granularity::Week, None, None).unwrap();
        let weeks: Vec<(&str, usize)> =
            weeks.iter().map(|b| (b.bucket.as_str(), b.count)).collect();
        assert_eq!(weeks, [("2024-01-08", 1), ("2024-01-15", 6), ("2024-01-22", 1)]);

        let page = timeline_period_for_root(root, "2024-01-15", "2024-01-22", 1, 3).unwrap();
        assert_eq!(page.total, 6);
        let names: Vec<&str> = page.entries.iter().map(|e| e.relative_path.as_str()).collect();
        assert_eq!(names, ["burst1.jpg", "burst2.jpg", "burst3.jpg"]);
        assert!(
            timeline_period_for_root(root, "2024-01-15", "2024-01-22", 6, 10)
                .unwrap()
                .entries
                .is_empty()
        );

        let same = identical_timestamps_for_root(root, 1).unwrap();
        assert_eq!((same.groups, same.photos), (2, 5));
        assert_eq!(same.largest.len(), 1, "limited");
        assert_eq!(same.largest[0].count, 3);
        assert_eq!(same.largest[0].normalized_datetime, "2024-01-17T12:00:00.000Z");
        assert_eq!(same.largest[0].sample.len(), 3);
        assert_eq!(identical_timestamps_for_root(root, 10).unwrap().largest[1].count, 2);
    }

    #[test]
    fn distribution_and_gaps_over_a_real_folder() {
        let dir = TempDir::new("exit-dist");
        let root = dir.path();
        let shots: [&'static str; 5] = [
            "2024:01:15 10:00:00",
            "2024:01:15 10:10:00",
            "2024:01:15 10:20:00",
            "2024:01:15 11:30:00",
            "2024:01:18 09:00:00",
        ];
        for (i, s) in shots.iter().enumerate() {
            write_jpeg(&root.join(format!("p{i}.jpg")), &ExifSpec::original(s));
        }
        // A filename-dated frame inside the last stretch.
        write_jpeg(&root.join("IMG_20240118_091000.jpg"), &ExifSpec::none());
        import_folder(root).unwrap();
        backfill_for_root(root).unwrap();

        let days = distribution_for_root(root, Granularity::Day, None, None).unwrap();
        let days: Vec<(&str, usize)> = days.iter().map(|b| (b.bucket.as_str(), b.count)).collect();
        assert_eq!(days, [("2024-01-15", 4), ("2024-01-18", 2)]);
        let hours = distribution_for_root(root, Granularity::Hour, None, None).unwrap();
        assert_eq!(hours[0].bucket, "2024-01-15T10");
        // A bucket key is a valid `from` bound: drilling into that hour keeps the 10:00:00.000 shot.
        let drill = distribution_for_root(
            root,
            Granularity::Hour,
            Some(&hours[0].bucket),
            Some("2024-01-15T11"),
        )
        .unwrap();
        assert_eq!(drill.len(), 1);
        assert_eq!(drill[0].count, 3);
        assert_eq!(hours[0].count, 3);
        let month = distribution_for_root(root, Granularity::Month, None, None).unwrap();
        assert_eq!(month.len(), 1);
        assert_eq!(month[0].count, 6);
        let period =
            distribution_for_root(root, Granularity::Day, Some("2024-01-16"), Some("2024-02"))
                .unwrap();
        assert_eq!(period.len(), 1);
        assert_eq!(period[0].bucket, "2024-01-18");

        // Intervals: 10m, 10m, 70m, ~3 days, 10m → median 10m; threshold max(1h, 100m).
        let report = gaps_for_root(root, GapOptions::default()).unwrap();
        assert_eq!(report.median_interval_seconds, Some(600.0));
        assert_eq!(report.threshold_seconds, 6000);
        assert_eq!(report.gaps.len(), 1);
        let g = &report.gaps[0];
        assert_eq!(g.before.relative_path, "p3.jpg");
        assert_eq!(g.after.relative_path, "p4.jpg");
        assert_eq!(g.seconds, 69 * 3600 + 30 * 60, "Jan 15 11:30 → Jan 18 09:00");
        assert!(!g.source_changed);
        // Reporting a gap changes nothing in the data.
        let t = timeline_for_root(root).unwrap();
        assert_eq!(
            by_path(&t, "p4.jpg").normalized_datetime.as_deref(),
            Some("2024-01-18T09:00:00.000Z")
        );
    }
}
