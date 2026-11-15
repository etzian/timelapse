# PHASES

Phases 0–2 are **complete** and archived in [archive/](./archive/). This file tracks the milestone in work: **Phase 3 — Brightness analysis & cleaning**. The canonical scope and exit criteria stay in [docs/DEVELOPMENT_ROADMAP.md](../../docs/DEVELOPMENT_ROADMAP.md), quoted below. This file does **not** re-scope the phase: the slices implement that scope and its exit criterion, nothing more.

> ## Phase 3 — Brightness analysis & cleaning
> **Scope:** Lightroom-sidecar / brightness heuristics, clean/quarantine policy, `image_quality` state in SQLite. (Pipeline §4.)
> **Entry:** Phase 2 exit; **Phase 6 file-management safety primitives landed** (quarantine must be recoverable).
> **Exit:** Cleaning decisions are reproducible from SQLite + original files; quarantined images are recoverable without data loss.

**User decisions, 2026-11-14 (recorded here and in ADR-020):**

1. **Entry criterion:** Phase 3 lands the *minimal* Phase 6 safety primitive that quarantine needs: a `file_operations` journal, a hash-verified move, restore, and crash reconciliation. Consolidation, copy and hardlink/symlink policy stay in Phase 6.
2. **Dependencies:** `image` 0.25 (default features off; `jpeg`, `png`, `tiff` only) and `rayon` 1. Both are MIT OR Apache-2.0; every transitive licence is checked by `ez licence-audit`.
3. **Quarantine** physically moves the original to `<root>/.timelapse/quarantine/<relative path>`, only after explicit confirmation. Each move is hash-verified and journaled, restore is one click, and nothing is ever overwritten.
4. **Lightroom:** XMP is read only, from a `<stem>.xmp` sidecar or the packet embedded in the file. A reject flag (`xmp:Rating = -1`) becomes a review reason. Nothing is ever written. No XML dependency.

**Invariants for this phase:** originals are never written (ADR-006). A flagged image is never moved without confirmation (ADR-011). No new Tauri capability or plugin. The CSP already allows `data:` images, so thumbnails cross IPC as small JPEG bytes. Each derived value records its algorithm version (DATABASE_SCHEMA.md §5).

---

## Slices

A slice is *done* only when its gate has actually run and passed. The gate names are the same as in Phase 2: `GATE-FAST` = `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`. `GATE-UI` = GATE-FAST + `npm --prefix frontend run check|test|build` (from PowerShell on this host) + `scripts/verify.sh` (Git Bash) + `ez i18n-check .`. `GATE-FULL` = GATE-UI + `ez legal check .` + `ez legal gen .` + `ez licence-audit .`.

### P3.1 — Dependencies, ADR-020, scan exclusion
Add `image` (no default features; `jpeg`, `png`, `tiff`) and `rayon` to `crates/core`. Write ADR-020, covering the libraries, the quarantine model and the decisions above. Make the Phase 1 scan skip `<root>/.timelapse/`, so quarantined files and thumbnails are never re-imported, and add a test for it.
**Gate:** GATE-FAST + `ez licence-audit .`.

### P3.2 — Luminance analysis unit
`crates/core/src/clean/analysis.rs`: decode each image at analysis resolution (longest side ≤ 512 px) and compute Rec. 709 luma on the 8-bit encoded values. The measurements are mean, median, the fraction of pixels at or below a dark threshold, and a 32-bin histogram. Each result carries `ANALYSIS_VERSION` and the analysed size. A decode failure is a per-image outcome, not an error for the run. Unit tests use synthetic images (black, white, gradient, half-dark) in each format.
**Gate:** GATE-FAST.

### P3.3 — XMP (Lightroom) reader
`crates/core/src/clean/xmp.rs`: read `xmp:Rating` (attribute or element form) from `<stem>.xmp` / `<stem>.XMP`, or from the XMP APP1 packet embedded in a JPEG (bounded read). Read-only. Tests cover the attribute form, the element form, a sidecar taking precedence over the embedded packet, a malformed packet and an absent one.
**Gate:** GATE-FAST.

