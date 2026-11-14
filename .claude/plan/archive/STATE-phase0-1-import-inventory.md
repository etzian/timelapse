# STATE

STATUS: PENDING
<!-- PENDING | RUNNING | DONE | BLOCKED -->

## Contract

- Repo root: `.` (`~/etzian/timelapse`). UI/UX contract: `../_uidesign/GUIDELINES.md`, **read-only**.
- Product: **Timelapse Studio**, `com.etzian.timelapse`. Turn large photo collections into high-quality timelapse videos, locally.
- Stack: Rust workspace (`crates/core` pure, `src-tauri` shell), Svelte 5 + TS on `@etzianco/ui`, Fluent (fr-CA + en).
- Verification: `make gate`.
- Invariants: permissive licences only; strings in both locales; `crates/core` free of tauri; data map updated with any change to what is stored or sent; narrow capabilities.

## Phases

| # | Phase | Status |
| --- | --- | --- |
| 0 | Baseline | done (2026-11-10) |
| 1 | First feature | **done (2026-11-12)** — full `make gate` sequence green on this host: fmt, clippy `-D warnings`, cargo tests, svelte-check, vitest, vite build, `verify.sh`, i18n, legal check/gen, licence-audit (all exit 0) |

## Handoff log

- **2026-11-12 — Phase 1 DONE: full release gate green on this host.** Toolchain gap closed (rustup + Clippy present; `ez` CLI installed via npm). Full `make gate` sequence executed in order, every step exit 0:
  1. `cargo fmt --all -- --check` — clean. (Clippy's 3× `let_unit_value` findings in `crates/core/src/import/store.rs:32,34,44` fixed by dropping the redundant `let _ =` bindings — mechanical, no logic change; re-fmt applied to the now-shorter lines.)
  2. `cargo clippy --workspace --all-targets -- -D warnings` — clean.
  3. `cargo test --workspace` — 10/10 core tests pass, all suites ok.
  4. `npm --prefix frontend run check` (svelte-check, fail-on-warnings) — 0 errors / 0 warnings.
  5. `npm --prefix frontend run test` (vitest) — 2/2 pass.
  6. `npm --prefix frontend run build` (vite) — built ok.
  7. MSYS2 `bash scripts/verify.sh` — 8/8 invariant checks ok, `verify.sh: pass`.
  8. `ez i18n-check .` — ok: fr-CA 141/141 keys match en; 9 UI files, no hardcoded strings; 83 message references all exist in en.
  9. `ez legal check .` — ok, 2 documented warnings (AI/minors special trigger → lawyer review before launch; 0/3 templates lawyer-reviewed) — non-blocking by design.
  10. `ez legal gen .` — regenerated `legal/generated/**` (6 docs in en/fr-CA/internal) to track the `images-inventory` data-map entry.
  11. `ez licence-audit .` — ok: Rust (deny.toml policy) + JS (23 prod packages) clean; ADR-018 deps and the `option-ext` exception audit green.
  - **Phase 1 → done in the status table.** Not committed (working tree still staged for user review, including pre-existing unrelated modifications). Remaining deferred release items unchanged: Apple signing inputs + NÉQ/`ez profile` commercial-model switch (see Phase 0 entry and `docs/DEVELOPMENT_ROADMAP.md`). Next: Phase 2 only after user confirms Phase 1 acceptance/commit.
- **2026-11-12 — Gate sub-checks verified locally (superseded; all steps subsequently green in the entry above).** Windows toolchain is now available on this host (Rust stable with rustfmt; MSYS2 bash for `verify.sh`; Node 22 + installed `frontend/node_modules`). Executed this session, all real exits:
  - `cargo fmt --all` (applied) — exit **0**. Formatted only `crates/core/src/import/{dedup,mod,scan,store}.rs` and `src-tauri/src/lib.rs`; whitespace/commas/wrapping to the 100-col config (`rustfmt.toml`), no logic changes. Working-tree file set otherwise unchanged.
  - `cargo fmt --all -- --check` — exit **0** (clean).
  - `cargo test --workspace` — **10/10 core tests pass**, all suites ok, exit **0**.
  - `npm --prefix frontend run check` (svelte-check, fail-on-warnings) — 0 errors / 0 warnings, exit **0**.
  - `npm --prefix frontend run test` (vitest) — 2/2 pass, exit **0**.
  - `npm --prefix frontend run build` (vite) — built ok, exit **0**.
  - MSYS2 `bash scripts/verify.sh` — all 8 invariant checks `ok`, `verify.sh: pass`, exit **0**. This is the `ui-check` tail of the gate.
  - Frontend colour-token fix landed earlier this session: `Home.svelte` hard-coded colours replaced with `--ez-color-text-muted` / `--ez-color-warning` / `--ez-color-error` (verified present in both `@etzianco/ui` light/dark token sets) — this was the blocker for `verify.sh`.
- **PENDING — not executed on this host, so NOT part of the verified set (AGENTS.md §7):**
  - `cargo clippy --workspace -- -D warnings` — clippy component/binary unavailable here. PENDING in an authorized toolchain-complete environment or CI.
  - `ez i18n-check .` — `ez` CLI not on PATH. PENDING.
  - `ez legal check .` / `ez legal gen .` — `ez` CLI not on PATH. PENDING. (Note: `legal/data-map.yaml` was amended this session with the `images-inventory` entry, so a fresh `ez legal gen` must run before release to confirm generated artefacts stay in sync.)
  - `ez licence-audit .` — `ez` CLI not on PATH. PENDING. (`option-ext` 2026-11-09 exception and new deps `rusqlite 0.32 bundled` + `sha2 0.10` per ADR-018 must all audit clean.)
- **Status (at that point): Phase 1 was INCOMPLETE / AWAITING RELEASE GATE** — since superseded: Clippy and all three `ez` steps subsequently executed here green (see entry above).
- **2026-11-12 — Phase 1 implementation record (superseded status above; retained).** First real feature ("Import") implemented end-to-end.
  - `crates/core`: new `import/` module tree — `scan.rs` (walk, `SUPPORTED_EXTENSIONS = jpg|jpeg|png|tiff`, silent-skip on subdirs per PRD), `dedup.rs` (8 KiB-streaming SHA-256 via `sha2 0.10`; exact-byte `duplicate_groups`; 3 unit tests), `store.rs` (SQLite via `rusqlite 0.32 bundled` with `foreign_keys=ON`, WAL, versioned DDL for `projects` + `images`, idempotent upsert keyed on `original_path` via `ON CONFLICT DO NOTHING`, integrity snapshot with missing-files + duplicate groups; 2 unit tests), `mod.rs` (root-only public API: `import_folder`, `list_for_root`, `integrity_for_root`, `default_db_dir`, `derive_name`, `ImportReport`; 5 unit tests). `lib.rs` gains an `Error` enum with stable `kind()`/`args()` strings for the IPC contract; no `tauri` import anywhere in the crate (CLAUDE.md invariant).
  - `src-tauri`: thin command adapters — `app_info`, `import_folder`, `list_images`, `integrity_check`. Every command opens a short-lived DB handle and closes it on return (ADR-007, ADR-011), so concurrent IPC never serialises on one writer. `IpcError { kind, args }` derived from `core::Error` — no new strings introduced at the shell, no prose crossing IPC. Dialog plugin registered **unconditionally** (frontend uses it for folder-pick on every platform); `window_state` + `single_instance` remain desktop-only (`#[cfg(desktop)]`). Added `tauri_plugin_dialog = "2"` to `Cargo.toml` and `"dialog:default"` to `capabilities/default.json`.
  - `frontend`: replaced the sample greet page with an Import page — folder picker (`@tauri-apps/plugin-dialog ^2.6.0`), import call with a 180 s timeout (default is 10 s; hashing many large TIFFs is far longer), report card (scanned / hashed / new / already present / duplicates / skipped-unreadable), integrity section (missing files + duplicate groups), inline `role=alert` on error, root echo. All new strings are in **both** `locales/en/app.ftl` and `locales/fr-CA/app.ftl` (`home-*`, `error-import-root-unreadable`, `error-image-unreadable`, `error-database`, `error-store-transaction`) — 27/27 keys match, no dupes (verified by script this session).
  - `legal/data-map.yaml`: added `images-inventory` entry alongside `app-settings`; `data_leaves_device: false` unchanged. `docs/DECISIONS.md` gained **ADR-018** recording the user-approved dependency additions (`rusqlite 0.32 bundled`, `sha2 0.10`) with the permissive-licence rationale; "SQLite Rust library" open item now points to ADR-018 as Resolved.
  - **Honest local status (AGENTS.md §7):** Rust toolchain and `ez` CLI are not on this machine, and `frontend/node_modules` is absent; none of the `cargo check/clippy/test`, `svelte-check`, or gate steps could be executed here. What *was* actually verified in-session: (a) en/fr-CA key parity — 27/27 identical, no dupes; (b) every `t('…')` call in `Home.svelte` resolves to an existing `app.ftl` key in both locales; (c) every `kind()` string in `core::Error` has a matching `error-<kind>` Fluent key; (d) core → shell contract — command names (`app_info`, `import_folder`, `list_images`, `integrity_check`), the shared `root: PathBuf` parameter, and the `ImportReport` / `ImageRow` / `IntegrityResult` field sets match field-for-field between the Rust structs and the TS types in `frontend/src/lib/ipc/app.ts`. **Tauri's default is to preserve snake_case command names** (no `rename_all` in this codebase), so JS `'import_folder'`, `'app_info'`, `'list_images'`, `'integrity_check'` are the correct call strings — verified by re-reading `frontend/src/lib/ipc/app.ts` and the Rust command signatures side-by-side. What is **not** yet verified: that any of the Rust code *compiles* (e.g., `rusqlite 0.32` exact API surface, whether edition 2024 + rust-version 1.90 accept every new file, whether `tauri_plugin_dialog::init()` is usable unconditionally on the `Builder`, whether `get_webview_window` needs a `Manager` import in scope — it does, now added under `#[cfg(desktop)]`), or that the Svelte/TS build passes against the real `@etzianco/ui` type definitions (not locally resolvable without `node_modules`). **Gate therefore cannot be asserted green from this machine.**
  - **Next:** user approves running `make gate` (or CI run) and shares the output; then review any failures, fix, re-run until green; then Phase 1 → done and move to Phase 2. If the user can provide Rust toolchain locally the Rust compile check becomes available on this machine.
- **2026-11-12 — Windows verification-prep checkpoint (resume point).** User is about to install missing prerequisites, restart, and resume. **Nothing installed/changed by me this session**; no files modified beyond STATE.md. Findings verified against the live machine:
  - **Available:** Node v22.14.0 + npm 10.9.2; MSVC linker `link.exe` (VS 2022 Community, MSVC 14.42) — satisfies Rust's default `x86_64-pc-windows-msvc` target; MSYS2 tools for `verify.sh` at `C:\msys64\usr\bin` (`bash`, `sh`, `perl`, `sed`, `grep`, `find`).
  - **Missing:** `cargo`/`rustc`/`rustup` (no `~/.cargo`); GNU `make`; `ez` CLI; FFmpeg (not needed by the gate — Phase 2+ only).
  - **Auth gap:** `gh` is installed but not logged in, and `gh auth token -u etzian` fails; `NPM_TOKEN`/`NODE_AUTH_TOKEN` unset. So `npm ci` for the private `@etzianco/ui` package needs either a `gh login` to the etzian org or an explicit PAT in `$env:NPM_TOKEN` before install.
  - **Gate layout:** `make gate` = `check` (cargo fmt/clippy/test) + `ui-check` (svelte-check, vitest, vite build, `scripts/verify.sh`) + `i18n` (`ez i18n-check .`) + `legal` (`ez legal check/gen`) + `licence` (`ez licence-audit .`). On Windows: use MSYS2 bash to run `./scripts/verify.sh`; native target is msvc; run commands directly instead of `make` (or install make).
  - **Platform limits:** `ez` CLI i18n/legal/licence steps can't run locally unless `_foundation`/`ez` is provisioned → fallback is the `.github/workflows/ci.yml` macos-latest runner (its header explicitly says these three are canonical in CI; do NOT weaken it). `make build` of the `.app` is mac-only and not part of the gate.
  - **Ordered procedure handed to user:** (1) install Rust via `rustup-init.exe -y --default-toolchain stable --profile minimal`, add `~/.cargo\bin` to PATH, `rustup component add rustfmt clippy`; (2) `gh login`/set `NPM_TOKEN` for etzian org; (3) `npm ci --no-audit --no-fund` in `frontend/` then `npm run check/test/build`, then MSYS2 bash to run `./scripts/verify.sh`; (4) `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings` + `cargo test --workspace`; (5) run the three `ez` steps locally if possible, else in CI.
  - **Status:** user is installing these tools and will reboot. After they come back, re-run the procedure above and capture output before declaring anything.
- **2026-11-10 — Phase 0 done.** Gate `make gate` green locally (rust check/clippy/test, svelte-check, vitest, i18n fr-CA/en parity, legal generated from `legal/data-map.yaml`, licence audit with the 2026-11-09 `option-ext` exception). CI (`ci.yml`) also passes on main — run 42755343971 green — building the unsigned `.app` (aarch64-apple-darwin) and uploading it as `timelapse-app-aarch64`; `@etzianco/ui` is pulled from GitHub Packages into CI via the repo's `NPM_TOKEN` secret. Remaining deferred item before release: Apple `team_id` + signing (see `docs/DEVELOPMENT_ROADMAP.md` "Cross-phase release gate" §Apple signing; `docs/DECISIONS.md` ADR-017), and the temporary `free` commercial model in `legal/data-map.yaml` must be switched back to one-time purchase once the NÉQ is entered via `ez profile`.
