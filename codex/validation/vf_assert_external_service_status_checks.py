#!/usr/bin/env python3
import argparse, re, sys
from pathlib import Path

def has_status_check_near(text, fn_name):
    m=re.search(r'pub async fn '+re.escape(fn_name)+r'[\s\S]*?\n}\n', text)
    if not m: return False
    body=m.group(0)
    return '.status().is_success()' in body or 'error_for_status' in body or 'ensure_success' in body

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--repo', default='.')
    repo=Path(ap.parse_args().repo).resolve(); failures=[]
    comfy=repo/'src-tauri/src/comfyui/client.rs'
    if comfy.exists():
        t=comfy.read_text(errors='ignore')
        for fn in ['get_history','get_queue_status','free_memory','interrupt']:
            if not has_status_check_near(t, fn): failures.append(f'ComfyUI {fn} lacks explicit status check')
    ollama=repo/'src-tauri/src/pipeline/ollama.rs'
    if ollama.exists():
        t=ollama.read_text(errors='ignore')
        m=re.search(r'pub async fn unload_model[\s\S]*?\n}\n', t)
        if not m: failures.append('missing unload_model')
        else:
            body=m.group(0)
            if 'let _ =' in body: failures.append('Ollama unload_model still discards send result')
            if '.status().is_success()' not in body and 'error_for_status' not in body and 'ensure_success' not in body:
                failures.append('Ollama unload_model lacks status check')
    if failures:
        print('FAIL external API status checks:'); [print(' -', f) for f in failures]
        return 1
    print('PASS: external service status checks detected')
    return 0
if __name__ == '__main__': raise SystemExit(main())
