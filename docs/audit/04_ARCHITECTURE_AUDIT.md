# Audit 04 — ARCHITECTURE.md

**Audited:** docs/ARCHITECTURE.md (v0.1, 292 lines)
**Audit date:** 2026-11-09
**Verdict:** Clean, consistent with AGENTS.md rules and ADR register. The manifest issue recurs a third time (now strongest — it is named as a first-class persistence concern). One new gap: the mapping between the six workflow stages and the eight architectural subsystems is never made explicit.

## Alignment confirmed

| Rule | ARCH ref | Policy source | Status |
|---|---|---|---|
| No business logic in frontend | §2 rule 1 | AGENTS.md §6, TDD §3 | ✅ |
| No reverse dependency Rust→UI | §2 rule 2 | TDD §2 | ✅ |
| SQLite authoritative | §2 rule 3 | ADR-005 | ✅ |
| Originals external to DB | §2 rule 4 | ADR-006 | ✅ |
| Destructive ops need explicit intent | §2 rule 5 | AGENTS.md §4 | ✅ |
| Stages independently executable | §2 rule 6 | ADR-009/ADR-013 | ✅ |
| No silent invalidation of prior work | §2 rule 7 | ADR-006 immutability | ✅ Strong |
| Advanced processing (flicker/stabilize) read-only vs source | §3 line 190 | PRD §5.10, ADR-013 | ✅ |
| Reject premature abstraction | §7 final line | AGENTS.md §8 discipline | ✅ |
| CLI / future platform extensibility | §7 line 278, 290 | TDD §2 "reusable independently of Tauri" | ✅ Consistent (permissive, non-binding) |

The separation of capability-subsystems (8) from user-facing workflow stages is the right architectural choice — it keeps processing testable headless.

## Findings

### F1 — Manifest named as first-class persistence concern (3rd occurrence)

This doc makes the strongest claim yet:
- Line 42 (architecture diagram): "Persistence" box lists **"Project manifest"** alongside SQLite and operation history.
- Line 80 (Project subsystem): **"Manifest generation"** is a listed responsibility.
- Line 197 (directory tree): `manifest.json`.

AGENTS.md §6 remains authoritative: **"Avoid duplicating database state in JSON manifests."** Listing the manifest inside "Persistence" (rather than under an *export*/services concern) reads as parallel state, not derived output.

**Action:** same ADR-015 recommendation as audits 02–03 — fix scope to project identification + schema version only, and present it as a **derived export** (Project subsystem responsibility "portable-export generation" rather than "manifest" as a persistence store). This is the single doc that most needs correcting because the diagram positions it as state.

### F2 — Stage ⇄ subsystem mapping not explicit

Three documents each describe the workflow differently, and none cross-references the others:

- **ADR-009 / PRD §4:** six user-facing stages (import, clean, consolidate, sample, video-config, generate/export).
- **TDD §4 `commands/`:** eight command modules.
- **ARCHITECTURE.md §3:** eight capability subsystems (Project, Import, Timeline, Image analysis, File mgmt, Sampling, Video, Advanced processing).

None says which stage is served by which subsystem(s). An implementer must infer, e.g., that the "Clean" and "Consolidate" stages are jointly served by *File management* + *Image analysis*. This invites drift between what the UI shows (6 steps) and what the engine exposes.

**Action:** add a small mapping table to §3 or §5 (stage → serving subsystems → TDD command module). Cheap, prevents re-alignment work in every implementation session. This is the highest-value single addition in this doc.

## Minor notes (no action required)

- §5 data flow correctly labels itself "logical workflow, not mandatory linear" (line 246) — good, matches PRD's non-linear model.
- The 8-subsystem count vs 6-stage PRD is not a defect once F2 maps them; flagging only the missing cross-reference.
- No i18n mention here is expected (belongs in TDD/UI layer, already tracked as audit 03 F2).

## Summary

| ID | Description | Resolution vehicle |
|---|---|---|
| F1 | Manifest positioned as persistence state (strongest occurrence) | ADR-015 + reword §3/§4 (export, not store) |
| F2 | Stage ↔ subsystem ↔ TDD-command mapping absent | Add mapping table to §3 or §5 |
