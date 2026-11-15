# STATE

STATUS: PENDING
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
| P1 | ADR-027 and baseline debts | sonnet | GATE-UI | pending |
| P2 | Project store and continuity | sonnet | GATE-UI + GATE-FAST + legal | pending |
| P3 | Home as the project overview | sonnet | GATE-UI | pending |
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

_(empty)_

## Open user items

- Real-GoPro visual check of deflicker and stabilization (from Phase 8).
- The rendered pass of this milestone on the Mac (P15).

## Roadmap

| Milestone | Status |
| --- | --- |
| Phases 0–8 (`docs/DEVELOPMENT_ROADMAP.md`) | done (2026-11-15) |
| UX milestone | **current** |
