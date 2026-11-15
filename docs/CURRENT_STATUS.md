# Timelapse Studio

## Current Development Status

**Version:** 0.1  
**Last updated:** 2026-11-15  
**Development status:** Phases 0–8 complete; the UX milestone (P1–P15) is built on branch `ux-milestone`, not merged, not released. The user items in §7 are open.

---

## 1. Project status

The product concept and initial architecture are defined, and implementation has begun.

**Phase 0 — Baseline is complete (2026-11-10).** The generated application builds and its verification gate passes: `make gate` is green locally and CI (`ci.yml`) is green on main. See `.claude/plan/STATE.md` for the authoritative record.

**Phase 1 — Import & inventory is complete (2026-11-12)** and **Phase 2 — EXIF & timeline is complete (2026-11-14)**: capture times resolved per image (EXIF → filename → filesystem, with source, confidence and time-zone basis), schema v2, timeline / distribution / gap queries, and the timeline view in both locales. **Phase 3 — Brightness analysis & cleaning is complete (2026-11-14)**: luma analysis, Lightroom XMP reject flags, reproducible classification, user decisions, a journaled no-overwrite quarantine with restore (the minimal Phase 6 primitive, ADR-020), and the Cleaning page in both locales. **Phase 4 — Temporal sampling is complete (2026-11-14)**: fixed-interval / every-Nth / uniform selection, deterministic and stored with its configuration and input fingerprint, with a reproducibility check and the Sampling page in both locales. **Phase 5 — FFmpeg video generation is complete (2026-11-15)**: the LGPL FFmpeg 9.0.2 + SVT-AV1 4.2.0 build is made and licence-checked in CI; sampled selections are exported to MP4 in AV1 (default) or VideoToolbox H.264/HEVC, each verified with ffprobe; ADR-021/022. The FFmpeg and SVT-AV1 licence exceptions are approved (2026-11-14). **Phase 6 — Consolidation & file management is complete (2026-11-15)**: photographs are copied (APFS clones) or moved into a working folder outside the collection after a dry run, with no overwrite, verification, undo and an originals check; schema v6; ADR-023. **Phase 7 — Advanced UI & contact sheets is complete (2026-11-15)**: PDF contact sheets of all photographs, a period, flagged, quarantined, a selection or a video's sequence, generated from SQLite state only (streamed, reproducible, built-in Helvetica), with a Contact sheets page; timeline week buckets, zoom, a period's photographs and identical capture times; the derived export record (ADR-015) shown and saved on demand; schema v7; ADR-024. **Phase 8 — Flicker reduction & stabilization is complete (2026-11-15)**: luminance and colour deflicker against a slow trend, translation-only stabilization with a fixed crop, both measured on a saved selection and stored in SQLite (schema v8), applied by the export at encode time (no manifest); the Adjust page with charts and before/after stills; ADR-025. Open user item: a visual check on a real GoPro sequence.

**UX milestone (2026-11-15, branch `ux-milestone`, P1–P15).** Built on the finished engine, with no schema change and no change to the algorithms or the encode:

