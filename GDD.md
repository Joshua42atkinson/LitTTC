# LitTCG — Game Design Document

> **LitTCG** (*Literary Trading Card Game*) is a pet collection game where words become living creatures.
>
> The child does not learn what a word means. The child builds a pet from a word and watches it come alive. The learning is the playing. The playing is the learning.
>
> This document is the single source of truth. Everything else is negotiable.

---

## Document Status

| Field | Value |
|-------|-------|
| **Project Name** | LitTCG: Word Slimes MVP |
| **Engine** | Bevy 0.18.1 ECS (Rust) |
| **Target** | Web (WASM) MVP → Desktop → Android XR |
| **Audience** | Homeschool market, ages 7-12 |
| **Status** | MVP Refactor + Tao of Fun Slice — companion, lore UI, and adaptive music in |
| **Date** | July 2026 |
| **Codebase** | 38 source files, 33 integration tests passing, `cargo clippy` clean on desktop/flat2d, `cargo check` clean on desktop/flat2d/xr, 9,582 words in database |
| **Recent Milestone** | Persistent companion follows camera; `PetLore` shown in HUD and collection; state-driven procedural music with crossfade and `music_volume` |

---

## 0. Executive Summary (Read This First)

**LitTCG** is a game where kids spell real English words to summon unique 3D pets.

**The 60-second loop:**
1. A child collects letter crystals in a 3D world.
2. They arrange the letters into a real word and press Submit.
3. A card flips over and a creature bursts out — the "Pokémon moment."
4. The creature's color, shape, face, and stats come from the word's meaning. "Inferno" is fiery and angry. "Serenity" is calm and glowing.
5. The child uses the creature in battles and quests to learn synonyms, antonyms, and grammar.

**Why it is special:** No two words produce the same pet. The pet is generated from the word's dictionary definition, etymology, and psycholinguistic data. This is the core IP.

**Who it is for:** Homeschool families first (ages 6–16), then K–12 schools via Chromebook.

**How we make money:** Free demo → $9.99 one-time full game → $4.99 expansion packs → optional $39.99/year parent dashboard.

**Current status:** The engine is complete. The next 30 days focus on a polished web demo: pet card reveal, async loading, demo limit, parent report, and a landing page.

---

## STRATEGIC PIVOT: The 4-Pillar Combat System (July 2026)

**Status:** We are pivoting from a pet collection focus to a **syntax-driven combat system** where language itself is the weapon. The core game loop must be validated in 2D before XR integration.

### The New Core Vision

LitTCG is a game where **language structure is the weapon**. Players don't just collect pets — they forge spells from grammar, battle with synonyms/antonyms, and evolve their vocabulary through semantic mastery.

**The Semantic Slime as Avatar/Inventory:**
- The Slime is the player's physical deck and weapon
- It oozes out to form letters, morphs into holographic cards, and consumes words
- In XR, it floats beside the player like a drone companion
- It IS the Grimoire — all collected words live inside it

**The Trainer Philosophy:**
- We are not "teachers" or "doctors" — we are **Trainers**
- We tame wild words, correct corrupted typos, and train vocabulary to mean more
- Learning happens through mastery, not instruction

### The 4 Pillars of Combat

**Pillar 1: Thesaurus Dance Battle (synonym_database.json)**
- **Synonyms** = Heavy Attacks / Overpower (semantic distance < 2.0)
- **Antonyms** = Parries / Counters / Shields (semantic distance > 4.0)
- Uses existing `WordStats` (Concreteness, Valence, Dominance, Intensity) for damage math
- Implemented in `battle.rs` with `semantic_distance()` function

**Pillar 2: Emotional Semantics (faces-protocol crate)**
- Players attach emotions to the Slime when playing words
- FACES protocol alters `WordStats` dynamically:
  - "FIRE" + [Fierce Face] = High damage/intensity
  - "FIRE" + [Joyful Face] = Healing campfire (High valence)
- Teaches connotation and emotional intelligence

**Pillar 3: Syntax Spell Crafting (Grammar at altar.rs)**
- Spells are forged using grammatical structure:
  - **Nouns** = Summons/Targets (Wolf, Wall, Sword)
  - **Adjectives** = Auras/Elements (Searing, Impenetrable, Swift)
  - **Verbs** = Actions (Strikes, Defends, Heals)
- "Searing Sword Strikes" = immediate fire damage
- "Impenetrable Wolf Defends" = fiery meat-shield
- Bevy ECS reads part-of-speech tags and combines stats for multiplier combos

**Pillar 4: Literary Devices as Metamagic**
- **Oxymoron** (Armor Piercing): Combining antonyms ("Deafening Silence") bypasses shields
- **Alliteration** (Combo System): 3 cards with same letter trigger "Echo Cast" (spell duplication)
- **Hyperbole** (Overcharge): Triples damage but exhausts player (recoil damage)
- **Foreshadowing** (Trap Cards): Face-down cards that activate on enemy conditions
- **Palindromes** (Reflection): Words spelled same backward/forward (RADAR, KAYAK) reflect attacks
- **Etymology Factions**: Latin/Greek roots = arcane buffs, Germanic/Norse = physical/brutal buffs

### The 12 Jungian NPCs as Mentors

Each NPC embodies a Jungian Archetype and teaches through their genre:
- **The Grammarian** (The Enforcer Boss): Requires proper Subject-Verb-Object sentences or recoil damage
- **The Poet** (The Bard): Fights using meter — must match syllable counts (5-7-5 Haiku combos stun)
- **The Editor** (The Final Boss): "Red Pen" mechanic — can delete cards from player's hand mid-battle

### RPG Progression: Vocabulary as Skill Tree

- Leveling = unlocking stronger synonyms
- "Hit" (5 dmg) → "Strike" (10 dmg) → "Pummel" (15 dmg + Stun) → "Obliterate" (50 dmg)
- Mastery = visual evolution (decorations, golden aura, Dream Layer poetry)

---

## The 2D Gray-Box Vertical Slice — "Pokémon Red for Words"

### Why 2D First

The 2D build is not a downgrade of the XR vision. It is a **cheap, fast prototype of the same loop**. If walking around a 2D world, scanning objects, spelling words, and battling typos is not fun, then pass-through AR and ASL will not save it. The 2D slice answers one question: *is the core gameplay loop fun?*

Once the loop is fun in 2D, we port it to XR by replacing keyboard/mouse with hand tracking and colored squares with holograms. The game logic stays identical.

### The 2D World

A single top-down explorable map built from colored rectangles and simple sprites. No 3D rendering, no tilemap crate, no external dependencies.

**What's in the world:**
- **Player Avatar** — a small controllable sprite (WASD / click-to-move).
- **Semantic Slime Companion** — follows the avatar at a short distance; shows the active FACES emotion; IS the player's deck/grimoire.
- **NPC Mentors** — the 12 Jungian archetypes standing in themed zones. Walk up and press `E` to talk. They give quests, teach etymology, and route the player into the Tutor Loop on defeat.
- **Scannable Objects** — rocks, trees, doors, signs, rivers. Walk up and press `E` to "scan" them. This is the 2D equivalent of the XR pinch-to-capture flow. Scanning yields a word card.
- **Wild Typos** — corrupted word creatures roaming the map. Touch one → transition to the Thesaurus Dance battle.
- **Districts** — 2-3 themed zones (Garden, Shadow Library, Irony Junction) to test the FACES/setting system.

**2D ↔ XR Mapping:**

| XR Action | 2D Equivalent |
|---|---|
| Look at real-world object | Walk avatar near object |
| Pinch to capture | Press `E` |
| ASL fingerspell word | Type the word or tap letters |
| Play a card | Click hand card |
| Change Slime face | Click face button |

### The Core 2D Loop

```
Explore world
    → Scan object → harvest word card
    → Talk to NPC → get quest
    → Touch wild typo → enter battle

Spell word (Constructing)
    → Word becomes a pet card
    → Pet card is added to the Semantic Slime

Use pet cards
    → In battle: Thesaurus Dance
    → In quest: fill grammar slots
    → In bonding: feed, pet, attune

Progress
    → XP, mastery, evolution, new districts
    → Defeat → Tutor Loop with matching NPC
```

