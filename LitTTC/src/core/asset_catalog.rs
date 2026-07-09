// assets.rs — Central catalog of embedded and runtime asset paths

#[cfg(not(feature = "flat2d"))]
pub const ASSETS_DIR: &str = "assets/";

// 2D UI textures
pub const CARD_BACKGROUND: &str = "ui/card_background.png";
#[cfg(not(feature = "xr"))]
pub const QUEST_BOARD: &str = "ui/quest_board.png";

// Character portraits
pub const BARNABY_AVATAR: &str = "textures/avatars/barnaby.png";

// Audio clips
pub const SOUND_PET: &str = "sounds/pet.ogg";
pub const SOUND_FEED: &str = "sounds/feed.ogg";
pub const SOUND_ATTUNE: &str = "sounds/attune.ogg";
#[cfg(any(not(feature = "tts"), target_arch = "wasm32"))]
pub const SOUND_BLIP: &str = "sounds/blip.ogg";

// Music stems
pub const MUSIC_MENU: &str = "sounds/music_menu.wav";
pub const MUSIC_WORLD: &str = "sounds/music_world.wav";
pub const MUSIC_BATTLE: &str = "sounds/music_battle.wav";

// TTS scratch file
#[cfg(all(feature = "tts", not(target_arch = "wasm32")))]
pub const TTS_OUTPUT_PATH: &str = "assets/sounds/tts_output.mp3";

// Embedded JSON databases
pub const WORD_DATABASE: &str = "word_database.json";
pub const SYNONYM_DATABASE: &str = "synonym_database.json";
pub const ETYMOLOGY_DATABASE: &str = "etymology_db.json";
pub const QUEST_DATABASE: &str = "quest_data.json";
pub const NPC_DATABASE: &str = "lore_db.json";

pub fn load_word_database() -> &'static str {
    include_str!("../../assets/word_database.json")
}

pub fn load_synonym_database() -> &'static str {
    include_str!("../../assets/synonym_database.json")
}

pub fn load_etymology_database() -> &'static str {
    include_str!("../../assets/etymology_db.json")
}

pub fn load_quest_database() -> &'static str {
    include_str!("../../assets/quest_data.json")
}

pub fn load_npc_database() -> &'static str {
    include_str!("../../assets/lore_db.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_constants_are_non_empty() {
        assert!(!CARD_BACKGROUND.is_empty());
        assert!(!BARNABY_AVATAR.is_empty());
        assert!(!SOUND_PET.is_empty());
        assert!(!WORD_DATABASE.is_empty());
    }

    #[test]
    fn embedded_databases_load_and_are_valid_json() {
        let word_json = load_word_database();
        let synonym_json = load_synonym_database();
        let etymology_json = load_etymology_database();
        let quest_json = load_quest_database();
        let npc_json = load_npc_database();

        assert!(serde_json::from_str::<serde_json::Value>(word_json).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(synonym_json).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(etymology_json).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(quest_json).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(npc_json).is_ok());
    }
}
