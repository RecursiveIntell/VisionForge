# Phase 01 — Duplicate Backend Source-of-Truth Repair

## Goal

Remove source-of-truth drift caused by duplicate divergent backend roots.

## Required actions

1. Treat `src-tauri/` as canonical.
2. Run:

```bash
diff -qr src-tauri s || true
```

3. For each differing file, decide:
   - ignore stale `s` content;
   - salvage exact hunk into `src-tauri/`;
   - record why.
4. Write `.codex-runs/P31/duplicate_backend_salvage_ledger.md`.
5. Delete `s/` or move it to `docs/source-quarantine/P31_duplicate_backend/s` outside package scope.
6. Add/update z.py or a validation guard so future packages fail when both roots exist.

## Acceptance

```bash
python3 codex-p31-superpass/validation/vf_assert_no_duplicate_tauri_root.py --repo .
find . -name Cargo.toml -print | sort
```

## Stop condition

Do not continue to feature work while `s/` remains in package root.
