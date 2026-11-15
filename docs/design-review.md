# Design review

Review of the interface against `@etzianco/ui` and `../_uidesign/GUIDELINES.md`. The rendered pass is recorded in the second table, at the end of the UX milestone (P14).

## Baseline: source review, 2026-11-15

**Not rendered.** This review was made from source only. The app was not run, so focus order, contrast, layout at the supported widths and screen-reader behaviour are unchecked. A review from source is weaker than a rendered pass.

| # | Finding | Where | Slice |
| --- | --- | --- | --- |
| 1 | The project is forgotten: the folder is held in memory, and the import report and timeline are local state of Home. | `stores/project.svelte.ts`, `pages/Home.svelte` | P2, P3 |
| 2 | The pipeline is invisible: eight flat labels, no icons, no order, no state, no next step. | `App.svelte`, `stores/nav.svelte.ts` | P1, P4, P5 |
| 3 | No motion before export; the only way to see a result is to export and find the file. | Sampling, Adjust, Export | P6, P7, P10, P11 |
| 4 | Parameters come before pictures; selections are named by number. | Cleaning, Sampling, Export | P5, P9 |
| 5 | Triage is slow: several clicks per photograph, a confirmation for undoable bulk decisions, no previous/next in the inspector. | `pages/Cleaning.svelte` | P8 |
| 6 | Long jobs are quiet: a bare bar, no time remaining, nothing in the navigation. | Cleaning, Adjust, Export, Organize, Contact sheets | P4, P11 |
| 7 | Contract debts: raw `rem`/`em` and unformatted numbers on Home (fixed in P1); Home copy about hashing and a database (P3); no sidebar footer with the sun mark and endorsement (P1); no native menu (P12); no design review (this file). | `pages/Home.svelte`, `App.svelte`, `src-tauri/src/lib.rs` | P1, P3, P12 |

Note on P1: `Sidebar` shows `icon` as text, not markup, so the icons are drawn in `app.css` as masked inline SVGs (`currentColor`) keyed on each item's `href`. If `@etzianco/ui` rendered trusted markup, this would move to the `icon` field.

## Rendered pass (P14)

Not done yet. Fill one row per check: keyboard only, VoiceOver, light and dark, compact density, reduced motion, fr-CA and en, 375 / 768 / 1440 widths, empty / one / few / many records.

| Check | Page | Result | Fix |
| --- | --- | --- | --- |
|  |  |  |  |

## For `@etzianco/ui` (not done here; the system is read-only)

- `Sidebar` could render `icon` as trusted markup, as its own doc comment says.
- A progress bar with time remaining and a flipbook (scrubber over a frame list) could be shared once two apps need them.
