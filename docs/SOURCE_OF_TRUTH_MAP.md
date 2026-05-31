# VisionForge Source-of-Truth Map

| Concept | Canonical owner | Non-owner / suspect | Enforcement |
|---|---|---|---|
| Frontend UI/components/hooks/types | `src/` | generated bundles, README claims | TypeScript build + component/hook tests |
| Tauri backend commands/state/db/queue/gallery | `src-tauri/` | `s/` duplicate root | delete/quarantine `s/`; command parity check |
| Product intent | `docs/SPEC.md` | README marketing copy | issue matrix + implementation receipts |
| Public claims | `README.md` after proof packet | old README/spec claims | `VISIONFORGE_PROOF_PACKET.md` required |
| Config persistence | Rust config manager + shared frontend ConfigProvider | isolated `useConfig()` instances | provider tests + manual settings smoke |
| Queue/job truth | SQLite queue tables + command APIs | frontend optimistic state only | queue tests + job event receipts |
| Gallery truth | SQLite images table + filesystem staged files | generated thumbnails/files alone | DB/file rollback tests |
| External ComfyUI/Ollama status | backend client response handling | UI assumptions, best-effort calls | mocked non-2xx tests |
| Security/dependency state | lockfiles + audit receipts | package.json ranges alone | npm audit gate |

## Explicit deletion/quarantine rule

`src-tauri/` wins over `s/` by default. Codex may salvage code from `s/` only by producing:

1. file path;
2. diff hunk;
3. why hunk is newer/better;
4. test impacted;
5. changed destination file in `src-tauri/`;
6. confirmation that `s/` is then removed or moved outside package scope.
