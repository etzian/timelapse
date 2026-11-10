# Timelapse Studio

## Architecture Decision Record (ADR) Register

**Version:** 0.1  
**Status:** Initial decisions and open questions

This document records architectural and product decisions to prevent inconsistent implementation across development sessions.

---

## ADR-001 — Local-first application

**Status:** Accepted

The application operates locally. No cloud account, mandatory online service or cloud storage is required.

**Reason:** User privacy, independence and reliable processing of large photographic collections.

## ADR-002 — Tauri 2 desktop framework

**Status:** Accepted

Tauri 2 is the desktop application framework.

**Reason:** Lightweight desktop integration with a Rust backend and web-based frontend.

## ADR-003 — Svelte and TypeScript frontend

**Status:** Accepted

Svelte and TypeScript are used for the user interface.

**Reason:** Modern, maintainable frontend with a clear separation from backend processing.

## ADR-004 — Rust processing engine

**Status:** Accepted

All core processing logic is implemented in Rust.

**Reason:** Performance, control over filesystem operations, maintainability and potential future CLI reuse.

## ADR-005 — SQLite project database

**Status:** Accepted

Each project uses a SQLite database as its authoritative source of structured project data.

**Reason:** Local persistence, transactional operations, indexing and portability.

## ADR-006 — Original photographs remain immutable

**Status:** Accepted

Processing must not modify original photographs.

**Reason:** Protect source material and allow repeatable processing.

## ADR-007 — Project data separated from original photographs

**Status:** Accepted

The project directory contains the database and generated assets (the derived export record among them). Original photographs may remain elsewhere.

**Reason:** Avoid mandatory duplication of large source collections. An earlier reading listed a "manifest" here as a stored project file; that was superseded by [ADR-015](#adr-015---the-project-manifest-is-a-derived-export-record-sqlite-only-source-of-truth) — the manifest is a derived on-demand export record, not a persisted file, so only the SQLite database and generated products are retained in the project directory.

## ADR-008 — FFmpeg for video generation

**Status:** Accepted in principle

FFmpeg is the planned video encoding engine.

**Open question:** Packaging, licensing, codec availability and hardware-encoding strategy must be evaluated before release.

## ADR-009 — Six-stage primary workflow

**Status:** Accepted

The main workflow consists of import, cleaning, consolidation, sampling, video configuration and export.

**Reason:** Matches the actual user workflow.

## ADR-010 — Timestamp-based sampling

**Status:** Accepted

Sampling must use chronological timestamps when available, not merely filesystem order.

**Reason:** Timelapse quality depends on temporal distribution.

## ADR-011 — Quarantine instead of automatic deletion

**Status:** Accepted

Rejected images are quarantined through a traceable operation. Permanent deletion is not automatic.

**Reason:** Recoverability and user control.

## ADR-012 — Flicker reduction is a first-class advanced capability

**Status:** Accepted

Flicker reduction must be designed as an independent module, not as an incidental brightness adjustment.

**Reason:** GoPro exposure fluctuations are a significant quality problem.

## ADR-013 — Stabilization is independent of flicker reduction

**Status:** Accepted

Stabilization and flicker reduction are separate processing capabilities.

**Reason:** Different algorithms, parameters and validation requirements.

## ADR-014 — Contact sheets are a core feature

**Status:** Accepted

Contact sheets are part of the main workflow, not merely an optional reporting feature.

**Reason:** Visual review is essential when working with large image collections.

## ADR-015 — The project manifest is a derived export record

**Status:** Accepted

There is no independently maintained project-manifest file in the data model. Project identification, export configuration and export results exist only in SQLite (`projects`, `video_exports`). Where a manifest still appears, it is redefined as an on-demand, derived, human-readable export record that can be generated from SQLite for portability and inspection; it is not read back into the application and is not a load-bearing source of state.

**Reason:** `PRD.md:83`, `TDD.md:172/188` and `ARCHITECTURE.md:42/80/197` treated the JSON manifest as a persistent artifact that had to stay consistent with SQLite, while `DECISIONS.md` (ADR-007) framed it as the portable format. A second hand-maintained source of truth is exactly the failure mode this product must avoid over an immutable image corpus. `PROCESSING_PIPELINE.md:410` already states the canonical intent — store export configuration and result in SQLite. Reducing the manifest to a derived snapshot removes the drift surface while preserving the portability and inspection `PRD.md:83` requires.

**Consequences applied:** `PRD.md:83/459`, `TDD.md:172/188/198`, `ARCHITECTURE.md:42/80/197` reworded on approval.

## ADR-016 — Product ships in fr-CA and en; fr-CA is a release blocker

**Status:** Accepted

The application ships in two locales, fr-CA and en (BCP 47 tags), managed as Fluent (`.ftl`) catalogs, one directory per locale (`locales/fr/`, `locales/en/`). A build that cannot be verified against the complete fr-CA catalog must not ship. The default locale follows the OS locale (French or Québec region → fr-CA; otherwise en). The authoritative locale list is `_foundation/company/etzian.yaml` (`locales:`), read via `ez profile`, never re-typed into project code.

**Reason:** Company policy requires products to ship in fr-CA and en with fr-CA as a release blocker, but no approved document codified the supported locales or the blocking rule. Codifying it closes the gap and gives Phase 0 concrete acceptance gates: both catalogs present, UI strings fully extracted, locale-completeness check on CI.

**Consequences applied:** `PRD.md` §6 Localization NFR added; §9 portability question closed (related to ADR-015); `TDD.md` §9 i18n dependency and `locales/` layout recorded.

---

## Open decisions

The following decisions must be investigated and presented for approval before implementation makes them difficult to change.

### Technical dependencies

- SQLite Rust library.

- EXIF extraction library.

- Image-processing library.

- Async runtime and task coordination.

- Thumbnail format.

- Video preview implementation.

### FFmpeg

- Bundled versus separately installed.

- Static versus dynamic build.

- Codec licensing implications.

- Apple VideoToolbox availability.

- AV1 support and distribution.

### Image processing

- Flicker algorithm.

- Exposure reference strategy.

- Temporal smoothing method.

- Stabilization algorithm.

- Optional image-adjustment pipeline.

### Data model

- Handling multiple source directories.

- Source relocation.

- Persistent versus computed timeline events.

- Processing dependency tracking.

- Audit history for image status changes.

### Product

- RAW image support.

- CLI availability.

- Future Windows support.

- Future mobile support.

- Application update strategy.

---

## Decision-making rule

The implementation agent must not silently change an accepted decision.

When a change is proposed:

1. Explain the problem.

2. Describe the alternatives.

3. Compare performance, complexity, reliability and licensing.

4. Recommend one approach.

5. Wait for approval when the change materially affects architecture or product behavior.

6. Update this document after approval.
