# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-05-27T01:50:13Z`
- Root: `/home/sikmindz/Coding/visionforge`
- Archive root: `/home/sikmindz/Coding/visionforge`
- Output: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T015012Z.zip`
- Include roots: `1`
- External Cargo path dependency roots: `0`
- Profile: `generic` requested as `auto`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `233`
- Included bytes: `2376232`
- Excluded files: `37`
- Pruned dirs: `4`
- Findings: `2` (`1` errors, `0` warnings)
- Content manifest SHA-256: `fb9588589e1dfef2cd66067c1a13eaae70ae53267217268ac73a2ab197b9ef0d`
- Ecosystems detected: `rust, node, git`
- Codex archive enabled: `True`
- Codex archive planned: `0`
- Codex archive moved: `0`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `3`
- Root Markdown protected: `3`
- Root Markdown candidates: `0`
- Root Markdown ambiguous: `0`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `True`
- Root package inspected: `20`
- Root package protected: `3`
- Root package candidates: `6`
- Root package moved: `6`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 2 | 0 | `available-not-run` |
| `python` | `False` | 0 | 0 | `not-applicable` |
| `node` | `True` | 2 | 0 | `available-not-run` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `True` | 1 | 0 | `available-not-run` |

## Decision provenance

- Decisions recorded: `274`
- Includes: `233`
- Excludes: `37`
- Pruned dirs: `4`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| error | `context-package-command-evidence-missing` | `/` | Context/audit package manifest must include command-run evidence (commands_run.log, commands_run.receipts.jsonl, COMMAND_RECEIPTS.jsonl, COMMAND_EXECUTION_RECEIPTS.jsonl, or *_COMMANDS_RUN.md). |
| info | `git-metadata-excluded` | `.git/` | Git metadata detected and intentionally excluded from transferable package contents. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.rs` | 132 |
| `.tsx` | 45 |
| `.ts` | 21 |
| `.json` | 16 |
| `.md` | 6 |
| `<no-extension>` | 3 |
| `.css` | 2 |
| `.js` | 2 |
| `.lock` | 2 |
| `.toml` | 2 |
| `.html` | 1 |
| `.py` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `s` | 75 |
| `src-tauri` | 75 |
| `src` | 67 |
| `docs` | 4 |
| `.gitignore` | 1 |
| `CLAUDE.md` | 1 |
| `README.md` | 1 |
| `index.html` | 1 |
| `package-lock.json` | 1 |
| `package.json` | 1 |
| `postcss.config.js` | 1 |
| `tailwind.config.js` | 1 |
| `tsconfig.json` | 1 |
| `tsconfig.node.json` | 1 |
| `vite.config.ts` | 1 |
| `z.py` | 1 |

## Exclusion reasons

| Reason | Count |
|---|---:|
| `image-disabled` | 33 |
| `unsupported-extension-or-basename` | 2 |
| `archive-file` | 1 |
| `generated-output` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T015012Z.manifest.json`
- Markdown report: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T015012Z.report.md`
- Excluded file list: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T015012Z.excluded.json`
- Findings: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T015012Z.findings.json`

## Interpretation

This package has validation errors. Under `--strict`, it should not be treated as a complete handoff until corrected or explicitly waived.
