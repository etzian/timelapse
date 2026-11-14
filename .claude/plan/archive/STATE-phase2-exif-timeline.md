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
| 1 | First feature (Import) | done (2026-11-12) — full `make gate` green, committed `421f2d0`, pushed to `origin/main` |
| 2 | EXIF & timeline | **done (2026-11-14)** — full `make gate` green, reviewer sign-off (with notes, all actionable ones fixed); committed and pushed (see handoff log) |

## Slice status (Phase 2)

| Slice | Status | Gate run this slice |
| --- | --- | --- |
| P2.1 Deps + ADR wiring | done | GATE-FAST green |
| P2.2 EXIF + filename + priority unit & tests | done | GATE-FAST green |
| P2.3 Resolution + UTC-normalisation & tests | done | GATE-FAST green |
| P2.4 Store backfill + 3 queries, migration v2 | done | GATE-FULL green |
| P2.5 Tauri commands + IPC wrappers | done | GATE-UI green |
| P2.6 Timeline UI, both locales | done | GATE-UI green |
| P2.7 Exit-criterion integration tests + reviewer pass | done | reviewer: SIGN-OFF WITH NOTES; GATE-FULL green after fixes |
| P2.8 Final gate + status record + commit/push | done | GATE-FULL green |

## Handoff log

- **2026-11-14 — Phase 2 DONE.**
  - **Recovered start state:** the 2026-11-12 session left `crates/core/src/import/metadata/` as non-compiling drafts (not wired into the crate; invalid syntax in `filename.rs`, a non-existent `test_support` module). Rewritten from scratch.
  - **Core** (`crates/core/src/import/`): `metadata/{mod,exif,filename,resolver}.rs` — EXIF `DateTimeOriginal` / `DateTimeDigitized` with `SubSecTime*` and `OffsetTime*` (JPEG/TIFF only), filename dates (compact and separated shapes, boundary-anchored, calendar-validated), priority EXIF original (high) → EXIF digitized (medium) → filename (medium) → filesystem (low); normalised to fixed-width UTC `YYYY-MM-DDTHH:MM:SS.mmmZ`; explicit offsets applied, otherwise UTC assumed and recorded. `metadata/fixture.rs` (test only) builds real decodable JPEG/TIFF bytes with controlled EXIF. `store.rs`: migrations on `PRAGMA user_version` (v1 = Phase 1 schema, v2 = `idx_images_normalized_datetime` + `images.timestamp_offset_minutes`), `BEGIN IMMEDIATE` + re-read for concurrent first opens; idempotent `backfill_timestamps`; `timeline`, `distribution` (hour/day/month, optional `[from, to)`), `gaps` (`max(min_seconds, factor × median)`, `source_changed` flag, options clamped); `timeline_summary`. Facade: `backfill_for_root`, `timeline_for_root`, `distribution_for_root`, `gaps_for_root`.
  - **Scope note:** the plan's migration v2 named only the index; `timestamp_offset_minutes` was added in the same step because the exit criterion's "mixed tz" case and DATABASE_SCHEMA.md ("preserve … timezone uncertainty") need the zone basis persisted. Documented in ADR-019 and DATABASE_SCHEMA.md §6a.
  - **ADR-019 correction:** `kamadak-exif` is BSD-2-Clause (not MIT/Apache-2.0); still on the allow-list. `time` declared with default features (the `formatting` feature was unnecessary).
  - **Shell:** `backfill_metadata`, `get_timeline`, `timeline_distribution`, `detect_timeline_gaps` — thin adapters, no new capability.
  - **Frontend:** `ipc/app.ts` types + wrappers; `lib/timeline.ts` (+ tests); `pages/TimelineSection.svelte` (summary stats, per-source counts, distribution bars with hour/day/month tabs, gaps, paged chronological list with source / confidence / zone badges, loading / empty / error-with-retry states); `Home.svelte` runs the backfill after import. 37 new keys in both `app.ftl` catalogs; no new error kinds.
  - **Data map:** new `capture-timeline` item; `ez legal gen .` regenerated `legal/generated/**`.
  - **Exit criterion:** `crates/core/src/import/mod.rs` `timeline_exit_tests` — import → backfill → queries on real files: mixed sources and zones (monotonic, +05:00 offset vs assumed UTC), missing-EXIF fallback with originals preserved, the `0000:00:00` placeholder falling through, unresolved images kept, burst sub-seconds kept exactly (including a tie), idempotency, distribution and gaps. 52 Rust tests, 7 vitest.
  - **Reviewer pass (independent agent):** SIGN-OFF WITH NOTES. Fixed: Import re-clickable during backfill (+ stale-root result), concurrent-first-open migration race, gap-option overflow, hour bucket keys not usable as bounds, fr-CA non-breaking space before `:`, 12/14-digit names with an impossible time losing the date, gap defaults moved into core.
  - **Decided by the user (2026-11-14):** (1) Mixed ordering accepted: naive EXIF/filename times stay assumed-UTC while filesystem times are true instants, so in a non-UTC locale they may interleave by the local offset; disclosed via `tz_assumed` / `source_changed` and the UI note. Phase 3+ may rely on the timeline order as is. (2) A row is never re-dated once resolved, even if its original is later edited or replaced: the first-read (original) EXIF data is what is wanted. Recorded in ADR-019.
  - **Gate (this host, Windows):** run step by step (no GNU make): cargo fmt/clippy `-D warnings`/test, `npm --prefix frontend run check|test|build` (from PowerShell; under Git Bash vitest fails on `d:` vs `D:` drive-letter casing — environmental, pre-existing tests fail identically), `bash scripts/verify.sh` (Git Bash), `ez i18n-check .` (fr-CA 174/174), `ez legal check .` (ok, the two known lawyer-review warnings), `ez legal gen .`, `ez licence-audit .` — all exit 0.
- **2026-11-12 — Phase 2 kicked off.** Phase 0 & 1 archive moved into `archive/`. ADR-019 written; 8 slices defined in `PHASES.md`.
- **Phase 0 & 1 history:** see [archive/STATE-phase0-1-import-inventory.md](./archive/STATE-phase0-1-import-inventory.md).
