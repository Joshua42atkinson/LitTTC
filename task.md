# LitTCG — Autonomous Task Tracker

> **This tracker is now superseded by `TODO_FULL_PLAN.md`**, a current, comprehensive plan built from a code-and-doc audit. The old content below is archived for reference.
> Active work is tracked in `TODO_FULL_PLAN.md`.

## Workflow
1. Read `TODO_FULL_PLAN.md` for the active phase and next unchecked task.
2. Execute one task per turn; run `cargo test` and `cargo clippy` after each change.
3. Update checkmarks in `TODO_FULL_PLAN.md` as tasks complete.
4. Morning review: `git log --oneline -10` + `cargo test`

---

## Phase 0: Safety ✅ COMPLETE
- [x] Rename arousal→intensity (serde alias for backward compat)
- [x] Profanity blocklist module + integration in letter.rs
- [x] Clean all compiler warnings (zero warnings)
- [x] cargo test (19/19 pass) + cargo check (0 warnings)

---

## Phase 1: Command-Driven Architecture (Windsurf — complex reasoning)

### P1.1: Create GameCommand enum ✅
- [x] **Windsurf task** — Create `src/commands.rs` with GameCommand enum
  - Enum variants: SubmitSpelling, SelectCard, PlayCard, SkipCard, StartBattle,
    PlayBattleCard, FleeBattle, StartQuest, FillQuestSlot, CompleteQuest,
    CancelQuest, Swipe, DismissReview, NewGame, ContinueGame, OpenSettings, TransitionTo
  - Register as Bevy Message: `app.add_message::<GameCommand>()` (Bevy 0.18 uses Messages, not Events)
  - Add `mod commands;` to both lib.rs and main.rs
  - Write integration test: messages sent → messages received
  - 22 tests passing, 0 warnings

### P1.2: Create command handler system ✅
- [x] **Windsurf task** — Create `fn handle_game_commands` system
  - Reads `MessageReader<GameCommand>` and dispatches to existing logic
  - This is the bridge between input systems and game logic
  - Does NOT replace input systems yet — just receives messages
  - Resources grouped into tuples to stay within Bevy's system-parameter limit
  - Optional `AssetServer` / `Assets<Mesh>` / `Assets<StandardMaterial>` so tests don't need full render stack
  - Made `letter::submit_spelling_word` pub and use plain references for broader reuse
  - 24 tests passing, 0 warnings

### P1.3: Reroute input to commands ✅
- [x] **Windsurf task** — Refactor input systems to send GameCommand messages
  - `handle_ui_button_interactions` now sends SelectCard, PlayCard, StartQuest, StartBattle, FleeBattle, CancelQuest
  - `handle_keyboard_spelling` / `handle_vr_spelling` send AddLetter, Backspace, SubmitSpelling
  - `drag_end` / `keyboard_input` send Swipe
  - `keyboard_quest_interaction` / `vr_quest_interaction` send CompleteQuest, FillQuestSlot
  - `keyboard_battle_interaction` / `vr_battle_interaction` send PlayBattleCard
  - `review_input_system` sends DismissReview
  - `menu_interaction` sends NewGame, ContinueGame, OpenSettings
  - Removed unused `PendingSwipe` resource
  - All input systems ordered `.before(commands::handle_game_commands)`
  - 24 tests passing, 0 warnings

---

## Phase 1.5: Mechanical Tasks (Claude Code local — overnight)

### P1.5.1: Add GameState transitions logging
- [x] Add `info!()` log statements at every `next_state.set()` call
  - Search for `next_state.set` in all .rs files
  - Add: `info!("State transition: {:?} → {:?}", current_state, new_state);`
  - Run cargo test to verify
  - Commit: "P1.5.1: Add state transition logging"

### P1.5.2: Add input action logging
- [x] Add `info!()` to all input handlers in input.rs
  - Log swipe direction when detected
  - Log card selection when made
  - Log button presses when triggered
  - Run cargo test to verify
  - Commit: "P1.5.2: Add input action logging"

### P1.5.3: Create .gitignore for build artifacts
- [x] Add/Update `.gitignore` in LitTCG/
  - Ignore: /target, /dist, *.wasm, *.js (in dist), overnight.log
  - Keep: src/, assets/, tests/, Cargo.toml, Cargo.lock
  - Commit: "P1.5.3: Update .gitignore for build artifacts"

