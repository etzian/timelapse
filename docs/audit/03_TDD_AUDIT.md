# Audit 03 — TDD.md (Technical Design Document)

**Audited:** docs/TDD.md (v0.1, 454 lines)
**Audit date:** 2026-11-09
**Verdict:** Well-structured, disciplined, and consistent with AGENTS.md on architecture, safety, testing, and error handling. Three items need action: manifest semantics (same thread as audits 01/02), absent i18n design, and missing explicit licence-policy pointer.

## Alignment confirmed

| Topic | TDD reference | Policy source | Status |
|---|---|---|---|
| Processing in Rust, not UI | §2, §3 (Rust engine) | AGENTS.md §6 | ✅ Consistent |
| No shell interpolation for FFmpeg | §11 line 345–347 | AGENTS.md §4 "No shell-interpolated commands" | ✅ Consistent |
| No silent overwrite / no wildcard | §11 lines 347, 351 | AGENTS.md §4 | ✅ Consistent |
| Never delete originals automatically | §11 line 356 | AGENTS.md §4 + PRD §2 | ✅ Consistent |
| SQLite authoritative; no binary data in DB | §2, §3 (Persistence), §5 | ADR-005, ADR-006 | ✅ Consistent |
| Job model: ID, status, config snapshot, progress, errors, cancellation, recovery | §6 | AGENTS.md §4 "job must be cancelable, resumable" | ✅ Exceeds minimum |
| Error categories + batch failure reporting | §8 | PRD §10 | ✅ Consistent |
| Testing: unit, integration, reliability, visual | §12 | AGENTS.md §§7–8, PRD test notes | ✅ Comprehensive |
| Definition of done (7 criteria) | §13 | AGENTS.md §8 quality loop | ✅ Consistent |
| Decisions requiring approval before dependency commit | §14 + §9 line 305 | AGENTS.md §9 (no new dep without licence name) | ✅ Consistent |
| Module structure maps to six-stage workflow | §4 `commands/` (8 files = project + 6 stages + export) | ADR-009 | ✅ Consistent |

## Findings requiring action

### F1 — Manifest: still ambiguous, now stronger (repeats from audits 01–02)

Two independent lines compound the concern:

- §5 line 198: *"The JSON manifest provides a portable summary and project-identification information."*
- §4 storage tree line 172: `storage/manifest.rs` — a dedicated write/read module.

AGENTS.md §6 states categorically: **"Avoid duplicating database state in JSON manifests."**

"Portable summary" is compatible with the rule *only if* the manifest never re-enters any process as input and contains only:
- project ID, name, version, schema version, created/last-modified timestamps

If it mirrors any DB rows (image inventory, job history, configuration), it is state duplication.

**Action (same recommendation as 02_DECISIONS_AUDIT):** add ADR-015 fixing manifest scope to **project identification + schema version only**, and either:
- rename `storage/manifest.rs` → `services/portable_export.rs` to make derivation explicit, or
- remove it from `storage/` entirely (it's an export, not a store).

### F2 — No i18n / localization design

The TDD is silent on:

- Which strings are translatable (UI labels, error messages, log output vs user-facing text)
- Framework choice (Fluent `.ftl` files are the company standard per `~/.claude/CLAUDE.md`)
- Locale detection and default (fr-CA default for `fr-*` locale, per `etzian.yaml`)
- Whether Rust-side error strings (`errors.rs` in the module tree) must be translatable or are developer-only

This is a release-blocker requirement that should appear in §2 (principle), §3 (frontend responsibility), and §4 (at least one file, e.g. `services/i18n.rs` or `utils/strings.rs`).

**Action:** add a short "Localization" subsection to §2 and a one-line note in §4 referencing the company Fluent workflow.

### F3 — FFmpeg licence constraint not cited explicitly

§9 line 306: *"Particular attention is required for: FFmpeg and its build configuration."*
§14 line 444: *"FFmpeg distribution strategy"* listed as requiring approval.

This correctly flags the issue but never names the company allow-list/deny-list. Any agent or reviewer reading §9 in isolation cannot know GPL is banned.

**Action:** add one sentence to §9: *"FFmpeg build and all linked codec libraries must use only the licence families allowed by `~/_foundation/tech/licences/policy.yaml`. GPL/AGPL components are prohibited."* — then cross-reference ADR-008 in 02_DECISIONS_AUDIT.

## Minor notes (no action required)

- §6 status list (8 states) is clean; "interrupted" + "needs_recovery" adequately covers the crash case without a separate `paused` state.
- §7 IPC guidance ("do not send large image data through frequent events") is good; consider adding a one-line note on Tauri event channel payload limits for very large contact-sheet batches. Non-blocking.
- §12 "Visual validation" correctly gates flicker/stabilization on user sign-off rather than CI automation — appropriate for solo-dev.
- §14 lists 9 pending decisions; all are explicitly deferred with an approval requirement (AGENTS.md §8). No premature commitment detected.

## Summary of open items carried forward

| ID | Description | Resolution vehicle |
|---|---|---|
| F1 | Manifest semantics + `storage/manifest.rs` placement | ADR-015 + TDD §5 edit + module rename |
| F2 | i18n absent from TDD | TDD §2/§4 patch referencing company Fluent workflow |
| F3 | FFmpeg licence constraint not stated in TDD §9 | TDD §9 one-line cross-reference to policy.yaml |
