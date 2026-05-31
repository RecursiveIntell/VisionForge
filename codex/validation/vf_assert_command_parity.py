#!/usr/bin/env python3
import argparse, re, sys
from pathlib import Path

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    repo=Path(ap.parse_args().repo).resolve()
    src=repo/'src'; lib=repo/'src-tauri/src/lib.rs'
    invoked=set()
    for p in src.rglob('*'):
        if p.suffix not in ('.ts','.tsx'): continue
        text=p.read_text(errors='ignore')
        for m in re.finditer(r"invoke(?:<[^>]+>)?\(['\"]([^'\"]+)['\"]", text):
            invoked.add(m.group(1))
    registered=set()
    if lib.exists():
        text=lib.read_text(errors='ignore')
        registered=set(re.findall(r'commands::[a-zA-Z0-9_]+::([a-zA-Z0-9_]+)', text))
    missing=sorted(invoked-registered)
    unused=sorted(registered-invoked)
    print(f'invoked={len(invoked)} registered={len(registered)}')
    if missing:
        print('FAIL missing registrations:'); [print(' -', x) for x in missing]
        return 1
    if unused:
        print('WARN unused registered commands:'); [print(' -', x) for x in unused]
    print('PASS: no frontend invoke lacks backend registration')
    return 0
if __name__ == '__main__': raise SystemExit(main())
