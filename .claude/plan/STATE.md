# STATE

STATUS: RUNNING
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
| 6 | Consolidation & file management | **in progress**: destination, copy+move, clone policy, input approved by the user 2026-11-14; slices in [PHASES.md](./PHASES.md) |

## Slice status (Phase 6)

| Slice | Status | Gate |
| --- | --- | --- |
| P6.0 ADR-023, plan | in progress | — |
| P6.1 Plan (pure) | pending | — |
| P6.2 Placement + journal + store v6 | pending | — |
| P6.3 Consolidation engine | pending | — |
| P6.4 Commands | pending | — |
| P6.5 Organise page | pending | — |
| P6.6 Exit tests + review | pending | — |
| P6.7 Final gate + commit/push | pending | — |

## Handoff log

- **2026-11-14 — Phase 6 kicked off.** Phase 5 plan/state archived. User decisions: a picked destination outside the collection; copy (default) and move, each undoable; APFS clone else byte copy, never links (`libc` 0.2 approved as a direct dependency); input = all non-rejected/non-quarantined or a saved selection, duplicates once. Branch `phase-6-consolidation` (CI runs the macOS clone path).
- **Earlier phases:** see [archive/](./archive/).
