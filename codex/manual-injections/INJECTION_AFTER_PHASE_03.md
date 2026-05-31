# Manual Guardrail Injection After Phase 03 Config provider

Stop before continuing.

Prove with file paths and command output:

1. You did not edit both `src-tauri/` and `s/` as if both were canonical.
2. No new source-of-truth drift, compatibility shim, or silent semantic widening was introduced.
3. All touched behavior has an acceptance gate or a recorded blocker.
4. Any failed/skipped validation is recorded in `.codex-runs/P31/validation_results.md`.
5. Rollback for the phase is clear.
6. The next phase is still safe to execute.

If you cannot prove these, stop and repair or report the blocker. Do not continue by momentum.
