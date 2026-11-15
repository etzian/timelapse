# STATE

STATUS: RUNNING
<!-- PENDING | RUNNING | DONE | BLOCKED -->

## Preconditions

- Phases 0–8 done on `main` (`dea573d`); their plan and state are in [archive/](./archive/).
- The user approved the UX milestone and `tauri-plugin-opener` on 2026-11-15.
- `make gate` was green at the end of Phase 8.

## Contract

- Repo root: `.` (`~/etzian/timelapse`). Branch: `ux-milestone`. Nothing is pushed or tagged unless the user says so.
- Product: **Timelapse Studio**, `com.etzian.timelapse`. Turn large photo collections into high-quality timelapse videos, locally.
- Goal of this milestone: the app remembers the project, shows the pipeline and the next step, plays the timelapse before export, makes triage fast, and finishes an export with something to watch.
- Stack: Rust workspace (`crates/core` pure, `src-tauri` shell), Svelte 5 + TS on `@etzianco/ui`, Fluent (fr-CA + en).
- UI/UX contract: `../_uidesign/GUIDELINES.md` and the `etzian:design-ui` skill. `../_uidesign` is **read-only**.
- Gates: as named in `PHASES.md` (`GATE-FAST`, `GATE-UI`, `GATE-FULL` = `make gate`). Redirect output to a file and read the tail.
- Invariants:
  - permissive licences only; no dependency other than `tauri-plugin-opener` is added;
  - `capabilities/default.json` does not change; a slice that would need more stops and reports;
  - `crates/core` has no `tauri` dependency;
  - every user-visible string is in both locales; errors cross IPC as `{ kind, args }`; numbers and dates go through `Intl`;
  - app CSS uses tokens only (`--app-*`, `.app-*` on `--ez-*`);
  - a change in what is stored or sent updates `legal/data-map.yaml` in the same commit;
  - no schema change and no change to the algorithms or the encode.
- Commits: Conventional Commits, one logical change each, named paths.
- Key paths: `frontend/src/lib/pages/`, `frontend/src/lib/components/`, `frontend/src/lib/stores/`, `locales/{en,fr-CA}/app.ftl`, `src-tauri/src/lib.rs`, `docs/DECISIONS.md` (ADRs), `docs/design-review.md`.

### Locked decisions

- **Opener (ADR-027, P1):** reached only through Rust commands that resolve an id to a stored path and check it lies inside `<root>/Timelapse exports/`; the webview gets no opener permission.
- **Recent projects:** `localStorage`, at most 8, erased with "app settings"; in the data map.
- **Flipbook:** plays cached thumbnails; no video is decoded in the webview.
- **Output folders (ADR-026):** `<root>/Timelapse exports/`, `<root>/Timelapse quarantine/`; `.timelapse/` holds only erasable project data.
- **Selections** keep their numeric id; the label shown is derived from method, count and date.
- **Copy:** authored in English, translated to Canadian French; internal terms (hash, run, database, backfill) stay out of the main flow.

## Phases

| # | Slice | Model | Gate | Status |
| --- | --- | --- | --- | --- |
| P1 | ADR-027 and baseline debts | sonnet | GATE-UI | done |
| P2 | Project store and continuity | sonnet | GATE-UI + GATE-FAST + legal | done |
| P3 | Home as the project overview | sonnet | GATE-UI | done |
| P4 | Pipeline navigation | sonnet | GATE-UI | pending |
| P5 | Next step and named selections | sonnet | GATE-UI | pending |
| P6 | Flipbook | sonnet | GATE-UI | pending |
| P7 | Flipbook in Adjust and Export, wipe | sonnet | GATE-UI | pending |
| P8 | Fast triage | sonnet | GATE-UI | pending |
| P9 | Presets and summaries | sonnet | GATE-UI | pending |
| P10 | Open and reveal commands | opus | full | pending |
| P11 | Progress and the export finish | sonnet | GATE-UI | pending |
| P12 | Motion, menu and shortcuts | sonnet | GATE-UI | pending |
| P13 | Copy pass, en and fr-CA | sonnet | GATE-UI | pending |
| P14 | Review slice | opus | full + sign-off | pending |
| P15 | Polish | sonnet | full | pending |

## Handoff log

### P1 — pass
- CHANGED: ADR-027 in docs/DECISIONS.md; new docs/design-review.md; Home.svelte tokens, formatNumber, .app-home__*; Sidebar footer (sun mark, "by Etzian") and per-destination icons; app-endorsement key in en and fr-CA.
- DECISIONS: The installed `Sidebar` renders `icon` as escaped text, not markup. Icons are drawn in app.css as `.ez-sidebar__item[href='#<page>']::before` with a currentColor mask over a data-URI SVG, and `icon` is not passed. A later phase that renames or adds a page must add a matching icon rule. ADR-027 names the P10 commands `open_output` and `reveal_output`, checked against `video_exports.output_path` and `contact_sheets.output_path`; P2 and P10 must follow that.
- DEFERRED: Home copy about hashing and a database goes to P3. Moving icons to the `icon` field is noted in design-review.md for the design system.
- MANUAL: none (sidebar icons and footer not rendered; covered by the P14/P15 rendered pass)


### P2 — pass
- CHANGED: core `project_summary` (read-only) + Tauri command; pure recent-list helpers `project.ts` with vitest; project store holds root, report, integrity, backfill, recent, status; restore on launch without rescan; unreachable-folder card (Retry, Remove); erase clears recent; data-map `recent-projects` entry, legal docs regenerated.
- DECISIONS: Reopen fills `project.backfill` with `{0, 0, summary}`; Home hides the resolved/unresolved line when both are 0. Reopen runs `integrityCheck` in the background, failure ignored. `project.report` is null after reopen (not persisted). Erase does not close the open project. localStorage key `timelapse.recent-projects`.
- DEFERRED: P3 redesigns Home (overview, copy, recent list placement).
- MANUAL: none (unreachable state and launch restore not rendered; P14/P15 pass)

### P3 — pass
- CHANGED: new `ProjectSteps.svelte` (one line per step with skeleton, error and refreshing states); Home rewritten (welcome with drop target and recent list; open project with title, Rescan, Switch project, steps, short summary, details disclosure, timeline); home-* keys in en and fr-CA, hashing and database wording removed.
- DECISIONS: Drag-drop works with `core:default` (`onDragDropEvent` uses only `listen`); capabilities unchanged. Drop is accepted anywhere in the window and a dropped file surfaces the import error. Figures come from `projectSummary`, `classifyImages`, `listSamplingRuns`, `listExports`; `classifyImages` runs each time the steps load. Exports counts completed ones only. Rescan is `importPath(root)` and bumps `scanVersion`. Recent list sits at the bottom of Home, excluding the open project. No new `--app-*` tokens.
- DEFERRED: Selections shows a run count, named labels come in P5. Photographs line has no link. Rendered pass (drop highlight, skeleton, fr-CA length) goes to P14/P15.
- MANUAL: none

## Open user items

- Real-GoPro visual check of deflicker and stabilization (from Phase 8).
- The rendered pass of this milestone on the Mac (P15).

## Roadmap

| Milestone | Status |
| --- | --- |
| Phases 0–8 (`docs/DEVELOPMENT_ROADMAP.md`) | done (2026-11-15) |
| UX milestone | **current** |
