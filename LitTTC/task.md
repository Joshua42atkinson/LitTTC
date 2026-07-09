# LitTCG — Autonomous Task Tracker

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

---

## Diapers Mode: 2D flat2d Screen-by-Screen E2E Playthrough

> Goal: Every screen reachable and functional in the flat2d build. Use the temporary `LITTCG_AUTOPLAY=1` harness to capture screenshots and verify transitions.

### Setup
- [x] Auto-play screenshot harness exists (`main.rs` `autoplay_system`) and captures `flat2d_autoplay_<State>.png`.
- [x] `cargo test` passes.
- [x] `cargo check --features flat2d` passes with 0 warnings.

### Core Loop (happy path)
- [x] **Loading** — database loads, progress bar visible, transitions to MainMenu.
- [x] **MainMenu** — clean UI, New Game/Continue/Settings/Difficulty/Pet Collection buttons work.
- [x] **Collecting** → **Constructing** — flat2d skips crystal collection and opens the 2D Constructing panel.
- [x] **Constructing** — stash visible, typing works (keyboard + buttons), Submit/Backspace work.
- [x] **RevealingPet** — valid word triggers 2D card flip + particles + pet sprite.
- [x] **Playing** — pet visible, HUD stats/cards/action buttons render, no 3D overlays.
- [x] **Questing** — clicking "Talk (Quest)" opens a 2D quest UI, can fill slots and complete a quest.
- [x] **Battling** — clicking "Explore (Battle)" opens a 2D battle UI, can play cards and resolve battle.
- [x] **Reviewing** — victory/defeat review panel shows, Enter/click dismisses and returns to Playing.

### Secondary screens
- [x] **Settings** — reachable from MainMenu, toggles apply, save/load works.
- [x] **Difficulty** — reachable from MainMenu, grade selection works.
- [x] **PetCollection** — reachable from MainMenu, shows collected pets, companion button works.
- [x] **Paywall** — reachable after 10 words in demo; verified with `LITTCG_AUTOPLAY_PAYWALL=1`.
- [x] **ContinueGame** — loads `save.json` and continues from `Collecting`; verified with `LITTCG_AUTOPLAY_CONTINUE=1` and a test save file.

### Exit criteria
- [x] Run `cargo test` and `cargo check --features flat2d` with 0 warnings.
- [x] Provide a list of "works / doesn't work / needs polish" per screen.

### What works / what doesn't / needs polish
- **Loading / MainMenu / Constructing / RevealingPet / Playing**: Works. Clean 2D UI, no 3D overlays.
- **Questing**: Works. Quest panel opens, cards fill slots, completes and returns to Playing.
- **Battling**: Works. Battle HUD shows, playing cards resolves combat, transitions to Reviewing.
- **Reviewing**: Works. Victory/defeat panel shows, dismisses to Playing.
- **Settings / Difficulty / PetCollection**: Works. Reachable from MainMenu, HUD hidden, return to MainMenu works.
- **Paywall**: Works. Demo limit triggers after 10 words; paywall UI shows purchase CTA and return-to-menu button.
- **ContinueGame**: Works. Loads `save.json` and continues the session from `Collecting`.
- **Polish gaps**: placeholder pet sprite (colored square), no card art, no enemy sprite in battle, quest/battle text overflows, review panel shows action buttons behind it.

---

## 2D Gray-Box Vertical Slice — Implementation Plan

> Goal: Turn the screen-by-screen flat2d build into a small Pokémon-style overworld RPG that validates the Thesaurus Dance, FACES emotional stance, and literary-device combos before any XR work.

### Phase A — 2D Overworld Skeleton
- [x] Add an `Exploring` game state (or repurpose `Playing`) as the main 2D field.
- [x] Create `src/core/overworld.rs` module gated behind `#[cfg(feature = "flat2d")]`.
- [x] Spawn a 2D orthographic camera and a bounded world plane.
- [x] Spawn the player avatar as a colored `Sprite` / `Mesh2d` rectangle.
- [x] Implement WASD / arrow-key movement for the avatar.
- [x] Implement a smooth camera follow system (copy from official `2d_top_down_camera.rs`).
- [x] Add simple world bounds so the avatar cannot walk off the map.
- [x] Keep 3D companion disabled in flat2d; spawn a 2D companion sprite that follows the avatar.
- [x] Wire `MainMenu → New Game` to enter `Exploring` instead of `Collecting`.
- [x] Hide the old HUD action buttons during `Exploring`.

