# PHASES

Phases 0–3 are **complete** and archived in [archive/](./archive/). This file tracks **Phase 4 — Temporal sampling**. The canonical scope and exit criteria stay in [docs/DEVELOPMENT_ROADMAP.md](../../docs/DEVELOPMENT_ROADMAP.md), quoted below; the slices implement that scope and nothing more.

> ## Phase 4 — Temporal sampling
> **Scope:** Sampling strategies (uniform, threshold, keyframe) over the timeline; output sequence materialized; preview. (Pipeline §6.)
> **Entry:** Phase 3 exit.
> **Exit:** Given a config, produces the exact selected sequence; selection is deterministic for identical inputs (reproducibility, Pipeline §13).

**User decisions, 2026-11-14:**

1. **Strategies** follow PROCESSING_PIPELINE.md §6 and PRD §5.6, since the roadmap's "threshold" and "keyframe" are defined nowhere:
   - **fixed interval**: targets every *Δt*, matched closest / first-after / last-before within a tolerance;
   - **every Nth** photograph, in chronological order;
   - **uniform**: *N* photographs spread over the period.

   No photograph is selected twice; unmatched targets are reported.
2. **Input set:** every photograph *except* rejected and quarantined ones. Flagged and not-analysed photos are included, with a per-run option to exclude flagged ones. Photographs without a capture time are always excluded and counted (ADR-010).
3. **Output** ("materialized"): an ordered selection in SQLite (`sampling_runs`, `sampling_items`). Each run stores its configuration, algorithm version and an input fingerprint. No files are written.
4. **Preview:** counts, reduction %, effective interval, and estimated duration at a chosen FPS × playback multiplier (PRD §5.7; these stay separate concepts). Also a distribution bar of the selection and a strip of selected thumbnails. Contact sheets stay in Phase 7.

No new dependency; no new capability.

---

## Slices

Gate names are as in Phase 3: `GATE-FAST` = fmt + clippy `-D warnings` + `cargo test`; `GATE-UI` = GATE-FAST + svelte-check / vitest / build + `verify.sh` + `ez i18n-check`; `GATE-FULL` = GATE-UI + `ez legal check/gen` + `ez licence-audit`.

### P4.1 — Selection algorithms (pure)
`crates/core/src/sampling/select.rs`:
- **Input:** frames sorted by `(epoch_ms, original_path)`. **Output:** a `Selection` holding the ordered picks (target and actual time) and the unmatched targets.
- **Fixed interval:** targets from the first frame onward, matched with the *monotone no-duplicate* rule: each pick is later in the sequence than the previous pick. Tolerance defaults: half the interval for closest, one interval otherwise. Ties go to the earlier frame.
- **Every Nth:** with an offset.
- **Uniform:** *N* targets spread evenly over the first to last frame, matched closest.
- **Limits:** configurations that would generate more than 5 000 000 targets, or have non-positive parameters, are rejected as `invalid-sampling`.

Unit tests: each policy, tolerance edge cases, gaps that produce unmatched targets, ties, no duplicates, N ≥ the frame count, empty input, and determinism.
**Gate:** GATE-FAST.

### P4.2 — Store v4 + runs
- **Migration v4:** `sampling_runs` (method, `configuration_json`, `algorithm_version`, input count, input fingerprint as the SHA-256 of `id:normalized_datetime` lines, selected count, unmatched targets, excluded counts, status, `created_at`) and `sampling_items` (`run`, `image`, `target_datetime`, `actual_datetime`, `sequence_index`, unique per run).
- **Input query:** applies the input rules (status, period `[from, to)`, flagged option) and counts what was excluded and why.
- **Functions:** `preview` (no write), `create_run` (one transaction), `list_runs`, `run_items`, and `verify_run`, which re-runs the stored configuration on the current inputs and reports whether the inputs and the selection are identical.

**Gate:** GATE-FULL (the data map gains `sampling-runs`).

### P4.3 — Tauri commands + IPC
Thin commands: `preview_sampling`, `create_sampling_run`, `list_sampling_runs`, `get_sampling_items`, `verify_sampling_run`. TS types and wrappers.
**Gate:** GATE-UI.

### P4.4 — Sampling page (both locales)
- **Controls:** method; parameters (interval value + unit, policy, optional tolerance; N + offset; count); optional period; include-flagged toggle; FPS (12, 15, 24, 25, 30, 50, 60) and multiplier (0.5, 1, 2, 4), which drive the estimate only.
- **Preview:** stats, distribution by day, thumbnail strip, unmatched count.
- **Runs:** save as a run; a list of saved runs with a reproducibility check.
- Every state from GUIDELINES §4; all strings in both catalogs.

**Gate:** GATE-UI.

### P4.5 — Exit tests + independent review
Integration tests on real files: import → (clean) → sample. They check:
- an exact expected sequence for each method on a known set;
- identical inputs give an identical sequence;
- a changed input is detected by `verify_run`;
- rejected and quarantined images are never selected;
- missing timestamps are excluded and counted.

Then an independent reviewer pass.
**Gate:** reviewer sign-off in STATE.md; GATE-FULL.

### P4.6 — Final gate, status, commit, push
**Gate:** GATE-FULL + push to `origin/main`.

---

## Out of scope for Phase 4
- "Threshold" / "keyframe" strategies (undefined; a future, separately specified addition), adaptive sampling.
- Writing a sequence folder or any file (Phase 5 / Phase 6).
- Contact sheets (Phase 7). Video encoding (Phase 5).
- Repeating a photograph for several targets (PRD: only as an explicit separate option; not requested).
