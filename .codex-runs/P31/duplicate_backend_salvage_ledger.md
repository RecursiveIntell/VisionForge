# Duplicate Backend Salvage Ledger

Source of truth decision:

- Canonical backend: `src-tauri/`
- Duplicate backend: `s/`
- Final action: quarantined `s/` to `docs/source-quarantine/P31_duplicate_backend/s`

Comparison command:

```bash
diff -qr src-tauri s || true
```

Differing files reviewed:

| File | Decision | Reason |
|---|---|---|
| `src/ai_batch/queue.rs` | No salvage from `s` | `src-tauri` hunk is newer and improves poisoned mutex handling |
| `src/comfyui/client.rs` | No salvage from `s` | `src-tauri` hunk is newer and improves WebSocket message limits |
| `src/commands/queue_cmds.rs` | No salvage from `s` | `src-tauri` hunk adds queue pruning command |
| `src/db/migrations.rs` | No salvage from `s` | `src-tauri` hunk adds indexes |
| `src/db/queue.rs` | No salvage from `s` | `src-tauri` hunk adds queue pruning |
| `src/gallery/storage.rs` | No salvage from `s` | `src-tauri` hunk makes thumbnail failure nonfatal with warning |
| `src/lib.rs` | No salvage from `s` | `src-tauri` hunk validates scoped image directory and adds shutdown signal |
| `src/pipeline/engine.rs` | No salvage from `s` | `src-tauri` hunk validates input and avoids empty-output panics |
| `src/pipeline/engine_streaming.rs` | No salvage from `s` | `src-tauri` hunk mirrors safer pipeline input/output handling |
| `src/pipeline/ollama.rs` | No salvage from `s` | `src-tauri` hunk avoids sending `think` to known non-thinking models |
| `src/queue/executor.rs` | No salvage from `s` | `src-tauri` hunk improves cancellation cleanup and typed settings parsing |
| `src/queue/executor_test.rs` | No salvage from `s` | `src-tauri` hunk matches typed settings error behavior |
| `src/types/generation.rs` | No salvage from `s` | `src-tauri` hunk adds typed generation settings validation |
| `tauri.conf.json` | No salvage from `s` | `src-tauri` hunk tightens explicit CSP endpoints |

Acceptance:

- `python3 codex/validation/vf_assert_no_duplicate_tauri_root.py --repo .`: pass.
- `find . -name Cargo.toml -print | sort`: active root only under `src-tauri/`; quarantined copy under `docs/source-quarantine`.