### P1.5.4: Add doc comments to all public functions
- [x] Add `///` doc comments to all `pub fn` in:
  - src/database.rs (load_from_embedded, etc.)
  - src/deck.rs (shuffle, draw, etc.)
  - src/save.rs (save, load, etc.)
  - src/blocklist.rs (is_banned, is_clean, etc.)
  - Do NOT add comments to functions that already have them
  - Run cargo test to verify
  - Commit: "P1.5.4: Add doc comments to public functions"

### P1.5.5: Add clippy lint annotations
- [x] Run `cargo clippy` and fix all warnings
  - Add `#![warn(clippy::all)]` to lib.rs
  - Fix any clippy warnings that appear
  - Run cargo test to verify all tests still pass
  - Commit: "P1.5.5: Enable clippy lints and fix warnings"

### P1.5.6: Extract magic numbers to constants
- [x] Find hardcoded numeric literals in:
  - src/letter.rs: pet spawn position (0.0, 1.5, -2.0)
  - src/letter.rs: stat multipliers (20.0, 10.0, 10.0, 10.0)
  - src/battle.rs: damage multipliers (2.5, 1.5, 0.75)
  - src/render.rs: particle counts (10, 20)
  - Extract to `const PET_SPAWN_POSITION: Vec3 = ...` etc.
  - Put constants at top of each file after imports
  - Run cargo test to verify
  - Commit: "P1.5.6: Extract magic numbers to named constants"

---

## Phase 2: Bridge Isolation & WASM (Windsurf — architecture)

### P2.1: Feature-gate TTS for WASM
- [x] **Windsurf task** — Gate `chat.rs` Kokoro TTS behind `#[cfg(not(target_arch = "wasm32"))]`

### P2.2: Directory split
- [ ] **Windsurf task** — Split into src/core/ + src/bridge/

### P2.3: WASM build script
- [x] **Windsurf task** — Create build_wasm.sh with trunk + wasm-opt

---

## Phase 3-4: Visual Polish + Pokémon Moment (Later)

### P3.1: Element-specific materials
- [ ] **Windsurf task** — 7 material presets

### P3.2: Pet card reveal animation
- [ ] **Windsurf task** — THE emotional hook — card flip + pet burst

### P3.3: Pet collection screen
- [ ] **Windsurf task** — Browse all collected pets as cards

---

## Post-Sprint Phases (Decompose Later)
- [ ] Phase 5: P1 Features (50 tasks)
- [ ] Phase 6: P2 Features (19 tasks)
- [ ] Phase 7: Demo Ship (21 tasks)
- [ ] Phase 8: Full Game (29 tasks)
- [ ] Phase 9: Collaboration (20 tasks)
- [ ] Phase 10: Future (17 tasks)

---

## Phase 1.6: Marketability & Code Quality Hardening (from full code review)

Quantitative improvements to harden the engine and align every code-level string/identifier with the brand voice before demo ship.

- [x] Reduce `main.rs` from 682 lines to <150 lines by extracting 6 systems (database loading, VR/2D review UI, VR interactions, frame diagnostics, world setup, initialization) into modules
- [x] Complete `lib.rs` Android entry: add 10 missing plugins (`HudPlugin`, `MenuPlugin`, `TutorialPlugin`, `PaywallPlugin`, `TimeCyclePlugin`, `SpatialUiPlugin`, `SpatialDeckPlugin`, `AltarPlugin`, `DialogueUiPlugin`, `DatabasePlugin`)
- [x] Remove 10 module-level `#![allow(dead_code)]` suppressions and fix underlying unused code
- [x] Refactor 8 `#[allow(clippy::...)]` suppressions (6 `type_complexity` + 2 `too_many_arguments`) by splitting systems or grouping parameters
- [x] Replace 3 `.expect()` calls in `main.rs` database loading with `warn!` + graceful degradation
- [x] Fix 4 incorrect state transitions / hardcoded behaviors:
  - `altar.rs` transitions to `Reviewing` instead of submitting the spell
  - `deck.rs` sends to `Reviewing` on empty hand
  - `paywall.rs` hardcodes "10 words" instead of using `DemoSettings.max_words`
  - `commands.rs` `NewGame` deletes `save.json` without confirmation
