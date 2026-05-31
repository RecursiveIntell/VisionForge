#!/usr/bin/env python3
import argparse, re, sys
from pathlib import Path

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    repo=Path(ap.parse_args().repo).resolve()
    offenders=[]
    for p in (repo/'src-tauri/src').rglob('*.rs'):
        text=p.read_text(errors='ignore')
        # Heuristic: db lock followed shortly by config read before a closing block comment boundary.
        for m in re.finditer(r'state\.db\.lock\(\)[\s\S]{0,700}?state\.config\.read\(\)', text):
            line=text[:m.start()].count('\n')+1
            offenders.append(f'{p.relative_to(repo)}:{line}')
    if offenders:
        print('FAIL possible db->config lock-order inversions:')
        for o in offenders: print(' -', o)
        return 1
    print('PASS: no obvious db->config inversion pattern')
    return 0
if __name__ == '__main__': raise SystemExit(main())
