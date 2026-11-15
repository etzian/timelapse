# PHASES

Phases 0–6 are **complete** and archived in [archive/](./archive/). This file tracks **Phase 7 — Advanced UI & contact sheets**. The canonical scope and exit criteria stay in [docs/DEVELOPMENT_ROADMAP.md](../../docs/DEVELOPMENT_ROADMAP.md), quoted below; the slices implement that scope and nothing more.

> ## Phase 7 — Advanced UI & contact sheets
> **Scope:** Contact-sheet generation (PDF/image), per-page image + timestamp layout (`PRD.md:441`), advanced timeline UI, export-record (derived) preview per ADR-015.
> **Entry:** Phases 1–4 exit; ADR-015 adopted (export record is what the sheet/preview represents).
> **Exit:** Contact sheets are generated from SQLite state only; layout/timestamp configurable and both locales render correctly.

PRD §5.3 (timeline analysis), §5.11 (contact sheets) and §5.12 (export history, the derived export record) are the functional spec. ADR-014 makes contact sheets a core feature; ADR-015 makes the export record derived, never read back.

**User decisions, 2026-11-15:**
1. **Format: PDF only**, written with **`pdf-writer`** 0.15 (MIT OR Apache-2.0), the only new direct dependency. Text uses the PDF standard font Helvetica (WinAnsi encoding), so no font is bundled. It covers English and French completely; a character outside WinAnsi (for example a Japanese filename) prints as `?`. The review thumbnails are JPEGs already and are embedded as they are (no re-encode).
2. **Location:** written to `.timelapse/contact-sheets/` inside the project, like video exports, with a history on the page. No capability change.
3. **Sources:** all six from PRD §5.11:
   - all imported photographs;
   - a period (`from <= time < to`);
   - flagged photographs;
   - quarantined photographs;
   - a saved sampling selection, in its sequence order;
   - an export's final sequence (the selection that export encoded).

---

## Slices

Gate names are as before: `GATE-FAST` = fmt + clippy `-D warnings` + `cargo test`; `GATE-UI` = GATE-FAST + svelte-check / vitest / build + `verify.sh` + `ez i18n-check`; `GATE-FULL` = GATE-UI + `ez legal check/gen` + `ez licence-audit`.

### P7.0 — ADR-024, plan
ADR-024 records decisions 1–3, the layout model, what "from SQLite state only" means, determinism, the export record and the timeline additions.
**Gate:** GATE-FAST (docs only).

### P7.1 — Layout + PDF writer (pure)
`crates/core/src/contact_sheet/layout.rs` and `pdf.rs`:
- **Layout:**
  - page size A4 or Letter, portrait or landscape;
  - a grid of columns × rows (1–12 each), which sets the images per page;
  - caption: any of filename, timestamp (both, one, or neither);
  - a header (title, source line) and a footer (page n of m, generation note), from labels the caller passes already translated;
  - each thumbnail fitted into its cell, keeping its aspect ratio.
- **Text:** WinAnsi encoding (Latin-1 plus the 0x80–0x9F extras such as `€ œ – — ’ …`, others become `?`). Widths come from Helvetica's metrics for truncation with `…`, and each caption is clipped to its cell as a safety.
- **Streaming:** objects are written one at a time to the file with their offsets recorded, and the xref and trailer are written at the end. Memory stays bounded by one page, whatever the number of photographs.
- **Determinism:** no creation date or random id in the file, so the same input gives the same bytes.

Unit tests cover the grid geometry, pagination, aspect fit, WinAnsi mapping, truncation, a structurally valid PDF (every xref offset points to its object) and identical bytes for identical input.
**Gate:** GATE-FAST.

### P7.2 — Sources, generation, store v7
`crates/core/src/contact_sheet/mod.rs` and `store.rs`:
- **Sources:** `all` · `period { from, to }` · `flagged` · `quarantined` · `sampling { run }` · `export { export }`. Each is a query over SQLite only:
  - order: chronological, or the run's sequence order;
  - caption data: relative path, `normalized_datetime` (shown in UTC, as the app shows it);
  - image: the review thumbnail of the photo at its location of record.
