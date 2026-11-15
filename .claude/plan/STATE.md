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
| P4 | Pipeline navigation | sonnet | GATE-UI | done |
| P5 | Next step and named selections | sonnet | GATE-UI | done |
| P6 | Flipbook | sonnet | GATE-UI | done |
| P7 | Flipbook in Adjust and Export, wipe | sonnet | GATE-UI | done |
| P8 | Fast triage | sonnet | GATE-UI | done |
| P9 | Presets and summaries | sonnet | GATE-UI | done |
| P10 | Open and reveal commands | opus | full | done |
| P11 | Progress and the export finish | sonnet | GATE-UI | done |
| P12 | Motion, menu and shortcuts | sonnet | GATE-UI | done |
| P13 | Copy pass, en and fr-CA | sonnet | GATE-UI | done |
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

### P4 — blocked, then unblocked (lockfile bumped to @etzianco/ui 2.6.1, commit separate)
- BLOCKER (verbatim from runner): The P4 spec needs Sidebar `groups`. The installed @etzianco/ui is 2.5.3, and its Sidebar.svelte takes only flat `items` `{label, icon?, href?, onclick?}` with no `groups`, no badge and no per-item state slot. `groups` exists only in ../_uidesign v2.6.0/2.6.1 (read-only). `npm view @etzianco/ui versions` lists up to 2.5.3 only, so 2.6.x is not published and the dependency cannot be bumped.
- Options for the user: (a) publish @etzianco/ui 2.6.x, bump the pin in frontend/package.json and the lockfile, then re-run P4; (b) run P4 against 2.5.3 with flat items in the order Import, Clean, Select, Adjust, Export, Organize, Contact sheets, Settings, no group headings, step state as glyph in `icon` and a word in `label`, with an icon rule per page in app.css.
- No files changed by the runner.

### P4 — pass
- CHANGED: pure `pipeline.ts` (stepStates, endedJobs, JOB_PAGE) with 6 vitest cases; `pipeline.svelte.ts` store; Sidebar `groups` (Workflow, Tools, App) and TabBar in the same order; state suffix on labels; sr-only live region announcing job ends; Cleaning reports analysis start/end; verb nav labels and page titles in en and fr-CA.
- DECISIONS: 2.6.1 Sidebar still renders `icon` as escaped text, so the CSS icon rules in app.css stay and `icon` is not passed. Sidebar renders `items` before `groups`, so Settings sits in its own "App" group to stay last. Import is done when photos exist; Clean is in progress if some photos are unmeasured or flagged; Select, Adjust, Export are done when a selection, adjustment or completed export exists. Sidebar state is text in the label (e.g. "Clean · ◐ in progress"); TabBar has no state suffix. Nav figures refresh on root, page or running-job change; a failed refresh keeps the previous figures.
- DEFERRED: The state glyph is read literally by VoiceOver; a proper icon or aria slot needs a design-system change. Rendered pass (sidebar width with long fr-CA labels) goes to P14/P15.
- MANUAL: none

### P5 — pass
- CHANGED: pure `selectionLabel` in sampling.ts and `selectionLabel.ts` binding (t, formatNumber, formatDate); pure `nextStep` in pipeline.ts; new `NextStep.svelte` placed at the end of Home, Cleaning, Sampling, Adjust, Export; labels used in Sampling, Adjust, Export, Organize, Contact sheets selects and lists; selection-*, next-* keys in en and fr-CA.
- DECISIONS: Chain is home→cleaning→sampling→adjust→export→organize. Readiness: Home needs photos; Clean needs photos and no analysis running; Select and Adjust need a saved selection; Adjust also waits while an adjustment runs. The id stays secondary ("(#id)" in selects). Label dates are day plus short month, year only if it differs from now. The Button component has no aria-describedby passthrough, so the reason is plain adjacent text.
- DEFERRED: Home's Selections line still shows a run count. Two selections can share a label (same method, count, day); only "#id" tells them apart. Rendered pass (fr-CA label length in selects, disabled Next look) goes to P14/P15.
- MANUAL: none

