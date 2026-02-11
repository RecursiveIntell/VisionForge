# VisionForge — Project Specification (v2)

## Overview

VisionForge is a desktop application that bridges local LLMs with Stable Diffusion through an intelligent multi-agent prompt engineering pipeline. A user provides a simple idea — "a cat sitting on a throne" — and a chain of specialized LLM agents collaborates to produce vivid, optimized SD prompts, generate images, and manage a gallery with AI-powered tagging and captioning.

Built for single-GPU homelabs where every VRAM byte matters.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                          VisionForge UI                              │
│                   (Tauri + React/TypeScript)                         │
├──────────┬──────────┬──────────┬──────────┬─────────────────────────┤
│  Prompt  │  Smart   │ Gallery  │  A/B     │ Settings/Model Config   │
│  Studio  │  Queue   │  View    │ Compare  │ + Checkpoint Knowledge  │
└────┬─────┴────┬─────┴────┬─────┴────┬─────┴─────────────┬───────────┘
     │          │          │          │                    │
     ▼          ▼          ▼          ▼                    ▼
┌─────────┐ ┌────────┐ ┌──────────┐ ┌──────────┐  ┌──────────────────┐
│ Prompt  │ │ComfyUI │ │ Gallery  │ │Comparison│  │  Config Manager   │
│Pipeline │ │  API   │ │  Store   │ │  Engine  │  │ + Checkpoint DB   │
│ Engine  │ │Client  │ │(SQLite)  │ │          │  │ (future RAG)      │
└────┬────┘ └────┬───┘ └────┬─────┘ └──────────┘  └──────────────────┘
     │          │          │
     ▼          ▼          ▼
┌─────────┐ ┌────────┐ ┌──────────┐
│ Ollama  │ │ComfyUI │ │ AI Tag/  │
│  API    │ │ Server │ │ Caption  │
│(LLMs)  │ │  (SD)  │ │ Pipeline │
└─────────┘ └────────┘ └──────────┘
```

### Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Desktop shell | **Tauri 2 (Rust)** | Familiar from Palisade, lightweight, native feel |
| Frontend | **React + TypeScript + Tailwind** | Fast UI iteration, component reuse |
| Backend logic | **Rust (Tauri commands)** | Performance, type safety, async |
| Database | **SQLite via rusqlite** | Gallery metadata, tags, captions, generation history, checkpoint knowledge |
| LLM communication | **Ollama REST API** | Already running, model switching is a simple parameter |
| SD communication | **ComfyUI REST API** | Workflow-as-JSON, headless, queue-based |
| Image handling | **image crate (Rust)** + thumbnails | Gallery grid, zoom, metadata |

### Why Tauri over Electron/Web

- You already built Palisade with Tauri, so the toolchain is warm.
- Rust backend handles file I/O, image processing, and SQLite without a separate server process.
- ~10MB binary vs 200MB+ Electron. Matters on a homelab.
- No browser overhead eating into your limited system resources.

---

## Feature Specification

### 1. Prompt Studio (Main View)

The creative workspace where ideas become SD prompts.

#### Input

- A simple text field: "a cat sitting on a throne in a ruined castle"
- Optional style hints dropdown (photorealistic, anime, oil painting, etc.)
- Optional aspect ratio selector (1:1, 2:3, 3:2, 16:9)
- "Number of concepts to explore" slider (1-10, default 5)

#### The Prompt Pipeline

Five distinct LLM stages, each assignable to a different model. Each stage can be individually toggled on/off via bypass switches in the Prompt Studio header.

```
User Input
    │
    ▼
