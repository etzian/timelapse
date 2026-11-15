//! Phase 3 exit criterion (DEVELOPMENT_ROADMAP.md): cleaning decisions are reproducible
//! from SQLite + original files, and quarantined images are recoverable without data loss.
//! Full path on real files: import → analyse → classify → decide → quarantine → restore.

use std::fs;
use std::path::Path;

use super::analysis::tests::write_image;
use super::*;
use crate::files::quarantine_dir;
use crate::import::metadata::fixture::TempDir;
use crate::import::{import_folder, integrity_for_root};

const BRIGHT: [u8; 3] = [200, 200, 200];
const DARK: [u8; 3] = [5, 5, 5];

struct Collection {
    dir: TempDir,
}

impl Collection {
    /// bright.png, half.png (50 % dark, mean 105), dark.png, night/dark2.tiff,
    /// lr_reject.png (bright, Lightroom-rejected via sidecar), junk.jpg (undecodable).
    fn new(tag: &str) -> Self {
        let dir = TempDir::new(tag);
        let r = dir.path();
        write_image(&r.join("bright.png"), 64, 64, |_, _| BRIGHT);
        write_image(&r.join("half.png"), 64, 64, |x, _| if x < 32 { [10, 10, 10] } else { BRIGHT });
        write_image(&r.join("dark.png"), 64, 64, |_, _| DARK);
        fs::create_dir_all(r.join("night")).unwrap();
        write_image(&r.join("night").join("dark2.tiff"), 32, 32, |_, _| DARK);
        write_image(&r.join("lr_reject.png"), 64, 64, |_, _| BRIGHT);
        fs::write(
            r.join("lr_reject.xmp"),
            r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:Description xmp:Rating="-1"/></x:xmpmeta>"#,
        )
        .unwrap();
        fs::write(r.join("junk.jpg"), b"not an image at all").unwrap();
        import_folder(r).unwrap();
        Self { dir }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn id(&self, rel: &str) -> i64 {
        // Stored relative paths use the platform separator.
        let same = |stored: &str| stored.replace('\\', "/") == rel;
        review_for_root(self.root(), &[])
            .unwrap()
            .into_iter()
            .find(|i| same(&i.relative_path))
            .unwrap_or_else(|| panic!("{rel} imported"))
            .id
    }

    fn item(&self, rel: &str) -> ReviewItem {
        let id = self.id(rel);
        review_for_root(self.root(), &[]).unwrap().into_iter().find(|i| i.id == id).unwrap()
    }

    fn statuses(&self) -> Vec<(String, Option<String>, Vec<Reason>)> {
        review_for_root(self.root(), &[])
            .unwrap()
            .into_iter()
            .map(|i| (i.relative_path, i.status, i.reasons))
            .collect()
    }
}

fn analyse(c: &Collection) -> AnalyzeReport {
    analyze_for_root(c.root(), 0, |_| {}).unwrap()
}

#[test]
fn a_stopped_analysis_keeps_its_work_and_the_next_run_continues() {
    let c = Collection::new("stop");
    let token = 9_001;
    cancel_analysis(token);
    let stopped = analyze_for_root(c.root(), token, |_| {}).unwrap();
    cancel_analysis(0);
    assert!(stopped.cancelled);
    assert_eq!(stopped.analyzed, 0);
    assert!(stopped.remaining > 0);
    // Another run (another token) is not affected, and measures everything left.
    let rest = analyze_for_root(c.root(), token + 1, |_| {}).unwrap();
    assert!(!rest.cancelled);
    assert_eq!((rest.analyzed, rest.remaining), (stopped.remaining, 0));
}

#[test]
fn analysis_classifies_brightness_xmp_and_failures() {
    let c = Collection::new("p3-classify");
    let mut seen = Vec::new();
    let report = analyze_for_root(c.root(), 0, |p| seen.push(p)).unwrap();
    assert_eq!((report.analyzed, report.failed), (6, 1));
    assert_eq!(seen.first(), Some(&AnalyzeProgress { done: 0, total: 6 }));
    assert_eq!(seen.last(), Some(&AnalyzeProgress { done: 6, total: 6 }));
    assert_eq!(
        report.counts,
        StatusCounts { eligible: 2, flagged: 4, rejected: 0, quarantined: 0, unanalyzed: 0 }
    );
    assert_eq!(c.item("bright.png").status.as_deref(), Some("eligible"));
    assert_eq!(c.item("half.png").status.as_deref(), Some("eligible"), "50 % dark ≤ 60 %");
    let dark = c.item("dark.png");
    assert_eq!(dark.status.as_deref(), Some("flagged"));
    assert!(matches!(dark.reasons[0], Reason::MeanBelow { .. }));
    assert!(matches!(dark.reasons[1], Reason::TooManyDarkPixels { .. }));
    assert_eq!(dark.dark_fraction, Some(1.0));
    assert_eq!(dark.histogram.len(), analysis::DISPLAY_BINS);
    let lr = c.item("lr_reject.png");
    assert_eq!(
        (lr.reasons.as_slice(), lr.xmp_rating),
        ([Reason::XmpRejected].as_slice(), Some(-1))
    );
    let junk = c.item("junk.jpg");
    assert_eq!(junk.reasons, [Reason::AnalysisFailed { failure: "undecodable".into() }]);
    assert_eq!(junk.failure.as_deref(), Some("undecodable"));

    // Idempotent.
    assert_eq!(analyse(&c).analyzed, 0);
}

#[test]
fn decisions_are_reproducible_from_sqlite_alone() {
    let c = Collection::new("p3-repro");
    analyse(&c);
    decide_for_root(c.root(), &[c.id("dark.png")], Decision::Reject).unwrap();
    decide_for_root(c.root(), &[c.id("lr_reject.png")], Decision::Accept).unwrap();
    let before = c.statuses();

    // Reclassifying from SQLite alone, even with every original moved away, gives the same.
    let away = c.root().with_extension("away");
    let _ = fs::remove_dir_all(&away);
    fs::create_dir_all(&away).unwrap();
    for f in ["bright.png", "half.png", "dark.png", "lr_reject.png", "junk.jpg"] {
        fs::rename(c.root().join(f), away.join(f)).unwrap();
    }
    classify_for_root(c.root()).unwrap();
    assert_eq!(c.statuses(), before);
    for f in ["bright.png", "half.png", "dark.png", "lr_reject.png", "junk.jpg"] {
        fs::rename(away.join(f), c.root().join(f)).unwrap();
    }
    let _ = fs::remove_dir_all(&away);

    // New thresholds change only undecided rows; restoring them restores every decision.
    let lenient = Thresholds {
        min_mean_luma: 0.0,
        max_dark_fraction: 1.0,
        flag_xmp_rejected: false,
        ..Thresholds::default()
    };
    let counts = set_thresholds_for_root(c.root(), lenient).unwrap().counts;
    assert_eq!(c.item("night/dark2.tiff").status.as_deref(), Some("eligible"));
    assert_eq!(c.item("dark.png").status.as_deref(), Some("rejected"), "user decision kept");
    assert_eq!(c.item("lr_reject.png").status.as_deref(), Some("eligible"));
    assert_eq!(counts.rejected, 1);
    assert_eq!(thresholds_for_root(c.root()).unwrap(), lenient);
    set_thresholds_for_root(c.root(), Thresholds::default()).unwrap();
    assert_eq!(c.statuses(), before);

    // Each automatic reason records the thresholds it was made with.
    let conn = open_root(c.root()).unwrap();
    let reason: String = conn
        .query_row("SELECT reason FROM image_status WHERE image_id = ?1", [c.id("half.png")], |r| {
            r.get(0)
        })
        .unwrap();
    let stored: store::StoredReason = serde_json::from_str(&reason).unwrap();
    assert_eq!(stored.thresholds, Thresholds::default());

    // Clearing a decision hands the image back to the classifier.
    decide_for_root(c.root(), &[c.id("dark.png")], Decision::Clear).unwrap();
    assert_eq!(c.item("dark.png").status.as_deref(), Some("flagged"));
}

#[test]
fn quarantine_and_restore_round_trip_byte_identically() {
    let c = Collection::new("p3-roundtrip");
    analyse(&c);
    let originals: Vec<(String, Vec<u8>)> = ["dark.png", "night/dark2.tiff"]
        .iter()
        .map(|f| (f.to_string(), fs::read(c.root().join(f)).unwrap()))
        .collect();
    let ids = [c.id("dark.png"), c.id("night/dark2.tiff")];
    decide_for_root(c.root(), &ids, Decision::Reject).unwrap();

    let out = quarantine_for_root(c.root(), &ids).unwrap();
    assert!(out.iter().all(OpOutcome::ok), "{out:?}");
    for (rel, bytes) in &originals {
        assert!(!c.root().join(rel).exists(), "moved out of the collection");
        assert_eq!(&fs::read(quarantine_dir(c.root()).join(rel)).unwrap(), bytes, "byte-identical");
    }
    assert_eq!(c.item("dark.png").status.as_deref(), Some("quarantined"));
    assert!(c.item("dark.png").current_path.is_some());
    assert!(
        integrity_for_root(c.root()).unwrap().missing_files.is_empty(),
        "located in quarantine"
    );
    // Thumbnails still work from quarantine, and a re-import does not bring the files back.
    assert!(!thumbnail_for_root(c.root(), ids[0], 256).unwrap().is_empty());
    let again = import_folder(c.root()).unwrap();
    assert_eq!((again.scanned_count, again.new_rows), (4, 0));
    // Decisions cannot be changed while quarantined.
    decide_for_root(c.root(), &ids, Decision::Accept).unwrap();
    assert_eq!(c.item("dark.png").status.as_deref(), Some("quarantined"));

    let back = restore_for_root(c.root(), &ids).unwrap();
    assert!(back.iter().all(OpOutcome::ok), "{back:?}");
    for (rel, bytes) in &originals {
        assert_eq!(&fs::read(c.root().join(rel)).unwrap(), bytes, "restored byte-identical");
        assert!(!quarantine_dir(c.root()).join(rel).exists());
    }
    let dark = c.item("dark.png");
    assert_eq!((dark.status.as_deref(), dark.current_path), (Some("rejected"), None));

    let ops = operations_for_root(c.root(), 10).unwrap();
    assert_eq!(ops.len(), 4);
    assert!(ops.iter().all(|o| o.status == "completed"));
    assert_eq!(ops[0].operation_type, "restore");
}

#[test]
fn quarantine_refuses_anything_unsafe_and_touches_nothing() {
    let c = Collection::new("p3-guards");
    analyse(&c);
    let dark = c.id("dark.png");
    let dark_bytes = fs::read(c.root().join("dark.png")).unwrap();

    // Only explicitly rejected images qualify.
    let out = quarantine_for_root(c.root(), &[dark, 999_999]).unwrap();
    assert_eq!(out[0].failure, Some("not-rejected"));
    assert_eq!(out[1].failure, Some("unknown-image"));

    // An occupied destination is never overwritten.
    decide_for_root(c.root(), &[dark], Decision::Reject).unwrap();
    fs::create_dir_all(quarantine_dir(c.root())).unwrap();
    fs::write(quarantine_dir(c.root()).join("dark.png"), b"someone else").unwrap();
    let out = quarantine_for_root(c.root(), &[dark]).unwrap();
    assert_eq!(out[0].failure, Some("destination-exists"));
    assert_eq!(fs::read(c.root().join("dark.png")).unwrap(), dark_bytes);
    assert_eq!(fs::read(quarantine_dir(c.root()).join("dark.png")).unwrap(), b"someone else");
    fs::remove_file(quarantine_dir(c.root()).join("dark.png")).unwrap();

    // A file changed since import is not moved.
    let d2 = c.id("night/dark2.tiff");
    decide_for_root(c.root(), &[d2], Decision::Reject).unwrap();
    fs::write(c.root().join("night").join("dark2.tiff"), b"edited").unwrap();
    let out = quarantine_for_root(c.root(), &[d2]).unwrap();
    assert_eq!(out[0].failure, Some("changed-since-import"));
    assert!(c.root().join("night").join("dark2.tiff").exists());

    // Restore never overwrites a file that reappeared at the original path.
    assert!(quarantine_for_root(c.root(), &[dark]).unwrap()[0].ok());
    fs::write(c.root().join("dark.png"), b"new file with the same name").unwrap();
    let out = restore_for_root(c.root(), &[dark]).unwrap();
    assert_eq!(out[0].failure, Some("destination-exists"));
    assert_eq!(fs::read(quarantine_dir(c.root()).join("dark.png")).unwrap(), dark_bytes);
    assert_eq!(c.item("dark.png").status.as_deref(), Some("quarantined"));
    assert_eq!(
        restore_for_root(c.root(), &[c.id("bright.png")]).unwrap()[0].failure,
        Some("not-quarantined")
    );

    // Every attempt is in the journal.
    let failures: Vec<Option<String>> =
        operations_for_root(c.root(), 50).unwrap().into_iter().map(|o| o.failure).collect();
    assert!(failures.contains(&Some("destination-exists".into())));
    assert!(failures.contains(&Some("changed-since-import".into())));
}

#[test]
fn a_crash_mid_operation_is_reconciled_on_the_next_open() {
    let c = Collection::new("p3-crash");
    analyse(&c);
    let dark = c.id("dark.png");
    let half = c.id("half.png");
    decide_for_root(c.root(), &[dark, half], Decision::Reject).unwrap();
    let hash = |p: &Path| crate::import::dedup_hash(p).unwrap();
    let src_dark = c.root().join("dark.png");
    let dst_dark = quarantine_dir(c.root()).join("dark.png");
    let src_half = c.root().join("half.png");
    let dst_half = quarantine_dir(c.root()).join("half.png");
    {
        // Journal two pending moves as `files::quarantine` would, then "crash": one after the
        // rename, one before it.
        let conn = open_root(c.root()).unwrap();
        for (id, s, d) in [(dark, &src_dark, &dst_dark), (half, &src_half, &dst_half)] {
            conn.execute(
                "INSERT INTO file_operations (image_id, operation_type, source_path, destination_path, status, checksum, created_at)
                 VALUES (?1, 'quarantine', ?2, ?3, 'pending', ?4, 0)",
                rusqlite::params![id, s.to_string_lossy(), d.to_string_lossy(), hash(s)],
            )
            .unwrap();
        }
        fs::create_dir_all(quarantine_dir(c.root())).unwrap();
        fs::rename(&src_dark, &dst_dark).unwrap();
    }
    // Reconcile (also run on every idle open; called explicitly here because tests in this
    // process may be running file operations in parallel).
    crate::files::reconcile(&open_root(c.root()).unwrap()).unwrap();
    let items = review_for_root(c.root(), &[]).unwrap();
    let find = |id: i64| items.iter().find(|i| i.id == id).unwrap().clone();
    assert_eq!(find(dark).status.as_deref(), Some("quarantined"));
    assert_eq!(find(dark).current_path.as_deref(), Some(dst_dark.to_string_lossy().as_ref()));
    assert_eq!(find(half).status.as_deref(), Some("rejected"));
    assert!(src_half.exists() && !dst_half.exists());
    let ops = operations_for_root(c.root(), 10).unwrap();
    assert!(ops.iter().any(|o| o.status == "completed"));
    assert!(ops.iter().any(|o| o.failure.as_deref() == Some("interrupted")));
    assert!(ops.iter().all(|o| o.status != "pending"));
    // And the reconciled quarantine is restorable.
    assert!(restore_for_root(c.root(), &[dark]).unwrap()[0].ok());
    assert!(src_dark.exists());
}

#[test]
fn re_analysing_the_originals_reproduces_measurements_and_decisions() {
    let c = Collection::new("p3-reanalyse");
    analyse(&c);
    decide_for_root(c.root(), &[c.id("bright.png")], Decision::Reject).unwrap();
    let snapshot = |c: &Collection| -> Vec<(i64, Option<String>, Option<f64>)> {
        let conn = open_root(c.root()).unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT image_id, histogram_json, mean_luma FROM image_analysis ORDER BY image_id",
            )
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    let (measures, statuses) = (snapshot(&c), c.statuses());
    open_root(c.root()).unwrap().execute("DELETE FROM image_analysis", []).unwrap();
    assert_eq!(analyse(&c).analyzed, 6);
    assert_eq!(snapshot(&c), measures, "same originals → same measurements");
    assert_eq!(c.statuses(), statuses, "→ same decisions");
}

#[test]
fn a_crash_between_placing_and_removing_the_source_is_undone() {
    let c = Collection::new("p3-crash-both");
    analyse(&c);
    let dark = c.id("dark.png");
    decide_for_root(c.root(), &[dark], Decision::Reject).unwrap();
    let src = c.root().join("dark.png");
    let dst = quarantine_dir(c.root()).join("dark.png");
    let bytes = fs::read(&src).unwrap();
    {
        let conn = open_root(c.root()).unwrap();
        conn.execute(
            "INSERT INTO file_operations (image_id, operation_type, source_path, destination_path, status, checksum, created_at)
             VALUES (?1, 'quarantine', ?2, ?3, 'pending', ?4, 0)",
            rusqlite::params![dark, src.to_string_lossy(), dst.to_string_lossy(), crate::import::dedup_hash(&src).unwrap()],
        )
        .unwrap();
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(&src, &dst).unwrap(); // placed, source not yet removed
        crate::files::reconcile(&conn).unwrap();
    }
    assert_eq!(fs::read(&src).unwrap(), bytes, "original untouched");
    assert!(!dst.exists(), "the duplicate we created is gone");
    assert_eq!(c.item("dark.png").status.as_deref(), Some("rejected"));
    let op = &operations_for_root(c.root(), 1).unwrap()[0];
    assert_eq!((op.status.as_str(), op.failure.as_deref()), ("failed", Some("interrupted")));
}

#[test]
fn a_partial_copy_left_by_a_crash_is_cleared() {
    let c = Collection::new("p3-crash-partial");
    analyse(&c);
    let dark = c.id("dark.png");
    decide_for_root(c.root(), &[dark], Decision::Reject).unwrap();
    let src = c.root().join("dark.png");
    let dst = quarantine_dir(c.root()).join("dark.png");
    let bytes = fs::read(&src).unwrap();
    {
        let conn = open_root(c.root()).unwrap();
        conn.execute(
            "INSERT INTO file_operations (image_id, operation_type, source_path, destination_path, status, checksum, created_at)
             VALUES (?1, 'quarantine', ?2, ?3, 'pending', ?4, 0)",
            rusqlite::params![dark, src.to_string_lossy(), dst.to_string_lossy(), crate::import::dedup_hash(&src).unwrap()],
        )
        .unwrap();
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::write(&dst, &bytes[..bytes.len() / 2]).unwrap();
        crate::files::reconcile(&conn).unwrap();
    }
    assert!(!dst.exists());
    assert_eq!(fs::read(&src).unwrap(), bytes);
    // The photo can be quarantined normally afterwards.
    assert!(quarantine_for_root(c.root(), &[dark]).unwrap()[0].ok());
}

#[test]
fn a_crash_during_restore_is_completed_from_disk() {
    let c = Collection::new("p3-crash-restore");
    analyse(&c);
    let dark = c.id("dark.png");
    decide_for_root(c.root(), &[dark], Decision::Reject).unwrap();
    assert!(quarantine_for_root(c.root(), &[dark]).unwrap()[0].ok());
    let q = quarantine_dir(c.root()).join("dark.png");
    let orig = c.root().join("dark.png");
    {
        let conn = open_root(c.root()).unwrap();
        conn.execute(
            "INSERT INTO file_operations (image_id, operation_type, source_path, destination_path, status, checksum, created_at)
             VALUES (?1, 'restore', ?2, ?3, 'pending', ?4, 0)",
            rusqlite::params![dark, q.to_string_lossy(), orig.to_string_lossy(), crate::import::dedup_hash(&q).unwrap()],
        )
        .unwrap();
        fs::rename(&q, &orig).unwrap(); // moved back, not yet recorded
        crate::files::reconcile(&conn).unwrap();
    }
    let item = c.item("dark.png");
    assert_eq!((item.status.as_deref(), item.current_path), (Some("rejected"), None));
    assert!(integrity_for_root(c.root()).unwrap().missing_files.is_empty());
}

#[test]
fn reconcile_on_open_leaves_a_running_operation_alone() {
    let c = Collection::new("p3-live");
    analyse(&c);
    let dark = c.id("dark.png");
    let src = c.root().join("dark.png");
    let conn = open_root(c.root()).unwrap();
    conn.execute(
        "INSERT INTO file_operations (image_id, operation_type, source_path, destination_path, status, checksum, created_at)
         VALUES (?1, 'quarantine', ?2, ?3, 'pending', ?4, 0)",
        rusqlite::params![dark, src.to_string_lossy(), quarantine_dir(c.root()).join("dark.png").to_string_lossy(), crate::import::dedup_hash(&src).unwrap()],
    )
    .unwrap();
    // While an operation holds the lock (as a live quarantine does), opening does not settle it.
    crate::files::with_operation_lock_for_tests(|| {
        assert_eq!(crate::files::reconcile_if_idle(&conn).unwrap(), 0);
    });
    let status: String = conn
        .query_row("SELECT status FROM file_operations WHERE image_id = ?1", [dark], |r| r.get(0))
        .unwrap();
    assert_eq!(status, "pending");
    crate::files::reconcile(&conn).unwrap();
    assert_eq!(
        operations_for_root(c.root(), 1).unwrap()[0].failure.as_deref(),
        Some("interrupted")
    );
}

#[test]
fn an_image_without_import_hash_still_round_trips_verified() {
    let c = Collection::new("p3-nohash");
    analyse(&c);
    let dark = c.id("dark.png");
    let bytes = fs::read(c.root().join("dark.png")).unwrap();
    open_root(c.root())
        .unwrap()
        .execute("UPDATE images SET file_hash = NULL WHERE id = ?1", [dark])
        .unwrap();
    decide_for_root(c.root(), &[dark], Decision::Reject).unwrap();
    assert!(quarantine_for_root(c.root(), &[dark]).unwrap()[0].ok());
    assert!(restore_for_root(c.root(), &[dark]).unwrap()[0].ok());
    assert_eq!(fs::read(c.root().join("dark.png")).unwrap(), bytes);
}

#[test]
fn clearing_a_decision_on_an_unanalysed_image_returns_it_to_not_analysed() {
    let c = Collection::new("p3-clear-unanalysed");
    let bright = c.id("bright.png");
    decide_for_root(c.root(), &[bright], Decision::Reject).unwrap();
    assert_eq!(c.item("bright.png").status.as_deref(), Some("rejected"));
    decide_for_root(c.root(), &[bright], Decision::Clear).unwrap();
    assert_eq!(c.item("bright.png").status, None);
    assert_eq!(quarantine_for_root(c.root(), &[bright]).unwrap()[0].failure, Some("not-rejected"));
}