### Phase B — World Entities
- [x] Add `ScannableObject` component with a word label (e.g., "rock", "tree", "river").
- [/] Add an interaction radius check and prompt: *"Press E to scan"* — radius check implemented; visual prompt pending.
- [x] On scan, transition to `Constructing` with a stash seeded from the object's word.
- [x] After a valid spelling, transition back to `Exploring` and add the card to the deck.
- [x] Add `NpcEntity` component with an `npc_name` mapping to `lore_db.json`.
- [/] On interact, show a dialogue panel and offer a quest — quest starts directly; dedicated dialogue panel pending.
- [x] Add `WildTypoEntity` component that roams or stands in place.
- [x] On avatar-typo overlap, transition to `Battling`.
- [x] Add 2-3 districts using background color/rect zones (Garden, Shadow Library, Irony Junction).

### Phase C — Thesaurus Dance Combat
- [x] Replace the current single-card battle with the sentence-crafting system.
- [x] Add `Plot` resource with max length 3.
- [x] Add a live sentence preview + weakness hint in the 2D battle screen.
- [/] Add an `AltarDropZone` or equivalent hand/altar UI in the 2D battle screen.
- [x] Show the enemy word and its weakness hint ("weak to antonyms / verbs").
- [ ] Render hand cards with word + part-of-speech + synonym hint.
- [ ] Allow clicking a card to add it to the Plot; allow removing cards.
- [x] Show the constructed sentence preview: "[searing] [sword] [strikes]".
- [ ] Add a "CAST SPELL" button that resolves the Plot.
- [ ] Implement `detect_literary_devices(plot: &Plot) -> Vec<LiteraryDevice>`:
  - [ ] Alliteration
  - [ ] Oxymoron
  - [ ] Hyperbole
  - [ ] Palindrome
  - [ ] Personification (stretch)
  - [ ] Onomatopoeia (stretch)
- [ ] Implement FACES face selection buttons in battle:
  - [ ] Fierce: +20% damage, fire verbs blast
  - [ ] Joyful: slight heal, group heals
  - [ ] Calm: no hyperbole recoil, +block
  - [ ] Angry: +30% damage, self-damage recoil
- [ ] Update `play_battle_card` (or replace with `cast_plot_spell`) to:
  - [ ] Compute base damage from POS multipliers.
  - [ ] Apply synonym/antonym distance vs enemy word.
  - [ ] Apply literary device multipliers.
  - [ ] Apply active FACES modifier.
  - [ ] Deal damage to enemy; enemy deals damage to player if not countered.
  - [ ] Show a combat log breakdown: base × distance × device × face = final.
- [ ] Add win/lose transitions: win → `Reviewing`, lose → Tutor Loop quest.

### Phase D — FACES & Companion Polish
- [ ] Display the active FACES emotion on the 2D companion sprite (color/tint or text label).
- [ ] Add face-change hotkeys/buttons outside battle too.
- [ ] Make the companion react to scanning, battles, and quest completion.
- [ ] Consider upgrading `SlimeFace` enum to a fuller `faces_protocol::FacesState` later.

### Phase E — Quests Reinterpreted
- [ ] Implement 2D AR Bounties: NPC asks for a concept, player scans a matching object.
- [ ] Add grammar-slot validation in Mad-Lib quests (reject noun in verb slot).
- [ ] Show the completed sentence and reward XP based on word fit.

### Phase F — Feedback & Juiciness
- [ ] Add screen shake on critical hits.
- [ ] Add simple particle bursts on scan, spell cast, and victory.
- [ ] Add sound effects for movement, scan, cast, hit, win, lose.
- [ ] Add a "would you like to continue?" summary after battle/review.

### Phase G — XR Parity & Future-Proofing
- [ ] Keep all overworld code behind `#[cfg(feature = "flat2d")]` or generic enough to port.
- [ ] Document the 2D ↔ XR mapping in a new `docs/2D_TO_XR.md` file.
- [ ] Leave `#[cfg(feature = "xr")]` stubs for pinch, ASL, and holographic spelling.

### Phase H — Verification
- [ ] `cargo test` passes.
- [ ] `cargo check --features flat2d` passes with 0 warnings.
- [ ] Update the autoplay harness to walk through the overworld and capture each new screen.
- [ ] Manual playtest: explore → scan → spell → battle → quest → review.
- [ ] Player can lose and be routed to Tutor Loop.
