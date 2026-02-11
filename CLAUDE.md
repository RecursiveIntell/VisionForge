# CLAUDE.md — VisionForge

## What This Project Is

VisionForge is a Tauri 2 desktop app (Rust backend, React/TypeScript frontend) that bridges local LLMs (via Ollama) with Stable Diffusion (via ComfyUI) through a multi-agent prompt engineering pipeline. The user enters a simple idea, a chain of LLM agents refines it into optimized SD prompts, images are generated, and a gallery with AI-powered tagging/captioning manages the results.

The full specification is at `docs/SPEC.md`. Read it before writing any code. Re-read the relevant section before starting each phase.

## Critical Rules

### 1. Build Incrementally — Compile After Every File

After writing or modifying any Rust file, run `cargo check` before moving to the next file. After modifying any TypeScript file, ensure `npm run build` still passes. Do NOT write 10 files and then try to compile. You will lose track of errors.

### 2. One Module At A Time

Complete one module fully — types, implementation, tests, compilation — before starting the next. Do not scaffold stubs across the entire codebase and fill them in later. This leads to a half-built project where nothing works.

### 3. Test What You Build

Every database function gets a unit test with an in-memory SQLite instance. Every JSON parser gets tests with both valid and malformed input. Every API client function gets a test with mocked HTTP responses. Run `cargo test` after completing each module. Do not defer testing to "later."

### 4. No Premature Abstraction

Write concrete functions, not trait hierarchies. No generic frameworks, no builder patterns unless the code literally requires it. If a function is only called in one place, it doesn't need to be generic. Keep it simple and direct.

### 5. Error Handling Is Mandatory

Use `anyhow::Result` with context. Every error path must produce a message the frontend can display to the user. No `.unwrap()` in production code (tests are fine). No silent failures. If ComfyUI is offline, the user sees "Cannot connect to ComfyUI at http://...:8188 — is the service running?"

### 6. File Size Discipline

No single file should exceed 400 lines. If it does, split it. This is a hard rule — large files cause context window problems for both you and me. The `db/` module is split per-domain specifically for this reason.

### 7. Don't Fight the Spec

The spec defines the database schema, the pipeline stages, the API surface, and the UI structure. Follow it. If something in the spec seems wrong or suboptimal during implementation, add a `// SPEC_NOTE: <observation>` comment and implement it as specified anyway. We can refactor later. Do not redesign on the fly.

### 8. Frontend/Backend Contract

Tauri commands in `src-tauri/src/commands/` are **thin wrappers**. They validate input, call business logic from domain modules, and format the response. No business logic in command handlers. Commands map 1:1 to the API Reference section of the spec.

TypeScript invoke wrappers in `src/api/` are the **only** place `invoke()` is called. Components never call `invoke()` directly. They use hooks which use API wrappers.

### 9. Commit Hygiene

Commit after completing each module or meaningful unit of work. Commit message format:
```
feat(module): what was done

- Detail 1
- Detail 2
```

Good: `feat(db/images): implement image CRUD with gallery filtering`
Bad: `work in progress`

### 10. When You're Stuck

If you hit a problem you can't resolve in 3 attempts:
1. Add a `// TODO: <description of problem>` comment
2. Implement a minimal working fallback
3. Move on

Do not spend 20 minutes fighting a single issue. Flag it and continue.

---

## Technology Decisions (Non-Negotiable)

| Decision | Choice | Do NOT Use |
|----------|--------|------------|
| Desktop framework | Tauri 2 | Electron, web-only |
| Rust async runtime | tokio | async-std |
| HTTP client | reqwest | hyper directly, ureq |
| Database | rusqlite (bundled) | diesel, sqlx, sled |
| Serialization | serde + serde_json | manual parsing |
| Config format | TOML via toml crate | JSON, YAML |
| Image processing | image crate | imagemagick bindings |
| ZIP creation | zip crate | tar, flate2 directly |
| WebSocket | tokio-tungstenite | tungstenite (sync) |
| Frontend framework | React 18+ | Vue, Svelte, vanilla |
| CSS | Tailwind CSS | CSS modules, styled-components |
| State management | React hooks (useState, useReducer, useContext) | Redux, Zustand, MobX |
| Drag and drop | @dnd-kit/core | react-beautiful-dnd (deprecated) |

