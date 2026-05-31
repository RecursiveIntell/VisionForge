#!/usr/bin/env python3
import argparse, subprocess, sys
from pathlib import Path

SCRIPTS = [
 'vf_preflight.py',
 'vf_assert_no_duplicate_tauri_root.py',
 'vf_assert_command_parity.py',
 'vf_assert_frontend_config_provider.py',
 'vf_assert_autoapprove_contract.py',
 'vf_assert_no_lock_order_inversions.py',
 'vf_assert_external_service_status_checks.py',
 'vf_assert_export_filename_validation.py',
 'vf_assert_release_proof_packet.py',
]

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    args=ap.parse_args(); here=Path(__file__).resolve().parent
    failures=0
    for s in SCRIPTS:
        print('\n===',s,'===')
        p=subprocess.run([sys.executable, str(here/s), '--repo', args.repo], text=True)
        if p.returncode:
            failures += 1
    print(f'\nvalidation_failures={failures}')
    return 1 if failures else 0
if __name__=='__main__': raise SystemExit(main())
