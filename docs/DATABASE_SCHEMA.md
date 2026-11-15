# Timelapse Studio

## Database Schema

**Version:** 0.1  
**Status:** Initial logical schema — implementation pending

SQLite is the authoritative source of project data.

The following schema is a starting point. The implementation agent must review constraints, indexes, relationships and migration strategy before creating the first migration.

---

## 1. Design principles

- Store metadata, not photographic binary data.

- Use stable internal IDs.

- Preserve original paths.

- Separate original inventory from derived selections.

- Record processing history.

- Use transactions for related database changes.

- Version the schema using migrations.

- Store timestamps in a consistent normalized representation, preferably UTC internally, while preserving original EXIF information and timezone uncertainty.

## 2. Proposed tables

### projects

| Column         | Purpose                        |
| -------------- | ------------------------------ |
| id             | Primary key                    |
| name           | Project name                   |
| project_path   | Project directory              |
| source_root    | Optional main source directory |
| created_at     | Creation timestamp             |
| updated_at     | Last update                    |
| schema_version | Database schema version        |
| settings_json  | Project-level settings         |

### images

One row per discovered source image.

| Column                 | Purpose                           |
| ---------------------- | --------------------------------- |
| id                     | Primary key                       |
| original_path          | Full original path                |
| relative_path          | Relative path when meaningful     |
| filename               | Original filename                 |
| extension              | File extension                    |
| file_size              | Bytes                             |
| modified_at            | Filesystem modification timestamp |
| file_hash              | Exact-content hash                |
| width                  | Image width                       |
| height                 | Image height                      |
| orientation            | EXIF orientation                  |
| camera_make            | Camera manufacturer               |
| camera_model           | Camera model                      |
| exif_datetime_original | Original EXIF timestamp           |
| exif_create_date       | EXIF creation timestamp           |
| filename_datetime      | Timestamp parsed from filename    |
| filesystem_datetime    | Filesystem fallback               |
| normalized_datetime    | Selected normalized timestamp     |
| timestamp_source       | Source used                       |
| timestamp_confidence   | Confidence classification         |
| gps_latitude           | Optional latitude                 |
| gps_longitude          | Optional longitude                |
| import_status          | Inventory status                  |
| created_at             | Registration timestamp            |

### image_analysis

Stores derived image measurements.

| Column                | Purpose                    |
| --------------------- | -------------------------- |
| id                    | Primary key                |
| image_id              | Foreign key                |
| average_luminance     | Average luminance          |
| median_luminance      | Median luminance           |
| dark_pixel_percentage | Percentage below threshold |
| histogram_json        | Optional histogram         |
| analysis_version      | Algorithm version          |
| analyzed_at           | Timestamp                  |

### image_status

Stores classification and user decisions (exactly one row per image).

| Column        | Purpose                                          |
| ------------- | ------------------------------------------------ |
| id            | Primary key                                      |
| image_id      | Foreign key, unique (one status row per image)   |
| status        | Eligible, flagged, rejected, quarantined, excluded |
| reason        | Classification reason                            |
| user_override | Manual decision flag                             |
| updated_at    | Last update                                      |

`image_id` is unique — each image has at most one status record, so re-classifying replaces the row rather than appending a duplicate. (Resolves audit finding D1 in `docs/audit/08_SYNTHESIS.md`: without this key the table allowed multiple rows per image.)

### timeline_events

Stores identified timeline conditions.

| Column         | Purpose                              |
| -------------- | ------------------------------------ |
| id             | Primary key                          |
| start_datetime | Start                                |
| end_datetime   | End                                  |
| event_type     | Gap, anomaly, metadata warning, etc. |
| severity       | Informational, warning, critical     |
| details_json   | Supporting information               |

### sampling_runs

Stores each sampling configuration and execution.

| Column             | Purpose                      |
| ------------------ | ---------------------------- |
| id                 | Primary key                  |
| method             | Interval, every Nth, uniform |
| configuration_json | Parameters                   |
| created_at         | Creation timestamp           |
| status             | Run status                   |

### sampling_items

Stores images selected during a sampling run.

