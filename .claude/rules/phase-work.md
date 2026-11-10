# Working in phases

- The gate of a phase is `make gate` unless `PHASES.md` says otherwise; run it exactly as written.
- Do not widen a Tauri capability, add a plugin that talks to a host, or add a dependency outside the plan without asking.
- A change that stores or sends more data updates `legal/data-map.yaml` in the same commit.
- Commits are Conventional Commits, one logical change each; stage named paths.
