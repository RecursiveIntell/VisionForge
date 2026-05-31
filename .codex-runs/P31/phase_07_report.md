# Phase 07 Report

Status: implemented/proven by static validation.

Changes:

- Added shared HTTP status helpers for ComfyUI and Ollama.
- ComfyUI `get_history`, `get_queue_status`, `free_memory`, and `interrupt` now check non-2xx status and preserve response bodies in errors.
- Ollama `unload_model` now checks network result, HTTP status, and response body.

Validation:

- `vf_assert_external_service_status_checks.py`: pass.
- `cargo test --all-targets`: pass.

Remaining:

- Add non-2xx mock tests for these routes.
- Run live Ollama/ComfyUI smoke tests.