| Column          | Purpose          |
| --------------- | ---------------- |
| id              | Primary key      |
| sampling_run_id | Foreign key      |
| image_id        | Selected image   |
| target_datetime | Target timestamp |
| actual_datetime | Actual timestamp |
| sequence_index  | Output order     |

### processing_jobs

Stores long-running operations.

| Column             | Purpose                |
| ------------------ | ---------------------- |
| id                 | Primary key            |
| job_type           | Operation type         |
| status             | Job status             |
| configuration_json | Configuration snapshot |
| progress_current   | Completed items        |
| progress_total     | Total items            |
| started_at         | Start                  |
| completed_at       | Completion             |
| error_json         | Error details          |
| recovery_data_json | Recovery information   |

### file_operations

Tracks filesystem operations.

| Column           | Purpose                         |
| ---------------- | ------------------------------- |
| id               | Primary key                     |
| job_id           | Associated job                  |
| operation_type   | Copy, move, quarantine, restore |
| source_path      | Original path                   |
| destination_path | Destination                     |
| status           | Operation status                |
| checksum         | Optional verification hash      |
| created_at       | Timestamp                       |
| completed_at     | Timestamp                       |

### video_exports

Stores video export configurations and results.

| Column              | Purpose           |
| ------------------- | ----------------- |
| id                  | Primary key       |
| sampling_run_id     | Source selection  |
| output_path         | Video destination |
| container           | MP4, MOV          |
| codec               | H.264, HEVC, etc. |
| fps                 | Output FPS        |
| resolution          | Output resolution |
| playback_multiplier | Playback speed    |
| duration_seconds    | Result duration   |
| file_size           | Output size       |
| status              | Export status     |
| configuration_json  | Full settings     |
| created_at          | Creation          |
| completed_at        | Completion        |

### flicker_runs

Stores flicker analysis and correction configuration.

| Column             | Purpose            |
| ------------------ | ------------------ |
| id                 | Primary key        |
| sampling_run_id    | Source selection   |
| configuration_json | Algorithm settings |
| algorithm_version  | Version            |
| status             | Processing status  |
| results_json       | Analysis summary   |
| created_at         | Timestamp          |

## 3. Relationships

```
projects
   │
   ├── images
   │     ├── image_analysis
   │     └── image_status
   │
   ├── timeline_events
   ├── processing_jobs
   │     └── file_operations
   │
   ├── sampling_runs
   │     └── sampling_items
   │           └── images
   │
   ├── flicker_runs
   └── video_exports
```

The exact relationship strategy must be reviewed. Since each project has its own database, a project_id column in every table may not be necessary.

## 4. Indexes

Evaluate indexes on:

- images.normalized_datetime

- images.original_path

- images.file_hash

- image_status.status

- sampling_items.sampling_run_id

- sampling_items.sequence_index

- processing_jobs.status

- file_operations.job_id

- video_exports.created_at

Indexes should be added according to actual query patterns.

## 5. Data integrity

- Use foreign keys.

- Enable SQLite foreign-key enforcement.

- Use transactions for related updates.

- Define uniqueness constraints where appropriate.

- Avoid storing derived values without identifying the algorithm version.

- Do not assume timestamps are always valid.

- Handle missing files without deleting their inventory records automatically.

## 6. Migrations

Every schema change must be versioned.

The application must:

- Detect the current schema version.

- Apply migrations in order.

- Fail safely when migration cannot complete.

- Preserve existing project data.

- Never silently recreate an incompatible project database.

## 6a. Implemented versions (Phase 1–2)

The version lives in `PRAGMA user_version`; `crates/core/src/import/store.rs` applies each
step in order, in its own transaction together with the version bump, and refuses a database
whose version is newer than the build (left untouched, never recreated).

