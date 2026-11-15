# PHASES

Phases 0–7 are **complete** and archived in [archive/](./archive/). This file tracks **Phase 8 — Flicker reduction & stabilization**. The canonical scope and exit criteria stay in [docs/DEVELOPMENT_ROADMAP.md](../../docs/DEVELOPMENT_ROADMAP.md), quoted below; the slices implement that scope and nothing more.

> ## Phase 8 — Flicker reduction & stabilization
> **Scope:** Frame-level temporal smoothing, exposure-consistency heuristics, stabilization pass; **encode-time gains stored in `flicker_runs` (SQLite)** per D5 — the engine consumes these at Phase 5 encode, not via a manifest. (Pipeline §8.)
> **Entry:** Phases 4 exit (need the sequence) and 5's encoder fixed (gains must be applied at encode time).
> **Exit:** Flicker/stabilization results are traceable in `flicker_runs`; re-running with identical config reproduces gains; Phase 5 encode consumes them without a separate manifest file.

PRD §5.9 (flicker reduction) and §5.10 (stabilization), Pipeline §8–9, ADR-012 (flicker is a first-class module) and ADR-013 (stabilization is independent of flicker) are the functional spec.

**User decisions, 2026-11-15:**
1. **Stabilization: translation only, in Rust.** Frame-to-frame shift is estimated on the cached 1024 px review thumbnails by coarse-to-fine block matching. The camera path is smoothed, and each frame is shifted and slightly cropped (the crop is capped at 5% per side). No new dependency, and no rotation. FFmpeg's `vidstab` is GPL and is excluded from the LGPL build.
2. **Deflicker: luminance and colour.** Each frame's brightness follows its slow trend, so sunrise, sunset and clouds stay, and quick jumps are pulled back toward it. Each colour channel is also corrected against its own trend (white-balance flicker). Strength (0–100%) and window length are adjustable; colour correction can be turned off. Exposure corrections are capped at ±1 stop and colour corrections at ±⅓ stop.
3. **Validation:** tests on synthetic sequences prove the algorithms, reproducibility and the encode path. **The user's visual check on a real GoPro sequence is a phase item** that only the user can tick (PRD §5.9: "validated against real GoPro sequences").

---

## Slices

Gate names are as before: `GATE-FAST` = fmt + clippy `-D warnings` + `cargo test`; `GATE-UI` = GATE-FAST + svelte-check / vitest / build + `verify.sh` + `ez i18n-check`; `GATE-FULL` = GATE-UI + `ez legal check/gen` + `ez licence-audit`.

### P8.0 — ADR-025, plan
ADR-025 records decisions 1–3, the algorithms, storage, how the encode applies them, and what "traceable" and "reproducible" mean here.
**Gate:** GATE-FAST (docs only).

### P8.1 — Deflicker (pure)
`crates/core/src/adjust/deflicker.rs`:
- **Measurement:** per frame, the mean of each channel in linear light, ignoring clipped and near-black pixels.
- **Trend:** in log space, a centred running median followed by a running mean over a window of N frames (shrinking at the ends).
- **Gains:** `exp(strength × (trend − value))` for luminance, plus a per-channel colour term, each clamped.
- **Application:** three 256-entry lookup tables per frame (sRGB → linear × gain → sRGB). Black stays black.

Unit tests:
- synthetic random flicker is reduced to well under its amplitude;
- a slow ramp (sunset) is left as it is;
- strength 0 changes nothing, and the clamp holds;
- a colour cast that flickers is corrected, and a steady one is left;
- the result is deterministic.

**Gate:** GATE-FAST.

### P8.2 — Stabilization (pure)
`crates/core/src/adjust/stabilize.rs`:
- **Shift estimation:** between consecutive frames, on mean- and contrast-normalised grayscale, with a 4-level pyramid, an exhaustive search at the coarsest level, ±2 px refinement per level, and a sub-pixel parabola fit. A confidence value comes with it; a pair with too little confidence counts as no shift.
- **Path:** cumulative; smoothed with a centred running mean.
- **Correction:** smoothed − measured, clamped by the crop limit. The crop is the largest correction used, at most 5% per side.

