# Remaining Delta

- Run live Ollama health, pipeline stage, and unload smoke tests.
- Run live ComfyUI health, queue status, txt2img generation, interrupt/free smoke tests.
- Fix AppImage `linuxdeploy` bundling failure or disable AppImage for release builds with an explicit packaging decision.
- Add non-2xx mock tests for Ollama/ComfyUI status handling.
- Add staged-file/DB rollback regression tests.
- Replace placeholder frontend `lint` and `test` scripts with real ESLint/Vitest or equivalent tooling.
- Decide whether selected concept should rerun prompt engineering; current implementation records lineage metadata only.
