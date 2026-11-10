# Timelapse Studio

## Development Roadmap (Canonical)

**Status:** Canonical planning document (approved 2026-11-09). Supersedes the inline Phase 0–8 table formerly at `CURRENT_STATUS.md` §5, which now points here as the status of record. Phases and order are unchanged from `CURRENT_STATUS.md`; what this adds is per-phase scope, entry/exit criteria, and explicit dependency ordering so no phase starts on unresolved upstream state.
**Governing docs:** `PRD.md`, `TDD.md`, `ARCHITECTURE.md`, `DATABASE_SCHEMA.md`, `PROCESSING_PIPELINE.md`, `DECISIONS.md`. This roadmap does not change any of those; it sequences work against them.

**Global gate:** Per `CURRENT_STATUS.md` §8 (Implementation restriction) — do **not** begin application implementation until the Phase 0 review has been presented and approved. Every phase below is downstream of that gate.

---

## Dependency graph (read first)

```
Phase 0 Foundation ──────────────┬─► Phase 1 Import/Inventory
                                 │        │
                                 │        ├─► Phase 2 EXIF/Timeline
                                 │        │        │
                                 │        │        ├─► Phase 3 Brightness/Clean
                                 │        │        │        │
                                 │        │        │        ├─► Phase 4 Temporal Sampling
                                 │        │        │        │        │
                                 │        │        │        │        ├─► Phase 5 Video Generation (FFmpeg/AV1)
                                 │        │        │        │        │        │
                                 │        │        │        │        └─► Phase 7 Advanced UI + Contact Sheets
                                 │        │        │        └──────────► Phase 8 Flicker + Stabilization
                                 │        └────────────────────────────► (Phase 6 Consolidation/File Mgmt feeds Phases 3–9)
                                 └─────────────────────────────────────► (Phase 5 blocked until ADR-017 / FFmpeg-AV1 resolved)
```

