# Acceptance Gates — VisionForge P31

## Gate A — Source-of-truth cleanup

Pass only if:

```bash
test -d src-tauri
test ! -d s
find . -name Cargo.toml -print | sort | tee /tmp/vf_cargo_tomls.txt
! grep -q '^./s/Cargo.toml$' /tmp/vf_cargo_tomls.txt
python3 codex-p31-superpass/validation/vf_assert_no_duplicate_tauri_root.py --repo .
```

If `s/` is not deleted because salvage is incomplete, the pass must stop before feature work.

## Gate B — Build/test baseline

Pass only if:

```bash
npm ci
npm run build
npm audit --audit-level=moderate
cd src-tauri && cargo fmt --all -- --check
cd src-tauri && cargo check --all-targets
cd src-tauri && cargo test --all-targets
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run tauri build
```

If Rust/Tauri tools are unavailable, record exact installed versions/missing command and do not claim release readiness.

## Gate C — Config source-of-truth

Pass only if:

- `ConfigProvider` or equivalent shared store exists at app root.
- `PromptStudio` and `SettingsPanel` consume the same shared config instance.
- `useConfig` no longer creates isolated state per consumer.
- A settings save updates visible Prompt Studio behavior without reload.

## Gate D — Auto-approve real behavior

Pass only if:

- When `pipeline.autoApprove=true`, completed pipeline queues jobs without manual `ApprovalGate` click.
- Queued jobs preserve `selectedConcept` and `autoApproved` in queue payload or normalized columns.
- Gallery insert uses actual values, not `None`/`false` hard-codes.
- Auto-approve off preserves manual gate behavior.

## Gate E — Concurrency and cancellation

Pass only if:

- No production path acquires `db` and then `config` in the same lock scope.
- Queue cancel, generation cancel, permanent delete, and AI batch path resolution tests pass.
- Cancel after image download cleans staged files and does not create gallery rows.

## Gate F — External API honesty

Pass only if:

- Ollama unload checks status/body and returns warning/error.
- ComfyUI `/queue`, `/history`, `/free`, `/interrupt` check status/body.
- UI receives user-visible failure/degradation messages.
- Tests mock non-2xx responses.

## Gate G — File/DB integrity

Pass only if:

- Generated images are written to staging/temp path before finalization.
- DB insert and queue completion are one transaction or have equivalent rollback semantics.
- Permanent delete handles DB and filesystem failure without lying.
- Export validates filenames.

## Gate H — Public proof packet

Pass only if `VISIONFORGE_PROOF_PACKET.md` exists and records:

- commit hash and dirty state;
- Node/npm versions;
- Rust/cargo versions;
- commands run;
- pass/fail/skipped checks;
- Ollama smoke result or blocker;
- ComfyUI smoke result or blocker;
- screenshots/demo evidence if public release is claimed;
- README claim matrix.
