# Phase 06 — Lock-Order, Queue Cancellation, and Concurrency Hardening

## Goal

Eliminate deadlock risk and cancellation/file persistence races.

## Required implementation

1. Refactor every path that locks `db` then reads `config` in the same scope.
2. Clone needed config values before DB lock, or release DB lock before config lock.
3. Queue cancel must interrupt ComfyUI best-effort but report failures.
4. Cancellation after image download must clean staged files and avoid gallery rows.
5. Add regression tests for queue cancel, generation cancel, permanent delete, AI batch path resolution, interrupted job requeue.

## Acceptance

```bash
python3 codex-p31-superpass/validation/vf_assert_no_lock_order_inversions.py --repo .
cd src-tauri && cargo test --all-targets
```
