# Audit 06 — PROCESSING_PIPELINE.md

**Audited:** docs/PROCESSING_PIPELINE.md (v0.1, 446 lines)
**Audit date:** 2026-11-09
**Verdict:** Strongest doc of the set so far for safety culture (quarantine, dry-run, explicit operation states, §12's "a database transaction alone cannot guarantee atomicity" is exactly right). Gaps concentrate in the derived-image stages (7–9): their outputs' representation (`pixels` vs `processing instructions`), relative order, and schema provenance — all already flagged by audit 05 F1.

## Alignment confirmed

| Rule | PP ref | Policy source | Status |
|---|---|---|---|
| Never modify originals; derived only | §1 line 26; §7 line 274; §9 line 339 | AGENTS.md §4, ADR-006, PRD §7 | ✅ Stated at principle level and repeated in the stages that could violate it |
| Flagged ≠ deleted; explicit user confirmation of quarantine | §4 cleaning policy lines 142–146 | PRD clean rule, AGENTS.md (quarantine not delete) | ✅ "The user must explicitly confirm" — correct wording |
| Quarantine records src/dst + recovery + verify + restore | §4 lines 148–160 | AGENTS.md §5 (recoverability) | ✅ Matches `file_operations` expectations from schema audit |
| Preview/dry-run before filesystem operations | §5 safety; §9 preview | Arch rule "preview before irreversible action" | ✅ Consolidation has explicit dry-run report + confirmation step |
| Controlled FFmpeg input, no shell strings, no wildcards | §10 FFmpeg safety lines 376–388 | TDD §7 error handling, security best practice | ✅ Good list, including "avoid wildcard-based image selection" |
| Export validation against stated parameters | §11 lines 391–409 | PRD video params; AGENTS.md (validating output) | ✅ codec/container/resolution/fps/duration all checked |
| Reproducibility: config, algorithm version, inputs, status, warnings | §13 lines 428–445 | PRD open #8, TDD §7 | ✅ Present but see F4 for "algorithm version" mechanics |
| Export result stored in SQLite (no external manifest) | §11 line 410 | AGENTS.md §6, ADR-006 | ✅ Consistent with the AGENTS-derived position (contrary to the TDD/ARCH `manifest.json` reading — supports the resolution direction already proposed in audits 02/03/04) |
| Recovery beyond DB transactions | §12 lines 412–426 | ARCH rule (interruption recovery) | ✅ Explicit, correct |

## Findings

### F1 — Stage 8 flicker output is representation-ambiguous (major, cross-doc with 05-F1)

Line 308: "A derived corrected sequence **or equivalent processing instructions**." Those are two very different implementations:

- Corrected sequence = decoded pixels re-encoded as a staged image sequence (storage cost, codec choice, where it lives — nowhere named).
- Processing instructions = exposure-correction curves applied at encode time via FFmpeg filter (`curves`/`colorchannelmixer`-style), i.e. state in SQLite passed to the encoder.

Everything downstream changes based on which one wins: disk-space estimation (stage 5), the `video_exports` provenance (audit 05 F1), and the independence rule with stabilization. TDD stage 2.3.4 and DB schema (`flicker_runs` exists; no staging area) both depend on this answer.

**Action:** decide and state it in this doc before TDD stage 2.3.4 is written. Recommend stating the chosen representation explicitly (a one-line "Flicker correction is applied at encode time as per-frame gain values stored in `flicker_runs`; no intermediate image sequence is produced" — or the opposite) rather than leaving the "or" open.

### F2 — Stage 9 stabilization has no Input/Output, conflicts with Stage 7 crop (major)

Stage 9 (lines 324–338) reads as an investigation plan, not a spec: "Evaluate…", "Potential operations:", and no Input or Output section while every other stage has both. Two concrete issues:

1. **Crop conflict.** Stage 7 lists `Crop` as a user adjustment; Stage 9's stabilization includes "Crop compensation." If the user cropped to a region, frame-alignment cropping has less/invalid area to work with — the interaction is unaddressed anywhere in this doc.
2. **Order vs. Stage 7.** Is stabilization after adjustments (aligning already-cropped/resized frames) or before? §7 line 276 says "Exact order of adjustments must be documented and tested" but never documents it, and no doc so far defines the 7→8→9 relative order.

**Action:** convert stage 9 to the same Input/Processing/Output shape as the others; state the canonical order (or rule) across 7/8/9; add one line on how user crop constrains stabilization's crop compensation (e.g. "stabilization operates on full-frame analysis regardless of a user crop, and output crop is applied after alignment").

### F3 — Video duration formula doesn't cover frame-duplication case (minor)

Lines 368–374: `duration = selected_image_count / output_fps`, then "Frame duplication and interpolation must be separate explicit options." If frames are duplicated to reach a target capture span, encoded frames ≠ selected images, so the formula no longer holds. The doc doesn't say what happens to `duration` in that case (target duration wins? selected-count formula is pre-duplication?).

**Action:** one clarifying sentence, e.g. "with duplication enabled, duration is driven by the target span / first-to-last timestamps instead of by selected_image_count; selected_image_count then describes input, not output, frames." Also define whether "2x playback" halves or doubles the stored video's duration so the formula and the multiplier don't contradict each other when implemented.

### F4 — "Algorithm version" (§13) is unanchored (minor)

Line 434: store "Algorithm version." No doc says what that means in practice — per-subsystem build-version string embedded at build time? A user-visible version per algorithm family? Without a definition, reproducibility claims in the UI and in SQLite (`processing_jobs` etc.) can't be checked for consistency later.

**Action:** define once: recommend embedding a single Rust-binary version (Cargo.toml) + per-algorithm semantic-version constants registered at startup into `processing_jobs.algorithm_versions_json`, rather than relying on binary build metadata alone.

## Notes (no further action)

- §3 "Allow manual timeline adjustments in a future iteration" — consistent with PRD; fine.
- §2 handling of GoPro numbering resets / missing timestamps — good operational detail, matches DB `timestamp_source`/`timestamp_confidence` fields.
- No i18n or licensing findings in this doc (technical spec only); user-facing strings from stages (reports, warnings) fall under the PRD/TDD i18n gaps already logged in audits 01 and 03.

## Summary

| ID | Description | Severity | Recommendation |
|---|---|---|---|
| F1 | Flicker output: "sequence" vs "processing instructions" undecided; cascades into DB + TDD | Major | Decide and state representation explicitly |
| F2 | Stabilization has no I/O spec; conflicts with Stage 7 crop; 7→8→9 order undefined | Major | Reformat stage 9; document canonical order + crop interaction |
| F3 | Duration formula breaks under frame-duplication; multiplier direction ambiguous | Minor | One clarifying sentence each |
| F4 | "Algorithm version" (reproducibility) not defined mechanically | Minor | Anchor to binary version + per-algorithm constants |
