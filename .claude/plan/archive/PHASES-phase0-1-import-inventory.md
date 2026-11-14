# Plan — Timelapse Studio v0.1

Spec file. Never edited by the loop. Every phase has a gate that exits 0 or
non-zero. Replace the placeholder phases below (with `etzian:plan-milestone`)
once the product is scoped; keep the shape.

## Phase 0 — Baseline

Goal: the generated app builds and its gate passes, before any feature.
Gate: `make gate` (the licence step waits on the user's approval, see CLAUDE.md).

## Phase 1 — First feature

Goal: replace the sample "greet" feature with the first real one: core logic in
`crates/core` with tests, a command in `src-tauri`, a page in `frontend`, strings in
both locales, and `legal/data-map.yaml` updated if it stores or sends anything.
Gate: `make gate`.
