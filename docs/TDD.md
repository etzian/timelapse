# Timelapse Studio

## Technical Design Document (TDD)

**Version:** 0.1  
**Status:** Initial technical proposal — pending review

---

## 1. Technology stack

| Layer             | Technology                      |
| ----------------- | ------------------------------- |
| Desktop framework | Tauri 2                         |
| Frontend          | Svelte + TypeScript             |
| Backend           | Rust                            |
| Database          | SQLite                          |
| Video processing  | FFmpeg                          |
| Image processing  | Rust image-processing libraries |
| Metadata          | EXIF extraction library         |
| Logging           | Rust tracing ecosystem          |

The exact versions and dependencies must be selected and documented during implementation preparation.

## 2. Architectural principles

- Processing logic belongs in Rust.

- Svelte is responsible for presentation and user interaction.

- Tauri connects the frontend and backend.

- SQLite is the authoritative source of project state.

- FFmpeg is invoked through a controlled process interface.

- Filesystem operations must be explicit and traceable.

- Processing modules must not depend on UI components.

The Rust processing engine should be reusable independently of Tauri in the future.

## 3. Application layers

### Frontend

Responsibilities:

- Project navigation.

- Workflow navigation.

- Configuration forms.

- Image galleries.

- Timeline visualization.

- Contact sheets.

- Progress and status display.

- Preview and comparison interfaces.

- Error presentation.

The frontend must not implement business-critical image or file processing.

### Tauri integration

Responsibilities:

- Expose validated commands.

- Send processing events to the frontend.

- Manage application lifecycle.

- Handle file and folder selection.

- Coordinate frontend requests with Rust services.

Long-running operations must not block the main application thread.

### Rust engine

Responsibilities:

- Import and inventory.

- Metadata extraction.

- Timeline analysis.

- Brightness analysis.

- Duplicate detection.

- Sampling.

- File management.

- Flicker reduction.

- Stabilization.

- Contact-sheet generation.

- FFmpeg orchestration.

### Persistence

SQLite stores:

- Project configuration.

- Imported image inventory.

- Metadata.

- Analysis results.

- Image classifications.

- Sampling configuration and results.

- Processing jobs.

- File-operation history.

- Export history.

Photographic binary data must not be stored in SQLite.

## 4. Proposed Rust module structure

```
src-tauri/src/
├── main.rs
├── lib.rs
├── commands/
│   ├── project.rs
│   ├── import.rs
│   ├── analyze.rs
│   ├── clean.rs
│   ├── consolidate.rs
│   ├── sampling.rs
│   ├── video.rs
│   └── export.rs
├── services/
│   ├── filesystem.rs
│   ├── metadata.rs
│   ├── image_analysis.rs
│   ├── duplicate_detection.rs
│   ├── timeline.rs
│   ├── sampling_engine.rs
│   ├── flicker_reduction.rs
│   ├── stabilization.rs
│   ├── thumbnail.rs
│   ├── contact_sheet.rs
│   └── ffmpeg.rs
├── models/
│   ├── project.rs
│   ├── image.rs
│   ├── timeline.rs
│   ├── processing.rs
│   └── video.rs
├── storage/
│   ├── database.rs
│   ├── migrations/
│   ├── project_store.rs
│   └── export_record.rs
└── utils/
    ├── errors.rs
    ├── logging.rs
    └── validation.rs
```

This is a proposed logical structure. The implementation agent may recommend adjustments, but must explain them before changing the architecture.

## 5. Project storage

Proposed project directory:

```
TimelapseProject/
├── project.db
├── thumbnails/
├── contact-sheets/
├── temp/
├── exports/
└── logs/
```

Original photographs remain in their original locations unless the user explicitly requests a copy or move.

SQLite is the authoritative project database (ADR-015). There is no stored manifest file: the on-demand, human-readable export record is generated from SQLite into `exports/` when requested, for portability and inspection; it is never read back into the application as state.

## 6. Processing model

Every long-running operation should have:

- A unique job identifier.

- A defined status.

- A configuration snapshot.

- Start and completion timestamps.

- Progress information where available.

- Error details.

- Cancellation support where practical.

- A recovery strategy.

