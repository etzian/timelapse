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
| 3 | Brightness analysis & cleaning | **done (2026-11-14)**: full gate green; independent review BLOCK → fixed → SIGN-OFF WITH NOTES (notes fixed); committed and pushed (see log) |

## Slice status (Phase 3)

| Slice | Status | Gate |
| --- | --- | --- |
| P3.1 Deps, ADR-020, scan exclusion | done | GATE-FAST + licence-audit green |
| P3.2 Luminance analysis | done | GATE-FAST green |
| P3.3 XMP reader | done | GATE-FAST green |
| P3.4 Store v3 + classification | done | GATE-FULL green |
| P3.5 Quarantine primitive | done | GATE-FAST green (see deviation below) |
| P3.6 Thumbnails | done | GATE-FAST green |
| P3.7 Commands + IPC | done | GATE-UI green |
| P3.8 Cleaning UI | done | GATE-UI green |
| P3.9 Exit tests + review | done | reviewer sign-off (2nd pass); GATE-FULL green |
| P3.10 Final gate + commit/push | done | GATE-FULL green |

## Handoff log

- **2026-11-14 — Phase 3 DONE.**
  - **Core:**
    - `crates/core/src/clean/`:
      - `analysis` (Rec. 709 luma, 256-bin histogram, ≤ 512 px, decode limits);
      - `xmp` (Lightroom rating, sidecar first, read-only);
      - `classify` (a pure function of the metrics, rating and thresholds);
      - `store` (measurements, proposals, `user_override` decisions, thresholds in `projects.settings_json`, review query);
      - `thumbs` (cached JPEG 256/1024);
      - facade (parallel analysis on ≤ 3 threads with progress).
    - `crates/core/src/files/`: the quarantine/restore journal.
    - Schema v3: `image_analysis`, `image_status`, `file_operations`, `images.current_path`.
    - The import scan skips `<root>/.timelapse/`. Two new error kinds (`image-undecodable`, `unknown-image`).
  - **Shell:** 10 thin commands. Analysis, moves and thumbnails are async on the blocking pool; analysis progress goes over a `Channel`; thumbnails are raw bytes shown as `data:` URLs (the CSP already allowed them). No capability change.
  - **UI:** a new Cleaning page (nav item) with:
    - thresholds;
    - an analysis run with progress;
    - filter tabs;
    - a thumbnail grid with metrics, reasons and keep/reject/undo;
    - an inspect modal with a histogram;
    - a confirm-before-move dialog;
    - restore;
    - all GUIDELINES §4 states.

    The project root is shared with Home in memory only. All strings are in en and fr-CA (264 keys at parity).
  - **Data map:** `cleaning-analysis`, `quarantine-journal`, `review-thumbnails`; legal docs regenerated.
  - **Deviation from the frozen P3.5 text:** P3.5 says the move is a `rename`. The review found that `std::fs::rename` replaces an existing destination on Windows and Unix, so the move is instead a hard link (or a `create_new` copy on FAT/exFAT), then verify, then source removal. This keeps the slice's intent (never overwrite, hash-verified, recoverable). Recorded in ADR-020; `PHASES.md` left untouched.
  - **Review:**
    - **First pass: BLOCK** on (1) reconcile running on every open and racing live moves, and (2) check-then-act over a clobbering rename. Both fixed, together with 12 notes:
      - process-wide file-op lock, with reconcile only when idle;
      - journal updates guarded by `status = 'pending'`;
      - per-item `record-failed`;
      - NULL-hash rule documented;
      - Clear on an unanalysed image;
      - quarantined images not re-analysed;
      - decode limits;
      - Svelte stale-root guards and failed-thumbnail state;
      - copy and fr-CA typography;
      - the `current_path` rename;
      - docs.
    - **Second pass: SIGN-OFF WITH NOTES.** Fixed as well:
      - a failed source removal never discards the copy unless the source provably remains;
      - reconcile refuses to delete when both paths resolve to the same entry;
      - a partial copy left by a crash is cleared.
  - **Flake found and fixed during the final gate:** concurrent first opens could fail because `PRAGMA journal_mode = WAL` ran before the busy timeout and can return SQLITE_BUSY without the handler. The busy timeout is now set first, with a bounded retry. Verified 30/30 on the race test and 5/5 on the full suite.
  - **Gate (Windows host):**
    - cargo fmt, clippy `-D warnings`, test (89);
    - svelte-check, vitest (12), vite build (from PowerShell);
    - `verify.sh` (Git Bash; one `verify:allow` on the XMP namespace literal in a test fixture, which is not a host);
    - `ez i18n-check` (264/264), `ez legal check` (ok, the two known lawyer-review warnings), `ez legal gen`, `ez licence-audit`.

    All exit 0.
  - **For the user:**
    - quarantine on FAT/exFAT drives uses the copy fallback (slower, still verified);
    - a Lightroom `.xmp` sidecar stays in the collection when its photo is quarantined;
    - Home's import report is not kept when you switch pages (the root is).
- **2026-11-14 — Phase 3 kicked off.** Phase 2 plan/state archived. The user approved: Phase 3 + the minimal Phase 6 primitive; `image` + `rayon`; a physical quarantine move into `.timelapse/quarantine/`; read-only Lightroom XMP.
- **Earlier phases:** see [archive/](./archive/).
