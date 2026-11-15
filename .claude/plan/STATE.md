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
| 5 | FFmpeg video generation | **in progress**: ADR-021 (subprocess), licence drafts, CI build/test and the 9.0.2 pin approved by the user 2026-11-14; slices in [PHASES.md](./PHASES.md) |

## Slice status (Phase 5)

| Slice | Status | Gate |
| --- | --- | --- |
| P5.0 Prerequisites, ADR-021/022, licence drafts | in progress | — |
| P5.1 Build script + CI job | pending | — |
| P5.2 Core export engine | pending | — |
| P5.3 Commands + resources | pending | — |
| P5.4 Export page + About notice | pending | — |
| P5.5 Exit tests + review | pending | — |
| P5.6 Final gate + commit/push | pending | — |

## Handoff log

- **2026-11-14 — Phase 5 kicked off.**
  - **Archive:** Phase 4 plan/state archived.
  - **Verified:**
    - SVT-AV1 is BSD-3-Clause-Clear + AOMPL-1.0, not on the allow-list, so ADR-017 is wrong on that point;
    - ffmpeg.org/legal.html requires dynamic linking, exact sources and the configure line;
    - FFmpeg 9.0.2's libsvtav1 wrapper supports the SVT-AV1 4.x API.
  - **Pinned:** the tarballs were downloaded only to hash them.
  - **No FFmpeg on this host;** the app target is macOS aarch64 only.
- **Earlier phases:** see [archive/](./archive/).
