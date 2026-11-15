//! Phase 4 exit criterion (DEVELOPMENT_ROADMAP.md): given a config, the exact selected
//! sequence; deterministic for identical inputs (PROCESSING_PIPELINE.md §13). Real files:
//! import → capture times → (clean) → sample.

use std::fs;

use super::*;
use crate::clean::{Decision, decide_for_root, quarantine_for_root, review_for_root};
use crate::import::metadata::fixture::{ExifSpec, TempDir, write_jpeg};
use crate::import::{backfill_for_root, import_folder};

/// 25 hourly shots on 2024-01-15 from 00:00 to 24:00 (`h00.jpg` … `h24.jpg`), plus one
/// photograph without any capture time (`nodate.jpg`, mtime cleared).
fn hourly() -> TempDir {
    let dir = TempDir::new("p4-hourly");
    for h in 0..=24u32 {
        let (day, hour) = if h == 24 { (16, 0) } else { (15, h) };
        let stamp: &'static str =
            Box::leak(format!("2024:01:{day:02} {hour:02}:00:00").into_boxed_str());
        write_jpeg(&dir.path().join(format!("h{h:02}.jpg")), &ExifSpec::original(stamp));
    }
    write_jpeg(&dir.path().join("nodate.jpg"), &ExifSpec::none());
    import_folder(dir.path()).unwrap();
    crate::import::open_root(dir.path())
        .unwrap()
        .execute("UPDATE images SET modified_at = 0 WHERE relative_path = 'nodate.jpg'", [])
        .unwrap();
    backfill_for_root(dir.path()).unwrap();
    dir
}

fn cfg(method: Method) -> Config {
    Config { method, from: None, to: None, include_flagged: true }
}

fn names(items: &[SampledItem]) -> Vec<String> {
    items.iter().map(|i| i.relative_path.clone()).collect()
}

fn id_of(dir: &TempDir, name: &str) -> i64 {
    review_for_root(dir.path(), &[])
        .unwrap()
        .into_iter()
        .find(|i| i.relative_path == name)
        .unwrap()
        .id
}

#[test]
fn each_method_produces_the_exact_expected_sequence() {
    let d = hourly();
    let every_6h = preview_for_root(
        d.path(),
        &cfg(Method::FixedInterval {
            interval_seconds: 6.0 * 3600.0,
            policy: Policy::Closest,
            tolerance_seconds: None,
        }),
    )
    .unwrap();
    assert_eq!(names(&every_6h.items), ["h00.jpg", "h06.jpg", "h12.jpg", "h18.jpg", "h24.jpg"]);
    assert_eq!(every_6h.items[1].target_datetime.as_deref(), Some("2024-01-15T06:00:00.000Z"));
    assert_eq!(every_6h.items[1].actual_datetime, "2024-01-15T06:00:00.000Z");
    assert_eq!(every_6h.input_count, 25);
    assert_eq!(every_6h.excluded.no_timestamp, 1, "nodate.jpg left out and counted");
    assert_eq!(every_6h.effective_interval_seconds, Some(6.0 * 3600.0));
    assert!((every_6h.reduction - 0.8).abs() < 1e-9);

    let nth = preview_for_root(d.path(), &cfg(Method::EveryNth { n: 10, offset: 2 })).unwrap();
    assert_eq!(names(&nth.items), ["h02.jpg", "h12.jpg", "h22.jpg"]);

    let uni = preview_for_root(d.path(), &cfg(Method::Uniform { count: 3 })).unwrap();
    assert_eq!(names(&uni.items), ["h00.jpg", "h12.jpg", "h24.jpg"]);

    // A period restricts the input; `to` is exclusive.
    let mut morning = cfg(Method::EveryNth { n: 1, offset: 0 });
    morning.from = Some("2024-01-15T06".into());
    morning.to = Some("2024-01-15T09".into());
    let p = preview_for_root(d.path(), &morning).unwrap();
    assert_eq!(names(&p.items), ["h06.jpg", "h07.jpg", "h08.jpg"]);
    assert_eq!(p.excluded.out_of_period, 22);
}

#[test]
fn identical_inputs_give_an_identical_saved_sequence() {
    let d = hourly();
    let c = cfg(Method::FixedInterval {
        interval_seconds: 5400.0,
        policy: Policy::FirstAfter,
        tolerance_seconds: None,
    });
    let a = create_run_for_root(d.path(), &c).unwrap();
    let b = create_run_for_root(d.path(), &c).unwrap();
    assert_ne!(a.id, b.id);
    assert_eq!(
        run_items_for_root(d.path(), a.id).unwrap(),
        run_items_for_root(d.path(), b.id).unwrap()
    );
    assert_eq!(
        run_items_for_root(d.path(), a.id).unwrap(),
        preview_for_root(d.path(), &c).unwrap().items
    );
    let v = verify_run_for_root(d.path(), a.id).unwrap();
    assert!(v.inputs_unchanged && v.algorithm_unchanged && v.selection_identical);
    let runs = runs_for_root(d.path()).unwrap();
    assert_eq!(runs[0].id, b.id, "newest first");
    assert_eq!(runs[0].config, c, "configuration stored");
    assert_eq!(runs[0].method, "fixed-interval");
}

#[test]
fn a_changed_input_is_detected_and_the_stored_run_is_kept() {
    let d = hourly();
    let c = cfg(Method::EveryNth { n: 6, offset: 0 });
    let run = create_run_for_root(d.path(), &c).unwrap();
    let before = run_items_for_root(d.path(), run.id).unwrap();
    // A new photograph appears in the period.
    write_jpeg(&d.path().join("extra.jpg"), &ExifSpec::original("2024:01:15 00:30:00"));
    import_folder(d.path()).unwrap();
    backfill_for_root(d.path()).unwrap();
    let v = verify_run_for_root(d.path(), run.id).unwrap();
    assert!(!v.inputs_unchanged);
    assert!(!v.selection_identical, "every 6th now lands on different photographs");
    assert_eq!(run_items_for_root(d.path(), run.id).unwrap(), before, "the saved run is unchanged");
}