| Version | Change |
| --- | --- |
| 1 | `projects`, `images` (columns of §2), `idx_images_hash`, `idx_images_relative`. Phase 1 databases carry `user_version = 0` with exactly this schema; step 1 is a no-op for them (`IF NOT EXISTS`). |
| 2 | `idx_images_normalized_datetime` on `images(normalized_datetime)`; new column `images.timestamp_offset_minutes INTEGER` — the UTC offset applied when resolving, `NULL` when the source recorded no zone and UTC was assumed (ADR-019). |
| 3 | Phase 3 cleaning (ADR-020): `image_analysis`, `image_status`, `file_operations` (below), indexes `idx_image_status_status`, `idx_file_operations_job`, `idx_file_operations_status`; new column `images.current_path` — where the original is now (`NULL` = at `original_path`; the quarantine path while quarantined). `original_path` stays the path of record and is never rewritten. |
| 4 | Phase 4 sampling: `sampling_runs`, `sampling_items` (below), index `idx_sampling_items_run`. |
| 5 | Phase 5 export: `video_exports` (below), index `idx_video_exports_created`. |
| 6 | Phase 6 consolidation (ADR-023): `consolidation_runs` (below). `file_operations` is rebuilt in the same step (SQLite cannot alter a CHECK in place): its rows are copied, `operation_type` also accepts `copy` · `move` · `move-back` · `remove-copy`, `job_id` references `consolidation_runs`, and the new index `idx_file_operations_image` serves restore. `images.current_path` may now point into a working folder (a moved original). |

**Timestamp columns as written by the Phase 2 backfill** (rows with `timestamp_source IS NULL` only — idempotent):

| Column | Content |
| --- | --- |
| `exif_datetime_original` | EXIF `DateTimeOriginal` text, verbatim (even an unusable `0000:00:00 00:00:00`) |
| `exif_create_date` | EXIF `DateTimeDigitized` text, verbatim |
| `filename_datetime` | The date/time substring matched in the file name, verbatim |
| `filesystem_datetime` | `modified_at` in the normalised format |
| `normalized_datetime` | Winning value as fixed-width UTC `YYYY-MM-DDTHH:MM:SS.mmmZ` (text order = time order) |
| `timestamp_source` | `exif_datetime_original` · `exif_datetime_digitized` · `filename` · `filesystem`; `NULL` = unresolved (retried next run) |
| `timestamp_confidence` | `high` (EXIF original) · `medium` (EXIF digitized, filename) · `low` (filesystem) |
| `timestamp_offset_minutes` | See version 2 above; `0` for the filesystem time (a true instant) |

**Phase 3 tables (v3):**

| Table | Columns | Notes |
| --- | --- | --- |
| `image_analysis` | `image_id` (PK, FK), `algorithm_version`, `source_hash`, `width`, `height`, `mean_luma`, `median_luma`, `histogram_json` (256 luma counts), `failure` (`unreadable` · `undecodable`), `xmp_rating`, `analyzed_at` | One row per image. Re-measured when `algorithm_version` changes or the import hash (`images.file_hash`) differs from `source_hash`; quarantined images keep their measurement. The dark fraction at any level and the 32-bin display histogram are derived from `histogram_json`, so changing the dark level needs no new pass. |
| `image_status` | `id`, `image_id` (unique FK), `status` (`eligible` · `flagged` · `rejected` · `quarantined`, CHECK), `reason` (JSON `{ thresholds, reasons[] }`), `user_override`, `updated_at` | Automatic proposals are only ever `eligible` / `flagged`. `rejected` comes from a user decision; `quarantined` comes from a completed move. Rows with `user_override = 1` are never reclassified. |
| `file_operations` | `id`, `job_id`, `image_id`, `operation_type` (`quarantine` · `restore`), `source_path`, `destination_path`, `status` (`pending` · `completed` · `failed`), `checksum` (SHA-256), `failure`, `created_at`, `completed_at` | Written *before* the file is touched. `pending` rows are reconciled from the disk state when a project is opened while no file operation is running (ADR-020). Updates only ever apply to rows still `pending`. |

**Cleaning queries (Phase 3):** `pending_analysis`: images without a current-version row for their current hash. `classify_all`: recomputes proposals from `image_analysis` and the thresholds in `projects.settings_json` → `clean`, a pure function, so it is reproducible from SQLite alone. `review`: images joined with status and analysis, chronological.

**Phase 4 tables (v4):**

