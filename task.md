# LitTTC — Autonomous Task Tracker

> Single source of truth for Windsurf (orchestrator) + Claude Code (executor).
> Synced with MASTER_TASK_LIST.md (285 tasks across 10 phases).

## Workflow
1. Windsurf plans and writes detailed task specs below
2. Claude Code local executes via `./scripts/agent-loop.sh`
3. Both read/write this file — tasks checked off as completed
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
- [ ] Add `info!()` to all input handlers in input.rs
  - Log swipe direction when detected
  - Log card selection when made
  - Log button presses when triggered
  - Run cargo test to verify
  - Commit: "P1.5.2: Add input action logging"

### P1.5.3: Create .gitignore for build artifacts
- [ ] Add/Update `.gitignore` in LitTTC/
  - Ignore: /target, /dist, *.wasm, *.js (in dist), overnight.log
  - Keep: src/, assets/, tests/, Cargo.toml, Cargo.lock
  - Commit: "P1.5.3: Update .gitignore for build artifacts"

### P1.5.4: Add doc comments to all public functions
- [ ] Add `///` doc comments to all `pub fn` in:
  - src/database.rs (load_from_embedded, etc.)
  - src/deck.rs (shuffle, draw, etc.)
  - src/save.rs (save, load, etc.)
  - src/blocklist.rs (is_banned, is_clean, etc.)
  - Do NOT add comments to functions that already have them
  - Run cargo test to verify
  - Commit: "P1.5.4: Add doc comments to public functions"

### P1.5.5: Add clippy lint annotations
- [ ] Run `cargo clippy` and fix all warnings
  - Add `#![warn(clippy::all)]` to lib.rs
  - Fix any clippy warnings that appear
  - Run cargo test to verify all tests still pass
  - Commit: "P1.5.5: Enable clippy lints and fix warnings"

### P1.5.6: Extract magic numbers to constants
- [ ] Find hardcoded numeric literals in:
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
- [ ] **Windsurf task** — Gate `chat.rs` Kokoro TTS behind `#[cfg(not(target_arch = "wasm32"))]`

### P2.2: Directory split
- [ ] **Windsurf task** — Split into src/core/ + src/bridge/

### P2.3: WASM build script
- [ ] **Windsurf task** — Create build_wasm.sh with trunk + wasm-opt

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
