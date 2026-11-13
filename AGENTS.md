# Timelapse Studio — Agent Instructions

## 1. Instruction hierarchy

Follow all applicable instructions in:

1. `CLAUDE.md` — project architecture, engineering rules and invariants.
2. `.claude/plan/PHASES.md` — canonical development phase specification.
3. `.claude/plan/STATE.md` — canonical development status and session handoff.
4. This file — additional instructions for OpenCode agents.
5. `docs/` — supporting technical and product documentation.

If instructions conflict:
- Identify the conflict.
- Explain its implications.
- Follow the most authoritative applicable instruction.
- Never silently change an approved architectural decision.

The canonical phase specification and status are maintained separately:
- Never modify `PHASES.md` unless explicitly authorized by the user.
- Update `STATE.md` when development progress changes.
- Keep supporting documentation consistent with approved decisions.

## 2. Current development state

The project is Timelapse Studio, a local-first desktop application.

Phase 0 — Baseline is complete.
Phase 1 — First feature is pending.

Do not repeat the initial documentation audit.
Do not assume that the old roadmap in `docs/DEVELOPMENT_ROADMAP.md`
defines the current phase sequence.

Consult `PHASES.md` and `STATE.md` at the beginning of each new task.

## 3. Development workflow

The primary developer agent owns:
- Task planning and coordination.
- Architecture consistency.
- Specialist delegation.
- Integration and validation.
- Documentation and status continuity.

Specialists receive narrowly scoped assignments.
Avoid overlapping file ownership and concurrent edits to the same files.

The reviewer must independently inspect changes and must not modify project files.

For each significant task:
1. Inspect the relevant source and documentation.
2. Identify dependencies and affected files.
3. Define a bounded implementation scope.
4. Implement and test.
5. Obtain an independent review when appropriate.
6. Integrate and run the relevant verification gate.
7. Report changed files, decisions and test results.
8. Update `STATE.md`.

## 4. Phase management

Work must follow `.claude/plan/PHASES.md`.

A phase must satisfy its stated goal and gate before being declared complete.

Do not invent phase completion criteria or mark a phase complete without evidence.

When the next phase lacks a sufficiently defined feature scope:
- Analyze the available product and technical documentation.
- Recommend a concrete first feature and explain the rationale.
- Identify dependencies, risks and affected components.
- Present the proposed scope for user approval before implementation.

Do not edit the frozen phase specification to accommodate implementation convenience.

## 5. Architecture and product invariants

Follow `CLAUDE.md` and the applicable ADRs.

In particular:
- Keep `crates/core` independent of Tauri.
- Keep UI and processing logic separate.
- Preserve original photographs.
- Never silently delete, move, rename or overwrite source photographs.
- Use recoverable operations and dry-runs where appropriate.
- Keep SQLite as the authoritative source for project metadata and processing history.
- Keep user-visible strings in both locale catalogs.
- Keep Tauri capabilities as narrow as possible.
- Respect the legal data map and licence policy.
- Do not introduce dependencies or licence exceptions without the required user approval.

## 6. Git and workspace safety

- Work only in the intended repository.
- Inspect `git status` before making changes.
- Preserve unrelated user changes.
- Do not reset, rebase, force-push or discard user changes without explicit authorization.
- Do not commit or push unless explicitly authorized.
- Do not modify shared design-system source files.
- Do not modify another agent's work.

## 7. Validation and reporting

Use the verification commands documented in `CLAUDE.md`.

Do not claim that a check passed unless it was actually executed successfully.

Distinguish clearly between:
- Implemented.
- Tested.
- Reviewed.
- Integrated.
- Committed.

Report incomplete checks and known limitations honestly.
