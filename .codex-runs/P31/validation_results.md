# Validation Results

Final validation command:

```bash
python3 codex/validation/run_all_validations.py --repo .
```

Result: pass, `validation_failures=0`.

Per-script results:

- `vf_preflight.py`: pass.
- `vf_assert_no_duplicate_tauri_root.py`: pass.
- `vf_assert_command_parity.py`: pass with unused-command warning for `prune_old_queue_jobs`.
- `vf_assert_frontend_config_provider.py`: pass.
- `vf_assert_autoapprove_contract.py`: pass.
- `vf_assert_no_lock_order_inversions.py`: pass.
- `vf_assert_external_service_status_checks.py`: pass.
- `vf_assert_export_filename_validation.py`: pass.
- `vf_assert_release_proof_packet.py`: pass.

Known non-static blocker:

- `npm run tauri build`: fails at AppImage `linuxdeploy` after binary/deb/rpm build.
