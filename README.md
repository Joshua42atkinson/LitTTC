# LitTCG — Literary Trading Card Game

> *Every word is a card. Every card is a creature.*

LitTCG is a Bevy 0.18.1 XR EdTech game where kids spell real English words to summon unique 3D pets, then use those pets in tactical battles and grammar quests. Each pet's element, stats, face, and combat moves come from the word's actual meaning.

- 🌟 **Spell & Summon** — Build words from letter crystals and watch them become collectible pets.
- ⚔️ **Battle Wild Typos** — Use synonyms, antonyms, and etymology roots in turn-based combat.
- 🏛️ **Complete Grammar Quests** — Fill Mad-Lib slots with the right parts of speech to help NPCs.
- 🎴 **Collect & Evolve** — Master words to evolve pets and unlock golden auras and dream poetry.
- 🐾 **Persistent Companion** — Choose one pet to follow you through the world as an emotional anchor.
- 🎵 **Procedural Music** — State-driven soundtrack that crossfades between menu, explore, and battle themes.

## Why LitTCG?

Traditional literacy apps feel like dressed-up worksheets. LitTCG uses **isomorphism**: the game mechanic *is* the skill being taught.

- **Spelling IS summoning**
- **Synonyms ARE combat attacks**
- **Grammar IS questing**

Kids don't memorize vocabulary — they experience it. A fiery word becomes a fiery pet. A calm word becomes a calm pet. The learning is the playing.

## Tech Stack

- **Engine:** [Bevy 0.18.1](https://bevyengine.org/) (Rust ECS)
- **Web Build:** [Trunk](https://trunkrs.dev/) for WASM
- **XR:** `bevy_mod_xr` / `bevy_mod_openxr` (Android XR target)
- **Emotive System:** [`faces-protocol`](../crates/faces-protocol/) — 4-byte FACES state from word meanings

## Build

```bash
# Desktop
cargo run --features desktop

# Web (WASM)
trunk serve

# Android XR check
ANDROID_HOME="/home/joshua/Android/Sdk" \
NDK_HOME="/home/joshua/Android/Sdk/ndk/30.0.14904198" \
cargo ndk -t aarch64-linux-android check
```

## Project Status

Alpha — engine complete, building product surface.

- ✅ 5 embedded JSON databases
- ✅ Procedural 3D pet generation
- ✅ Semantic distance combat
- ✅ Mad-Lib quest engine
- ✅ Local save/load (COPPA-safe)
- ✅ Persistent companion (3D/XR camera follow)
- ✅ Pet lore shown in HUD and collection
- ✅ State-driven procedural music (`music.rs` + `scripts/generate_music.py`)
- 🔄 Pet card reveal animation (P0 in progress)
- 🔄 Collection screen + roster selection

## Documentation

For the full design doc, marketing plan, and roadmap, see the workspace root:

- [`../GDD.md`](../GDD.md) — Game Design Document
- [`../ROADMAP.md`](../ROADMAP.md) — Development phases
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — ECS architecture and data flow
- [`../TECHNICAL_MANUAL.md`](../TECHNICAL_MANUAL.md) — Build pipeline and integration patterns
- [`../docs/TAO_OF_FUN_REVIEW.md`](../docs/TAO_OF_FUN_REVIEW.md) — Fun-first design lens
- [`../docs/MUSIC_DESIGN.md`](../docs/MUSIC_DESIGN.md) — Music and sound design proposal
- [`../MARKETING_PLAN.md`](../MARKETING_PLAN.md) — Marketing strategy
- [`../BRAND_GUIDE.md`](../BRAND_GUIDE.md) — Brand voice and visual identity

## License

Proprietary — Joshua Atkinson

---

*LitTCG — The learning is the playing. The playing is the learning.*
