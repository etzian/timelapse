# STATE

STATUS: DONE
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
| 1 | First feature | pending |

## Handoff log

- **2026-11-10 — Phase 0 done.** Gate `make gate` green locally (rust check/clippy/test, svelte-check, vitest, i18n fr-CA/en parity, legal generated from `legal/data-map.yaml`, licence audit with the 2026-11-09 `option-ext` exception). CI (`ci.yml`) also passes on main — run 42755343971 green — building the unsigned `.app` (aarch64-apple-darwin) and uploading it as `timelapse-app-aarch64`; `@etzianco/ui` is pulled from GitHub Packages into CI via the repo's `NPM_TOKEN` secret. Remaining deferred item before release: Apple `team_id` + signing (see ROADMAP §8 / CLAUDE.md ADR-017), and the temporary `free` commercial model in `legal/data-map.yaml` must be switched back to one-time purchase once the NÉQ is entered via `ez profile`.
