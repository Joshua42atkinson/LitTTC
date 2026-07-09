# Communication Class — Master Task List

> Every piece of work needed to ship the demo and beyond.
> Ordered by dependency. Vision statements provide context for each phase.
> Last updated: July 2026 — STRATEGIC PIVOT: 4-Pillar Combat System

---

## STRATEGIC PIVOT: NEW PHASED EXECUTION

**CRITICAL:** All legacy phases (0-10) are SUSPENDED. We are pivoting to a 4-phase execution plan focused on validating the core combat loop in 2D before any XR integration.

**Current Focus:** PHASE 1 — 2D Gray-Box Combat Slice

---

## PHASE 1: The 2D "Gray-Box" Combat Slice (IMMEDIATE FOCUS)

**Vision:** Validate the core combat loop on a flat screen. If the game isn't fun with 2D squares, 3D VR won't save it.

### 1.1 Hardcoded Micro-Deck
- [ ] **1.1.1** Create `MICRO_DECK` constant in `deck.rs` — 20 hardcoded words:
  - 10 Nouns: wolf, wall, sword, shield, dragon, castle, river, mountain, forest, star
  - 5 Verbs: strikes, defends, heals, burns, freezes
  - 5 Adjectives: searing, impregnable, swift, ancient, radiant
- [ ] **1.1.2** Add part-of-speech tags to each word (Noun/Verb/Adjective)
- [ ] **1.1.3** Add synonym/antonym pairs for testing:
  - hot ↔ cold, fast ↔ slow, big ↔ small, bright ↔ dark, strong ↔ weak
- [ ] **1.1.4** Create `initialize_micro_deck()` function to load hardcoded deck
- [ ] **1.1.5** Modify `initialize_player_deck()` to use micro-deck in Phase 1 mode

### 1.2 2D Battle UI Scene
- [ ] **1.2.1** Create `spawn_battle_ui_2d_graybox()` in `battle.rs`
- [ ] **1.2.2** Target Dummy UI:
  - Display "PROMPT WORD: [WORD]" at top center
  - HP bar (100 HP max) with health text
  - Visual feedback when damaged (flash red)
- [ ] **1.2.3** Player Hand UI:
  - 5 card buttons at bottom of screen
  - Each button shows word text and part-of-speech icon
  - Highlight selected card
  - Click to play card to altar
- [ ] **1.2.4** Altar Drop-Zone UI:
  - Central drop zone (clickable area)
  - Show "Active Card: [WORD]" when card placed
  - "CAST SPELL" button to submit
- [ ] **1.2.5** Slime Face UI:
  - 4 emotion buttons: Fierce, Joyful, Calm, Angry
  - Show current active face with icon
  - Click to change face before casting
- [ ] **1.2.6** Combat Log UI:
  - Scrollable text panel on right side
  - Log each action with damage math
  - Color-code: green (effective), red (ineffective), gold (critical)

### 1.3 Thesaurus Battle Math Implementation
- [ ] **1.3.1** Implement `calculate_synonym_distance(word1, word2)` using existing `semantic_distance()`
- [ ] **1.3.2** Implement `is_synonym(word1, word2)` — distance < 2.0
- [ ] **1.3.3** Implement `is_antonym(word1, word2)` — distance > 4.0
- [ ] **1.3.4** Damage formula:
  - Base damage: 25
  - Synonym multiplier: 2.0x
  - Antonym multiplier: 1.5x + (distance - 4.0) × 0.2
  - Normal: 1.0x
- [ ] **1.3.5** Log damage calculation to combat log with full math breakdown
- [ ] **1.3.6** Add visual feedback: screen shake on critical, particle burst on hit

### 1.4 Face/Emotion Modifier System
- [ ] **1.4.1** Create `SlimeFace` enum: Fierce, Joyful, Calm, Angry
- [ ] **1.4.2** Create `ActiveFace` resource to track current face
- [ ] **1.4.3** Implement face modifiers to `WordStats`:
  - Fierce: +20% intensity, +10% dominance
  - Joyful: +20% valence, +10% health
  - Calm: +20% concreteness, +10% defense
  - Angry: +30% intensity, -10% valence (high risk/reward)
- [ ] **1.4.4** Apply face modifier in damage calculation
- [ ] **1.4.5** Update combat log to show face modifier effect
- [ ] **1.4.6** Add face change animation (smooth transition between icons)

### 1.5 Combat Loop Integration
- [ ] **1.5.1** Implement `start_graybox_battle()` — spawn target dummy with random prompt word
- [ ] **1.5.2** Implement `play_card_to_altar(word)` — move card from hand to altar
- [ ] **1.5.3** Implement `cast_spell()` — calculate damage, apply to dummy, log result
- [ ] **1.5.4** Implement enemy turn — dummy attacks with random word from deck
- [ ] **1.5.5** Implement victory condition — dummy HP ≤ 0
- [ ] **1.5.6** Implement defeat condition — player HP ≤ 0
- [ ] **1.5.7** Add "Next Battle" button to restart loop

### 1.6 Testing & Validation
- [ ] **1.6.1** Add integration test: synonym attack deals 2.0x damage
- [ ] **1.6.2** Add integration test: antonym attack deals 1.5x+ damage
- [ ] **1.6.3** Add integration test: face modifier alters damage correctly
- [ ] **1.6.4** Add integration test: combat log shows full math breakdown
- [ ] **1.6.5** Manual playtest: complete 10 battles, measure fun factor
- [ ] **1.6.6** Adjust damage multipliers based on playtest feedback
- [ ] **1.6.7** Verify 1-minute combat loop is replayable