### Rust Dependencies (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
rusqlite = { version = "0.31", features = ["bundled"] }
anyhow = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
toml = "0.8"
image = "0.25"
base64 = "0.22"
tokio-tungstenite = "0.24"
zip = "2"
```

### Frontend Dependencies (package.json)

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^18",
    "react-dom": "^18",
    "@dnd-kit/core": "^6",
    "@dnd-kit/sortable": "^8"
  },
  "devDependencies": {
    "typescript": "^5",
    "tailwindcss": "^3",
    "autoprefixer": "^10",
    "postcss": "^8",
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "vite": "^5",
    "@vitejs/plugin-react": "^4"
  }
}
```

---

## Architecture Overview

```
Frontend (React/TS)              Backend (Rust/Tauri)
─────────────────               ──────────────────────
Components                      Commands (thin wrappers)
    │                               │
    ▼                               ▼
Hooks (state + logic)           Domain Modules
    │                           ├── pipeline/   (LLM orchestration)
    ▼                           ├── comfyui/    (SD generation)
API wrappers                    ├── queue/      (job management)
    │                           ├── gallery/    (file storage)
    ▼                           ├── ai/         (tagger/captioner)
invoke() ──────────────────►    └── config/     (TOML management)
                                    │
                                    ▼
                                Database (SQLite)
                                    │
                                    ▼
                                db/ modules (one per domain)
```

Data flows DOWN. Components → Hooks → API → invoke() → Commands → Domain → DB.
Events flow UP. Backend emits Tauri events → Hooks subscribe → Components re-render.

---

## Directory Structure

```
visionforge/
├── CLAUDE.md                          # This file
├── docs/
│   └── SPEC.md                        # Full project specification
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs                    # Entry point, command registration
│       ├── lib.rs                     # Module declarations
│       ├── state.rs                   # AppState: DB pool, config, queue handle
│       ├── types/
│       │   ├── mod.rs
│       │   ├── pipeline.rs
│       │   ├── generation.rs
│       │   ├── gallery.rs
│       │   ├── seeds.rs
│       │   ├── checkpoints.rs
│       │   ├── comparison.rs
│       │   ├── config.rs
│       │   └── queue.rs
│       ├── db/
│       │   ├── mod.rs                 # Connection pool setup
│       │   ├── migrations.rs          # Schema creation
│       │   ├── images.rs
│       │   ├── tags.rs
│       │   ├── seeds.rs
│       │   ├── checkpoints.rs
│       │   ├── comparisons.rs
│       │   └── queue.rs
│       ├── pipeline/
│       │   ├── mod.rs
│       │   ├── ollama.rs              # Ollama REST client
│       │   ├── engine.rs              # Pipeline orchestrator
│       │   ├── stages.rs              # Individual stage implementations
│       │   └── prompts.rs             # System prompt templates
│       ├── comfyui/
│       │   ├── mod.rs
│       │   ├── client.rs              # REST + WebSocket client
│       │   ├── workflow.rs            # Workflow JSON builder
│       │   └── models.rs              # Checkpoint discovery
│       ├── queue/
│       │   ├── mod.rs
│       │   ├── executor.rs            # Background queue runner
│       │   └── manager.rs             # Priority, reorder, pause
│       ├── gallery/
│       │   ├── mod.rs
│       │   ├── storage.rs             # File management, thumbnails
│       │   └── export.rs              # ZIP bundle creation
│       ├── ai/
│       │   ├── mod.rs
│       │   ├── tagger.rs
│       │   └── captioner.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── manager.rs             # TOML read/write
│       └── commands/
│           ├── mod.rs
│           ├── pipeline_cmds.rs
│           ├── comfyui_cmds.rs
│           ├── gallery_cmds.rs
│           ├── queue_cmds.rs
│           ├── seed_cmds.rs
│           ├── checkpoint_cmds.rs
│           ├── comparison_cmds.rs
│           ├── ai_cmds.rs
│           ├── config_cmds.rs
│           └── export_cmds.rs
├── src/
│   ├── App.tsx
│   ├── main.tsx
│   ├── types/                         # Mirror Rust types
│   │   └── index.ts                   # All types in one file (frontend is simpler)
│   ├── api/                           # Tauri invoke wrappers
│   │   ├── pipeline.ts
│   │   ├── comfyui.ts
│   │   ├── gallery.ts
│   │   ├── queue.ts
│   │   ├── seeds.ts
│   │   ├── checkpoints.ts
│   │   ├── comparison.ts
│   │   ├── ai.ts
│   │   ├── config.ts
│   │   └── export.ts
│   ├── hooks/
│   │   ├── usePipeline.ts
│   │   ├── useQueue.ts
│   │   ├── useGallery.ts
│   │   ├── useConfig.ts
│   │   └── useComparison.ts
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppShell.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── Header.tsx
│   │   ├── prompt-studio/
│   │   │   ├── PromptStudio.tsx
│   │   │   ├── IdeaInput.tsx
│   │   │   ├── PipelineStepper.tsx
│   │   │   ├── StageCard.tsx
│   │   │   ├── JudgeRanking.tsx
│   │   │   ├── ApprovalGate.tsx
│   │   │   └── PromptEditor.tsx
│   │   ├── gallery/
│   │   │   ├── GalleryView.tsx
│   │   │   ├── ImageGrid.tsx
│   │   │   ├── ImageCard.tsx
│   │   │   ├── Lightbox.tsx
│   │   │   ├── MetadataPanel.tsx
│   │   │   ├── FilterBar.tsx
│   │   │   └── LineageViewer.tsx
│   │   ├── queue/
│   │   │   ├── QueuePanel.tsx
│   │   │   ├── QueueItem.tsx
│   │   │   └── ProgressBar.tsx
│   │   ├── comparison/
│   │   │   ├── ComparisonView.tsx
│   │   │   ├── SliderOverlay.tsx
│   │   │   └── DiffTable.tsx
│   │   ├── seeds/
│   │   │   ├── SeedLibrary.tsx
│   │   │   ├── SeedCard.tsx
│   │   │   └── SeedPicker.tsx
│   │   ├── settings/
│   │   │   ├── SettingsPanel.tsx
│   │   │   ├── ConnectionSettings.tsx
│   │   │   ├── ModelAssignments.tsx
│   │   │   ├── QualityPresets.tsx
│   │   │   ├── PipelinePrompts.tsx
│   │   │   ├── CheckpointManager.tsx
│   │   │   └── HardwareSettings.tsx
│   │   └── shared/
│   │       ├── TagChips.tsx
│   │       ├── StarRating.tsx
│   │       ├── ConfirmDialog.tsx
│   │       └── LoadingSpinner.tsx
│   └── styles/
│       └── globals.css
├── package.json
└── tsconfig.json
```

