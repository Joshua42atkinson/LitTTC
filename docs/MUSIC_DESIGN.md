# Music Design for LitTCG — From VoixVive to the Syllabus Symphony

This is a design proposal for music in `LitTCG`. It is inspired by the audio-first philosophy of `VoixVive` (`/home/joshua/Workflow/Bertrand-Masterclass`), especially:

- **Sound as the nervous system** of the app.
- The `Be / Do / Play` pedagogical progression.
- Real-time procedural drones and spatial audio.
- The rule that *every sound should make the player a better listener*.

---

## 1. What should LitTCG music feel like?

The game already validates English words and turns them into procedural pets. The soundtrack should do the same thing in audio: **turn language into music**.

| Layer | Purpose | Example |
|---|---|---|
| **World ambience** | Makes the districts feel like places | Dusk in the Verdant district = warm pads + slow arpeggio |
| **Companion leitmotif** | Gives the chosen pet a musical voice | A short 2-bar motif derived from the companion word's FACES aura |
| **Battle ostinato** | Adds tension to fights | Enemy element sets the key; player cards add counter-melody |
| **Reveal flourish** | Reinforces the "Pokémon moment" | Chord that matches the summoned pet's element |
| **Time-of-day drone** | Background presence | Night = low, slow drone; Dawn = rising fifths |
| **Static corruption** | Antagonist signature | Dissonant, detuned cluster when typos/mutants appear |

The music should **never** be a looped MP3 that ignores gameplay. It should react to:

- `GameState` (Menu / Playing / Battling / Questing / RevealingPet)
- `DayNightCycle` phase
- Current district / `WorldState`
- Active companion word's FACES stats and element
- Battle enemy element and intent

---

## 2. The VoixVive lessons we can port directly

From `docs/XR_AUDIO_MATURATION.md` and `PHASE 4 SOVEREIGN SOUND ARCHITECTURE.md`:

### 2.1 Procedural drones, not static tracks
VoixVive uses a Web Audio oscillator drone with a root frequency, Pythagorean interval ratios, and LFO breathing modulation. For LitTCG this maps cleanly onto:

- **Root = current companion word's aura pitch** or a district tonic.
- **Interval ratios = element-to-element relationships** (Fire-Water could be a 5th, Shadow-Light an octave).
- **LFO breathing** = tied to the player's attunement or the day/night phase.

### 2.2 Spatial audio = harmonic geometry
In VoixVive the frets emit tones from their physical positions. In LitTCG we can place:

- The **companion drone** at the companion entity position.
- **Battle sounds** from the enemy position.
- **Reveal sounds** from the card spawn position.

Bevy 0.18 `bevy_audio` supports `SpatialListener` on the camera and `SpatialBundle`-style positioning.

### 2.3 Sound must train the ear
The soundtrack should encode pedagogy. Example: when a player misspells a word and a mutant spawns, the music briefly becomes **dissonant**. When the word is corrected, the dissonance resolves. The game is teaching the player to *hear* correctness and corruption.

---

## 3. Implementation options

### Option A — Pre-generated stems + Bevy audio crossfader (safest)

What it is:
- Ship a small set of `.ogg`/`.wav` loops per game state and district.
- A `MusicState` resource crossfades between them.
- Uses the `bevy_audio` system already in `chat.rs` and `pet_reveal.rs`.

Pros:
- Works immediately with the current stack.
- Easy to tune, replace, and localize.
- No new dependencies.

Cons:
- Static; needs more assets to feel "procedural".
- Larger binary if many loops.

Best for: a fast first slice that makes `GameSettings.music_volume` do something.

### Option B — `bevy_kira_audio` 0.25 (the "radio" engine)

What it is:
- `bevy_kira_audio` 0.25 is built for Bevy 0.18.
- Kira is a game audio *arrangement* library: parameter tweens, looping, arrangement-based music, sound effects, and transitions.
- It can handle the interactive-music case naturally: start a backing loop, add/remove layers per state, change tempo/volume/pitch, and crossfade with parameters.

Pros:
- Purpose-built for adaptive game music.
- Can do "radio" style generative playlists and stems.
- `Parameter` system lets music follow FACES stats or attunement.

Cons:
- `bevy_kira_audio` cannot run alongside `bevy_audio` — you must disable `bevy_audio` and `bevy/vorbis` features and migrate SFX to Kira.
- Bigger refactor of all existing `AudioPlayer` calls in `chat.rs`, `pet_reveal.rs`, etc.

Best for: making music a first-class, interactive system.

### Option C — `fundsp` + `cpal` (fully procedural)

What it is:
- `fundsp` is a functional DSP audio library: oscillators, filters, envelopes, sequencers.
- You write the music as a Rust signal graph and feed it to `cpal` or a Bevy-compatible sink.

Pros:
- Music can be 100% generated from word data: frequency maps from letters, timbre from elements, rhythm from FACES action.
- Very small asset footprint.

Cons:
- Most complex to integrate with Bevy audio output.
- Cross-platform (WASM/Android/XR) output is non-trivial.
- Harder to tune than pre-generated assets.

Best for: a future "spell any word, hear its music" feature.

---

## 4. Recommended path

Start with **Option A** for the Tao of Fun slice:

1. Add a `MusicPlugin` and `MusicState` resource.
2. Define three simple generated loop assets:
   - `world_ambient.ogg`
   - `battle_tension.ogg`
   - `reveal_fanfare.ogg`
3. On `GameState` change, crossfade to the matching loop.
4. Respect `GameSettings.music_volume` and the existing `sound_volume` separation.
5. Add a `music.py` or `fundsp` generator script to `/scripts` so we can regenerate loops from word lists later.

Once that works, **migrate to Option B (`bevy_kira_audio`)** when the project is ready to make all audio interactive, because Kira's `Parameter`/`Arrangement` model matches the FACES/state-driven design perfectly.

---

## 5. Concrete first slice

Add a `src/core/music.rs` module with:

- `MusicState` resource tracking current track and target volume.
- `MusicPlugin` registered in `main.rs` and `lib.rs`.
- On `OnEnter(GameState::Playing)` start the world ambient loop.
- On `OnEnter(GameState::Battling)` crossfade to battle tension.
- On `OnEnter(GameState::RevealingPet)` play the reveal fanfare once.
- `Update` system fades volume toward `GameSettings.music_volume`.

This would reuse the existing `AudioPlayer` + `PlaybackSettings::LOOP` pattern, so it is a low-risk, high-fun addition.

---

## 6. VoixVive sounds worth stealing

| VoixVive sound | LitTCG equivalent |
|---|---|
| Generative drone engine | Companion / world ambience |
| Pothole `pling` | Correct word cast |
| Reference tone | Pet reveal pitch |
| Metronome | Spelling rhythm / battle turn pulse |
| Breath-state audio | Stillness before a hard word? |
| TTS in the teacher's voice | NPC dialogue could be sung/spoken |
| Ambisonic environments | District backgrounds (Zen Garden → Verdant) |

---

## 7. Next decision

The open question is asset generation:
- Do we **hand-author small loops** and ship them?
- Do we **generate them offline** with a Python music model (Stable Audio, MusicGen) tied to district themes?
- Or do we invest now in **Kira** and write procedural stems in Rust?

The safest next action is the `MusicPlugin` with three `.ogg` placeholders and a script to generate them.
