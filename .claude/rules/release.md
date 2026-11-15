# Release

Version lives in `Cargo.toml` (`[workspace.package]`), `src-tauri/tauri.conf.json` and
`frontend/package.json`; `etzian:release` bumps all three and generates the changelog
with git-cliff. Before a release: `make gate` green, fr-CA complete, generated legal
documents regenerated, templates reviewed by a lawyer (`ez legal check .` says when).

The bundled FFmpeg (ADR-022) adds two steps:

- `tools/build-ffmpeg/check-licence.sh` passes on the build that ships, and the
  release attaches the exact source tarballs (`sources/`) and `LICENSES/` from that
  build's CI artifact (LGPL-2.1 §6). The artifact expires after 90 days, so attach
  them when the release is cut.
- The hardware path is verified once on a real Apple Silicon Mac:
  `TIMELAPSE_FFMPEG_DIR=<bundle>/resources/ffmpeg TIMELAPSE_REQUIRE_VIDEOTOOLBOX=1 cargo test -p timelapse-core export:: -- --nocapture`.
