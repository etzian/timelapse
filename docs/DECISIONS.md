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

## ADR-017 — FFmpeg distribution path: LGPL build, SVT-AV1 default, VideoToolbox for H.264/HEVC

**Status:** Accepted (2026-11-09)

We ship a self-contained FFmpeg sidecar built on the **LGPLv2.1+** licence path with every GPL-only encoder disabled (`--disable-gpl`; `x264` and `x265` absent from the shipped binary). **AV1 encoding uses SVT-AV1 (BSD-3-Clause) as the default output codec**; H.264/HEVC is available as a compatibility profile, produced exclusively via the macOS **VideoToolbox** hardware encoder — no software fallback. FFmpeg runs as a controlled subprocess with an explicit, recorded argument list; it is never statically linked into the Rust binary.

A build-time assertion (`strings` + `lipo` + symbol scan on the shipped binary) gates Phase 0 exit: **no `libx264`/`libx265`/GPL-only symbol may be present**. FFmpeg's LGPL notice text and modification list ship in the app bundle under `LICENSES/`, satisfying the *restricted* class conditions of `_foundation/tech/licences/policy.yaml`.

**Reason:** Closes synthesis finding **C2 / D2** (`docs/audit/08_SYNTHESIS.md`), which blocked Phase 0 exit as a legal/licence gate. The default Homebrew `ffmpeg` build (Option A) links `x264/x265` → denied class. Pointing at the user's system `ffmpeg` (Option C) violates ADR-008 + `TDD.md:318/346/444` self-containment and reproducibility intent. Option B keeps every *required* codec in the allow list, uses the restricted class only under its conditions (dynamic, unmodified, text shipped), matches the controlled-subprocess model of ADR-008/TDD, and gives us a fast multi-threaded AV1 encoder for the timelapse workload. SVT-AV1 is chosen over libaom because timelapse encode times at 30–60 fps × minutes-long timeline scale, not seconds; a slow AV1 default would push users to the compatibility (H.264) path and defeat the purpose of offering AV1.

**Consequences applied:**
- `docs/DECISIONS.md` — this ADR; open FFmpeg sub-decisions below (lines § "## Open decisions" → **FFmpeg**) now point at it.
- `TDD.md` §9 (dependencies / FFmpeg distribution) — updated to cite this ADR, SVT-AV1, and VideoToolbox, and to keep its rule that "not every build has identical codec support" (`TDD.md:318`).
- `PRD.md` §5.7 supported codecs / default output — **AV1 (SVT-AV1) is now the default output format**; H.264/HEVC (VideoToolbox) is listed as the compatibility profile for players that do not yet support AV1.
- `docs/FFMPEG_AV1_ANALYSIS.md` header — status line updated to point at ADR-017; the document's recommendation is adopted as-is with one deliberate change (AV1 is **default**, not opt-in).
- Phase 5 exit criteria (`docs/DEVELOPMENT_ROADMAP.md`) now include the GPL-symbol build-time assertion and LGPL-notice placement inside the packaged bundle.

**Open items carried to Phase 5:** (a) exact provenance, version pin, and build script (`tools/build-ffmpeg/`) for the shipped FFmpeg; (b) VideoToolbox availability guard per minimum supported macOS version — below that, refuse AV1-tier jobs with a clear error rather than silently falling back; (c) final placement of the LGPL notice text inside the packaged bundle.

## ADR-018 — Rust libraries: `rusqlite` (bundled) and `sha2` for the Phase 1 store and content hashing

**Status:** Accepted (2026-11-12, approved by user on request per AGENTS.md §5)

Two library choices introduced with the Phase 1 import pipeline are locked in:

- **Project database — `rusqlite` 0.32** (dependency feature `bundled`). Synchronous binding to SQLite; `bundled` compiles the SQLite C source into our binary so there is no runtime dependency on a system-provided `libsqlite3`. Fits the synchronous, local-first core in `crates/core` without an async runtime.
- **Content hashing — `sha2` 0.10** (RustCrypto). Canonical pure-Rust SHA-256; used for exact-duplicate detection in `crates/core/src/import/dedup.rs`. Streaming-safe via the `Digest` trait.

**Reason:** Both are pure Rust, well-audited, minimal-dependency, and dual-licensed (MIT/Apache-2.0) with SQLite itself in the public domain — no licence exceptions and no runtime/system library coupling, matching AGENTS.md §5 ("Keep Tauri capabilities narrow", "Do not introduce dependencies without approval"). `rusqlite` avoids the `tokio`-driven weight of `sqlx`, which would conflict with our synchronous core API; `sha2` avoids the heavier `ring` dependency for a job that is only SHA-256. These were previously listed under "Open decisions → Technical dependencies" (the SQLite library) and have now been investigated, compared, and approved.

