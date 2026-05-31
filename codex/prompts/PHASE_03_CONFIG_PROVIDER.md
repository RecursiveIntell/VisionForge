# Phase 03 — Frontend Config Source-of-Truth Repair

## Goal

Replace isolated `useConfig()` state copies with one app-level config source.

## Required implementation

1. Add `src/context/ConfigContext.tsx` or equivalent.
2. Provider loads config once at app shell/root.
3. Provider exposes `config`, `loading`, `error`, `saving`, `save`, `update`, `reload`.
4. `PromptStudio`, `SettingsPanel`, and all settings components consume provider state.
5. On `save_config`, update shared state immediately and emit/consume a config-changed event if useful.
6. Hidden mounted pages must not keep stale config snapshots.

## Acceptance

```bash
python3 codex-p31-superpass/validation/vf_assert_frontend_config_provider.py --repo .
npm run build
```

Manual smoke:

1. Change Ollama endpoint/model in settings.
2. Save.
3. Navigate to Prompt Studio without reload.
4. Run health/model action and prove new config is used.