### The Thesaurus Dance Battle (2D Combat)

Combat is a 1v1 vocabulary duel. The enemy is a Wild Typo with a word. The player builds a 1-3 card **sentence** (a "plot") and casts it.

**Sentence structure:**

```
[Adjective] + [Noun] + [Verb]
```

Example:
> **Searing Sword Strikes**

Each card contributes:
- **Adjective** — element/damage-type multiplier
- **Noun** — summon/target base effect
- **Verb** — action type (attack, defend, heal, burn, freeze)

**FACES Emotional Stance:**

Before casting, the player selects the Slime's emotional face. The face modifies the sentence's effect:

| Face | Effect |
|---|---|
| **Fierce** | +20% damage; fire verbs become blasts |
| **Joyful** | Heals player slightly; heals become group heals |
| **Calm** | No recoil from hyperbole; +block |
| **Angry** | +30% damage but take recoil |

FACES is a real choice independent of cards. The same three cards produce different results depending on the chosen emotion.

### Literary Devices as Plot Mechanics

The sentence the player builds can trigger literary-device "metamagic." These are the combo system of the card game.

| Device | Trigger | Combat Effect |
|---|---|---|
| **Alliteration** | 2+ cards start with same letter | Echo Cast — repeat the last card's effect |
| **Oxymoron** | Adjective and noun are antonyms | Armor Piercing — ignore enemy resistance |
| **Hyperbole** | Any card has high intensity | Overcharge — 3× damage, self-damage recoil |
| **Palindrome** | Any card is a palindrome | Reflect — return part of next enemy attack |
| **Personification** | Noun + animate verb | Summon a temporary companion/tank |
| **Onomatopoeia** | Verb is a sound word | Stun — enemy skips next turn |
| **Metaphor** | Two nouns in the sentence | Transform damage into the second noun's element |

The player is not just picking a card — they are building a sentence with synergies.

### Enemy Design

Each Wild Typo has:
- A **word** (e.g., "fire")
- A **part of speech weakness** (e.g., "weak to antonyms / verbs")
- An **element** (from etymology root)
- A **role** (from suffix)

The player reads the weakness, picks cards that counter it, and chooses a FACES emotion that amplifies the counter.

### Quests in 2D

NPCs give **AR Bounties** reinterpreted for 2D:

> *"My Slime is hot. Find something cold in this district."*
> 
> The child walks to the ice cave sprite, scans it ("ice"), spells I-C-E, and returns with the word.

Mad-Lib quests remain:

> *"The {ADJECTIVE} {NOUN} {VERB} loudly."*
> 
> The child plays cards that match the grammar slots.

### Success Criteria for the 2D Slice

- Player can explore a small map.
- Player can scan objects and spell words to get cards.
- Player can talk to NPCs and receive quests.
- Player can battle Wild Typos using sentence crafting + FACES + literary devices.
- The combat log explains why damage happened.
- Losing routes the player to a Tutor Loop NPC.
- The 1-minute loop is fun enough to replay.

Once this slice is fun, we port the exact same systems to XR: the avatar becomes the player, the scan button becomes a pinch, and the 2D sprites become holograms.

---

## 1. Core Vision

LitTCG is a pet collection game where **words are pets**.

A child spells a word. The word validates against a database of 9,582 English words. The word's etymology determines its element (Fire, Water, Earth, Air, Shadow, Light). Its suffix determines its role (Tank, Striker, Caster, Healer, etc.). Its psycholinguistic profile — real research data on concreteness, valence, arousal, and dominance — determines its combat stats. Its dictionary definition drives a 4-byte emotional state that gives it a unique facial expression. A 3D creature appears, colored by its element, shaped by its meaning, expressing emotion through its face.

**No two words produce the same pet.** "Inferno" becomes a fiery, aggressive creature with angry eyes. "Serenity" becomes a calm, gentle creature with soft features. The pet's personality emerges from the word's meaning. This is the game's core intellectual property — no other game does this.

> *In simple terms: Kids spell words, and each word becomes a unique pet. The pet's look, personality, and powers all come from what the word means. "Fire" makes a hot, angry pet. "Calm" makes a peaceful, gentle pet. Collecting words IS collecting pets.*

### Design Principles

- **Isomorphism** — The game mechanic IS the skill being taught. Spelling IS summoning. Synonyms IS combat. Grammar IS questing.
- **Active Imagination** — Words are not text on a page. They are living creatures with personalities. The child's imagination is the primary interface.
- **Stealth Assessment** — The game tracks what words the child uses, how they use them, and what patterns emerge. No tests. No quizzes. The play IS the assessment.
- **Local-First** — No cloud. No tracking. No accounts. Save files live on the family's device. COPPA compliant by design.

### The Tao of Fun

Our working design lens from `docs/TAO_OF_FUN_REVIEW.md`:

1. **Presence before points.** A world exists — a floating companion, a talking NPC, a changing sky, a shifting soundtrack — before any score is shown.
2. **Personality before procedure.** Every NPC, pet, and typo has a voice, a face, and a preference. The game does not explain; the world reacts.
3. **Permission before punishment.** Mistakes are discoveries. A misspelled word becomes a mutant, a lost battle becomes a tutor visit, a wrong root becomes a hint.
4. **Play before pedagogy.** The challenge can be quantitative and Common-Core aligned, but the exercise must feel like play — Montessori self-direction plus Steiner head/heart/hands balance.

---

## 2. The Board Game (Visual Analogy)

*Imagine the game as a board game on a kitchen table. This is the mental model for understanding all systems.*

### What's on the Table

**The child's side:**
- A **letter tray** — Scrabble tiles they've collected (A, B, C, etc.)
- A **spell pad** — where they arrange letters into a word and press Submit
- A **pet card** — face-down, like a Pokéball. When flipped, the pet appears.
- A **pet roster** — 3-6 pet cards face-up, their active battle team
- A **pet box** — all their collected pets, browsable, sortable
- A **companion** — one pet standing next to them, following them around the board

**The board:**
- **12 districts** — themed zones with different NPCs, letter fields, and difficulty
- **Quest board** — NPC request cards with color-coded slots to fill
- **Battle arena** — where corrupted words (Typos) appear as enemy pets
- **Nuisance zone** — roaming letters that chase the player

### One Round of Play

1. **Explore** — Move through a district. SemanticSlime companion follows. Collect letter crystals (curriculum-biased to grade level).
2. **Construct** — Arrange letters into a word. Press Submit.
3. **The Pokéball Moment** — A card appears face-down. The child flips it. The pet bursts out — with element colors, a FACES expression, rarity tier, and stats. "Inferno" is a Rare Fire Slime with angry eyes. The child goes "WHOAAAA."
4. **Bond** — New pet joins the Grimoire. The child can pet it (FACES → Happy), feed it (it eats related words), attune it (channel alignment), set it as companion.
5. **Battle** — A wild Typo appears as a corrupted pet. The child uses their SemanticSlime. To counter, they play an antonym (high semantic distance). To attack heavily, they play a synonym (low semantic distance).
6. **Quest** — An NPC gives a Mad-Lib with color-coded grammar slots. The child plays a word card into the matching slot. SemanticSlime bonus XP applies.
7. **Reward** — XP, evolution points, new letter crystals. Pet gains mastery. At mastery thresholds, it evolves — new visual decorations, stat boost, golden aura.
8. **Tutor Loop** — If defeated, the game routes to an appropriate NPC for targeted practice on the failed word concept. No "Game Over" screens.

> *In simple terms: Collect letters → spell a word → get a pet → use the pet in battles and quests → earn rewards → do it again with harder words. Like Pokémon, but you create the creatures by spelling.*

---

## 3. VAAM — Vocabulary Acquisition Autonomous Meaning

