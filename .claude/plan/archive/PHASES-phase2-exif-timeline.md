# PHASES

Phase 0 and Phase 1 are **complete** and archived at [archive/PHASES-phase0-1-import-inventory.md](./archive/PHASES-phase0-1-import-inventory.md). This file tracks the milestone in work: **Phase 2 — EXIF & timeline**. The canonical scope entry and exit criteria for Phase 2 remain unchanged at [docs/DEVELOPMENT_ROADMAP.md#L46-L49](../../docs/DEVELOPMENT_ROADMAP.md), quoted below. Phases are **not** re-scoped in this file; the slices under "Slices" implement that scope and its exit criteria, and nothing more.

> ## Phase 2 — EXIF & timeline
>
> **Scope:** Timestamp/tz normalization, chronology table, the "distribution over a period" query used downstream. (Stage [`PROCESSING_PIPELINE.md` §3](../../docs/PROCESSING_PIPELINE.md).)
>
> **Entry:** Phase 1 exit. **(satisfied — see STATE.md)**
>
> **Exit:** For any imported set, `SELECT` produces a monotonic timeline with per-image capture time resolved to a single normalized epoch; edge cases (mixed tz, missing EXIF, burst offsets) covered by tests.

**Governing decisions for this phase:**

- [ADR-019](../../docs/DECISIONS.md#adr-019---phase-2-timestamping-kamadak-exif-for-exif-reads-and-a-time-based-normalizer) — `kamadak-exif` 0.5 (EXIF timestamp reads, JPEG/TIFF only) and `time` 0.3 (`formatting`; no IANA tzdb). Both MIT/Apache-2.0, on the licence allow-list; approved by the user.
- [ADRs-010 & 011](../../docs/DECISIONS.md#adr-010---timestamp-based-sampling) — chronological timestamps are authoritative when present; a flagged value is **not** silently corrected (camera clock drift stays visible as low confidence, never reconciled).
- Phase 2 is a **read-side** feature only: it resolves and persists *normalised* capture times for images already imported in Phase 1. It does **not** import new files, move or mutate originals, and adds no new Tauri capabilities (no plugin, no host access) — the four existing shell commands are reused, and Phase 2's four new commands open short-lived write connections for the backfill only per ADR-007.

---

## Slices

Work proceeds one slice at a time. A slice is *done* only when its gate (below) has actually been executed and passed — "implemented" never means "gated". `GATE-FAST` = `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (equivalent to the `check` target of [Makefile](../../Makefile), run directly because GNU `make` is not required on this Windows host). `GATE-UI` = everything in `GATE-FAST` plus `npm --prefix frontend run check|test|build`, `MSYS2 bash scripts/verify.sh`, and any new-string i18n parity (`ez i18n-check .`). **`GATE-FULL`** = a complete `make gate`: the above plus `ez legal check .`, `ez legal gen .`, and `ez licence-audit .`.

### P2.1 — Dependencies + ADR-019 wiring
Add `kamadak-exif = "0.5"` and `time = { version = "0.3", features = ["formatting"] }` to [crates/core/Cargo.toml](../../crates/core/Cargo.toml); `Cargo.lock` updates; confirm `deny.toml`'s existing licence policy accepts both (they should — see ADR-019) with no `licences/exceptions` entry added.

**Gate:** `GATE-FAST`.

### P2.2 — EXIF + filename + priority-resolution unit, with tests
New sub-tree under [crates/core/src/import/mod.rs](../../crates/core/src/import/mod.rs):

- **EXIF read** (`metadata/exif.rs`): `Option<String>` raw `DateTimeOriginal`, then fall back to `DateTimeDigitized` (same calendar format), JPEG/TIFF only — never assume a tag absent on one format exists on the other.
- **Filename date** (`metadata/filename.rs`): recognise common camera naming patterns in the bare filename (without extension) — `\d{8}` and `\d{14}` digit runs at a word boundary, optional `HHMM` / `HHMMSS` suffix — and parse them as `YYYY[-. ]MM[-. ]DD[ HH[:]MM[:SS]]?`, no timezone implied.
- **Priority + confidence** (`metadata/mod.rs`): EXIF → filename → filesystem `modified_at` (already captured at import time in Phase 1); each source emits an explicit `TimestampSource` and `Confidence` (high / medium / low), never a silent default.

Unit tests must cover: a real JPEG with valid `Exif::DateTimeOriginal`; a file with no EXIF at all; a filename that parses to a different day than the EXIF value (filename must lose — EXIF priority); a burst of 5+ files whose EXIF seconds are non-monotonic but sub-second apart (confidence recorded, no reordering attempted); and a mixed-source folder.

**Gate:** `GATE-FAST`.

### P2.3 — Resolution + normalisation to ISO-8601 (UTC-assumed), with tests
`metadata/resolver.rs`: given the EXIF / filename / filesystem candidate values collected per file in P2.2, pick the winning value and its source per the priority above, then normalise it with `time` 0.3 into an explicit-format UTC ISO-8601 string (no tzdb lookup — we never resolve a user's local zone in Phase 2; [ADR-019](../../docs/DECISIONS.md)). The same resolver must handle the "mixed source" case: EXIF on some images, filename-on-others, filesystem-only on the rest — and the mixed output still sorts monotonically by resolved value.

**Gate:** `GATE-FAST`.

### P2.4 — Store backfill + timeline / distribution / gap queries, migration v2
[crates/core/src/import/store.rs](../../crates/core/src/import/store.rs):
1. Migration v2 (via `PRAGMA user_version`, not a new `schema_migrations` table) that adds the index required for the chronology query below (`CREATE INDEX IF NOT EXISTS idx_images_normalized_datetime ON images(normalized_datetime)`).
2. `backfill_timestamps(conn, &candidates)`: one transaction; for each row it can resolve, writes `exif_datetime_original`, `filename_datetime`, `filesystem_datetime`, `normalized_datetime`, `timestamp_source`, `timestamp_confidence`; idempotent — an already-populated row (from a prior run) is left untouched.
3. The three read queries the exit criterion and Phase 3/4/7 depend on:
   - **Monotonic timeline**: rows ordered by `normalized_datetime` (then `original_path`) with source + confidence carried through; images where resolution failed are still returned, marked absent, so callers can see the gap instead of silently dropping them.
   - **Distribution / bucketing** over a period (`strftime()` on `normalized_datetime` at hour/day/month granularity — the "distribution over a period" query named in the Phase 2 scope).
   - **Gaps**: interval boundaries where the gap between consecutive resolved timestamps is unusually large; reported as data, never used to *correct* adjacent values (ADR-010 / ADR-011 — camera-clock drift must stay visible, not smoothed over).

**Gate:** `GATE-FULL`.

### P2.5 — Tauri shell commands + frontend IPC wrappers
[src-tauri/src/lib.rs](../../src-tauri/src/lib.rs): four new thin commands (`backfill_metadata`, `get_timeline`, `timeline_distribution`, `detect_timeline_gaps`) mapping to the core functions in P2.4, using the same `root: PathBuf` parameter pattern and the existing `IpcError { kind, args }` shape — **no prose**, no new capabilities, no shell-side logic (shell stays a thin adapter per CLAUDE.md). [frontend/src/lib/ipc/app.ts](../../frontend/src/lib/ipc/app.ts): matching TS types + wrappers for all four.

**Gate:** `GATE-UI`.

### P2.6 — Timeline UI in Svelte (both locales)
[Home.svelte](../../frontend/src/lib/pages/Home.svelte): after a successful import (+ its integrity snapshot), surface the backfill result (how many images had to resolve to which source, how many are low-confidence / unresolved) and render a timeline section: monotonic list with per-row source + confidence badge, the distribution-over-period bar view, and the flagged gap boundaries. **Every new user-visible string goes into both [locales/fr-CA/app.ftl](../../locales/fr-CA/app.ftl) and [locales/en/app.ftl](../../locales/en/app.ftl)** (never inlined), and any *new* `Error::kind()` introduced by P2.2–P2.4 gets a matching `error-<kind>` key in both catalogs.

**Gate:** `GATE-UI`.

### P2.7 — Exit-criterion tests + independent verification pass
Add integration tests in [crates/core/src/import/mod.rs](../../crates/core/src/import/mod.rs)'s test module that exercise the *full* path on a real temp folder with real `.jpg` bytes: import → backfill → timeline / distribution / gap queries, covering **all three** named exit conditions — resolved-and-monotonic output under mixed sources, missing-EXIF fallback actually used (not silently dropped), burst/edge offsets recorded without silent reconciliation. Then a full [reviewer](../../.opencode/agents) pass over the diffs: licence scope, i18n parity across both catalogs (no key missing on either side), error-kind strings vs. fluent keys, `crates/core` still free of `tauri`, and [DATABASE_SCHEMA.md](../../docs/DATABASE_SCHEMA.md) updated to document the v2 index + the timeline/distribution/gap query shapes if they are new.

**Gate:** reviewer sign-off recorded in STATE.md; `GATE-FULL`.

### P2.8 — Final gate, status record, commit, push
`make gate` (all five targets: `check`, `ui-check`, `i18n`, `legal`, `licence`) run in order and green on this host. Update [STATE.md](./STATE.md) handoff log + the phase table; [docs/CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md) if it names Phase 2 as "next"; confirm [docs/DEVELOPMENT_ROADMAP.md](../../docs/DEVELOPMENT_ROADMAP.md)'s Phase 2 entry and exit criteria are *met as written* (do not edit the roadmap to fit the implementation). Commit (Conventional Commits, one logical change, staged paths only) and push.

**Gate:** `GATE-FULL` green + commit & push to `origin/main`.

---

## Out of scope for Phase 2 (explicitly)
- HEIC/HEIX reading (`kamadak-exif`'s TIFF-based segment model does not cover HEIC's container; flagged as a known limitation in ADR-019 — if HEIC import lands later it needs its own, separately-approved resolution pass).
- Actual time-zone resolution for user-local clocks (the UTC-assumed + confidence-flagging model in ADR-019 is deliberately chosen; named-zone conversion is a future, explicitly-scoped decision, not silently inferred here).
- Any file moves/copies/quarantine/rename of originals — that belongs to Phase 3/6 territory and its own safety model (ADR-011), not the timeline's read-side queries.
