# Phase 11 — Final Validation and Hostile-Auditor Handoff

## Goal

Prove the pass closed the intended issues, or record exact remaining blockers.

## Required actions

```bash
python3 codex-p31-superpass/validation/run_all_validations.py --repo .
npm ci
npm run build
npm audit --audit-level=moderate
cd src-tauri && cargo fmt --all -- --check
cd src-tauri && cargo check --all-targets
cd src-tauri && cargo test --all-targets
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run tauri build
```

Write:

```text
.codex-runs/P31/final_audit_report.md
.codex-runs/P31/validation_results.md
.codex-runs/P31/remaining_delta.md
.codex-runs/P31/rollback_notes.md
```

Final response must not say complete unless these exist or their absence is explicitly justified.