- [x] Rename 3 educational identifiers to brand voice: `StudentTrail` → `WordTrail`, `CurriculumManager` → `GradeManager`, expose `active_grade` as "Rank" in UI
- [x] Standardize 25 log/UI strings to brand voice (replace "Meme Template", "educational game" in comments, inconsistent grade/district labels, etc.)
- [x] Add 15 regression tests for edge cases (empty hand, invalid word, missing NPC, blocked word, TTS fallback, quest completion with empty slots, etc.)
- [x] Replace 12 hardcoded asset paths (`ui/quest_board.png`, `ui/card_background.png`, `textures/avatars/barnaby.png`, `sounds/*.ogg`, JSON DB paths) with constants or asset catalog entries
- [x] Implement 4 missing stubs: difficulty menu, settings screen, pet collection screen, 2D mode (`flat2d` feature)
- [x] Fix hand tracking ASL recognition to support 26 letters instead of 2 (`A`/`L` only)
- [x] Resolve 1 incorrect touch-input coordinate mapping in `input.rs` (screen coordinates written to world translation)

Verification:
- [x] `cargo test` passes (target: 30+ tests)
- [x] `cargo clippy --features desktop` passes with 0 warnings
- [x] `cargo check --features xr` passes
- [x] No `#![allow(dead_code)]` or `#[allow(clippy::...)]` suppressions remain

---

## Turbo Test Session (Claude Code — test expansion)

Added unit and integration tests across the engine. Test count: 39 → 128 (48 unit + 32 integration, duplicated across lib/bin targets).

### Unit tests added
- [x] `asset_catalog::tests` — constants exist, embedded JSON valid
- [x] `database::tests` — parse words/synonyms, malformed entry skipping, embedded load
- [x] `deck::tests` — `Hand`/`Deck` defaults
- [x] `save::tests` — `SaveData` JSON roundtrip
- [x] `time_cycle::tests` — phase defaults and progression order
- [x] `paywall::tests` — demo settings default
- [x] `quest::tests` — slot filling, display sentence, `GradeManager` rank logic
- [x] `battle::tests` — semantic distance, `BattleResult` defaults
- [x] `letter::tests` — `key_to_char` mapping, stash/spelling defaults
- [x] `chat::tests` — `ChatLog` cap, pet dialogue formatting
- [x] `hand_tracking::tests` — refactored `recognize_asl_letter` to pure function; default state, fist→A, open hand→M, missing wrist→None
- [x] `input::tests` — `DragState` defaults, swipe choice logic
- [x] `components::tests` — `CharacterSheet` attunement/class, `SpellBook` encounter/mastery, `WordTrail` defaults

### Integration tests added
- [x] Settings/Difficulty/PetCollection command state transitions
- [x] Complete quest flow awards XP
- [x] Ineffective battle word damages player

### Verification
- [x] `cargo test` passes — 128 tests
- [x] `cargo clippy --features desktop` passes with 0 warnings
- [x] `cargo check --features xr` passes
- [x] `cargo check --features flat2d` passes

---

## Phase 3: Demo Sprint — Ship a Playable Web Demo (30 Days)

> Goal: A public web demo your wife and early customers can open in a browser and understand in 60 seconds.

### Week 1: Pokémon Moment + Web Stability

- [x] **P3.2 Pet card reveal animation** — THE emotional hook
  - [x] `RevealingPet` game state added
  - [x] `src/pet_reveal.rs` plugin with `PendingReveal`, `RevealCard`, `RevealConfig`, `RevealSounds`
  - [x] Face-down card appears after word submit (3D desktop)
  - [x] Card flips with particles and sound
  - [x] Pet spawns at end of reveal and transitions back to `Playing`
  - [x] Shared `PET_SPAWN_POSITION` constant in `components.rs`
  - [x] `PetRevealPlugin` registered in Android `lib.rs` builds
  - [x] Integration test: valid word transitions through `RevealingPet` to `Playing` and spawns a `PetAvatar`
  - [x] 2D `flat2d` card/sprite variant with Sprite flip and particles
  - [x] Floating rarity/element/name label above the revealed pet

- [x] **P2.2 Directory split** — Split `src/` into `core/` + `bridge/` for clean WASM interop
  - [x] Moved all game logic modules into `src/core/`.
  - [x] Extracted `reqwest` Kokoro TTS client into `src/bridge/tts_client.rs`.
  - [x] Extracted `web_sys` purchase URL opener into `src/bridge/url_opener.rs`.
  - [x] Verified `cargo test`, `cargo check --features flat2d`, and `cargo check --features xr`.