VAAM is the design philosophy that words are not memorized — they are **experienced**. The child does not look up a definition. The child builds the word, watches it become a creature, uses it in combat, feeds it to other pets, fills quest slots with it, and watches it evolve. Meaning is acquired through the journey, not the definition.

### How VAAM Works in the Game

Each word in the database carries **psycholinguistic metadata** — real research data from academic linguistics studies:

| Metric | What It Measures | Game Effect |
|--------|-----------------|-------------|
| **Concreteness** (C) | How physical/tangible the word is (1-5) | → Attack power (Logos). "Rock" hits hard. "Freedom" hits soft. |
| **Valence** (V) | How positive/negative the word feels (1-9) | → Health/survivability (Pathos). "Joy" has high HP. "Despair" has low HP. |
| **Intensity** (A) | How exciting/calm the word is (1-9) | → Speed (turn order). "Rage" is fast. "Sleep" is slow. |
| **Dominance** (D) | How much control/power the word implies (1-9) | → Defense (Ethos). "King" defends well. "Whisper" defends poorly. |
| **Age of Acquisition** (AoA) | When children typically learn this word | → Grade level / curriculum placement. "Cat" is K-2. "Ephemeral" is Graduate. |

### The VAAM Pipeline (Implemented in Code)

```
Child spells "thunder"
    → Validated against 9,582-word database
    → Psycholinguistic data loaded: C=3.2, V=5.8, A=7.2, D=5.1
    → Etymology root "ton" (sound) → Element: Air
    → Suffix "-er" → Role: Bruiser
    → Stats: Logos=64, Pathos=58, Ethos=51, Speed=72
    → FACES detection on definition → angry eyes, open mouth, intense focus
    → Rarity: Uncommon (120 pts, 1.15x multiplier)
    → 3D pet spawns: Air-colored, fast, aggressive, storm particle effects
```

The child never sees the numbers. They see a fast, aggressive, storm-colored creature with an angry face. They learn that "thunder" is powerful, loud, and energetic — because the pet IS those things.

> *In simple terms: The game uses real brain science data about words. Physical words (like "rock") make strong pets. Happy words (like "joy") make tough pets. Exciting words (like "rage") make fast pets. The child learns what words mean by feeling what the pets are like.*

### VAAM as Deck-Building Guide

The child's pet collection IS their vocabulary. Building a good roster requires understanding words:

- **Need a fast attacker?** Collect high-intensity words (Rage, Thunder, Storm).
- **Need a tank?** Collect high-dominance words (King, Fortress, Mountain).
- **Need a healer?** Collect high-valence words (Joy, Serenity, Comfort).
- **Need a defender?** Collect high-concreteness words (Wall, Stone, Shield).

The child learns word categories through gameplay, not instruction. They develop an intuitive understanding of psycholinguistics without ever hearing the term.

---

## 4. The Pet System

### 4.1 Pet Anatomy

Every pet is a Bevy ECS entity with these components:

| Component | What It Is | Source |
|-----------|-----------|--------|
| **PetAvatar** | Marks entity as a pet, stores word and class | `components.rs` |
| **PetFacesState** | 4-byte FACES emotional state | `faces-protocol` crate |
| **PetStats** | Logos (attack), Pathos (health), Ethos (defense), Speed | `components.rs` |
| **Element** | Fire/Water/Earth/Air/Shadow/Light/Normal | `components.rs` |
| **Role** | Tank/Bruiser/Striker/Assassin/Caster/Support/Buffer/Healer | `components.rs` |
| **SummonClass** | SemanticSlime / GrammarGolem / RhetoricRobot | `components.rs` |
| **PetVisualState** | Idle/Alert/Battle/Happy/Sleeping | `components.rs` |

### 4.2 Pet Creation Pipeline (Implemented in `letter.rs:307`)

```
Player collects letter crystals → LetterStash
    ↓
Player arranges letters → CurrentSpelling
    ↓
Press Enter / pinch Submit button
    ↓
submit_spelling_word() validates against GameDatabase.words (9,582 entries)
    ↓
If invalid → spawn Unstable Mutant (glitch entity, magenta)
    ↓
If valid:
    → Root analysis: scan 25 etymology roots → Element + stat focus
    → Suffix analysis: scan 27 suffixes → Role
    → PetStats: logos = concreteness × 20, pathos = valence × 10,
                ethos = dominance × 10, speed = intensity × 10
    → FACES: detect_scored() → 4-byte emotional state
    → Spawn 3D entity at (0, 1.5, -2.0) with all components
    → render.rs adds: head mesh, glow core, eyes, mouth, wings, ears,
                       orbital ring, 10 aura particles, 20 burst particles
    → Transition to GameState::Playing
```

### 4.3 Pet Card → Pet Reveal (The Pokéball Moment)

**Status: Not yet implemented. #1 feature to build.**

When a word validates, instead of immediately spawning the 3D pet:

1. Spawn a **PetCard** entity — flat card, face-down, glowing border
2. Child clicks/taps/pinches the card
3. Card flips with animation (0.5s rotation)
4. Pet bursts out — 3D mesh, burst particles, FACES expression
5. Card now shows pet stats face-up (element, role, stats, rarity)
6. Pet added to PetCollection

The card is the pet's home. When battle ends, pet goes back in. The child always has the card. This gives both the collectible card AND the living creature.

### 4.4 Pet Collection (Pet Box)

**Status: Not yet implemented. Replaces SpellBook.**

`PetCollection` resource stores all pets the child has ever summoned. Each entry stores: word, class, element, role, stats, FACES state, rarity, mastery, times_used, evolution_stage. Collection screen shows all pets as cards in a grid — sortable by element, class, rarity, mastery.

### 4.5 Roster (Battle Party)

**Status: Not yet implemented. Replaces Deck/Hand.**

Child selects 3-6 pets from collection to form a battle roster. Strategic choices:
- **RPS balance** — Don't bring all Slimes (a Golem enemy will crush you)
- **Element diversity** — Different elements for different enemy weaknesses
- **Role coverage** — Need a tank, a striker, and a healer
- **Mastery level** — Mastered pets are stronger, but maybe you want to level up a new one

### 4.6 Pet Bonding (Partially in `chat.rs`)

- **[P] Pet** — FACES → Happy. Pet smiles. Builds trust.
- **[F] Feed** — Pet "eats" related words. Synonyms nourish. Builds mastery.
- **[T] Attune** — Aligns pet to a Channel (Mind/Heart/Body/Action).

### 4.7 Companion Follow System

**Status: Implemented in `src/core/companion.rs`.**

The child chooses one pet from the collection and marks it as their companion (`SpellBookEntry::companion: true`). In 3D / XR modes the companion spawns as a persistent `PetAvatar` entity and smoothly follows the camera, giving the world an emotional anchor. In `flat2d` mode the system is disabled to keep the 2D gray-box clean. Future passes will add companion reactions to pickups, NPCs, and battle outcomes.

### 4.8 Music & Somatic Soundtrack

**Status: Implemented in `src/core/music.rs`.**

The soundtrack is not a passive loop. It is a state-aware procedural layer:

- `scripts/generate_music.py` writes loop-safe WAV stems from integer harmonic stacks.
- Three stems exist: `music_menu.wav` (calm), `music_world.wav` (explore), `music_battle.wav` (tense).
- `MusicPlugin` crossfades between them as `GameState` changes and respects `GameSettings.music_volume`.
- Future passes will tie drone pitch to the companion word, add spatial audio around the companion/altar, and add reveal flourishes per element.

This is the first step toward a VoixVive-style audio-first pedagogy: every sound teaches the ear.

### 4.9 Pet Dream Layer

**Status: Not yet implemented. Designed in Roblox version.**

When a pet reaches Mastered, it gains the Dream Layer — idle state where the pet emits pseudo-poetry from its etymology. "Inferno" might whisper "from fire I rise, from ash I fall..." Cosmetic, collectible, makes pets feel alive when not in use.

