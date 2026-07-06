// database.rs — Deserializes and manages game metadata
#![allow(dead_code)]
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext, io::Reader, AsyncReadExt, AssetApp};
use bevy::reflect::TypePath;
use serde::Deserialize;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct RawJsonAsset {
    pub text: String,
}

#[derive(Default, TypePath)]
pub struct RawJsonLoader;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RawJsonLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF8 Error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl AssetLoader for RawJsonLoader {
    type Asset = RawJsonAsset;
    type Settings = ();
    type Error = RawJsonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = String::from_utf8(bytes)?;
        Ok(RawJsonAsset { text })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

pub struct DatabasePlugin;

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RawJsonAsset>()
           .init_asset_loader::<RawJsonLoader>();
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct WordStats {
    #[serde(rename = "C")]
    pub concreteness: f32,
    #[serde(rename = "AoA")]
    pub age_of_acquisition: u32,
    #[serde(rename = "V")]
    pub valence: f32,
    #[serde(rename = "A")]
    pub arousal: f32,
    #[serde(rename = "D")]
    pub dominance: f32,
    #[serde(rename = "GradeLevel")]
    pub grade_level: String,
    #[serde(rename = "CommonCoreStandard", default)]
    pub common_core: String,
    #[serde(rename = "SummonClass", default)]
    pub summon_class: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SynonymEntry {
    #[serde(rename = "Synonyms")]
    pub synonyms: Vec<String>,
    #[serde(rename = "Antonyms")]
    pub antonyms: Vec<String>,
    #[serde(rename = "Distractors")]
    pub distractors: Vec<String>,
    #[serde(rename = "Element")]
    pub element: String,
    #[serde(rename = "Difficulty")]
    pub difficulty: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RootData {
    #[serde(rename = "Element")]
    pub element: String,
    #[serde(rename = "StatFocus")]
    pub stat_focus: String,
    #[serde(rename = "Color")]
    pub color: [u8; 3],
    #[serde(rename = "Examples")]
    pub examples: Vec<String>,
    #[serde(rename = "SummonClass", default)]
    pub summon_class: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SuffixData {
    #[serde(rename = "Role")]
    pub role: String,
    #[serde(rename = "StatBonus")]
    pub stat_bonus: String,
    #[serde(rename = "Examples")]
    pub examples: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct EtymologyDB {
    #[serde(rename = "Roots")]
    pub roots: HashMap<String, RootData>,
    #[serde(rename = "Suffixes")]
    pub suffixes: HashMap<String, SuffixData>,
    #[serde(rename = "MorphemeWhitelist")]
    pub whitelist: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct QuestTemplate {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Template")]
    pub template: String,
    #[serde(rename = "RequiredRoles")]
    pub required_roles: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Rewards {
    #[serde(rename = "XP")]
    pub xp: u32,
    #[serde(rename = "Insight")]
    pub insight: u32,
    #[serde(rename = "EvolutionPoints")]
    pub evolution_points: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NpcQuest {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Template")]
    pub template: String,
    #[serde(rename = "Difficulty")]
    pub difficulty: u32,
    #[serde(rename = "Rewards")]
    pub rewards: Rewards,
}

#[derive(Deserialize, Clone, Debug)]
pub struct QuestData {
    #[serde(rename = "ArchetypeQuests")]
    pub archetype_quests: HashMap<String, Vec<QuestTemplate>>,
    #[serde(rename = "NPCChains")]
    pub npc_chains: HashMap<String, Vec<NpcQuest>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NpcDialogue {
    #[serde(rename = "Dawn")]
    pub dawn: Vec<String>,
    #[serde(rename = "Day")]
    pub day: Vec<String>,
    #[serde(rename = "Dusk")]
    pub dusk: Vec<String>,
    #[serde(rename = "Night")]
    pub night: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NpcData {
    #[serde(rename = "Archetype")]
    pub archetype: String,
    #[serde(rename = "District")]
    pub district: String,
    #[serde(rename = "PreferredElement")]
    pub preferred_element: Vec<String>,
    #[serde(rename = "PreferredClass")]
    pub preferred_class: Vec<String>,
    #[serde(rename = "Teaches")]
    pub teaches: Vec<String>,
    #[serde(rename = "EvolutionRole")]
    pub evolution_role: String,
    #[serde(rename = "Dialogue")]
    pub dialogue: NpcDialogue,
}

#[derive(Resource, Debug, Clone)]
pub struct GameDatabase {
    pub words: HashMap<String, WordStats>,
    pub synonyms: HashMap<String, SynonymEntry>,
    pub etymology: EtymologyDB,
    pub quests: QuestData,
    pub npcs: HashMap<String, NpcData>,
}

impl GameDatabase {
    pub fn parse_words(words_str: &str) -> HashMap<String, WordStats> {
        let mut words = HashMap::new();
        match serde_json::from_str::<HashMap<String, serde_json::Value>>(words_str) {
            Ok(raw_words) => {
                for (key, val) in raw_words {
                    match serde_json::from_value::<WordStats>(val) {
                        Ok(stats) => { words.insert(key, stats); },
                        Err(e) => bevy::log::warn!("Malformed word '{}' in word_database.json: {}", key, e),
                    }
                }
            }
            Err(e) => bevy::log::error!("Failed to parse word_database.json completely: {}", e),
        }
        words
    }

    pub fn parse_synonyms(syns_str: &str) -> HashMap<String, SynonymEntry> {
        let mut syns = HashMap::new();
        match serde_json::from_str::<HashMap<String, serde_json::Value>>(syns_str) {
            Ok(raw_syns) => {
                for (key, val) in raw_syns {
                    match serde_json::from_value::<SynonymEntry>(val) {
                        Ok(entry) => { syns.insert(key, entry); },
                        Err(e) => bevy::log::warn!("Malformed synonym '{}': {}", key, e),
                    }
                }
            }
            Err(e) => bevy::log::error!("Failed to parse synonym_database.json: {}", e),
        }
        syns
    }

    pub fn load_from_embedded() -> Result<Self, String> {
        Ok(Self {
            words: Self::parse_words(include_str!("../assets/word_database.json")),
            synonyms: Self::parse_synonyms(include_str!("../assets/synonym_database.json")),
            etymology: serde_json::from_str(include_str!("../assets/etymology_db.json")).map_err(|e| e.to_string())?,
            quests: serde_json::from_str(include_str!("../assets/quest_data.json")).map_err(|e| e.to_string())?,
            npcs: serde_json::from_str(include_str!("../assets/lore_db.json")).map_err(|e| e.to_string())?,
        })
    }
}
