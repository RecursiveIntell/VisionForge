#!/usr/bin/env python3
import argparse, re, sys
from pathlib import Path

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    repo=Path(ap.parse_args().repo).resolve(); failures=[]
    exp=repo/'src-tauri/src/gallery/export.rs'
    if not exp.exists():
        failures.append('missing gallery/export.rs')
    else:
        t=exp.read_text(errors='ignore')
        idx=t.find('zip.start_file(&image.filename')
        if idx!=-1:
            window=t[max(0, idx-500):idx+200]
            if 'validate_filename' not in window:
                failures.append('zip.start_file(&image.filename) not guarded by validate_filename nearby')
        if 'validate_filename' not in t:
            failures.append('gallery export never calls validate_filename')
    if failures:
        print('FAIL export safety:'); [print(' -', f) for f in failures]
        return 1
    print('PASS: export filename validation detected')
    return 0
if __name__ == '__main__': raise SystemExit(main())
