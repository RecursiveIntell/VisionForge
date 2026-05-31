# P31 Source Inventory

Canonical roots:

- `src/`: React/TypeScript frontend.
- `src-tauri/`: Rust/Tauri backend.
- `docs/SPEC.md`: product intent.
- `README.md`: public docs, updated to proof-bounded status.

Duplicate/stale roots:

- `s/` existed at startup with a full duplicate backend tree.
- `s/` was compared against `src-tauri/` with `diff -qr src-tauri s`.
- `s/` was moved to `docs/source-quarantine/P31_duplicate_backend/s`.

Active Cargo manifests after quarantine:

- `src-tauri/Cargo.toml`

Cargo manifests outside active package scope:

- `docs/source-quarantine/P31_duplicate_backend/s/Cargo.toml`

Pre-existing dirty/untracked state:

- The worktree already contained many modified frontend/backend files and untracked P31 bundle files before this pass.
- Those files were treated as package/user state and were not reverted.
