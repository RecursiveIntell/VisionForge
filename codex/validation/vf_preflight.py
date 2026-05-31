#!/usr/bin/env python3
import argparse, json, os, subprocess, sys
from pathlib import Path


def run(cmd, cwd):
    try:
        p = subprocess.run(cmd, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        return {"cmd": cmd, "code": p.returncode, "stdout": p.stdout.strip(), "stderr": p.stderr.strip()}
    except Exception as e:
        return {"cmd": cmd, "error": str(e)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--repo', default='.')
    args = ap.parse_args()
    repo = Path(args.repo).resolve()
    checks = []
    checks.append({"name":"repo_exists", "ok": repo.exists(), "detail": str(repo)})
    checks.append({"name":"src_exists", "ok": (repo/'src').is_dir(), "detail":"src/"})
    checks.append({"name":"src_tauri_exists", "ok": (repo/'src-tauri').is_dir(), "detail":"src-tauri/"})
    checks.append({"name":"duplicate_s_present", "ok": not (repo/'s').exists(), "detail":"s/ must be absent by end of Phase 01"})
    checks.append({"name":"package_json", "ok": (repo/'package.json').is_file(), "detail":"package.json"})
    checks.append({"name":"package_lock", "ok": (repo/'package-lock.json').is_file(), "detail":"package-lock.json"})
    checks.append({"name":"cargo_toml", "ok": (repo/'src-tauri'/'Cargo.toml').is_file(), "detail":"src-tauri/Cargo.toml"})
    commands = [run(['git','status','--short'], repo), run(['node','--version'], repo), run(['npm','--version'], repo), run(['cargo','--version'], repo)]
    ok = all(c['ok'] for c in checks if c['name'] != 'duplicate_s_present')
    print(json.dumps({"repo": str(repo), "checks": checks, "commands": commands}, indent=2))
    if not ok:
        return 2
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