> *In simple terms: Every word becomes a pet card, like a Pokéball. You flip the card, the pet appears. You keep all pet cards in a box. You pick 3-6 to bring to battle. You can pet them, feed them, and they follow you around. When fully mastered, they dream and whisper poetry.*

---

## 5. FACES Protocol — How Pets Get Their Faces

The FACES protocol maps English grammar to visual appearance. It produces **38,400 unique emotional states** using zero compute — just keyword detection on the word's dictionary definition.

### The Four Bytes

| Byte | Grammar Role | Range | What It Controls |
|------|-------------|-------|-----------------|
| **Aura** (256) | Adjective | Mood, atmosphere | Pet color (ANSI-256 spectrum), emissive glow |
| **Container** (5) | Noun | Entity boundary | Head mesh: Neutral→IcoSphere, Rigid→Cuboid, Fluid→Torus, Defensive→Cylinder, Sharp→Cone |
| **Focus** (6) | Adverb | How action is performed | Eye shape: Intense→squinted, Open→wide, etc. |
| **Action** (5) | Verb | Kinetic output | Mouth shape: flat, open, curved smile, etc. |

### How It Works in Code

1. Child spells "inferno"
2. `faces_protocol::detect::detect_scored("inferno")` runs keyword detection
3. Definition contains "fire," "burn," "intense" → specific Aura/Container/Focus/Action
4. `render.rs:spawn_avatar_visuals()` reads FacesState:
   - `aura.index()` → `ansi_to_color()` → head color (deep red-orange)
   - `container` → mesh (Sharp → Cone)
   - `focus` → eye scale (Intense → squinted)
   - `action` → mouth (Aggressive → open)
5. Pet spawns with red-orange skin, cone head, angry eyes, open mouth

### FACES + SemanticSlime

| Class | Material | Mesh |
|-------|----------|------|
| **SemanticSlime** | Metallic 0.8, rough 0.15 (glossy) | FACES container determines shape |

> *In simple terms: The game reads what the word means and builds a face. "Inferno" gets red skin, sharp cone head, angry squinted eyes, shouting mouth. "Serenity" gets soft blue skin, round head, calm wide eyes, gentle smile. 38,400 possible faces, all from word meanings.*

---

## 6. Rarity & Evolution

### 6.1 Rarity Tiers

**Status: Not yet implemented. Existed in Roblox SlimeFactory.**

| Rarity | Point Pool | Stat Multiplier | Visual | Example Words |
|--------|-----------|-----------------|--------|---------------|
| **Common** | 80 | 1.0x | Basic blob, muted color | cat, dog, run |
| **Uncommon** | 120 | 1.15x | Element color, clear face | thunder, garden |
| **Rare** | 180 | 1.35x | Decorations (spikes, droplets) | inferno, fortress |
| **Epic** | 260 | 1.6x | Particle effects, glowing eyes | ephemeral |
| **Legendary** | 380 | 2.0x | Full VFX, aura, wings | transcendence |
| **Mythic** | 550 | 2.5x | Golden aura, dream layer | antidisestablishmentarianism |

Rarity is calculated from word difficulty: high AoA, low concreteness, long words, rare roots → higher rarity. This incentivizes learning harder words.

### 6.2 Evolution (Mastery = Growth)

**Status: Mastery tracking exists. Visual evolution not yet implemented.**

| Mastery | Icon | How to Reach | Visual Change |
|---------|------|-------------|---------------|
| **Encountered** | 🔮 | Spell the word | Basic blob, neutral color |
| **Experienced** | ⚡ | Use in battle or quest | Element colors, FACES active |
| **Owned** | 🌟 | Critical hit or quest slot fill | Decorations added |
| **Mastered** | 👑 | Use across multiple contexts | Full flourish, golden aura, +10% stats, Dream Layer |

> *In simple terms: Easy words make common pets. Hard words make rare, powerful pets. "Cat" is common. "Ephemeral" is legendary. Every pet evolves as you use it — bigger, fancier, stronger. A mastered pet gets a golden aura and whispers poetry.*

---

## 7. The SemanticSlime — Sole Companion (MVP)

**Status: MVP Refactor — GrammarGolem and RhetoricRobot deprecated.**

For the Word Slimes MVP, we have simplified to a single companion class: **SemanticSlime**. This reduces complexity while maintaining the core learning loop.

### Why SemanticSlime Only

- **Simplified Onboarding** — One class means less to learn for younger players (ages 7-12)
- **Focused Mastery** — Players master one combat system deeply instead of three shallowly
- **Curriculum Alignment** — Semantic relationships (synonyms/antonyms) are foundational vocabulary skills
- **Technical Debt Reduction** — Removes RPS balance complexity, class-specific rendering, and fusion mechanics

### How SemanticSlime Plays

| Playstyle | Attack Type | Damage Logic |
|-----------|-------------|--------------|
| **Wand Duel** | Semantic relationship | High distance = antonym/counter (block), Low distance = synonym/heavy attack, Mid-range = normal damage |

### Current Implementation in `battle.rs`

The combat system uses semantic distance for Wand Duel mechanics:

- **Counter/Block** (Distance > 4.0): Antonym logic. `1.5 + (distance - 4.0) × 0.2` multiplier. Effective against opposing concepts.
- **Heavy Attack** (Distance < 2.0): Synonym logic. `2.0x` multiplier. Overwhelms with similar concepts.
- **Normal** (2.0-4.0): `1.0x` damage. Standard attack.

### Grimoire — Physical Inventory

The `Grimoire` resource represents the SemanticSlime as the player's physical inventory/deck. Words collected become part of the Slime's knowledge base, stored as a `Vec<String>` with a `max_capacity` of 50 words.

---

## 8. Combat System

### 8.1 The Core Idea: Learning IS Combat

Combat is not separate from learning. The child demonstrates vocabulary knowledge **through** the combat mechanics. Every attack is a vocabulary exercise. Every defense is a grammar check.

### 8.2 Wand Duel Combat (Implemented in `battle.rs`)

When a Wild Typo appears, it carries the psycholinguistic coordinates of its word. The child must play a word based on semantic relationship to the Typo's word.

**Damage Formula:**

```
Distance = √((ΔC)² + (ΔV)² + (ΔD)² + (ΔA)²)
```

- **Counter/Block (Distance > 4.0):** Antonym logic. `1.5 + (distance - 4.0) × 0.2` multiplier. Blocks opposing concepts.
- **Heavy Attack (Distance < 2.0):** Synonym logic. `2.0x` multiplier. Overwhelms with similar concepts.
- **Normal (2.0-4.0):** `1.0x` damage. Standard attack.

Example: Typo is "fire" (C=4.5, V=5.0, A=7.0, D=5.0). Child plays "ice" (C=4.8, V=6.5, A=2.0, D=3.0). Distance ≈ 5.6. Counter/block!

### 8.3 Battle Flow

```
start_battle() → random word at child's grade level
    → BattleSession { typo_health: 50, player_health: 100, failed_word: None }
    → Pet visual state → Battle
    → UI: "WILD TYPO: [WORD]" + HP bars
    ↓
Child plays a card → play_battle_card() calculates damage
    → Effective: typo_health -= damage, mastery upgrade
    → Counter: antonym blocks Typo, damage multiplier applies
    → Synonym: heavy attack, 2.0x damage
    → Critical: screen shake + 30 burst particles
    ↓
typo_health ≤ 0 → Victory! → Mastered → Reviewing
player_health ≤ 0 → Defeat → Tutor Loop (Questing with NPC routing)
```

### 8.4 Tutor Loop — No Game Over (Implemented)

**Status: MVP Refactor — Failure routing added.**

When player health reaches 0, instead of a "Game Over" screen, the game enters the **Tutor Loop**:

1. `BattleSession.failed_word` tracks the word that caused defeat
2. `quest::route_to_tutor_npc()` maps the failed word to an appropriate NPC based on etymology (element/role)
3. `battle::start_tutor_loop()` initiates a targeted Mad-Lib quest with that NPC
4. Player practices with grade-appropriate words related to the failed concept
5. On quest completion, player returns to exploration with restored confidence

