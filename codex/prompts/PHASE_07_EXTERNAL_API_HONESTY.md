# Phase 07 — Ollama and ComfyUI External API Honesty

## Goal

Stop hiding failures from Ollama unload and ComfyUI control/status routes.

## Required implementation

1. Add shared HTTP status helper preserving status and response body.
2. Ollama `unload_model` must check network result, HTTP status, and body.
3. ComfyUI `get_history` must distinguish non-2xx from not-yet-complete/missing history.
4. ComfyUI `get_queue_status`, `free_memory`, and `interrupt` must check status.
5. UI should surface warning/degradation for nonfatal cleanup failures.
6. Tests must mock non-2xx responses where possible.

## Acceptance

```bash
python3 codex-p31-superpass/validation/vf_assert_external_service_status_checks.py --repo .
cd src-tauri && cargo test --all-targets
```