### P3.4 — Store v3: analysis, status, classification
Migration v3 adds `image_analysis` (one row per image, algorithm version, metrics, XMP rating, failure kind), `image_status` (one row per image: `eligible | flagged | rejected | quarantined`, a reason as JSON, `user_override`, `updated_at`), `file_operations`, and `images.current_path`. Also:
- `analyze_for_root(root, progress)`: analyses images with no current-version row, in parallel with rayon, written in chunked transactions, idempotent.
- `classify(conn, thresholds)`: a pure function of the stored metrics and the given thresholds, so the result is reproducible. It stores the thresholds used in each reason and never overrides a user decision. The thresholds themselves are persisted in `projects.settings_json`.
- A review query: images with metrics, status, reasons and current location.
- User decisions (`accept` / `reject`), recorded as `user_override`.

**Gate:** GATE-FULL (the data map gains the analysis and decision items).

### P3.5 — Quarantine primitive (the minimal Phase 6 subset)
`crates/core/src/files/`: `quarantine(ids)` and `restore(ids)`. For each file: (1) journal a `pending` operation; (2) verify the source hash against the import hash; (3) refuse if the destination exists; (4) `rename`; (5) verify the destination hash; (6) complete the journal entry and update `current_path` and the status in one transaction. A failure leaves the file where it was and records the reason. `reconcile_operations` runs on open and settles any `pending` entry from the state on disk. The Phase 1 integrity check reads `current_path`. Tests cover the round trip with byte-identical files, a hash mismatch refused, an occupied destination refused, a crash between rename and commit reconciled, restore when the original path is occupied refused, and a quarantined file not re-imported.
**Gate:** GATE-FAST.

### P3.6 — Thumbnails
`thumbnail(root, id, size ∈ {256, 1024})`: produce a JPEG cached at `<root>/.timelapse/thumbs/<id>-<size>.jpg`, regenerated when the source hash changes. Data map: thumbnails stored on the device.
**Gate:** GATE-FAST.

### P3.7 — Tauri commands + IPC wrappers
Thin commands: `analyze_images` (async, with progress over a `Channel`), `get_clean_settings`, `set_clean_settings`, `classify_images`, `get_review`, `set_decision`, `quarantine_images`, `restore_images`, `get_thumbnail` and `list_file_operations`. Errors cross IPC as `{ kind, args }`, and each per-item failure carries a `kind`. No new capability. Matching TS types and wrappers.
**Gate:** GATE-UI.

### P3.8 — Cleaning UI (both locales)
A new "Cleaning" page sharing the current project root with Home. It has:
- threshold controls;
- an analysis run with progress;
- a flagged gallery with thumbnails, metrics, reasons (dark, mean, XMP reject) and accept/reject;
- an inspect view at 1024 px;
- a preview of the consequences ("N photos will move to …") behind an explicit confirmation dialog;
- a quarantine list with restore;
- every state from GUIDELINES §4.

Every string is in both catalogs, and every new error or failure kind has a key.
**Gate:** GATE-UI.

### P3.9 — Exit-criterion tests + independent review
Integration tests on real files: import → analyse → classify → reject → quarantine → restore, with byte-identical originals. Reclassifying from SQLite alone gives the same decisions. A changed threshold changes only non-overridden rows. A crash is reconciled. An independent reviewer pass checks licences, i18n parity, error kinds, core purity, docs (DATABASE_SCHEMA v3) and correctness.
**Gate:** reviewer sign-off in STATE.md; GATE-FULL.

### P3.10 — Final gate, status, commit, push
Full gate green, then update STATE.md and CURRENT_STATUS.md. Commit (Conventional Commits, staged paths only) and push.
**Gate:** GATE-FULL + push to `origin/main`.

---

## Out of scope for Phase 3
- Contact sheets and side-by-side comparison (Phase 7, ADR-014).
- Consolidation, copy, hardlink/symlink policy (Phase 6, beyond the quarantine subset).
- Permanent deletion of anything (ADR-011: never automatic; no delete action in this phase).
- Writing XMP or any metadata back to originals (ADR-006).
- HEIC and RAW decoding.
