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

## Slice status (Phase 6)

| Slice | Status | Gate |
| --- | --- | --- |
| P6.0 ADR-023, plan | done | GATE-FAST |
| P6.1 Plan (pure) | done | GATE-FAST |
| P6.2 Placement + journal + store v6 | done | GATE-FAST; macOS CI (clonefile, renamex_np) |
| P6.3 Consolidation engine | done | GATE-FAST |
| P6.4 Commands | done | GATE-UI |
| P6.5 Organise page | done | GATE-UI + legal check/gen |
| P6.6 Exit tests + review | done | reviewer sign-off (2nd pass) + notes fixed; GATE-FULL; CI green |
| P6.7 Final gate + commit/push | done | GATE-FULL; merged to main |

## Handoff log

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
