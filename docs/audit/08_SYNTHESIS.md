# Audit 08 — Cross-cutting Synthesis (Phase 0 exit gate)

**Scope:** Consolidation of `docs/audit/01`–`07`. No new source reads; this is the roll-up.
**Audit date:** 2026-11-09
**Purpose:** Surface which findings are *blocking* Phase 0 exit (they contradict other docs or violate company policy), group them into decision clusters, and produce a prioritized edit list + open-decision list.

---

## 1. Cross-document conflict clusters (blocking)

These each recur in ≥2 audited docs and must be resolved before implementation.

### C1 — "Manifest / derived export" semantics *(01, 02, 03, 04; resolved-in-principle by 06)*
- PRD §/export + AGENTS.md §6: **no external manifest**, metadata lives in SQLite.
- ADR-007: "export metadata stored in SQLite" (agrees).
- TDD §5: lists `storage/manifest.rs` as a module (contradicts).
- ARCH §3+§4: treats the export manifest as a *persistence state* object (strongest contradiction).
- PROCESSING_PIPELINE §11 line 410: "export result stored in SQLite" + ADR-006 — **supports** the AGENTS/PRD position.

**Resolution (recommended):** adopt the SQLite-only stance. `manifest` means *a record of a derived artifact*, not a persistence format.
**Edits:** delete/rename `storage/manifest.rs` → `storage/export_record.rs` in TDD §5; reword ARCH §3+§4 to "derived export record (SQLite)", removing it from the state/persistence language; add **ADR-015** recording this and citing ADR-006 + AGENTS.md. *This unblocks C4 and removes the single most-recurring contradiction in the set.*

### C2 — FFmpeg / AV1 encoder licence decision *(02, 03)*
- Company `policy.yaml` gates: GPL/AGPL **denied**, LGPL **restricted**. FFmpeg is dual-licensed (GPL/LGPL) → its use needs an explicit, defensible position.
- ADR-008 + TDD §9 name "FFmpeg" but **do not cite `policy.yaml`** and do not fix *which* AV1 encoder.

**Resolution options:** (a) link FFmpeg's LGPL path via a system/shared library with no GPL components; or (b) drive an external, allowed-licence encoder (e.g. SVT-AV1 BSD-3, libaom BSD-3, libdav1d ISC) as the *only* FFmpeg use for encode.
**Edits:** add an explicit licence-justification paragraph to ADR-008 **and** TDD §9 that names `policy.yaml`, lists the chosen encoder + its licence, and states which FFmpeg build path is used. **Open decision needed from Etzian** (see §3, D2) — this is a legal/licence gate, not a code one.

### C3 — Localization (fr-CA release blocker + en) *(01, 02, 03; PP user-facing strings)*
- Company `etzian.yaml` mandates **fr-CA (release blocker)** and **en** via Fluent `.ftl`.
- No ADR exists. PRD never mentions UI locale; TDD has no i18n module or design; ARCH deliberately omits it (fine) but there is no owner assigned to the strings produced by PROCESSING_PIPELINE (reports, warnings, preview labels).

**Edits:** add **ADR-016** adopting the company i18n standard (Fluent `.ftl`, `fr-CA` + `en`, fr-CA default for fr-* / CA locales); give TDD an i18n module + a "user-facing strings must come from `.ftl`" rule; add one PRD line that both locales are in scope and fr-CA is a release gate. *Without this, the product cannot legally ship per company policy.*

### C4 — Derived-image stages 7/8/9 cluster *(05-F1 + 06-F1/F2; also 04 mapping)*
All three interlock — fixing one without the others leaves a dead reference:
- **Flicker representation** (06-F1): "corrected sequence **or** processing instructions" undecided.
- **video_exports lineage** (05-F1): no `flicker_run`/`stabilization_run` reference in the export row; no `stabilization_runs` table at all.
- **Stabilization I/O + crop conflict + 7→8→9 order** (06-F2): stage 9 has no Input/Output and conflicts with Stage 7 crop.
- **Stage ⇄ subsystem ⇄ TDD-command mapping** absent (04-F2) means implementers must *infer* C1–C4's relationships.

