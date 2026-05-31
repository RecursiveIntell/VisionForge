# Rollback and Quarantine Plan

## Before edits

```bash
git status --short
git branch --show-current
git rev-parse HEAD
mkdir -p .codex-runs/P31/receipts
cp -a src-tauri .codex-runs/P31/receipts/src-tauri.before
[ ! -d s ] || cp -a s .codex-runs/P31/receipts/s.before
cp package.json package-lock.json .codex-runs/P31/receipts/
```

## Duplicate backend quarantine

Preferred end state: delete `s/` after salvage.

If Codex is not confident enough to delete it:

```bash
mkdir -p docs/source-quarantine/P31_duplicate_backend
mv s docs/source-quarantine/P31_duplicate_backend/s
```

Then update z.py/package policy to exclude `docs/source-quarantine/` and fail if `s/` reappears.

## Dependency rollback

Before dependency edits:

```bash
cp package.json package-lock.json .codex-runs/P31/receipts/dependency-before/
```

Rollback:

```bash
cp .codex-runs/P31/receipts/dependency-before/package.json .
cp .codex-runs/P31/receipts/dependency-before/package-lock.json .
npm ci
```

## Large refactor rollback

Use per-phase commits if possible:

```bash
git add -A && git commit -m "p31 phase N: <scope>"
```

If no git commits are allowed, create file-level backups in `.codex-runs/P31/receipts/backups/` before each risky refactor.

## Stop conditions

Stop instead of fixing forward when:

- `src-tauri/` vs `s/` ownership is unclear after diff review;
- Rust build fails with broad type/API breakage unrelated to the current phase;
- dependency upgrades require major migration;
- smoke tests require unavailable local services and behavior cannot be mocked;
- validation script detects a regression after Codex claims completion.
