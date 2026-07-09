# LitTCG Development Roadmap

## Current Status: Strategic Pivot — 4-Pillar Combat System (July 2026)

**Last verified:** July 2026 — Infrastructure complete, pivoting to 2D vertical slice for core combat validation.

**CRITICAL RULE:** Do not build Phase 2 until Phase 1 is flawlessly executing. We must stay in Bevy 2D UI mode (`docs/2D_MODE.md`). NO OPENXR OR 3D RENDERING YET.

---

## NEW PHASED EXECUTION PLAN

### PHASE 1: The 2D "Gray-Box" Combat Slice (IMMEDIATE FOCUS)

**Vision:** Validate the core combat loop on a flat screen before any XR or 3D rendering. If the game isn't fun with 2D squares, 3D VR won't save it.

**Deliverables:**
- Hardcoded micro-deck of 20 words (mix of Nouns, Verbs, Adjectives with obvious synonyms/antonyms)
- 2D Bevy UI scene with:
  1. Target Dummy (100 HP) displaying a "Prompt Word"
  2. Player's Hand (5 buttons at the bottom)
  3. Altar (drop-zone/click-zone for active card)
  4. Slime Face UI (4 buttons to change active Emotion/Face)
  5. Combat Log UI that prints the math behind damage calculation
- Basic Thesaurus Battle math: Playing a synonym deals damage. Modifying the Face alters the multiplier.

**Success Criteria:**
- Player can play a card from hand to altar
- Damage calculation is visible in combat log
- Synonym vs Antonym behavior is clear and satisfying
- Face modification changes damage multiplier visibly
- The 1-minute combat loop is addictive enough to replay

---

## Interlude — Tao of Fun / Somatic Slice (DONE)

A parallel "fun-first" pass that builds emotional presence while the 2D combat slice is being validated.

- [x] Persistent companion entity (`src/core/companion.rs`)
  - Spawns from `SpellBookEntry::companion` selected in the collection screen
  - Follows the camera in 3D / XR (feature-gated behind `#[cfg(not(feature = "flat2d"))]`)
- [x] AI-generated `PetLore` surfaced in HUD and collection detail panel (`hud.rs`, `pet_collection.rs`)
- [x] Procedural music system (`src/core/music.rs`)
  - State-driven track selection: menu → world → battle
  - Crossfades between looped WAV stems
  - Respects `GameSettings.music_volume`
  - Generated stems script at `scripts/generate_music.py`
- [x] Documented music design in `docs/MUSIC_DESIGN.md` using VoixVive audio-first principles

---

### PHASE 2: Metamagic & Sentence Construction

**Vision:** Upgrade from single-card combat to multi-card syntax crafting. Implement literary device mechanics and etymology factions.

**Deliverables:**
- Upgrade `altar.rs` to allow 3 cards per turn (Adjective + Noun + Verb)
- Literary Device engine:
  - Oxymoron detection (conflicting words pierce armor)
  - Alliteration detection (same letter = Echo Cast multiplier)
  - Etymology faction bonuses (Latin/Greek vs Germanic/Norse)
- Etymology database integration for faction multipliers
- Visual feedback for metamagic triggers

**Success Criteria:**
- 3-card syntax crafting feels natural
- Oxymoron combos are discoverable and rewarding
- Alliteration chains create satisfying combo moments
- Etymology factions create strategic depth

---

### PHASE 3: The Mentors & RPG Progression

**Vision:** Implement the loss-state loop, boss battles with genre-specific mechanics, and synonym skill tree progression.

**Deliverables:**
- Loss-state loop: Defeat → NPC routing → targeted practice → return to exploration
- Mock "The Grammarian" boss battle with syntax-checking mechanics
- Synonym Skill Tree: Upgrade cards in deck (Hit → Strike → Obliterate)
- XP and grade progression tied to vocabulary expansion
- NPC quest chains with archetype-specific challenges

**Success Criteria:**
- No punitive "Game Over" screens
- Tutor Loop feels like learning, not punishment
- Skill tree progression is motivating
- Boss battles teach specific literary concepts

---

### PHASE 4: XR & ASL Re-Integration

**Vision:** Once 2D loop is proven fun, port to XR with hand tracking and ASL fingerspelling for resource gathering.

**Deliverables:**
- Port 2D UI panels to `spatial_ui.rs` and `spatial_deck.rs`
- Re-enable `hand_tracking.rs` for physical ASL finger-spelling
- Connect `lit-asset-forge` (ComfyUI) for dynamic card art generation
- AR object capture with VLM (Janus-Pro-1B) for word discovery
- Spatial altar for 3D spell construction

