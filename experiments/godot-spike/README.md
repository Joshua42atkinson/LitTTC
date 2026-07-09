# LitTCG Godot 2D 8-bit Prototype

This is a 2D 8-bit-style version of LitTCG built in Godot 4.4.1.
It reuses the existing LitTCG JSON data (words, synonyms, etymology, quests, NPCs) and is designed to be XR-ready later via Godot's OpenXR support.

## Project layout

- `project.godot` — Engine config (256×224 viewport, nearest-neighbor scaling, Mobile renderer)
- `assets/data/` — Symlinks to `LitTTC/assets/*.json` for live data
- `assets/sounds/` — Symlinks to `LitTTC/assets/sounds/*`
- `scripts/` — Autoloads (`Database`, `GameState`) and helpers
- `scenes/` — Game screens (main menu first)
- `ui/` — 8-bit UI themes and widgets
- `sprites/` — Pixel-art sprites and animations

## Running the project

```bash
# From the repo root
godot --path godot
```

## Current scope

1. Database autoload with JSON ingest.
2. Main menu with Play / Pets / Settings buttons.
3. 2D collecting, spelling, summoning, battle, questing loop.
4. OpenXR export configuration.

## Data sources

All data is pulled from `LitTTC/assets/`:

- `word_database.json` — psycholinguistic word stats
- `synonym_database.json` — synonyms and elements
- `etymology_db.json` — roots and stat focus
- `quest_data.json` — NPC quest chains and archetype quest pools
- `lore_db.json` — NPC lore and dialogue
