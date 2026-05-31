# Current Research Notes Used by This Bundle

These notes are implementation constraints, not marketing copy.

## Tauri 2

Current Tauri v2 docs describe the command system as the primitive for frontend-to-Rust calls, including async commands and error returns. The fix must keep command registration and frontend invocation in parity.

Current Tauri v2 docs also describe application state management through the Manager API and reading state when commands are called. VisionForge already uses managed backend state; the issue is not Tauri capability, but frontend config state drift and lock-order discipline.

## Ollama

Ollama's API supports `keep_alive` on `/api/generate` and `/api/chat`. Official docs describe `keep_alive: 0` as unloading immediately after a response, and docs also describe unloading a model through an empty chat/messages request with `keep_alive: 0`. VisionForge should keep the unload behavior but must stop discarding HTTP/network failures.

## ComfyUI

ComfyUI official route docs list `/queue`, `/interrupt`, and `/free` as server routes for queue operations, interrupting execution, and freeing memory. VisionForge must treat non-2xx responses from those control/status routes as material failures or warnings, not success.

## Codex

Current OpenAI Codex docs support repo-level `AGENTS.md`, reusable skills, and deterministic hooks. This bundle uses a repo-level `AGENTS.md`, phase prompts, manual injections, and validation scripts rather than relying on a single long prompt.

## npm security

The current `npm audit` from the extracted package reports vulnerabilities in `vite`, `rollup`, `picomatch`, and `postcss`. GitHub Advisory Database confirms current Vite high-severity advisory ranges include `7.0.0 <= vite <= 7.3.1`, and the extracted build used Vite `7.3.1`. Audit must be rerun after dependency changes.
