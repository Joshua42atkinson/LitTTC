// database.rs — Deserializes and manages game metadata
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext, io::Reader, AssetApp};
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
    #[serde(rename = "V")]
    pub valence: f32,
    #[serde(rename = "A", alias = "arousal")]
    pub intensity: f32,
    #[serde(rename = "D")]
    pub dominance: f32,
    #[serde(rename = "GradeLevel")]
    pub grade_level: String,
    #[serde(rename = "POS", default)]
    pub part_of_speech: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SynonymEntry {
    #[serde(rename = "Synonyms")]
    pub synonyms: Vec<String>,
    #[serde(rename = "Element")]
    pub element: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RootData {
    #[serde(rename = "Element")]
    pub element: String,
    #[serde(rename = "StatFocus")]
    pub stat_focus: String,
    #[serde(rename = "Color")]
    pub color: [u8; 3],
}

#[derive(Deserialize, Clone, Debug)]
pub struct SuffixData {
    #[serde(rename = "Role")]
    pub role: String,
}

#[derive(Deserialize, Clone, Debug)]
#[derive(Default)]
pub struct EtymologyDB {
    #[serde(rename = "Roots")]
    pub roots: HashMap<String, RootData>,
    #[serde(rename = "Suffixes")]
    pub suffixes: HashMap<String, SuffixData>,
}


#[derive(Deserialize, Clone, Debug, Default)]
pub struct Rewards {
    #[serde(rename = "XP")]
    #[serde(default)]
    pub xp: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NpcQuest {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Template")]
    pub template: String,
    #[serde(rename = "Difficulty")]
    #[serde(default)]
    pub difficulty: u32,
    #[serde(rename = "Rewards")]
    #[serde(default)]
    pub rewards: Rewards,
}

#[derive(Deserialize, Clone, Debug)]
#[derive(Default)]
pub struct QuestData {
    #[serde(rename = "NPCChains")]
    pub npc_chains: HashMap<String, Vec<NpcQuest>>,
    #[serde(rename = "ArchetypeQuests")]
    pub archetype_quests: HashMap<String, Vec<NpcQuest>>,
}


#[derive(Deserialize, Clone, Debug, Default)]
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

#[derive(Deserialize, Clone, Debug, Default)]
pub struct NpcData {
    #[serde(rename = "AvatarPath")]
    pub avatar_path: Option<String>,
    #[serde(rename = "Dialogue")]
    pub dialogue: NpcDialogue,
    #[serde(rename = "Archetype")]
    #[serde(default)]
    pub archetype: String,
}

#[derive(Resource, Debug, Clone)]
#[derive(Default)]
pub struct GameDatabase {
    pub words: HashMap<String, WordStats>,
    pub synonyms: HashMap<String, SynonymEntry>,
    pub etymology: EtymologyDB,
    pub quests: QuestData,
    pub npcs: HashMap<String, NpcData>,
}


impl GameDatabase {
    /// Parses the word_database.json string into a map of word -> psycholinguistic stats.
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

    /// Parses the synonym_database.json string into a map of word -> synonyms/antonyms/distractors.
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

    /// Loads all five embedded JSON databases (words, synonyms, etymology, quests, NPCs).
    pub fn load_from_embedded() -> Result<Self, String> {
        Ok(Self {
            words: Self::parse_words(crate::asset_catalog::load_word_database()),
            synonyms: Self::parse_synonyms(crate::asset_catalog::load_synonym_database()),
            etymology: serde_json::from_str(crate::asset_catalog::load_etymology_database()).map_err(|e| e.to_string())?,
            quests: serde_json::from_str(crate::asset_catalog::load_quest_database()).map_err(|e| e.to_string())?,
            npcs: serde_json::from_str(crate::asset_catalog::load_npc_database()).map_err(|e| e.to_string())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_words_extracts_expected_fields() {
        let json = r#"{"abandoned":{"C":2.0,"V":3.0,"A":4.0,"D":5.0,"GradeLevel":"6-8"}}"#;
        let words = GameDatabase::parse_words(json);
        let stats = words.get("abandoned").expect("word should parse");
        assert!((stats.concreteness - 2.0).abs() < f32::EPSILON);
        assert!((stats.valence - 3.0).abs() < f32::EPSILON);
        assert!((stats.intensity - 4.0).abs() < f32::EPSILON);
        assert!((stats.dominance - 5.0).abs() < f32::EPSILON);
        assert_eq!(stats.grade_level, "6-8");
    }

    #[test]
    fn parse_words_skips_malformed_entries() {
        let json = r#"{"good":{"C":1.0,"V":2.0,"A":3.0,"D":4.0,"GradeLevel":"K-2"},"bad":"not an object"}"#;
        let words = GameDatabase::parse_words(json);
        assert_eq!(words.len(), 1);
        assert!(words.contains_key("good"));
    }

    #[test]
    fn parse_synonyms_extracts_element_and_synonyms() {
        let json = r#"{"abandoned":{"Synonyms":["deserted","forsaken"],"Element":"Fire"}}"#;
        let syns = GameDatabase::parse_synonyms(json);
        let entry = syns.get("abandoned").expect("synonym should parse");
        assert_eq!(entry.element, "Fire");
        assert_eq!(entry.synonyms, vec!["deserted", "forsaken"]);
    }

    #[test]
    fn load_from_embedded_populates_all_collections() {
        let db = GameDatabase::load_from_embedded().expect("embedded databases should load");
        assert!(!db.words.is_empty());
        assert!(!db.synonyms.is_empty());
        assert!(!db.etymology.roots.is_empty());
        assert!(!db.quests.npc_chains.is_empty());
        assert!(!db.quests.archetype_quests.is_empty(), "Archetype quest pools should load");
        assert!(!db.npcs.is_empty());

        // Every NPC should have an archetype and at least one quest chain
        for (name, npc) in &db.npcs {
            assert!(!npc.archetype.is_empty(), "NPC {} should have an archetype", name);
            let archetype = npc.archetype.trim().strip_prefix("The ").unwrap_or(&npc.archetype);
            assert!(
                db.quests.npc_chains.contains_key(name) || db.quests.archetype_quests.contains_key(archetype),
                "NPC {} (archetype {}) should have a quest chain",
                name,
                archetype
            );
        }
    }
}

#[derive(Resource)]
pub struct LoadingDatabases {
    pub words: Handle<RawJsonAsset>,
    pub syns: Handle<RawJsonAsset>,
    pub etym: Handle<RawJsonAsset>,
    pub quest: Handle<RawJsonAsset>,
    pub npcs: Handle<RawJsonAsset>,
}

impl LoadingDatabases {
    const TOTAL: usize = 5;

    /// Count how many of the tracked database assets have finished loading.
    pub fn loaded_count(&self, assets: &Assets<RawJsonAsset>) -> usize {
        let mut count = 0;
        if assets.contains(&self.words) { count += 1; }
        if assets.contains(&self.syns) { count += 1; }
        if assets.contains(&self.etym) { count += 1; }
        if assets.contains(&self.quest) { count += 1; }
        if assets.contains(&self.npcs) { count += 1; }
        count
    }
}

#[derive(Component)]
pub struct LoadingUiRoot;

#[derive(Component)]
pub struct LoadingProgressBar;

#[derive(Component)]
pub struct LoadingStatusText;

pub fn start_loading_database(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("Starting database asset loading...");
    commands.insert_resource(LoadingDatabases {
        words: asset_server.load(crate::asset_catalog::WORD_DATABASE),
        syns: asset_server.load(crate::asset_catalog::SYNONYM_DATABASE),
        etym: asset_server.load(crate::asset_catalog::ETYMOLOGY_DATABASE),
        quest: asset_server.load(crate::asset_catalog::QUEST_DATABASE),
        npcs: asset_server.load(crate::asset_catalog::NPC_DATABASE),
    });
}

pub fn check_database_loading(
    mut commands: Commands,
    loading: Res<LoadingDatabases>,
    assets: Res<Assets<RawJsonAsset>>,
    mut next_state: ResMut<NextState<crate::components::GameState>>,
    state: Res<State<crate::components::GameState>>,
) {
    let words = assets.get(&loading.words);
    let syns = assets.get(&loading.syns);
    let etym = assets.get(&loading.etym);
    let quest = assets.get(&loading.quest);
    let npcs = assets.get(&loading.npcs);

    if let (Some(w), Some(s), Some(e), Some(q), Some(n)) = (words, syns, etym, quest, npcs) {
        info!("All database assets loaded! Parsing...");
        let words = GameDatabase::parse_words(&w.text);
        let synonyms = GameDatabase::parse_synonyms(&s.text);

        let etymology: EtymologyDB = match serde_json::from_str(&e.text) {
            Ok(data) => data,
            Err(err) => {
                warn!("Failed to parse runtime etymology_db.json: {}. Falling back to embedded.", err);
                GameDatabase::load_from_embedded().map(|db| db.etymology).unwrap_or_default()
            }
        };

        let quests: QuestData = match serde_json::from_str(&q.text) {
            Ok(data) => data,
            Err(err) => {
                warn!("Failed to parse runtime quest_data.json: {}. Falling back to embedded.", err);
                GameDatabase::load_from_embedded().map(|db| db.quests).unwrap_or_default()
            }
        };

        let npcs: HashMap<String, NpcData> = match serde_json::from_str(&n.text) {
            Ok(data) => data,
            Err(err) => {
                warn!("Failed to parse runtime lore_db.json: {}. Falling back to embedded.", err);
                GameDatabase::load_from_embedded().map(|db| db.npcs).unwrap_or_default()
            }
        };

        let db = GameDatabase {
            words,
            synonyms,
            etymology,
            quests,
            npcs,
        };

        validate_database(&db);
        commands.insert_resource(db);

        info!("Database parsed successfully. Transitioning to MainMenu.");
        crate::commands::log_state_transition(state.get(), crate::components::GameState::MainMenu);
        next_state.set(crate::components::GameState::MainMenu);
    }
}

fn validate_database(db: &GameDatabase) {
    let mut empty_synonyms = 0usize;
    for (word, entry) in &db.synonyms {
        if entry.element.is_empty() {
            warn!("Synonym entry '{}' has no element", word);
        }
        if entry.synonyms.is_empty() {
            empty_synonyms += 1;
        }
    }

    let mut roots_without_color = 0usize;
    for (root, data) in &db.etymology.roots {
        if data.stat_focus.is_empty() {
            warn!("Root '{}' has no stat focus", root);
        }
        if data.color.len() != 3 {
            roots_without_color += 1;
        }
    }

    info!(
        "Database validation: {} words, {} synonyms, {} etymology roots, {} roots with missing color",
        db.words.len(),
        db.synonyms.len(),
        db.etymology.roots.len(),
        roots_without_color
    );

    if empty_synonyms > 0 {
        warn!("{} synonym entries have empty synonym lists", empty_synonyms);
    }
}

pub fn hot_reload_database(
    mut events: MessageReader<AssetEvent<RawJsonAsset>>,
    loading: Option<Res<LoadingDatabases>>,
    assets: Res<Assets<RawJsonAsset>>,
    db: Option<ResMut<GameDatabase>>,
) {
    let loading = match loading {
        Some(l) => l,
        None => return,
    };
    let mut db = match db {
        Some(d) => d,
        None => return,
    };

    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            if *id == loading.words.id() {
                if let Some(asset) = assets.get(*id) {
                    info!("Hot-reloading word_database.json!");
                    db.words = GameDatabase::parse_words(&asset.text);
                }
            } else if *id == loading.syns.id() {
                if let Some(asset) = assets.get(*id) {
                    info!("Hot-reloading synonym_database.json!");
                    db.synonyms = GameDatabase::parse_synonyms(&asset.text);
                }
            } else if *id == loading.etym.id() {
                if let Some(asset) = assets.get(*id) {
                    info!("Hot-reloading etymology_db.json!");
                    if let Ok(etym) = serde_json::from_str(&asset.text) {
                        db.etymology = etym;
                    }
                }
            } else if *id == loading.quest.id() {
                if let Some(asset) = assets.get(*id) {
                    info!("Hot-reloading quest_data.json!");
                    if let Ok(q) = serde_json::from_str(&asset.text) {
                        db.quests = q;
                    }
                }
            } else if *id == loading.npcs.id() {
                if let Some(asset) = assets.get(*id) {
                    info!("Hot-reloading lore_db.json!");
                    if let Ok(n) = serde_json::from_str(&asset.text) {
                        db.npcs = n;
                    }
                }
            }
        }
    }
}

/// Spawns a branded loading screen with a progress bar and status text.
pub fn spawn_loading_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.04, 0.08, 1.0)),
        LoadingUiRoot,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Summoning vocabulary..."),
            TextFont { font_size: 36.0, ..default() },
            TextColor(Color::srgb(0.8, 0.7, 1.0)),
            Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
        ));

        // Progress bar background
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        )).with_children(|parent| {
            // Progress bar fill
            parent.spawn((
                LoadingProgressBar,
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.4, 0.3, 0.8)),
            ));
        });

        parent.spawn((
            LoadingStatusText,
            Text::new("Loading 0 / 5 databases..."),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.8)),
            Node { margin: UiRect::top(Val::Px(16.0)), ..default() },
        ));
    });
}

/// Updates the loading bar width and status text based on how many assets are ready.
pub fn update_loading_progress(
    loading: Res<LoadingDatabases>,
    assets: Res<Assets<RawJsonAsset>>,
    mut bar: Query<&mut Node, With<LoadingProgressBar>>,
    mut text: Query<&mut Text, With<LoadingStatusText>>,
) {
    let loaded = loading.loaded_count(&assets);
    let percent = (loaded as f32 / LoadingDatabases::TOTAL as f32) * 100.0;

    if let Ok(mut bar) = bar.single_mut() {
        bar.width = Val::Percent(percent);
    }
    if let Ok(mut text) = text.single_mut() {
        *text = Text::new(format!("Loading {} / {} databases...", loaded, LoadingDatabases::TOTAL));
    }
}

/// Removes the loading screen once all databases are loaded.
pub fn cleanup_loading_ui(
    mut commands: Commands,
    roots: Query<Entity, With<LoadingUiRoot>>,
) {
    for entity in roots.iter() {
        commands.entity(entity).despawn();
    }
}