┌──────────────────────────────────────────┐
│  Stage 1: IDEATOR              [ON/OFF]  │
│  Role: Creative divergence               │
│  Input: User's simple prompt             │
│  Output: N distinct creative directions  │
│  Model: Configurable (e.g. mistral:7b)   │
│                                          │
│  Example output:                         │
│  1. "Medieval fantasy throne room with   │
│     a Persian cat as king, crumbling     │
│     stone, shafts of light"             │
│  2. "Post-apocalyptic twist — cat on a   │
│     car seat 'throne' in ruins"         │
│  3. "Miniature/tilt-shift — real cat on  │
│     a dollhouse throne"                 │
│  4. "Dark gothic — black cat, iron       │
│     throne, candlelight, cobwebs"       │
│  5. "Whimsical storybook — illustrated   │
│     cat king with crown and scepter"    │
└──────────────┬───────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────┐
│  Stage 2: COMPOSER             [ON/OFF]  │
│  Role: Detail enrichment                 │
│  Input: Each idea from Stage 1           │
│  Output: Rich scene descriptions with    │
│    lighting, mood, materials, camera     │
│    angle, color palette, atmosphere      │
│  Model: Configurable (e.g. llama3.1:8b)  │
│                                          │
│  Adds: specific materials (weathered     │
│  granite, velvet cushion, tarnished      │
│  gold), lighting (volumetric god rays,   │
│  warm amber torchlight), camera (low     │
│  angle looking up at the cat, shallow    │
│  DOF), emotional tone                    │
└──────────────┬───────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────┐
│  Stage 3: JUDGE                [ON/OFF]  │
│  Role: Quality evaluation & ranking      │
│  Input: All composed descriptions        │
│  Output: Ranked list with reasoning      │
│  Model: Configurable (e.g. qwen2.5:7b)  │
│                                          │
│  Evaluates on:                           │
│  - Visual clarity (can SD render this?)  │
│  - Composition coherence                 │
│  - Prompt-friendliness (avoidance of     │
│    abstract concepts SD struggles with)  │
│  - Originality vs. the user's intent    │
│  - Technical feasibility on SD 1.5       │
└──────────────┬───────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────┐
│  Stage 4: PROMPT ENGINEER      [ON/OFF]  │
│  Role: SD-native prompt construction     │
│  Input: Top-ranked description(s)        │
│  Output: Optimized positive + negative   │
│    prompts in SD syntax                  │
│  Model: Configurable                     │
│                                          │
│  ** Checkpoint-aware ** — receives the   │
│  active checkpoint name + behavioral     │
│  notes from the Checkpoint Knowledge DB  │
│  to tailor prompt syntax and style.      │
│                                          │
│  Knows SD prompt grammar:                │
│  - (emphasis:1.3) weighting              │
│  - Comma-separated tag style             │
│  - Quality boosters (masterpiece, best   │
│    quality, highly detailed)             │
│  - Effective negative prompt patterns    │
│  - Model-specific quirks (SD1.5 vs SDXL) │
│                                          │
│  Output example:                         │
│  Positive: "masterpiece, best quality,   │
│    (persian cat:1.3) sitting on ornate   │
│    stone throne, medieval castle throne  │
│    room, crumbling walls, (volumetric    │
│    god rays:1.2), dust particles,        │
│    tarnished gold crown, velvet red      │
│    cushion, low angle shot, cinematic    │
│    lighting, 8k, highly detailed"        │
│  Negative: "lowres, bad anatomy, bad     │
│    hands, text, watermark, deformed,     │
│    blurry, disfigured, extra limbs,      │
│    fused fingers, gross proportions"     │
└──────────────┬───────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────┐
│  Stage 5: REVIEWER             [ON/OFF]  │
│  Role: Final sanity check                │
│  Input: Final prompts + original intent  │
│  Output: Approval, or suggested edits    │
│  Model: Configurable                     │
│                                          │
│  Catches:                                │
│  - Prompt drift from original idea       │
│  - Conflicting terms                     │
│  - Over-stuffed prompts (token limits)   │
│  - Missing critical elements from the    │
│    user's original request               │
└──────────────────────────────────────────┘
```

**Stage bypass logic:** When a stage is toggled off, the pipeline skips it and passes its input directly to the next enabled stage. Minimum viable pipeline is just the Prompt Engineer (Stage 4) — user's raw idea goes straight to SD prompt construction.

#### Pipeline Execution UI

- Each stage runs sequentially (no concurrent GPU use needed — these are all LLM calls via Ollama).
- A vertical stepper/timeline shows progress through stages.
- Each stage's output is visible and expandable in the UI.
- The user can intervene at any stage: edit, re-roll, skip, or manually override.
- At the Judge stage, the user sees all ranked options and can pick differently.
- At the Prompt Engineer stage, the user can manually edit the final positive/negative prompts before generation.
- **Full lineage** is preserved for every generation (see Section 7: Prompt Lineage).

#### Approval Gate

Before sending to ComfyUI:
- Side-by-side display of positive and negative prompts.
- Editable text fields for both.
- Quality settings panel (see Generation Settings below).
- **"Generate" button** — only fires after explicit approval.
- **"Regenerate Pipeline"** — re-runs from any stage with modifications.
- **"Batch" option** — generate the top N ranked ideas (queued via Smart Queue).
- **☑ Auto-approve checkbox** — when enabled, the pipeline output goes directly to the Smart Queue without stopping at the approval gate. Intended for batch/fire-and-forget workflows. The checkbox state persists across sessions (stored in config). A small warning indicator shows when auto-approve is active so the user doesn't forget. Auto-approved generations are flagged in the gallery metadata so you can distinguish them from manually reviewed ones.

---

### 2. Generation Engine

#### ComfyUI Integration

Communication with ComfyUI via its REST API at the configured endpoint.

```
POST /prompt          — Queue a workflow for execution
GET  /history/{id}    — Poll for completion
GET  /view?filename=  — Retrieve generated image
POST /free            — Release VRAM (for GPU arbitration)
WS   /ws              — WebSocket for real-time progress
```

The application constructs a ComfyUI workflow JSON programmatically — no need for the user to touch ComfyUI's node editor. The workflow template is stored internally:

```json
{
  "3": {
    "class_type": "KSampler",
    "inputs": {
      "seed": "<random or user-specified>",
      "steps": "<from settings>",
      "cfg": "<from settings>",
      "sampler_name": "<from settings>",
      "scheduler": "<from settings>",
      "denoise": 1,
      "model": ["4", 0],
      "positive": ["6", 0],
      "negative": ["7", 0],
      "latent_image": ["5", 0]
    }
  },
  "4": {
    "class_type": "CheckpointLoaderSimple",
    "inputs": { "ckpt_name": "<from settings>" }
  },
  "5": {
    "class_type": "EmptyLatentImage",
    "inputs": {
      "width": "<from settings>",
      "height": "<from settings>",
      "batch_size": 1
    }
  },
  "6": {
    "class_type": "CLIPTextEncode",
    "inputs": {
      "text": "<positive prompt from pipeline>",
      "clip": ["4", 1]
    }
  },
  "7": {
    "class_type": "CLIPTextEncode",
    "inputs": {
      "text": "<negative prompt from pipeline>",
      "clip": ["4", 1]
    }
  },
  "8": {
    "class_type": "VAEDecode",
    "inputs": {
      "samples": ["3", 0],
      "vae": ["4", 2]
    }
  },
  "9": {
    "class_type": "SaveImage",
    "inputs": {
      "filename_prefix": "VisionForge",
      "images": ["8", 0]
    }
  }
}
```

#### Generation Settings (User-Configurable)

| Setting | Default | Range/Options |
|---------|---------|---------------|
| Checkpoint | (user selects) | Pulled from ComfyUI's available models |
| Width | 512 | 256–768 (SD1.5 safe range) |
| Height | 512 | 256–768 |
| Steps | 20 | 1–50 |
| CFG Scale | 7.0 | 1.0–20.0 |
| Sampler | euler_ancestral | euler, euler_ancestral, dpmpp_2m, dpmpp_sde, etc. |
| Scheduler | karras | normal, karras, exponential, sgm_uniform |
| Seed | Random | -1 (random) or specific value, or pick from Seed Library |
| Batch size | 1 | 1–4 (warn user about PSU above 2) |

---

### 3. Smart Queue

Replaces the simple PSU-aware queue from v1. This is the central job manager for all generation work.

#### Queue Features

- **Priority levels:** High, Normal, Low. New jobs default to Normal.
- **Drag-to-reorder** in the queue UI — manually reprioritize any pending job.
- **Add while running** — submit new ideas to the pipeline while a generation is in progress. They enter the queue at their assigned priority and wait their turn.
- **Queue visibility panel** — always-visible sidebar or bottom drawer showing:
  - Currently generating (with WebSocket progress bar from ComfyUI)
  - Pending jobs with their priority, estimated wait time, and prompt preview
  - Completed jobs (last N, clickable to jump to gallery)
- **Batch pipeline support** — when running the pipeline with auto-approve on batch, each approved concept becomes its own queue entry at Normal priority. A manually submitted idea at High priority will jump ahead of the batch.
- **Cancel/pause** — cancel any pending job, or pause the entire queue (finish current generation, then hold).

#### PSU-Aware Throttling

Built into the queue executor, not a separate system:

- Configurable **cooldown period** between generations (default: 30 seconds).
- **Max consecutive generations** before forced cooldown (default: 5).
- Optional Home Assistant power monitoring integration — the queue executor checks wattage before starting the next job and waits if draw is too high.
- Cooldown timer visible in the queue UI.

#### Queue Persistence

The queue survives app restarts. Pending jobs are stored in SQLite and restored on launch. A job that was mid-generation when the app closed is re-queued at High priority.

```toml
[hardware]
cooldown_seconds = 30
max_consecutive_generations = 5
enable_ha_power_monitoring = false
ha_entity_id = "sensor.gpu_power_draw"
ha_max_watts = 180
```

---

### 4. Gallery

All generated images are stored and cataloged.

#### Storage Structure

```
~/.visionforge/
├── config.toml           # All settings
├── gallery.db            # SQLite database
├── images/
│   ├── originals/        # Full-res generations
│   │   └── 2026-02-10_18-30-45_abc123.png
│   └── thumbnails/       # 256px thumbnails for grid
│       └── 2026-02-10_18-30-45_abc123_thumb.jpg
```

#### Gallery UI Features

- **Grid view** with thumbnails, infinite scroll.
- **Lightbox** — click to expand with full metadata panel.
- **Filter/search** by tags, date range, prompt text, rating, favorites, checkpoint used, auto-approved vs. manually approved.
- **Batch operations** — select multiple, delete, tag, export.
- **Sort** by date, rating, or shuffle.
- **Delete** — soft delete with "trash" view and permanent delete option.
- **Re-use prompt** — click any image to load its prompts back into Prompt Studio for iteration.
- **View lineage** — button to open the full pipeline trace for any image (see Section 7).
- **Iterate with seed** — one-click to load the image's seed into Prompt Studio with the same prompts, ready for variation tweaks.
- **Compare** — select 2 images to open them in A/B Comparison Mode (see Section 8).
- **Export bundle** — export selected images as a ZIP with a JSON/CSV manifest containing all prompts, settings, tags, captions, and lineage data.

---

### 5. AI Tagger

Analyzes images and suggests descriptive tags.

#### Pipeline

```
Image → Ollama Vision Model (e.g. llava:7b, moondream) 
      → Structured tag extraction 
      → Confidence scoring 
      → User review/edit
