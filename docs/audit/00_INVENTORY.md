# Timelapse Studio — Documentation Audit: Inventory

**Date:** 2026-11-09
**Status:** Preparation complete — awaiting approval to start per-document audits
**Scope rule:** one document per audit session; context is capped per session.

---

## 1. Documentation inventory (present)

All in `docs/`, all version 0.1, project stage: pre-implementation (per AGENTS.md §10).

| Document | Lines (approx.) | Size | Stated status |
| --- | --- | --- | --- |
| PRD.md | ~300 | 11.4 kB | Initial specification — pending review |
| TDD.md | ~230 | 8.8 kB | Initial technical proposal — pending review |
| ARCHITECTURE.md | ~150 | 6.3 kB | Proposed architecture |
| DATABASE_SCHEMA.md | ~260 | 10.1 kB | Initial logical schema — implementation pending |
| PROCESSING_PIPELINE.md | ~200 | 7.6 kB | Initial workflow specification |
| DECISIONS.md (ADR register) | ~130 | 5.0 kB | Initial decisions and open questions |
| CURRENT_STATUS.md | ~90 | 3.6 kB | Pre-implementation, last updated 2026-11-09 |

Total docs: ~72 kB. Each file is small enough to audit in one low-context session.

## 2. Missing expected documents

| Document | Why expected | Source of expectation |
| --- | --- | --- |
| `docs/DEVELOPMENT_ROADMAP.md` | Named as "expected but not yet present" | AGENTS.md §3, line 32 |
| `LICENSE*` / dependency licence file (deny.toml or equivalent) | License policy requires shipped-package tracking | `_foundation/tech/licences/policy.yaml` |
| `THIRD-PARTY.md` or attribution record | Required for CC-BY-4.0 and OFL-1.1 usage | same policy file |
| Localisation assets (Fluent `.ftl`, fr-CA + en) | Release requires fr-CA and en; fr-CA is a release blocker | `etzian.yaml` dev.locales; `_foundation/i18n/README.md` |
| ADRs for project-specific stack deviations | Stack defaults may change only with an ADR | EXECUTION_PLAN.md D7 |
| Release / distribution plan (Merchant of Record, Apple team/signing) | Bundle ID gaps in profile | `etzian.yaml` apple section (team_id, signing_identity empty) |

## 3. Available agent definitions

Project-local (`.opencode/agents/`):

| Agent | File | Notes |
| --- | --- | --- |
| Global rules | AGENTS.md (137 lines) | Project instructions; documentation is source of truth; filesystem safety rules; reviewer must not modify files |
| developer.md | ~2.2 kB | Implementation planning/coordination per AGENTS.md §9 |
| reviewer.md | ~2.3 kB | Independent read-only review |
| specialist.md | ~2.0 kB | Scoped technical tasks with reporting of files/decisions/tests |

Shared company agents (in `etzian` plugin, `_foundation/claude-plugin/agents/`, available but not project-specific): architect, i18n-guardian, legal-compliance, phase-runner, rust-reviewer, translator, ui-reviewer.

## 4. Relevant company policies and sources

| Policy | Rule (summary) | Source |
| --- | --- | --- |
| Licences — allowed | MIT, Apache-2.0, BSD, ISC, Zlib, CC0 and similar free licences allowed for shipping | `~/etzian/_foundation/tech/licences/policy.yaml` (allow block) |
| Licences — restricted | MPL-2.0, EPL-2.0, LGPL dev-only; shipped use needs per-project `licence-exceptions.yaml` entry | same file (restricted block) |
| Licences — denied | GPL-2.0/3.0, AGPL-3.0, SSPL-1.0, BUSL-1.1, Elastic-2.0, CC-BY-NC/SA never ship | same file (deny block) |
| Localisation | Ship in fr-CA and en (BCP 47); fr-CA default when OS locale is French/QC; fr-CA is a release blocker | `etzian.yaml` dev.locales; CLAUDE.md |
| Naming | "<App> — by Etzian" endorsement architecture ("Timelapse Studio — by Etzian") | `etzian.yaml` brand.architecture (D2) |
| Bundle ID | `<bundle_prefix>.<app-slug>` with prefix `com.etzian` → expected `com.etzian.timelapse-studio` (slug to confirm) | `etzian.yaml` apple.bundle_prefix |
| Distribution / commercialization | Merchant of Record for payments (D4); dev self-hosted on LAN (D5), production ADR per project; Law 25 privacy constraints apply to data location | EXECUTION_PLAN.md §9 (D4, D5) |
| Apple identity | team_id and signing_identity are empty gaps in the company profile — blocker for shipping | `etzian.yaml` apple section |
| Git / commits | Conventional Commits; changelog via git-cliff (D6); no new framework/dependency without justification | EXECUTION_PLAN.md D6; AGENTS.md §6 |
| Tech stack defaults | Rust, Tauri 2, Svelte 5 + TS — matches project docs | EXECUTION_PLAN.md; AGENTS.md §2 |

## 5. Proposed audit sequence (one document per session)

Ordered so each document's audit can rely on the previous conclusions:

1. **PRD.md** — product scope, target users, workflows (baseline for everything else).
2. **DECISIONS.md** — verify ADRs are consistent with PRD and company defaults; list open questions.
3. **TDD.md** — stack choices, library candidates against the licence policy.
4. **ARCHITECTURE.md** — separation of UI/processing vs TDD and AGENTS.md §6.
5. **DATABASE_SCHEMA.md** — SQLite authoritative-store rule, quarantine/deletion safety vs AGENTS.md §5.
6. **PROCESSING_PIPELINE.md** — non-destructive rules, recoverability vs PRD/TDD features.
7. **CURRENT_STATUS.md** — reconcile with audit findings; refresh if approved.
8. **Cross-cutting checks:** missing-document gaps from §2 (roadmap, licences, i18n assets), bundle ID / naming confirmation, company profile gaps (`ez profile --check`).

Each session produces `docs/audit/NN_<TOPIC>_AUDIT.md`. No document is modified during its own audit.

## 6. Blockers and missing information

- `DEVELOPMENT_ROADMAP.md` absent (flagged as expected by AGENTS.md).
- Apple `team_id` and `signing_identity` empty in company profile → distribution blocked until filled (`ez profile --check`).
- Production hosting decision is "tbd" per D5 — needed before any deploy-related audit conclusion.
- Exact app slug for bundle ID not documented anywhere yet (assumed `timelapse-studio`; needs confirmation).
- No licence/dependency manifest exists yet (pre-implementation), so licence audit is policy-verification only until code exists.
- Prior session transcript (`session-ses_e3e3.md`, 331 kB) exists in the repo root; not read in this phase — context hygiene.
