#!/usr/bin/env python3
import argparse, sys
from pathlib import Path
REQUIRED=['Build/test receipts','Runtime smoke receipts','Claim matrix','Release decision']

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    repo=Path(ap.parse_args().repo).resolve(); p=repo/'VISIONFORGE_PROOF_PACKET.md'
    if not p.exists():
        print('FAIL missing VISIONFORGE_PROOF_PACKET.md'); return 1
    t=p.read_text(errors='ignore')
    missing=[s for s in REQUIRED if s not in t]
    if missing:
        print('FAIL proof packet missing sections:'); [print(' - '+m) for m in missing]; return 1
    print('PASS: proof packet skeleton exists')
    return 0
if __name__=='__main__': raise SystemExit(main())
