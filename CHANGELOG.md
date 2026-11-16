# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project uses [Semantic Versioning](https://semver.org/).

## [0.2.0] — 2026-11-16

### Added
- **ux:** P1 ADR-027 and baseline debts
- **ux:** P2 project store and continuity
- **ux:** P3 Home as the project overview
- **ux:** P4 pipeline navigation
- **ux:** P5 next step and named selections
- **ux:** P6 flipbook
- **ux:** P7 flipbook in Adjust and Export, before/after wipe
- **ux:** P8 fast triage
- **ux:** P9 presets and plain-language summaries
- **ux:** P10 open and reveal commands
- **ux:** P11 progress and the export finish
- **ux:** P12 motion, native menu and shortcuts
- **ux:** P13 copy pass in en and fr-CA
- **shell:** Size the first window to the screen

### Fixed
- **outputs:** Open the project database read-only and allow only known file types
- **ui:** Apply the P14 review findings on accessibility and erase
- **ui:** Move Next above the file list, drop the double scrollbar, even the card padding
- **ui:** Fixed top-right Next, single scroll area, no sideways overflow

### Documentation
- **plan:** Block P4 on the unpublished @etzianco/ui 2.6 Sidebar groups
- **legal:** Record the sidebar fold state in the data map
- **review:** Record the P14 review pass
- Close the UX milestone

## [0.1.0] — 2026-11-15

### Added
- Phase 0 foundation
- Phase 1 — photo import pipeline (folder scan, SHA-256 dedup, SQLite image registry)
- Phase 2 — EXIF & timeline (capture-time resolution, schema v2, timeline queries and view)
- Phase 3 — brightness analysis & cleaning (luma analysis, XMP rejects, reproducible classification, recoverable quarantine)
- Phase 4 — temporal sampling (fixed interval, every Nth, uniform; reproducible saved runs)
- Phase 5 — FFmpeg video generation (LGPL FFmpeg 9.0.2 + SVT-AV1, VideoToolbox, verified MP4 export)
- **consolidate:** Plan, copy/move into a working folder, undo, verify (schema v6)
- **shell:** Consolidation commands (preview, start, cancel, undo, list, verify)
- **ui:** Organize page — dry run, copy/move with confirmation, history with check and undo, originals check
- **contact-sheet:** Page layout, WinAnsi text and a streamed PDF writer
- **contact-sheet:** Six sources, generation and schema v7
- **export:** Derived export record, shown and saved on demand (ADR-015)
- **timeline:** Week buckets, a period's photographs and identical times
- **shell:** Contact sheet, export record and timeline commands
- **ui:** Contact sheets page, timeline zoom and periods, export record
- **cleaning:** Keep, reject or clear several photographs at once
- **cleaning:** Stop the brightness analysis
- **sampling:** From and To start at the photographs' first and last day
- **adjust:** Deflicker gains — luminance and colour against their slow trend
- **adjust:** Translation stabilization — block matching, smoothed path, fixed crop
- **adjust:** Deflicker and stabilization runs, schema v8
- **export:** Apply deflicker gains and stabilization shifts at encode (D5)
- **shell:** Deflicker and stabilization commands, chart series, before/after
- **ui:** Adjust page — deflicker and stabilization with charts and before/after
- **core:** Visible exports and quarantine folders, erasable project data (ADR-026)
- **settings:** Choose what Erase all data removes

### Fixed
- **export:** Review findings — serial encode tests, honest VideoToolbox skip, panic/cancel safety, no-symlink LGPL bundle
- **export:** Never remove a pre-existing .part; clear another project's result; record Phase 5 done
- **consolidate:** Review findings — an undo never removes a location of record; reconcile never removes or claims a file in a user folder
- **consolidate:** Second review — undo keeps a copy that is no longer the file it made; restore targets are user locations
- **consolidate:** Compare names NFC-normalised as well as case-insensitively
- **contact-sheet:** Review findings — count, races, plurals, error kinds
- **import:** Large imports no longer time out or freeze the window
- **cleaning:** Use the design-system z-index token for the selection checkbox
- **import:** Skip hidden files such as macOS `._*` AppleDouble files
- **adjust:** Review findings — corrected ends, stale runs, colour keeps luminance

### Documentation
- Record Phase 2 timeline decisions
- **release:** Attach the exact FFmpeg sources and verify VideoToolbox before a release
- **plan:** Phase 6 plan and ADR-023 (consolidation: picked folder, copy/move, clones never links, undo)
- **plan:** Record Phase 6 done
- **release:** Check consolidation onto an exFAT drive on a real Mac
- **plan:** Phase 7 plan and ADR-024 (contact sheets, export record, timeline)
- **plan:** Record Phase 7 done
- **plan:** Phase 8 plan and ADR-025 (deflicker and stabilization applied at encode)
- **plan:** Record Phase 8 done (the real-GoPro check stays the user's)
- **plan:** Cut the UX milestone into 15 slices
- Record the UX milestone (findings, decisions, plan)