### 1.7 Phase 1 Success Criteria
- [ ] **1.7.1** All integration tests pass
- [ ] **1.7.2** Zero compiler warnings
- [ ] **1.7.3** Combat loop is playable from start to finish
- [ ] **1.7.4** Synonym vs Antonym behavior is visually clear
- [ ] **1.7.5** Face modification has visible impact on gameplay
- [ ] **1.7.6** Combat log provides clear feedback on all mechanics
- [ ] **1.7.7** 5 consecutive playtesters report "I want to play again"

---

## LEGACY PHASES (SUSPENDED — DO NOT WORK ON THESE)

## Phase 0: Safety Landmines (Do First or Project Dies)

**Vision:** Make the game safe for conservative homeschool parents before any feature work. Two issues can kill the project on day one if shipped without fixing.

- [ ] **0.1** Rename `arousal` → `intensity` in `database.rs:66` (struct field, use `#[serde(alias = "arousal", alias = "A")]` so existing JSON datasets still work without rebuilding 3.3MB files)
- [ ] **0.2** Rename `arousal` → `intensity` in `letter.rs:373` (speed calc: `word_stats.arousal * 10.0`)
- [ ] **0.3** Rename `arousal` → `intensity` in `battle.rs:21` (semantic distance: `a.arousal - b.arousal`)
- [ ] **0.4** Rename `arousal` → `intensity` in `battle.rs:81` (social combat: `typo_stats.arousal + typo_stats.valence`)
- [ ] **0.5** Rename `arousal` → `intensity` in `components.rs:414` (`PetAvatar2D` struct field)
- [ ] **0.6** Rename `arousal` → `intensity` in `render.rs:720` (animation pulse: `avatar.arousal * 5.0`)
- [ ] **0.7** Verify all GDD references say "Intensity" not "Arousal" (most done, scan remaining)
- [ ] **0.8** Source profanity/slur word list (LDNOOBW — Shutterstock's open list, or similar)
- [ ] **0.9** Create `blocklist.rs` module — `pub fn is_banned(word: &str) -> bool` using `HashSet`
- [ ] **0.10** Embed blocklist as `const` array or include JSON at compile time
- [ ] **0.11** Add blocklist check in `submit_spelling_word()` at `letter.rs:307` — if banned, clear spelling silently, stay in `Constructing` state, no glitch entity
- [ ] **0.11a** **Glitch Entity UI masking** — if invalid/banned word somehow reaches the render path, mask the raw text string in the UI (replace with `[ANOMALY]` or `!#?@*`) BEFORE handing to render system. Prevents screenshots of slurs floating above pets.
- [ ] **0.12** Add test: banned word returns no pet, no state change, no entity spawned
- [ ] **0.13** Add test: normal word still works after blocklist integration
- [ ] **0.14** Clean up 12 compiler warnings:
  - [ ] **0.14.1** `count_hits` never used (faces-protocol crate)
  - [ ] **0.14.2** `AsyncReadExt` unused import (2 locations)
  - [ ] **0.14.3** Variable does not need to be mutable (2 locations)
  - [ ] **0.14.4** `bonus_evolution` value never read (`quest.rs`)
  - [ ] **0.14.5** `LetterStash` unused import
  - [ ] **0.14.6** `db` unused variable
  - [ ] **0.14.7** `spawn_vr_hand` / `cleanup_vr_hand` / `vr_quest_interaction` / `vr_battle_interaction` never used (XR stubs)
- [ ] **0.15** Remove or `#[allow(dead_code)]` the `Summon` component in `components.rs:260` (now unused after grammar_fusion fix)
- [ ] **0.16** Run `cargo test` — all 8 tests must pass
- [ ] **0.17** Run `cargo check` — zero warnings

---

## Phase 1: Architecture Scaffolding

**Vision:** Decouple hardware input from game logic. A VR pinch on Google Aura, a desktop mouse click, and an AI test script should all fire the exact same `GameCommand`. This makes testing trivial and enables future AI integration.

### Command System
- [ ] **1.1** Create `src/commands.rs` module
- [ ] **1.2** Define `GameCommand` enum:
  - `SpawnPet { word: String }`
  - `StartBattle { typo_word: Option<String> }`
  - `PlayBattleCard { word: String }`
  - `StartQuest { npc_name: String }`
  - `FillQuestSlot { slot_idx: usize, word: String }`
  - `CompleteQuest`
  - `SelectCard { index: usize }`
  - `DrawCard`
  - `StartCollecting`
  - `StartConstructing`
  - `PetInteraction { action: PetAction }`
  - `SaveGame`
  - `LoadGame`
  - `SkipToQuest { npc_name: String }`
  - `RetreatFromBattle`
  - `CancelQuest`
- [ ] **1.3** Define `PetAction` enum: `Pet`, `Feed`, `Attune(Channel)`, `SetName`, `SetCompanion`
- [ ] **1.4** Define `GameEvent` enum for return values: `PetSpawned { word, element, role, rarity }`, `BattleStarted { typo_word }`, `BattleWon { word }`, `BattleLost`, `QuestCompleted { sentence, xp }`, `QuestFailed`, `CardSelected`, `CardDrawn`, `Error { msg }`
- [ ] **1.5** Implement `GameCommand` as **Bevy Events** (`EventWriter<GameCommand>` / `EventReader<GameCommand>`) — NOT a monolithic `handle_command(cmd, &mut World)` function. Direct `&mut World` mutation is a Bevy anti-pattern that bottlenecks parallel execution and fights the borrow checker. Events achieve the same headless testability and AI integration while keeping systems decoupled.
- [ ] **1.6** Add `mod commands` to `main.rs` and `lib.rs`

### Reroute Existing Systems
- [ ] **1.7** Reroute `submit_spelling_word()` → send `GameCommand::SpawnPet { word }` event via `EventWriter`
- [ ] **1.8** Reroute `start_battle()` → send `GameCommand::StartBattle { typo_word: None }` event
- [ ] **1.9** Reroute `play_battle_card()` → send `GameCommand::PlayBattleCard { word }` event
- [ ] **1.10** Reroute `start_quest()` → send `GameCommand::StartQuest { npc_name }` event
- [ ] **1.11** Reroute `fill_slot()` → send `GameCommand::FillQuestSlot { slot_idx, word }` event
- [ ] **1.12** Reroute `complete_quest()` → send `GameCommand::CompleteQuest` event
- [ ] **1.13** Reroute `save_game()` / `load_game()` → command event flow
- [ ] **1.14** Reroute `input.rs:handle_ui_button_interactions()` → send command events instead of calling systems directly
- [ ] **1.15** Reroute XR pinch handlers (`handle_pinch_crystals`, `handle_vr_spelling`, `vr_quest_interaction`, `vr_battle_interaction`) → send command events
- [ ] **1.16** Reroute keyboard spelling (`handle_keyboard_spelling`) → send command events

### Testing
- [ ] **1.17** Add integration test: `SpawnPet("fire")` → verify `PetSpawned` event
- [ ] **1.18** Add integration test: `StartBattle(None)` → verify `BattleStarted` event
- [ ] **1.19** Add integration test: `StartQuest("Barnaby")` → `FillQuestSlot(0, "brave")` → `CompleteQuest` → verify XP gained
- [ ] **1.20** Add integration test: banned word via command → verify `Error` event, no entity spawned

### Architecture Enforcement
- [ ] **1.21** Create `scripts/check_arch.py` — enforce: (1) `main.rs` has no game logic, (2) `render.rs` no database imports, (3) `database.rs` no render imports, (4) no `web_sys` outside bridge, (5) all state transitions via `NextState`, (6) no bare `unwrap()` in production code, (7) public functions documented or `#[allow(dead_code)]`
- [ ] **1.22** Create `.windsurf/hooks/on-session-start.sh` — run `cargo test`, display pass/fail summary
- [ ] **1.23** Create `.windsurf/hooks/post-edit-lint.sh` — run `cargo clippy --workspace -- -D warnings` on edited `.rs` files (Clippy catches Bevy-specific anti-patterns like conflicting queries that standard compiler checks miss)
- [ ] **1.24** Create `.windsurf/hooks/on-stop.sh` — run `cargo test` to verify no regressions before session ends
- [ ] **1.25** Initialize `task.md` with the 4-step Integration Roadmap from TECHNICAL_MANUAL.md (required by AGENTS.md for autonomous chaining protocol)

---

## Phase 2: Bridge Isolation & Build Pipeline

**Vision:** Split code into pure Rust core (no browser deps) and WASM bridge (interop only). This fixes `reqwest::blocking` WASM incompatibility and enables clean cross-compilation for Google Aura. Dual WASM build gives WebGPU for capable devices and WebGL2 fallback for rural homeschoolers with old laptops.

### Directory Restructure
- [ ] **2.1** Create `src/core/` directory
- [ ] **2.2** Create `src/bridge/` directory
- [ ] **2.3** Move all game logic files into `src/core/` (components, database, deck, input, render, quest, battle, chat, letter, save, etc.)
- [ ] **2.4** Move TTS code from `chat.rs` into `src/bridge/tts.rs` (the `reqwest` calls behind `tts` feature)
- [ ] **2.5** Create `src/bridge/mod.rs` — re-exports for WASM target
- [ ] **2.6** Create `src/core/mod.rs` — re-exports for all targets
- [ ] **2.7** Update `main.rs` module declarations for new structure
- [ ] **2.8** Update `lib.rs` module declarations for WASM target
- [ ] **2.9** Update `Cargo.toml` — `reqwest` only in bridge feature, not core
- [ ] **2.9d** **Quick fix (before full bridge split):** Feature-gate the entire TTS module with `#[cfg(not(target_arch = "wasm32"))]` since `reqwest::blocking` panics in WASM (single-threaded). Kokoro TTS is already disabled in WASM per GDD, so this is a zero-cost fix that unblocks WASM builds immediately without waiting for the full directory restructure.

### Async JSON Loading (WASM Performance)
- [ ] **2.9a** Transition embedded JSON databases from `include_str!` + synchronous `serde_json::from_str()` to async loading via Bevy `AssetServer` in a `GameState::Loading` state. Parsing 3.3MB synchronously on a Chromebook WASM build will freeze the browser's main thread, causing a black screen. Display a polished loading spinner during async parse.
- [ ] **2.9b** Add `LoadingScreen` UI — spinner + progress bar while databases parse
- [ ] **2.9c** Only transition from `GameState::Loading` to `GameState::Menu` after all 5 databases are loaded

### Verify Isolation
- [ ] **2.10** `cargo check` — verify core compiles without browser deps
- [ ] **2.11** `cargo check --target wasm32-unknown-unknown` — verify WASM compiles
- [ ] **2.12** `cargo ndk -t aarch64-linux-android check` — verify Android cross-compile works
- [ ] **2.13** Run `scripts/check_arch.py` — verify bridge isolation enforced

### Dual WASM Build
- [ ] **2.14** Create `build_wasm.sh` — build WebGL2 WASM binary (`trunk build --release`)
- [ ] **2.15** Add WebGPU WASM binary build to `build_wasm.sh` (second build with WebGPU feature)
- [ ] **2.16** Add `wasm-opt -Oz` optimization step to both binaries
- [ ] **2.17** Output both `.wasm` files to `dist/` directory
- [ ] **2.18** Update `index.html` — JavaScript to detect `navigator.gpu` and load correct binary
- [ ] **2.19** Test WASM build in browser — verify game loads and runs
- [ ] **2.20** Test on Chrome with WebGPU — verify WebGPU binary loads
- [ ] **2.21** Test on Firefox/older Chrome — verify WebGL2 fallback loads

### PWA & Offline
- [ ] **2.22** Create `manifest.json` — PWA manifest (name, icons, start_url, display: standalone, theme_color)
- [ ] **2.23** Create `service-worker.js` — cache WASM + assets for offline play
- [ ] **2.24** Register service worker in `index.html`
- [ ] **2.25** Test offline mode — load game, disconnect network, reload, verify still works

---

## Phase 3: Visual Polish

**Vision:** Make the game look professional, not like programmer art. Element-specific materials, smooth transitions, and performance presets for varying hardware.

### Element-Specific Materials
- [ ] **3.1** Fire material: emissive orange-red, low roughness (0.15), subtle flicker animation on emissive intensity
- [ ] **3.2** Water material: semi-transparent blue (alpha 0.7), very low roughness (0.05), alpha blend mode
- [ ] **3.3** Earth material: brown, high roughness (0.9), rocky feel (could use normal map later)
- [ ] **3.4** Air material: near-transparent white (alpha 0.3), low opacity, shimmer effect
- [ ] **3.5** Shadow material: dark purple, high metallic (1.0), Fresnel glow rim
- [ ] **3.6** Light material: bright yellow-white, high emissive, bloom-friendly
- [ ] **3.7** Normal material: neutral gray, medium roughness (0.5), no special effects
- [ ] **3.8** Apply element materials in `render.rs:spawn_avatar_visuals()` replacing flat `StandardMaterial`

### Fade Transitions
- [ ] **3.9** Create `FadeOverlay` resource: `{ alpha: f32, target_state: Option<GameState>, fading_out: bool, timer: f32 }`
- [ ] **3.10** Implement `transition_to_state()` — sets fade target, begins fade-out
- [ ] **3.11** Implement `update_fade()` system — lerps alpha 0→1, switches state at alpha=1.0, fades 1→0
- [ ] **3.12** Add `FadeOverlay` UI entity — full-screen black quad with alpha, rendered on top
- [ ] **3.13** Register fade systems in `main.rs` — run on every `Update`
- [ ] **3.14** Replace all direct `next_state.set()` calls with `transition_to_state()` for smooth transitions
- [ ] **3.15** Add fade duration config (0.3s out, 0.3s in = 0.6s total transition)

### Quality Presets
- [ ] **3.16** Create `QualityPreset` enum: `Low`, `Medium`, `High`, `Ultra`
- [ ] **3.17** Implement `apply_quality()` — configures MSAA (1x/2x/4x/8x), shadow settings, particle count, bloom intensity
- [ ] **3.18** Add quality auto-detection — check GPU vendor/renderer on startup, select preset
- [ ] **3.19** Add quality override in settings menu (if we have one, or add to main menu)
- [ ] **3.20** Low preset: no shadows, 5 aura particles, no bloom, 1x MSAA
- [ ] **3.21** Medium preset: simple shadows, 10 aura particles, mild bloom, 2x MSAA
- [ ] **3.22** High preset: soft shadows, 20 aura particles, bloom, 4x MSAA
- [ ] **3.23** Ultra preset: soft shadows, 30 aura particles, bloom + SSAO, 8x MSAA

### Data-Driven Prefabs
- [ ] **3.24** Create `assets/pet_prefabs.ron` — material/mesh definitions per element+class combo
- [ ] **3.25** Define prefab format: `{ element: Element, class: SummonClass, material: MaterialDef, mesh: MeshDef, decorations: Vec<DecorationDef> }`
- [ ] **3.26** Load prefabs in `render.rs:spawn_avatar_visuals()` instead of hardcoded materials
- [ ] **3.27** Add fallback to procedural if prefab missing for a combo

---

## Phase 4: Core Game Features (P0 — The Pokémon Moment)

**Vision:** The child spells a word, a card appears face-down, they flip it, and the pet bursts out. This is the emotional hook that makes the game addictive. Without this, the game is a tech demo, not a product.

### Pet Card Reveal
- [ ] **4.1** Create `PetCard` component: `{ word: String, flipped: bool, flip_timer: f32, rarity: Rarity }`
- [ ] **4.2** Define `Rarity` enum: `Common`, `Uncommon`, `Rare`, `Epic`, `Legendary`, `Mythic`
- [ ] **4.3** Implement `calculate_rarity()` — score from AoA + (5-concreteness) + word_length + root_rarity
- [ ] **4.4** Map score to tiers: <80 Common, <120 Uncommon, <180 Rare, <260 Epic, <380 Legendary, >=380 Mythic
- [ ] **4.5** Modify `submit_spelling_word()` — after validation, spawn `PetCard` entity (face-down card mesh) instead of immediately spawning 3D pet
- [ ] **4.6** Implement card flip animation — 0.5s rotation on Y axis, scale x from 1.0 → 0 → 1.0 (flip effect)
- [ ] **4.7** On flip complete — spawn 3D pet with burst particles, screen shake, sound effect
- [ ] **4.8** Card face-up shows: word text, element color border, role icon, stats panel, rarity tier glow
- [ ] **4.9** Card stays in world as pet's "home" — pet returns to card after battle/quest
- [ ] **4.10** Add rarity stat multiplier: Common 1.0x, Uncommon 1.15x, Rare 1.35x, Epic 1.6x, Legendary 2.0x, Mythic 2.5x
- [ ] **4.11** Display rarity on card with color border (gray/green/blue/purple/gold/rainbow)

### Pet Collection System
- [ ] **4.12** Create `PetCollection` resource: `{ pets: Vec<PetEntry> }` — replaces `SpellBook` as primary collection
- [ ] **4.13** Define `PetEntry` struct: `{ word, class, element, role, stats, faces_state, rarity, mastery, times_used, evolution_stage, first_seen }`
- [ ] **4.14** Add `PetCollection` to save data in `save.rs`
- [ ] **4.15** Create Pet Collection screen — grid of all collected pet cards, sortable
- [ ] **4.16** Add `GameState::Collection` or overlay on `Playing` state
- [ ] **4.17** Add sort buttons: by element, class, rarity, mastery, alphabetical, recent
- [ ] **4.18** Add pet detail view — click card to see full stats, FACES state, etymology, mastery progress
- [ ] **4.19** Add "Set as Companion" button in detail view
- [ ] **4.20** Add "Add to Roster" button in detail view
- [ ] **4.21** Add search/filter by element or grade level

### Roster Selection
- [ ] **4.22** Create `Roster` resource: `{ pets: Vec<usize> }` (indices into PetCollection, max 6)
- [ ] **4.23** Create roster selection UI — pick 3-6 pets from collection for battle
- [ ] **4.24** Reroute `start_battle()` to use roster pets instead of random deck draw
- [ ] **4.25** Reroute `play_battle_card()` to use roster pet's word
- [ ] **4.26** Add roster display in battle UI — show active roster pets as cards at bottom
- [ ] **4.27** Add roster swap mechanic — replace a fainted pet with another from collection
- [ ] **4.28** Add test: spawn pet → verify PetCollection has 1 entry → verify PetCard entity exists
- [ ] **4.29** Add test: roster selection → verify max 6 → verify battle uses roster words
- [ ] **4.30** Add test: rarity calculation → verify Common for easy word, Rare for hard word

---

## Phase 5: P1 Features (Depth & Engagement)

**Vision:** Add the systems that make kids want to come back. Rarity gives collecting meaning. Evolution gives mastery a visual reward. RPS makes combat strategic. Active learning makes combat educational. Color-coded quests make grammar visual. ASL makes VR spelling real.

### Visual Evolution
- [ ] **5.1** Define evolution stages per mastery level:
  - `Encountered` → basic pet (current visuals)
  - `Experienced` → element colors intensify, slight size increase
  - `Owned` → decoration meshes appear (element-specific ornaments)
  - `Mastered` → golden aura, particle flourish, +10% stat bonus
- [ ] **5.2** Add decoration meshes per element:
  - Fire: flame spikes on shoulders
  - Water: droplet ornaments orbiting
  - Earth: rock crystals embedded
  - Air: wind puffs trailing
  - Shadow: shadow tendrils wisping
  - Light: light rays radiating
- [ ] **5.3** Add golden aura particle effect for Mastered pets (20 extra particles, gold color)
- [ ] **5.4** Trigger visual upgrade in `render.rs` when mastery changes
- [ ] **5.5** Add evolution fanfare — particle burst + screen flash when pet evolves

### RPS Class Modifier
- [ ] **5.6** Add `rps_modifier(attacker: SummonClass, defender: SummonClass) -> f32` to `battle.rs`
- [ ] **5.7** Slime > Robot (+50%), Robot > Golem (+50%), Golem > Slime (+50%), Same class (1.0x), Disadvantage (-25%)
- [ ] **5.8** Apply modifier after class-specific combat calculation, before final damage
- [ ] **5.9** Add test: Slime attacks Robot → verify +50% modifier
- [ ] **5.10** Add test: Golem attacks Slime → verify +50% modifier
- [ ] **5.11** Add test: Slime attacks Golem → verify -25% penalty
- [ ] **5.12** Display RPS advantage indicator in battle UI ("WEAKNESS EXPLOITED!" or "RESISTANT!")

### Combat Feedback Floaters (Gemini Insight)
- [ ] **5.12a** Add visual "feedback floaters" during combat — when an attack lands, pop up UI text explaining the hidden semantic math: "Opposite Energy!" for high valence distance, "Too similar in Emotion!" for ineffective attacks, "Same Element — Resisted!" for element matches. A 10-year-old won't inherently understand why "Rock" and "Stone" is a critical hit but "Rock" and "Bird" is ineffective — these floaters bridge the gap.
- [ ] **5.12b** Style floaters with color coding: green for effective, red for ineffective, gold for critical
- [ ] **5.12c** Add floating animation — text rises and fades over 1.5s
- [ ] **5.12d** Add screen shake intensity proportional to damage dealt

### Active Combat Learning
- [ ] **5.13** Add battle sub-state enum: `BattlePhase::SelectCard`, `BattlePhase::SynonymChallenge`, `BattlePhase::AntonymChallenge`, `BattlePhase::EtymologyChallenge`, `BattlePhase::Result`
- [ ] **5.14** On card play, present synonym challenge: "Type a synonym of [your pet's word] to attack!"
- [ ] **5.15** Validate synonym against `SynonymDatabase` — if correct, damage lands with multiplier
- [ ] **5.16** If wrong, Typo counter-attacks for 20 HP
- [ ] **5.17** Add antonym challenge for defense: "Type an antonym of [enemy word] to block!"
- [ ] **5.18** Add etymology root challenge for critical hit: "What root does [word] come from?"
- [ ] **5.19** Add class-specific special moves:
  - Slime: Synonym Chain (chain 3 synonyms for combo damage)
  - Golem: Grammar Combo (form a sentence for massive damage)
  - Robot: Rhetorical Counter (argue against the typo's meaning)
- [ ] **5.20** Add battle UI text input field for typing answers
- [ ] **5.21** Add hint system — 3 hints available per battle (show first letter, show category, show answer)
- [ ] **5.22** Add test: synonym challenge correct → damage multiplied
- [ ] **5.23** Add test: synonym challenge wrong → player takes counter damage

### Color-Coded Quest Slots
- [ ] **5.24** Define slot colors: Orange=WHO(noun), Yellow=WHAT_DOING(verb), Green=WHAT(noun-object), Blue=WHERE(location), Purple=HOW(adverb)
- [ ] **5.25** Render quest slots as colored boxes in quest UI (both 2D and XR)
- [ ] **5.26** Validate part of speech when filling slot — reject mismatched cards with visual feedback (red shake animation)
- [ ] **5.27** Add part-of-speech detection — use FACES Container/Focus/Action mapping or simple heuristic from suffix
- [ ] **5.28** Add accepted card animation — card slides into slot with satisfying click
- [ ] **5.29** Add test: noun card into noun slot → accepted
- [ ] **5.30** Add test: verb card into noun slot → rejected

### Curriculum-Biased Letter Spawning (Bumped from P2 → P1 for demo)
**Gemini insight:** Without vowel/common-consonant weighting, a 1st grader playing the demo will get `X, Q, Z, J, P` and churn in 60 seconds. This must be P1, not P2.
- [ ] **5.42** Build letter frequency table from valid words at child's grade level
- [ ] **5.43** Weight random letter spawning by frequency table — common letters for grade appear more
- [ ] **5.44** Ensure vowels always available (minimum 40% vowel ratio in spawn pool)
- [ ] **5.45** Add "lucky letter" mechanic — rare letters spawn with golden glow occasionally
- [ ] **5.46** Recalculate frequency table when grade level changes

### ASL Fingerspelling (Full A-Z)
- [ ] **5.31** Research ASL hand shapes for all 26 letters (reference: ASL alphabet charts)
- [ ] **5.32** Define hand pose data for each letter: joint angles, finger extensions, thumb position
- [ ] **5.33** Expand `recognize_asl_letter()` from 2-letter stub to full A-Z
- [ ] **5.34** Add letter confidence scoring — return top 3 candidates with confidence values
- [ ] **5.35** Add letter confirmation UI — detected letter appears as floating text, pinch to confirm
- [ ] **5.36** Add spelling buffer display in XR — letters appear above altar as child signs
- [ ] **5.37** Add backspace gesture (shake hand left-right) to remove last letter
- [ ] **5.38** Add "submit" gesture (two-handed clap or specific pose) to submit word
- [ ] **5.39** Add ASL tutorial mode — show target letter, guide hand shape
- [ ] **5.40** Test ASL spelling on Google Aura hardware (if available)
- [ ] **5.41** Add test: mock hand poses for each letter → verify correct detection

---

## Phase 6: P2 Features (World & Life)

**Vision:** Make the world feel alive. Pets follow you. Letters chase you. The world adapts to your grade level. Mastered pets dream.

### Companion Follow System
- [ ] **6.1** Add `Companion` component to mark active companion pet
- [ ] **6.2** Implement follow behavior — pet lerps toward player position with offset (floating beside player)
- [ ] **6.3** Add companion idle animation — gentle bob, occasional look at player
- [ ] **6.4** Add "Set as Companion" button in pet collection screen
- [ ] **6.5** Only one companion at a time — setting new companion removes old
- [ ] **6.6** Companion reacts to events — happy bounce on word collection, alert pose on battle start

### Nuisance Letters
- [ ] **6.7** Add `NuisanceLetter` component with chase AI (move toward player at constant speed)
- [ ] **6.8** Spawn nuisance letters periodically during `Collecting` state
- [ ] **6.9** On catch: letter forced into `LetterStash` (can be helpful or annoying)
- [ ] **6.10** Rare letters (Z, X, Q) are valuable nuisances — glowing differently, slower chase
- [ ] **6.11** Shake-off mechanic — spelling a word quickly clears nuisance letters
- [ ] **6.12** Add nuisance spawn rate config — increases with grade level
- [ ] **6.13** Add visual distinction — nuisance letters have jagged edges, red tint

### Pet Dream Layer
- [ ] **6.19** Define dream poem templates per element ("from fire I rise...", "in shadow I dream...")
- [ ] **6.20** Create `dream_poems.json` or add to `lore_db.json`
- [ ] **6.21** Trigger dream state when pet is idle + mastered — floating text particles emit poetry
- [ ] **6.22** Dream poems incorporate the pet's word and etymology root
- [ ] **6.23** Add visual dream effect — soft purple haze, closed eyes, gentle sway
- [ ] **6.24** Add "listen" interaction — pinch sleeping pet to hear dream poem via TTS

---

## Phase 7: Demo Preparation & Ship

**Vision:** Ship a polished 10-word demo to itch.io that makes homeschool parents say "my kid needs this." The demo is the marketing funnel for the $9.99 full version.

### Demo Content
- [ ] **7.1** Curate 10 demo words — mix of elements, classes, grades:
  - 2 Fire element words (different roles)
  - 2 Water element words
  - 2 Earth element words
  - 1 Air, 1 Shadow, 1 Light, 1 Normal
  - Mix of SummonClasses (3 Slime, 4 Golem, 3 Robot)
  - Mostly K-2 and 3-5 for accessibility
  - At least one dramatic FACES expression
- [ ] **7.2** Implement demo word filtering in `paywall.rs` — only 10 words valid in demo mode
- [ ] **7.3** Add demo word list to `DemoSettings` resource
- [ ] **7.4** Verify demo blocks saving (already implemented — test it)
- [ ] **7.5** Add "Get Full Version" prompt after first battle or quest completion
- [ ] **7.6** Design "Get Full Version" overlay UI — text + link to itch.io full version page
- [ ] **7.7** Add demo timer — after 5 minutes of play, show "Enjoying the game? Get the full version!" prompt

### itch.io Packaging
- [ ] **7.8** Review and update `itch_page.md` copy
- [ ] **7.9** Create itch.io cover image (1280x720)
- [ ] **7.10** Create 3-5 gameplay screenshots
- [ ] **7.11** Create animated GIF of pet card reveal (the Pokéball moment)
- [ ] **7.12** Package WASM + assets + index.html into ZIP for itch.io upload
- [ ] **7.13** Upload to itch.io as WebGL game
- [ ] **7.14** Set itch.io pricing: demo free, full version $9.99

### Demo Testing
- [ ] **7.15** Test demo in browser — verify 10-word limit, no save, full version prompt
- [ ] **7.16** Test on slow connection — verify load time acceptable with `wasm-opt -Oz`
- [ ] **7.17** Test on old laptop — verify WebGL2 fallback works
- [ ] **7.18** Test on mobile browser — verify touch controls work
- [ ] **7.19** Test full playthrough: menu → tutorial → spell → card reveal → battle → quest → paywall prompt
- [ ] **7.20** Verify all 8 integration tests pass
- [ ] **7.21** Verify zero compiler warnings

---

## Phase 8: Post-Demo — Full Game

**Vision:** The demo proves the concept. Now build the full product.

### Full Game Unlock
- [ ] **8.1** Remove 10-word limit for paid version
- [ ] **8.2** Implement license key or itch.io entitlement check
- [ ] **8.3** Verify all 9,582 words accessible in paid mode
- [ ] **8.4** Verify all 12 NPCs with quest chains accessible
- [ ] **8.5** Verify save/load works in paid mode
- [ ] **8.6** Verify all 12 districts unlock progressively

### Google Aura XR Build
- [ ] **8.7** Test `cargo ndk -t aarch64-linux-android check --features xr`
- [ ] **8.8** Resolve any Android NDK compilation issues
- [ ] **8.9** Configure `AndroidManifest.xml` for Google Aura (XR permissions, spatial computing flags)
- [ ] **8.10** Test OpenXR initialization on Aura emulator or hardware
- [ ] **8.11** Replace simulated hand tracking positions with real OpenXR joint data
- [ ] **8.12** Test ASL fingerspelling with real hand tracking
- [ ] **8.13** Test spatial UI panels in XR — verify readable at Aura resolution
- [ ] **8.14** Test pinch interaction precision on Aura
- [ ] **8.15** Build APK and test on device

### Expansion Pack System
- [ ] **8.16** Define expansion pack format — JSON with word list + etymology overrides + quest chains
- [ ] **8.17** Add expansion pack loader to `database.rs`
- [ ] **8.18** Create first expansion: "SAT Prep Pack" (500 advanced words)
- [ ] **8.19** Create second expansion: "Science Vocabulary Pack" (300 science terms)
- [ ] **8.20** Create third expansion: "Spanish-English Bridge Pack" (200 cognates)
- [ ] **8.21** Add expansion pack selection in main menu
- [ ] **8.22** Add expansion pack store/activation UI

### Parent Dashboard
- [ ] **8.23** Design parent dashboard as separate web app (React or simple HTML+JS)
- [ ] **8.24** Create `save.json` parser — reads save file, displays analytics
- [ ] **8.25** Display: words learned, mastery levels, attunement profile, time spent, struggling areas
- [ ] **8.26** Add progress charts (words per week, mastery distribution)
- [ ] **8.27** Add "recommended words" section — suggest words child hasn't tried
- [ ] **8.28** Add export to PDF for homeschool portfolio documentation
- [ ] **8.29** Deploy as standalone web app (separate from game)

---

## Phase 9: Post-Demo — SpawnForge Collaboration

**Vision:** Share our innovations with the Bevy ecosystem and adopt SpawnForge's best patterns.

### Shared Crates
- [ ] **9.1** Extract `faces-protocol` as standalone crate on crates.io
- [ ] **9.2** Add comprehensive docs and examples to `faces-protocol`
- [ ] **9.3** Extract psycholinguistic database as `psycholinguistic-data` crate
- [ ] **9.4** Add data validation and query API to `psycholinguistic-data` crate
- [ ] **9.5** Extract etymology → element mapping as reusable crate
- [ ] **9.6** Contact Tristan578 with GDD.md and TECHNICAL_MANUAL.md
- [ ] **9.7** Propose shared `bevy-wasm-build` crate for dual WASM pipeline
- [ ] **9.8** Propose shared `bevy-arch-validator` crate from `check_arch.py`

### Three-Tier Pet Generation (Tier 2 & 3)
- [ ] **9.9** Run all 9,582 words through FACES hash to group by visual archetype
- [ ] **9.10** Identify top ~500 visual archetypes for pregeneration
- [ ] **9.11** Set up offline Janus Pro + Trellis pipeline for glTF generation
- [ ] **9.12** Generate polished `.glb` models for top 500 archetypes
- [ ] **9.13** Name files `assets/pets/{archetype}.glb` (code at `render.rs:267` auto-loads)
- [ ] **9.14** Verify glTF models load correctly in-game
- [ ] **9.15** Design Pet Studio mode (paid desktop feature):
  - [ ] **9.16** Accessory generation UI (hats, armor, capes, belts)
  - [ ] **9.17** Aura effect customization (fire trail, ice mist, shadow tendrils)
  - [ ] **9.18** Companion creature generation (mini-pets)
  - [ ] **9.19** Save accessories as `.glb` overlays on pet
  - [ ] **9.20** Embed Janus Pro 1B quantized for desktop only (not WASM/mobile)

---

## Phase 10: Future / Advanced Polish

**Vision:** Long-term improvements that make the game world-class.

### Advanced Rendering
- [ ] **10.1** Custom WGSL shaders per element (fire flicker, water refraction, earth crackle, air distortion, shadow dissolve, light bloom)
- [ ] **10.2** GPU compute particles for pet auras (replace CPU-side particle system)
- [ ] **10.3** SSAO on desktop and WebGPU targets
- [ ] **10.4** Color grading post-processing per district (warm tones for Fire district, cool for Water, etc.)
- [ ] **10.5** Dynamic weather effects per district (embers, rain, dust, wind, fog, light rays)

### Advanced Features
- [ ] **10.6** Visual quest builder for teachers (React Flow or in-game tool)
- [ ] **10.7** Multi-tool AI support (Claude Code, Copilot, Gemini CLI, beyond Windsurf)
- [ ] **10.8** Joint `bevy-edu` meta-crate combining FACES + psycholinguistic data + etymology + WASM tools
- [ ] **10.9** Multiplayer classroom mode (teacher spawns words, students collect and battle)
- [ ] **10.10** Voice recognition for spelling (alternative to keyboard/ASL)
- [ ] **10.11** Adaptive difficulty — AI tracks struggling patterns and adjusts word pool
- [ ] **10.12** Story mode — overarching narrative across 12 districts

### Audio
- [ ] **10.13** Spatial 3D audio system (positional sound for pets, letters, UI)
- [ ] **10.14** Adaptive music — changes per district and battle intensity
- [ ] **10.15** Pet voice synthesis — each pet "speaks" with FACES-driven voice modulation
- [ ] **10.16** Sound effects: card flip, pet spawn burst, battle hit, quest complete, level up
- [ ] **10.17** Ambient soundscapes per district

---

## Summary Statistics

| Phase | Task Count | Priority |
|-------|-----------|----------|
| Phase 0: Safety | 18 | **Critical — do first** |
| Phase 1: Architecture | 25 | High |
| Phase 2: Bridge & Build | 29 | High |
| Phase 3: Visual Polish | 27 | Medium |
| Phase 4: Core Features (P0) | 30 | **Critical for demo** |
| Phase 5: P1 Features | 50 | High |
| Phase 6: P2 Features | 19 | Medium |
| Phase 7: Demo Ship | 21 | **Critical for launch** |
| Phase 8: Full Game | 29 | Post-demo |
| Phase 9: Collaboration | 20 | Post-demo |
| Phase 10: Future | 17 | Long-term |
| **Total** | **285 tasks** | |

---

*This document is the complete work inventory for Communication Class. Update task status as work progresses. When this document and the GDD disagree on priorities, the GDD is the authority on what to build; this document is the authority on how to build it.*

*Last updated: July 2026*
