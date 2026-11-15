# PHASES

**Plan:** UX milestone — a guided, visual, fun Timelapse Studio. **15 slices.**

**Goal:** the app remembers the project, shows the pipeline and what comes next, plays the timelapse before any export, makes triage fast, and finishes an export with something to watch.

**Exit test:** on a relaunch the last project opens without the folder picker; from Home, with the keyboard only, the user reaches a saved selection, plays it in the flipbook, exports it, and reveals the video in the file manager; the same holds in fr-CA and en; `make gate` is green.

Phases 0–8 are complete and archived in [archive/](./archive/). This milestone is outside `docs/DEVELOPMENT_ROADMAP.md` (which ends at Phase 8); it comes from the UI review of 2026-11-15, approved by the user the same day.

**User decisions, 2026-11-15:**
1. The seven-phase plan (U0 baseline, U1 project continuity, U2 guided pipeline, U3 flipbook, U4 fast triage, U5 export finish, U6 polish) is approved.
2. **`tauri-plugin-opener` is approved** for "Play" and "Show in Finder", limited to files the app produced. It needs an ADR (P1).
3. **Recent projects** are stored on the device (folder paths); `legal/data-map.yaml` changes in the same commit.
4. The progress bar and the flipbook are `.app-*` components in this repo. `@etzianco/ui` is read-only; anything it should adopt is listed in `docs/design-review.md`, not changed there.

## Gates

- `GATE-FAST` = `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test --workspace`
- `GATE-UI` = `npm --prefix frontend run check` + `npm --prefix frontend run test` + `npm --prefix frontend run build` + `./scripts/verify.sh` + `ez i18n-check .` (plus GATE-FAST when the slice touches Rust)
- `GATE-FULL` = `make gate`

Redirect gate output to a file and read the tail. Every UI slice keeps fr-CA and en at parity (`ez i18n-check .` is part of GATE-UI) and draws the GUIDELINES §4 states for what it adds.

---

## P1 — ADR-027 and baseline debts (U0)

**Files:** `docs/DECISIONS.md`, `docs/design-review.md` (new), `frontend/src/lib/pages/Home.svelte`, `frontend/src/App.svelte`, `frontend/src/app.css`, `locales/{en,fr-CA}/app.ftl`.

- **ADR-027** records decisions 1–4 and how each is built:
  - the opener is reached only through Rust commands that take an id, look the path up in SQLite and check it (P10); the webview gets no opener permission and `capabilities/default.json` does not change;
  - recent projects live in `localStorage` beside the settings, at most 8, erased with "app settings";
  - the flipbook plays cached thumbnails; no video is decoded in the webview.
- `Home.svelte`: replace raw `rem`/`em` values with `--ez-*` tokens, put numbers through `formatNumber`, rename classes to `.app-home__*`.
- Sidebar: a footer with the sun mark and the "by Etzian" endorsement (read `../_uidesign/docs/brand.md` and the `Sidebar` footer snippet), and an icon per destination (inline SVG authored here, `currentColor`).
- `docs/design-review.md`: the findings of the 2026-11-15 source review as a baseline, with a "not rendered" note and an empty table for the rendered pass (P14).

**Gate:** GATE-UI.
**Model:** sonnet

## P2 — Project store and continuity (U1)

**Files:** `frontend/src/lib/stores/project.svelte.ts`, `frontend/src/lib/project.ts` (new, pure) + test, `frontend/src/lib/stores/settings.svelte.ts`, `frontend/src/lib/erase.ts`, `frontend/src/lib/ipc/app.ts`, `crates/core` and `src-tauri/src/lib.rs` only if a summary command is missing, `legal/data-map.yaml`, `legal/generated/`, locales.

- The project store holds the root, the import report, the integrity result and the backfill, so they survive leaving Home.
- The last project and up to 8 recent ones are kept in `localStorage` (path and last-opened time). Pure helpers (add, dedupe, cap, remove) with vitest.
- On launch the last project is opened **without a rescan**: use the existing read commands; if none returns the inventory and timeline summary of an already imported folder, add one read-only core function and command `project_summary`.
- A folder that is missing or on an unmounted volume is a designed state: the entry stays, says the folder cannot be reached, and offers Retry and Remove from the list.
- "Erase app settings" clears the recent list. The data map gains the recent-projects entry; `ez legal gen .` regenerates.

**Gate:** GATE-UI + GATE-FAST + `ez legal check .` + `ez legal gen .`.
**Model:** sonnet

## P3 — Home as the project overview (U1)

**Files:** `frontend/src/lib/pages/Home.svelte`, `frontend/src/lib/pages/TimelineSection.svelte` (placement only), `frontend/src/lib/components/` (new pieces as needed), locales.

- **No project:** a welcome with one primary action (choose a folder), a drop target for a folder, and the recent projects.
- **Project open:** the folder name as the title, one line per pipeline step with its figure (photographs, flagged, selections, exports), Rescan and Switch project as secondary actions. The import report becomes a short summary; duplicates, missing files and the technical counts go under a disclosure. The timeline stays below.
- Dropping a folder uses the webview drag-and-drop event of `@tauri-apps/api`. If it needs a permission beyond `core:default`, stop and report instead of widening the capability; the picker remains the path.
- Home copy loses the hashing and database wording.

