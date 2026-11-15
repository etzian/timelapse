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
| 1 | Import & inventory | done (2026-11-12), `421f2d0` |
| 2 | EXIF & timeline | done (2026-11-14), `7528260` + `2aeab8d` |
| 3 | Brightness analysis & cleaning | done (2026-11-14), `e5318e7` |
| 4 | Temporal sampling | done (2026-11-14), `60b554f` |
| 5 | FFmpeg video generation | **done (2026-11-15)**, exit criterion proven on the macOS CI runner (run 43664224239). Review BLOCK → fixed → SIGN-OFF WITH NOTES. **Open for the user:** approval dates on the ffmpeg + svt-av1 licence exceptions |

## Slice status (Phase 5)

| Slice | Status | Gate |
| --- | --- | --- |
| P5.0 Prerequisites, ADR-021/022, licence drafts | done | GATE-FAST + licence-audit (drafts noted, not applied) |
| P5.1 Build script + CI job | done | CI green: build + check-licence.sh pass (14 checks) |
| P5.2 Core export engine | done | GATE-FAST; CI encode tests green |
| P5.3 Commands + resources | done | GATE-UI; CI .app build with the bundle |
| P5.4 Export page + About notice | done | GATE-UI |
| P5.5 Exit tests + review | done | reviewer sign-off (2nd pass); GATE-FULL; CI green |
| P5.6 Final gate + commit/push | done | GATE-FULL; merged to main |

## Handoff log

- **2026-11-15 — Phase 5 DONE.**
  - **Prerequisites (a)–(c):** resolved by ADR-022, after verifying the facts:
    - SVT-AV1 is BSD-3-Clause-Clear + AOMPL-1.0, so ADR-017 was corrected and the exceptions were drafted;
    - ffmpeg.org/legal.html requires dynamic linking, the exact sources and the configure line;
    - FFmpeg 9.0.2 supports the SVT-AV1 4.x API.

    ADR-021 records the approved subprocess rule.
  - **Build:**
    - `tools/build-ffmpeg/{build.sh,pins.env,check-licence.sh}`: pinned, checksummed tarballs; an LGPL shared build with `--disable-autodetect` and only the components in use; `@rpath`; one real dylib per install name; licences and exact sources collected.
    - The licence assertion checks: `-L` reports LGPL, no gpl/nonfree/version3 flags, the committed encoders present, no x264/x265 encoders, links or symbols, no symlinks, only system and bundled libraries, all notices present.
  - **CI** (macos-latest, timeout 75 min): FFmpeg is built once and cached by the pin hash, then the licence assertion runs. The binaries are copied into `src-tauri/resources/ffmpeg` and the tests run with `TIMELAPSE_FFMPEG_DIR`. CI uploads `ffmpeg-9.0.2-lgpl-aarch64` (binaries, LICENSES, sources; 90 days) and the .app, and now also runs on `phase-*` branches.
  - **Core** (`crates/core/src/export/`):
    - `ffmpeg.rs`: probe, a pure argv builder, an Encoder that kills and reaps on drop, ffprobe validation;
    - `frames.rs`: decode with EXIF orientation, letterbox, raw RGB piped on stdin;
    - `settings.rs`: codecs, AV1 the default, speed separate from fps, the frame plan;
    - `store.rs`: schema v5 `video_exports`;
    - `mod.rs`: one export at a time; a per-export cancel token; a cleanup guard; no-clobber placement (hard link or `create_new` copy); a pre-existing `.part` is never touched.
    - A new `export` error kind with reasons.
  - **Shell:** `ffmpeg_info`, `start_export` (Channel), `cancel_export(token)`, `list_exports`. FFmpeg is resolved from the bundle resources; the env override is debug-only; there is no relative fallback. Resources come through `bundle.resources`, with no externalBin and no capability change.
  - **UI:** the Export page (tiers disabled when unavailable, estimate, progress, cancel, result with technical details, history), the export store survives page switches, and Settings shows the About notice. 386+ keys in both catalogs. Data map: `video-exports`.
  - **Exit criterion, proven on CI run 43664224239:**
    - AV1 1× and 2×, `h264_videotoolbox` and `hevc_videotoolbox` (`-allow_sw 0`) each produced an MP4 that passed ffprobe on codec, size, packets = frame plan and duration;
    - `check-licence.sh: pass`;
    - no test skipped.
  - **Review:**
    - **First pass: BLOCK.** Findings:
      - a test race on the export lock;
      - a false positive on FFmpeg's own `"x264 - core"` SEI string;
      - a VideoToolbox skip that was too broad;
      - panic, cancel, stale-row and UI-state issues;
      - LGPL source retention.

      All fixed.
    - **Second pass: SIGN-OFF WITH NOTES.** Notes 1 (a pre-existing `.part` file) and 4 (a stale result after switching project) are fixed. Accepted as notes:
      - a failed `store::complete` after a successful encode leaves the row `interrupted` while the MP4 exists;
      - `nm` cannot see a statically linked, stripped x264, which the configure, `-L`, encoder and link checks cover;
      - `verify.sh` does not count spawn sites, because the file is template-owned (raised with the user).
  - **Also fixed:** a pre-existing Phase 1 test flake (a temp dir reused when a PID repeats), found while gating.
  - **For the user:**
    1. Date-approve the `ffmpeg` and `svt-av1` entries in `licence-exceptions.yaml`.
    2. The release must attach the exact sources (roadmap release gate). Consider adding that step to `.claude/rules/release.md`; I did not edit the rules file.
    3. The roadmap gate asks for one `TIMELAPSE_REQUIRE_VIDEOTOOLBOX=1` run on a real Apple Silicon Mac, although CI's VM already encoded VideoToolbox.
    4. The `verify.sh` spawn-site count is a template change.
- **2026-11-14 — Phase 5 kicked off.**
  - **Archive:** Phase 4 plan/state archived.
  - **Verified:**
    - SVT-AV1 is BSD-3-Clause-Clear + AOMPL-1.0, not on the allow-list, so ADR-017 is wrong on that point;
    - ffmpeg.org/legal.html requires dynamic linking, exact sources and the configure line;
    - FFmpeg 9.0.2's libsvtav1 wrapper supports the SVT-AV1 4.x API.
  - **Pinned:** the tarballs were downloaded only to hash them.
  - **No FFmpeg on this host;** the app target is macOS aarch64 only.
- **Earlier phases:** see [archive/](./archive/).
