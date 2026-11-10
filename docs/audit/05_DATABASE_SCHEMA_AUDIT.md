# Audit 05 — DATABASE_SCHEMA.md

**Audited:** docs/DATABASE_SCHEMA.md (v0.1, 304 lines)
**Audit date:** 2026-11-09
**Verdict:** Solid, policy-aligned draft with one significant lineage gap (video exports not tied back to the processing stage that produced them), one correctness defect (image_status lacks a primary key / uniqueness constraint), and several open design questions already flagged by the doc itself.

## Alignment confirmed

| Rule | DB ref | Policy source | Status |
|---|---|---|---|
| SQLite authoritative for project data | §0 (line 8) | ADR-005 | ✅ Explicitly stated at top of doc |
| One database per project (`project.db`) | §3 line 232 | ADR-006, AGENTS.md §6 | ✅ Consistent; `project_id` column noted as "may not be necessary" — acceptable given per-project DB |
| Originals external / metadata only | §1 line 16 | ADR-006, PRD §5.3 | ✅ "Store metadata, not photographic binary data" |
| Immutable source inventory separate from derived selections | §1 line 22; `images` vs `sampling_items` | ADR-006 §7 invariant, AGENTS.md §6 | ✅ Clean separation |
| Processing history recorded | §1 line 24; `processing_jobs`, `file_operations`, `flicker_runs`, `video_exports` | PRD open #3 (recoverability), ARCH line 43 | ✅ Multiple operation-log tables present |
| Missing files ≠ auto-delete inventory | §5 line 272 | AGENTS.md §4 (never delete originals), PRD quarantine | ✅ Explicitly stated; strong rule |
| Schema versioning + safe migrations | §6 (lines 274–289) | ADR-004, AGENTS.md §8 | ✅ "Never silently recreate" is the right guardrail |
| Timestamps normalized UTC, EXIF + uncertainty preserved | §1 line 30; `images.timestamp_source`, `timestamp_confidence` | PRD open #6 (timezone awareness), ADR-004 | ✅ Best-handled of all policy cross-checks in this doc |
| Foreign keys + transactions + indexes | §5, §4 | TDD §7 error handling | ✅ |

## Findings

### F1 — video_exports lineage gap (major)

`video_exports.sampling_run_id` links the output back to the sampling stage only. After sampling, §5 data flow inserts "Optional image adjustments → Flicker reduction / stabilization → FFmpeg encoding." The schema has `flicker_runs` (which references `sampling_run_id`) but:

- `video_exports` has no column referencing `flicker_runs` (or a stabilization run).
- Two different flicker configurations on the same sampling run would produce two visually different videos, both pointing at the same `sampling_run_id`, with no way to tell which settings produced the stored file.
- No `stabilization_runs` table exists at all — flicker is captured, stabilization (PRD §5.10, TDD stage 2.3, ARCH advanced processing subsystem) has no schema home.

**Action:** add a `source_run_id`/`stage_run_id` on `video_exports` that may reference either `sampling_runs.id` or `flicker_runs.id` (or a generalization), or add an intermediate `processing_stage` table so exports point at the concrete last stage applied. Also decide whether stabilization needs its own runs table (symmetry with flicker) before schema is finalised. This is the highest-priority fix in this doc.

### F2 — image_status has no primary key / uniqueness constraint (correctness defect)

The `image_status` table (lines 95–103) defines `image_id` as "Foreign key" but specifies no PK and no UNIQUE constraint on `image_id`. §5 line 266 says "Define uniqueness constraints where appropriate" — this is exactly such a case. Without it, the same image can accumulate multiple status rows (Eligible + Rejected simultaneously), making classification semantics ambiguous.

**Action:** add either `PRIMARY KEY (image_id)` (if one current status per image) or `UNIQUE(image_id)` + a separate audit-history table (matching §7 open question "Whether image statuses require a full audit history" — resolve that first; if yes, this table becomes an event log and a separate `current_status` materialized view is needed). Don't leave it ambiguous in the implemented migration.

### F3 — Two overlapping status fields on images (minor design clarity)

- `images.import_status` (line 75, "Inventory status")
- `image_status.status` (line 100, "Eligible, flagged, rejected, quarantined, excluded")

Both classify an image's standing. It's not impossible they're complementary (e.g. `import_status`: pending/in_progress/done; `image_status.status`: eligibility), but the relationship is never stated, and "quarantined" as a value in `image_status.status` overlaps conceptually with what you'd expect `import_status` to track during the Clean stage. An implementer has to guess the boundary.

**Action:** one sentence per column stating its allowed value set and how they interact (e.g. "import_status tracks ingestion lifecycle only; image_status.status tracks eligibility decision; quarantined is a state of image_status, not import_status"). Low cost, prevents drift between UI labels and DB values.

## Minor notes (no action required)

- `file_operations` hangs under `processing_jobs` in the relationship diagram (§3). Consistent with "every destructive/long-running op is a job," but worth confirming that Import-stage quarantine always creates a `processing_job` row, or `job_id` needs to be nullable.
- `images.file_hash` described as "Exact-content hash." Full content hash over HEIC/CR3 can be expensive at scale; the doc's own §7 doesn't list this as an open question (unlike timestamp strategy etc.). Worth one line in §7: full-hash vs. sampling, and at-import vs. lazy.
- `timeline_events` references no image range directly (uses `details_json`) — already covered by §7 open question "Whether timeline events should be computed or persisted."; no additional action.
- No i18n concern here (stored data, not UI text).

## Summary

| ID | Description | Severity | Recommendation |
|---|---|---|---|
| F1 | video_exports not linked to flicker/stabilization stage; no stabilization_runs table | Major | Add stage-provenance column before first migration |
| F2 | image_status missing PK/uniqueness | Correctness defect | Fix in schema v0.1.1 or first migration |
| F3 | import_status vs image_status.status boundary undefined | Design clarity | One-sentence value-set disambiguation |
