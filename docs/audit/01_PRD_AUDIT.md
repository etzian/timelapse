# Audit 01 — PRD.md

**Audited:** docs/PRD.md (v0.1, "Initial specification — pending review")
**Audit date:** 2026-11-09
**Verdict:** Substantively sound; no internal contradictions found. Four policy gaps to close before approval: localization, distribution/legal, branding, and one manifest tension confirmed against AGENTS.md.

## Consistent with project rules (AGENTS.md)

- Local-first, non-destructive, quarantine-over-deletion, dry-run, recoverability (§2, §5.4, §5.5, §6) match AGENTS.md §§4–5 and the ADR direction in DECISIONS.md.
- Stack-neutral in body: SQLite as project store, FFmpeg for export, no cloud — matches AGENTS.md §2 and TDD direction (§3 audit pending).
- 100,000+ image design target (§6) matches AGENTS.md "10,000–100,000+".
- Rust processing independent from UI (§2 principle 7, §6 maintainability) matches AGENTS.md §2 and §6.

## Gaps against company policy (must be added or explicitly deferred)

1. **Localization (release blocker).** PRD is entirely English; no requirement states the UI must ship in fr-CA and en with fr-CA as default for French/QC locales. Company rule: fr-CA is a release blocker (`_foundation/company/etzian.yaml` dev.locales). Recommend adding a localization requirement section (language, BCP 47 codes, `.ftl` resources) before PRD approval.
2. **Distribution / commercialization.** No statement on how the app will be delivered (App Store vs Developer ID), whether it is free or paid, or that payments (if any) go through a Merchant of Record per D4. Not strictly a PRD duty, but "out of scope" (§7) does not cover this; recommend a one-paragraph distribution note pointing to the company profile so downstream docs (bundle ID, naming) have a basis.
3. **Naming and identity.** No product naming or "— by Etzian" endorsement rule anywhere in the PRD (`etzian.yaml` brand.architecture D2). Expected later in TDD or a release checklist; flag as missing.
4. **Privacy / Law 25.** PRD records GPS coordinates from EXIF (§5.2) and keeps everything local — compliant posture, but no explicit declaration that "no personal data leaves the machine" exists. Recommend adding a privacy statement section (supports later legal review via `_foundation/legal/`).

## Tensions to resolve in later audits

- **Project manifest** (§5.1: "Maintain a project manifest for portability and inspection") vs AGENTS.md §6 ("Avoid duplicating database state in JSON manifests"). Not an outright contradiction — a *read-only export* manifest is compatible — but the PRD should specify the manifest is derived/read-only, not a second source of truth. Confirm during DATABASE_SCHEMA audit.
- **"Standalone CLI" open question** (§9) vs AGENTS.md §2 (engine independence "allowing future CLI reuse"). Direction compatible; wording suggests CLI is undecided. Recommend PRD open-questions entry be marked "architecture already requires engine independence; CLI exposure TBD."
- **Windows**: AGENTS.md says "potential future Windows support"; PRD says "macOS first" with no forward note — consistent, no action needed.

## Quality issues (minor)

- §5.7 frame rate list mixes standards (23.976/29.97 missing) — acceptable as-is; custom covers the rest.
- §5.7 "AV1 if supported by available encoder" — fine, but note AV1 licensing is clean (libaom/RV: BSD/Apache-ish families); no policy issue with current allow list unless a specific GPL build is chosen — check when TDD names the FFmpeg build.
- §5.9 requires validating flicker reduction against "real GoPro sequences" — test-asset procurement/consent is an open dependency not tracked anywhere; add to roadmap when it exists.
- No acceptance thresholds anywhere (e.g., what counts as a "dark image" baseline) — intentionally left configurable, acceptable for v0.1.

## Open questions inherited from PRD (carry into DECISIONS audit)

Formats/RAW roadmap · EXIF library choice · FFmpeg build & licensing · thumbnail/contact-sheet formats · flicker algorithm · stabilization deps · project relocation strategy · CLI.

## Recommended actions (approval required before doing any of them)

1. Add localization + privacy + distribution sections to PRD (or a short addendum referencing the company profile).
2. Clarify manifest semantics as read-only export.
3. Re-state PRD status after DECISIONS.md audit closes the open questions above.