Unit tests:
- known shifts of a textured image are recovered within 0.25 px (at thumbnail scale);
- the estimate holds through a brightness change;
- jitter around a slow pan is removed while the pan stays;
- the crop limit holds;
- a featureless frame gives zero shift with low confidence;
- the result is deterministic.

**Gate:** GATE-FAST.

### P8.3 — Runs, store v8, analysis
`crates/core/src/adjust/mod.rs` and `store.rs`:
- **Schema v8:**
  - `flicker_runs`: sampling run, configuration, algorithm version, status, results summary, times;
  - `flicker_frames`: run, sequence index, image, measured channel means, trend, the three gains;
  - `stabilization_runs` and `stabilization_frames`, likewise: the pairwise shift and confidence, path, correction, and the run's crop.
- **Analysis drivers:** for a saved sampling run in sequence order, from the cached review thumbnails (made when missing); parallel per batch; progress; cancel; one analysis at a time. A run is stored complete or not at all (`cancelled` and `failed` keep no frames).
- **Reproducibility:** the same selection, configuration and thumbnails give identical stored numbers. A run records the sampling run's sequence fingerprint.

**Gate:** GATE-FAST.

### P8.4 — Encode integration (D5)
- `ExportSettings` gains `flicker_run` and `stabilization_run` (both optional).
- The export loads that run's per-frame values from SQLite and refuses with `adjustment-mismatch` when a run belongs to another selection or is incomplete.
- `frames::prepare` crops and shifts the source before fitting it, then applies the lookup tables.
- `video_exports.settings_json` records which runs were used, so the export record (ADR-015) shows them. There is no manifest file.

Tests:
- a prepared frame's brightness matches its stored gain;
- a stabilized frame is the shifted crop;
- without runs, frames are byte-identical to before.

**Gate:** GATE-FAST.

### P8.5 — Commands
- Analysis: `start_deflicker`, `start_stabilization` (token + progress Channel), `cancel_adjustment`.
- Data: `list_adjustments`, `adjustment_series` (downsampled for the charts), `adjusted_frame` (before/after of one frame at 1024 px, as JPEG).
- TS types and wrappers. No capability change.

**Gate:** GATE-UI.

### P8.6 — UI (both locales)
- **"Adjust" page** (between Sampling and Export), for a saved selection:
  - deflicker settings, analysis with progress and cancel;
  - a brightness chart (before, trend, after) and a colour chart;
  - before/after stills of any frame;
  - stabilization settings, analysis, a path chart (measured vs smoothed), the crop, and a before/after still;
  - each run's history.
- **Export page:** choose a deflicker run and a stabilization run for the selection (None by default).
- Every GUIDELINES §4 state. The data map gains `adjustments`.

**Gate:** GATE-UI + `ez legal check/gen`.

### P8.7 — Exit tests + independent review
Integration tests on real files:
- a synthetic flickering sequence, analysed twice with the same configuration, gives identical `flicker_frames`, and the corrected frames' brightness varies far less than the input's;
- a shaking sequence's stabilization recovers the shifts;
- an export with both runs encodes, with frames taken from the stored values and no manifest file (an FFmpeg test where the bundled FFmpeg is present, as in Phase 5's encode tests);
- `adjustment-mismatch` is refused;
- cancel leaves no run.

Then an independent reviewer pass.
**Gate:** reviewer sign-off; GATE-FULL; CI green on the macOS runner.

### P8.8 — Final gate, status, commit, push
**User item:** visually check deflicker and stabilization on a real GoPro sequence (decision 3).

---

## Out of scope
- Rotation, perspective or rolling-shutter correction; local (per-region) deflicker.
- Preview videos (the stills and charts are the before/after); a combined auto-tune.
- Any GPL filter (vidstab), any new dependency.
