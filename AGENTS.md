# AGENTS.md — VisionForge P31 Closing/Hardening Pass

## Project purpose

VisionForge is a local-first Tauri 2 desktop app for prompt-engineering and image-generation workflow orchestration using Ollama and ComfyUI. It has React/TypeScript frontend code and Rust/Tauri backend code.

## Canonical source roots

- `src/` owns the React/TypeScript frontend.
- `src-tauri/` owns the Rust/Tauri backend.
- `docs/SPEC.md` is product intent, not proof of implementation.
- `README.md` is public documentation, not source truth.
- `s/` is **not canonical**. It is a stale duplicate backend root unless a specific hunk is proven newer and deliberately salvaged into `src-tauri/` with a receipt.

## Hard rules

1. Inspect current files before editing.
2. Do not edit `s/` except to quarantine/delete it after salvage review.
3. Do not claim Rust, Tauri, ComfyUI, or Ollama workflows work without command/smoke-test receipts.
4. Do not silently widen API semantics, config semantics, queue semantics, or gallery metadata.
5. Do not hide HTTP failures from ComfyUI or Ollama.
6. Do not add compatibility shims to preserve broken behavior.
7. Do not proceed past a phase boundary until validation and manual invariant checks are recorded.
8. If a validation cannot run because tools are absent, record exact blocker and continue only with bounded non-build work.
9. Public claims must follow proof. Downgrade README claims before release if receipts are missing.

## Required commands

Run these from repo root unless unavailable:

```bash
npm ci
npm run build
npm audit --audit-level=moderate
cd src-tauri && cargo fmt --all -- --check
cd src-tauri && cargo check --all-targets
cd src-tauri && cargo test --all-targets
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run tauri build
python3 codex-p31-superpass/validation/run_all_validations.py --repo .
```

## Required final report

The final report must include:

1. Changed files.
2. Deleted/quarantined files.
3. Commands run with pass/fail/skipped and exact reasons.
4. Validation script results.
5. Feature receipts for auto-approve, config provider, queue/gallery metadata, ComfyUI/Ollama error handling, file/DB rollback, and export validation.
6. Remaining blockers and exact next action.
7. Rollback instructions.