This ensures continuous learning without punitive failure states.

### 8.5 Planned: Active Learning During Combat

**Status: Not yet implemented. Inspired by Prodigy Math.**

| Action | Challenge | Reward |
|--------|-----------|--------|
| **Attack** | Type a synonym of your pet's word | Damage lands |
| **Counter** | Type an antonym of the enemy's word | Block enemy attack |
| **Critical Hit** | Identify the etymology root | 2x damage + VFX |

> *In simple terms: Fighting is learning. To attack heavily, find a word that means the same as the enemy's word (synonym). To counter/block, find a word that means the opposite (antonym). Your SemanticSlime uses semantic relationships to battle.*

---

## 9. Quest System

### 9.1 Mad-Lib Engine (Implemented in `quest.rs`)

NPCs give the child Mad-Lib style quests — sentences with blank slots that must be filled with pet cards. Each slot requires a specific part of speech.

**Flow:** `start_quest()` picks an NPC quest at the child's grade level → parses `{ADJECTIVE}`, `{NOUN}`, `{VERB}` slots → child plays pet cards via `fill_slot()` → `complete_quest()` reconstructs sentence, upgrades mastery, awards XP + evolution points, checks for grade-up.

### 9.2 Color-Coded Grammar (Planned — Inspired by Colourful Semantics)

**Status: Not yet implemented. Quest slots are currently text labels.**

| Color | Part of Speech | Example Slot | Example Word |
|-------|---------------|-------------|-------------|
| Orange | WHO (noun) | "{WHO} went to the store" | dragon |
| Yellow | WHAT DOING (verb) | "The dragon {WHAT_DOING} loudly" | roared |
| Green | WHAT (noun-object) | "The dragon ate {WHAT}" | treasure |
| Blue | WHERE (location) | "The dragon flew {WHERE}" | mountains |
| Purple | HOW (adverb) | "The dragon flew {HOW}" | gracefully |

The child must play a pet card whose word matches the part of speech. "Inferno" can't go in a WHO slot — it's not a person. But "dragon" can. Grammar validation through play.

### 9.3 Quest Data (60+ templates in `quest_data.json`)

**12 Archetypes, 5 quests each (60 total) + 33 NPC chain quests = 93 total quests.**

Each NPC has a 3-quest chain with increasing difficulty, plus time-of-day dialogue (Dawn/Day/Dusk/Night).

### 9.4 SemanticSlime Quest Bonus (Implemented in `quest.rs:108`)

- **SemanticSlime** → +5 XP (word consumption bonus)

> *In simple terms: NPCs give you fill-in-the-blank sentences. You put your pet cards in the blanks. But the pet's word has to match — a noun pet goes in a noun slot, a verb pet goes in a verb slot. It's like Mad-Libs with your collected pets. Your SemanticSlime gets bonus XP for consuming words in quests.*

---

## 10. World & Lore

### 10.1 The 12 Districts (Implemented in `quest.rs:162`)

| # | District | Theme | Grade |
|---|----------|-------|-------|
| 1 | Garden District | Growth, nature, beginnings | 1 |
| 2 | Outlaw Outpost | Rebellion, rule-breaking | 2 |
| 3 | Shadow Library | Mystery, hidden knowledge | 3 |
| 4 | Great Railway | Journey, connection | 4 |
| 5 | Maintenance Bay | Repair, practical work | 5 |
| 6 | Irony Junction | Contradiction, humor | 6 |
| 7 | Adjective Valley | Description, color | 7 |
| 8 | Central Station | Hub, crossroads | 8 |
| 9 | Metaphor Mountains | Figurative language | 9 |
| 10 | Logic Labyrinth | Reasoning, structure | 10 |
| 11 | Semantic Sea | Meaning, depth | 11 |
| 12 | Mastery Monolith | Final mastery | 12 |

### 10.2 The 12 NPCs (Implemented in `lore_db.json`)

| NPC | Archetype | District | Teaches |
|-----|-----------|----------|---------|
| Barnaby | The Innocent | Brainy Borough | -s, -ed |
| Yorick | The Everyman | Heartwood Grove | struct-, -ment, -tion |
| Kael | The Hero | Action Alley | -ing, -er |
| Martha | The Caregiver | Whisper Winds | -ful, -ly, -ness |
| Gribble | The Explorer | Action Alley | -able, -ible |
| Nyx | The Rebel | Whisper Winds | un-, de-, anti-, -ify |
| Vlad | The Lover | Heartwood Grove | phil-, amat-, path- |
| Pygmalion | The Creator | Whisper Winds | struct-, form-, -ify |
| Chesty | The Jester | Heartwood Grove | -ish, -esque, pseudo- |
| Ozymandias | The Sage | Brainy Borough | vis-, vid-, cogn-, -ology |
| Zafir | The Magician | Action Alley | trans-, meta-, hyper- |
| Ignis | The Ruler | Brainy Borough | -cracy, -archy, reg- |

### 10.3 Day/Night Cycle (Implemented in `time_cycle.rs`)

NPCs have four dialogue pools — Dawn, Day, Dusk, Night — making the world feel alive. Sky lighting changes dynamically via `update_sky_lighting()` in `render.rs`.

> *In simple terms: The game world has 12 areas to explore, each with a different character who teaches you new word parts. The characters say different things depending on the time of day. As you learn more, you unlock harder areas.*

---

## 11. Letter Collection & Nuisance System

### 11.1 Letter Crystals (Implemented in `letter.rs:24`)

Floating, rotating blue cubes with letters A-Z. Max 5 at a time. Collected by walking close (desktop) or pinching (XR). Stored in `LetterStash`.

### 11.2 Spelling (Implemented in `letter.rs:114`)

Child types letters (keyboard) or pinches holographic blocks (XR). Letters must be in stash. Backspace returns letters. Enter submits.

### 11.3 Nuisance Letters (Planned — From Roblox Version)

**Status: Not yet implemented.**

