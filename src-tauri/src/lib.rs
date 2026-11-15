//! Thin Tauri shell: commands are adapters over `timelapse-core`.
//!
//! Errors cross IPC as `{ kind, args }`; the UI renders them through Fluent.
//! The shell owns no business logic and no database state — every command calls a
//! core function that opens a short-lived connection (ADR-007, ADR-011) and closes
//! it on return, so concurrent commands never hold locks on each other.

use serde::Serialize;
use std::path::PathBuf;

#[cfg(desktop)]
use tauri::Manager as _;

#[derive(Serialize)]
struct IpcError {
    kind: &'static str,
    args: serde_json::Value,
}

impl From<timelapse_core::Error> for IpcError {
    fn from(e: timelapse_core::Error) -> Self {
        Self { kind: e.kind(), args: e.args() }
    }
}

#[derive(Serialize)]
struct AppInfo {
    version: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo { version: env!("CARGO_PKG_VERSION") }
}

/// Phase 1 "import": the UI passes `root` (the folder the user picked). Core derives
/// the project name + database location and does scan → hash → dedup → upsert,
/// returning a report. No long-lived writer remains in the shell.
#[tauri::command]
fn import_folder(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::ImportReport, IpcError> {
    Ok(timelapse_core::import::import_folder(&root)?)
}

/// List the inventory rows for `root`.
#[tauri::command]
fn list_images(
    root: PathBuf,
) -> std::result::Result<Vec<timelapse_core::import::store::ImageRow>, IpcError> {
    Ok(timelapse_core::import::list_for_root(&root)?)
}

/// Integrity snapshot for `root`: missing files + duplicate groups.
#[tauri::command]
fn integrity_check(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::store::IntegrityResult, IpcError> {
    Ok(timelapse_core::import::integrity_for_root(&root)?)
}

/// Phase 2: resolve and persist capture times for every imported image of `root` that
/// has none yet (EXIF → filename → filesystem). Reads originals only.
#[tauri::command]
fn backfill_metadata(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::BackfillReport, IpcError> {
    Ok(timelapse_core::import::backfill_for_root(&root)?)
}

/// Phase 2: the chronological timeline of `root` (unresolved images last).
#[tauri::command]
fn get_timeline(
    root: PathBuf,
) -> std::result::Result<Vec<timelapse_core::import::store::TimelineEntry>, IpcError> {
    Ok(timelapse_core::import::timeline_for_root(&root)?)
}

/// Phase 2: image counts per hour / day / month over an optional `[from, to)` period.
#[tauri::command]
fn timeline_distribution(
    root: PathBuf,
    granularity: timelapse_core::import::store::Granularity,
    from: Option<String>,
    to: Option<String>,
) -> std::result::Result<Vec<timelapse_core::import::store::DistributionBucket>, IpcError> {
    Ok(timelapse_core::import::distribution_for_root(
        &root,
        granularity,
        from.as_deref(),
        to.as_deref(),
    )?)
}

/// Phase 2: unusually long intervals on the timeline; omitted options take core's defaults.
#[tauri::command]
fn detect_timeline_gaps(
    root: PathBuf,
    factor: Option<f64>,
    min_seconds: Option<i64>,
) -> std::result::Result<timelapse_core::import::store::GapReport, IpcError> {
    let opts = timelapse_core::import::store::GapOptions::from_parts(factor, min_seconds);
    Ok(timelapse_core::import::gaps_for_root(&root, opts)?)
}

// ---- Phase 3: brightness analysis & cleaning (ADR-020) -------------------------------
//
// The long-running commands (analysis, moves, thumbnails) are `async` and run their work on
// the blocking pool, so the window never freezes; the others are quick SQLite reads.

use timelapse_core::clean;

/// Run blocking core work off the main thread and flatten the join error.
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> timelapse_core::Result<T> + Send + 'static,
) -> std::result::Result<T, IpcError> {
    match tauri::async_runtime::spawn_blocking(f).await {
        Ok(r) => Ok(r?),
        Err(_) => Err(IpcError { kind: "generic", args: serde_json::json!({}) }),
    }
}

/// Measure every image not yet measured, reporting progress on `on_progress`, then classify.
#[tauri::command]
async fn analyze_images(
    root: PathBuf,
    on_progress: tauri::ipc::Channel<clean::AnalyzeProgress>,
) -> std::result::Result<clean::AnalyzeReport, IpcError> {
    blocking(move || {
        clean::analyze_for_root(&root, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

#[tauri::command]
fn get_clean_settings(root: PathBuf) -> std::result::Result<clean::Thresholds, IpcError> {
    Ok(clean::thresholds_for_root(&root)?)
}

/// Store thresholds and re-propose every undecided image.
#[tauri::command]
fn set_clean_settings(
    root: PathBuf,
    thresholds: clean::Thresholds,
) -> std::result::Result<clean::ThresholdsApplied, IpcError> {
    Ok(clean::set_thresholds_for_root(&root, thresholds)?)
}

#[tauri::command]
fn classify_images(root: PathBuf) -> std::result::Result<clean::StatusCounts, IpcError> {
    Ok(clean::classify_for_root(&root)?)
}

/// Images whose status is in `statuses` (every image when empty), chronological.
#[tauri::command]
fn get_review(
    root: PathBuf,
    statuses: Vec<String>,
) -> std::result::Result<Vec<clean::ReviewItem>, IpcError> {
    Ok(clean::review_for_root(&root, &statuses)?)
}

#[tauri::command]
fn set_decision(
    root: PathBuf,
    ids: Vec<i64>,
    decision: clean::Decision,
) -> std::result::Result<clean::StatusCounts, IpcError> {
    Ok(clean::decide_for_root(&root, &ids, decision)?)
}

/// Move rejected originals into `<root>/.timelapse/quarantine/` (journaled, hash-verified).
#[tauri::command]
async fn quarantine_images(
    root: PathBuf,
    ids: Vec<i64>,
) -> std::result::Result<Vec<timelapse_core::files::OpOutcome>, IpcError> {
    blocking(move || clean::quarantine_for_root(&root, &ids)).await
}

#[tauri::command]
async fn restore_images(
    root: PathBuf,
    ids: Vec<i64>,
) -> std::result::Result<Vec<timelapse_core::files::OpOutcome>, IpcError> {
    blocking(move || clean::restore_for_root(&root, &ids)).await
}

/// JPEG bytes of a review thumbnail, sent raw (an `ArrayBuffer` on the JS side).
#[tauri::command]
async fn get_thumbnail(
    root: PathBuf,
    id: i64,
    size: u32,
) -> std::result::Result<tauri::ipc::Response, IpcError> {
    blocking(move || clean::thumbnail_for_root(&root, id, size))
        .await
        .map(tauri::ipc::Response::new)
}

#[tauri::command]
fn list_file_operations(
    root: PathBuf,
    limit: usize,
) -> std::result::Result<Vec<timelapse_core::files::FileOperation>, IpcError> {
    Ok(clean::operations_for_root(&root, limit)?)
}

// ---- Phase 4: temporal sampling ------------------------------------------------------

use timelapse_core::sampling;

/// Run a sampling configuration against the current photographs without saving it.
#[tauri::command]
async fn preview_sampling(
    root: PathBuf,
    config: sampling::Config,
) -> std::result::Result<sampling::SamplingPreview, IpcError> {
    blocking(move || sampling::preview_for_root(&root, &config)).await
}

/// Compute and save a sampling run (configuration, input fingerprint, ordered selection).
#[tauri::command]
async fn create_sampling_run(
    root: PathBuf,
    config: sampling::Config,
) -> std::result::Result<sampling::RunSummary, IpcError> {
    blocking(move || sampling::create_run_for_root(&root, &config)).await
}

#[tauri::command]
fn list_sampling_runs(root: PathBuf) -> std::result::Result<Vec<sampling::RunSummary>, IpcError> {
    Ok(sampling::runs_for_root(&root)?)
}

/// One page of a saved run's sequence (at most 1000 items) and its day counts.
#[tauri::command]
fn get_sampling_items(
    root: PathBuf,
    run: i64,
    offset: usize,
    limit: usize,
) -> std::result::Result<sampling::RunPage, IpcError> {
    Ok(sampling::run_page_for_root(&root, run, offset, limit)?)
}

/// Re-run a saved configuration on the current photographs and compare.
#[tauri::command]
async fn verify_sampling_run(
    root: PathBuf,
    run: i64,
) -> std::result::Result<sampling::VerifyReport, IpcError> {
    blocking(move || sampling::verify_run_for_root(&root, run)).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Dialog is used by the frontend for folder picking on every platform; the
    // window_state / single_instance plugins are desktop-only (GUIDELINES §8.2).
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build()).plugin(
        tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }),
    );

    builder
        .invoke_handler(tauri::generate_handler![
            app_info,
            import_folder,
            list_images,
            integrity_check,
            backfill_metadata,
            get_timeline,
            timeline_distribution,
            detect_timeline_gaps,
            analyze_images,
            get_clean_settings,
            set_clean_settings,
            classify_images,
            get_review,
            set_decision,
            quarantine_images,
            restore_images,
            get_thumbnail,
            list_file_operations,
            preview_sampling,
            create_sampling_run,
            list_sampling_runs,
            get_sampling_items,
            verify_sampling_run
        ])
        .run(tauri::generate_context!())
        .expect("error while running Timelapse Studio");
}
