# Phase 02 — Build/Test Baseline and Dependency Audit

## Goal

Separate real build failures from prior package-only success.

## Required actions

```bash
npm ci
npm run build
npm audit --audit-level=moderate || true
cd src-tauri && cargo fmt --all -- --check
cd src-tauri && cargo check --all-targets
cd src-tauri && cargo test --all-targets
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run tauri build
```

Record every command in `.codex-runs/P31/commands_run.log`.

## Fix scope

- If TypeScript fails, fix source.
- If Rust fails, fix source.
- If npm audit fails, defer actual dependency bump to Phase 09 unless build depends on it.

## Deliverables

- `.codex-runs/P31/phase_02_report.md`
- updated issue matrix if new blockers appear.