**Gate:** GATE-UI.
**Model:** sonnet

## P4 — Pipeline navigation (U2)

**Files:** `frontend/src/App.svelte`, `frontend/src/lib/stores/nav.svelte.ts`, `frontend/src/lib/stores/pipeline.svelte.ts` (new), `frontend/src/lib/pipeline.ts` (new, pure) + test, locales.

- Sidebar `groups`: **Workflow** (Import, Clean, Select, Adjust, Export), **Tools** (Organize, Contact sheets), then Settings. The TabBar keeps the same order.
- Each workflow step shows its state (not started, in progress, done) by an icon and text, never by colour alone. The state is derived in `pipeline.ts` from counts the pages already load; tested.
- A running job (analysis, adjustment, export, consolidation, contact sheet) shows on its nav item and is announced by a live region when it ends.
- Nav labels become verbs a newcomer understands; page titles follow.

**Gate:** GATE-UI.
**Model:** sonnet

## P5 — Next step and named selections (U2)

**Files:** the five workflow pages, `frontend/src/lib/sampling.ts` + test, `frontend/src/lib/components/NextStep.svelte` (new), locales.

- Each workflow page ends with one "Next" action naming the following step, enabled when the step has what it needs, with the reason shown when it does not.
- A saved selection is labelled by what it is ("Every 5 min · 1,033 photos · 15 Nov"), built by one pure function from the run's method, count and date, and used in every select and list (Sampling, Adjust, Export, Organize, Contact sheets). The number stays as a secondary detail. No schema change.

**Gate:** GATE-UI.
**Model:** sonnet

## P6 — Flipbook (U3)

**Files:** `frontend/src/lib/components/Flipbook.svelte` (new), `frontend/src/lib/flipbook.ts` (new, pure) + test, `frontend/src/lib/pages/Sampling.svelte`, `frontend/src/lib/ipc/app.ts`, `src-tauri/src/lib.rs` and `crates/core` only for a batched thumbnail read, locales.

- Plays a sequence of photograph ids from the cached 256 px thumbnails at the chosen frame rate: play/pause, a scrubber (`input type="range"`), frame number and capture time, previous/next frame.
- `flipbook.ts`: frame timing, the window of frames to keep loaded around the playhead, and eviction; frames load ahead in batches and playback waits rather than skipping. Tested (order, window, wrap, a failed frame shows a placeholder and playback goes on).
- A batched read (`get_thumbnails(root, ids, size)`) may be added if per-frame `invoke` is too slow (GUIDELINES §8.4). No capability change.
- Keyboard: Space plays and pauses, arrows step, Home/End jump; all shown in the control labels. With `prefers-reduced-motion` it never starts by itself.
- Sampling: the flipbook sits above the strip, for the preview and for a saved selection.

**Gate:** GATE-UI (+ GATE-FAST if Rust changed).
**Model:** sonnet

## P7 — Flipbook in Adjust and Export, before/after wipe (U3)

**Files:** `frontend/src/lib/pages/Adjust.svelte`, `frontend/src/lib/pages/Export.svelte`, `frontend/src/lib/components/Wipe.svelte` (new), `frontend/src/lib/stores/adjust.svelte.ts`, locales.

- **Wipe:** the two stills become one frame with a divider; the divider is a slider (pointer and keyboard), labelled Before and After in text.
- **Adjust:** a loop of at most 60 frames around the chosen frame plays as shot or adjusted (`adjusted_frame`, fetched once and cached for the run shown).
- **Export:** the flipbook of the chosen selection at the chosen frame rate and speed, as shot.

**Gate:** GATE-UI.
**Model:** sonnet

## P8 — Fast triage (U4)

**Files:** `frontend/src/lib/pages/Cleaning.svelte`, `frontend/src/lib/components/Inspector.svelte` (new, split out of Cleaning), `frontend/src/lib/clean.ts` + test, locales.

- The grid is one tab stop with arrow-key movement; `A` keeps, `R` rejects, `U` clears, Space opens the inspector. The keys are listed on the page and are never the only way.
- The inspector gains Previous/Next and the decision buttons, and moves on after a decision.
- Single and bulk keep/reject/clear apply at once and show a toast with **Undo** (the previous decisions are restored through `set_decision`); the three bulk confirmation dialogs go. The quarantine confirmation stays, because it moves files.
- Focus stays on a sensible tile when a decided photograph leaves the current filter.

**Gate:** GATE-UI.
**Model:** sonnet

## P9 — Presets and plain-language summaries (U4, U5)

**Files:** `frontend/src/lib/pages/Cleaning.svelte`, `Sampling.svelte`, `Export.svelte`, `frontend/src/lib/clean.ts`, `frontend/src/lib/export.ts` + tests, locales.

