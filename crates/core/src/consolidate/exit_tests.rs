//! Phase 6 exit criterion (DEVELOPMENT_ROADMAP.md): a move/copy is reversible; originals
//! remain byte-identical and their path of record in SQLite never points at a mutated file.
//! Full path on real files: import → consolidate → (edit) → verify → undo.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::*;
use crate::clean::{self, Decision};
use crate::import::metadata::fixture::TempDir;
use crate::import::{import_folder, integrity_for_root};

/// Encode/consolidation tests share the process-wide run lock; run them one at a time so
/// none sees another's `busy`.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

struct Fixture {
    collection: TempDir,
    work: TempDir,
}

/// `157gopro/G001.JPG`, `158gopro/G001.JPG`, `158gopro/G002.JPG` and `159gopro/C.JPG`,
/// a byte-identical copy of `157gopro/G001.JPG`. Capture times (mtimes) run
/// 158/G002 → 157/G001 → 159/C → 158/G001.
impl Fixture {
    fn new(tag: &str) -> Self {
        let collection = TempDir::new(&format!("p6-{tag}"));
        let r = collection.path();
        let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        for (rel, bytes, minutes) in [
            ("157gopro/G001.JPG", &b"photo one"[..], 1),
            ("158gopro/G001.JPG", b"photo two", 3),
            ("158gopro/G002.JPG", b"photo three", 0),
            ("159gopro/C.JPG", b"photo one", 2),
        ] {
            let p = r.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, bytes).unwrap();
            let f = fs::File::options().write(true).open(&p).unwrap();
            f.set_modified(t0 + Duration::from_secs(60 * minutes)).unwrap();
        }
        import_folder(r).unwrap();
        crate::import::backfill_for_root(r).unwrap();
        Self { collection, work: TempDir::new(&format!("p6-{tag}-work")) }
    }

    fn root(&self) -> &Path {
        self.collection.path()
    }

    fn dest(&self) -> PathBuf {
        self.work.path().join("working")
    }

    fn cfg(&self, operation: Operation, naming: Naming) -> Config {
        Config { destination: self.dest(), operation, naming, sampling_run: None }
    }

    fn id(&self, rel: &str) -> i64 {
        let conn = open_root(self.root()).unwrap();
        conn.query_row(
            "SELECT id FROM images WHERE replace(relative_path, '\\', '/') = ?1",
            [rel],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn current(&self, rel: &str) -> Option<String> {
        store::current_path(&open_root(self.root()).unwrap(), self.id(rel)).unwrap()
    }

    fn run(&self, cfg: &Config) -> RunRecord {
        let p = preview(self.root(), cfg).unwrap();
        execute(self.root(), cfg, &p.fingerprint, 0, |_| {}).unwrap()
    }

    fn originals_ok(&self) {
        let check = verify_originals(self.root(), |_| {}).unwrap();
        assert_eq!((check.ok, check.changed, check.missing), (4, 0, 0), "{check:?}");
    }
}

/// Every file under `dir`, relative and `/`-separated, sorted.
fn files_under(dir: &Path) -> Vec<String> {
    fn walk(base: &Path, d: &Path, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(d) else { return };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                out.push(p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn a_copy_is_planned_placed_verified_and_undone() {
    let _s = serial();
    let f = Fixture::new("copy");
    let cfg = f.cfg(Operation::Copy, Naming::Original);
    let p = preview(f.root(), &cfg).unwrap();
    assert_eq!(p.input_count, 4);
    assert_eq!((p.counts.planned, p.counts.duplicates, p.renamed), (3, 1, 1));
    assert_eq!(
        p.counts.bytes,
        ("photo one".len() + "photo two".len() + "photo three".len()) as u64
    );
    assert!(!f.dest().exists(), "a dry run writes nothing");
    // Chronological: 158/G002 → 157/G001 → (159/C duplicate) → 158/G001 renamed.
    let targets: Vec<_> = p.items.iter().map(|i| i.target.as_deref()).collect();
    assert_eq!(targets, [Some("G002.JPG"), Some("G001.JPG"), None, Some("G001_2.JPG")]);

    let mut seen = Vec::new();
    let rec = execute(f.root(), &cfg, &p.fingerprint, 0, |x| seen.push(x)).unwrap();
    assert_eq!((rec.status.as_str(), rec.placed, rec.failed), ("completed", 3, 0));
    assert_eq!(seen.last().map(|x| (x.done, x.total)), Some((3, 3)));
    assert_eq!(files_under(&f.dest()), ["G001.JPG", "G001_2.JPG", "G002.JPG"]);
    assert_eq!(fs::read(f.dest().join("G001_2.JPG")).unwrap(), b"photo two");
    assert_eq!(f.current("157gopro/G001.JPG"), None, "a copy leaves the original in place");
    f.originals_ok();
    let check = verify_run(f.root(), rec.id).unwrap();
    assert_eq!((check.checked, check.ok), (3, 3));

    let undone = undo(f.root(), rec.id, 0, |_| {}).unwrap();
    assert_eq!(
        (undone.undo_status.as_deref(), undone.undone, undone.undo_failed),
        (Some("undone"), 3, 0)
    );
    assert!(files_under(&f.dest()).is_empty());
    f.originals_ok();
    assert!(matches!(
        undo(f.root(), rec.id, 0, |_| {}),
        Err(Error::Consolidation { reason: "already-undone" })
    ));
}

#[test]
fn a_move_keeps_the_path_of_record_on_the_verified_file_and_undoes() {
    let _s = serial();
    let f = Fixture::new("move");
    let rec = f.run(&f.cfg(Operation::Move, Naming::Preserve));
    assert_eq!((rec.status.as_str(), rec.placed), ("completed", 3));
    // The duplicate stays where it was; the three others are in the working folder.
    assert_eq!(
        files_under(f.root())
            .into_iter()
            .filter(|p| !p.starts_with(".timelapse"))
            .collect::<Vec<_>>(),
        ["159gopro/C.JPG"]
    );
    assert_eq!(
        files_under(&f.dest()),
        ["157gopro/G001.JPG", "158gopro/G001.JPG", "158gopro/G002.JPG"]
    );
    let moved = f.current("157gopro/G001.JPG").unwrap();
    assert_eq!(PathBuf::from(&moved), f.dest().join("157gopro").join("G001.JPG"));
    // The location of record holds the imported bytes; a rescan adds nothing and misses nothing.
    f.originals_ok();
    assert_eq!(import_folder(f.root()).unwrap().new_rows, 0);
    assert!(integrity_for_root(f.root()).unwrap().missing_files.is_empty());

    let undone = undo(f.root(), rec.id, 0, |_| {}).unwrap();
    assert_eq!(undone.undo_status.as_deref(), Some("undone"));
    assert_eq!(f.current("157gopro/G001.JPG"), None, "back at its original path");
    assert_eq!(fs::read(f.root().join("157gopro/G001.JPG")).unwrap(), b"photo one");
    assert!(files_under(&f.dest()).is_empty(), "the folders it made are gone");
    f.originals_ok();
}

#[test]
fn editing_a_copy_never_reaches_the_original_and_survives_undo() {
    let _s = serial();
    let f = Fixture::new("edit");
    let rec = f.run(&f.cfg(Operation::Copy, Naming::FolderPrefix));
    let copy = f.dest().join("157gopro_G001.JPG");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let a = fs::metadata(f.root().join("157gopro/G001.JPG")).unwrap();
        let b = fs::metadata(&copy).unwrap();
        assert_ne!((a.dev(), a.ino()), (b.dev(), b.ino()), "never a hard link");
    }
    fs::write(&copy, b"retouched in another app").unwrap();
    f.originals_ok();
    let check = verify_run(f.root(), rec.id).unwrap();
    assert_eq!((check.ok, check.modified), (2, 1));
    assert_eq!(check.problems[0].state, "modified");

    let undone = undo(f.root(), rec.id, 0, |_| {}).unwrap();
    assert_eq!(
        (undone.undo_status.as_deref(), undone.undone, undone.undo_failed),
        (Some("partially-undone"), 2, 1)
    );
    assert_eq!(
        fs::read(&copy).unwrap(),
        b"retouched in another app",
        "a modified copy is never removed"
    );
    // Repeating the undo only retries what is left.
    let again = undo(f.root(), rec.id, 0, |_| {}).unwrap();
    assert_eq!(again.undone, 2);
    assert!(copy.exists());
}

#[test]
fn nothing_at_the_destination_is_ever_overwritten() {
    let _s = serial();
    let f = Fixture::new("noclobber");
    fs::create_dir_all(f.dest()).unwrap();
    fs::write(f.dest().join("G002.JPG"), b"photo three").unwrap(); // identical
    fs::write(f.dest().join("G001.JPG"), b"someone else's file").unwrap(); // different
    let cfg = f.cfg(Operation::Copy, Naming::Original);
    let p = preview(f.root(), &cfg).unwrap();
    assert_eq!((p.counts.planned, p.counts.already_present, p.counts.duplicates), (2, 1, 1));
    let targets: Vec<_> = p.items.iter().map(|i| (i.target.as_deref(), i.action)).collect();
    assert_eq!(targets[0], (Some("G002.JPG"), Action::AlreadyPresent));
    assert_eq!(targets[1], (Some("G001_2.JPG"), Action::Place));
    assert_eq!(targets[3], (Some("G001_3.JPG"), Action::Place));
    let rec = execute(f.root(), &cfg, &p.fingerprint, 0, |_| {}).unwrap();
    assert_eq!(rec.placed, 2);
    undo(f.root(), rec.id, 0, |_| {}).unwrap();
    assert_eq!(fs::read(f.dest().join("G001.JPG")).unwrap(), b"someone else's file");
    assert_eq!(fs::read(f.dest().join("G002.JPG")).unwrap(), b"photo three", "not ours to undo");
    assert_eq!(files_under(&f.dest()), ["G001.JPG", "G002.JPG"]);
}

#[test]
fn a_changed_plan_a_bad_destination_and_an_empty_plan_are_refused() {
    let _s = serial();
    let f = Fixture::new("refuse");
    let cfg = f.cfg(Operation::Copy, Naming::Original);
    let p = preview(f.root(), &cfg).unwrap();
    fs::create_dir_all(f.dest()).unwrap();
    fs::write(f.dest().join("G002.JPG"), b"arrived after the dry run").unwrap();
    assert!(matches!(
        execute(f.root(), &cfg, &p.fingerprint, 0, |_| {}),
        Err(Error::Consolidation { reason: "plan-changed" })
    ));
    assert_eq!(fs::read(f.dest().join("G002.JPG")).unwrap(), b"arrived after the dry run");

    for (dest, reason) in [
        (f.root().join("inside"), "destination-overlaps-collection"),
        (f.root().to_path_buf(), "destination-overlaps-collection"),
        (f.root().parent().unwrap().to_path_buf(), "destination-overlaps-collection"),
        (PathBuf::from("relative/folder"), "destination-invalid"),
        (f.root().join("157gopro/G001.JPG"), "destination-invalid"),
        (f.dest().join("G002.JPG"), "destination-invalid"),
    ] {
        let cfg = Config { destination: dest.clone(), ..cfg.clone() };
        match preview(f.root(), &cfg) {
            Err(Error::Consolidation { reason: r }) => assert_eq!(r, reason, "{dest:?}"),
            other => panic!("{dest:?}: {other:?}"),
        }
    }

    // Everything already there: nothing to do.
    let full = TempDir::new("p6-refuse-full");
    let cfg = Config { destination: full.path().to_path_buf(), ..cfg };
    let rec = f.run(&cfg);
    assert_eq!(rec.placed, 3);
    let p = preview(f.root(), &cfg).unwrap();
    assert_eq!((p.counts.planned, p.counts.already_present), (0, 3));
    assert!(matches!(
        execute(f.root(), &cfg, &p.fingerprint, 0, |_| {}),
        Err(Error::Consolidation { reason: "nothing-to-do" })
    ));
}

#[test]
fn a_selection_is_numbered_in_its_sequence_order() {
    let _s = serial();
    let f = Fixture::new("selection");
    let run = crate::sampling::create_run_for_root(
        f.root(),
        &crate::sampling::Config {
            method: crate::sampling::select::Method::EveryNth { n: 1, offset: 0 },
            from: None,
            to: None,
            include_flagged: true,
        },
    )
    .unwrap();
    let cfg = Config { sampling_run: Some(run.id), ..f.cfg(Operation::Copy, Naming::Sequence) };
    let rec = f.run(&cfg);
    assert_eq!(rec.placed, 3);
    assert_eq!(files_under(&f.dest()), ["000001.jpg", "000002.jpg", "000003.jpg"]);
    // 158/G002 is first in time, 158/G001 last (the duplicate 159/C is skipped).
    assert_eq!(fs::read(f.dest().join("000001.jpg")).unwrap(), b"photo three");
    assert_eq!(fs::read(f.dest().join("000003.jpg")).unwrap(), b"photo two");

    let gone = Config { sampling_run: Some(9999), ..cfg };
    assert!(matches!(preview(f.root(), &gone), Err(Error::UnknownRun { id: 9999 })));
}

#[test]
fn rejected_and_quarantined_photos_are_left_out_and_a_moved_photo_can_be_quarantined() {
    let _s = serial();
    let f = Fixture::new("quarantine");
    let g2 = f.id("158gopro/G002.JPG");
    clean::decide_for_root(f.root(), &[g2], Decision::Reject).unwrap();
    let p = preview(f.root(), &f.cfg(Operation::Copy, Naming::Original)).unwrap();
    assert_eq!((p.input_count, p.excluded.rejected), (3, 1));
    clean::decide_for_root(f.root(), &[g2], Decision::Clear).unwrap();

    let rec = f.run(&f.cfg(Operation::Move, Naming::Preserve));
    let moved = f.dest().join("158gopro").join("G002.JPG");
    assert_eq!(f.current("158gopro/G002.JPG"), Some(moved.to_string_lossy().into_owned()));
    // Rejected after the move: quarantine takes it from the working folder…
    clean::decide_for_root(f.root(), &[g2], Decision::Reject).unwrap();
    assert!(clean::quarantine_for_root(f.root(), &[g2]).unwrap()[0].ok());
    assert!(!moved.exists());
    // …so the run's undo cannot move it back, and says why.
    let undone = undo(f.root(), rec.id, 0, |_| {}).unwrap();
    assert_eq!((undone.undo_status.as_deref(), undone.undo_failed), (Some("partially-undone"), 1));
    // Restore returns it to where the quarantine took it from: the working folder.
    assert!(clean::restore_for_root(f.root(), &[g2]).unwrap()[0].ok());
    assert_eq!(fs::read(&moved).unwrap(), b"photo three");
    assert_eq!(f.current("158gopro/G002.JPG"), Some(moved.to_string_lossy().into_owned()));
    // Now the undo can finish.
    let undone = undo(f.root(), rec.id, 0, |_| {}).unwrap();
    assert_eq!(undone.undo_status.as_deref(), Some("undone"));
    assert_eq!(f.current("158gopro/G002.JPG"), None);
    f.originals_ok();
}

#[test]
fn verify_originals_reports_a_changed_and_a_missing_original() {
    let _s = serial();
    let f = Fixture::new("tamper");
    fs::write(f.root().join("157gopro/G001.JPG"), b"overwritten by someone").unwrap();
    fs::remove_file(f.root().join("158gopro/G002.JPG")).unwrap();
    let check = verify_originals(f.root(), |_| {}).unwrap();
    assert_eq!((check.total, check.ok, check.changed, check.missing), (4, 2, 1, 1));
    // A changed original is never consolidated (its bytes are not the imported ones).
    let rec = f.run(&f.cfg(Operation::Copy, Naming::Original));
    // The missing one is excluded; 159/C is still a duplicate of the (changed) 157/G001 by
    // its import hash; 157/G001 itself fails, its bytes no longer being the imported ones.
    assert_eq!((rec.plan.planned, rec.plan.duplicates), (2, 1));
    assert_eq!((rec.status.as_str(), rec.placed, rec.failed), ("partial", 1, 1));
}

#[test]
fn cancelling_stops_between_photographs() {
    let _s = serial();
    let f = Fixture::new("cancel");
    let cfg = f.cfg(Operation::Copy, Naming::Original);
    let p = preview(f.root(), &cfg).unwrap();
    let token = 7_000_001;
    let rec = execute(f.root(), &cfg, &p.fingerprint, token, |x| {
        if x.done == 1 {
            cancel(token);
        }
    })
    .unwrap();
    cancel(0);
    assert_eq!((rec.status.as_str(), rec.placed), ("cancelled", 1));
    assert_eq!(files_under(&f.dest()).len(), 1);
    assert_eq!(undo(f.root(), rec.id, 0, |_| {}).unwrap().undo_status.as_deref(), Some("undone"));
}

/// Pending rows a crash could leave, settled from the disk state when the project opens.
#[test]
fn every_interruption_state_is_reconciled() {
    let _s = serial();
    let f = Fixture::new("reconcile");
    let conn = open_root(f.root()).unwrap();
    let src = f.root().join("157gopro/G001.JPG");
    let id = f.id("157gopro/G001.JPG");
    let hash = store::file_hash(&conn, id).unwrap().unwrap();
    fs::create_dir_all(f.dest()).unwrap();
    let pending = |kind: &str, s: &Path, d: &Path| {
        conn.execute(
            "INSERT INTO file_operations (image_id, operation_type, source_path, destination_path,
                                          status, checksum, created_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5, 0)",
            rusqlite::params![id, kind, s.to_string_lossy(), d.to_string_lossy(), hash],
        )
        .unwrap();
        conn.last_insert_rowid()
    };
    let outcome = |op: i64| -> (String, Option<String>) {
        conn.query_row("SELECT status, failure FROM file_operations WHERE id = ?1", [op], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap()
    };
    let done_copy = f.dest().join("done.JPG");
    fs::write(&done_copy, b"photo one").unwrap();
    let a = pending("copy", &src, &done_copy);
    let partial = f.dest().join("partial.JPG");
    fs::write(&partial, b"photo").unwrap();
    let b = pending("copy", &src, &partial);
    let c = pending("copy", &src, &f.dest().join("never.JPG"));
    let half_moved = f.dest().join("half-moved.JPG");
    fs::write(&half_moved, b"photo one").unwrap();
    let d = pending("move", &src, &half_moved);
    let e = pending("remove-copy", &f.dest().join("already-removed.JPG"), Path::new(""));
    let kept = f.dest().join("kept.JPG");
    fs::write(&kept, b"photo one").unwrap();
    let g = pending("remove-copy", &kept, Path::new(""));
    let odd = f.dest().join("odd.JPG");
    fs::write(&odd, b"something else entirely").unwrap();
    let h = pending("copy", &f.dest().join("no-source.JPG"), &odd);

    assert_eq!(crate::files::reconcile(&conn).unwrap(), 7);
    let ok = |s: &str| (s.to_owned(), None);
    let failed = |s: &str| ("failed".to_owned(), Some(s.to_owned()));
    assert_eq!(outcome(a), ok("completed"));
    assert!(done_copy.exists(), "a finished copy is kept");
    assert_eq!(outcome(b), failed("interrupted"));
    assert!(!partial.exists(), "a partial copy is removed");
    assert_eq!(outcome(c), failed("interrupted"));
    assert_eq!(outcome(d), failed("interrupted"));
    assert!(!half_moved.exists() && src.exists(), "a half move goes back to the original");
    assert_eq!(outcome(e), ok("completed"));
    assert_eq!(outcome(g), failed("interrupted"));
    assert!(kept.exists());
    assert_eq!(outcome(h), failed("inconsistent"));
    assert_eq!(fs::read(&odd).unwrap(), b"something else entirely", "nothing is touched");
    f.originals_ok();
}

#[test]
fn the_v6_migration_keeps_the_existing_journal() {
    let _s = serial();
    let f = Fixture::new("migrate");
    let conn = open_root(f.root()).unwrap();
    let id = f.id("157gopro/G001.JPG");
    clean::decide_for_root(f.root(), &[id], Decision::Reject).unwrap();
    assert!(clean::quarantine_for_root(f.root(), &[id]).unwrap()[0].ok());
    let ops = crate::files::list_operations(&conn, 10).unwrap();
    assert_eq!((ops[0].operation_type.as_str(), ops[0].job_id), ("quarantine", None));
    let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
    assert_eq!(v, crate::import::store::SCHEMA_VERSION);
}