**Resolution:** decide flicker representation first (encode-time gain in SQLite vs staged image sequence) → that fixes the `video_exports` provenance columns and whether a `stabilization_runs` table is needed → then write stage 9 with real I/O, state canonical order across 7/8/9, and define how user crop constrains stabilization. 
**Edits:** PROCESSING_PIPELINE §F1/F2 (representation + stage-9 reformat); DATABASE_SCHEMA add `stabilization_runs` + lineage columns in `video_exports`; ARCH §3 or §5 add a stage⇄subsystem⇄command cross-ref table so C4's relationships are explicit rather than inferred.

---

## 2. Single-doc defects (non-conflicting, but fix before Phase 1)

| ID | Doc | Finding | Severity |
|---|---|---|---|
| D1 | DATABASE_SCHEMA (05-F2) | `image_status` missing PK/uniqueness on `image_id` — a correctness defect (duplicate rows possible) | **Correctness** |
| D2 | DATABASE_SCHEMA (05-F3) | `import_status` vs `image_status.status` boundary undefined (which owns "ready/failed") | Clarity |
| D3 | PROCESSING_PIPELINE (06-F3) | duration formula breaks under frame-duplication; playback-multiplier direction ambiguous | Minor |
| D4 | PROCESSING_PIPELINE (06-F4) | "Algorithm version" for reproducibility not defined mechanically | Minor |
| D5 | PRD (01) | CLI open-question + 4 policy gaps (see C2/C3 + §3-D1/D3 below) | Varies |
| D6 | CURRENT_STATUS (07-F1) | references missing `DEVELOPMENT_ROADMAP.md`; §5 already has an inline roadmap; phase-5/6 build-order needs a one-line justification | Minor (broken ref) |
| D7 | CURRENT_STATUS (07-F3) | ownership of this file / tie-in to `docs/audit/` as the Phase 0 report unstated | Process |

---

## 3. Open decisions requiring Etzian's input (not resolvable from docs alone)

| # | Decision | Where it lands |
|---|---|---|
| **D1** | Distribution model: signed macOS App Store vs notarized DMG vs both; Apple `team_id`/`signing_identity` are **empty** in `etzian.yaml`. Also Law 25 (Québec) privacy posture + any legal review. PRD §distribution is thin. | New ADR (distribution/legal); `etzian.yaml` fields |
| **D2** | FFmpeg/AV1 licence path (see C2) — which encoder/FFmpeg build, and the licence justification. | ADR-008 + TDD §9 (C2) |
| **D3** | Branding / naming / bundle ID slug: assumed `timelapse-studio`, prefix `com.etzian` — confirm exact slug. Phone-number display rule where law requires it. | PRD + `etzian.yaml` |
| **D4** | Canonical roadmap location: split out `DEVELOPMENT_ROADMAP.md`, or keep CURRENT_STATUS §5 table inline (07-F1). | Either the new file or CURRENT_STATUS §5 |
| **D5** | Flicker representation choice within C4 (encode-time gains vs staged sequence — I recommend encode-time, SQLite-stored, per ADR-006/AGENTS.md consistency). | PROCESSING_PIPELINE + DB schema (C4) |

---

## 4. Priority order to exit Phase 0

1. **C2 + D2** (FFmpeg licence) — legal gate; blocks dependency lock-in and any build.
2. **C3** (i18n ADR-016) — release blocker, cheap to decide now, expensive later.
3. **C1** (manifest reword + ADR-015) — removes the #1 recurring contradiction; prerequisite for clean C4 schema work.
4. **C4** (flicker/stabilization/export lineage + D1 in §2) — largest technical cluster; needs the representation decision (D5).
5. **C3's sibling D1–D7** single-doc defects and **D3/D4** naming/roadmap decisions — low effort, do in one pass.

## 5. Recommended next task (per CURRENT_STATUS §9)

Resolve D1–D5 above. I can draft ADR-015 (manifest) and ADR-016 (i18n) right away since both are decided by existing company policy; C2 and C4-F1 need your call before I draft the corresponding ADR/schema edits. Nothing here requires code implementation — all are doc/decision edits consistent with AGENTS.md's Phase 0 restriction.