- **Project continuity.** A project store holds the root, report, integrity, backfill and a recent list (at most 8, `localStorage`, erased with "app settings"; in the data map). The last project reopens on launch without a rescan, through the read-only core command `project_summary`; an unreachable folder shows a card with Retry and Remove. Home is the project overview (steps, short summary, details, timeline, recent projects, drop a folder anywhere in the window). All `localStorage` keys are named in `storageKeys.ts`, and a test checks that Erase leaves no `timelapse*` key.
- **Pipeline navigation.** The sidebar has Workflow, Tools and App groups with a state per step (not started, in progress, done), and the tab bar follows the same order. A live region announces when jobs start and end. `NextStep` closes Home, Cleaning, Sampling, Adjust and Export with the next action and, when disabled, the reason.
- **Selections** keep their numeric id; the label shown is derived from method, count and date.
- **Flipbook and Wipe.** The Flipbook plays cached thumbnails (no video is decoded in the webview) in Sampling, plays the adjusted loop in Adjust and previews the sequence in Export. Wipe compares as shot and corrected frames.
- **Fast triage.** Cleaning is a keyboard-driven grid with an Inspector, immediate Keep/Reject decisions and an Undo toast (also ⌘Z).
- **Presets and summaries.** Clean presets (Lenient, Balanced, Strict) and export presets (Share, Archive, Master); the thresholds and the other selects sit under "Advanced settings"; a one-sentence summary of the resulting video.
- **Open and reveal.** `open_output` and `reveal_output` resolve an id to a stored path in `video_exports` or `contact_sheets`, open the database read-only, and accept only a regular file inside `<root>/Timelapse exports/` with an allow-listed extension (mp4, pdf, json), through `tauri-plugin-opener` 2.7.3 used from Rust only. The webview has no opener permission (ADR-027; a shell test checks it). Core: `outputs.rs`, `open_read_only`.
- **Progress and the export finish.** One `Progress` component (smoothed rate, time remaining) replaces every bare progress bar; Export shows a thumbnail while encoding and a finish card with poster, duration, size, codec, copyable path, Play, Show in Finder and Contact sheet; history rows offer the same actions.
- **Native menu and view transitions.** The menu is built in JS (⌘1–⌘5, ⌘, Settings, ⌘O Open folder) and rebuilt on locale change; page swaps cross-fade except under reduced motion.
- **Copy.** English reworded in plain words; fr-CA translated and reviewed.
- **Review.** P14 reviewed the source (UI, Rust, legal) and applied the findings; the reviewers did not re-review the fixes, so there is no reviewer sign-off. See `docs/design-review.md`.

## 2. Confirmed technology direction

- Tauri 2.

- Svelte.

- TypeScript.

- Rust.

- SQLite.

- FFmpeg.

Specific library versions and dependency choices remain subject to technical review.

## 3. Confirmed architecture

- Local-first desktop application.

- Rust processing engine independent of the UI.

- SQLite as the project database.

- Original photographs remain immutable.

- Project data stored separately from source photographs.

- Processing jobs are traceable.

- File operations must be recoverable where practical.

- UI communicates with Rust through Tauri commands and events.

## 4. Confirmed product workflow

1. Import & Analyze.

2. Clean Images.

3. Consolidate.

4. Sample & Select.

5. Video Configuration.

6. Generate & Export.

Advanced capabilities:

- Flicker reduction.

- Stabilization.

- Contact sheets.

- Image adjustments.

## 5. Development roadmap

**Authoritative planning and status:** the current development sequence is governed by `.claude/plan/PHASES.md` (frozen phase specification) and `.claude/plan/STATE.md` (authoritative development status). Consult those two documents first; this file is a supporting summary only.

**Supporting technical analysis:** `docs/DEVELOPMENT_ROADMAP.md` — per-phase scope, entry/exit criteria, dependency order, and the cross-phase release gate (locale, licence audit, Apple signing, ADR consistency). It is retained as technical context; it is no longer the canonical sequencing document.

The FFmpeg/AV1 licensing decision is settled: **ADR-017** (adopted 2026-11-09) — LGPL-only FFmpeg sidecar, SVT-AV1 (BSD-3-Clause) default, VideoToolbox for H.264/HEVC compatibility. See `docs/DECISIONS.md` (ADR-017) and `docs/FFMPEG_AV1_ANALYSIS.md`. ADR-017 remains adopted and is not reopened by this reconciliation; three open *technical implementation* items are carried forward as pre-Phase-5 prerequisites (see `docs/FFMPEG_AV1_ANALYSIS.md` §4).

Status of record: per `.claude/plan/STATE.md` — Phase 0 **done (2026-11-10)**, Phase 1 **done (2026-11-12)**, Phase 2 **done (2026-11-14)**, Phase 3 **done (2026-11-14)**, Phase 4 **done (2026-11-14)**, Phase 5 **done (2026-11-15)**, Phase 6 **done (2026-11-15)**, Phase 7 **done (2026-11-15)**, Phase 8 **done (2026-11-15)**.