| Table | Columns | Notes |
| --- | --- | --- |
| `sampling_runs` | `id`, `method` (`fixed-interval` · `every-nth` · `uniform`), `configuration_json` (method + parameters + `from`/`to`/`include_flagged`), `algorithm_version`, `input_count`, `input_fingerprint` (SHA-256 of `id:normalized_datetime` lines, in order), `excluded_json` (rejected · quarantined · flagged · no_timestamp · out_of_period), `target_count`, `selected_count`, `unmatched_count`, `unmatched_json` (first 1000 unmatched targets), `status` (`completed`), `created_at` | Everything Pipeline §13 asks for: configuration, algorithm version, an identification of the inputs, and the output (below). |
| `sampling_items` | `id`, `sampling_run_id` (FK, cascade), `image_id` (FK), `target_datetime` (`NULL` for every-Nth), `actual_datetime`, `sequence_index` | The ordered selection; unique per `(run, sequence_index)` and `(run, image)`, so no photograph appears twice. |

**Sampling input rule:** every image except `rejected` and `quarantined`; `flagged` optional; images without `normalized_datetime` are excluded and counted; ordered as the timeline (`normalized_datetime`, `original_path`). `verify_run` recomputes the stored configuration on the current input and compares both the fingerprint and the sequence.

**Phase 5 table (v5):**

| Table | Columns | Notes |
| --- | --- | --- |
| `video_exports` | `id`, `sampling_run_id` (FK), `settings_json` (codec, fps, speed, width, height, quality), `ffmpeg_version`, `argv_json` (the exact FFmpeg arguments), `output_path`, `status` (`running` · `completed` · `failed` · `cancelled`), `input_photos`, `output_frames`, `duration_seconds`, `file_size`, `probe_json` (ffprobe: codec, size, packets, duration), `failure`, `stderr_tail`, `created_at`, `completed_at` | One row per attempt, written before FFmpeg starts. A row left `running` by a crash becomes `failed`/`interrupted` the next time exports are listed while none is running. |

**Phase 6 table (v6):**

| Table | Columns | Notes |
| --- | --- | --- |
| `consolidation_runs` | `id`, `operation` (`copy` · `move`), `destination`, `configuration_json` (destination, operation, naming, sampling run), `sampling_run_id` (FK, `NULL` = all eligible photographs), `plan_fingerprint` (SHA-256 of the confirmed plan), `planned`, `duplicates`, `already_present`, `unsafe_names`, `bytes`, `status` (`running` · `completed` · `partial` · `cancelled` · `interrupted`), `undo_status` (`undone` · `partially-undone`), `created_at`, `completed_at`, `undone_at` | What a run planned. What it did is the journal: `file_operations` rows with `job_id` = the run (placements `copy` / `move`, undo steps `remove-copy` / `move-back`), counted when the run is read. A row left `running` by a crash becomes `interrupted` the next time runs are listed while none is running. |

**Location of record:** every reader uses `COALESCE(current_path, original_path)`. `current_path` is `NULL` exactly when the file is at `original_path`. A quarantine starts from the current location and its restore returns there (the source of the last completed `quarantine` row).

**Read queries (Phase 2):**

- *Timeline* — every image `ORDER BY normalized_datetime IS NULL, normalized_datetime, original_path`, with `unixepoch(normalized_datetime, 'subsec')` as epoch milliseconds; unresolved images last, never dropped.
- *Distribution over a period* — `strftime('%Y-%m-%dT%H' | '%Y-%m-%d' | '%Y-%m', normalized_datetime)` (each key is a prefix of the values it counts, so it can be passed back as a bound) with `COUNT(*)`, optionally bounded by `from <= normalized_datetime < to`.
- *Gaps* — computed over the resolved timeline: consecutive intervals longer than `max(min_seconds, factor × median interval)` (defaults 3600 s, ×10; caller values clamped to `[0, 1e6]` and `[0, 100 years]`), each flagged `source_changed` when its two sides differ in source or zone assumption. Reported only; no value is adjusted.

## 7. Open design questions

- Whether file metadata should be stored in one table or separated further.

- How to represent multiple source directories. (Phase 6 gathers the folders of one collection into a working folder; several collections per project remain open.)

- Whether image statuses require a full audit history.

- Whether timeline events should be computed or persisted.

- How to store flicker correction curves efficiently.

- How to represent processing dependencies between jobs.

- Whether thumbnails need a dedicated database table.