**Success Criteria:**
- XR mode maintains 2D combat fun factor
- ASL spelling feels responsive and accurate
- AR capture pipeline is performant (no main thread blocking)
- Generated card art enhances, not replaces, core gameplay

---

## LEGACY PHASES (Pre-Pivot) — ✅ DONE

## Phase 1-8: Engine Core — ✅ DONE

- [x] 22 source files compile cleanly
- [x] 5 embedded JSON databases (~3.3MB) loaded and parsed
- [x] Full GameState machine (Loading → MainMenu → Collecting → Constructing → Playing → Questing → Battling → Reviewing → Paywall)
- [x] Pet spawning pipeline (word → etymology → element/role → stats → FACES → 3D mesh)
- [x] Battle system with semantic distance combat
- [x] Quest system with Mad-Lib templates
- [x] Chat system with FACES dialogue and Kokoro TTS
- [x] Save/load system (local JSON, COPPA compliant)
- [x] HUD, Main Menu, Tutorial, Paywall UI
- [x] XR scaffolding (hand tracking, spatial UI, spatial deck)
- [x] Procedural rendering (FACES morphs, particles, screen shake)
- [x] 33 integration tests covering database, battle, quest, save, chat, curriculum

---

## Phase 9: Word Slimes MVP Refactor — ✅ DONE

**January 2025** — Refactored codebase for Web/WASM MVP targeting ages 7-12 homeschool market.

- [x] Deprecated GrammarGolem and RhetoricRobot classes (components.rs)
- [x] Updated SummonClass enum to only include SemanticSlime
- [x] Added Grimoire resource as physical inventory/deck representation
- [x] Removed RPS class modifiers from battle system
- [x] Implemented Wand Duel combat (1v1 Synonym/Antonym based on semantic distance)
- [x] Updated semantic_distance() logic for new combat rules
- [x] Removed Game Over screens from quest.rs and battle.rs
- [x] Implemented Tutor Loop state transition on player health = 0
- [x] Added NPC routing logic based on failed word (route_to_tutor_npc)
- [x] Integrated targeted Mad-Lib quest generation for tutoring (start_tutor_loop)
- [x] Removed pure RNG A-Z letter spawning from letter.rs
- [x] Implemented curriculum-biased letter spawning using GradeLevel
- [x] Added database query for grade-appropriate words
- [x] Updated integration tests for new combat mechanics
- [x] Updated integration tests for Tutor Loop failure routing
- [x] Fixed all compilation errors and warnings (cargo check passes with 0 warnings)
- [x] All 33 integration tests passing

---

## Phase 10: Product Surface (UI & Flow) — ⬜ PENDING

- [ ] Verify full game loop: menu → collect letters → spell word → pet spawns → quest → battle → review
- [ ] Polish visual letter collection and spelling feedback
- [ ] Add smooth XP bar animation (currently snaps)
- [ ] Add emergent class badge to HUD
- [ ] Add visual feedback for critical hits in battle (screen shake, particles)
- [ ] Show enemy psychometric stats on health bar for strategic deduction
- [ ] Polish pet renderer (ensure glTF fallback works)
- [ ] Tune swipe threshold to prevent accidental micro-drags
- [ ] Add error handling in chat.rs for offline Kokoro TTS

---

## Phase 11: Web Demo (WASM) — ⬜ PENDING

- [ ] Verify `trunk serve` runs without crashes in browser
- [ ] Implement demo word limit (10 words) in paywall.rs
- [ ] Disable save system in demo mode
- [ ] Add "Get Full Version" prompt after significant play
- [ ] Prepare itch.io page assets and copy
- [ ] Test on multiple browsers (Chrome, Firefox, Safari)

---

## Phase 12: Revenue Infrastructure — ⬜ FUTURE

- [ ] Scaffold Parent Dashboard (separate web app to view `save.json`)
- [ ] Design dashboard UI showing emergent class, mastery, grade level
- [ ] Add curriculum export for school district integration

---

## Phase 13: Android XR — ⬜ FUTURE

- [ ] Verify `cargo ndk` cross-compilation
- [ ] Test XR mode on HTC VIVE XR Elite
- [ ] Calibrate pinch-to-select distance threshold
- [ ] Disable Bloom/SSAO for XR battery life
- [ ] Test hand tracking ASL fingerspelling
- [ ] Package as APK

---

## Shipping Checklist

- [x] Zero compiler warnings
- [x] All integration tests passing
- [x] Companion spawns and follows camera
- [x] Procedural music crossfades by game state
- [ ] Main menu loads on launch
- [ ] Tutorial plays for first-time users
- [ ] Player can spell words and see visual feedback
- [ ] HUD displays required information
- [ ] WASM build runs in browser without crashing
- [ ] Demo limitations apply correctly
- [ ] Itch.io page copy and assets ready
