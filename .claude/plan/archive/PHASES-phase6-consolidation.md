# PHASES

Phases 0–5 are **complete** and archived in [archive/](./archive/). This file tracks **Phase 6 — Consolidation & file management**. The canonical scope and exit criteria stay in [docs/DEVELOPMENT_ROADMAP.md](../../docs/DEVELOPMENT_ROADMAP.md), quoted below; the slices implement that scope and nothing more.

> ## Phase 6 — Consolidation & file management  *(cross-cutting; pull forward)*
> **Scope:** Copy/move originals into the project store, hardlink/symlink policy, recovery of moves, dedup, and the "originals immutable" invariant end-to-end. (Pipeline §4, §12.)
> **Entry:** Phase 1 exit (need the inventory to manage); should **land before Phase 3** so cleaning/quarantine can rely on recoverable file ops.
> **Exit:** A move/copy is reversible; originals remain byte-identical and their path of record in SQLite never points at a mutated file.

The minimal primitive (journaled no-clobber quarantine move, reconciliation) already landed in Phase 3 (ADR-020). Phase 6 builds consolidation on it. PRD §5.5 and Pipeline §5 (Stage 4 — Consolidation) and §12 (Recovery) are the functional spec.

**User decisions, 2026-11-14:**
1. **Destination:** a folder the user picks with the existing folder dialog. It must lie **outside** the imported collection (the import rescans the collection, so copies inside it would be re-imported). Nothing there is ever overwritten; collisions appear in the dry run. No capability change.
2. **Operations:** **copy** (default) and **move** (a second confirmation). A move verifies the file at its new place before the old name is removed, and `images.current_path` records where it is now. `original_path` stays the path of record and is never rewritten. Each run can be **undone**: copies we made are removed only while still byte-identical to what we wrote; moved originals go back to where they came from, never overwriting.
3. **Link policy:** an **APFS clone** (copy-on-write, an independent file) where the volume supports it, otherwise a byte copy. Both refuse to overwrite. **Never a hard link or a symlink into the working collection**: either would let an edit of the "copy" change the original (ADR-006). New direct dependency: **`libc` 0.2** (MIT OR Apache-2.0, already in the lockfile through Tauri), only for `clonefile` and `statvfs`.
4. **Input:** by default every photograph except rejected and quarantined ones (as in Phase 4), or a saved sampling selection in its sequence order. Exact duplicates (same SHA-256) go in once; the dry run lists the skipped copies.

---

## Slices

Gate names are as before: `GATE-FAST` = fmt + clippy `-D warnings` + `cargo test`; `GATE-UI` = GATE-FAST + svelte-check / vitest / build + `verify.sh` + `ez i18n-check`; `GATE-FULL` = GATE-UI + `ez legal check/gen` + `ez licence-audit`.

### P6.0 — ADR-023, plan
ADR-023 records decisions 1–4, the naming rules, the journal states, the undo rules and the recovery table.
**Gate:** GATE-FAST (docs only).

### P6.1 — Plan (pure)
`crates/core/src/consolidate/plan.rs`:
- **Naming** (PRD §5.5):
  - `preserve`: the original folder structure;
  - `original`: flat, original filenames;
  - `folder-prefix`: flat, `<source folder>_<filename>`;
  - `sequence`: flat, `000001.jpg`… in chronological (or selection) order, zero-padded to at least 6 digits.
- **Collisions:** within the plan, compared case-insensitively (APFS default); resolved with `_2`, `_3`… Every target is a safe relative path (normal components only).
- **Duplicates:** the first photo of each SHA-256 group in order is kept; the others are skipped, each with the id it duplicates. Photos with no hash are never treated as duplicates.
- **Plan fingerprint:** SHA-256 of the ordered `(image id, operation, target)` lines.

Unit tests cover each naming rule, collisions, duplicates, unsafe names, determinism and an empty input.
**Gate:** GATE-FAST.

