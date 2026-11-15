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

## Review pass at the end of the milestone (P14, 2026-11-15)

**Reviewed from source only. Nothing was rendered.** Four reviewers read the milestone's diff (`30e7736..HEAD`): `etzian:ui-reviewer` (screens and components), `etzian:rust-reviewer` (the open and reveal commands, `project_summary`), `etzian:legal-compliance` (recent projects as newly stored data) and the i18n-guardian on the French copy (P13). `ez licence-audit .` passes; the opener plugin is `Apache-2.0 OR MIT`. These reviews found and fixed real problems, but they do not replace the rendered pass below.

### Fixed in P14

| # | Finding | Fix |
| --- | --- | --- |
| 1 | The app's sidebar icon rule redefined `.ez-sidebar__item::before`, which the design system uses as the active accent bar. | The icon is drawn on `::after`; the bar is untouched. |
| 2 | The fixed 720 px viewer modal with a five-button footer overflowed in fr-CA; "Undo decision" collided with the toast's "Undo". | Previous/Next and the position moved into the body; the footer holds three wrapping buttons; the button reads "Clear decision". |
| 3 | The triage grid used `role="option"` around buttons and a checkbox, so assistive technology could not reach them. | Plain list of `role="group"` tiles with the roving tab stop and all keys kept. |
| 4 | A focused selected tile lost its selection outline. | Selection uses a border and inset ring; focus keeps the system ring. |
| 5 | Undo existed only as a mouse button on an 8 s toast. | ⌘Z / Ctrl+Z on the page; 12 s when the decision came from the keyboard. |
| 6 | Focus fell to `<body>` after a page change from the menu or Next. | The new page heading takes focus; the sidebar and tab bar keep the user's place. |
| 7 | A repeated "job ended" message was not announced again; starts were never announced. | The region is cleared before each message; starts are announced. |
| 9, 10 | Disabling the focused button dropped focus; the disabled Next hid its reason. | `aria-disabled` with a guard; the reason is linked with `aria-describedby`. |
| 11 | A rescan blanked the overview and the skeleton showed at once. | Old figures stay; the skeleton waits 300 ms. |
| 12, 13, 14 | Live regions mounted with their text; recent-project rows had identical button names; the export finish card was out of sight. | One persistent status region; named buttons and focus after Remove; focus moves to the finish card. |
| 16, 17, 19 | Raw focus outlines; playback ran while the window was hidden; overlapping menu rebuilds. | System focus tokens; pause on hide; a sequence guard. |
| R1 | `project_summary` and open/reveal could migrate the database and move legacy folders under a "read-only" name. | A real read-only opener. An outdated database reads as "not imported" and opens nothing. |
| R2 | Any file type in the exports folder was handed to the OS. | Allow-list by kind (mp4, pdf, json); new reasons `unsupported-type` and `unsupported-name`. |
| L1 | A stored key could be missed by "Erase all data"; the erase also wrote settings back. | One key list, a test, and the write-back bug fixed. |
| L2 | The data map omitted the sidebar's folded groups. | Added; documents regenerated. |

### Open, not done here

- **Needs the design system:** sidebar state glyphs (○ ◐ ✓ ↻) are read aloud by VoiceOver and the sidebar takes only a text label; a `meta` or `description` slot would fix it (finding 8). The tab bar shows no state.
- **Not done:** wiring the native Edit > Undo item to the app's undo (part of 5); required field errors linked to inputs (15); per-frame retry and skeleton delay in flipbooks (18); raw `rem`, `px` and `vh` values to promote to tokens, histogram text, image alt wording, caps on long lists, a side-by-side compare, a named sidebar view transition, and `preventDefault` on sidebar links (20 to 26).
- **For the user and the lawyer:** whether launch-time reopening needs a setting or a notice (it is on by default and disclosed in the data map); whether a folder path counts as personal information for local-only data; the window-state file (position and size, written by the Tauri plugin, not erased by "Erase all data" and not in the data map; it predates this milestone); the privacy policy does not say the app hands exported files to the system's default player; all three legal templates are still unreviewed.

### Rendered pass

Not done. This is a user item (P15). Fill one row per check: keyboard only, VoiceOver, light and dark, compact density, reduced motion, fr-CA and en, 375 / 768 / 1440 widths, empty / one / few / many records. Points the source review could not settle: the sidebar icon and accent bar after the `::after` change; the viewer footer in fr-CA at the 520 px minimum; sidebar labels with the state suffix in French; VoiceOver on the triage tiles and the wipe slider; ⌘Z against the native Edit menu; `aria-disabled` buttons against disabled ones; contrast of muted text in dark mode; the native `<progress>` look in WKWebView.

| Check | Page | Result | Fix |
| --- | --- | --- | --- |
|  |  |  |  |

## For `@etzianco/ui` (not done here; the system is read-only)

- `Sidebar` could render `icon` as trusted markup, as its own doc comment says.
- A progress bar with time remaining and a flipbook (scrubber over a frame list) could be shared once two apps need them.
