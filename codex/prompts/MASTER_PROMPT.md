# MASTER PROMPT — VisionForge P31 Closing/Hardening/Release-Proof Super-Pass

You are executing a deterministic, auditable Codex implementation pass on VisionForge.

Your job is not to maximize apparent progress. Your job is to make the repository correct and release-provable.

## Source hierarchy

1. Current repo files, command output, build/test logs.
2. This P31 bundle: `AGENTS.md`, phase prompts, validation scripts, issue matrix, acceptance gates.
3. Existing `docs/SPEC.md` and `README.md` as intent/public docs only.
4. Prior generated sidecars only as package evidence, not implementation proof.

## Non-negotiable first repair

Before feature work, resolve duplicate backend roots.

- `src-tauri/` is canonical by default.
- `s/` is stale duplicate source unless a specific diff hunk is proven newer and salvaged.
- Do not edit both roots.
- Do not leave both roots in final package scope.

## Required phase order

Run phases in order:

0. Preflight and inventory.
1. Duplicate backend salvage/quarantine/delete.
2. Build/test baseline and dependency audit.
3. Frontend config source-of-truth repair.
4. Auto-approve end-to-end behavior.
5. Selected concept semantics.
6. Lock-order/concurrency/cancellation repair.
7. Ollama/ComfyUI external API honesty.
8. File/DB transactional integrity and export safety.
9. Dependency/security/tooling hardening.
10. README/spec/proof packet alignment.
11. Final validation and hostile-auditor handoff.

At every phase boundary, run the relevant validation scripts and record results.

## Required final artifacts

Create or update:

```text
.codex-runs/P31/
  startup_preflight.md
  source_inventory.md
  duplicate_backend_salvage_ledger.md
  commands_run.log
  validation_results.md
  phase_00_report.md
  ...
  phase_11_report.md
  final_audit_report.md
  rollback_notes.md
VISIONFORGE_PROOF_PACKET.md
README.md
```

## Required final response

Your final response must contain:

1. Changed files.
2. Deleted/quarantined files.
3. Commands run with pass/fail/skipped.
4. Validation script results.
5. Acceptance gates passed/failed.
6. Remaining blockers.
7. Rollback instructions.

Do not claim completion unless receipts exist.
