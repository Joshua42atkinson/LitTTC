# LitTCG Architecture Overview

LitTCG runs on **Bevy 0.18.1**, leveraging an Entity Component System (ECS) to handle the complex state interactions required for a gamified XR literacy tool.

## 1. Module Structure (38 source files)

### Core
- `src/main.rs` — Desktop entry point. Wires plugins, resources, and systems.
- `src/lib.rs` — Library crate entry point for WASM and Android targets.
- `src/components.rs` — All ECS components and resources: `GameState`, `CharacterSheet`, `SpellBook`, `Channel`, `SummonClass`, `PetStats`, `Element`, `Role`, `PetAvatar`, `Deck`/`Hand`/`DiscardPile`, `WordTrail`, `ActiveGestures`, `GameGrid`, etc.
- `src/commands.rs` — `GameCommand` enum and `handle_game_commands` dispatcher. All input systems now send commands instead of calling game logic directly.

### Data
- `src/database.rs` — Deserializes 5 embedded JSON databases into `GameDatabase` resource. Hot-reload support via `AssetServer`.
- `src/asset_catalog.rs` — Central constants for runtime and embedded asset paths. Replaces hardcoded strings across the codebase.
- `src/save.rs` — `serde_json` persistence. Saves `CharacterSheet`, `SpellBook`, `WordTrail` to local `save.json`.

### Game Systems
- `src/letter.rs` — Letter crystal spawning, collection, word constructor UI, spelling validation, pet spawning pipeline.
- `src/deck.rs` — Card deck shuffling, draw, hand management (max 3 cards), discard pile.
- `src/battle.rs` — Turn-based semantic distance combat. `BattleSession`, `play_battle_card()`, synonym/antonym matching, SummonClass-specific combat logic.
- `src/quest.rs` — Mad-Lib quest engine. `QuestSession`, `QuestPlugin`, `GradeManager` (formerly `CurriculumManager`), NPC dialogue, slot filling, quest completion.
- `src/chat.rs` — FACES-driven pet dialogue, taming interactions (Pet/Feed/Attune), Kokoro TTS sidecar integration.
- `src/altar.rs` — Pet altar/summoning system.

### Rendering
- `src/render.rs` — Procedural 3D pet meshes from FACES state. Element-colored materials, eyes, mouths, orbital rings, wings, particle effects, screen shake. `RenderPlugin`.

### Input
- `src/input.rs` — Swipe gesture decoding (Yes/No/Deeper), keyboard input, touch handling, UI button interactions, drag state.
- `src/hand_tracking.rs` — XR hand joint tracking, ASL fingerspelling for all 26 letters, pinch events, grammar fusion system.

### UI
- `src/hud.rs` — 2D screen-space HUD: XP bar, rank, deck counter, letter stash, attunement display. `HudPlugin`.
- `src/menu.rs` — Main menu state, difficulty selection, settings screen. `MenuPlugin`.
- `src/tutorial.rs` — Onboarding flow. `TutorialPlugin`.
- `src/paywall.rs` — Demo limitations (max 10 words, no save). `PaywallPlugin`.
- `src/spatial_ui.rs` — Floating holographic 3D UI panels for XR mode. `SpatialUiPlugin`.
- `src/spatial_deck.rs` — 3D spatial card deck for XR mode. `SpatialDeckPlugin`.
- `src/dialogue_ui.rs` — NPC dialogue UI panels. `DialogueUiPlugin`.
- `src/pet_collection.rs` — Pet collection browser (card grid, sorting, companion selection). Implemented as part of the missing-stubs cleanup.

### World
- `src/time_cycle.rs` — Day/Night cycle and timing states. `TimeCyclePlugin`.
- `src/companion.rs` — Persistent player companion. Spawns the chosen pet from `SpellBookEntry::companion` and makes it follow the camera in 3D / XR. `CompanionPlugin`.
- `src/music.rs` — State-driven adaptive soundtrack. Crossfades between menu, world, and battle stems while respecting `GameSettings.music_volume`. `MusicPlugin`.

## 2. GameState Machine

```
Loading → MainMenu → Collecting → Constructing → RevealingPet → Playing → Questing/Battling → Reviewing → (loop)
                                                                                            ↓
                                                                                        Paywall (demo)
```

1. **`Loading`** — Database asset loading and parsing
2. **`MainMenu`** — Initial user routing
3. **`Collecting`** — Player explores 3D space to collect `LetterCrystal` entities
4. **`Constructing`** — Keyboard/spatial input captures spelling attempts
5. **`RevealingPet`** — Card flip / pet reveal animation and SFX
6. **`PetCollection`** — Browse and manage collected pets (set companion, sort)
7. **`Playing`** — Default interaction state, card selection
8. **`Questing`** — NPC Mad-Lib quest interaction
9. **`Battling`** — Semantic synonym/antonym combat
10. **`Reviewing`** — End-of-session summary
11. **`Paywall`** — Demo restriction barrier

## 3. Data Flow: Word to Pet

```
letter.rs: Player presses Enter, reads CurrentSpelling
    ↓
database.rs: Validate word against GameDatabase.words
    ↓
database.rs: EtymologyDB lookup → Element (root) + Role (suffix)
    ↓
components.rs: Psycholinguistic vectors → PetStats (Logos, Pathos, Ethos, Speed)
    ↓
faces-protocol: Keyword detection on definition → PetFacesState (4 bytes)
    ↓
render.rs: FacesState + Element → StandardMaterial + procedural mesh → PetAvatar entity
```

## 4. Command-Driven Architecture

Input systems in `input.rs`, `hud.rs`, `spatial_deck.rs`, and `hand_tracking.rs` now send `GameCommand` messages. A single `handle_game_commands` system reads these messages and dispatches to the existing game logic.

```
Input System (keyboard / touch / XR / UI)
    ↓
GameCommand message
    ↓
handle_game_commands (src/commands.rs)
    ↓
Existing logic: letter.rs, deck.rs, battle.rs, quest.rs, etc.
```

Benefits:
- Input sources are decoupled from game logic.
- Integration tests can drive the game via command sequences.
- Future AI/LLM integration can send commands directly.

## 5. Feature Flags

| Feature | Purpose | Key Dependencies |
|---------|---------|-----------------|
| `default` | Minimal/WASM | Bevy core only |
| `flat2d` | 2D-only mode | Bevy 2D subsets |
| `desktop` | Dev + desktop target | `bevy_panorbit_camera`, HDR, Bloom, SSAO, `reqwest` (TTS) |
| `xr` | Android XR target | `bevy_mod_xr`, `bevy_mod_openxr` |
| `tts` | Kokoro TTS sidecar | `reqwest` |