**Consequences applied:** `crates/core/Cargo.toml` records both dependencies; the previously open "SQLite Rust library" line under Open decisions now points at this ADR. The EXIF extraction library is resolved by ADR-019 (Phase 2). Image-processing libraries remain **open** (later phases, not yet introduced).

## ADR-019 — Phase 2 timestamping: `kamadak-exif` for EXIF reads and a `time`-based normalizer

**Status:** Accepted (2026-11-12, approved by user on request per AGENTS.md §5)

Two library choices underpin the Phase 2 EXIF capture and chronological-timeline feature:

- **EXIF extraction — `kamadak-exif` 0.5.** Pure-Rust reader for the EXIF segments inside JPEG/TIFF (the extensions we support in Phase 1). We only need the two timestamp tags — `DateTimeOriginal` (when the picture was taken) and, as a fallback, `DateTimeDigitized` — plus camera make/model where present. It reads from in-memory bytes, so it fits the synchronous core and needs no async runtime; it does not depend on image-decoding stacks we would otherwise pull in for thumbnail work later.
- **Date/time normalisation — `time` 0.3** (default features only; see *Correction* below). EXIF timestamp tags carry a wall-clock stamp with **no timezone**; we parse those stamps and record them as UTC-normalised ISO-8601 text with an explicit `timestamp_source` and `timestamp_confidence`, per ADR-010. `time` 0.3 is a small, well-tested formatter/parser chosen for stable ISO-8601 output without pulling in a full IANA time-zone database (we do not resolve named zones in Phase 2 — we assume UTC and record confidence), keeping the dependency surface minimal.

**Reason:** Both are pure Rust and on the licence allow-list (`_foundation/tech/licences/policy.yaml`) — no licence exceptions, matching AGENTS.md §5. `kamadak-exif` gives tag-level access (all we need) without the weight of a full image decoder; `time` 0.3's `formatting` feature is the lightest path to deterministic ISO-8601 output for SQLite `strftime()` bucketing of the timeline.

**Correction (2026-11-14, during implementation):** `kamadak-exif` 0.5.5 is licensed **BSD-2-Clause**, not MIT/Apache-2.0 as first written here; BSD-2-Clause is on the same allow-list, so the decision and the "no exception" conclusion are unchanged (`time` 0.3 is MIT/Apache-2.0). Its only dependency, `mutate_once`, is BSD-2-Clause too. The `formatting` feature of `time` turned out to be unnecessary — the fixed-width normalised string is written directly from `time`'s validated fields — so the dependency is declared with default features only. Explicit EXIF `OffsetTime*` offsets are applied when present (a fixed offset, not a named-zone lookup); their absence is recorded in `images.timestamp_offset_minutes = NULL` (schema v2).

**Known limitation (recorded, accepted):** HEIX/HEIC EXIF layout differs from JPEG/TIFF and Rust-side readers have historically been weaker there. Phase 2 scopes `kamadak-exif` to the supported extensions (JPEG/TIFF). If we later add HEIC import, EXIF timestamp resolution must be re-validated as a separate, explicitly-approved effort — it is **not** assumed to work in Phase 2.

**Consequences applied:**
- `crates/core/Cargo.toml` records both dependencies (see ADR-019 above and the Phase 2 plan).
- Schema v2 (`PRAGMA user_version`): `idx_images_normalized_datetime` and `images.timestamp_offset_minutes` (docs/DATABASE_SCHEMA.md §6a).
- `open-decisions → Technical dependencies → EXIF extraction library` now points at this ADR; it is resolved for Phase 2.
- The timeline feature stores `normalized_datetime` as ISO-8601 TEXT and buckets it with SQLite `strftime()` (no new time-zone machinery, per the reason above).

---

## Open decisions

The following decisions must be investigated and presented for approval before implementation makes them difficult to change.

### Technical dependencies

- **Resolved (ADR-018):** `rusqlite` 0.32 (bundled) for SQLite persistence; `sha2` 0.10 for content hashing in the import pipeline. See ADR-018.

- **Resolved (ADR-019):** `kamadak-exif` 0.5 for EXIF timestamp reads and `time` 0.3 (feature `formatting`) for date-string normalisation into ISO-8601; both on the licence allow-list, EXIF stamped-as-UTC assumption recorded with an explicit confidence field (see ADR-019). HEIC EXIF resolution is **not** in scope for Phase 2 and would be a separately approved effort.

- Image-processing library.

- Async runtime and task coordination.

- Thumbnail format.

- Video preview implementation.

### FFmpeg

- **Resolved (ADR-017):** bundled LGPL-only sidecar; SVT-AV1 for AV1 as the default codec tier; macOS VideoToolbox for H.264/HEVC as a compatibility profile, no x264/x265 fallback. See `docs/FFMPEG_AV1_ANALYSIS.md` and ADR-017 for the full justification.

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
