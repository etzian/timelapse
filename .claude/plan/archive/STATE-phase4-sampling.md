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
| 4 | Temporal sampling | **done (2026-11-14)**: full gate green; independent review BLOCK → fixed → SIGN-OFF WITH NOTES (notes fixed); committed and pushed |

## Slice status (Phase 4)

| Slice | Status | Gate |
| --- | --- | --- |
| P4.1 Selection algorithms | done | GATE-FAST green |
| P4.2 Store v4 + runs | done | GATE-FULL green |
| P4.3 Commands + IPC | done | GATE-UI green |
| P4.4 Sampling page | done | GATE-UI green |
| P4.5 Exit tests + review | done | reviewer sign-off (2nd pass); GATE-FULL green |
| P4.6 Final gate + commit/push | done | GATE-FULL green |

## Handoff log

- **2026-11-14 — Phase 4 DONE.**
  - **Core** (`crates/core/src/sampling/`):
    - `select.rs`: the pure algorithms, `ALGORITHM_VERSION` 1.
      - Fixed interval: integer-millisecond targets; closest / first-after / last-before. The default reach is half an interval for closest and strictly under one interval otherwise; an explicit tolerance is inclusive.
      - Every Nth, with an offset.
      - Uniform: exactly N photographs when N ≤ the available photographs; every photograph on spread targets when N is larger.
      - Selection rules: no duplicates, picks stay in chronological order, ties go to the earlier frame, at most 5 000 000 targets.
    - `store.rs`:
      - input rules: everything except rejected and quarantined photographs, flagged ones optional; undated photographs and those outside the period are excluded and counted;
      - a SHA-256 input fingerprint;
      - `preview`: the first 200 items plus day counts over the whole selection;
      - `create_run`: one transaction;
      - `list_runs`, and `run_page` (SQL `LIMIT/OFFSET` and `GROUP BY` day counts);
      - `verify_run`: re-runs the stored configuration on the current input.
    - Schema v4: `sampling_runs`, `sampling_items`. Two new error kinds (`invalid-sampling` with a reason, `unknown-run`).
  - **Shell:** 5 thin commands (preview, create and verify are async on the blocking pool). No capability or dependency change.
  - **UI:** a new Sampling page (nav item) with:
    - the method tabs and their parameters;
    - an optional period (the "to" date is inclusive in the UI and becomes the exclusive next-day bound in the configuration);
    - the include-flagged toggle;
    - preview stats: input, selected, reduction, average interval, unmatched steps;
    - an estimated length from FPS × playback multiplier (kept separate, PRD §5.7);
    - the excluded breakdown, day distribution and thumbnail strip;
    - saving exactly the previewed configuration;
    - saved runs with Show (paged) and Check (reproducibility);
    - all GUIDELINES §4 states.

    All strings are in en and fr-CA (340 keys at parity).
  - **Data map:** `sampling-runs`; legal docs regenerated. DATABASE_SCHEMA.md documents v4 and the input rule.
  - **Review:**
    - **First pass: BLOCK**, on:
      - H1: the floating-point target count dropped the last step (2 504 misses in a sweep);
      - H2: the first-after default tolerance let a step take the next step's exact photograph.

      Both fixed, together with:
      - M1: uniform returning fewer than N;
      - M2: the preview sending every item over IPC;
      - L1–L4: copy and fr-CA wording, run labels, resetting state on a project change;
      - L6: more real-file exit tests (a tie, an unmatched step, rejection detected by verify, paging).
    - **Second pass: SIGN-OFF WITH NOTES**, with the reviewer's fuzzer at 60 000 sets × 3 methods. Both notes are fixed too: uniform with N > photographs now uses every photograph, and run pages are paged in SQL. L5 (verify does not compare the counts, which follow from the inputs, configuration and algorithm) was accepted.
  - **Gate (Windows host):** cargo fmt / clippy `-D warnings` / test (111); svelte-check / vitest (16) / vite build; `verify.sh`; `ez i18n-check` (340/340); `ez legal check` (the two known lawyer-review warnings); `ez legal gen`; `ez licence-audit`. All exit 0.
  - **For the user:** the period dates and the strip times are in UTC, as on the timeline.
- **2026-11-14 — Phase 4 kicked off.** Phase 3 plan/state archived. User answers: the pipeline/PRD strategies (fixed interval, every Nth, uniform); input = all but rejected/quarantined (flagged optional); the output is an ordered list in SQLite; preview = stats + timeline + thumbnail strip.
- **Earlier phases:** see [archive/](./archive/).
