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
/// returning a report. No long-lived writer remains in the shell. It reads every byte of
/// the collection, so it runs on the blocking pool: the window never freezes.
#[tauri::command]
async fn import_folder(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::ImportReport, IpcError> {
    blocking(move || timelapse_core::import::import_folder(&root)).await
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
async fn integrity_check(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::store::IntegrityResult, IpcError> {
    blocking(move || timelapse_core::import::integrity_for_root(&root)).await
}

/// Phase 2: resolve and persist capture times for every imported image of `root` that
/// has none yet (EXIF → filename → filesystem). Reads originals only.
#[tauri::command]
async fn backfill_metadata(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::BackfillReport, IpcError> {
    blocking(move || timelapse_core::import::backfill_for_root(&root)).await
}

/// UX: what an already imported folder holds (inventory size and timeline counts), read-only.
#[tauri::command]
async fn project_summary(
    root: PathBuf,
) -> std::result::Result<timelapse_core::import::ProjectSummary, IpcError> {
    blocking(move || timelapse_core::import::project_summary(&root)).await
}

/// Phase 2: the chronological timeline of `root` (unresolved images last).
#[tauri::command]
fn get_timeline(
    root: PathBuf,
) -> std::result::Result<Vec<timelapse_core::import::store::TimelineEntry>, IpcError> {
    Ok(timelapse_core::import::timeline_for_root(&root)?)
}

/// Phase 2 (weeks: Phase 7): image counts per hour / day / week / month over an optional `[from, to)` period.
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
    token: u64,
    on_progress: tauri::ipc::Channel<clean::AnalyzeProgress>,
) -> std::result::Result<clean::AnalyzeReport, IpcError> {
    blocking(move || {
        clean::analyze_for_root(&root, token, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

/// Stop the analysis started with `token` after the chunk it is measuring; what was
/// measured is kept.
#[tauri::command]
fn cancel_analysis(token: u64) {
    clean::cancel_analysis(token);
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

/// Move rejected originals into `<root>/Timelapse quarantine/` (journaled, hash-verified).
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

// ---- Phase 5: FFmpeg video generation (ADR-017, ADR-021, ADR-022) ---------------------

use timelapse_core::export;

/// The bundled FFmpeg: `<resources>/resources/ffmpeg`. Debug builds honour
/// `TIMELAPSE_FFMPEG_DIR` so development on a machine without the bundle can still
/// encode; release builds only ever use the bundle (ADR-021).
/// `None` when the resource directory cannot be resolved: never fall back to a relative
/// path (the working directory is not ours).
fn ffmpeg_paths(app: &tauri::AppHandle) -> Option<export::FfmpegPaths> {
    use tauri::Manager as _;
    if cfg!(debug_assertions)
        && let Some(dir) = std::env::var_os("TIMELAPSE_FFMPEG_DIR")
    {
        return Some(export::FfmpegPaths::in_dir(std::path::Path::new(&dir)));
    }
    let dir = app.path().resource_dir().ok()?.join("resources").join("ffmpeg");
    Some(export::FfmpegPaths::in_dir(&dir))
}

/// Which tiers the bundled FFmpeg can encode (unavailable ones are disabled in the UI).
#[tauri::command]
async fn ffmpeg_info(app: tauri::AppHandle) -> std::result::Result<export::FfmpegInfo, IpcError> {
    let paths = ffmpeg_paths(&app);
    blocking(move || Ok(paths.map(|p| export::info(&p)).unwrap_or_default())).await
}

/// Encode saved sampling run `run` into an MP4 under `<root>/Timelapse exports/`.
#[tauri::command]
async fn start_export(
    app: tauri::AppHandle,
    root: PathBuf,
    run: i64,
    settings: export::ExportSettings,
    token: u64,
    on_progress: tauri::ipc::Channel<export::ExportProgress>,
) -> std::result::Result<export::ExportRecord, IpcError> {
    let paths =
        ffmpeg_paths(&app).ok_or(timelapse_core::Error::Export { reason: "ffmpeg-missing" })?;
    blocking(move || {
        export::export_run(&root, &paths, run, settings, token, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

/// Stop the export started with `token` (a token chosen by the UI per export).
#[tauri::command]
fn cancel_export(token: u64) {
    export::cancel(token);
}

#[tauri::command]
fn list_exports(root: PathBuf) -> std::result::Result<Vec<export::ExportRecord>, IpcError> {
    Ok(export::exports_for_root(&root)?)
}

// ---- Phase 6: consolidation (ADR-023) --------------------------------------------------

use timelapse_core::consolidate;

/// The dry run: targets, duplicates, collisions, bytes vs free space. Writes nothing.
#[tauri::command]
async fn preview_consolidation(
    root: PathBuf,
    config: consolidate::Config,
) -> std::result::Result<consolidate::Preview, IpcError> {
    blocking(move || consolidate::preview(&root, &config)).await
}

/// Carry out the plan the user confirmed (`fingerprint` from the dry run).
#[tauri::command]
async fn start_consolidation(
    root: PathBuf,
    config: consolidate::Config,
    fingerprint: String,
    token: u64,
    on_progress: tauri::ipc::Channel<consolidate::Progress>,
) -> std::result::Result<consolidate::RunRecord, IpcError> {
    blocking(move || {
        consolidate::execute(&root, &config, &fingerprint, token, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

/// Stop the consolidation or undo started with `token` after the current photograph.
#[tauri::command]
fn cancel_consolidation(token: u64) {
    consolidate::cancel(token);
}

/// Remove a run's unmodified copies, or move its originals back.
#[tauri::command]
async fn undo_consolidation(
    root: PathBuf,
    run: i64,
    token: u64,
    on_progress: tauri::ipc::Channel<consolidate::Progress>,
) -> std::result::Result<consolidate::RunRecord, IpcError> {
    blocking(move || {
        consolidate::undo(&root, run, token, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

#[tauri::command]
fn list_consolidations(
    root: PathBuf,
) -> std::result::Result<Vec<consolidate::RunRecord>, IpcError> {
    Ok(consolidate::runs_for_root(&root)?)
}

/// Re-hash every file a run placed against what it wrote.
#[tauri::command]
async fn verify_consolidation(
    root: PathBuf,
    run: i64,
) -> std::result::Result<consolidate::RunCheck, IpcError> {
    blocking(move || consolidate::verify_run(&root, run)).await
}

/// Re-hash every original at its location of record against its import hash.
#[tauri::command]
async fn verify_originals(
    root: PathBuf,
    on_progress: tauri::ipc::Channel<consolidate::Progress>,
) -> std::result::Result<consolidate::OriginalsCheck, IpcError> {
    blocking(move || {
        consolidate::verify_originals(&root, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

// ---- Phase 7: contact sheets, export record, timeline (ADR-024) ------------------------

use timelapse_core::contact_sheet;

/// Counts and the first page's cells for a source and layout. Writes nothing.
#[tauri::command]
async fn preview_contact_sheet(
    root: PathBuf,
    source: contact_sheet::Source,
    layout: contact_sheet::Layout,
) -> std::result::Result<contact_sheet::SheetPreview, IpcError> {
    blocking(move || contact_sheet::preview(&root, &source, &layout)).await
}

/// Generate a sheet into `<root>/Timelapse exports/contact-sheets/`; `labels` are translated by
/// the UI.
#[tauri::command]
async fn start_contact_sheet(
    root: PathBuf,
    source: contact_sheet::Source,
    layout: contact_sheet::Layout,
    labels: contact_sheet::Labels,
    expected: Option<usize>,
    token: u64,
    on_progress: tauri::ipc::Channel<contact_sheet::SheetProgress>,
) -> std::result::Result<contact_sheet::SheetRecord, IpcError> {
    blocking(move || {
        contact_sheet::generate(&root, &source, &layout, &labels, expected, token, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

/// Stop the sheet started with `token` before its next page.
#[tauri::command]
fn cancel_contact_sheet(token: u64) {
    contact_sheet::cancel(token);
}

#[tauri::command]
async fn list_contact_sheets(
    root: PathBuf,
) -> std::result::Result<Vec<contact_sheet::SheetRecord>, IpcError> {
    blocking(move || contact_sheet::sheets_for_root(&root)).await
}

/// The derived export record (ADR-015) as pretty JSON.
#[tauri::command]
async fn export_record(root: PathBuf, export: i64) -> std::result::Result<String, IpcError> {
    blocking(move || export::record::record_json(&root, export)).await
}

/// Save the export record next to the exports; returns its path.
#[tauri::command]
async fn write_export_record(root: PathBuf, export: i64) -> std::result::Result<PathBuf, IpcError> {
    blocking(move || export::record::write_record(&root, export)).await
}

/// One page of the photographs of `[from, to)` on the timeline.
#[tauri::command]
async fn timeline_period(
    root: PathBuf,
    from: String,
    to: String,
    offset: usize,
    limit: usize,
) -> std::result::Result<timelapse_core::import::store::TimelinePage, IpcError> {
    blocking(move || {
        timelapse_core::import::timeline_period_for_root(&root, &from, &to, offset, limit)
    })
    .await
}

/// Groups of photographs that share one capture time, the largest `limit` listed.
#[tauri::command]
async fn identical_timestamps(
    root: PathBuf,
    limit: usize,
) -> std::result::Result<timelapse_core::import::store::IdenticalReport, IpcError> {
    blocking(move || timelapse_core::import::identical_timestamps_for_root(&root, limit)).await
}

// ---- Phase 8: deflicker and stabilization (ADR-025) ------------------------------------

use timelapse_core::adjust;

/// Measure a saved selection's brightness and store a deflicker run (`null` = cancelled).
#[tauri::command]
async fn start_deflicker(
    root: PathBuf,
    run: i64,
    config: adjust::DeflickerConfig,
    token: u64,
    on_progress: tauri::ipc::Channel<adjust::AdjustProgress>,
) -> std::result::Result<Option<adjust::FlickerRun>, IpcError> {
    blocking(move || {
        adjust::deflicker_for_root(&root, run, &config, token, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

/// Measure a saved selection's motion and store a stabilization run (`null` = cancelled).
#[tauri::command]
async fn start_stabilization(
    root: PathBuf,
    run: i64,
    config: adjust::StabilizeConfig,
    token: u64,
    on_progress: tauri::ipc::Channel<adjust::AdjustProgress>,
) -> std::result::Result<Option<adjust::StabilizationRun>, IpcError> {
    blocking(move || {
        adjust::stabilize_for_root(&root, run, &config, token, |p| {
            let _ = on_progress.send(p);
        })
    })
    .await
}

/// Stop the analysis started with `token` after its current batch; nothing is stored.
#[tauri::command]
fn cancel_adjustment(token: u64) {
    adjust::cancel(token);
}

/// What Settings → Erase all data would remove for `root` (ADR-026).
#[tauri::command]
async fn erase_inventory(
    root: PathBuf,
) -> std::result::Result<timelapse_core::erase::Inventory, IpcError> {
    blocking(move || timelapse_core::erase::inventory(&root)).await
}

/// Erase what the user chose for `root`; refused (`busy`) while work runs on it.
#[tauri::command]
async fn erase_project_data(
    root: PathBuf,
    targets: timelapse_core::erase::Targets,
) -> std::result::Result<timelapse_core::erase::Report, IpcError> {
    blocking(move || timelapse_core::erase::erase(&root, targets)).await
}

#[tauri::command]
async fn list_adjustments(root: PathBuf) -> std::result::Result<adjust::Adjustments, IpcError> {
    blocking(move || adjust::adjustments_for_root(&root)).await
}

/// Chart points of a deflicker run (at most `max_points`, spikes kept).
#[tauri::command]
async fn flicker_series(
    root: PathBuf,
    run: i64,
    max_points: usize,
) -> std::result::Result<Vec<adjust::preview::FlickerPoint>, IpcError> {
    blocking(move || adjust::preview::flicker_series(&root, run, max_points)).await
}

/// Chart points of a stabilization run, and its crop.
#[tauri::command]
async fn stabilization_series(
    root: PathBuf,
    run: i64,
    max_points: usize,
) -> std::result::Result<adjust::preview::StabilizationSeries, IpcError> {
    blocking(move || adjust::preview::stabilization_series(&root, run, max_points)).await
}

/// Frame `index` of selection `run` as JPEG: as shot, or with the chosen runs applied.
#[tauri::command]
async fn adjusted_frame(
    root: PathBuf,
    run: i64,
    index: usize,
    flicker_run: Option<i64>,
    stabilization_run: Option<i64>,
    after: bool,
) -> std::result::Result<tauri::ipc::Response, IpcError> {
    blocking(move || {
        adjust::preview::frame(&root, run, index, flicker_run, stabilization_run, after)
    })
    .await
    .map(tauri::ipc::Response::new)
}

// ---- UX milestone: open and reveal what the app made (ADR-027) ------------------------
//
// The webview names a kind and an id, never a path; the core resolves it to the stored
// path and refuses anything that is not an existing file inside `<root>/Timelapse exports/`
// with the extension the app writes for that kind (the same check for open and reveal).
// The opener plugin is called from Rust only: the webview holds no opener permission.

use timelapse_core::outputs::{self, OutputKind};

/// Resolve `kind` `id` of `root` (checked by the core) and hand the path to `act`.
async fn with_output(
    app: tauri::AppHandle,
    root: PathBuf,
    kind: OutputKind,
    id: i64,
    act: fn(&tauri::AppHandle, &str) -> std::result::Result<(), tauri_plugin_opener::Error>,
) -> std::result::Result<(), IpcError> {
    blocking(move || {
        let path = outputs::resolve(&root, kind, id)?;
        act(&app, &path).map_err(|_| timelapse_core::Error::Output { reason: "open-failed" })
    })
    .await
}

/// Open an export, its record or a contact sheet in the default app.
#[tauri::command]
async fn open_output(
    app: tauri::AppHandle,
    root: PathBuf,
    kind: OutputKind,
    id: i64,
) -> std::result::Result<(), IpcError> {
    use tauri_plugin_opener::OpenerExt as _;
    with_output(app, root, kind, id, |app, path| app.opener().open_path(path, None::<&str>)).await
}

/// Show an export, its record or a contact sheet in Finder, selected.
#[tauri::command]
async fn reveal_output(
    app: tauri::AppHandle,
    root: PathBuf,
    kind: OutputKind,
    id: i64,
) -> std::result::Result<(), IpcError> {
    use tauri_plugin_opener::OpenerExt as _;
    with_output(app, root, kind, id, |app, path| app.opener().reveal_item_in_dir(path)).await
}

/// The fixed size in `tauri.conf.json` is only a fallback. On the first launch (no saved
/// window state yet) the window takes 80 % of the screen it opens on, never below its
/// minimum; afterwards the window-state plugin restores what the user chose.
#[cfg(desktop)]
fn fit_first_window(app: &tauri::App) {
    use tauri::Manager as _;
    let saved = app
        .path()
        .app_config_dir()
        .map(|d| d.join(tauri_plugin_window_state::DEFAULT_FILENAME).exists())
        .unwrap_or(true);
    if saved {
        return;
    }
    let Some(window) = app.get_webview_window("main") else { return };
    let Ok(Some(monitor)) = window.current_monitor() else { return };
    let scale = monitor.scale_factor();
    let width = (f64::from(monitor.size().width) / scale * 0.8).max(520.0);
    let height = (f64::from(monitor.size().height) / scale * 0.8).max(560.0);
    let _ = window.set_size(tauri::LogicalSize::new(width, height));
    let _ = window.center();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Dialog is used by the frontend for folder picking on every platform; the
    // window_state / single_instance plugins are desktop-only (GUIDELINES §8.2).
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    // Opener (ADR-027): used from Rust only, by `open_output` and `reveal_output`. No link
    // script is injected and no capability grants its commands to the webview.
    let builder =
        builder.plugin(tauri_plugin_opener::Builder::new().open_js_links_on_click(false).build());

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build()).plugin(
        tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }),
    );

    builder
        .setup(|app| {
            #[cfg(desktop)]
            fit_first_window(app);
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            import_folder,
            list_images,
            integrity_check,
            backfill_metadata,
            project_summary,
            get_timeline,
            timeline_distribution,
            detect_timeline_gaps,
            analyze_images,
            cancel_analysis,
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
            verify_sampling_run,
            ffmpeg_info,
            start_export,
            cancel_export,
            list_exports,
            preview_consolidation,
            start_consolidation,
            cancel_consolidation,
            undo_consolidation,
            list_consolidations,
            verify_consolidation,
            verify_originals,
            preview_contact_sheet,
            start_contact_sheet,
            cancel_contact_sheet,
            list_contact_sheets,
            export_record,
            write_export_record,
            timeline_period,
            identical_timestamps,
            start_deflicker,
            start_stabilization,
            cancel_adjustment,
            list_adjustments,
            flicker_series,
            stabilization_series,
            adjusted_frame,
            erase_inventory,
            erase_project_data,
            open_output,
            reveal_output
        ])
        .run(tauri::generate_context!())
        .expect("error while running Timelapse Studio");
}

#[cfg(test)]
mod tests {
    /// ADR-027: the opener is reached through `open_output` / `reveal_output` only; the
    /// webview holds no opener permission.
    #[test]
    fn the_webview_has_no_opener_permission() {
        let caps = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities")).unwrap();
        for entry in caps.flatten() {
            let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
            assert!(!text.contains("opener"), "{:?} grants an opener permission", entry.path());
        }
    }
}
