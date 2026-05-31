# Phase 04 — Auto-Approve End-to-End Repair

## Goal

Make auto-approve real, not just a checkbox.

## Required implementation

1. Ensure auto-approve setting persists through shared config save.
2. When pipeline completes with `autoApprove=true`, automatically enqueue generation job(s) using current generation settings.
3. Add `autoApproved` and `selectedConcept` to queue payload or normalized queue metadata.
4. Ensure backend queue executor inserts gallery row with real `auto_approved` and `selected_concept`, not hard-coded false/None.
5. Preserve manual gate when auto-approve is false.
6. Surface queue success/failure toast/log.

## Acceptance

```bash
python3 codex-p31-superpass/validation/vf_assert_autoapprove_contract.py --repo .
npm run build
cd src-tauri && cargo test --all-targets
```

Manual smoke:

- Auto-approve on: submit idea, pipeline completes, job appears in queue without pressing Generate.
- Generated gallery image shows Auto-approved badge/metadata.
- Auto-approve off: approval gate still appears and requires Generate.
