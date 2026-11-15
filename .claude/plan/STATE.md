# STATE

STATUS: DONE
<!-- PENDING | RUNNING | DONE | BLOCKED -->

## Contract

- Repo root: `.` (`~/etzian/timelapse`). UI/UX contract: `../_uidesign/GUIDELINES.md`, **read-only**.
- Product: **Timelapse Studio**, `com.etzian.timelapse`. Turn large photo collections into high-quality timelapse videos, locally.
- Stack: Rust workspace (`crates/core` pure, `src-tauri` shell), Svelte 5 + TS on `@etzianco/ui`, Fluent (fr-CA + en).
- Verification: `make gate`.
- Invariants: permissive licences only; strings in both locales; `crates/core` free of tauri; data map updated with any change to what is stored or sent; narrow capabilities.

## Phases

| # | Phase | Status |
| --- | --- | --- |
| 0 | Baseline | done (2026-11-10) |
| 1 | Import & inventory | done (2026-11-12), `421f2d0` |
| 2 | EXIF & timeline | done (2026-11-14), `7528260` + `2aeab8d` |
| 3 | Brightness analysis & cleaning | done (2026-11-14), `e5318e7` |
| 4 | Temporal sampling | done (2026-11-14), `60b554f` |
| 5 | FFmpeg video generation | **done (2026-11-15)**, exit criterion proven on the macOS CI runner (run 43664224239). Review BLOCK → fixed → SIGN-OFF WITH NOTES. ffmpeg + svt-av1 licence exceptions approved by the user 2026-11-14 |
| 6 | Consolidation & file management | **done (2026-11-15)**, exit criterion proven by the consolidation exit tests on Windows and the macOS CI runner (run 43679518712, APFS clone path). Review BLOCK → fixed → SIGN-OFF WITH NOTES → notes A–C fixed |
| 7 | Advanced UI & contact sheets | **done (2026-11-15)**, exit criterion proven by the contact-sheet exit tests on Windows and the macOS CI runner (run 43713955977). Review SIGN-OFF WITH NOTES → defects 1–7 fixed |
| 8 | Flicker reduction & stabilization | **done (2026-11-15)**, exit criterion proven by the adjust and export exit tests on Windows and the macOS CI runner (run 43733756772, adjusted AV1 encode included). Review SIGN-OFF WITH NOTES → fixed. **User item open:** real GoPro check |

## Slice status (Phase 8)

| Slice | Status | Gate |
| --- | --- | --- |
| P8.0 ADR-025, plan | done | GATE-FAST |
| P8.1 Deflicker (pure) | done | GATE-FAST |
| P8.2 Stabilization (pure) | done | GATE-FAST |
| P8.3 Runs, store v8, analysis | done | GATE-FAST |
| P8.4 Encode integration | done | GATE-FAST |
| P8.5 Commands | done | GATE-UI |
| P8.6 UI | done | GATE-UI + legal check/gen |
| P8.7 Exit tests + review | done | reviewer sign-off; GATE-FULL; CI green |
| P8.8 Final gate + commit/push | done (user item open) | GATE-FULL; user item: real GoPro check |

## Handoff log