- [x] **P3.4 Async JSON loading** — Prevent web demo freeze on startup
  - [x] Load databases asynchronously with a progress bar.
  - [x] Add loading screen with "Summoning vocabulary..." brand voice.
  - [x] Loading UI registered in desktop and Android entry points.

- [x] **P3.5 Demo limit + paywall polish**
  - [x] Enforce 10-word demo limit using `DemoSettings.max_words` and `words_used` counter.
  - [x] Show paywall with "Unlock Full Game — $9.99" CTA.
  - [x] Purchase button opens checkout URL on web (logs URL on desktop/Android).

### Week 2: Visual + Mobile Polish

- [x] **P3.1 Element-specific materials**
  - [x] 7 element material presets (Fire, Water, Earth, Air, Shadow, Light, Normal) via `Element::material()`.
  - [x] Distinct emissive, metallic, and roughness values per element.
  - [x] Reveal card front and spawned pet now use the element-specific PBR material.

- [x] **P3.6 Touch-first UI**
  - [x] Main menu buttons enlarged (280x70) and spaced for touch.
  - [x] Letter crystals enlarged to 0.5 with 1.8 pickup distance; XR holographic letters + submit button larger.
  - [x] HUD scaled up: larger fonts, bigger action buttons, larger hand cards and progress bar.

- [x] **P3.3 Pet collection screen polish**
  - [x] `SpellBookEntry` extended with `element`, `role`, `stats`, and `companion` fields.
  - [x] Pet collection grid with Word / Element / Mastery sorting.
  - [x] Detail panel shows element, role, mastery, times encountered, and stats.
  - [x] Set Companion button marks a pet as the player's companion.

- [x] **P3.7 Settings / difficulty fully wired**
  - [x] Settings: TTS toggle, hints toggle, reset save button.
  - [x] `GameSettings` resource with `sound_volume`, `music_volume`, `tts_enabled`, `hints_enabled`.
  - [x] Settings save/load to `settings.json` (desktop path; web/Android bridge later).
  - [x] Difficulty screen sets `GradeManager.active_grade` which controls valid word grade ranges.

### Week 3: Parent + Storefront

- [x] **P3.8 Parent progress report**
  - [x] Static HTML dashboard `parent_report.html` reads uploaded `save.json`.
  - [x] Shows rank, class, total XP, words learned, favorite element, recent words.
  - [x] Generates a conversation prompt from recent play.

- [x] **P3.9 Landing page**
  - [x] One-page site `landing_page.html` with hero, tagline, CTA buttons, and features grid.
  - [x] "Play the Demo" and "Get the Full Game" buttons.
  - [x] Copy aimed at homeschool parents.

- [x] **P3.10 Storefront setup**
  - [x] `itch_page.md` has copy, pricing ($9.99), platforms, and purchase CTA.
  - [x] Paywall purchase button links to `https://polar.sh/your-product`.

### Week 4: Test + Ship

- [/] **P3.11 Internal testing**
  - [x] Run `cargo test`, `cargo clippy`, `cargo check --features xr`.
  - [x] `trunk build` succeeds for WASM.
  - [ ] Test web demo on Chromebook, Android phone, desktop.
  - [ ] Fix any critical bugs in the happy path.

- [ ] **P3.12 Beta families**
  - Share demo with 3–5 homeschool families or friends.
  - Collect feedback on first 5 minutes and pet reveal.
  - Iterate on biggest friction points.

- [ ] **P3.13 Public demo deploy**
  - Deploy web demo to itch.io and/or your domain.
  - Post in 2–3 homeschool forums or groups.
  - Record a 60-second demo video.

### Definition of Done

- [/] Player can open the demo in a browser and play without instructions.
- [x] Spelling a word produces a visible, unique pet within 3 seconds.
- [x] Demo ends naturally after 10 words with a clear purchase path.
- [x] Parent can view a simple progress report.
- [ ] Demo runs on a mid-range Chromebook at 30+ FPS.
- [ ] Wife understands the game after 60 seconds.

### Verification

- [/] `cargo test` passes — 81 total tests (48 unit + 33 integration; target 128+)
- [x] `cargo clippy --features desktop` passes with 0 warnings
- [x] `cargo check --features xr` passes
- [x] `cargo check --features flat2d` passes
- [x] `trunk build` succeeds (WASM target compiles)
- [ ] `trunk serve` tested in Chrome
- [ ] No critical bugs in the happy path
