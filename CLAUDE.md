# Timelapse Studio

Turn large photo collections into high-quality timelapse videos, locally.

Bundle `Timelapse Studio.app` · identifier `com.etzian.timelapse` · repo `~/etzian/timelapse`.
Generated from the `etzian` standalone template; `copier update` brings in later
template changes. Facts about the company come from `ez profile`, never typed here.

## Stack

- **Rust** edition 2024 workspace: `crates/core` (pure library, no `tauri`) + `src-tauri` (thin shell)
- **Tauri 2**, capabilities as narrow as possible
- **Svelte 5 + TypeScript + Vite** in `frontend/`, on **`@etzianco/ui`** (GitHub Packages; source `../_uidesign`, **read-only**)
- **Fluent** i18n: `locales/{fr-CA,en}/*.ftl`, fr-CA is a release blocker
- Data stays on the device unless `legal/data-map.yaml` says otherwise

## Run / verify

```sh
make gate      # check + ui-check + i18n + legal + licence
make dev       # cargo tauri dev
```

Redirect gate output to a file and read the tail; the full stream is huge.
`make licence` fails until the user approves the `option-ext` exception in
`licence-exceptions.yaml` (Tauri 2 needs it; only the user approves it).

## Invariants

- **Licences:** the `allow` list in `~/etzian/_foundation/tech/licences/policy.yaml`; anything `restricted` needs an entry in `licence-exceptions.yaml`. No new dependency without naming its licence and asking.
- **`crates/core` has no `tauri` dependency.** Logic in a command is a smell.
- **Every user-visible string is in both `.ftl` locales.** Errors cross IPC as `{ kind, args }`. Numbers and dates go through `Intl`.
- **The data map is the truth.** A change that stores or sends more updates `legal/data-map.yaml` in the same commit; `ez legal gen .` regenerates the documents.
- **Tokens only** in app CSS (`--app-*`, `.app-*`); `--ez-*` belongs to the design system.
- **Capabilities stay narrow.** No `shell`, `http` or `fs` without the user and an ADR.
- **Code language: English.** UI copy is authored in English and translated to Canadian French.

## Key paths

| Path | Role |
| --- | --- |
| `crates/core/` | Pure logic and its error type (`kind` + `args`) |
| `src-tauri/src/lib.rs` | The `invoke` surface |
| `frontend/src/lib/i18n/` | Fluent loader, `t()`, `Intl` helpers |
| `frontend/src/lib/legal-ux/` | Managed copy of `@etzianco/legal-ux` (do not edit; `ez vendor-ui .`) |
| `locales/` | `.ftl`: `app.ftl` is ours, the rest is the shared set from `_foundation/i18n/common` |
| `legal/data-map.yaml` | What the app stores and sends; `legal/generated/` is its output |
| `.claude/plan/` | Phase plan (`PHASES.md` frozen spec, `STATE.md` state) |
| `docs/adr/` | Decisions that deviate from the defaults |

## Phased work

`etzian:next-phase` runs the next phase of `.claude/plan/PHASES.md`;
`etzian:plan-milestone` cuts a milestone into phases. Never edit `PHASES.md`
mid-flight.
