# Rollback Notes

To roll back this P31 pass manually:

1. Restore `s/` from quarantine if needed:

```bash
mv docs/source-quarantine/P31_duplicate_backend/s ./s
```

2. Revert P31 source edits with git after reviewing unrelated pre-existing changes:

```bash
git diff
git checkout -- README.md package.json package-lock.json VISIONFORGE_PROOF_PACKET.md src src-tauri
```

Use `git checkout --` only if you intend to discard all uncommitted changes in those paths, including pre-existing package edits.

3. Remove P31 audit artifacts if discarding the run:

```bash
rm -rf .codex-runs/P31 docs/source-quarantine/P31_duplicate_backend
```

4. Reinstall dependencies from the restored lockfile:

```bash
npm ci
```
