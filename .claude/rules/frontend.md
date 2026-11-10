---
paths:
  - "frontend/**"
---
# Frontend

Load `etzian:design-ui` before touching UI. Components come from `@etzianco/ui`;
check `../_uidesign/src/index.js` before inventing one. App CSS defines only
`--app-*` tokens and `.app-*` classes, layered on `--ez-*`. All text goes through
`t()`; props of shared components take translated strings. `src/lib/legal-ux/`
is a managed copy: never edit it (`ez vendor-ui .`).
