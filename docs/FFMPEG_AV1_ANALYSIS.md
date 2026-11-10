# Timelapse Studio

## FFmpeg & AV1 — Comparative Analysis (No Decision)

**Status:** Analysis only. **This document makes no technical decision.** It exists to inform D2 (the open FFmpeg/AV1 selection, synthesis roll-up `08_SYNTHESIS.md`). A decision becomes a new ADR (candidate ADR-017) only after this is reviewed and one option chosen.
**Policy basis:** `_foundation/tech/licences/policy.yaml` (`ez licence-audit`): allow MIT/Apache-2.0/BSD/ISC/Zlib/CC0/Unlicense; **restricted** LGPL (only if dynamically linked/imported, unmodified, text shipped — requires `licence-exceptions.yaml`); **deny** GPL/AGPL even with an exception. A project-specific exception goes in `licence-exceptions.yaml` (who/when/why/conditions), never here.

---

## 1. What the product actually needs from FFmpeg

From `TDD.md`: a controlled process interface to FFmpeg (`:18`, `:35`, `:109`, `:161`), explicit argument lists (`:346`), and an explicit acknowledgement that **"Do not assume that every FFmpeg build has identical codec or hardware-encoding support" `:318`**, with the **FFmpeg distribution strategy** open (`:444`). The concrete encoding workload (from `PROCESSING_PIPELINE.md`, Stages 8–9) is:

- H.264/HEVC frame sequences → MP4 (image-sequence input, no source video decode required).
- Optional **AV1** encoding of the final timeline sequence.
- Audio is out of scope (timeline of stills); FFmpeg here is primarily an *encoder* and muxer, not a player.

This matters for licensing: we need an AV1 **encoder**, plus H.264/HEVC (the latter two have patent encumbrances, distinct from the licence question below). We do **not** need an ffmpeg GUI, decode-only stack, or any proprietary codec.

## 2. Components and their licences

| Component | Role in our build | Licence(s) |
|---|---|---|
| **FFmpeg** core | Muxer / encoder front-end; invoked as a controlled subprocess | Dual-licensed: **GPLv3+** *or* **LGPLv2.1+** (the LGPL build must have GPL-only libraries disabled) |
| **libaom** (AV1 encode/decode) | AV1 **encoder** | **BSD-2-Clause** → in allow list, unconditional |
| **SVT-AV1** (AV1 encoder) | AV1 **encoder** (fast, multithreaded) | **BSD-3-Clause** → allow list, unconditional |
| **libdav1d** (AV1 decode only) | AV1 *decoder* (we don't encode with it) | **LGPL-2.1** → **restricted**: allowed only if dynamically linked, unmodified, library text shipped |
| **x264 / x265** (H.264/HEVC encoders) | Commonly bundled | **GPL** (x264) / **BSL/GPL** (x265) → **deny** list if linked into a static GPL build |
| **Hardware** (macOS VideoToolbox / libx264 path) | H.264/HEVC encode via OS | System API; x264 fallback is GPL — avoided if we use hardware/HW paths |

Key point: an **LGPL FFmpeg build** that omits x264/x265 and uses the system's hardware encoder for H.264/HEVC keeps the *denied* GPL encoders out of the shipped binary. Adding a **BSD** AV1 encoder (libaom or SVT-AV1) into an LGPL FFmpeg is unproblematic — linking BSD code into an LGPL product does not pull that product up to BSD's terms in any incompatible way, and keeps every *required* codec outside the restricted class.

## 3. Distribution options under macOS (Phase 0/5 target)

| Option | AV1 encoder | H.264/HEVC | FFmpeg licence posture | Compliance with `policy.yaml` | Notes / risk |
|---|---|---|---|---|---|
| **A. Homebrew `ffmpeg` (default formula)** | typically on, via bundled libaom/SVT-AV1 | x264/x265 present → **built with `--enable-gpl`** | **GPLv3+** runtime distributed | **Non-compliant**: GPL in a shipped product is the deny class (`policy.yaml:45`) | Most convenient, but we are shipping the binary to end users → GPL contamination. Not acceptable without an explicit, dated `licence-exceptions.yaml`, which the policy says is disallowed for the deny class. |
| **B. LGPL FFmpeg + BSD AV1 encoder (libaom or SVT-AV1) + hardware H.264/HEVC** | libaom / SVT-AV1 (BSD) | VideoToolbox hardware (no GPL x264) | **LGPLv2.1+** with GPL-only libs off | **Compliant**: restricted LGPL satisfied *if* dynamically linked, unmodified, and FFmpeg's LGPL text + modification list shipped; BSD codecs are allowed. x264/x265 absent from binary (verify). | Preferred path. Must verify at build time that no GPL encoder is statically linked (`strings`/`lipo`/symbol check); must ship LGPL text and a notice in the app. libaom is slow on CPU; SVT-AV1 is far faster — recommend **SVT-AV1** for the AV1 tier. |
| **C. System-`ffmpeg` (user installs, e.g. via Homebrew) referenced at runtime** | whatever user's build has | whatever | We ship *no* FFmpeg binary; we ship an *integration test* that exercises it | **Compliant with policy on shipped bytes**, but **violates ADR-008/TDD:346/444 intent** (self-contained, controlled, reproducible process; no assumption of the user's codec set per `TDD.md:318`). | Acceptable only as a debug/dev convenience, never as the shipping mode. Conflicts with the "controlled and understandable workflow" and the reproducibility requirement (`PROCESSING_PIPELINE.md` §13). |

## 4. Recommendation (to feed ADR-017, not adopted here)

Choose **Option B** — distribute an **LGPLv2.1+ FFmpeg** built with GPL-only encoders disabled, use **SVT-AV1** for the AV1 tier and the macOS **hardware encoder** (VideoToolbox) for the H.264/HEVC tier, and add a build-time assertion that no GPL/x264/x265 symbols are linked in the shipped binary. Ship FFmpeg's LGPL notice + modification list with the app to keep the *restricted* class within its conditions. This satisfies `policy.yaml`, matches ADR-008's "controlled, reproducible FFmpeg process," and keeps AV1 fast enough for a real timelapse workload.

Open items ADR-017 must close before Phase 5: (a) exact provenance/build of the shipped FFmpeg (static vs dynamic, version); (b) confirmation that H.264/HEVC uses hardware path only and never falls back to x264; (c) where the LGPL notice text lives in the packaged app; (d) whether AV1 is a default or opt-in profile (it affects "default output format" in `PRD.md` §5.7).

## 5. What this document does NOT decide

- It does not pick an encoder, build, or distribution mode.
- It does not amend `DECISIONS.md`, `TDD.md` or `policy.yaml`.
- It records no exception in `licence-exceptions.yaml`.

On approval of the recommendation, the selected option becomes **ADR-017** and its consequences are applied to `TDD.md` §9 (dependencies/FFmpeg distribution) and `PRD.md` §5.7 (supported codecs / default output).
