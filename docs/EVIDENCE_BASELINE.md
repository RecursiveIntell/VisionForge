# Evidence Baseline — VisionForge P31

## Uploaded package facts

- Package created UTC: `2026-05-27T01:50:57Z`.
- Included files: `234`.
- Detected ecosystems: `rust`, `node`, `git`.
- Rust manifests detected: `2`.
- Node manifests detected: `2`.
- Rust dry-run status: `available-not-run`.
- Node dry-run status: `available-not-run`.
- Top-level included paths include both `s` and `src-tauri`, 75 files each.

## Local inspection facts from this bundle generation

Commands run against extracted archive:

```bash
unzip -q visionforge-generic-next-codex-context-20260527T015056Z.zip
find . -maxdepth 2 -type f | sort
find . -name Cargo.toml -print | sort
diff -qr src-tauri s
npm ci
npm run build
npm audit --json
cargo --version
```

Observed:

- `src-tauri/Cargo.toml` and `s/Cargo.toml` both exist.
- `diff -qr src-tauri s` reports divergent files.
- `npm run build` passed after `npm ci`.
- `npm audit` reports 4 vulnerabilities: 3 high, 1 moderate.
- `cargo` was not available in the sandbox, so Rust build/test remains unproven.

## Non-negotiable interpretation

A package can pass z.py gates while still failing release gates. This pass must produce build/test/runtime receipts before claiming release readiness.