### P6 — pass
- CHANGED: pure `flipbook.ts` (frameIntervalMs, wrapIndex, clampIndex, windowIndexes, planLoad, advance) with 12 vitest cases; `Flipbook.svelte` (screen, range scrubber, frame number and capture time, first/prev/play/next/last, Space/arrows/Home/End; empty, buffering, failed, partial states); placed above the strip in Sampling; 15 flipbook-* keys in en and fr-CA.
- DECISIONS: No batched IPC and no Rust change: frames load in batches of 8 with 4 concurrent `getThumbnail` calls. P7 reuses `<Flipbook root frames={{image_id, epoch_ms}[]} fps speed total resetKey/>`; `resetKey` must change for a different sequence. Never autostarts (covers reduced motion). Arrow/Home/End skipped on the range input, Space skipped on buttons. A saved selection plays the pages already loaded. Orchestrator added a svelte-ignore on the section keydown (svelte-check warning the runner missed; keys bubble from child controls).
- DEFERRED: Rendered checks (WebView, VoiceOver, fr-CA button widths, dark mode) go to P14/P15. No 300 ms loading-indicator rule yet.
- MANUAL: none

### P7 — pass
- CHANGED: new `Wipe.svelte` (role=slider divider with pointer and keyboard, As shot / Corrected tags in text); pure `wipe.ts` and bounded `frameCache.ts` with tests; `loopWindow` (max 60) in flipbook.ts; Flipbook gained an optional `loadFrame(i)` prop (default getThumbnail, Sampling unchanged); `cachedAdjustedFrame` in adjust store; Adjust compare uses Wipe and gains a "Play the loop" card (as shot / corrected); Export gains a Preview flipbook as shot; wipe-*, adjust-loop-*, export-preview-* keys in en and fr-CA.
- DECISIONS: `loadFrame` is the way to play non-thumbnail frames. The Adjust loop's resetKey includes root, run, window start, adjusted flag and applied run ids. Wipe and loop share one adjusted-frame cache (bounded 240). Export preview is capped at 1000 frames (partial note shows the total). No Rust, capability, dependency or data-map change.
- DEFERRED: Rendered checks (wipe drag in the WebView, VoiceOver on the slider, fr-CA length, dark mode) go to P14/P15. Switching as shot / corrected resets to the loop start.
- MANUAL: none