---

## Build Phases

Follow these phases IN ORDER. Do not skip ahead. Complete each phase fully — code compiles, tests pass — before starting the next.

### Phase 1: Scaffold + Types + Database
1. Initialize Tauri 2 + React project
2. Set up all dependencies (Cargo.toml, package.json)
3. Create the full directory structure with empty mod.rs/mod.ts files
4. Define ALL Rust types in `src-tauri/src/types/`
5. Define ALL TypeScript types in `src/types/index.ts`
6. Implement `db/migrations.rs` with the complete schema
7. Implement `db/mod.rs` with connection pool setup
8. Implement `state.rs` with AppState struct
9. Implement `config/manager.rs` — read/write TOML, defaults
10. Wire up `main.rs` with database init, config load, empty command registration
11. **Gate: `cargo check` passes, `cargo test` passes, `cargo tauri dev` launches a window**

### Phase 2: Database Layer
1. Implement `db/images.rs` — all CRUD, filtering, pagination
2. Implement `db/tags.rs` — tag CRUD, image-tag associations
3. Implement `db/seeds.rs` — seed CRUD, checkpoint notes
4. Implement `db/checkpoints.rs` — profile CRUD, prompt terms, observations
5. Implement `db/comparisons.rs` — comparison CRUD
6. Implement `db/queue.rs` — job CRUD, ordering, status updates
7. **Gate: `cargo test` — all DB tests pass with in-memory SQLite**

### Phase 3: Ollama + Pipeline Engine
1. Implement `pipeline/ollama.rs` — Ollama REST client (generate, chat, tags)
2. Implement `pipeline/prompts.rs` — system prompt templates with variable substitution
3. Implement `pipeline/stages.rs` — all 5 stage functions with JSON parsing
4. Implement `pipeline/engine.rs` — orchestrator with stage bypass, lineage collection
5. Implement `commands/pipeline_cmds.rs` + `commands/config_cmds.rs`
6. **Gate: Unit tests pass. If Ollama is running, manual test with a real prompt works.**