Read: Phases 2→4 are strictly sequential (each consumes the prior stage's SQLite output per `PROCESSING_PIPELINE.md`). Phase 6 is a cross-cutting safety concern that must exist before any phase that moves/deletes files. Phase 5 is *explicitly* gated on the FFmpeg/AV1 decision. Phase 8's flicker-reduction gains are stored in SQLite (`flicker_runs`, D5), feeding Phase 5's export input.

---

## Phase 0 — Foundation & technical validation
**Scope:** Repository scaffold (Rust workspace + Svelte 5 shell, Tauri 2); CI; i18n skeleton (Fluent, fr-CA + en per ADR-016); licence gate (`ez licence-audit` clean on all *shipped* deps); wire in the adopted ADR-015 export-record approach and resolve the remaining open D2 (FFmpeg/AV1 → candidate ADR-017); confirm Apple signing inputs (`etzian.yaml:66,68`).
**Entry:** Docs reviewed; the roll-up `08_SYNTHESIS.md` C1–C4 and D1–D5 triage agreed.
**Exit:** CI builds a signed (or notarization-ready) empty Tauri app; both locale catalogs present and complete (fr-CA complete = release gate, ADR-016); `ez licence-audit` green for the shipped set; ADR-015 adopted in the register (done 2026-11-09) with the export-record approach wired into Phase 1+ design; D2 resolved (candidate FFmpeg build provenance and AV1 encoder selected, feeding candidate ADR-017).
**Blocks:** all of Phase 1+.

## Phase 1 — Import & inventory
**Scope:** Folder ingest, image enumeration, EXIF pre-normalization, SQLite registration (`images`), read-side integrity. (Pipeline Stage 2, `PROCESSING_PIPELINE.md`.)
**Entry:** Phase 0 exit; DB schema for `images` approved in `DATABASE_SCHEMA.md`.
**Exit:** A large folder (thousands of files) imports idempotently and every image is queryable from SQLite; re-import does not duplicate.

## Phase 2 — EXIF & timeline
**Scope:** Timestamp/tz normalization, chronology table, the "distribution over a period" query used downstream. (Stage `PROCESSING_PIPELINE.md` §3.)
**Entry:** Phase 1 exit.
**Exit:** For any imported set, `SELECT` produces a monotonic timeline with per-image capture time resolved to a single normalized epoch; edge cases (mixed tz, missing EXIF, burst offsets) covered by tests.

## Phase 3 — Brightness analysis & cleaning
**Scope:** Lightroom-sidecar / brightness heuristics, clean/quarantine policy, `image_quality` state in SQLite. (Pipeline §4.)
**Entry:** Phase 2 exit; **Phase 6 file-management safety primitives landed** (quarantine must be recoverable).
**Exit:** Cleaning decisions are reproducible from SQLite + original files; quarantined images are recoverable without data loss.

## Phase 4 — Temporal sampling
**Scope:** Sampling strategies (uniform, threshold, keyframe) over the timeline; output sequence materialized; preview. (Pipeline §6.)
**Entry:** Phase 3 exit.
**Exit:** Given a config, produces the exact selected sequence; selection is deterministic for identical inputs (reproducibility, Pipeline §13).

## Phase 5 — FFmpeg video generation
**Scope:** Sequence → MP4 (H.264/HEVC hardware path; AV1 tier per selected option); duration calculation; FFmpeg safety (explicit args, no shell interpolation). (Pipeline §10.)
**Entry:** **ADR-017 / D2 resolved** (FFmpeg build + AV1 encoder fixed) — hard gate, not skippable. Phases 4 exit.
**Exit:** A valid MP4 is produced from a sampled sequence; encode uses the committed encoder path (hardware H.264/HEVC; SVT-AV1 if AV1 tier enabled); no GPL/denied-licence encoder linked in the shipped binary.

## Phase 6 — Consolidation & file management  *(cross-cutting; pull forward)*
**Scope:** Copy/move originals into the project store, hardlink/symlink policy, recovery of moves, dedup, and the "originals immutable" invariant end-to-end. (Pipeline §4, §12.)
**Entry:** Phase 1 exit (need the inventory to manage); should **land before Phase 3** so cleaning/quarantine can rely on recoverable file ops.
**Exit:** A move/copy is reversible; originals remain byte-identical and their path of record in SQLite never points at a mutated file.

## Phase 7 — Advanced UI & contact sheets
**Scope:** Contact-sheet generation (PDF/image), per-page image + timestamp layout (`PRD.md:441`), advanced timeline UI, export-record (derived) preview per ADR-015.
**Entry:** Phases 1–4 exit; ADR-015 adopted (export record is what the sheet/preview represents).
**Exit:** Contact sheets are generated from SQLite state only; layout/timestamp configurable and both locales render correctly.

## Phase 8 — Flicker reduction & stabilization
**Scope:** Frame-level temporal smoothing, exposure-consistency heuristics, stabilization pass; **encode-time gains stored in `flicker_runs` (SQLite)** per D5 — the engine consumes these at Phase 5 encode, not via a manifest. (Pipeline §8.)
**Entry:** Phases 4 exit (need the sequence) and 5's encoder fixed (gains must be applied at encode time).
**Exit:** Flicker/stabilization results are traceable in `flicker_runs`; re-running with identical config reproduces gains; Phase 5 encode consumes them without a separate manifest file.

---

## Cross-phase release gate (applies to every phase exit that ships a user build)

- Locale: fr-CA + en catalogs complete (fr-CA is a **release blocker**; see ADR-016).
- `ez licence-audit` green for the shipped dependency set.
- Apple signing inputs present (`etzian.yaml:66,68`); notarization attempted for non-MAS builds.
- No ADR in `DECISIONS.md` is contradicted by merged code; open ADRs are tracked here, not resolved silently.

## Maintenance of this document

Update the status column (or a per-phase "Last verified" date) at each session end, consistent with `CURRENT_STATUS.md` §9 session-continuity rule. Do not silently drop a phase: if scope changes, record the reason against the relevant ADR or open item and note it here.
