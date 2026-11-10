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

## 7. Open design questions

- Whether file metadata should be stored in one table or separated further.

- How to represent multiple source directories.

- Whether image statuses require a full audit history.

- Whether timeline events should be computed or persisted.

- How to store flicker correction curves efficiently.

- How to represent processing dependencies between jobs.

- Whether thumbnails need a dedicated database table.
