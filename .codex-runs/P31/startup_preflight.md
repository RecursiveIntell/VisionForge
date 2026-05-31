# P31 Startup Preflight

- Date: 2026-05-27
- Repo: `/home/sikmindz/Coding/visionforge`
- Branch: `master`
- HEAD: `51e4b163a07c636ba5e744f64fb4bbf81e405304`
- Initial blocker: duplicate backend root `s/` existed beside canonical `src-tauri/`.
- Validation path note: bundle prompts referenced `codex-p31-superpass/validation`, but checked-in scripts are under `codex/validation`.

Preflight results:

- `src/`: present.
- `src-tauri/`: present.
- `package.json` and `package-lock.json`: present.
- `src-tauri/Cargo.toml`: present.
- Node/npm/cargo available.
- Initial `vf_preflight.py`: pass for required roots/tools, with `duplicate_s_present=false` as expected before Phase 01.
- `vf_assert_command_parity.py`: pass; warning for unused registered `prune_old_queue_jobs`.