## 6. Immediate next actions

The Phase 0 documentation review is complete; its findings are recorded in `docs/audit/`. The immediate objective before any Phase 1 work is to define the first real feature and obtain user approval:

- Propose a concrete Phase 1 feature scope (core logic in `crates/core` with tests, a Tauri command, a frontend page, strings in both locales, and a `legal/data-map.yaml` update if it stores or sends anything) per `.claude/plan/PHASES.md`.

- Identify its dependencies, risks, and affected components.

- Present the proposed scope for user approval before implementation (per AGENTS.md §4).

## 7. Current milestone

**Milestone:** the roadmap's phases are delivered and the **UX milestone** (15 slices on branch `ux-milestone`; see `docs/UX_MILESTONE.md` and `.claude/plan/PHASES.md`) is built. `make gate` is green. The version bump and changelog are left to `etzian:release` on the user's word; nothing is pushed or tagged.

**Open user items:**

1. **Rendered pass on the Mac** (keyboard only, VoiceOver, light and dark, compact density, reduced motion, fr-CA and en, 375 / 768 / 1440 widths). Record the result in the table in `docs/design-review.md` ("Rendered pass"). Points to settle there: sidebar icons and accent bar, the viewer footer in fr-CA at 520 px, sidebar labels and the menu in French, VoiceOver on triage tiles and the wipe slider, ⌘Z against the native Edit menu, muted text contrast in dark mode.
2. **Play and Show in Finder on a real export**, and the missing-file error in en and fr-CA; the thumbnail beside the progress bar moving during an export.
3. **Native menu** in the running app: shortcuts, labels switching with the language, no cross-fade under reduced motion.
4. **Real-GoPro visual check** of deflicker and stabilization (carried over from Phase 8).
5. **Legal questions for the lawyer** (design-review.md, "Open, not done here"): whether launch-time reopening needs a setting or a notice; whether a folder path is personal information for local-only data; the window-state file (not erased, not in the data map; predates the milestone); the privacy policy does not mention handing exports to the default player; the three legal templates are unreviewed.
6. **Release checks** (`.claude/rules/release.md`): the hardware VideoToolbox test on Apple Silicon and the exFAT consolidation check (copy, check, undo, then move and undo).

**Known gaps (not done):** sidebar state glyphs are read aloud and the tab bar shows no state (needs a design-system slot); native Edit > Undo is not wired to the app's undo; field-linked errors in Advanced; per-frame flipbook retry; localized predefined menu items; a few fr-CA wording nice-to-haves (see STATE.md, P13).

Phase 0 — Baseline was completed on 2026-11-10: the generated application builds and `make gate` passes locally and in CI (see `.claude/plan/STATE.md`). The Phase 0 documentation review required by this file's former §7 produced the reports in `docs/audit/` and is considered satisfied.

## 8. Implementation restriction

The implementation gate formerly stated here — "do not start implementing until the Phase 0 review has been presented and approved" — is **satisfied**; Phase 0 is complete as recorded in `.claude/plan/STATE.md`.

The governing rule going forward: **Phase 1 feature implementation requires an approved scope.** An agent must not begin Phase 1 feature work, or edit the frozen phase specification (`.claude/plan/PHASES.md`), to accommodate convenience.

## 9. Session continuity

At the end of every development session, update this file with:

- Current phase and milestone.

- Completed tasks.

- Files created or modified.

- Tests executed and results.

- Known bugs.

- Outstanding questions.

- Decisions made.

- Recommended next task.

**Authoritative status record:** `.claude/plan/STATE.md` is the canonical development-status and session-handoff record (per AGENTS.md). Update it whenever development progress changes; treat this file as a supporting summary rather than the status of record.

Do not assume that an undocumented implementation decision has been approved.
