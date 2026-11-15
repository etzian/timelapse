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

**Correction (2026-11-14, verified against the SVT-AV1 sources):** SVT-AV1 is licensed under the **BSD-3-Clause-Clear** licence ("The Clear BSD License", which expressly grants no patent rights) together with the **Alliance for Open Media Patent License 1.0**, not plain BSD-3-Clause as written above. That combination is not on the licence allow-list, so SVT-AV1 needs its own dated exception in `licence-exceptions.yaml` (drafted in Phase 5, approved only by the user). The distribution decision itself is unchanged and not reopened. Every AV1 encoder (libaom included) comes under the same AOM patent licence.

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

**Decisions confirmed by the user (2026-11-14):**
- *Mixed time bases are accepted.* A wall clock with no recorded zone (EXIF without `OffsetTime*`, filename dates) is read as UTC, while the filesystem time is a true instant. In a non-UTC locale the two kinds can therefore interleave out of order by up to the local offset. This is accepted; it stays visible (`tz_assumed`, gap `source_changed`, the UI note) and is never corrected. Later phases may rely on the timeline order as is.
- *A resolved capture time is never re-read.* Once a row has a `timestamp_source`, the backfill does not revisit it, even if the original file is later edited or replaced (Phase 1's `ON CONFLICT DO NOTHING` keeps the row): the EXIF data as first read from the original is the record we want.

**Known limitation (recorded, accepted):** HEIX/HEIC EXIF layout differs from JPEG/TIFF and Rust-side readers have historically been weaker there. Phase 2 scopes `kamadak-exif` to the supported extensions (JPEG/TIFF). If we later add HEIC import, EXIF timestamp resolution must be re-validated as a separate, explicitly-approved effort — it is **not** assumed to work in Phase 2.

**Consequences applied:**
- `crates/core/Cargo.toml` records both dependencies (see ADR-019 above and the Phase 2 plan).
- Schema v2 (`PRAGMA user_version`): `idx_images_normalized_datetime` and `images.timestamp_offset_minutes` (docs/DATABASE_SCHEMA.md §6a).
- `open-decisions → Technical dependencies → EXIF extraction library` now points at this ADR; it is resolved for Phase 2.
- The timeline feature stores `normalized_datetime` as ISO-8601 TEXT and buckets it with SQLite `strftime()` (no new time-zone machinery, per the reason above).

## ADR-020 — Phase 3 cleaning: `image` + `rayon`, luma analysis, journaled quarantine move

**Status:** Accepted (2026-11-14, approved by the user: dependencies, sequencing, quarantine model and XMP reading)

**Libraries.**
- **`image` 0.25** (MIT OR Apache-2.0), built with `default-features = false` and only `jpeg` (zune-jpeg), `png` and `tiff`, the formats Phase 1 imports. It decodes for analysis and encodes the review thumbnails. The transitive crates (zune-jpeg/zune-core, png, fdeflate, flate2, tiff, weezl, half, moxcms, pxfm, byteorder-lite, fax, quick-error, crunchy) are all MIT, Apache-2.0, BSD-3-Clause, Zlib or Unlicense alternatives on the allow-list, checked by `ez licence-audit`.
- **`rayon` 1** (MIT OR Apache-2.0) parallelises per-image analysis (TDD: "use parallel processing for independent image tasks"). SQLite writes stay on one connection, in chunked transactions.

**Analysis.** Each image is decoded and reduced so its longest side is at most 512 px. We then compute Rec. 709 *luma* (`0.2126 R + 0.7152 G + 0.0722 B` on the encoded 8-bit values, not linear-light luminance), plus the mean, median, the fraction of pixels at or below a dark threshold, and a 32-bin histogram. Each stored result carries an `algorithm_version`. A version bump re-analyses; old values are never mixed with new ones.

**Classification is a pure function** of the stored metrics, the stored XMP rating and the thresholds. The thresholds are persisted in `projects.settings_json` and copied into each decision's reason, so a decision can be reproduced from SQLite alone. A user decision (`user_override`) is never changed by reclassification.

**Lightroom XMP** is read only, from a `<stem>.xmp` sidecar (which takes precedence) or the XMP packet embedded in the file. `xmp:Rating = -1` (Lightroom's reject flag) becomes a review reason. No XML library: only the `Rating` attribute or element is extracted, with bounded reads.

**Quarantine (the minimal Phase 6 primitive pulled forward).** It runs only after explicit confirmation, and only for images the user rejected. The original goes to `<root>/.timelapse/quarantine/<relative path>`. Every step is designed so that nothing can be overwritten and the source is never lost before its copy is verified:

1. Verify the source has the import hash.
2. Journal a `pending` `file_operations` row.
3. Create the destination atomically *without replacing*: a hard link, or a `create_new` copy that keeps the mtime where the filesystem has no links (FAT/exFAT). Either fails if anything exists there, so there is no check-then-act window. (`std::fs::rename` is deliberately **not** used: it replaces an existing destination on Windows and Unix.)
4. Verify the destination hash; on a mismatch, remove only the copy we just created.
5. Remove the source name; if the file is in use, undo step 3 instead.
6. In one transaction, complete the row and set `images.current_path` and the status.

Restore runs the same steps in reverse. All file operations of the process share one lock. Reconciliation of `pending` rows runs when a project is opened, but only while no operation holds that lock, so it only ever settles rows left by a crashed process. Journal updates only touch rows that are still `pending`. An image whose import could not hash it has no reference to check in step 1; its move is still verified byte-for-byte in step 4. A Lightroom `.xmp` sidecar stays in the collection when its photo is quarantined; quarantined images are therefore not re-analysed, and keep the rating already read. Decoding is bounded at 30 000 px a side and 768 MiB per image, with at most three images decoded at once. Nothing is ever deleted except the duplicate this process itself created (ADR-011), and no byte of an original is written (ADR-006). The import scan skips `<root>/.timelapse/`, so a quarantined file is never re-imported.

**Out of scope here:** consolidation, copy and link policy (Phase 6), contact sheets (Phase 7), HEIC/RAW decoding.

## ADR-021 — FFmpeg runs as a bounded child process

**Status:** Accepted (2026-11-14, approved by the user)

The app may start **only** the FFmpeg and ffprobe executables that ship inside its own bundle, as child processes, under all of these rules:

- by **absolute path** resolved from the bundle's resource directory (a `TIMELAPSE_FFMPEG_DIR` override exists for development and the test suite only), never through `PATH`;
- with an **explicit argument vector** built by a pure function in `crates/core`: no shell, no string interpolation into a command line, and no user-supplied text in the arguments. Input frames are piped as raw pixels on stdin, so photograph paths never appear in FFmpeg's argv;
- output only inside `<root>/.timelapse/exports/`, with `-n` (FFmpeg never overwrites) into a `.part` file, then a no-clobber rename;
- spawned from `crates/core` (`std::process::Command`), never from the WebView. No Tauri shell plugin, no `externalBin`, no capability change;
- each spawn site is marked `verify:allow` with a reference to this ADR, so `scripts/verify.sh` keeps catching any *other* subprocess.

**Reason:** ADR-008 and ADR-017 make FFmpeg a controlled subprocess. CLAUDE.md requires the user's approval and an ADR for anything that widens what the app can do, and this ADR is that record. The rules keep the surface to two known binaries and arguments we fully control.

## ADR-022 — FFmpeg build, provenance and notices (resolves ADR-017 open items a–c)

**Status:** Accepted (2026-11-14, approved by the user: CI build, 9.0.2 pin)

**(a) Provenance, version pin, build.**
- **Pinned sources:** FFmpeg **9.0.2** "Lei" (`ffmpeg-9.0.2.tar.xz`, sha256 `8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e`) and SVT-AV1 **v4.2.0** (GitLab archive, sha256 `c7b13c4a84bd3751aa35fcc72be13e6875467e7c2216879251a486e5b1e4e740`). v4.3.0 (2026-11-02) was too recent to pin. FFmpeg 9.0.2's `libsvtav1` wrapper supports the 4.x API.
- **Build:** `tools/build-ffmpeg/build.sh` downloads both tarballs, verifies the checksums and builds on the macOS CI runner (aarch64):
  - **SVT-AV1:** shared library only.
  - **FFmpeg:** **shared libraries**, because ffmpeg.org/legal.html asks for dynamic linking of an LGPL build. Configured *without* `--enable-gpl` / `--enable-nonfree`, with `--disable-autodetect` (so nothing on the build machine is picked up silently) and only the components we use: rawvideo input, the mp4/mov muxer and demuxer, `libsvtav1`, `h264_videotoolbox`, `hevc_videotoolbox`, swscale.
  - **Paths:** binaries and dylibs use `@rpath` so the bundle is relocatable.
- **Licence scan:** `tools/build-ffmpeg/check-licence.sh` fails the build if any of these hold:
  - `ffmpeg -L` does not report the GNU Lesser General Public License;
  - `-buildconf` contains `--enable-gpl` or `--enable-nonfree`;
  - any binary or dylib mentions or links x264/x265;
  - `libx264`/`libx265` encoders are present, or a required encoder is missing.

  This is ADR-017's build-time assertion.
- **Sources for redistribution:** CI uploads the exact source tarballs and the configure line next to the binaries. Publishing them alongside each release is a release-checklist item (LGPL-2.1 §6), flagged for the pending lawyer review.

**(b) Committed encoder path, no fallback.** At runtime the app probes the bundled FFmpeg (`-encoders`, `-L`, `-version`) and offers a tier only if its encoder is present. H.264/HEVC use `h264_videotoolbox` / `hevc_videotoolbox` with `-allow_sw 0`, so VideoToolbox may not fall back to software. A missing or failing encoder is an explicit error and is never substituted by another one. The app targets Apple Silicon (aarch64) only, where VideoToolbox H.264/HEVC is always present, so no OS-version table is needed. The probe still guards against an incomplete bundle or a virtualised runner without a media engine.

**(c) Notices.** The bundle carries `ffmpeg/LICENSES/`:
- FFmpeg's `COPYING.LGPLv2.1`;
- `CONFIGURE.txt` (the exact configure line and versions);
- `SOURCES.txt` (source URLs and sha256);
- SVT-AV1's `LICENSE.md` and `PATENTS.md`.

Settings → About names FFmpeg (LGPL-2.1-or-later) and SVT-AV1 (BSD-3-Clause-Clear, AOM Patent License 1.0) and says where those texts are.

**Licence exceptions:** FFmpeg (restricted LGPL) and SVT-AV1 (BSD-3-Clause-Clear + AOMPL-1.0, not allow-listed) are recorded in `licence-exceptions.yaml`, approved by the user on 2026-11-14.

## ADR-023 — Consolidation: copy or move into a picked folder, clones never links, undoable runs

**Status:** Accepted (2026-11-14, approved by the user: destination, operations, link policy, input, `libc`)

**What it does.** It gathers photographs from the collection's folders into one working folder (PRD §5.5, Pipeline §5). The flow is: preview → dry run → confirmation → execute → verify → history.

**Destination.** A folder the user picks with the existing folder dialog. It must be absolute and must not be the collection or lie inside it: the import rescans the collection, so copies there would be imported again. It may be outside the project folder (ADR-007 allows originals and working files elsewhere), and the core does the file work, so there is no capability change.

**Operations.**
- **Copy** (default). The originals stay where they are.
- **Move** (after a second confirmation). `images.current_path` records the new place. `original_path` stays the path of record and is never rewritten, and every reader already uses `COALESCE(current_path, original_path)`.

**Link policy.** A copy is made by **APFS `clonefile`** when the volume supports it: copy-on-write, so it is an independent file that only takes space once edited. Otherwise it is a **`create_new` byte copy** that keeps the modification time. Both fail when the target exists, so there is no check-then-act window. Moves into the working folder are placed the same way, before the old name is removed.
- **Hard links** are **never** used for the working collection: the "copy" would be the same file, so an edit in another app would change the original (ADR-006). Only the quarantine move (ADR-020) still uses one, inside the project folder, and only for the instant before the old name goes.
- **Symlinks** are never created: they dangle when the original moves, and editors write through them.

**`libc` 0.2** (MIT OR Apache-2.0, already in the lockfile through Tauri) becomes a direct dependency of the core for `clonefile` (macOS) and `statvfs` (free space, Unix).

**Input.** By default, every photograph except rejected and quarantined ones; alternatively, a saved sampling selection in its sequence order. Exact duplicates (same SHA-256) are kept once (the first in order), and the dry run lists the skipped ones. Photos without a hash are never treated as duplicates.

**Naming.**
- `preserve`: the relative path in the collection;
- `original`: flat, original filenames;
- `folder-prefix`: flat, `<parent folder>_<filename>`;
- `sequence`: flat, `000001.ext`… with at least 6 digits.

Names that collide within the plan, compared case-insensitively, get `_2`, `_3`… A file already at the destination:
- that is byte-identical is `already-present`. It is skipped, and an undo never touches it;
- otherwise, the next free suffix is used.

Every target is a relative path of normal components joined under the destination.

**Confirmation.** The dry run returns a plan fingerprint (SHA-256 of the ordered id/operation/target lines). Execution re-plans and refuses with `plan-changed` if the fingerprint differs, and with `insufficient-space` if the bytes needed exceed the free space (when it is known).

**Execution.** Execution runs under the process-wide file-operation lock (ADR-020), one run at a time, journaled before any file is touched. For each photo:
1. verify the source has the import hash;
2. journal a `pending` row (`job_id` = the run);
3. place the file without replacing anything;
4. verify the target hash;
5. for a move only: remove the old name (if it is in use, the placed file is removed instead), then complete the row and set `current_path` in one transaction.

Cancellation and progress happen between photos. A run ends `completed`, `partial` (some photos failed), `cancelled` or `failed`.

**Undo.** A run can be undone once.
- **Copy:** a copy we made is removed only while it still has the hash we wrote. `modified` and `missing` copies are left and reported.
- **Move:** a moved photo goes back only while `current_path` is still the run's target (`moved-since` otherwise), never overwriting.

Each removal or move-back is journaled (`remove-copy`, `move-back`). Afterwards, the directories left empty under the destination are removed; `remove_dir` cannot remove a non-empty one. The run's undo state is `undone` or `partially-undone`.

**Quarantine of a moved photo.** Quarantine now starts from the current location, and restore returns to where that quarantine started. `current_path` becomes `NULL` only when that place is `original_path`.

**Recovery** (Pipeline §12). Reconciliation, when a project opens while no operation runs, settles `pending` rows from the disk state:

| Operation | Disk state | Outcome |
| --- | --- | --- |
| `copy` | target has the journaled hash | completed |
| `copy` | target shorter than the source | removed, `interrupted` |
| `copy` | target absent | `interrupted` |
| `move` / `move-back` | as for quarantine (ADR-020) | as for quarantine (ADR-020) |
| `remove-copy` | target gone | completed |
| `remove-copy` | target still has the hash | `interrupted` |
| any | anything else | `inconsistent`; no file is touched |

Execution checks that the target is absent just before journaling, so a pending row whose target has the hash is ours. Only a file dropped in that exact place within that instant, followed by a crash, could defeat this, and that file's bytes would equal the original's.

**Checks.** "Originals immutable" end to end:
- `verify_originals` re-hashes every photograph at its location of record and reports it as `ok`, `changed`, `missing` or `unhashed`;
- `verify_run` re-hashes a run's copies.

No byte of an original is written; the only files ever removed are copies this app made, or the old name of a moved original once its new place has been verified.

**Out of scope:** multi-project merges, making a working folder the project's source, links, HEIC/RAW, free space on Windows (reported as unknown).

---

## Open decisions

The following decisions must be investigated and presented for approval before implementation makes them difficult to change.

### Technical dependencies

- **Resolved (ADR-018):** `rusqlite` 0.32 (bundled) for SQLite persistence; `sha2` 0.10 for content hashing in the import pipeline. See ADR-018.

- **Resolved (ADR-019):** `kamadak-exif` 0.5 for EXIF timestamp reads and `time` 0.3 (feature `formatting`) for date-string normalisation into ISO-8601; both on the licence allow-list, EXIF stamped-as-UTC assumption recorded with an explicit confidence field (see ADR-019). HEIC EXIF resolution is **not** in scope for Phase 2 and would be a separately approved effort.

- **Resolved (ADR-020):** image-processing library: `image` 0.25 (jpeg/png/tiff only) with `rayon` for parallel analysis.

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
