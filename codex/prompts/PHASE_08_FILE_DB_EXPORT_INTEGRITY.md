# Phase 08 — File/DB Integrity and Export Safety

## Goal

Prevent orphan files, missing DB rows, unsafe ZIP paths, and dishonest delete/export outcomes.

## Required implementation

1. Write generated image bytes to a staging/temp filename.
2. Insert gallery row and mark queue completed in one DB transaction where feasible.
3. Rename/move staged file to final only after DB success, or provide equivalent cleanup rollback.
4. On failure, cleanup staged/final files and do not mark queue complete.
5. Permanent delete must handle DB and file failures explicitly.
6. ZIP export must validate every DB filename before `zip.start_file`.
7. Invalid DB rows must fail export or be quarantined with explicit report.

## Acceptance

```bash
python3 codex-p31-superpass/validation/vf_assert_export_filename_validation.py --repo .
cd src-tauri && cargo test --all-targets
```