### P8 — pass
- CHANGED: pure triage logic in clean.ts (columnsFromTops, moveIndex, focusAfterRemoval, neighbourId, decisionApplies, previousDecision, planDecision) with 19 vitest cases; new `Inspector.svelte` (Previous/Next, Keep/Reject/Undo, "n of N", moves on after a decision); Cleaning grid is role=listbox with roving tabindex, keys arrows/Home/End/A/R/U/Space/Enter/X, key legend, immediate decisions with an 8 s Undo toast through `set_decision`, three bulk dialogs removed, quarantine modal kept; `<Toast />` mounted once in App.svelte; clean-* keys in en and fr-CA.
- DECISIONS: Toast is mounted once; later phases call `toast.*` from @etzianco/ui. Keys act on the focused tile only; bulk buttons act on ticked photos; X ticks. Per-tile mouse buttons are native `ez-btn` buttons with tabindex=-1 (ui Button forwards no tabindex). Undo is a toast action, not Cmd+Z. No-op decisions are skipped without a toast. Analysis start/end still reported to the pipeline store; NextStep unchanged.
- DEFERRED: fr-CA copy polish (P13). `clean-close` key is unused (the viewer uses the Modal's own close); remove in P13. Modal footer may wrap tightly in fr-CA with 5 buttons. Rendered keyboard and VoiceOver pass goes to P14/P15.
- MANUAL: none

### P9 — pass
- CHANGED: CLEAN_PRESETS (Lenient 25/30/0.8, Balanced 40/30/0.6 = core defaults, Strict 60/30/0.4) with matchCleanPreset and presetThresholds; EXPORT_PRESETS (Share h264/1080/standard, Archive av1/1080/standard, Master hevc/2160/high) with matchExportPreset, presetAvailable, summarize, summaryText; vitest cases for edges and refusals; Cleaning presets row with the thresholds under "Advanced settings"; Export selection first, presets (disabled with a note when the codec is missing), summary sentence, other selects under Advanced; Sampling duration line is now the summary sentence; keys in en and fr-CA.
- DECISIONS: A preset click applies at once; the shown preset is matched against applied values, else "Custom settings". Presets do not touch the Lightroom flag, fps, speed or adjustments. Export defaults (av1/1080/standard) equal Archive. Preset buttons are native `ez-btn` (ui Button has no aria-pressed). Nothing new persisted, so the data map is unchanged. Balanced checked against `Thresholds::default` by the orchestrator.
- DEFERRED: Unused `sampling-duration` key and `estimatedDurationSeconds` (still tested) go in P13. fr-CA preset names ("Copie maître", "Indulgente") are drafts for P13. The Sampling summary now rounds frames up like the encoder.
- MANUAL: none

### P10 — pass
- CHANGED: `tauri-plugin-opener` 2.7.3 (Apache-2.0 OR MIT) plus transitive open 5.4.5, is-docker 0.2.0, is-wsl 0.4.0 (MIT), all on the allow list; new `crates/core/src/outputs.rs` (`OutputKind`, `resolve`, 5 tests) and `Error::Output { reason }`; `record_path_for` pulled out of export/record.rs; commands `open_output` / `reveal_output` using the plugin's Rust API on the blocking pool, plugin registered with `open_js_links_on_click(false)`; shell test `the_webview_has_no_opener_permission`; `openOutput` / `revealOutput` wrappers; `error-output` key in en and fr-CA; "As built (P10)" paragraph in ADR-027.
- DECISIONS: Refusal reasons are `unknown`, `missing`, `not-a-file`, `outside-exports`, `open-failed`. P11 calls `openOutput(root, 'export' | 'contact-sheet' | 'export-record', id)` and shows errors through `t('error-output', args)`. An export-record is opened by the export's id; if no record file exists the reason is `missing`, so P11 calls `writeExportRecord` first. Export status is not checked; the file checks guard. Non-UTF-8 paths are refused as `not-a-file`. No opener permission in capabilities (grep and test). No data-map change. Orchestrator re-ran `make gate`: exit 0.
- DEFERRED: fr-CA wording of `error-output` is a draft for P13. A real Open / Show in Finder click is checked in P14/P15.
- MANUAL: none

### P11 — pass
- CHANGED: pure `progress.ts` (smoothed rate, remainingSeconds, fraction; no estimate before 4 samples and 3 s) with 7 tests; `Progress.svelte` replacing all 7 bare `<progress>` blocks in Export, Cleaning, Adjust, Organize, Contact sheets; `ExportActions.svelte` (Play, Show in Finder / Show in folder, Contact sheet; inline role=alert on failure); Export completion card (poster, duration, size, codec, copyable path, Dismiss) and history rows with the same actions; export store `thumb` (one request per 800 ms) and `posterFor`; tokens --app-progress-thumb and --app-poster-width; progress-*, output-* keys, error-output reasons reworded to say what to do.
- DECISIONS: ExportProgress carries only {done,total}, so the in-export thumbnail is the selection item at the same fraction and the poster is the middle item; both approximate. Open/reveal failures are inline alerts (the runner found no toast store; P8 mounted `<Toast />` so P12/P13 may switch if wanted). The export-record kind is not offered, so writeExportRecord is not called. Platform label uses navigator.userAgent. Only Export shows a thumbnail. Progress is not a live region (P4 announcements reused). Failed/cancelled results still use the Banner.
- DEFERRED: The open-failed hint mentions Finder on every OS (macOS-first). P13 reviews the new strings.
- MANUAL: In the real app with a finished export: Play opens the default player, Show in Finder selects the file, the poster shows, the thumbnail beside the bar moves during an export. Check a missing-file error in en and fr-CA.

### P12 — pass
- CHANGED: pure `menu.ts` (menu model from NAV_ORDER and a label function, shortcut helpers) with 4 vitest cases; `menuNative.ts` builds the native menu with `@tauri-apps/api/menu` and `setAsAppMenu()`, rebuilt on locale change; App.svelte swaps pages in a feature-detected View Transition, skipped under reduced motion; `requestImport` / `importRequested` / `clearImportRequest` in nav store, and Home opens the existing `doImport` picker on request; view-transition CSS on `--ez-duration-fast` and `--ez-ease-in-out`; menu-* keys in en and fr-CA.
- DECISIONS: Menu is built in JS; `core:default` includes `core:menu:default` (checked by the orchestrator in the generated schema), so capabilities/default.json is unchanged and no Rust was touched. Shortcuts: CmdOrCtrl+1–5 for the workflow steps, CmdOrCtrl+, Settings, CmdOrCtrl+O Open folder. App.svelte renders `shown` (updated inside the transition); the sidebar highlight follows `nav.active` immediately. Predefined items (Hide, Quit, Undo, Copy, Window) keep OS labels. Every menu action also has a visible control.
- DEFERRED: Localizing the predefined menu item labels; per-page `view-transition-name` for shared elements.
- MANUAL: In the real app (`make dev`): the menu shows with shortcuts; ⌘1–⌘5, ⌘, and ⌘O work; labels switch with the language; the cross-fade is absent under reduced motion.

### P13 — pass
- CHANGED: about 48 English strings reworded to plain words (no database, inventory, fingerprint, consolidation, run, FFmpeg or "Sampling page" outside Advanced; page names used instead); "analyse" normalized to "analyze" in text; 6 unused keys removed in both locales (clean-close, sampling-duration, clean-detail-status, clean-thresholds-title, clean-preset-current, export-preset-current) and the unused `estimatedDurationSeconds` with its test; fr-CA run through `etzian:translator` (190 keys) then `etzian:i18n-guardian`; must-fix findings applied (no hardcoded "Une" in `[one]` variants, "{ $job } : terminé.", summary-line, progress-remaining, clean-inspector-keys wording, non-breaking spaces before colons, inside « » and in "nº { $id }" / "{ $n } s|min|h"), plus Exportations, Étapes, non commencé, "dans « path »", dimensions paires.
- DECISIONS: Dynamic key families (error-*, nav-*, files-failure-*, organize-*, clean-filter-*, clean-done-*, preset/codec help) kept. `error-output` [open-failed] no longer names Finder. Message IDs unchanged (adjust-analyse, adjust-analysing, next-reason-analysing keep the old spelling). Orchestrator re-ran check, test, build, verify, i18n-check after the fr-CA fixes.
- DEFERRED: "Dry run" and "Preview" wording on Organize; technical Timeline strings (EXIF source names, "Dates are shown in UTC"); "Analysis" as a noun in job names. Guardian nice-to-haves left: `selection-every-*` for n=1 ("Toutes les 1 s"), `pipeline-state-running` vs `in-progress` wording overlap, "par Organiser" phrasing, "timelapse" vs "vidéo accélérée" in sampling-intro, "Fréquence" vs "cadence", `wipe-hint`. Lawyer review of generated legal documents still pending.
- MANUAL: Read the French in the running app for line length (Sidebar, menu, Modal footer with 5 buttons) in P14/P15.

## Open user items

- Real-GoPro visual check of deflicker and stabilization (from Phase 8).
- The rendered pass of this milestone on the Mac (P15).

## Roadmap

| Milestone | Status |
| --- | --- |
| Phases 0–8 (`docs/DEVELOPMENT_ROADMAP.md`) | done (2026-11-15) |
| UX milestone | **current** |