### P6.2 — Placement + journal + store v6
- **`files::clone_or_copy_no_clobber`:** `clonefile` on macOS (it fails if the target exists), otherwise a `create_new` copy that keeps the mtime. Never a hard link.
- **`free_space`:** `statvfs` on Unix; `None` elsewhere.
- **Migration v6:**
  - `consolidation_runs`: operation, destination, source, naming, configuration, plan fingerprint, counts, bytes, status, undo status, times;
  - `file_operations` is rebuilt to allow `copy`, `move`, `remove-copy` and `move-back` (the existing rows are kept). `job_id` names the consolidation run.
- **Quarantine/restore generalised to moved photos:** quarantine starts from the current location, and restore returns to where the quarantine started (`current_path = NULL` only when that is `original_path`).

**Gate:** GATE-FAST.

### P6.3 — Consolidation engine
`crates/core/src/consolidate/mod.rs`:
- **`preview`** (dry run, no write):
  - destination checks: absolute, a directory or creatable, not inside the collection;
  - per-item conflicts at the destination: an identical file already there is `already-present` and skipped (never ours, never undone); a different file gets the next free suffix;
  - bytes needed vs free space;
  - counts, the first 200 items, and the fingerprint.
- **`execute`:**
  - refuses if the re-computed plan's fingerprint differs from the confirmed one (`plan-changed`), if space is short, or if another run is going (`busy`);
  - runs under the file-operation lock, journaled per item before any file is touched;
  - copy: verify the source hash, place, verify the target hash;
  - move: the same, then remove the source name and set `current_path`;
  - progress and cancellation between items; run status `completed` · `partial` · `cancelled` · `failed`.
- **`undo`:**
  - copy: remove each copy still byte-identical to what we wrote (`modified` and `missing` are left and reported);
  - move: move back only if the photo is still where the run put it (`moved-since`), never overwriting;
  - then remove the directories we left empty under the destination.
- **Reconciliation** extended to the new operation types.
- **Checks:**
  - `verify_originals`: re-hash every photograph at its location of record against its import hash (`ok` · `changed` · `missing` · `unhashed`);
  - `verify_run`: re-hash every copy against what was written.

**Gate:** GATE-FAST.

### P6.4 — Tauri commands
`preview_consolidation`, `start_consolidation` (token + progress Channel), `cancel_consolidation`, `undo_consolidation` (Channel), `list_consolidations`, `verify_consolidation`, `verify_originals`. TS types and wrappers. No capability change.
**Gate:** GATE-UI.

### P6.5 — Organise page (both locales)
Destination picker; operation (move behind a second confirmation); input (all or a saved selection); naming; dry run (counts, duplicates skipped, already present, renamed, bytes vs free space, the first items' targets); run with progress and cancel; result; run history with undo and verify; the originals check. Every GUIDELINES §4 state. The data map gains `consolidations`.
**Gate:** GATE-UI + `ez legal check/gen`.

### P6.6 — Exit tests + independent review
Integration tests on real files: import → consolidate → modify → undo. They check:
- a copy, then its undo, leaves the destination as it was and the originals byte-identical;
- a move puts `current_path` on a file with the import hash, and its undo restores `current_path = NULL` with the bytes unchanged;
- editing a copy never changes an original (clone/copy independence, checked by inode on Unix);
- no overwrite at the destination;
- a modified copy survives undo;
- every interruption state is reconciled;
- quarantine and restore of a moved photo;
- `verify_originals` reports a tampered original.

Then an independent reviewer pass.
**Gate:** reviewer sign-off; GATE-FULL; CI green on the macOS runner (the clone path).

### P6.7 — Final gate, status, commit, push

---

## Out of scope
- Consolidating outside one project (multi-project merges), HEIC/RAW.
- Making the working collection the project's source (later stages keep reading the originals at their location of record).
- Any capability change; deleting originals; hard or symbolic links into a working collection.
- Free-space checks on Windows (not a target; reported as unknown).