- **Thumbnails:** the cached review thumbnail (256 or 1024, chosen from the cell size); made from the original only when not cached. An unreadable or missing photo gets a placeholder cell with its caption and a "not readable" note; the sheet still completes and counts it.
- **Generation:** in pages, with thumbnails made in parallel per page; progress and cancel between pages. Written to a temporary name, then renamed. Status `completed` · `failed` · `cancelled` (`interrupted` after a crash).
- **Schema v7:** `contact_sheets` (source, layout, labels, output path, counts, pages, file size, status, failure, times).
- **Limits:** at most 100 000 photographs per sheet (the design target); larger refused as `too-many`.

**Gate:** GATE-FAST.

### P7.3 — Export record (ADR-015)
`export::record(conn, id)` builds a derived, human-readable JSON record from SQLite only:
- the export: settings, FFmpeg version, argv, output, result probe, status;
- its sampling run: configuration and sequence fingerprint;
- the photo count.

It is shown on the Export page and can be written to `.timelapse/exports/<id>.record.json` on demand. It is never read back.
**Gate:** GATE-FAST.

### P7.4 — Timeline additions
- **Week granularity:** buckets start on Monday (`YYYY-MM-DD` of the Monday).
- **Identical timestamps:** groups of photographs that share one `normalized_datetime`, with counts (PRD §5.3).
- **A period's photographs:** paginated (`offset`, `limit`), with thumbnails.

**Gate:** GATE-FAST.

### P7.5 — Tauri commands
`contact_sheet_count`, `start_contact_sheet` (token + progress Channel), `cancel_contact_sheet`, `list_contact_sheets`, `contact_sheet_preview` (the first page's geometry: cells, image ids, captions, so the page draws it with the existing thumbnails; no PDF in the webview and no CSP change), `export_record`, `write_export_record`, `timeline_period`, `identical_timestamps`, `distribution` gains `week`. TS types and wrappers. No capability change.
**Gate:** GATE-UI.

### P7.6 — UI (both locales)
- **Contact sheets page:**
  - source, layout and caption choices, with the photo and page count and a live preview of the first page before generating;
  - progress and cancel, then the result;
  - a history with each sheet's path, which can be copied.
- **Shortcuts:** from Cleaning (flagged, quarantined), Sampling (a selection), Export (an export) and the timeline (a period).
- **Timeline:**
  - week granularity;
  - zoom: a click on a bar narrows to that period at the next finer granularity, with a breadcrumb back;
  - the photographs of the chosen period (paginated thumbnails);
  - identical-timestamp groups.
- **Export page:** the export record preview and "Save record".
- Every GUIDELINES §4 state. The data map gains `contact-sheets`, and `video-exports` mentions the saved record.

**Gate:** GATE-UI + `ez legal check/gen`.

### P7.7 — Exit tests + independent review
Integration tests on real files:
- each of the six sources gives the expected photographs in the expected order, from SQLite alone. After the thumbnails are cached, the originals are removed and the sheet is generated again with identical bytes;
- a missing, uncached photograph gives a placeholder cell, not a failure;
- layout and caption choices change the grid and the page count as configured;
- fr-CA and en labels (with `é`, `à`, `ç`, `’`) are encoded and present in the PDF;
- identical state and configuration give identical bytes;
- the PDF is structurally valid;
- cancellation leaves no partial file.

Then an independent reviewer pass.
**Gate:** reviewer sign-off; GATE-FULL; CI green on the macOS runner.

### P7.8 — Final gate, status, commit, push

---

## Out of scope
- PNG or other image pages; embedded fonts (non-WinAnsi characters print as `?`).
- Printing from the app, showing the PDF itself in the app, or opening it in another app (would need the opener plugin or a CSP change).
- Editing a sheet after generation; sheets across projects.
- Flicker and stabilisation views (Phase 8).
