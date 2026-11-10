# Release

Version lives in `Cargo.toml` (`[workspace.package]`), `src-tauri/tauri.conf.json` and
`frontend/package.json`; `etzian:release` bumps all three and generates the changelog
with git-cliff. Before a release: `make gate` green, fr-CA complete, generated legal
documents regenerated, templates reviewed by a lawyer (`ez legal check .` says when).