Clingy letters roam the world and chase the player. If they catch you, they cling to your letter tray. This can help (you needed an X!) or annoy (you have 5 Q's). Shake them off by spelling quickly. Rare letters (Z, X, Q) are valuable nuisances.

### 11.4 Curriculum-Biased Spawning (Implemented in `letter.rs:31`)

**Status: MVP Refactor — Implemented.**

Letters spawn biased toward forming grade-appropriate words. The system:

1. Queries the database for words matching the player's current grade level
2. Builds a letter frequency map from those grade-appropriate words
3. Converts the frequency map to a weighted letter pool
4. Spawns letters randomly from the weighted pool (fallback to A-Z if no grade words available)

This prevents frustration from getting 5 Q's and no vowels, and ensures letter availability aligns with curriculum goals.

> *In simple terms: You walk around collecting glowing letter cubes. You use them to spell words. Sometimes wild letters chase you and stick to your tray — helpful or annoying. The game gives you letters that can actually make words at your grade level.*

---

## 12. Character Progression

### 12.1 Attunement Channels (Implemented in `components.rs:110`)

Four channels track the child's linguistic style:

| Channel | Color | Emergent Class | Word Type |
|---------|-------|---------------|-----------|
| **Mind** | Green | The Oracle | Logical, structural |
| **Heart** | Orange | The Bard | Emotional, social |
| **Body** | Blue | The Cultivator | Physical, concrete |
| **Action** | Gold | The Templar | Aggressive, dynamic |

Each word use bumps the corresponding attunement by 10% of remaining distance to 1.0 (asymptotic). When dominant channel exceeds 0.2, emergent class manifests.

### 12.2 XP & Grade Progression (Implemented in `quest.rs:184`)

- XP from quests and battles
- Grade = `(total_xp / 1000) + 1`
- Grade-up unlocks next district
- Grade levels filter battle words: K-2, 3-5, 6-8, 6-9, 9-10, 10-12, 11-12, Graduate

### 12.3 Word Distribution by Grade

| Grade | Words |
|-------|-------|
| K-2 | 1,797 |
| 3-5 | 2,892 |
| 6-9 | 2,632 |
| 10-12 | 1,518 |
| Graduate | 739 |
| **Total** | **9,582** |

> *In simple terms: The game watches what kind of words you like. Smart words make you "The Oracle." Action words make you "The Templar." It's a personality test from your vocabulary. You level up by earning XP, and each level unlocks new areas.*

---

## 13. Curriculum & Data

### 13.1 Five Embedded JSON Databases (~3.3MB)

All data embedded via `include_str!`, loaded in `database.rs:270`:

| Database | File | Size | Contents |
|----------|------|------|----------|
| **Words** | `word_database.json` | 1.4MB | 9,582 words with psycholinguistic stats |
| **Synonyms** | `synonym_database.json` | 2.1MB | 9,578 entries: synonyms, antonyms, distractors |
| **Etymology** | `etymology_db.json` | 14KB | 25 roots → elements, 27 suffixes → roles |
| **Quests** | `quest_data.json` | 24KB | 93 quest templates (60 archetype + 33 NPC chains) |
| **Lore** | `lore_db.json` | 17KB | 12 NPCs with dialogue and schedules |

### 13.2 Etymology Root → Element (25 roots)

| Root | Element | Example | Root | Element | Example |
|------|---------|---------|------|---------|---------|
| Ignis | Fire | ignite | Cryo | Water | cryogenics |
| Aqua | Water | aquatic | Astr | Light | astronomy |
| Terra | Earth | terrain | Psych | Shadow | psychic |
| Aer | Air | aerial | Phot | Light | photon |
| Umbra | Shadow | umbrella | Therm | Fire | thermal |
| Lux | Light | lucid | Geo | Earth | geology |
| Chron | Air | chronic | Hydr | Water | hydrate |
| Mort | Shadow | mortal | Helio | Light | heliocentric |
| Vita | Light | vital | Nyct | Shadow | nyctophobia |
| Sci | Light | science | Cred | Earth | credit |
| Dyna | Fire | dynamic | Bio | Water | biology |
| Dict | Air | dictate | Voc | Air | vocal |
| Lumina | Light | luminous | | | |

### 13.3 Suffix → Role (27 suffixes)

| Suffix | Role | Suffix | Role | Suffix | Role |
|--------|------|--------|------|--------|------|
| -tion | Tank | -ize | Striker | -less | Striker |
| -ity | Tank | -ate | Caster | -ful | Buffer |
| -ment | Bruiser | -fy | Assassin | -ic | Caster |
| -ness | Tank | -ship | Support | -ist | Caster |
| -ance | Bruiser | -ous | Support | -logy | Support |
| -ence | Bruiser | -ive | Healer | -phobia | Assassin |
| -er | Bruiser | -able | Healer | -cracy | Tank |
| -or | Tank | -ible | Buffer | | |
| -en | Striker | -y | Support | | |
| -ish | Assassin | -al | Buffer | | |

### 13.4 Spiral Curriculum

Same words return at increasing difficulty:
- Grade 1: "cat" → Common pet, simple battle
- Grade 3: "cat" as a Typo with higher stats → child uses synonyms
- Grade 5: "cat" in a quest requiring adjectives ("the ___ cat")
- Grade 8: "feline" (related) appears as a Legendary pet

### 13.5 Hot Reloading

Bevy's `AssetServer` watches JSON files. Teachers can edit word lists and the game updates instantly without restart.

> *In simple terms: 9,582 words, each with brain science data. 25 word roots determine what element a pet is. 27 suffixes determine what role a pet has. Teachers can edit the text files and the game updates instantly. Words come back at higher difficulty as you level up.*

---

## 14. Input Systems

### 14.1 Desktop (Implemented)

- **Keyboard**: Letter keys (spelling), Enter (submit), Backspace (remove), 1-5 (slot select), Escape (back)
- **Mouse**: Click to collect, click to select cards, drag for swipe

### 14.2 Touch (Implemented)

- Swipe right = Yes, left = No, down = Dig Deeper, tap = Select
- `SWIPE_THRESHOLD` in `input.rs` prevents accidental micro-drags

### 14.3 XR Hand Tracking (Implemented behind `xr` feature)

- **Pinch-to-select**: Thumb-index distance < threshold → `PinchEvent`
- **ASL Fingerspelling**: Hand joint tracking for signing letters
- **Gesture intensity → Intensity**: Stronger gestures = higher-intensity pets
- **Holographic letter blocks**: Float in arc, pinch to select
- **Submit button**: Floating 3D button, pinch to submit

> *In simple terms: Play with keyboard/mouse, touch screen, or VR hands. In VR, you pinch floating letters to spell and pinch a button to submit.*

---

## 15. Save System

### 15.1 Implementation (`save.rs`)

- `serde_json` serialization to local disk (`save.json`)
- Auto-save on every `GameState::Playing` transition
- Disabled in demo mode

### 15.2 What's Saved

- **CharacterSheet**: Attunement scores, emergent class, XP, active summon class
- **SpellBook**: All collected words with mastery levels and encounter counts
- **StudentTrail**: Visited words, swipe history, current word

### 15.3 Constraints

- Local-first, no cloud, no accounts
- COPPA compliant — no personally identifiable information
- Save file is human-readable JSON — parents can open in any text editor

> *In simple terms: The game saves automatically. No internet needed. No account. The save file is a text file parents can open and read. It shows what words the child has learned.*

---

## 16. Build & Deployment

### 16.1 Three Build Targets

| Target | Command | Purpose |
|--------|---------|---------|
| **Desktop** | `cargo run --features desktop` | Development & full version |
| **Web (WASM)** | `trunk serve` | itch.io demo distribution |
| **Android XR** | `cargo ndk -t aarch64-linux-android check --features xr` | Future VR target |

### 16.2 Feature Flags

- `desktop` — Orbit camera, HDR, Bloom, SSAO
- `xr` — OpenXR, hand tracking, spatial UI
- `flat2d` — 2D-only rendering (lighter for WASM)
- `tts` — Kokoro TTS sidecar (disabled in WASM)

### 16.3 Cross-Platform Architecture

All game logic is platform-agnostic. Only rendering and input are feature-flagged. Both desktop and XR paths call the same `submit_spelling_word()` function.

> *In simple terms: The game runs on computers, web browsers, and VR headsets. Same game logic on all three. Only controls and graphics change. Web is the free demo. Desktop is the full game. VR is future.*

---

## 17. Pedagogical Foundations

### 17.1 Spelling as Casting

LitTCG takes the word "Spelling" literally. The child is not memorizing vocabulary — they are casting spells. Each letter is a component. Each word is an incantation. The creature that appears is the spell made flesh.

### 17.2 Steiner's Head, Heart, and Hands

- **Head (Thinking/Semantics)** → Semantic Slime — tank that absorbs and analyzes meaning
- **Heart (Feeling/Rhetoric)** → Rhetoric Robot — support using voice and persuasion
- **Hands (Willing/Syntax)** → Grammar Golem — bruiser built through physical assembly

### 17.3 Cognitive Load Theory (Sweller)

Psycholinguistic data drives gameplay physics:
- Abstract words (low Concreteness) = heavier, slower entities = more cognitive load
- Positive, high-energy words = speed boosts via `TimeScale` = rewards deep engagement

### 17.4 Constructionism (Papert)

Spelling is physical block assembly. If grammatical validity fails, the entity doesn't spawn — consequences are procedural and immediate. The child learns by building, not by being told.

### 17.5 Zone of Proximal Development (Vygotsky)

The `StudentTrail` tracks every choice. The curriculum spirals — words return at higher difficulty. The game bridges the gap between what the child can do alone and what they can do with the game's scaffolding.

> *In simple terms: The game is built on real education science. Spelling is like casting spells. The three pet types match how kids learn — thinking, feeling, doing. Hard words feel heavy in the game because they're hard in real life. Kids learn by building, not by being lectured.*

---

## 18. Research-Informed Design

### 18.1 Games That Informed This Design

| Game | What We Took | How We Use It |
|------|-------------|---------------|
| **Pokémon** | Pet collection, rarity tiers, type advantages | Words = pets, etymology = types, RPS = class balance |
| **Prodigy Math** | Learning during combat, not before | Synonym/antonym challenges during battle |
| **Duolingo** | Immediate feedback, spaced repetition | Invalid words spawn glitch entities; words return at higher difficulty |
| **WonderLang** | RPG vocabulary through enemies | Wild Typos carry word stats; defeating them = mastering words |
| **Roblox Slime Simulator** | Slime factory, rarity pool, decorations | Pet rarity system, visual evolution, companion follow |
| **Colourful Semantics / Ice Maze** | Color-coded grammar | Quest slots color-coded by part of speech |
| **Tamagotchi** | Pet bonding, care, growth | Pet/Feed/Attune interactions, FACES emotional response |

### 18.2 Academic Foundations

| Theory | Application |
|--------|------------|
| **Psycholinguistics** (Warriner et al.) | 9,582 words with concreteness, valence, intensity, dominance data |
| **Cognitive Load Theory** (Sweller) | Word difficulty → entity weight/speed |
| **Constructionism** (Papert) | Spelling as physical block assembly |
| **Zone of Proximal Development** (Vygotsky) | Spiral curriculum, StudentTrail tracking |
| **Procedural Rhetoric** (Bogost) | Grammar rules enforced through game processes, not text |
| **Steiner's Threefold Nature** | Head/Heart/Hands → Slime/Robot/Golem |
| **Classical Trivium** | Grammar/Logic/Rhetoric → Golem/Slime/Robot |
| **Jungian Archetypes** | 12 NPCs mapped to 12 archetypes |

### 18.3 FACES Protocol Research

The FACES protocol is original research, documented in `crates/faces-protocol/docs/`. It maps English grammar to a 4-byte emotional state using zero-compute keyword detection, producing 38,400 unique states. This is the game's core technical innovation — no other game generates visual appearance from word meaning.

> *In simple terms: We studied Pokémon, Duolingo, Prodigy, and other games to find what works. We took the best ideas and combined them with real brain science about how kids learn words. The FACES system — making faces from word meanings — is our original invention. No other game does this.*

---

## 19. Engine Status — What Exists vs What Needs Building

### 19.1 What's Built and Working (38 source files, 33/33 integration tests)

| System | Status | Source File |
|--------|--------|------------|
| 5 JSON databases loaded | ✅ Done | `database.rs` |
| Word validation + pet spawning | ✅ Done | `letter.rs:307` |
| Etymology → Element + Role | ✅ Done | `letter.rs:335` |
| Psycholinguistic stat calculation | ✅ Done | `letter.rs:369` |
| FACES detection + 3D mesh | ✅ Done | `letter.rs:378`, `render.rs:206` |
| Procedural pet rendering | ✅ Done | `render.rs` (831 lines) |
| Eyes, mouth, wings, ears, rings, particles | ✅ Done | `render.rs:300-396` |
| Pet animations (idle, alert, battle, happy, sleeping) | ✅ Done | `render.rs:424` |
| Semantic distance battle system | ✅ Done | `battle.rs:17` |
| Class-specific combat (3 modes) | ✅ Done | `battle.rs:79-152` |
| Battle UI (2D + XR) | ✅ Done | `battle.rs:368-447` |
| Critical hit effects (screen shake, particles) | ✅ Done | `battle.rs:213` |
| Mad-Lib quest engine | ✅ Done | `quest.rs:19-144` |
| NPC dialogue system | ✅ Done | `quest.rs:146` |
| 12 districts + curriculum manager | ✅ Done | `quest.rs:162-218` |
| Pet bonding (Pet/Feed/Attune) | ✅ Done | `chat.rs` |
| FACES state observation | ✅ Done | `chat.rs` |
| Kokoro TTS integration | ⚠️ Feature-gated | `chat.rs` (behind `tts` flag, untested on WASM) |
| Save/load (JSON, auto-save) | ✅ Done | `save.rs` |
| HUD (XP, grade, deck counter) | ✅ Done | `hud.rs` |
| Main menu | ✅ Done | `menu.rs` |
| Tutorial system | ✅ Done | `tutorial.rs` |
| Paywall/demo limits | ✅ Done | `paywall.rs` |
| Day/night cycle | ✅ Done | `time_cycle.rs` |
| Spatial UI panels | ✅ Done | `spatial_ui.rs` |
| Spatial deck (XR) | ⚠️ Scaffolded | `spatial_deck.rs` (UI shell, no real XR interaction) |
| Altar/summoning system | ⚠️ Basic geometry | `altar.rs` (cylinder + button, no real summoning logic) |
| Dialogue UI | ✅ Done | `dialogue_ui.rs` |
| Hand tracking + pinch | ⚠️ Stub | `hand_tracking.rs` (simulated desktop positions, not real OpenXR joints) |
| ASL fingerspelling | ⚠️ Stub | `hand_tracking.rs:105` (only detects 'A' and 'L' via distance heuristic) |
| Grammar fusion system | ✅ Fixed | `hand_tracking.rs:125` (now queries `PetAvatar` instead of non-existent `Summon`) |
| Letter crystals + collection | ✅ Done | `letter.rs` |
| Keyboard spelling | ✅ Done | `letter.rs:114` |
| XR holographic spelling | ⚠️ Scaffolded | `letter.rs:188` (UI exists, input is simulated) |
| Deck/hand/discard system | ✅ Done | `deck.rs` |
| Swipe input | ✅ Done | `input.rs` |

### 19.2 What Needs Building (Priority Order)

| Priority | Feature | Description |
|----------|---------|-------------|
| **P0** | **"Arousal" → "Intensity" rename** | Workspace-wide rename of `arousal` to `intensity` in all structs, UI, and display traits. JSON key stays "A" for parsing. Safety: "Arousal" is inappropriate for a children's game UI. |
| **P0** | **Profanity blocklist** | Filter in `submit_spelling_word()` — banned words fail silently (no glitch entity, no reward). Safety: prevent kids from summoning pets from slurs/profanity. |
| **P0** | Pet Card reveal | Card flip animation before pet spawns (the Pokéball moment) |
| **P0** | Pet Collection screen | Browse all collected pets as cards, sortable |
| **P0** | Roster selection | Pick 3-6 pets for battle from collection |
| **P1** | ASL fingerspelling (full) | Expand `hand_tracking.rs:105` from 2-letter stub to full A-Z ASL recognition for Google Aura VR spelling |
| **P1** | Rarity system | Calculate and display rarity tiers with stat multipliers |
| **P1** | Visual evolution | Pet appearance changes at mastery thresholds |
| **P1** | RPS class modifier | +50%/-25% damage based on class matchup |
| **P1** | Active combat learning | Synonym/antonym challenges during battle |
| **P1** | Color-coded quest slots | Visual grammar validation |
| **P2** | ~~Companion follow system~~ | ✅ Done in `companion.rs` |
| **P2** | Nuisance letters | Roaming letters that chase player |
| **P2** | Curriculum-biased spawning | Letters biased toward grade-appropriate words |
| **P2** | Pet Dream Layer | Mastered pets whisper etymology poetry |
| **P3** | Parent Dashboard | Web app to read save.json and show analytics |

> *In simple terms: The engine core is done — spelling, pet creation, battles, quests, saving, rendering all work. But several XR systems are stubs (ASL only detects 2 letters, hand tracking is simulated, grammar fusion is broken). Two safety issues must be fixed before shipping: rename "arousal" to "intensity" and add a profanity blocklist. Then the "Pokémon moment" — card flip reveal, collection screen, roster selection — is the next thing to build.*

---

## 20. Commercialization

**Hybrid Freemium:**

1. **Free Web Demo** (itch.io, WASM): 10 starter words, 1 NPC quest chain, 1 battle. No saving. Top-of-funnel marketing for homeschool parents.
2. **Paid Desktop/Mobile** ($9.99): Full 9,582-word database, all 12 NPCs, all 93 quests, save progression.
3. **Expansion Packs** ($4.99): Themed word lists (SAT Prep, Science Vocab, Spanish-English Bridge).
4. **Parent Dashboard** (future $7.99/mo): Web app that reads save.json and shows analytics — words learned, mastery levels, attunement profile, time spent, struggling areas.

### Why This Model Works for Homeschool

- Parents control the budget — one-time $9.99, no microtransactions for kids
- Demo is free and accessible — no risk to try
- Save file is readable — parents can verify learning without the dashboard
- Expansion packs align with curriculum needs — SAT prep is a natural upsell
- Dashboard is optional — the game works fully without it

> *In simple terms: Free demo on the web. $10 for the full game. $5 expansion packs for specific topics. Optional $8/month parent dashboard to see how your kid is doing. No microtransactions for kids. Parents stay in control.*

---

## 21. Demo Scope

For the itch.io WASM release:

- 10 total words in the dictionary (curated for variety: mix of elements, classes, grades)
- 1 NPC quest chain (3 quests from one NPC)
- 1 battle encounter
- No local saving (progress resets on refresh)
- "Get Full Version" prompt after significant play
- Full visual polish — FACES expressions, particle effects, animations

### Demo Word Selection Criteria

The 10 demo words should showcase the system's range:
- 2 Fire element words (different roles)
- 2 Water element words
- 2 Earth element words
- 1 Air, 1 Shadow, 1 Light, 1 Normal
- Mix of SummonClasses (3 Slime, 4 Golem, 3 Robot)
- Mix of grade levels (mostly K-2 and 3-5 for accessibility)
- At least one word that produces a dramatic FACES expression

> *In simple terms: The free demo has 10 words, 1 character's quests, and 1 battle. No saving. It's a taste. If the kid loves it, parents buy the full version for $10.*

---

## 22. Shipping Checklist

### Must Have Before Demo Ship

- [ ] Zero compiler warnings (`cargo check` clean — currently 12 warnings)
- [ ] All 8 integration tests passing (`cargo test`)
- [x] **"Arousal" renamed to "intensity" in all code + UI** (safety)
- [ ] **Profanity blocklist implemented in `submit_spelling_word()`** (safety)
- [ ] Main menu loads on launch
- [ ] Tutorial plays for first-time users
- [ ] Player can spell words and see visual feedback
- [ ] Pet card reveal animation (the Pokéball moment)
- [ ] Pet collection screen (browse collected pets)
- [ ] HUD displays required information
- [ ] Battle encounter completes (start → fight → win/lose)
- [ ] Quest completes (start → fill slots → reward)
- [ ] WASM build runs in browser without crashing
- [ ] Demo limitations apply correctly (10 words, no save)
- [ ] itch.io page copy and assets ready

### Nice to Have Before Demo Ship

- [ ] Roster selection (pick pets for battle)
- [ ] Rarity tiers displayed on pet cards
- [ ] RPS class modifier in combat
- [ ] Color-coded quest slots
- [ ] Companion pet follows player
- [ ] Critical hit screen shake + particles (already implemented, needs wiring)

### Post-Demo Roadmap

- [ ] Full 9,582-word database unlocked
- [ ] All 12 NPCs with quest chains
- [ ] Visual evolution system
- [ ] Nuisance letters
- [ ] Pet Dream Layer
- [ ] Parent Dashboard web app
- [ ] Android XR build
- [ ] Expansion pack system

---

## Appendix A: English Skills Covered

| Skill | Mechanic | Source Code |
|-------|----------|------------|
| Spelling | Collect letters, arrange into words | `letter.rs` |
| Vocabulary | Each word becomes a collectible pet | `components.rs` |
| Etymology | Root analysis determines pet element/stats | `database.rs` |
| Parts of speech | FACES Container/Focus/Action = noun/adverb/verb | `faces-protocol` |
| Synonyms/antonyms | Battle mechanic — match word relationships | `battle.rs` |
| Sentence structure | Mad-Lib quests (fill noun/verb/adj slots) | `quest.rs` |
| Psycholinguistics | Concreteness/Valence/Intensity/Dominance → stats | `database.rs` |
| Grammar | Suffix analysis determines pet role | `letter.rs:351` |
| Creative writing | Mad-Lib quest completion produces sentences | `quest.rs:94` |

## Appendix B: What We Cut (And Why)

| System | Why Cut | Replaced With |
|--------|---------|--------------|
| Three separate games (Trivium) | Can't ship 3 games | One game, RPS class balance |
| Symbol / ARCANA | Redundant with FACES | FACES Container/Focus/Action |
| SynergyLinks / Wu Xing | Over-engineered | Simple synonym/antonym matching |
| Sled Vector DB | Too heavy | JSON save file |
| 12-phase ADDIECRAPEYE | Over-engineered | GameState (10 states) |
| Autopoietic code mutation | Self-modifying code, unsafe | Cut entirely |
| DAG Curriculum graph | Over-engineered | Simple word list with grade levels |
| Janus-Pro-1B / Trellis (runtime) | Too heavy for WASM/mobile, breaks determinism | Three-tier hybrid (see below) |

## Appendix B2: Three-Tier Pet Generation Strategy

Instead of embedding AI generation in the core game loop, we use a hybrid approach:

**Tier 1: Procedural FACES (current system) — covers all 9,582 words instantly**

The FACES protocol produces 6,451,200 unique configurations (256 aura × 5 container × 6 focus × 5 action × 7 elements × 8 roles × 3 classes). Every word gets a pet. No waiting, no AI, no downloads. The pet's appearance is deterministic from the word's meaning — this is the core IP.

**Tier 2: Pregenerated glTF for top ~500 words — offline AI, shipped as assets**

Use Janus Pro + Trellis offline as **artist tools**, not runtime engines. Generate high-quality 3D models for the most common words (K-2 and 3-5 grade levels). Ship as embedded `.glb` files. The code at `render.rs:267` already loads these automatically. First 500 pets look hand-crafted; remaining 9,082 words use the procedural system.

**Tier 3: Pet Studio (paid desktop feature) — embedded AI for custom accessories**

Janus Pro + Trellis at runtime, desktop only. Not for generating whole pets — for generating **accessories** (hats, armor, aura effects, companion creatures). The base pet is always FACES-procedural. Accessories are small, fast to generate, and don't break the game loop if they fail.

**Why not full AI for every pet:**
- WASM binary size: Janus Pro 1B quantized = ~500MB-1GB, can't embed in WASM
- Runtime latency: 5-30 seconds per pet breaks instant gratification
- Quality inconsistency: some generations look bad
- Loss of determinism: FACES guarantees "inferno" always looks angry/red
- Loss of pedagogical link: the pet's appearance IS the word's meaning — AI breaks that
- Compute drain: mobile/XR targets can't run a 1B model

| Great Railway of Lexis lore | Over-scoped | 12 NPC districts in lore_db.json |

## Appendix C: The FACES Protocol

FACES (Focus, Action, Container, Element, System) is a 4-byte emotive state protocol:

```
[Aura: 1 byte] [Container: 1 byte] [Focus: 1 byte] [Action: 1 byte]
    256 values        5 values          6 values         5 values
```

Total unique states: 256 × 5 × 6 × 5 = **38,400**

- **Aura** → ANSI-256 color spectrum → pet color + emissive glow
- **Container** → 5 mesh shapes (Neutral/Rigid/Fluid/Defensive/Sharp)
- **Focus** → 6 eye expressions (Intense/Open/Closed/ etc.)
- **Action** → 5 mouth shapes (Flat/Open/Curved/ etc.)

Detection is zero-compute: keyword matching on the word's dictionary definition. No neural networks, no API calls, no latency. The protocol is documented in `crates/faces-protocol/docs/` and implemented in the `faces-protocol` crate.

---

*This document is the single source of truth for LitTCG. All design decisions should reference this document. When this document and code disagree, the code is the current reality and this document is the target.*

*Last updated: July 2026*
