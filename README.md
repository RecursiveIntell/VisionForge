# VisionForge

A Tauri 2 desktop app that bridges local LLMs (via Ollama) with Stable Diffusion (via ComfyUI) through a multi-agent prompt engineering pipeline. Enter a simple idea, a chain of LLM agents refines it into optimized SD prompts, images are generated, and a gallery with AI-powered tagging/captioning manages the results.

Built for single-GPU homelabs where every VRAM byte matters.

## Architecture

```
Frontend (React/TypeScript)          Backend (Rust/Tauri)
───────────────────────             ──────────────────────
Components                          Commands (thin wrappers)
    │                                   │
    ▼                                   ▼
Hooks (state + logic)               Domain Modules
    │                               ├── pipeline/   (LLM orchestration)
    ▼                               ├── comfyui/    (SD generation)
API wrappers                        ├── queue/      (job management)
    │                               ├── gallery/    (file storage)
    ▼                               ├── ai/         (tagger/captioner)
invoke() ──────────────────►        └── config/     (TOML management)
                                        │
                                        ▼
                                    Database (SQLite)
```

## Features

- **Prompt Studio** — Enter an idea, run it through a 5-stage LLM pipeline (Ideator → Composer → Judge → Prompt Engineer → Reviewer), edit the output, and generate
- **Smart Queue** — Priority-based generation queue with pause/resume, GPU cooldown, and live progress via Tauri events
- **Gallery** — Image grid with lightbox, filtering, sorting, star ratings, favorites, AI auto-tagging/captioning, and pipeline lineage viewer
- **A/B Comparison** — Select two images, slider overlay for visual diff, parameter diff table
- **Seed Library** — Save, search, and reuse seeds across generations
- **Checkpoint Knowledge** — Track checkpoint profiles, prompt term effects, and observations
- **Settings** — Configure Ollama/ComfyUI endpoints with health checks, model assignments per pipeline stage, quality presets, hardware throttling
- **Export** — ZIP bundle with images and metadata manifest

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2 (Rust) |
| Frontend | React 18 + TypeScript + Tailwind CSS |
| Database | SQLite via rusqlite (bundled) |
| LLM communication | Ollama REST API |
| SD communication | ComfyUI REST + WebSocket API |
| Image processing | image crate (Rust) |
| HTTP client | reqwest |
| Async runtime | tokio |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (18+)
- [Ollama](https://ollama.ai/) running locally (default: `http://localhost:11434`)
- [ComfyUI](https://github.com/comfyanonymous/ComfyUI) running locally (default: `http://localhost:8188`)

## Getting Started

```bash
# Install frontend dependencies
npm install

# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+1`–`Ctrl+6` | Navigate pages (Studio, Gallery, Queue, Seeds, Compare, Settings) |
| `Ctrl+Enter` | Submit idea in Prompt Studio |
| `Escape` | Close lightbox |
| `←` / `→` | Navigate images in lightbox |

## Project Structure

```
visionforge/
├── src-tauri/src/          # Rust backend
│   ├── commands/           # Tauri command handlers (thin wrappers)
│   ├── db/                 # SQLite database layer (one module per domain)
│   ├── pipeline/           # Ollama client + 5-stage pipeline engine
│   ├── comfyui/            # ComfyUI REST/WebSocket client
│   ├── queue/              # Background job executor
│   ├── gallery/            # Image storage + thumbnails + export
│   ├── ai/                 # Vision model tagger + captioner
│   ├── config/             # TOML config management
│   └── types/              # Shared Rust types
├── src/                    # React frontend
│   ├── api/                # Tauri invoke wrappers
│   ├── hooks/              # React hooks (state + logic)
│   ├── components/         # UI components
│   └── types/              # TypeScript types
└── docs/
    └── SPEC.md             # Full project specification
```

## Development

```bash
# Run Rust tests (129 tests)
cd src-tauri && cargo test

# Type-check and build frontend
npm run build

# Check Rust compilation
cd src-tauri && cargo check
```

## License

MIT