- **2026-11-15 — Phase 8 DONE** (one item is the user's).
  - **ADR-025:** deflicker (luminance + colour, against a trend of running median then mean in log space; exposure capped at ±1 stop, colour at ±⅓ stop and renormalised to keep luminance) and translation-only stabilization (block matching on 512 px normalised grayscale, a 4-level pyramid, sub-pixel fit, confidence, cuts ignored; the smoothed path; one crop of at most 5% per side). Series are extended at both ends along a robust line, so the first and last frames are corrected too. No new dependency; vidstab (GPL) excluded.
  - **Core** (`crates/core/src/adjust/`): `deflicker.rs`, `stabilize.rs`, `series.rs` (pure); `mod.rs` (analyses of a saved selection from the cached review thumbnails, in parallel batches, with progress, cancel and a lock); `store.rs` (schema v8: `flicker_runs`/`flicker_frames`, `stabilization_runs`/`stabilization_frames`, each run written whole in one transaction); `preview.rs` (downsampled chart series, before/after stills).
  - **Encode (D5):** `ExportSettings.flicker_run` / `stabilization_run`. Values are read from SQLite and checked: `adjustment-mismatch` for another selection or one with a missing frame; `adjustment-stale` when the selection's sequence fingerprint changed since. The shift and crop apply before EXIF orientation, the LUTs after fitting. There is no manifest; the runs used are in `settings_json` and the export record.
  - **Shell:** 7 commands. The spec's single `adjustment_series` became `flicker_series` + `stabilization_series`.
  - **UI:** the Adjust page with analyses, figures, brightness/colour/path charts (LineChart: one axis, legend + direct labels, crosshair as a slider, table view; colours derived from design-system hues and validated by the dataviz palette check in light and dark) and before/after stills; on the Export page, run selects. 669 keys in both locales. Data map: `adjustments`.
  - **Exit criterion:**
    - identical configuration gives identical stored values (deflicker and stabilization);
    - flicker is reduced;
    - shake is recovered within 0.3 px;
    - prepared frames are steadier and more even;
    - another selection's runs and stale runs are refused;
    - an AV1 export with both runs completes on the macOS runner and leaves only the video (no manifest);
    - cancel stores nothing.
  - **Review:** SIGN-OFF WITH NOTES. Fixed: uncorrected sequence ends, view resets on refresh, chart text outside Fluent, slider value before hover, the stale fingerprint, per-frame preview cost, colour gains moving luminance, the measured count.
  - **Notes left:**
    - `app.css` carries its `verify:allow` exemptions in a dummy `--app-why` property, because verify.sh blanks CSS comments; extending verify.sh (a template file) to accept relative colour from `--ez-*` would be cleaner;
    - no test with an EXIF-rotated source plus a shift (correct by construction);
    - `shift_crop` copies the full-size image once per frame.
  - **User item:** check deflicker and stabilization visually on a real GoPro sequence on the Mac (PRD §5.9).
- **2026-11-15 — Phase 8 kicked off.** Phase 7 plan/state archived. Since Phase 7, on main: import skips hidden files (`._*`), large imports run off the main thread with a 6 h limit, dev builds optimise dependencies, bulk keep/reject, a Stop button for the analysis, Sampling dates default to the photographs' span. User decisions: translation-only stabilization in Rust (no new dependency; vidstab is GPL); deflicker on luminance and colour; the user's real-GoPro visual check is a phase item.
- **2026-11-15 — Phase 7 DONE.**
  - **ADR-024:** contact sheets are a streamed PDF (`pdf-writer` 0.15, MIT OR Apache-2.0, approved) with the standard Helvetica font in WinAnsi (no bundled font; other scripts print `?`), written to `.timelapse/contact-sheets/<id>.pdf` with a history.
  - **Core** (`crates/core/src/contact_sheet/`):
    - `layout.rs` / `text.rs` / `pdf.rs` (pure): grid geometry, WinAnsi text and Helvetica widths, one object at a time, identical bytes for identical input;
    - `store.rs`: schema v7 `contact_sheets`; the six sources (all, period, flagged, quarantined, sampling, export) as SQL only;
    - `mod.rs`: preview, generate (lock, cancel token, temporary name, placeholder cells for unreadable photos, `source-changed` when the count shown no longer holds), settle of crashed rows.
  - **Also:** `export::record` (derived export record, ADR-015, shown and saved as `<id>.record.json`); week buckets, `timeline_period`, `identical_timestamps`.
  - **Shell:** 8 commands; the spec's `contact_sheet_count` and `contact_sheet_preview` are one command, `preview_contact_sheet`.
  - **UI:** the Contact sheets page (live first-page preview), shortcuts from Cleaning, Sampling, Export and a timeline period, timeline zoom with a breadcrumb, period thumbnails and identical times, the export record on the Export page. A small nav store holds the page shown. 578 keys in both locales. Data map: `contact-sheets`, and `video-exports` mentions the saved record.
  - **Exit criterion:** the contact-sheet exit tests check every source's membership and order from SQLite; identical bytes after the originals are removed (cached thumbnails); placeholders; layout and captions; fr-CA and en labels; cancel, empty, busy, crash; the preview. They pass on Windows and on the macOS CI runner (run 43713955977, 182 passed).
  - **Review:** SIGN-OFF WITH NOTES, no blocker. Fixed: duplicate "Show more" pages, unsequenced distribution answers, another project's running sheet on this page, plurals, the printed count, open-error mapping, a DB error after the rename; nits (JPEG frame types, footer fit, record error copy, blocking commands, aria-pressed).
  - **Notes accepted, for later:**
    - thumbnails (and so sheets) ignore EXIF orientation: portrait photos print sideways (existing Phase 3 behaviour);
    - preview captions are hover-only; `preview` loads every item to count;
    - zoomed into a month, the Week tab shows clipped edge weeks;
    - two app instances on one project can settle each other's running sheet (no data loss; same as exports).
  - **Not verified by eye:** no PDF renderer was available in this session; the PDF is checked structurally and textually by tests.
- **2026-11-15 — Phase 7 kicked off.** Phase 6 plan/state archived. Before it, the Phase 6 follow-ups landed on main: `unicode-normalization` 0.1 approved and used for name comparison (`8740bbb`), the exFAT check added to the release rules (`e5c003c`). User decisions: PDF only with `pdf-writer` 0.15 (MIT OR Apache-2.0) and the built-in Helvetica (no bundled font); sheets go to `.timelapse/contact-sheets/` with a history; all six PRD sources.
- **2026-11-15 — Phase 6 DONE.**
  - **ADR-023:** consolidation into a picked folder outside the collection; copy (default) or move; APFS clone, else a byte copy published under a temporary name without replacing; never a link. `libc` 0.2 is a direct dependency (MIT OR Apache-2.0, approved).
  - **Core** (`crates/core/src/consolidate/`):
    - `plan.rs` (pure): `preserve` / `original` / `folder-prefix` / `sequence` naming, case-insensitive `_N` suffixes, duplicates once, `already-present` skip, unsafe names refused, plan fingerprint;
    - `mod.rs`: preview (dry run, free space), execute (fingerprint must match, under the file lock, journaled, cancellable), undo, `verify_run`, `verify_originals`;
    - `store.rs`: `consolidation_runs`.
  - **`files/`:**
    - placement through the journal for `copy` / `move` / `move-back` / `remove-copy`;
    - quarantine starts from the current location and restore returns there;
    - reconcile never removes or claims a file in a user folder; a row whose volume is unmounted stays pending.
  - **Schema v6:** `consolidation_runs`, and `file_operations` rebuilt with its rows kept (new operation types, `identity`). The timestamp backfill now reads the current location.
  - **Shell:** 7 commands. **UI:** the Organize page with dry run, confirmation (a second one for move), progress and cancel, history with check and undo, and the originals check. Strings are in both locales (487 keys). Data map: `consolidations`.
  - **Exit criterion:** the 14 consolidation exit tests check:
    - copy and move round trips;
    - edit independence (inode on Unix);
    - no overwrite;
    - a modified copy survives undo;
    - every reconcile state;
    - quarantine of a moved photo;
    - a tampered original is reported;
    - an old undo after a later move;
    - a replaced copy;
    - only the folders the run created are removed.

    They pass on Windows and on the macOS CI runner (run 43679518712).
  - **Review:**
    - **Pass 1: BLOCK.** An old copy run's undo could delete originals that a later move put at the same paths. Reconcile could also delete files in a user folder.
    - **Fixes:** location-of-record and identity checks, temporary-name publishing, settle under the lock, only the run's own folders removed.
    - **Pass 2: SIGN-OFF WITH NOTES.** Fixed since: A (undo was quadratic; now a set built once), B (restore targets treated as user locations), C (identity check across projects).
    - **Notes accepted:**
      - (D) v6 was edited in place before any release; recreate any development database made from `aa04fd8` or `d06e715`;
      - (E) leftovers after a crash (`unrecorded-copy`, an identical target after an interrupted move) are reported and left, never undone;
      - NFC filename normalisation needs a dependency; a clash only causes a `destination-exists` failure, never an overwrite.
  - **For the user:**
    1. Optional: approve a Unicode-normalisation dependency (e.g. `unicode-normalization`, MIT OR Apache-2.0) so that é-in-NFD and é-in-NFC names are planned as one.
    2. The real-Mac check of the release gate now also covers consolidation onto an external exFAT drive (the in-place fallback).
- **2026-11-14 — Phase 6 kicked off.** Phase 5 plan/state archived. User decisions: a picked destination outside the collection; copy (default) and move, each undoable; APFS clone else byte copy, never links (`libc` 0.2 approved as a direct dependency); input = all non-rejected/non-quarantined or a saved selection, duplicates once. Branch `phase-6-consolidation` (CI runs the macOS clone path).
- **Earlier phases:** see [archive/](./archive/).
