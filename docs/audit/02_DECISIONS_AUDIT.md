# Audit 02 — DECISIONS.md (ADR register)

**Audited:** docs/DECISIONS.md (v0.1, ADR-001…014 + open decisions)
**Audit date:** 2026-11-09
**Verdict:** Good structure and discipline (explicit decision-making rule). One policy conflict must be reconciled (manifest), one licence-risk ADR is under-specified (FFmpeg/AV1), and the register omits several mandatory dimensions that later audits depend on.

## Consistent with company policy & AGENTS.md

- ADR-001 local-first, ADR-006 immutable originals, ADR-011 quarantine — all match AGENTS.md §§4–5, PRD §2. No conflict.
- ADR-002 Tauri 2 / ADR-003 Svelte+TS / ADR-004 Rust engine / ADR-005 SQLite — match AGENTS.md §2 stack defaults (Rust, Tauri 2, Svelte 5 + TS, SQLite local). Good: no silent stack drift.
- Decision-making rule (lines 194–210) directly satisfies AGENTS.md §7 "no silent changes to accepted decisions" — this is the strongest part of the doc and should remain canonical.
- ADR-013 separates stabilization from flicker reduction, matching PRD §5.10 "must remain independent."

## Conflicts / gaps to resolve

1. **Manifest ambiguity (ADR-007).** "The project directory contains the database, **manifest** and generated assets." This is what collides with AGENTS.md §6 "Avoid duplicating database state in JSON manifests" and with 00_INVENTORY #2. Two readings are possible:
   - (A) manifest = human-readable *export* of DB rows → compatible but must be explicitly read-only/derived;
   - (B) manifest = parallel source of truth → violates AGENTS.md §6.
   **Action:** DECISIONS.md should state which one is intended, or add a new ADR (recommend ADR-015 "manifest is a derived, read-only export of the SQLite project") before the data model is built.
2. **FFmpeg / AV1 licensing under-specified (ADR-008 + open list).** AGENTS.md §6 bans GPL/AGPL; AV1 via `libaom`/`SVT-AV1` is BSD/Apache-2.0 (allowed) but many stock FFmpeg builds bundle GPL components and HEVC in some contexts. "Accepted in principle" is fine, but the register should list the **licence families that are acceptable for whichever build ships** so a GPL build is rejected at review time. Recommend adding an explicit line: "FFmpeg build must use only allow-list licences (_foundation/tech/licences/policy.yaml); no GPL/AGPL components."
3. **No localization ADR.** Every ADR is technical/product; none states the fr-CA + en dual-language requirement (company release blocker). A UI-only project would still trip this at release. Recommend adding ADR-016 "UI ships in fr-CA and en; fr-CA default for fr-CA QC locale" referencing `etzian.yaml`.
4. **No distribution/naming ADR.** Bundle-ID prefix `com.etzian`, "— by Etzian" naming, App Store vs Developer ID — none recorded here even though AGENTS.md §3 makes them company-policy items. They don't belong in an ADR *technical* sense, but their absence forces re-derivation per session. A short pointer ADR ("branding and identity come from the company profile via `ez profile`") prevents drift.
5. **SQLite + Svelte library choices remain "open"** (lines 132, 140) while AGENTS.md/TDD already assume SQLite+Tauri+Svelte. That is acceptable as *library-level* openness, but the registry should name the leading candidate per line when known, to avoid re-debate each session (e.g., rusqlite vs sqlx; svelte-kit vs bare). Not a blocker — flag for the TDD audit.

## Cross-references confirmed

- ADR-009 six-stage workflow == PRD §4 six stages (import, clean, consolidate, sample, video-config, generate/export). Consistent; note PRD splits "video configuration" from "generate & export" while ADR-009 lists them as two of the six — same set, wording slightly different. No action needed.
- ADR-010 timestamp sampling == PRD §5.6. Consistent.
- Open-decisions §Technical dependencies / §Data model feed directly into DATABASE_SCHEMA and TDD audits (next steps).

## Recommended actions (approval required)

1. Add ADR-015 fixing manifest semantics as read-only derived export (resolves 00_INVENTORY #2 + PRD §5.1 tension).
2. Add ADR-016 localization requirement and a branding/distribution pointer ADR referencing the company profile.
3. Extend ADR-008 with an explicit allow-list licence line for the FFmpeg/AV1 build.
4. When TDD audit names libraries, back-fill this register so "open decisions" shrink only after approval.
