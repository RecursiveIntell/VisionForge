# Phase 09 — Dependency Security and Tooling Hardening

## Goal

Clear npm audit findings and add repeatable frontend quality gates.

## Required implementation

1. Upgrade vulnerable packages using the smallest safe change.
2. Prefer `npm audit fix` if it does not cause risky major migration.
3. Add scripts:
   - `typecheck`: `tsc --noEmit`
   - `lint`: if ESLint is added/configured, otherwise record explicit deferred blocker
   - `test`: if Vitest is added/configured, otherwise record explicit deferred blocker
4. Do not add heavy dependencies unless needed.

## Acceptance

```bash
npm ci
npm run build
npm audit --audit-level=moderate
npm run typecheck
```

If lint/test are deferred, final report must explain why and list exact next setup.