```

#### Implementation

Send the image as base64 to a vision-capable model via Ollama with a system prompt like:

```
Analyze this image and return a JSON array of descriptive tags.
Categories: subject, style, mood, color_palette, setting, objects, lighting, composition.
Format: [{"tag": "persian cat", "category": "subject", "confidence": 0.95}, ...]
Return ONLY valid JSON.
```

#### UX

- **Auto-tag button** on each image in gallery.
- **Batch auto-tag** — process multiple images (queued via Smart Queue to avoid GPU strain).
- Tags appear as colored chips below each image.
- Click any tag to edit or delete it.
- Click "+" to manually add tags.
- Tag suggestions autocomplete from existing tags in the database.

---

### 6. AI Captioner

Generates natural-language descriptions of images.

#### Pipeline

```
Image + Generation metadata (prompts, settings)
      → Ollama Vision Model
      → Caption generation
      → User review/edit
```

#### System Prompt

```
Describe this image in 1-3 natural sentences. Be specific about 
subjects, actions, setting, mood, and artistic style. Do not 
mention that this is AI-generated. Write as if describing a 
photograph or painting for someone who cannot see it.

Context (the prompt used to generate this image):
{positive_prompt}
```

#### UX

- **Caption button** on each image.
- Inline editable text field — AI fills it, user can modify.
- `caption_edited` flag tracks whether the user changed it.
- **Batch caption** option for bulk processing.
- Captions are searchable in the gallery.

---

### 7. Prompt Lineage Tracking

Every image stores its complete creative genealogy — from the user's original spark through every pipeline stage's output to the final SD prompt. This isn't just metadata; it's a learning tool.

#### What's Stored

For each generation, the full pipeline trace is saved as a structured JSON document:

```json
{
  "original_idea": "a cat sitting on a throne in a ruined castle",
  "pipeline_config": {
    "stages_enabled": [true, true, true, true, false],
    "models_used": {
      "ideator": "mistral:7b",
      "composer": "llama3.1:8b",
      "judge": "qwen2.5:7b",
      "prompt_engineer": "mistral:7b"
    }
  },
  "stages": {
    "ideator": {
      "input": "a cat sitting on a throne in a ruined castle",
      "output": ["concept 1...", "concept 2...", "concept 3...", "concept 4...", "concept 5..."],
      "duration_ms": 4200,
      "model": "mistral:7b",
      "tokens_in": 42,
      "tokens_out": 380
    },
    "composer": {
      "input_concept_index": 3,
      "input": "Dark gothic — black cat, iron throne, candlelight, cobwebs",
      "output": "A sleek black cat with amber eyes sits upon a wrought-iron throne...",
      "duration_ms": 5100,
      "model": "llama3.1:8b",
      "tokens_in": 85,
      "tokens_out": 210
    },
    "judge": {
      "input": ["all 5 composed descriptions..."],
      "output": [
        {"rank": 1, "concept_index": 3, "score": 92, "reasoning": "..."},
        {"rank": 2, "concept_index": 0, "score": 87, "reasoning": "..."}
      ],
      "duration_ms": 3800,
      "model": "qwen2.5:7b"
    },
    "prompt_engineer": {
      "input": "A sleek black cat with amber eyes...",
      "checkpoint_context": "DreamShaper v8 — responds well to cinematic lighting terms, prefers (emphasis:weight) over [[nested brackets]]",
      "output": {
        "positive": "masterpiece, best quality, ...",
        "negative": "lowres, bad anatomy, ..."
      },
      "duration_ms": 2900,
      "model": "mistral:7b"
    }
  },
  "user_edits": {
    "prompt_edited": true,
    "edit_diff": {
      "positive_added": ["film grain"],
      "positive_removed": ["8k"],
      "negative_added": [],
      "negative_removed": []
    }
  },
  "auto_approved": false,
  "generation_settings": {
    "checkpoint": "dreamshaper_8.safetensors",
    "seed": 1847293,
    "steps": 25,
    "cfg": 7.5,
    "sampler": "dpmpp_2m",
    "scheduler": "karras",
    "width": 512,
    "height": 768
  }
}
```

#### Lineage Viewer UI

Accessible from the gallery lightbox via a "View Lineage" button:

- **Visual timeline** showing each pipeline stage as a card.
- Each card shows the stage name, model used, input → output, and duration.
- The user's edits (if any) are highlighted as a diff between pipeline output and what was actually sent to SD.
- **"What worked" annotations** — the user can add a free-text note to any lineage record: "This concept direction works great for gothic scenes" or "The judge was wrong here, concept 2 was better." These annotations feed into the Checkpoint Knowledge DB over time.

#### Analytics Value

Over time, the lineage database enables queries like:
- "Which Ideator model produces concepts that score highest with the Judge?"
- "How often do I override the Judge's top pick?"
- "What's the average token count of prompts that produce 5-star images?"
- "Which pipeline stage takes the longest?"

These are just SQL queries against structured JSON. No ML required — just data you're already collecting.

---

### 8. A/B Comparison Mode

A dedicated view for side-by-side image comparison. Answers the question: "What does changing this one variable actually do?"

#### Access Points

- **From gallery:** Select two images → "Compare" button.
- **From Prompt Studio:** "A/B Generate" button that creates two queue entries with a specified variable difference.
- **From seed iteration:** Automatically offered when generating a variation of an existing image.

#### Comparison UI

```
┌──────────────────────────────────────────────┐
│              A/B Comparison Mode              │
├──────────────────┬───────────────────────────┤
│                  │                           │
│     Image A      │◄── drag slider ──►│ Image B │
│                  │                           │
│                  │                           │
├──────────────────┴───────────────────────────┤
│  Differences:                                │
│  ┌─────────────────┬───────────────────────┐ │
│  │  Setting        │  A          │  B       │ │
│  │  Checkpoint     │  dreamshaper│  deliber │ │
│  │  CFG Scale      │  7.0        │  7.0     │ │
│  │  Sampler        │  euler_a    │  dpmpp   │ │
│  │  Seed           │  1847293    │  1847293 │ │
│  └─────────────────┴────────────┴──────────┘ │
│  ★ Same seed, different checkpoint           │
│                                              │
│  [Save comparison note]  [Add to checkpoint  │
│                           knowledge DB]      │
└──────────────────────────────────────────────┘
```

#### Features

- **Drag slider** overlay — like Photoshop's difference view. Drag left/right to reveal each image.
- **Auto-diff table** — automatically highlights which generation parameters differ between the two images.
- **Fixed variable, changed variable** — the UI calls out what was held constant and what was varied, making it a proper controlled experiment.
- **Comparison notes** — free-text field to record observations: "euler_ancestral gives softer textures on this checkpoint."
- **Push to Checkpoint Knowledge DB** — one-click to save the comparison observation as a behavioral note for the relevant checkpoint(s). This is how the knowledge base grows organically.
- **Linked pairs** — comparisons are stored and browsable. "Show me all my A/B tests for DreamShaper" is a valid gallery filter.

#### A/B Quick-Generate Workflow

From the Prompt Studio, instead of "Generate":
1. Click "A/B Generate."
2. A panel opens showing the current settings as Column A.
3. Column B starts as a copy — user changes one variable (checkpoint, CFG, sampler, seed, etc.).
4. Both jobs are submitted to the Smart Queue as a linked pair at High priority.
5. When both complete, the A/B Comparison view opens automatically.

---

### 9. Seed Library

Seeds aren't just numbers — they're latent space coordinates that produce specific structural compositions. The Seed Library treats them as reusable assets.

#### Features

- **Save seed** from any generated image with one click.
- **Comment field** — required on save. Forces the user to describe what this seed does: "Strong center composition, single subject, dramatic side lighting" or "Chaotic multi-element, good for busy scenes" or "Consistent portrait framing, 3/4 view."
- **Tag seeds** — reuse the same tag system from the gallery. A seed can be tagged "portrait", "landscape", "symmetric", etc.
- **Checkpoint association** — seeds behave differently across checkpoints. Each seed entry records which checkpoint it was discovered on, and comparison notes can be added for other checkpoints.
- **Browse & pick** — in the Generation Settings panel, instead of entering a seed manually, click "Seed Library" to browse saved seeds with their comments and sample thumbnails.
- **Seed variation** — from any saved seed, click "Iterate" to load it into Prompt Studio with a `denoise` variation slider (0.0–1.0). At 0.0 you get the exact same structure; at higher values the composition drifts further. This maps to the KSampler's denoise parameter with the seed pinned.

#### Database

```sql
CREATE TABLE seeds (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_value  INTEGER NOT NULL,
    comment     TEXT NOT NULL,               -- What this seed does
    checkpoint  TEXT,                         -- Checkpoint it was discovered on
    sample_image_id TEXT REFERENCES images(id), -- Thumbnail reference
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE seed_tags (
    seed_id  INTEGER REFERENCES seeds(id),
    tag_id   INTEGER REFERENCES tags(id),
    PRIMARY KEY (seed_id, tag_id)
);

CREATE TABLE seed_checkpoint_notes (
    seed_id     INTEGER REFERENCES seeds(id),
    checkpoint  TEXT NOT NULL,
    note        TEXT NOT NULL,               -- How this seed behaves on this checkpoint
    sample_image_id TEXT REFERENCES images(id),
    PRIMARY KEY (seed_id, checkpoint)
);
```

---

### 10. Checkpoint Knowledge Database

This is the long-game feature. Every checkpoint has quirks — which prompt terms it responds to, what its failure modes are, which samplers pair well with it. Right now that knowledge lives in community forums, Reddit threads, and your own trial and error. VisionForge makes it structured, queryable, and eventually vectorizable for RAG.

#### Data Model

```sql
CREATE TABLE checkpoints (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    filename    TEXT UNIQUE NOT NULL,        -- e.g. "dreamshaper_8.safetensors"
    display_name TEXT,                       -- e.g. "DreamShaper v8"
    base_model  TEXT,                        -- "SD 1.5", "SDXL", etc.
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- Structured behavioral profile
    strengths       TEXT,    -- JSON array: ["photorealism", "cinematic lighting", "portraits"]
    weaknesses      TEXT,    -- JSON array: ["text rendering", "hands", "multiple characters"]
    preferred_cfg   REAL,    -- Observed best CFG range center
    cfg_range_low   REAL,    -- Lower bound
    cfg_range_high  REAL,    -- Upper bound
    preferred_sampler TEXT,  -- What tends to work best
    preferred_scheduler TEXT,
    optimal_resolution TEXT, -- e.g. "512x768" or "512x512"
    
    -- Free-form knowledge
    notes           TEXT     -- General observations, tips, tricks
);

CREATE TABLE checkpoint_prompt_terms (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    checkpoint_id   INTEGER REFERENCES checkpoints(id),
    term            TEXT NOT NULL,           -- e.g. "cinematic lighting"
    effect          TEXT NOT NULL,           -- What it actually does on this model
    strength        TEXT CHECK(strength IN ('strong', 'moderate', 'weak', 'broken')),
    example_image_id TEXT REFERENCES images(id),  -- Proof
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE checkpoint_observations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    checkpoint_id   INTEGER REFERENCES checkpoints(id),
    observation     TEXT NOT NULL,           -- Free-form note
    source          TEXT CHECK(source IN ('user', 'ab_comparison', 'pipeline_note')),
    comparison_id   TEXT,                    -- Link to A/B comparison if applicable
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

#### How Knowledge Accumulates

Knowledge enters the Checkpoint DB from multiple natural touchpoints — the user never has to sit down and "fill out a form":

1. **First use auto-scaffold.** When a checkpoint is first selected, VisionForge creates a skeleton entry with the filename and base model. Everything else starts empty.

2. **A/B Comparison push.** After comparing two images (same prompt, different checkpoints), the "Add to Checkpoint Knowledge DB" button saves the comparison note as a `checkpoint_observation` for both checkpoints involved.

3. **Gallery rating feedback loop.** When you rate an image 4-5 stars, VisionForge logs the generation settings as "known good" for that checkpoint. Over time, the `preferred_cfg`, `preferred_sampler`, etc. fields are populated by computing the mode/median of high-rated generations. This happens automatically in the background.

4. **Prompt term discovery.** From the lineage viewer, you can highlight a specific term in the positive prompt and annotate it: "this term produces strong volumetric lighting on DreamShaper but does nothing on Deliberate." That becomes a `checkpoint_prompt_terms` entry.

5. **Manual notes.** A "Checkpoint Notes" panel in Settings where you can add free-form observations for any checkpoint.

6. **Pipeline integration.** The Prompt Engineer stage receives the active checkpoint's knowledge profile as additional context in its system prompt. It uses this to tailor prompt syntax.

#### Prompt Engineer Integration

When the Prompt Engineer stage runs, its system prompt is dynamically augmented:

```
You are an expert Stable Diffusion prompt engineer. Convert this scene 
description into optimized positive and negative prompts.

TARGET CHECKPOINT: {checkpoint_name}
Base model: {base_model}

CHECKPOINT BEHAVIORAL PROFILE:
Strengths: {strengths}
Weaknesses: {weaknesses}
Preferred CFG: {cfg_range_low}–{cfg_range_high}
Preferred sampler: {preferred_sampler}
Notes: {notes}

KNOWN EFFECTIVE TERMS FOR THIS CHECKPOINT:
{term_list_with_effects}

Use this knowledge to craft prompts that play to this checkpoint's 
strengths and avoid its weaknesses. Prefer terms known to be effective 
on this specific model.

[... rest of standard Prompt Engineer instructions ...]
```

#### Future: RAG Vectorization

The entire Checkpoint Knowledge DB is designed to be vectorized. The schema is deliberately structured so that each row in `checkpoint_prompt_terms` and `checkpoint_observations` is a self-contained knowledge chunk suitable for embedding.

Future pipeline:
```
checkpoint_prompt_terms rows + checkpoint_observations rows
    → Chunked text: "{term} on {checkpoint}: {effect} (strength: {strength})"
    → Embedding model (e.g. nomic-embed-text via Ollama)
    → Vector store (ChromaDB, Qdrant, or even SQLite-vec)
    → RAG retrieval at Prompt Engineer stage
```

This replaces the static system prompt injection with dynamic retrieval: instead of dumping the entire checkpoint profile into the LLM context, you embed the scene description, retrieve the most relevant checkpoint knowledge chunks, and inject only those. This scales to hundreds of observations per checkpoint without blowing up context windows.

**The key insight:** By building the structured knowledge DB now and collecting data through natural usage, you're creating a training/embedding dataset from day one. The RAG layer is a drop-in upgrade later — the data shape is already correct.

---

### 11. Settings Panel

#### Connection Settings

```toml
[comfyui]
endpoint = "http://192.168.50.69:8188"

[ollama]
endpoint = "http://localhost:11434"
```

#### Model Assignments

Each pipeline stage gets its own model selector. The UI pulls available models from `GET /api/tags` on the Ollama endpoint.

```toml
[models]
ideator         = "mistral:7b"
composer        = "llama3.1:8b"
judge           = "qwen2.5:7b"
prompt_engineer = "mistral:7b"
reviewer        = "qwen2.5:7b"    # Optional, can be disabled
tagger          = "llava:7b"       # Must be vision-capable
captioner       = "llava:7b"       # Must be vision-capable
```

#### Pipeline Stage Toggles

```toml
[pipeline]
enable_ideator        = true
enable_composer       = true
enable_judge          = true
enable_prompt_engineer = true   # Cannot be disabled — minimum viable stage
enable_reviewer       = false   # Off by default
auto_approve          = false   # Approval gate bypass
```

#### Quality Presets

```toml
[presets.quick_draft]
steps = 12
cfg = 7
width = 512
height = 512
sampler = "euler_ancestral"
scheduler = "normal"

[presets.quality]
steps = 25
cfg = 7.5
width = 512
height = 768
sampler = "dpmpp_2m"
scheduler = "karras"

[presets.max_effort]
steps = 40
cfg = 8
width = 768
height = 768
sampler = "dpmpp_sde"
scheduler = "karras"
```

Users can create/save/delete custom presets.

---

## Database Schema (Complete)

```sql
-- ============================================
-- Core Gallery
-- ============================================

CREATE TABLE images (
    id              TEXT PRIMARY KEY,           -- UUID
    filename        TEXT NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- Generation metadata
    positive_prompt TEXT,
    negative_prompt TEXT,
    original_idea   TEXT,                       -- The user's initial input
    checkpoint      TEXT,
    width           INTEGER,
    height          INTEGER,
    steps           INTEGER,
    cfg_scale       REAL,
    sampler         TEXT,
    scheduler       TEXT,
    seed            INTEGER,
    
    -- Pipeline metadata
    pipeline_log    TEXT,                       -- JSON: full lineage trace (see Section 7)
    selected_concept INTEGER,                   -- Which concept the judge/user picked
    auto_approved   BOOLEAN DEFAULT FALSE,      -- Was approval gate bypassed?
    
    -- AI-generated metadata
    caption         TEXT,
    caption_edited  BOOLEAN DEFAULT FALSE,
    
    -- User metadata
    rating          INTEGER,                    -- 1-5 stars, NULL if unrated
    favorite        BOOLEAN DEFAULT FALSE,
    deleted         BOOLEAN DEFAULT FALSE,      -- Soft delete
    user_note       TEXT                        -- Free-form annotation
);

-- ============================================
-- Tags (shared across images and seeds)
-- ============================================

CREATE TABLE tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT UNIQUE NOT NULL
);

CREATE TABLE image_tags (
    image_id    TEXT REFERENCES images(id) ON DELETE CASCADE,
    tag_id      INTEGER REFERENCES tags(id),
    source      TEXT CHECK(source IN ('ai', 'user')),
    confidence  REAL,
    PRIMARY KEY (image_id, tag_id)
);

-- ============================================
-- Seed Library
-- ============================================

CREATE TABLE seeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_value      INTEGER NOT NULL,
    comment         TEXT NOT NULL,
    checkpoint      TEXT,
    sample_image_id TEXT REFERENCES images(id),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE seed_tags (
    seed_id  INTEGER REFERENCES seeds(id) ON DELETE CASCADE,
    tag_id   INTEGER REFERENCES tags(id),
    PRIMARY KEY (seed_id, tag_id)
);

CREATE TABLE seed_checkpoint_notes (
    seed_id         INTEGER REFERENCES seeds(id) ON DELETE CASCADE,
    checkpoint      TEXT NOT NULL,
    note            TEXT NOT NULL,
    sample_image_id TEXT REFERENCES images(id),
    PRIMARY KEY (seed_id, checkpoint)
);

-- ============================================
-- Checkpoint Knowledge Database
-- ============================================

CREATE TABLE checkpoints (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    filename            TEXT UNIQUE NOT NULL,
    display_name        TEXT,
    base_model          TEXT,
    created_at          DATETIME DEFAULT CURRENT_TIMESTAMP,
    strengths           TEXT,       -- JSON array
    weaknesses          TEXT,       -- JSON array
    preferred_cfg       REAL,
    cfg_range_low       REAL,
    cfg_range_high      REAL,
    preferred_sampler   TEXT,
    preferred_scheduler TEXT,
    optimal_resolution  TEXT,
    notes               TEXT
);

CREATE TABLE checkpoint_prompt_terms (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    checkpoint_id   INTEGER REFERENCES checkpoints(id) ON DELETE CASCADE,
    term            TEXT NOT NULL,
    effect          TEXT NOT NULL,
    strength        TEXT CHECK(strength IN ('strong', 'moderate', 'weak', 'broken')),
    example_image_id TEXT REFERENCES images(id),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE checkpoint_observations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    checkpoint_id   INTEGER REFERENCES checkpoints(id) ON DELETE CASCADE,
    observation     TEXT NOT NULL,
    source          TEXT CHECK(source IN ('user', 'ab_comparison', 'pipeline_note', 'auto_rating')),
    comparison_id   TEXT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- A/B Comparisons
-- ============================================

CREATE TABLE comparisons (
    id              TEXT PRIMARY KEY,           -- UUID
    image_a_id      TEXT REFERENCES images(id),
    image_b_id      TEXT REFERENCES images(id),
    variable_changed TEXT NOT NULL,             -- What was different: "checkpoint", "cfg", "sampler", etc.
    note            TEXT,                        -- User's observation
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- Smart Queue (persistent)
-- ============================================

CREATE TABLE queue_jobs (
    id              TEXT PRIMARY KEY,           -- UUID
    priority        INTEGER DEFAULT 1,          -- 0=High, 1=Normal, 2=Low
    status          TEXT CHECK(status IN ('pending', 'generating', 'completed', 'failed', 'cancelled')),
    positive_prompt TEXT NOT NULL,
    negative_prompt TEXT NOT NULL,
    settings_json   TEXT NOT NULL,              -- Full generation settings as JSON
    pipeline_log    TEXT,                        -- Lineage trace JSON
    original_idea   TEXT,
    linked_comparison_id TEXT,                  -- If part of an A/B pair
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at      DATETIME,
    completed_at    DATETIME,
    result_image_id TEXT REFERENCES images(id)
);

-- ============================================
-- Indexes
-- ============================================

CREATE INDEX idx_images_checkpoint ON images(checkpoint);
CREATE INDEX idx_images_seed ON images(seed);
CREATE INDEX idx_images_created ON images(created_at);
CREATE INDEX idx_images_rating ON images(rating);
CREATE INDEX idx_images_deleted ON images(deleted);
CREATE INDEX idx_checkpoint_terms_checkpoint ON checkpoint_prompt_terms(checkpoint_id);
CREATE INDEX idx_queue_status ON queue_jobs(status, priority);
CREATE INDEX idx_seeds_value ON seeds(seed_value);
```

---

## Pipeline System Prompts (Defaults, User-Editable)

These are stored in `config.toml` and editable in the Settings panel under "Pipeline Prompts":

### Ideator

```
You are a creative director brainstorming visual concepts. Given a simple idea, 
generate {n} distinctly different creative interpretations. Each should be a 
unique visual direction — vary the style, mood, setting, or perspective.

Output as a numbered list. Each concept should be 2-3 sentences describing the 
visual scene. Be specific and vivid. Think like a cinematographer.

User's idea: {input}
```

### Composer

```
You are a visual scene designer. Take this concept and enrich it with specific 
visual details that would make it a stunning image.

Add: specific materials and textures, lighting direction and quality, color 
palette (name specific colors), camera angle and lens characteristics, 
atmospheric effects, small details that add realism or charm.

Do NOT write in prompt syntax. Write a rich paragraph of natural description.

Concept: {concept}
```

### Judge

```
You are an art director evaluating visual concepts for image generation with 
Stable Diffusion 1.5. Rank these concepts from best to worst.

Evaluate each on:
1. Visual clarity — can this be rendered as a single coherent image?
2. SD-friendliness — does it avoid things SD1.5 struggles with (hands, text, 
   multiple specific characters, complex spatial relationships)?
3. Composition — is there a clear focal point and visual hierarchy?
4. Faithfulness — does it honor the user's original idea?
5. Appeal — would this make someone go "wow"?

Original idea: {original_input}

Concepts:
{concepts}

Return a JSON array ranked best-to-worst:
[{"rank": 1, "concept_index": <n>, "score": <0-100>, "reasoning": "..."}, ...]
```

### Prompt Engineer

```
You are an expert Stable Diffusion prompt engineer. Convert this scene 
description into optimized positive and negative prompts.

TARGET CHECKPOINT: {checkpoint_name}
Base model: {base_model}

CHECKPOINT BEHAVIORAL PROFILE:
Strengths: {strengths}
Weaknesses: {weaknesses}
Preferred CFG: {cfg_range_low}–{cfg_range_high}
Preferred sampler: {preferred_sampler}
Notes: {checkpoint_notes}

KNOWN EFFECTIVE TERMS FOR THIS CHECKPOINT:
{term_list_with_effects}

Rules:
- Use comma-separated tags, not sentences
- Put the most important elements first
- Use (parentheses:weight) for emphasis, range 0.5-1.5
- Include quality boosters: masterpiece, best quality, highly detailed
- Negative prompt should cover common SD artifacts
- Keep total positive prompt under 75 tokens (CLIP limit for SD1.5)
- Match the style to the scene (photorealistic → photo terms, 
  illustration → art terms)
- Prefer terms known to be effective on the target checkpoint
- Avoid terms known to be weak or broken on the target checkpoint

Scene description:
{description}

Respond in EXACTLY this JSON format:
{
  "positive": "the positive prompt here",
  "negative": "the negative prompt here"
}
```

### Reviewer

```
Compare this SD prompt against the user's original idea. Check for:
1. Prompt drift — did we lose the core of what they asked for?
2. Conflicting terms — anything contradictory?
3. Token bloat — is the prompt over-stuffed?
4. Missing elements — anything from the original idea that got dropped?

Original idea: {original_input}
Positive prompt: {positive}
Negative prompt: {negative}

If the prompts are good, respond: {"approved": true}
If changes needed, respond: {"approved": false, "issues": [...], 
"suggested_positive": "...", "suggested_negative": "..."}
```

---

## Development Phases

### Phase 1: Foundation (MVP)
- Tauri 2 project scaffold with React frontend
- Settings panel: endpoint configuration, model assignment, quality presets
- Single-stage prompt generation (Prompt Engineer only, all other stages bypassed)
- ComfyUI API integration: queue workflow, poll, retrieve image
- Basic image viewer for results
- SQLite setup with core `images` table
- Simple sequential queue (no priority yet)

### Phase 2: Full Pipeline + Auto-Approve
- All 5 pipeline stages with configurable models
- Pipeline stepper UI with intervention points
- Stage bypass toggles (per-stage on/off)
- Stage output visibility and editing
- Judge ranking display with user override
- Auto-approve checkbox
- Lineage data collection (pipeline_log JSON stored per image)

### Phase 3: Gallery + Seed Library
- Gallery grid view with thumbnails and infinite scroll
- Lightbox with metadata panel
- Search, filter, sort (by tags, date, rating, checkpoint, etc.)
- Soft delete with trash view
- Re-use prompt from gallery images
- Seed Library: save, comment, tag, browse, pick
- Seed variation with denoise slider

### Phase 4: AI Tagger & Captioner
- Vision model integration for tagging
- Structured tag extraction and storage
- Caption generation
- Inline editing for both
- Batch processing via queue

### Phase 5: A/B Comparison + Smart Queue
- Smart Queue with priority levels and drag-to-reorder
- Queue persistence across restarts
- A/B Quick-Generate workflow
- Comparison viewer with drag slider
- Auto-diff table for parameter comparison
- Comparison notes and storage

### Phase 6: Checkpoint Knowledge DB
- Checkpoint auto-scaffold on first use
- Manual notes panel in settings
- A/B comparison → observation push
- Rating feedback loop (auto-populate preferred settings from high-rated images)
- Prompt term annotation from lineage viewer
- Prompt Engineer stage checkpoint context injection

### Phase 7: Polish + RAG Preparation
- Lineage viewer with full timeline UI and annotations
- Export bundles (ZIP + manifest)
- Pipeline prompt editing in settings
- PSU protection / cooldown logic
- Home Assistant integration (optional)
- Keyboard shortcuts
- Checkpoint Knowledge export for vectorization (JSON-lines format)
- Documentation for RAG integration path

---

## API Reference (Internal)

### Tauri Commands (Rust → Frontend)

```rust
// ---- Pipeline ----
#[tauri::command]
async fn run_pipeline_stage(stage: String, input: String, model: String, 
                            checkpoint_context: Option<String>) -> Result<String, String>;

#[tauri::command]
async fn run_full_pipeline(idea: String, num_concepts: u32, 
                           auto_approve: bool) -> Result<PipelineResult, String>;

// ---- ComfyUI ----
#[tauri::command]
async fn get_available_checkpoints(endpoint: String) -> Result<Vec<String>, String>;

#[tauri::command]
async fn queue_generation(prompt: GenerationRequest) -> Result<String, String>;

#[tauri::command]
async fn get_generation_status(prompt_id: String) -> Result<GenerationStatus, String>;

// ---- Smart Queue ----
#[tauri::command]
async fn add_to_queue(job: QueueJob) -> Result<String, String>;

#[tauri::command]
async fn get_queue() -> Result<Vec<QueueJob>, String>;

#[tauri::command]
async fn reorder_queue(job_id: String, new_position: u32) -> Result<(), String>;

#[tauri::command]
async fn cancel_queue_job(job_id: String) -> Result<(), String>;

#[tauri::command]
async fn pause_queue() -> Result<(), String>;

#[tauri::command]
async fn resume_queue() -> Result<(), String>;

// ---- Gallery ----
#[tauri::command]
async fn get_gallery_images(filter: GalleryFilter) -> Result<Vec<ImageEntry>, String>;

#[tauri::command]
async fn delete_image(id: String) -> Result<(), String>;

#[tauri::command]
async fn update_caption(id: String, caption: String) -> Result<(), String>;

#[tauri::command]
async fn add_tag(image_id: String, tag: String, source: String) -> Result<(), String>;

#[tauri::command]
async fn get_image_lineage(image_id: String) -> Result<PipelineLineage, String>;

// ---- AI Features ----
#[tauri::command]
async fn auto_tag_image(image_id: String, model: String) -> Result<Vec<Tag>, String>;

#[tauri::command]
async fn auto_caption_image(image_id: String, model: String) -> Result<String, String>;

// ---- Seed Library ----
#[tauri::command]
async fn save_seed(seed: SeedEntry) -> Result<i64, String>;

#[tauri::command]
async fn get_seeds(filter: SeedFilter) -> Result<Vec<SeedEntry>, String>;

#[tauri::command]
async fn add_seed_checkpoint_note(seed_id: i64, checkpoint: String, 
                                   note: String) -> Result<(), String>;

// ---- Checkpoint Knowledge ----
#[tauri::command]
async fn get_checkpoint_profile(filename: String) -> Result<CheckpointProfile, String>;

#[tauri::command]
async fn update_checkpoint_profile(profile: CheckpointProfile) -> Result<(), String>;

#[tauri::command]
async fn add_prompt_term(checkpoint_id: i64, term: String, effect: String,
                         strength: String) -> Result<(), String>;

#[tauri::command]
async fn add_checkpoint_observation(checkpoint_id: i64, observation: String,
                                     source: String) -> Result<(), String>;

#[tauri::command]
async fn get_checkpoint_context_for_prompt_engineer(checkpoint: String) 
    -> Result<String, String>;

// ---- A/B Comparison ----
#[tauri::command]
async fn create_comparison(image_a: String, image_b: String, 
                           variable: String) -> Result<String, String>;

#[tauri::command]
async fn save_comparison_note(comparison_id: String, note: String) -> Result<(), String>;

#[tauri::command]
async fn push_comparison_to_checkpoint_db(comparison_id: String) -> Result<(), String>;

// ---- Settings ----
#[tauri::command]
async fn get_config() -> Result<Config, String>;

#[tauri::command]
async fn save_config(config: Config) -> Result<(), String>;

#[tauri::command]
async fn get_available_models(endpoint: String) -> Result<Vec<String>, String>;

// ---- Export ----
#[tauri::command]
async fn export_bundle(image_ids: Vec<String>, path: String) -> Result<String, String>;

#[tauri::command]
async fn export_checkpoint_knowledge(format: String) -> Result<String, String>;
```

---

## Key Design Decisions

**Why sequential pipeline, not parallel?** Single GPU. Even though the LLM stages 
don't use the GPU simultaneously, Ollama loads one model at a time efficiently. 
If all stages use the same model, there's zero swap overhead. If they use different 
models, Ollama handles the swap with KEEP_ALIVE. Sequential is the correct choice 
for this hardware.

**Why construct ComfyUI workflows in code?** The user shouldn't need to touch 
ComfyUI's node editor for basic generation. VisionForge owns the workflow template 
and injects the dynamic values. Power users could add custom workflow JSON templates 
later (Phase 7+).

**Why SQLite over JSON files?** Tags, search, filtering, relational queries, 
checkpoint knowledge, seed cross-referencing — this is relational data. JSON files 
would require loading everything into memory for any query. SQLite also makes the 
future RAG vectorization pipeline straightforward: export rows as JSON-lines, embed, 
index.

**Why Tauri over a web app?** File system access for the gallery (images on disk), 
native window management, system tray for background generation queue, and no need 
for a separate backend server process.

**Why vision models for tagging instead of CLIP?** CLIP embeddings would be more 
technically elegant but require loading another model. Vision LLMs through Ollama 
are already on your box and produce human-readable tags with zero additional setup.

**Why build the Checkpoint Knowledge DB before RAG?** You can't do retrieval-augmented 
generation without a corpus. The structured DB is the corpus-building tool. By 
collecting checkpoint behavioral data through natural usage — A/B comparisons, 
rating feedback, prompt annotations — you're creating high-quality, domain-specific 
training data. The vector embedding layer is a drop-in upgrade once the corpus is 
rich enough to justify it.

**Why auto-approve is a persistent setting, not a per-job toggle?** It changes the 
workflow mode, not a single generation. When you're exploring, you want manual 
approval. When you're batching overnight, you want fire-and-forget. The persistent 
checkbox with a visible indicator avoids the "wait, why did it just generate without 
asking me?" surprise.
