# LitTCG Technical Manual

> Build pipeline, cross-project patterns, and integration notes for the LitTCG engine.

---

## Document Status

| Field | Value |
|-------|-------|
| **Date** | July 2026 |
| **Engine** | Bevy 0.18.1 ECS (Rust) |
| **Targets** | Desktop → Web (WASM) → Android XR |
| **Purpose** | Technical reference for build pipeline, architecture patterns, and reusable code from adjacent projects |

## Current Position

LitTCG is a **game**, not an engine. We evaluated [SpawnForge](https://github.com/Tristan578/project-forge) as a potential foundation and decided **not to switch engines**. The SpawnForge sections below are preserved as reference material for patterns we may adopt later (dual WASM build, bridge isolation, command-driven API, PWA export). The active integration targets are **Voix Vive** (working hand tracking, spatial UI, voice control, Google OAuth → Gemini) and **Trinity ID AI OS** (Socratic patterns, ADDIECRAPEYE methodology, optional persistent memory for premium features).

---

## 1. SpawnForge Overview

SpawnForge is an open-source, AI-native 2D/3D browser game engine. Bevy ECS → WASM + React editor + 350 MCP commands.

| Metric | Value |
|--------|-------|
| **Engine** | Bevy ECS → WebAssembly |
| **Rendering** | WebGPU (primary), WebGL2 (fallback) |
| **Frontend** | Next.js 16, Zustand, Tailwind |
| **MCP Commands** | 350 across 41 categories |
| **AI Tools** | 6 (Claude Code, Copilot, Gemini CLI, Windsurf, Antigravity, Codex) |
| **Material Presets** | 56 PBR presets, shader node editor |
| **Particles** | GPU compute shaders, 9 presets |
| **Physics** | Rapier 3D + Rapier 2D |
| **Audio** | Spatial 3D, bus mixer, adaptive music |
| **Post-Processing** | Bloom, SSAO, DoF, motion blur, color grading |

### Architecture

```
MCP Server (350 commands)
    AI agents + LLM tool use
        |  JSON commands
React Shell (Next.js 16)
    Visual editor UI
        |  JSON events via wasm-bindgen
Bevy Engine (Rust → WASM)
    Scene editing + WebGPU rendering
        |
    Game Runtime + TypeScript Scripting
```

Key insight: **MCP server and visual editor share the same command interface**. An agent calling `set_material` goes through the same code path as a user dragging a color picker. No separate "AI mode."

### Project Structure

```
project-forge/
├── engine/              # Bevy ECS (Rust → WASM)
│   ├── src/
│   │   ├── bridge/      # JS interop (wasm-bindgen, events)
│   │   └── core/        # Pure Rust: commands, ECS, pending queues
│   └── Cargo.toml
├── web/                 # Next.js frontend
│   ├── src/
│   │   ├── components/  # React UI panels, inspectors
│   │   ├── hooks/       # WASM loader, engine events
│   │   ├── stores/      # Zustand state
│   │   └── lib/         # Audio, scripting, export, shaders
│   └── package.json
├── mcp-server/          # MCP command manifest + tools
│   └── manifest/commands.json   # 350 commands
├── docs/                # Human + AI readable docs
├── specs/               # Feature specs and sprint plans
├── build_wasm.sh        # Dual WASM build (WebGPU + WebGL2)
└── README.md
```

---

## 2. Architecture Comparison

| Dimension | Communication Class | SpawnForge |
|-----------|-------------------|------------|
| **Type** | Game (fixed experience) | Engine (tool for building games) |
| **Engine** | Bevy 0.18.1 ECS | Bevy 0.18 ECS |
| **WASM** | `trunk serve` (single binary) | `build_wasm.sh` (dual: WebGPU + WebGL2) |
| **Frontend** | None (Bevy UI only) | Next.js + React + Zustand + Tailwind |
| **Command API** | Direct ECS system calls | `handle_command()` JSON API (350 commands) |
| **AI Integration** | AGENTS.md + diapers mode | 6 AI tools, shared hooks, skills, taskboard |
| **Physics** | None | Rapier 3D + 2D |
| **Audio** | Kokoro TTS sidecar + state-driven procedural music (`music.rs`) | Spatial 3D, bus mixer, adaptive music |
| **Particles** | CPU-side burst + aura | GPU compute shaders, 9 presets |
| **Materials** | Basic StandardMaterial | 56 PBR presets, WGSL shader editor |
| **Post-Processing** | Bloom, SSAO (desktop only) | Bloom, SSAO, DoF, motion blur, color grading |
| **Scene Mgmt** | GameState machine (10 states) | Multi-scene, transitions, prefabs |
| **UI** | Bevy UI (2D + spatial) | React editor + 10 widget types |
| **Scripting** | None | TypeScript `forge.*` API (14+ namespaces) |
| **Export** | None yet | ZIP + PWA, cloud publishing |
| **Testing** | 8 integration tests | Vitest (web + MCP) |
| **Data** | 5 embedded JSON (3.3MB) | Scene files, asset pipeline |

**Key difference:** We're a **game**, they're an **engine**. We can't use them as our engine, but we can adopt their patterns, build pipeline, and tooling. They can adopt our FACES protocol and educational systems as plugins.

---

## 3. What We Can Adopt from SpawnForge

### 3.1 High-Value (Immediate Impact)

| Pattern | Solves | How We Adapt |
|---------|--------|-------------|
| **Dual WASM build** | Need WebGPU for better browser rendering | Copy `build_wasm.sh`: build two WASM binaries, auto-select at runtime |
| **Bridge isolation** | `reqwest::blocking` breaks WASM | Rule: only `bridge/` imports `web_sys`/`js_sys`/`wasm_bindgen`. Core stays pure Rust. |
| **Architecture validator** | No automated arch checks | Adapt `check_arch.py` for our module boundaries |
| **Agentic hooks** | AGENTS.md exists but no enforcement | Adopt hook lifecycle: session-start, prompt-submit, post-edit-lint, on-stop |
| **PWA export** | Need itch.io distribution | Study their ZIP export with PWA generation |

### 3.2 Medium-Value (Next Sprint)

| Pattern | Solves | How We Adapt |
|---------|--------|-------------|
| **Command-driven API** | Testing is hard, systems tightly coupled | Wrap key ops (spawn_pet, start_battle) in `handle_command()` JSON API |
| **Quality presets** | WASM perf varies by device | Low/Medium/High/Ultra presets for particles, MSAA, shadows |
| **Scene transitions** | State transitions are abrupt | Add fade/wipe transitions between GameState changes |
| **Prefab system** | Pet creation is hardcoded | Define pet archetypes as data-driven prefabs |
| **Material presets** | Pets use basic colors | Create presets per element (Fire = emissive + noise, Water = transparent + ripple) |

### 3.3 Future (Post-Demo)

| Pattern | Solves | How We Adapt |
|---------|--------|-------------|
| **GPU compute particles** | CPU-side particles are limited | WebGPU compute shaders for pet aura effects |
| **Custom WGSL shaders** | Pets look basic | Custom shaders for element effects (fire distortion, water ripple) |
| **Visual scripting** | Teachers can't create custom quests | Long-term: visual quest builder with React Flow |
| **TypeScript scripting** | Can't extend without recompiling | `forge.*`-style API for educational modding |

---

## 4. What We Can Contribute to SpawnForge

### 4.1 FACES Protocol as a Bevy Plugin

Our `faces-protocol` crate is self-contained and extractable:

```rust
use faces_protocol::{FacesState, detect};

// Any Bevy game could spawn entities from text:
let state = detect::detect_scored("inferno").state;
// → Aura: red-orange, Container: Sharp (cone), Focus: Intense, Action: Aggressive
```

**Value to SpawnForge:** Their engine creates entities from manual config. With FACES, they could spawn entities from **text descriptions** — type "angry fire demon" and get a red, sharp, intense-faced entity. Aligns perfectly with their AI-first philosophy.

### 4.2 Psycholinguistic Data Pipeline

9,582 words with concreteness, valence, arousal, dominance — reusable as a shared resource:

```rust
pub struct PsycholinguisticStats {
    pub concreteness: f32,  // → weight/attack
    pub valence: f32,       // → health/positivity
    pub arousal: f32,       // → speed/energy
    pub dominance: f32,     // → defense/control
}
```

**Value:** Any game spawning entities from words could use this. AI describing "a fast, aggressive enemy" maps to high-arousal, low-valence words.

### 4.3 Etymology → Element Mapping

25 roots → 7 elements, 27 suffixes → 8 roles. Unique word-to-game-stat mapping.

### 4.4 Semantic Distance Combat

Euclidean distance across psycholinguistic vectors — a novel combat/matchmaking system:

```rust
pub fn semantic_distance(a: &WordStats, b: &WordStats) -> f32 {
    let dc = a.concreteness - b.concreteness;
    let dv = a.valence - b.valence;
    let dd = a.dominance - b.dominance;
    let da = a.arousal - b.arousal;
    (dc*dc + dv*dv + dd*dd + da*da).sqrt()
}
```

### 4.5 Educational Game Design Patterns

Our GDD — VAAM, RPS Trivium, spiral curriculum, mastery evolution — serves as a template for educational games built in SpawnForge.

### 4.6 COPPA-Compliant Save System

Local-first JSON, no cloud, no accounts, human-readable files. Pattern any educational game needs.

---

## 5. External Project Integration

### 5.1 Voix Vive (`/home/joshua/Workflow/Bertrand-Masterclass`)

| Asset | Use in LitTCG | Priority |
|-------|---------------|----------|
| Real OpenXR hand tracking | Replace LitTCG stubs; enable XR pinch-to-spell | P0 |
| Spatial UI panels (`holographic_ui.rs`, `spatial_ui.rs`, `widgets.rs`) | XR menus and pet cards after fixing Bevy ParamSet conflicts | P1 |
| Google OAuth → Gemini pattern | Optional AI tutor where student's Google quota pays API costs | P1 |
| Voice control system | Optional voice spelling / accessibility | P2 |
| i18n framework (EN/FR) | Localization pattern for dashboard or companion app | P2 |
| Procedural music & spatial audio feedback | State-driven WAV stems (`music.rs`) + future XR spatial drone placement | P1 |
| PWA / Cloudflare deploy | Template for LitTCG dashboard or landing page | P2 |
| BSL 1.1 license | IP protection model for LitTCG | P1 |

### 5.2 Trinity ID AI OS (`/home/joshua/Workflow/TRINITYIDAIOS`)

| Asset | Use in LitTCG | Priority |
|-------|---------------|----------|
| ADDIECRAPEYE methodology | Task/project management discipline (already used in `task.md`) | P2 |
| Socratic prompting pattern | Optional AI tutor behavior | P2 |
| Persistent memory / RAG | Optional premium dashboard: long-term student memory | P3 |
| EYE Package Export | Premium feature: export reports as mini-games or DOCX | P3 |
| Hotel Manager pattern | Local model orchestration if we add on-device LLM | P3 |

### 5.3 What We Do Not Adopt

| Project | What to Avoid | Why |
|---------|---------------|-----|
| SpawnForge | Do not use as engine | Would throw away working code and reset the 90-day clock |
| Trinity | Do not require Trinity backend | Would kill local-first/COPPA positioning |
| Voix Vive iOS | Do not port the full React app | Use only specific components if needed |

### 5.4 What We Are Adopting Now — Somatic Companion & Audio

The Tao of Fun slice added two pieces from the VoixVive / somatic UX playbook:

- **`src/core/companion.rs`**: a persistent 3D companion that follows the camera. This is the emotional anchor — the child chooses one pet from their collection and it stays with them through the world. In XR, the companion becomes a spatial companion drone.
- **`src/core/music.rs` + `scripts/generate_music.py`**: procedural, loop-safe WAV music that crossfades by `GameState`. The soundtrack is state-aware rather than a passive loop: menu → world → battle. Future passes will tie drone pitch to companion word data and place sounds spatially using `SpatialListener`.

Both are built with `bevy_audio` / `rodio` to avoid extra audio backends, and are feature-gated so the 2D combat slice still runs cleanly.

## 6. WASM Build Pipeline

### 6.1 Our Current Setup

```
trunk serve  →  single WASM binary  →  localhost:8080
```

Simple but limited: single binary, WebGL2 only, no WebGPU, no optimization.

### 6.2 SpawnForge's Setup

```bash
./build_wasm.sh
# 1. Compiles engine with wasm32-unknown-unknown
# 2. wasm-bindgen generates JS bindings
# 3. wasm-opt optimizes binary size
# 4. Two .wasm files: WebGPU + WebGL2
# 5. Frontend auto-selects at runtime
```

### 6.3 What We Should Adopt

**Phase 1: Dual WASM Build**

Create `build_wasm.sh` for Communication Class:
- Build WebGPU variant (`--features webgpu`)
- Build WebGL2 variant (default)
- Optimize both with `wasm-opt -Oz`
- `index.html` detects WebGPU support, loads correct binary

**Phase 2: Bridge Isolation**

Split code following SpawnForge's pattern:
```
src/
├── core/          # Pure Rust, no browser deps
│   ├── components.rs
│   ├── database.rs
│   ├── battle.rs
│   ├── quest.rs
│   └── ...
├── bridge/        # WASM interop only
│   ├── mod.rs     # wasm-bindgen exports
│   └── events.rs  # JS event callbacks
└── main.rs        # Desktop entry (non-WASM)
```

This fixes our `reqwest::blocking` WASM incompatibility — `reqwest` only in `bridge/`, not in core.

**Phase 3: wasm-opt**

Add `wasm-opt -Oz` to build pipeline for smaller binaries. Critical for itch.io load times.

---

## 7. Command-Driven Architecture

### 7.1 The Pattern

SpawnForge wraps every engine operation as a JSON command. Both the React UI and AI agents call the same `handle_command()`. This means AI can drive the editor without special "AI mode."

### 7.2 How We Adapt This

We don't need 350 commands. But wrapping key operations helps testing and future AI integration:

```rust
pub enum GameCommand {
    SpawnPet { word: String },
    StartBattle { typo_word: String },
    StartQuest { npc_name: String },
    PlayBattleCard { word: String },
    FillQuestSlot { slot_idx: usize, word: String },
    CompleteQuest,
    PetInteraction { action: PetAction },
    SaveGame,
    LoadGame,
}

pub fn handle_command(cmd: GameCommand, world: &mut World) -> Result<GameEvent, String> {
    match cmd {
        GameCommand::SpawnPet { word } => {
            // validate, analyze, spawn
            Ok(GameEvent::PetSpawned { word, element, role })
        }
        GameCommand::StartBattle { typo_word } => {
            Ok(GameEvent::BattleStarted { typo_word })
        }
        // ...
    }
}
```

**Benefits:**
- Integration tests become command sequences: `SpawnPet("fire") → StartBattle("water") → PlayBattleCard("fire") → assert victory`
- Future AI integration: MCP server wraps these commands
- Future visual editor: React UI calls these commands

### 7.3 Mapping Our Systems to Commands

| Current Function | GameCommand | Source |
|-----------------|-------------|--------|
| `submit_spelling_word()` | `SpawnPet { word }` | `letter.rs:307` |
| `start_battle()` | `StartBattle { typo_word }` | `battle.rs:25` |
| `play_battle_card()` | `PlayBattleCard { word }` | `battle.rs:62` |
| `start_quest()` | `StartQuest { npc_name }` | `quest.rs:19` |
| `fill_slot()` | `FillQuestSlot { slot_idx, word }` | `quest.rs:68` |
| `complete_quest()` | `CompleteQuest` | `quest.rs:80` |
| `save_game()` | `SaveGame` | `save.rs:15` |
| `load_game()` | `LoadGame` | `save.rs:34` |

---

## 8. Agentic Development Infrastructure

### 8.1 SpawnForge's Setup

SpawnForge supports 6 AI coding tools with shared enforcement:

```
Tool Config (JSON/TOML)
    |
Shared Bash Scripts (.claude/hooks/)
    ├── taskboard-state.sh       # API helpers, validation
    ├── github_project_sync.py   # GitHub Projects sync
    ├── on-session-start.sh      # Pull state, display backlog
    ├── on-prompt-submit.sh      # Ticket gate (must have ticket before code)
    ├── on-stop.sh               # Post-response validation
    ├── post-edit-lint.sh        # ESLint on changed files
    ├── sync-to-github.sh        # Push changes
    └── sync-from-github.sh      # Pull changes
```

### 8.2 Hook Lifecycle

| Hook | When | What It Does |
|------|------|-------------|
| `on-session-start.sh` | Session begins | Pull GitHub, start taskboard, display backlog |
| `on-prompt-submit.sh` | Before AI processes prompt | Validate ticket exists (no ticket = no code) |
| `on-stop.sh` | After AI response | Validate ticket state, check for stale tickets |
| `post-edit-lint.sh` | After file edit | Run linter on changed `.ts`/`.tsx` files |
| `sync-to-github.sh` | After changes | Push to GitHub Project |
| `sync-from-github.sh` | Session start | Pull from GitHub Project |

### 8.3 What We Already Have

| SpawnForge Feature | Our Equivalent |
|-------------------|---------------|
| `.claude/` config | `AGENTS.md` (workspace rules) |
| Skills (kanban, sync, planner, builder, cycle) | Diapers mode skill |
| Taskboard | `task.md` (manual) |
| Architecture validator | None |
| GitHub Projects sync | None |
| Hook enforcement | None (AGENTS.md is advisory) |
| Multi-tool support | Windsurf only |

### 7.4 What We Should Adopt

**Phase 1: Hook Scripts**

Create `.windsurf/hooks/` with:
- `on-session-start.sh` — Run `cargo test`, display pass/fail
- `post-edit-lint.sh` — Run `cargo check` on edited `.rs` files
- `on-stop.sh` — Run `cargo test` to verify no regressions

**Phase 2: Architecture Validator**

Create `scripts/check_arch.py` enforcing our rules:
1. `main.rs` may not contain game logic (only system registration)
2. `render.rs` may not import `database.rs` or `quest.rs`
3. `database.rs` may not import `render.rs` or `battle.rs`
4. No `web_sys`/`js_sys` imports outside of a `bridge/` module
5. All `GameState` transitions must go through `NextState<GameState>`
6. All public functions must have doc comments or `#[allow(dead_code)]`
7. No `unwrap()` in production code (use `unwrap_or` or `?`)

**Phase 3: Taskboard Integration**

Evaluate their taskboard tool (`github.com/tcarac/taskboard`) for our use. If it works, integrate with GitHub Issues for our repo.

---

## 8. Rendering & Visual Polish

### 8.1 What SpawnForge Has

| Feature | SpawnForge | Us |
|---------|-----------|-----|
| **PBR Materials** | 56 presets, clearcoat, transmission, IOR, parallax | Basic StandardMaterial |
| **Shader Editor** | Visual WGSL node editor, 30+ node types | None |
| **Custom Shaders** | 8 WGSL templates (Dissolve, Hologram, Toon, etc.) | None |
| **Particles** | GPU compute shaders, 9 presets | CPU-side burst + aura |
| **Post-Processing** | Bloom, SSAO, DoF, motion blur, color grading, CAS | Bloom + SSAO (desktop only) |
| **Skybox** | 5 procedural cubemap presets | Solid color |
| **LOD System** | Distance thresholds, performance budgets | None |
| **Quality Presets** | Low/Medium/High/Ultra | None |

### 8.2 What We Should Adopt

**Immediate: Material Presets per Element**

Instead of basic `StandardMaterial` for each pet, create element-specific presets:

```rust
fn fire_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(0.9, 0.3, 0.0),
        emissive: Color::srgb(1.5, 0.5, 0.0).into(),
        metallic: 0.1,
        perceptual_roughness: 0.3,
        ..default()
    }
}

fn water_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(0.0, 0.4, 0.8),
        metallic: 0.3,
        perceptual_roughness: 0.05,  // very smooth = reflective
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}
```

**Next: Quality Presets for WASM**

```rust
enum QualityPreset { Low, Medium, High, Ultra }

fn apply_quality(preset: QualityPreset, app: &mut App) {
    match preset {
        QualityPreset::Low => {
            // Disable particles, reduce MSAA, no shadows
        }
        QualityPreset::Medium => {
            // 10 particles per pet, 2x MSAA, simple shadows
        }
        QualityPreset::High => {
            // 20 particles per pet, 4x MSAA, soft shadows
        }
        QualityPreset::Ultra => {
            // 30 particles, 8x MSAA, SSAO, bloom
        }
    }
}
```

**Future: Custom WGSL Shaders**

Element-specific visual effects:
- **Fire**: Dissolve shader with animated noise (flickering flames)
- **Water**: UV scroll shader (rippling surface)
- **Shadow**: Fresnel glow shader (dark aura)
- **Light**: Hologram shader (translucent, scan lines)
- **Earth**: Parallax mapping (rough, rocky texture)
- **Air**: Force field shader (translucent, shimmering)

SpawnForge has 6 built-in shader effects we can study and adapt.

---

## 9. UI Systems

### 9.1 What SpawnForge Has

| Feature | SpawnForge | Us |
|---------|-----------|-----|
| **Editor** | Dockable React workspace | Bevy UI only |
| **Widgets** | 10 types, WYSIWYG, data binding | Hardcoded UI |
| **Dialogue** | Visual node editor, 5 node types, branching | Basic text panels |
| **Scene Transitions** | Fade, wipe, instant | Hard cuts |
| **Input Presets** | FPS, Platformer, Top-Down, Racing | Keyboard + swipe |
| **Mobile Controls** | Virtual joystick, action buttons | None |

### 9.2 What We Should Adopt

**Immediate: Scene Transitions**

Add fade transitions between GameState changes:

```rust
fn transition_to_state(
    new_state: GameState,
    mut fade: ResMut<FadeOverlay>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    fade.target_state = Some(new_state);
    fade.alpha = 0.0;
    fade.fading_out = true;
}

fn update_fade(
    mut fade: ResMut<FadeOverlay>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    if fade.fading_out {
        fade.alpha += time.delta_seconds() * 2.0;
        if fade.alpha >= 1.0 {
            if let Some(state) = fade.target_state.take() {
                next_state.set(state);
                fade.fading_out = false;
            }
        }
    } else if fade.alpha > 0.0 {
        fade.alpha -= time.delta_seconds() * 2.0;
    }
}
```

**Next: Dialogue System Upgrade**

SpawnForge has a visual node editor with 5 node types (text, choice, condition, action, end). We could adopt this pattern for NPC dialogue — branching conversations based on pet collection, attunement, and quest history.

**Future: Mobile Controls**

For the Android XR target, adopt their virtual joystick + action button pattern as a fallback when hand tracking isn't available.

---

## 10. Scene & Prefab Management

### 10.1 What SpawnForge Has

| Feature | Description |
|---------|-------------|
| **Multi-Scene** | Multiple named scenes per project, switching, import/export |
| **Prefabs** | Reusable entity templates, 8+ built-in, import/export |
| **Scene Hierarchy** | Tree view, parent-child relationships |
| **Play Mode** | Test games instantly with snapshot restore |

### 10.2 What We Should Adopt

**Prefab System for Pets**

Instead of hardcoded pet spawning in `letter.rs:307`, define pet archetypes as data:

```rust
// pet_prefabs.ron
[
    FireBruiser: (
        word_pattern: "*ign*",
        element: Fire,
        role: Bruiser,
        base_stats: (logos: 80, pathos: 50, ethos: 60, speed: 40),
        material: "fire_material",
        mesh: "sharp_cone",
        particles: "fire_aura",
    ),
    WaterTank: (
        word_pattern: "*aqu*",
        element: Water,
        role: Tank,
        base_stats: (logos: 40, pathos: 70, ethos: 80, speed: 30),
        material: "water_material",
        mesh: "fluid_torus",
        particles: "water_droplets",
    ),
]
```

This separates data from code, making it easier to balance and extend.

**Scene Management for Districts**

Each of our 12 districts could be a separate scene with its own lighting, skybox, and letter spawn points. SpawnForge's multi-scene pattern shows how to manage this.

---

## 11. Publishing & Export

### 11.1 What SpawnForge Has

| Feature | Description |
|---------|-------------|
| **ZIP Export** | Standalone ZIP with texture compression, custom loading screen, PWA |
| **Cloud Publishing** | Shareable URLs, version management, analytics |
| **PWA Generation** | Progressive Web App manifest, offline support |

### 11.2 What We Need for itch.io

**Phase 1: WASM Optimization**

```bash
# Our build_wasm.sh (to create):
#!/bin/bash
set -e

echo "Building WebGL2 WASM..."
trunk build --release --features flat2d
wasm-opt -Oz dist/*.wasm -o dist/communication-class-gl2.wasm

echo "Building WebGPU WASM..."
trunk build --release --features webgpu
wasm-opt -Oz dist/*.wasm -o dist/communication-class-gpu.wasm

echo "Building index.html with auto-detection..."
# index.html checks navigator.gpu and loads correct binary
```

**Phase 2: PWA Manifest**

Create `manifest.json` for offline play:
```json
{
  "name": "Communication Class",
  "short_name": "CommClass",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#1a1a2e",
  "theme_color": "#1a1a2e",
  "icons": [...]
}
```

**Phase 3: itch.io Upload**

Study SpawnForge's ZIP export pattern. Package WASM + assets + index.html into a single ZIP, upload to itch.io as a WebGL game.

---

## 12. Collaboration Plan

### 12.1 Shared Interests

Both projects are:
- **Bevy 0.18** student projects
- Targeting **WASM/browser** deployment
- Using **AI-assisted development** (we use Windsurf, they support 6 tools)
- Interested in **procedural generation** from text/data
- Building with **Rust** as the primary language

### 12.2 What We Offer Tristan

| Asset | Value to SpawnForge |
|-------|-------------------|
| **FACES protocol** | Text → 3D face generation. 38,400 unique states. Could be a SpawnForge plugin. |
| **Psycholinguistic database** | 9,582 words with C/V/A/D data. AI could spawn entities from descriptions. |
| **Etymology → element mapping** | Word → game element system. Reusable in any word-based game. |
| **Educational game design** | GDD with VAAM, RPS Trivium, spiral curriculum. Template for edu games. |
| **COPPA-compliant save** | Local-first JSON pattern for kids' games. |
| **Bevy 0.18 WASM experience** | We've solved Bevy WASM build issues (trunk, features, asset embedding). |

### 12.3 What Tristan Offers Us

| Asset | Value to Communication Class |
|-------|---------------------------|
| **Dual WASM build pipeline** | WebGPU + WebGL2 with auto-detection. Better browser rendering. |
| **Bridge isolation pattern** | Fixes our `reqwest::blocking` WASM issue. Clean architecture. |
| **350 MCP commands** | Pattern for wrapping game operations as testable commands. |
| **Agentic dev infrastructure** | Hook lifecycle, architecture validator, taskboard sync. |
| **Material/shader presets** | 56 PBR presets, 8 WGSL shader templates for pet visuals. |
| **GPU compute particles** | Better particle system for pet auras and effects. |
| **PWA export** | itch.io-ready ZIP export with offline support. |
| **Multi-tool AI support** | Their hook system works across 6 AI tools. We could expand beyond Windsurf. |

### 12.4 Collaboration Models

**Option A: Shared Crates**

Publish reusable crates to crates.io:
- `faces-protocol` (ours → shared)
- `psycholinguistic-data` (ours → shared)
- `bevy-wasm-build` (theirs → shared)
- `bevy-arch-validator` (theirs → shared)

**Option B: Fork & Contribute**

We fork SpawnForge, add educational game templates. They fork our FACES crate, integrate as plugin.

**Option C: Joint Project**

Create a shared `bevy-edu` meta-crate that combines:
- FACES protocol (face generation from text)
- Psycholinguistic data (word → stats)
- Etymology mapping (word → element/role)
- WASM build tools (dual binary, PWA)
- Agentic dev hooks (testing, validation)

### 12.5 Contact Plan

1. **Joshua calls Tristan** — discuss shared Bevy 0.18 student status, mutual benefits
2. **Share our GDD.md** — shows the educational game design, FACES protocol, data pipeline
3. **Share this technical manual** — shows exactly what each project offers
4. **Propose shared crates** — start with `faces-protocol` as a proof of concept
5. **Set up shared GitHub** — fork or joint repo for collaboration

---

## 13. Integration Roadmap

Aligned with the 4-step sprint plan (safety → architecture → bridge/build → polish).

### Step 1: Defuse the Landmines (Safety First)

| Task | Effort | Impact | Status |
|------|--------|--------|--------|
| Rename `arousal` → `intensity` in all structs, UI, display traits | Medium | **Safety: PR nightmare prevention** | Pending |
| Implement profanity blocklist in `submit_spelling_word()` | Medium | **Safety: prevent kids summoning pets from slurs** | Pending |
| Fix `Summon` vs `SummonClass` bug in `grammar_fusion_system` | Low | **Code correctness** | ✅ Done |
| Clean up 12 compiler warnings | Low | Code hygiene | Pending |

### Step 2: Architecture Scaffolding

| Task | Effort | Impact | Status |
|------|--------|--------|--------|
| Create `src/core/commands.rs` with `GameCommand` enum + `handle_command()` | Medium | Testable game operations, decouples input from logic | Pending |
| Reroute `submit_spelling_word()` and hardware input to use command flow | High | Unifies VR pinch / desktop click / AI test under one API | Pending |
| Set up Windsurf hooks (`on-session-start.sh`, `post-edit-lint.sh`) | Low | Automated `cargo test` + `cargo check` on edits | Pending |
| Create `scripts/check_arch.py` architecture validator | Medium | Enforce module boundaries (render vs logic vs data) | Pending |

### Step 3: Bridge Isolation & Build Pipeline

| Task | Effort | Impact | Status |
|------|--------|--------|--------|
| Audit `src/` — move WASM-incompatible deps to `src/bridge/` | High | Fixes `reqwest::blocking` WASM issue, clean architecture | Pending |
| Ensure `src/core/` is pure Rust (no `web_sys`/`js_sys`) | High | Cross-compilation safety for Google Aura | Pending |
| Create `build_wasm.sh` with dual binary (WebGPU + WebGL2) | Medium | Better browser rendering, Google Aura WebGPU support | Pending |
| Add `wasm-opt -Oz` to build pipeline | Low | Smaller WASM binary, faster itch.io load | Pending |

### Step 4: Visual Polish

| Task | Effort | Impact | Status |
|------|--------|--------|--------|
| Extract hardcoded pet materials into `pet_prefabs.ron` | Medium | Data-driven element-specific PBR materials | Pending |
| Implement `FadeOverlay` transitions between `GameState` changes | Low | Remove abrupt hard-cuts, professional feel | Pending |
| Add quality presets (Low/Medium/High) for WASM | Medium | Performance scaling across devices | Pending |
| Create PWA manifest for offline play | Low | itch.io ZIP distribution | Pending |

### Post-Sprint: Demo Ship

| Task | Effort | Impact |
|------|--------|--------|
| Pet Card reveal animation (Pokéball moment) | High | Core emotional hook |
| Pet Collection screen | Medium | Browse collected pets |
| Roster selection (3-6 pets) | Medium | Battle preparation |
| Package itch.io ZIP | Low | Demo distribution |
| Extract `faces-protocol` as shared crate for Tristan | Medium | Collaboration deliverable |

---

## Appendix: SpawnForge Feature Reference

### 3D Engine Features

- WebGPU rendering (wgpu 27) with WebGL2 fallback
- PBR materials: metallic/roughness, clearcoat, transmission/IOR, parallax
- 56 material presets across 9 categories
- Visual WGSL shader node editor (30+ node types)
- Quality presets: Low/Medium/High/Ultra
- Dynamic lighting: point, directional, spot with shadows
- 5 procedural skybox presets
- Rapier 3D physics: rigid bodies, colliders, joints, raycasting
- Spatial 3D audio, bus mixer, reverb zones, adaptive music
- GPU compute particles: 9 presets (fire, smoke, sparks, rain, snow, explosions)
- Skeletal animation (glTF), keyframe animation
- CSG boolean operations (union, subtract, intersect)
- Procedural terrain (Perlin/Simplex/Value noise)
- 6 custom shader effects (Dissolve, Hologram, Force Field, Lava, Toon, Fresnel)
- Post-processing: Bloom, SSAO, DoF, motion blur, color grading, CAS
- LOD system with distance thresholds

### 2D Engine Features

- Orthographic camera, sorting layers
- Sprite system with animation, sprite sheets, state machines
- Multi-layer tilemaps with paint/fill tools
- Rapier2D physics (6 collider shapes, 4 joint types)
- Skeletal 2D animation with IK

### Editor Features

- Dockable workspace with persistent layouts
- 3D/2D scene editor with gizmos, snapping, hierarchy
- 11 starter system bundles (Platformer, Runner, Shooter, Puzzle, Explorer)
- 6 camera modes (ThirdPerson, FirstPerson, SideScroller, TopDown, Fixed, Orbital)
- Dialogue system: visual node editor, 5 node types, branching
- Scene transitions: fade, wipe, instant
- In-game UI builder: 10 widget types, WYSIWYG, 7 screen presets
- TypeScript scripting with `forge.*` API (14+ namespaces)
- Material library with CSS sphere previews
- Play mode with snapshot restore
- Prefab system (8+ built-in, import/export)
- Multi-scene management
- Cloud publishing with version management
- ZIP export with PWA generation
- 12 pre-built game components (CharacterController, Health, Collectible, etc.)

### AI & Automation

- 350 MCP commands across 41 categories
- AI chat assistant with agentic tool loop
- 8 compound AI actions (create_scene, setup_character, etc.)
- Visual scripting: 73 node types, 10 categories
- AI asset generation: Meshy, ElevenLabs, Suno, DALL-E, Stable Diffusion
- Command-driven architecture (handle_command JSON API)
- Scene context builder for LLMs
- 28+ structured docs searchable via MCP tools

### Agentic Development

- 6 AI tools supported (Claude Code, Copilot, Gemini CLI, Windsurf, Antigravity, Codex)
- Shared hook system (.claude/hooks/)
- Skills: kanban, sync-push, sync-pull, planner, builder, cycle, arch-validator
- Taskboard with GitHub Projects sync
- Architecture validator (check_arch.py, 7 structural rules)
- Ticket validation (user story, acceptance criteria, subtasks)
- 3 Claude Code subagents: Planner (Opus), Builder (Sonnet), Validator (Sonnet)

---

*This document is a living reference. Update as collaboration develops and integration progresses.*

*Last updated: July 2026*
