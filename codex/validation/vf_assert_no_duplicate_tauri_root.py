#!/usr/bin/env python3
import argparse, sys
from pathlib import Path

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--repo', default='.')
    args = ap.parse_args()
    repo = Path(args.repo).resolve()
    failures = []
    if not (repo/'src-tauri').is_dir():
        failures.append('missing canonical src-tauri/')
    if (repo/'s').exists():
        failures.append('stale duplicate root s/ still exists')
    cargo_tomls = sorted(str(p.relative_to(repo)) for p in repo.rglob('Cargo.toml') if '.codex-runs' not in p.parts and 'source-quarantine' not in p.parts)
    if 's/Cargo.toml' in cargo_tomls:
        failures.append('s/Cargo.toml still in package scope')
    print('Cargo.toml files in active scope:')
    for p in cargo_tomls:
        print(' -', p)
    if failures:
        print('\nFAIL:')
        for f in failures: print(' -', f)
        return 1
    print('PASS: one canonical Tauri backend root')
    return 0
if __name__ == '__main__':
    raise SystemExit(main())
