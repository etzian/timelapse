# PHASES

Phases 0–4 are **complete** and archived in [archive/](./archive/). This file tracks **Phase 5 — FFmpeg video generation**. The canonical scope and exit criteria stay in [docs/DEVELOPMENT_ROADMAP.md](../../docs/DEVELOPMENT_ROADMAP.md); the slices implement them and nothing more.

> ## Phase 5 — FFmpeg video generation
> **Scope:** Sequence → MP4 (H.264/HEVC hardware path; AV1 tier per selected option); duration calculation; FFmpeg safety (explicit args, no shell interpolation). (Pipeline §10.)
> **Entry:** ADR-017 adopted; pre-Phase-5 prerequisites (a)–(c) resolved before Phase 5 begins. Phase 4 exit.
> **Exit:** A valid MP4 is produced from a sampled sequence; encode uses the committed encoder path (hardware H.264/HEVC; SVT-AV1 if AV1 tier enabled); no GPL/denied-licence encoder linked in the shipped binary.

**User decisions, 2026-11-14:**
1. **ADR-021 approved:** the app may start FFmpeg/ffprobe as child processes, under these limits:
   - only the bundled binaries, by absolute path;
   - an explicit argv (no shell, no interpolation), started from `crates/core`;
   - no Tauri shell plugin and no capability change;
   - output only into `<root>/.timelapse/exports/`;
   - each spawn site marked `verify:allow` citing ADR-021.
2. **Licences:**
   - Draft (undated) `licence-exceptions.yaml` entries for FFmpeg (LGPL-2.1-or-later) and SVT-AV1 (BSD-3-Clause-Clear + AOM Patent License 1.0). **Only the user dates them**, and until then the licence audit may stay red.
   - Correct ADR-017's "SVT-AV1 = BSD-3-Clause" statement.
3. **Build and test on the macOS CI runner.** `tools/build-ffmpeg/` builds FFmpeg 9.0.2 and SVT-AV1 v4.2.0 from pinned, checksummed tarballs, then:
   - runs the licence/GPL scan;
   - encodes real MP4s;
   - uploads the binaries and their exact sources.

   On this Windows host, encode tests are skipped when no FFmpeg is configured; argv building, duration maths, frame fitting and ffprobe parsing are tested everywhere.
4. **Pin:** FFmpeg **9.0.2** "Lei" (`sha256 8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e`); SVT-AV1 **v4.2.0** (`sha256 c7b13c4a84bd3751aa35fcc72be13e6875467e7c2216879251a486e5b1e4e740`, GitLab archive). v4.3.0 (2026-11-02) is too recent to pin.

---

## Slices

Gate names are as before: `GATE-FAST`, `GATE-UI`, `GATE-FULL` (see Phase 4).

### P5.0 — Prerequisites (a)–(c), ADR-021/022, licence drafts
- **ADR-021:** the subprocess rule.
- **ADR-022** resolves ADR-017's open items:
  - **(a) provenance:** pinned tarballs + checksums; LGPL build, shared libraries (per ffmpeg.org/legal.html, dynamic linking), `--disable-autodetect`, only the components we use;
  - **(b) no software fallback:** a runtime encoder probe; `-allow_sw 0` for VideoToolbox; refuse a tier whose encoder is missing;
  - **(c) notices:** `LICENSES/` placement (COPYING.LGPLv2.1, configure line, source URLs + checksums, SVT-AV1 LICENSE + PATENTS) and an About-box mention.
- Draft exception entries; correct ADR-017.

**Gate:** GATE-FAST (docs only, plus `ez licence-audit .` run and its result recorded).

### P5.1 — Build script + CI job
- `tools/build-ffmpeg/build.sh`: download, verify sha256, build SVT-AV1 (shared, no apps) and FFmpeg (LGPL, shared, minimal), set `@rpath`, collect licence files.
- `tools/build-ffmpeg/check-licence.sh`, failing when:
  - `-L` does not report the LGPL;
  - `-buildconf` contains `--enable-gpl`/`--enable-nonfree`;
  - any binary references or contains x264/x265;
  - a required encoder is missing, or `libx264`/`libx265` is present.
- CI job (macos-latest) that caches by pin hash, exports `TIMELAPSE_FFMPEG_DIR` to the test run, and uploads the binaries and sources.

**Gate:** the CI run is green (watched with `gh run`).

### P5.2 — Core export engine
`crates/core/src/export/`:
- **Locate and probe:** version, encoders, LGPL.
- **Settings:** codec `av1` (default) | `h264` | `hevc`; fps; playback multiplier (separate from fps, PRD §5.7); resolution preset; quality.
- **Frame preparation:** decode with EXIF orientation applied, fit with letterbox, piped as raw RGB on stdin, so file paths never reach FFmpeg's argv for inputs.
- **argv builder (pure):** `-n` (never overwrite), output to `.part`, then a no-clobber rename.
- **Process control:** progress, cancellation, bounded stderr capture.
- **Validation:** ffprobe checks codec, dimensions and packet count.
- **Store v5:** `video_exports`, recording settings, ffmpeg version, argv, encoder, output, status and probe results.
- **Duration maths** matching the Sampling estimate.

**Gate:** GATE-FAST, plus the CI encode tests for each available tier.

### P5.3 — Tauri commands + resources
- `ffmpeg_info`, `start_export` (async, progress Channel), `cancel_export`, `list_exports`.
- FFmpeg is resolved from the bundle's resource dir. A `TIMELAPSE_FFMPEG_DIR` override exists for dev and tests only.
- `bundle.resources` carries `resources/ffmpeg/**` (a placeholder README keeps non-mac builds working).

No capability change. **Gate:** GATE-UI.

### P5.4 — Export page (both locales) + About notice
Pick a saved selection, codec tier (unavailable tiers disabled with the reason), FPS, speed, resolution, quality; estimated duration; export with progress and cancel; result (path, codec, duration, size); history; every GUIDELINES §4 state. Settings → About mentions FFmpeg/SVT-AV1 and their licences.
**Gate:** GATE-UI.

### P5.5 — Exit tests + independent review
CI encode tests: a sampled sequence → MP4 per available tier, validated with ffprobe (codec, size, frame count ≈ the duration maths); no GPL symbols (P5.1 scan); an unavailable tier is refused, never substituted. Then an independent reviewer pass.
**Gate:** reviewer sign-off; GATE-FULL (with the licence audit pending the user's approval dates, recorded as such); CI green.

### P5.6 — Final gate, status, commit, push

---

## Out of scope
- Code signing and notarisation of the bundled FFmpeg (it ships with the app's signing, later).
- Audio, overlays, frame interpolation, deflicker (Phase 8+), contact sheets (Phase 7).
- Publishing the exact source tarballs on a download page: CI uploads them as an artifact, and the release process must publish them (LGPL §6). Flagged for the lawyer review.