### Phase 4: ComfyUI Integration
1. Implement `comfyui/client.rs` — REST client (queue, history, view, free)
2. Implement `comfyui/workflow.rs` — workflow JSON builder
3. Implement `comfyui/models.rs` — checkpoint discovery
4. Add WebSocket progress monitoring to client.rs
5. Implement `commands/comfyui_cmds.rs`
6. **Gate: Unit tests pass. If ComfyUI is running, can queue a generation and retrieve the image.**

### Phase 5: Queue System + Gallery Storage
1. Implement `queue/manager.rs` — add, reorder, cancel, pause/resume, persistence
2. Implement `queue/executor.rs` — background runner, PSU throttling, event emission
3. Implement `gallery/storage.rs` — save images, generate thumbnails, file management
4. Implement `commands/queue_cmds.rs` + `commands/gallery_cmds.rs`
5. **Gate: Can queue a job, executor picks it up, generates via ComfyUI, saves to gallery, emits completion event.**

### Phase 6: AI Features + Export
1. Implement `ai/tagger.rs` — vision model tagging via Ollama
2. Implement `ai/captioner.rs` — vision model captioning via Ollama
3. Implement `gallery/export.rs` — ZIP bundle with manifest
4. Implement remaining commands: `ai_cmds.rs`, `seed_cmds.rs`, `checkpoint_cmds.rs`, `comparison_cmds.rs`, `export_cmds.rs`
5. **Gate: All backend commands are implemented. `cargo test` passes. Full backend is functional.**

### Phase 7: Frontend — Layout + Settings
1. Set up Tailwind with dark theme in `globals.css`
2. Build shared components: TagChips, StarRating, ConfirmDialog, LoadingSpinner
3. Build AppShell, Sidebar, Header
4. Build all Settings panel components
5. Implement `useConfig` hook and `api/config.ts`
6. **Gate: App launches, can navigate to Settings, can configure endpoints and save.**

### Phase 8: Frontend — Prompt Studio
1. Build IdeaInput, PromptEditor
2. Build StageCard, PipelineStepper
3. Build JudgeRanking
4. Build ApprovalGate (with auto-approve checkbox, seed picker trigger)
5. Build PromptStudio (assembles all sub-components)
6. Implement `usePipeline` hook and `api/pipeline.ts`
7. **Gate: Can enter an idea, run the pipeline (with Ollama), see stage outputs, approve, and trigger generation.**

### Phase 9: Frontend — Gallery
1. Build ImageCard, ImageGrid
2. Build FilterBar
3. Build MetadataPanel
4. Build Lightbox (with keyboard nav)
5. Build LineageViewer
6. Build GalleryView (assembles all sub-components)
7. Implement `useGallery` hook and `api/gallery.ts`
8. **Gate: Generated images appear in gallery, can filter/sort, lightbox works, lineage viewer shows pipeline trace.**

### Phase 10: Frontend — Queue, Seeds, Comparison
1. Build QueuePanel, QueueItem, ProgressBar
2. Implement `useQueue` hook with Tauri event subscription
3. Build SeedLibrary, SeedCard, SeedPicker
4. Build ComparisonView, SliderOverlay, DiffTable
5. Implement `useComparison` hook
6. Wire up remaining API modules: `api/queue.ts`, `api/seeds.ts`, `api/checkpoints.ts`, `api/comparison.ts`, `api/ai.ts`, `api/export.ts`
7. **Gate: Full app is functional end-to-end.**

### Phase 11: Integration + Polish
1. End-to-end test: idea → pipeline → approve → generate → gallery → tag → caption
2. Test auto-approve flow
3. Test A/B comparison workflow
4. Test seed save → seed pick → iterate
5. Test checkpoint knowledge accumulation
6. Test queue priority and reordering
7. Test export bundle
8. Error handling pass: test with ComfyUI offline, Ollama offline, invalid models, disk full
9. Loading states and empty states for all views
10. Keyboard shortcuts

---

## Coding Patterns

### Tauri Command Pattern

```rust
// commands/gallery_cmds.rs
use crate::state::AppState;
use crate::types::gallery::{ImageEntry, GalleryFilter};
use crate::db;

#[tauri::command]
pub async fn get_gallery_images(
    state: tauri::State<'_, AppState>,
    filter: GalleryFilter,
) -> Result<Vec<ImageEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::images::list_images(&conn, &filter)
        .map_err(|e| format!("Failed to load gallery: {}", e))
}
```

