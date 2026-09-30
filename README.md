# VisionForge

VisionForge is a local-first Tauri 2 desktop app for prompt-engineering and image-generation workflow orchestration using a React/TypeScript frontend and a Rust/Tauri backend. It is designed to work with local Ollama and ComfyUI services.

## Current Status

This repository is not marked release-ready. The [May 27, 2026 proof packet](VISIONFORGE_PROOF_PACKET.md) reports frontend build, Rust check/test/clippy, audit, and static P31 validations passing, but runtime smoke tests against live Ollama and ComfyUI were not completed in that recorded pass. Its `npm run tauri build` attempt produced the release binary plus deb/rpm bundles, then failed during AppImage bundling at `linuxdeploy`.

Those historical receipts are tied to the snapshot named in the packet, including its then-dirty worktree. They are not fresh test results for the current checkout.

## Prerequisites

- Node.js compatible with the locked Vite 7 dependency (Node 20.19+ or 22.12+)
- Rust and the native Tauri 2 system dependencies for the target OS
- Running Ollama and ComfyUI services, with the intended models/checkpoints installed, for live workflows

## Quickstart

```bash
npm ci
npm run build
npm run typecheck
npm audit --audit-level=moderate

cd src-tauri
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cd ..

npm run tauri build
python3 codex/validation/run_all_validations.py --repo .
```

For local development:

```bash
npm run dev
```

For the desktop app:

```bash
npm run tauri dev
```

## Source Roots

- `src/`: React/TypeScript frontend.
- `src-tauri/`: canonical Rust/Tauri backend.
- `docs/SPEC.md`: product intent, not implementation proof.
- `VISIONFORGE_PROOF_PACKET.md`: dated build/test and claim evidence from May 27, 2026.
- `docs/source-quarantine/P31_duplicate_backend/s/`: quarantined stale duplicate backend root from the P31 pass.

## Feature Status

| Feature | Status | Evidence | Limitation |
|---|---|---|---|
| Single backend root | Implemented/proven | `vf_assert_no_duplicate_tauri_root.py` passes | Quarantined duplicate remains under docs for audit only |
| Shared frontend config provider | Implemented/proven by static validation/build | `src/context/ConfigContext.tsx`, `npm run build`, config-provider validation | Manual settings-to-prompt-studio smoke not run |
| Auto-approve queue handoff | Implemented/proven by static validation/build | `PromptStudio` auto-enqueues and queue/gallery metadata is wired | Live pipeline smoke not run |
| Selected concept metadata | Partial | Queue payload and gallery row carry `selectedConcept`/`selected_concept` | UI selection is lineage metadata/inspection, not a prompt-regeneration control |
| Queue/gallery metadata | Reported by the dated tests | May 27 packet reports 181 Rust tests passed | Live ComfyUI generation smoke not run |
| Ollama/ComfyUI HTTP status honesty | Implemented/proven by static validation | Status helpers check non-2xx for configured routes | Non-2xx mock tests still need expansion |
| Lock-order hardening | Implemented/proven by static validation | `vf_assert_no_lock_order_inversions.py` passes | Static heuristic only |
| Export filename validation | Implemented/proven by static validation/tests | Export validation script and Rust tests pass | Invalid-row quarantine policy is fail-fast |
| File/DB rollback | Partial | Cancellation cleanup path and permanent-delete errors are explicit | Full staged-file transaction tests remain needed |
| Tauri packaging | Partial | Binary, deb, rpm built during `npm run tauri build` | AppImage bundling failed at `linuxdeploy` |

## Known Limitations

- Release readiness is blocked until live Ollama and ComfyUI smoke tests are recorded.
- The recorded May 27 AppImage build failed at `linuxdeploy`; that run created deb/rpm artifacts first. Re-run packaging before claiming the blocker is fixed or still reproducible on a new host.
- Frontend lint and unit-test runners are not installed; `npm run lint` and `npm run test` are explicit deferred placeholders.
- The selected concept UI records lineage metadata. It does not rerun prompt engineering when changed after pipeline completion.
- File/DB persistence has cleanup safeguards, but a fuller staged-file transaction test suite is still needed.

## Proof

See [VISIONFORGE_PROOF_PACKET.md](VISIONFORGE_PROOF_PACKET.md) and [the P31 audit](.codex-runs/P31/final_audit_report.md) for historical command receipts, validation results, blockers, and rollback notes. The feature table summarizes that recorded proof; regenerate the checks and live-service smoke evidence for a new release decision.