- **Cleaning:** Lenient / Balanced / Strict presets over the three thresholds (Balanced = today's defaults); the fields move under "Advanced". The page opens on the photographs, not the form.
- **Export:** Share / Archive / Master presets over codec, resolution and quality; a preset whose codec is unavailable says so. The eight selects move under "Advanced", with the selection first.
- **Summary sentence** in Sampling and Export: photographs → frames → duration at the chosen rate, updating as settings change, through `Intl`.

**Gate:** GATE-UI.
**Model:** sonnet

## P10 — Open and reveal commands (U5)

**Files:** `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `crates/core/src/` (path lookup and checks, pure), `frontend/src/lib/ipc/app.ts`, `deny.toml` / licence files only if the audit asks, `docs/DECISIONS.md` (ADR-027 amended only with facts found).

- Add `tauri-plugin-opener` (name its licence from its manifest in the commit message; it must be on the `allow` list, otherwise stop and report).
- Commands `open_output(root, kind, id)` and `reveal_output(root, kind, id)`, `kind` ∈ export, contact-sheet, export-record. The core resolves the id to the stored path and refuses unless the path is canonical, exists, is a regular file, and lies inside `<root>/Timelapse exports/`. Errors cross IPC as `{ kind, args }`.
- The shell calls the plugin's Rust API. **No opener permission is added to `capabilities/default.json`**; a grep assertion checks it.
- Tests: a path outside the folder, a symlink pointing out, a missing file and an unknown id are refused.

**Gate:** full.
**Model:** opus

## P11 — Progress and the export finish (U5)

**Files:** `frontend/src/lib/components/Progress.svelte` (new), `frontend/src/lib/progress.ts` (new, pure) + test, `frontend/src/lib/pages/Export.svelte`, `Cleaning.svelte`, `Adjust.svelte`, `Organize.svelte`, `ContactSheets.svelte`, `frontend/src/lib/stores/export.svelte.ts`, locales.

- `Progress`: a determinate bar, done/total, and time remaining from a smoothed rate (`progress.ts`, tested; no estimate until it is stable). It replaces the five bare `<progress>` blocks.
- **During an export:** the thumbnail of the frame being encoded beside the bar.
- **When it completes:** a card with a poster frame (the middle frame's thumbnail), duration, size and codec, and the actions Play, Show in Finder / Show in folder (label by platform) and Contact sheet. The path stays copyable. Finished rows in the history get the same two actions.
- Failures of open/reveal say what to do.

**Gate:** GATE-UI.
**Model:** sonnet

## P12 — Motion, menu and shortcuts (U6)

**Files:** `frontend/src/App.svelte`, `frontend/src/lib/stores/nav.svelte.ts`, `frontend/src/lib/menu.ts` (new), `frontend/src/app.css`, `src-tauri/src/lib.rs` if the menu is built in Rust, locales.

- Page changes use the View Transitions API behind feature detection, with `--ez-duration-*` / `--ez-ease-*`; off under reduced motion (GUIDELINES §5.4).
- A native menu: Settings on ⌘, ; ⌘1–⌘5 for the workflow steps; Open folder on ⌘O. Labels come from Fluent in both locales and follow a language change. No capability beyond `core:default`; otherwise stop and report.
- The shortcuts appear in the menu and nowhere as the only way.

**Gate:** GATE-UI (+ GATE-FAST if Rust changed).
**Model:** sonnet

## P13 — Copy pass, en and fr-CA

**Files:** `locales/en/app.ftl`, `locales/fr-CA/app.ftl`, pages only where a key is renamed or removed.

- Read every string added or shown by P1–P12 as a first-time user: plain words, what to do next, no internal terms (hash, run, database, backfill) outside "Advanced" and technical details.
- English first, then `etzian:translator` for fr-CA. Remove keys no longer used. French strings that run long must wrap or show the full value on demand.

**Gate:** GATE-UI.
**Model:** sonnet

## P14 — Review slice

**Files:** whatever the findings touch; `docs/design-review.md`.

- `etzian:ui-reviewer` on every changed screen; `etzian:i18n-guardian` on the copy; `etzian:legal-compliance` (recent projects are newly stored data); `etzian:licence-audit` (the opener plugin).
- Fix the findings here. Record the pass in `docs/design-review.md`, stating plainly what was rendered and what was reviewed from source only.

**Gate:** full; reviewers sign off.
**Model:** opus

## P15 — Polish

**Files:** `docs/CURRENT_STATUS.md`, `docs/ARCHITECTURE.md`, `CLAUDE.md` (key paths, if they changed), `.claude/plan/STATE.md`.

- Documents brought up to date; the handoff log closed; `STATUS: DONE`.
- The version bump and changelog are left to `etzian:release` on the user's word. Nothing is pushed or tagged.
- **User items:** the rendered pass on the Mac (keyboard only, VoiceOver, light/dark, compact, reduced motion, fr-CA and en) with its result added to `docs/design-review.md`; Play and Show in Finder on a real export; the real-GoPro check carried over from Phase 8.

**Gate:** full.
**Model:** sonnet

---

## Out of scope

- Playing the exported video inside the app (asset protocol, codec support in the webview).
- Renaming selections by hand (a schema change); a notification when a job ends (a new plugin and capability).
- Any change in `../_uidesign`; any new dependency other than `tauri-plugin-opener`.
- Changes to the algorithms, the encode or the database schema.
