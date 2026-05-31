# Zip Source Certifier Report

## Summary

- Script version: `2026.05.22-p31`
- Created UTC: `2026-05-27T21:25:02Z`
- Root: `/home/sikmindz/Coding/visionforge`
- Archive root: `/home/sikmindz/Coding/visionforge`
- Output: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T212459Z.zip`
- Include roots: `1`
- External Cargo path dependency roots: `0`
- Profile: `generic` requested as `auto`
- Mode: `next-codex-context`
- Package role: `next-codex-context`
- Strict: `True`
- Dry run: `False`
- Included files: `190`
- Included bytes: `1553927`
- Excluded files: `21`
- Pruned dirs: `9`
- Findings: `1` (`0` errors, `0` warnings)
- Archive zip-byte SHA-256: `897c267a8d91c310d293aad57ec9f56c28229178edc65b44698bc9782957187c`
- Archive hash semantics: `zip-byte-sha256-not-canonical-content-hash`
- Content manifest SHA-256: `df97e8dab09143a0e7b7d06a5e22290b5bfafaac528375231eb6c71283ef343d`
- Ecosystems detected: `rust, node, git`
- Codex archive enabled: `True`
- Codex archive planned: `91`
- Codex archive moved: `91`
- Codex active stale after normalization: `0`
- Root Markdown archive enabled: `False`
- Root Markdown inspected: `5`
- Root Markdown protected: `4`
- Root Markdown candidates: `0`
- Root Markdown ambiguous: `1`
- Root Markdown moved: `0`
- Root Markdown collisions: `0`
- Root package archive enabled: `True`
- Root package inspected: `23`
- Root package protected: `4`
- Root package candidates: `6`
- Root package moved: `6`
- Root package skipped existing: `0`
- Root package collisions: `0`

## Ecosystem parity

| Ecosystem | Detected | Manifests | Missing expected | Dry-run status |
|---|---:|---:|---:|---|
| `rust` | `True` | 1 | 0 | `available-not-run` |
| `python` | `False` | 0 | 0 | `not-applicable` |
| `node` | `True` | 2 | 0 | `available-not-run` |
| `go` | `False` | 0 | 0 | `not-applicable` |
| `docker` | `False` | 0 | 0 | `not-applicable` |
| `git` | `True` | 1 | 0 | `available-not-run` |

## Decision provenance

- Decisions recorded: `220`
- Includes: `190`
- Excludes: `21`
- Pruned dirs: `9`

## Validation findings

| Severity | Code | Path | Detail |
|---|---|---|---|
| info | `git-metadata-excluded` | `.git/` | Git metadata detected and intentionally excluded from transferable package contents. |

## Included files by extension

| Extension | Count |
|---|---:|
| `.rs` | 66 |
| `.tsx` | 46 |
| `.md` | 34 |
| `.ts` | 21 |
| `.json` | 11 |
| `.css` | 2 |
| `.js` | 2 |
| `<no-extension>` | 2 |
| `.csv` | 1 |
| `.html` | 1 |
| `.lock` | 1 |
| `.log` | 1 |
| `.py` | 1 |
| `.toml` | 1 |

## Included files by top-level path

| Top-level path | Count |
|---|---:|
| `src-tauri` | 75 |
| `src` | 68 |
| `.codex-runs` | 20 |
| `docs` | 12 |
| `.gitignore` | 1 |
| `AGENTS.md` | 1 |
| `CLAUDE.md` | 1 |
| `PACK_MANIFEST.json` | 1 |
| `README.md` | 1 |
| `VISIONFORGE_PROOF_PACKET.md` | 1 |
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
| `image-disabled` | 18 |
| `archive-file` | 1 |
| `generated-output` | 1 |
| `unsupported-extension-or-basename` | 1 |

## Sidecar files

- Manifest: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T212459Z.manifest.json`
- Markdown report: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T212459Z.report.md`
- Excluded file list: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T212459Z.excluded.json`
- Findings: `/home/sikmindz/Coding/visionforge/visionforge-generic-next-codex-context-20260527T212459Z.findings.json`

## Interpretation

This package passed the configured validation gates.
