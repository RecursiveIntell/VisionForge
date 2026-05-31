# Phase 00 — Preflight and Source Inventory

## Goal

Establish current repo truth before edits.

## Required actions

```bash
pwd
git status --short || true
git branch --show-current || true
git rev-parse HEAD || true
find . -maxdepth 3 -type f | sed 's#^./##' | sort | head -300
find . -name Cargo.toml -print | sort
find . -name package.json -o -name package-lock.json -o -name tauri.conf.json | sort
python3 codex-p31-superpass/validation/vf_preflight.py --repo .
python3 codex-p31-superpass/validation/vf_assert_command_parity.py --repo .
```

## Deliverables

- `.codex-runs/P31/startup_preflight.md`
- `.codex-runs/P31/source_inventory.md`
- `.codex-runs/P31/commands_run.log`

## Stop conditions

Stop if repo root is ambiguous, `src-tauri/` is missing, or validation scripts cannot be found.
