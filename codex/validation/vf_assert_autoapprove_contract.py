#!/usr/bin/env python3
import argparse, re, sys
from pathlib import Path

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    repo=Path(ap.parse_args().repo).resolve()
    failures=[]
    qe=repo/'src-tauri/src/queue/executor.rs'
    if qe.exists():
        t=qe.read_text(errors='ignore')
        if 'auto_approved: false' in t:
            failures.append('queue executor still hard-codes auto_approved: false')
        if 'selected_concept: None' in t:
            failures.append('queue executor still hard-codes selected_concept: None')
    types=repo/'src-tauri/src/types/queue.rs'
    if types.exists():
        tt=types.read_text(errors='ignore')
        if 'auto_approved' not in tt and 'autoApproved' not in tt:
            failures.append('queue types do not carry auto-approved metadata')
        if 'selected_concept' not in tt and 'selectedConcept' not in tt:
            failures.append('queue types do not carry selected concept metadata')
    ps=repo/'src/components/prompt-studio/PromptStudio.tsx'
    if ps.exists():
        p=ps.read_text(errors='ignore')
        if 'autoApprove' in p and 'handleGenerate' in p and not re.search(r'autoApprove[\s\S]{0,400}(handleGenerate|addToQueue)', p):
            failures.append('PromptStudio does not obviously auto-queue after autoApprove completion')
    if failures:
        print('FAIL auto-approve contract:'); [print(' -', f) for f in failures]
        return 1
    print('PASS: auto-approve metadata no longer obviously hard-coded away')
    return 0
if __name__ == '__main__': raise SystemExit(main())