Suggested statuses:

- pending

- running

- completed

- failed

- cancelled

- interrupted

- needs_recovery

Operations must be designed to process manageable batches.

## 7. Frontend communication

Use Tauri commands for requests and events for asynchronous progress.

The frontend should receive structured progress events containing information such as:

- Job ID.

- Operation type.

- Current phase.

- Completed items.

- Total items, when known.

- Percentage, when meaningful.

- Current filename, where useful.

- Warning and error information.

Do not send large image data through frequent events. Prefer thumbnail files and controlled preview mechanisms.

## 8. Error handling

Define application-level error categories:

- Project errors.

- Database errors.

- Filesystem errors.

- Metadata errors.

- Image-processing errors.

- FFmpeg errors.

- Validation errors.

- Cancellation.

- Recovery errors.

Errors should be actionable and understandable.

Avoid silently ignoring failed files. Batch operations should record individual failures and provide a final report.

## 9. Dependencies and licensing

Before adopting a dependency:

1. Confirm that it is actively maintained.

2. Check macOS compatibility.

3. Check supported image formats.

4. Evaluate performance on large collections.

5. Review licensing.

6. Consider binary distribution requirements.

7. Document the reason for selecting it.

Particular attention is required for:

- FFmpeg and its build configuration.

- EXIF extraction.

- Image decoding and processing.

- Video preview.

- Stabilization and flicker algorithms.

Internationalization (ADR-016): user-facing strings are extracted into Fluent catalogs under `locales/fr/*.ftl` and `locales/en/*.ftl`, loaded at runtime via the Rust `fluent-bundle` crate re-exposed to the Svelte frontend (or an equivalent Tauri i18n approach chosen during Phase 0). The build gate must include a locale-completeness check: no shipping build with a missing or divergent fr-CA catalog. Supported locales come from `_foundation/company/etzian.yaml` (`locales:`) via `ez profile`, never hardcoded.

Do not assume that every FFmpeg build has identical codec or hardware-encoding support.

## 10. Performance strategy

- Use parallel processing for independent image tasks.

- Limit concurrency according to memory and disk throughput.

- Cache thumbnails.

- Avoid repeated EXIF extraction when source files are unchanged.

- Use database indexes for frequent queries.

- Stream large file operations.

- Avoid loading entire collections into memory.

- Separate preview quality from final processing quality.

Performance optimizations must be benchmarked rather than assumed.

## 11. Security and filesystem safety

- Validate all user-provided paths.

- Avoid shell command construction using interpolated user input.

- Invoke FFmpeg with explicit argument lists.

- Never use uncontrolled wildcard expansion.

- Verify source and destination paths.

- Detect collisions before writing.

- Avoid following unexpected symbolic links without a defined policy.

- Never delete original photographs automatically.

## 12. Testing strategy

### Unit tests

- Timestamp parsing.

- EXIF fallback rules.

- Chronological sorting.

- Brightness calculations.

- Duplicate hashing.

- Sampling algorithms.

- Video-duration calculations.

- Filename generation.

### Integration tests

- SQLite persistence.

- Import of sample directories.

- Thumbnail generation.

- File copy and move.

- Quarantine and recovery.

- FFmpeg invocation.

- Export validation.

### Reliability tests

- Interrupted processing.

- Insufficient disk space.

- Missing source files.

- Destination collisions.

- Corrupt images.

- Invalid EXIF.

- Large collections.

### Visual validation

Flicker reduction and stabilization require visual evaluation using real photographic sequences.

## 13. Definition of done

A feature is not complete merely because the application compiles.

Each milestone must include:

- Working implementation.

- Appropriate automated tests.

- Error handling.

- Validation on representative data.

- Updated documentation.

- No known regression in previous milestones.

- A documented list of remaining limitations.

## 14. Technical decisions requiring approval

- SQLite access library.

- EXIF library.

- Image-processing library and supported formats.

- Async runtime and task coordination.

- FFmpeg distribution strategy.

- Thumbnail storage format.

- Video preview strategy.

- Flicker-reduction algorithm.

- Stabilization implementation.

The agent must evaluate these choices and present recommendations before committing to dependencies with significant architectural or licensing implications.
