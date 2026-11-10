# Timelapse Studio — Project Instructions

## 1. Project overview

Timelapse Studio is a local-first desktop application for importing, analyzing, organizing, sampling and producing timelapse videos from large image collections.

The application is designed primarily for GoPro footage and long-running construction or project documentation.

## 2. Technology stack

- Desktop framework: Tauri 2
- Frontend: Svelte + TypeScript
- Backend: Rust
- Database: SQLite
- Video processing: FFmpeg
- Target: macOS first, with potential future Windows support

The Rust processing engine must remain independent from the UI wherever practical, allowing future CLI reuse.

## 3. Documentation is the source of truth

Before making architectural or implementation decisions, inspect:

- `docs/PRD.md`
- `docs/TDD.md`
- `docs/ARCHITECTURE.md`
- `docs/DATABASE_SCHEMA.md`
- `docs/PROCESSING_PIPELINE.md`
- `docs/DECISIONS.md`
- `docs/CURRENT_STATUS.md`

`docs/DEVELOPMENT_ROADMAP.md` is expected but is not yet present. Identify this gap and propose its creation.

If documents contradict each other:
1. Identify the contradiction.
2. Explain its implications.
3. Recommend a resolution.
4. Do not silently choose one interpretation.

Update relevant documentation when approved architectural or technical decisions change.

## 4. Development principles

- Local-first: no mandatory cloud services, accounts or telemetry.
- Original photographs are immutable.
- All image processing must be non-destructive.
- Never silently overwrite original files.
- Never permanently delete user photographs.
- Use quarantine rather than deletion for rejected images.
- Prefer preview and dry-run before filesystem operations.
- Operations involving many files must be recoverable after interruption.
- Maintain an audit trail of important processing operations.
- Handle large collections efficiently: 10,000 to 100,000+ images.
- Prioritize correctness, maintainability and predictable behavior.

## 5. Filesystem safety

Treat all source photographs as user data.

Never perform the following without explicit user authorization:

- Delete original photographs.
- Move or rename original photographs.
- Overwrite existing files.
- Empty a quarantine or recovery directory.
- Perform bulk filesystem operations outside the project workspace.

For potentially destructive operations:
1. Explain the operation.
2. Identify affected files and destinations.
3. Provide a preview or dry-run.
4. Obtain explicit approval.
5. Preserve recovery information whenever possible.

Never assume that a file is disposable because it appears duplicated or defective.

## 6. Architecture rules

- Keep UI concerns separate from processing logic.
- Keep business logic in Rust where appropriate.
- Use SQLite as the authoritative source for project metadata and processing history.
- Avoid duplicating database state in JSON manifests.
- Use asynchronous operations for long-running tasks.
- Provide progress reporting and cancellation where appropriate.
- Ensure interrupted operations can be detected and recovered.
- Avoid unnecessary dependencies.
- Do not introduce a new framework or major dependency without explaining why.

## 7. Engineering standards

- Inspect existing code before changing it.
- Follow established project conventions.
- Prefer small, focused changes.
- Avoid unnecessary rewrites.
- Handle errors explicitly.
- Validate external input.
- Consider performance and memory consumption.
- Add tests for important logic and edge cases.
- Do not invent APIs, libraries or capabilities.
- Run relevant tests and checks before declaring a task complete.

## 8. Git and collaboration

- Work on isolated branches or Git worktrees for independent tasks.
- Never have multiple agents modify the same working tree simultaneously.
- Do not commit another agent's changes.
- Do not force-push, reset, rebase or discard user changes without explicit authorization.
- Review diffs before merging.
- Keep commits focused and descriptive.

## 9. Agent responsibilities

### Developer
Owns implementation planning, coordination, integration and project consistency.

### Specialist
Handles explicitly scoped technical tasks and reports files changed, decisions made and tests performed.

### Reviewer
Inspects changes independently. Must not modify project files or silently fix issues.

## 10. Current project stage

The project currently contains design documentation only.

The first task is to audit and reconcile the existing documents, identify missing decisions and prepare the development roadmap.

Do not begin application implementation until the user has reviewed and approved the technical baseline.

## 11. Communication

- Be concise but technically precise.
- Explain important trade-offs.
- Identify assumptions explicitly.
- Distinguish confirmed facts from recommendations.
- Report changed files and test results.
- Ask before making significant architectural decisions.