#[test]
fn rejected_quarantined_and_optionally_flagged_photos_are_never_selected() {
    let d = hourly();
    let (h03, h04, h05) = (id_of(&d, "h03.jpg"), id_of(&d, "h04.jpg"), id_of(&d, "h05.jpg"));
    decide_for_root(d.path(), &[h03, h04], Decision::Reject).unwrap();
    assert!(quarantine_for_root(d.path(), &[h04]).unwrap()[0].ok());
    // Mark h05 as flagged, as the classifier would.
    crate::import::open_root(d.path())
        .unwrap()
        .execute(
            "INSERT INTO image_status (image_id, status, reason, user_override, updated_at) VALUES (?1, 'flagged', NULL, 0, 0)",
            [h05],
        )
        .unwrap();

    let all = preview_for_root(d.path(), &cfg(Method::EveryNth { n: 1, offset: 0 })).unwrap();
    let picked = names(&all.items);
    assert!(!picked.contains(&"h03.jpg".to_string()) && !picked.contains(&"h04.jpg".to_string()));
    assert!(picked.contains(&"h05.jpg".to_string()), "flagged included by default");
    assert_eq!((all.excluded.rejected, all.excluded.quarantined), (1, 1));

    let mut strict = cfg(Method::EveryNth { n: 1, offset: 0 });
    strict.include_flagged = false;
    let p = preview_for_root(d.path(), &strict).unwrap();
    assert!(!names(&p.items).contains(&"h05.jpg".to_string()));
    assert_eq!(p.excluded.flagged, 1);
    assert_eq!(p.input_count, 25 - 3);
}

#[test]
fn ties_and_gaps_on_real_files_and_rejection_is_detected_by_verify() {
    let d = hourly();
    let every_6h_closest = cfg(Method::FixedInterval {
        interval_seconds: 6.0 * 3600.0,
        policy: Policy::Closest,
        tolerance_seconds: None,
    });
    let run = create_run_for_root(d.path(), &every_6h_closest).unwrap();
    // Rejecting a selected photograph changes the input; verify says so.
    decide_for_root(d.path(), &[id_of(&d, "h06.jpg")], Decision::Reject).unwrap();
    let v = verify_run_for_root(d.path(), run.id).unwrap();
    assert!(!v.inputs_unchanged && !v.selection_identical);
    // 06:00 now ties between 05:00 and 07:00: the earlier one wins.
    let p = preview_for_root(d.path(), &every_6h_closest).unwrap();
    assert_eq!(names(&p.items), ["h00.jpg", "h05.jpg", "h12.jpg", "h18.jpg", "h24.jpg"]);
    // With no slack, the 06:00 step is reported, not filled.
    let exact = cfg(Method::FixedInterval {
        interval_seconds: 6.0 * 3600.0,
        policy: Policy::FirstAfter,
        tolerance_seconds: Some(0.0),
    });
    let p = preview_for_root(d.path(), &exact).unwrap();
    assert_eq!(names(&p.items), ["h00.jpg", "h12.jpg", "h18.jpg", "h24.jpg"]);
    assert_eq!(p.unmatched_targets, ["2024-01-15T06:00:00.000Z"]);
}

#[test]
fn previews_are_bounded_and_runs_are_paged() {
    let d = hourly();
    let all = cfg(Method::EveryNth { n: 1, offset: 0 });
    let p = preview_for_root(d.path(), &all).unwrap();
    assert_eq!(p.selected_count, 25);
    assert_eq!(p.day_counts.iter().map(|c| c.count).sum::<usize>(), 25);
    assert_eq!(p.day_counts[0].day, "2024-01-15");
    let run = create_run_for_root(d.path(), &all).unwrap();
    let page = run_page_for_root(d.path(), run.id, 20, 10).unwrap();
    assert_eq!((page.total, page.items.len()), (25, 5));
    assert_eq!(page.items[0].sequence_index, 20);
    assert_eq!(page.day_counts.len(), 2);
}

#[test]
fn invalid_configurations_and_unknown_runs_are_typed_errors() {
    let d = hourly();
    let zero = cfg(Method::EveryNth { n: 0, offset: 0 });
    assert!(matches!(
        preview_for_root(d.path(), &zero),
        Err(crate::Error::InvalidSampling { reason: "non-positive" })
    ));
    let dense = cfg(Method::FixedInterval {
        interval_seconds: 0.001,
        policy: Policy::Closest,
        tolerance_seconds: None,
    });
    assert!(matches!(
        create_run_for_root(d.path(), &dense),
        Err(crate::Error::InvalidSampling { reason: "too-many-targets" })
    ));
    assert!(runs_for_root(d.path()).unwrap().is_empty(), "nothing saved on error");
    assert!(matches!(run_items_for_root(d.path(), 42), Err(crate::Error::UnknownRun { id: 42 })));
    assert!(matches!(verify_run_for_root(d.path(), 42), Err(crate::Error::UnknownRun { id: 42 })));
}

#[test]
fn an_empty_collection_samples_to_an_empty_sequence() {
    let d = TempDir::new("p4-empty");
    fs::create_dir_all(d.path()).unwrap();
    import_folder(d.path()).unwrap();
    let p = preview_for_root(d.path(), &cfg(Method::Uniform { count: 10 })).unwrap();
    assert_eq!((p.input_count, p.selected_count, p.reduction), (0, 0, 0.0));
    assert_eq!(p.effective_interval_seconds, None);
}
