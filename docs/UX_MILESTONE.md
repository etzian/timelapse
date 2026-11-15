# Timelapse Studio

## UX milestone — a guided, visual, fun app

**Status:** planned on 2026-11-15 and approved by the user the same day. Not started.
**Governing docs:** `.claude/plan/PHASES.md` holds the slices and their gates and is authoritative; `.claude/plan/STATE.md` holds their status. This document records why the milestone exists, what it changes and what was decided. It follows Phases 0–8 of `DEVELOPMENT_ROADMAP.md`.
**Branch:** `ux-milestone`.

---

## 1. Why

A review of the interface on 2026-11-15 (from source, against `@etzianco/ui` and its guidelines; the app was not rendered) found that the engineering is careful, with states, stale-response guards and both locales, but the app reads as a set of forms around a pipeline and never shows a moving image.

| # | Finding | Where |
| --- | --- | --- |
| 1 | The project is forgotten. The folder is held in memory, so every launch starts at the folder picker. The import report and the timeline are local state of Home and disappear when the user leaves the page. | `stores/project.svelte.ts`, `pages/Home.svelte` |
| 2 | The pipeline is invisible. The sidebar is eight flat labels without icons. Nothing shows the order (import, clean, select, adjust, export), what is done or what comes next. Organize and Contact sheets are side tools placed in the middle. | `App.svelte`, `stores/nav.svelte.ts` |
| 3 | There is no motion before export. The only way to see a result is to export, copy a path out of a banner and find the file. | Sampling, Adjust, Export |
| 4 | Parameters come before pictures. Cleaning opens on three luminance thresholds, Sampling on method fields, Export on eight selects of equal weight. Selections are named by number. | Cleaning, Sampling, Export |
| 5 | Triage is slow. Each photograph takes several clicks, a bulk keep or reject opens a confirmation although it can be undone, and the inspector has no previous/next or decision buttons. | `pages/Cleaning.svelte` |
| 6 | Long jobs are quiet. Progress is a bare bar with "done / total" and no time remaining. An export survives a page change, but nothing in the navigation shows it. | Cleaning, Adjust, Export, Organize, Contact sheets |
| 7 | Contract debts: raw `rem`/`em` values and unformatted numbers on Home; Home copy about hashing and a database; no sidebar footer with the sun mark and the "by Etzian" endorsement; no native menu; no `docs/design-review.md`. | `pages/Home.svelte`, `App.svelte`, `src-tauri/src/lib.rs` |

## 2. What changes

- **Project continuity.** The last project reopens without a rescan; recent projects are listed; Home becomes the project's overview and accepts a dropped folder.
- **A guided pipeline.** The sidebar groups the workflow (Import, Clean, Select, Adjust, Export) apart from the tools (Organize, Contact sheets) and Settings. Each step shows its state, each page ends with the next step, and a running job shows in the navigation.
- **Flipbook.** A saved selection, or a preview of one, plays from the cached thumbnails with a scrubber, in Sampling, Adjust and Export. In Adjust, the before and after stills become one frame with a draggable divider.
- **Fast triage.** Arrow keys, `A` / `R` / `U`, an inspector with previous/next and decisions, and an undo toast in place of the confirmation dialogs.
- **Presets and plain language.** Lenient / Balanced / Strict for cleaning and Share / Archive / Master for export, with today's fields under "Advanced"; a sentence such as "12,400 photos → 1,033 frames → 34 s at 30 fps" that follows the settings.
- **An export worth finishing.** A progress bar with time remaining and the frame being encoded, then a card with a poster frame and the actions Play and Show in Finder.
- **Polish.** View transitions, a native menu with ⌘ shortcuts, and a copy pass in en and fr-CA.

## 3. Decisions (user, 2026-11-15)

1. The plan below is approved.
2. **`tauri-plugin-opener` is approved** for Play and Show in Finder. It is reached only through Rust commands that resolve an id to a path stored in SQLite and refuse anything outside `<root>/Timelapse exports/`. The webview gets no opener permission and `capabilities/default.json` does not change. ADR-027 records this (slice P1).
3. **Recent projects are stored on the device**: folder paths in `localStorage`, at most 8, erased with "app settings". `legal/data-map.yaml` changes in the same commit.
4. The progress bar and the flipbook are `.app-*` components in this repo. `@etzianco/ui` stays read-only; what it could adopt is listed in `docs/design-review.md`.

Choices made while cutting the plan, open to change before their slice runs:

- Selections keep their number and get a derived label (method, count, date), which avoids a schema change.
- Adjust plays a loop of at most 60 frames around the chosen frame, as shot or adjusted; Export plays the whole selection as shot.
- The version bump is left to `etzian:release`.

## 4. Plan

**Exit test:** on a relaunch the last project opens without the folder picker; from Home, with the keyboard only, the user reaches a saved selection, plays it in the flipbook, exports it, and reveals the video in the file manager; the same holds in fr-CA and en; `make gate` is green.

| Slice | Theme | Content | Gate |
| --- | --- | --- | --- |
| P1 | Baseline | ADR-027; Home token and number debts; sidebar footer endorsement and icons; `docs/design-review.md` | GATE-UI |
| P2 | Continuity | Project store; last and recent projects; reopen without a rescan; unreachable-folder state; data map | GATE-UI + GATE-FAST + legal |
| P3 | Continuity | Home as the project overview; drop a folder; plainer copy | GATE-UI |
| P4 | Pipeline | Sidebar groups, step states, running-job indicator | GATE-UI |
| P5 | Pipeline | "Next" action on each page; derived selection labels | GATE-UI |
| P6 | Flipbook | The component and its pure logic; in Sampling | GATE-UI |
| P7 | Flipbook | In Adjust and Export; the before/after wipe | GATE-UI |
| P8 | Triage | Keyboard decisions, inspector, undo toasts | GATE-UI |
| P9 | Presets | Cleaning and export presets; summary sentences | GATE-UI |
| P10 | Export finish | `open_output` / `reveal_output` commands with path checks | full |
| P11 | Export finish | Progress with time remaining; completion card | GATE-UI |
| P12 | Polish | View transitions, native menu, shortcuts | GATE-UI |
| P13 | Polish | Copy pass, en then fr-CA | GATE-UI |
| P14 | Review | `etzian:ui-reviewer`, `etzian:i18n-guardian`, `etzian:legal-compliance`, `etzian:licence-audit`; findings fixed | full + sign-off |
| P15 | Close | Documents, state, user items | full |

Gate names are defined in `.claude/plan/PHASES.md`; "full" is `make gate`.

**Invariants for every slice:** no dependency other than `tauri-plugin-opener`; no capability change; no schema change; no change to the algorithms or the encode; `crates/core` without `tauri`; every string in both locales; tokens only in app CSS.

## 5. Out of scope

- Playing the exported video inside the app (asset protocol, codec support in the webview).
- Renaming selections by hand (a schema change).
- A notification when a job ends (another plugin and capability).
- Any change in `../_uidesign`.

## 6. For the user

- The rendered pass on the Mac at the end of the milestone: keyboard only, VoiceOver, light and dark, compact density, reduced motion, fr-CA and en. Its result goes in `docs/design-review.md`.
- Play and Show in Finder on a real export.
- The real-GoPro check of deflicker and stabilization, carried over from Phase 8.