### Database Pattern

```rust
// db/images.rs
use rusqlite::{Connection, params};
use anyhow::{Result, Context};
use crate::types::gallery::{ImageEntry, GalleryFilter};

pub fn list_images(conn: &Connection, filter: &GalleryFilter) -> Result<Vec<ImageEntry>> {
    let mut sql = String::from("SELECT * FROM images WHERE deleted = ?");
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        Box::new(filter.show_deleted.unwrap_or(false))
    ];
    
    if let Some(ref checkpoint) = filter.checkpoint {
        sql.push_str(" AND checkpoint = ?");
        param_values.push(Box::new(checkpoint.clone()));
    }
    // ... build query dynamically ...
    
    let mut stmt = conn.prepare(&sql).context("Failed to prepare image query")?;
    // ... execute and map rows ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();
        conn
    }
    
    #[test]
    fn test_insert_and_retrieve_image() {
        let conn = setup_test_db();
        // ... test ...
    }
}
```

### Frontend Hook Pattern

```typescript
// hooks/useGallery.ts
import { useState, useEffect, useCallback } from 'react';
import { getGalleryImages, deleteImage } from '../api/gallery';
import type { ImageEntry, GalleryFilter } from '../types';

export function useGallery(initialFilter: GalleryFilter) {
  const [images, setImages] = useState<ImageEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState(initialFilter);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getGalleryImages(filter);
      setImages(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load gallery');
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => { refresh(); }, [refresh]);

  return { images, loading, error, filter, setFilter, refresh };
}
```

### API Wrapper Pattern

```typescript
// api/gallery.ts
import { invoke } from '@tauri-apps/api/core';
import type { ImageEntry, GalleryFilter } from '../types';

export async function getGalleryImages(filter: GalleryFilter): Promise<ImageEntry[]> {
  return invoke('get_gallery_images', { filter });
}

export async function deleteImage(id: string): Promise<void> {
  return invoke('delete_image', { id });
}
```

---

## UI Design Guidelines

- **Dark theme only** for initial build. Colors: zinc-900 background, zinc-800 cards, zinc-700 borders, zinc-100 text, blue-500 primary accent, amber-500 warnings, red-500 destructive.
- **No animations** in initial build. Get it working, then add transitions.
- **Loading skeletons** over spinners where possible (gallery grid especially).
- **Toast notifications** for async completions (generation done, tag complete, etc.) — use a simple custom toast, no library.
- **Consistent spacing**: p-4 for cards, gap-4 for grids, p-6 for page padding.
- **All icons from lucide-react.** Do not add another icon library.

---

## Common Pitfalls to Avoid

1. **Don't use `serde(flatten)` with Tauri commands.** It causes serialization issues. Use explicit fields.

2. **Don't store images as BLOBs in SQLite.** Store file paths. Images live on disk.

3. **Don't make the queue executor a Tauri command.** It's a background task spawned in `main.rs` with `tokio::spawn`. It communicates via Tauri's event system.

4. **Don't poll ComfyUI from the frontend.** The backend executor handles polling/WebSocket and emits events to the frontend.

5. **Don't use `std::sync::Mutex` for the DB connection in async context.** Use `tokio::sync::Mutex` or `std::sync::Mutex` with `.lock()` in a `spawn_blocking` context.

6. **Don't parse Ollama/ComfyUI responses with strict typing.** These APIs return variable JSON structures. Parse defensively with `serde_json::Value` first, then extract fields with fallbacks.

7. **Don't hardcode localhost.** Always use the configured endpoints from config.toml.

8. **Don't forget `#[serde(rename_all = "camelCase")]` on types shared with the frontend.** Rust uses snake_case, TypeScript uses camelCase.

9. **ComfyUI's /prompt endpoint returns `{ prompt_id: string }`.** The /history endpoint returns `{ [prompt_id]: { outputs: { [node_id]: { images: [...] } } } }`. The image filenames are in `outputs.<SaveImage_node_id>.images[].filename`. Don't assume a simpler structure.

10. **Ollama's /api/generate returns streaming JSON lines by default.** Either set `"stream": false` in the request body for a single response, or collect all lines. For pipeline stages, use `"stream": false` — simplicity over UX here.
