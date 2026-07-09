// quest.rs — Mad-Lib Quest systems and NPC dialogue management
use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::*;
use crate::database::*;

#[derive(Resource, Debug, Clone)]
pub struct QuestSession {
    pub title: String,
    pub template: String,
    pub slots: Vec<String>, // e.g. ["ADJECTIVE", "NOUN", "VERB"]
    pub filled_slots: HashMap<usize, (String, Option<SummonClass>)>, // slot_index -> (word, summon_class)
    pub xp_reward: u32,
}

fn archetype_key(npc: &NpcData) -> String {
    npc.archetype
        .trim()
        .strip_prefix("The ")
        .unwrap_or(&npc.archetype)
        .to_string()
}

pub fn start_quest(
    npc_name: &str,
    db: &GameDatabase,
    grade_manager: &GradeManager,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
    state: &State<GameState>,
) {
    let archetype = db.npcs.get(npc_name).map(archetype_key).unwrap_or_default();

    let quests = db.quests.npc_chains.get(npc_name)
        .filter(|chain| !chain.is_empty())
        .or_else(|| db.quests.archetype_quests.get(&archetype));

    let Some(quests) = quests else {
        warn!("No quests found for NPC: {} (archetype: {})", npc_name, archetype);
        return;
    };

    let target_diff = grade_manager.active_grade;
    let quest = match quests.iter()
        .find(|q| q.difficulty == target_diff)
        .or_else(|| quests.iter().find(|q| q.difficulty <= target_diff))
        .or(quests.first()) {
            Some(q) => q,
            None => {
                warn!("No quests found for NPC: {}", npc_name);
                return;
            }
        };

    // Parse slots out of template (e.g. "{ADJECTIVE}")
    let mut slots = Vec::new();
    let mut temp_str = quest.template.clone();

    while let Some(start) = temp_str.find('{') {
        if let Some(end) = temp_str.find('}') {
            let slot_type = &temp_str[start+1..end];
            slots.push(slot_type.to_string());
            temp_str = temp_str[end+1..].to_string();
        } else {
            break;
        }
    }

    commands.insert_resource(QuestSession {
        title: quest.title.clone(),
        template: quest.template.clone(),
        slots,
        filled_slots: HashMap::new(),
        xp_reward: quest.rewards.xp,
    });

    info!("Quest begun: {} with {}", quest.title, npc_name);
    crate::commands::log_state_transition(state.get(), GameState::Questing);
    next_state.set(GameState::Questing);
}

pub fn fill_slot(
    slot_idx: usize,
    word: &str,
    summon_class: Option<SummonClass>,
    session: &mut QuestSession,
) {
    if slot_idx < session.slots.len() {
        session.filled_slots.insert(slot_idx, (word.to_string(), summon_class));
        info!("Filled quest slot {} with word: {}", slot_idx, word);
    }
}

pub fn complete_quest(
    session: &QuestSession,
    sheet: &mut CharacterSheet,
    spellbook: &mut SpellBook,
    grade_manager: &mut GradeManager,
    next_state: &mut NextState<GameState>,
    commands: &mut Commands,
    state: &State<GameState>,
) {
    if session.filled_slots.len() < session.slots.len() {
        warn!("Cannot finish verse: not all slots are filled!");
        return;
    }

    // Reconstruct the final text
    let mut final_text = session.template.clone();
    let mut bonus_xp = 0;

    for i in 0..session.slots.len() {
        let placeholder = format!("{{{}}}", session.slots[i]);
        let (replacement, summon_class) = session.filled_slots.get(&i).cloned().unwrap_or_default();
        final_text = final_text.replace(&placeholder, &replacement);
        
        // Upgrade mastery for the used words
        spellbook.upgrade_mastery(&replacement, MasteryLevel::Experienced);

        // Story Weave logic based on SummonClass
        if let Some(s_class) = summon_class {
            match s_class {
                SummonClass::SemanticSlime => {
                    info!("Semantic Slime consumed a word for evolution!");
                    bonus_xp += 5; // Slime bonus for word consumption
                },
            }
        }
    }

    info!("Quest Verse completed! Sentence: '{}'", final_text);
    
    // Award rewards
    sheet.total_xp += (session.xp_reward + bonus_xp) as u64;
    sheet.words_encountered += 1;

    // Check for rank up and unlock realm
    if grade_manager.check_grade_up(sheet.total_xp) {
        if let Some(district) = DISTRICTS.get((grade_manager.active_grade as usize).saturating_sub(1)) {
            if !grade_manager.unlocked_districts.contains(&district.to_string()) {
                grade_manager.unlocked_districts.push(district.to_string());
                info!("Realm Unlocked: {}", district);
            }
        }
    }

    commands.remove_resource::<QuestSession>();
    let next = if cfg!(feature = "flat2d") { GameState::Exploring } else { GameState::Playing };
    crate::commands::log_state_transition(state.get(), next.clone());
    next_state.set(next);
}

pub fn get_npc_dialogue(npc_name: &str, db: &GameDatabase, time_of_day: &str) -> String {
    if let Some(npc) = db.npcs.get(npc_name) {
        let dialogues = match time_of_day {
            "Dawn" => &npc.dialogue.dawn,
            "Day" => &npc.dialogue.day,
            "Dusk" => &npc.dialogue.dusk,
            "Night" => &npc.dialogue.night,
            _ => &npc.dialogue.day,
        };
        if let Some(dialogue) = dialogues.first() {
            return dialogue.clone();
        }
    }
    format!("Hello, I am {}.", npc_name)
}

/// Route player to appropriate NPC based on failed word for Tutor Loop
pub fn route_to_tutor_npc(failed_word: &str, db: &GameDatabase) -> String {
    // Simple routing: pick NPC based on word's etymology root or suffix
    let lower_word = failed_word.to_lowercase();
    
    // Check for etymology roots that map to NPCs
    for (root, data) in &db.etymology.roots {
        if lower_word.contains(&root.to_lowercase()) {
            // Map element to NPC (simplified routing)
            match data.element.as_str() {
                "Fire" => return "Ignis".to_string(), // Fire-related words → Ignis
                "Water" => return "Marina".to_string(),
                "Earth" => return "Terra".to_string(),
                "Air" => return "Aero".to_string(),
                "Light" => return "Lux".to_string(),
                "Shadow" => return "Umbra".to_string(),
                _ => {}
            }
        }
    }
    
    // Fallback: check suffixes
    for (suffix, data) in &db.etymology.suffixes {
        if lower_word.ends_with(&suffix.to_lowercase()) {
            // Map role to NPC
            match data.role.as_str() {
                "Tank" => return "Barnaby".to_string(),
                "Bruiser" => return "Kael".to_string(),
                "Striker" => return "Nyx".to_string(),
                "Caster" => return "Ozymandias".to_string(),
                "Support" => return "Martha".to_string(),
                _ => {}
            }
        }
    }
    
    // Default fallback
    "Barnaby".to_string()
}

pub const DISTRICTS: [&str; 12] = [
    "Garden District", // C
    "Outlaw Outpost",  // C#
    "Shadow Library",  // D
    "Great Railway",   // D#
    "Maintenance Bay", // E
    "Irony Junction",  // F
    "Adjective Valley",// F#
    "Central Station", // G
    "Metaphor Mountains", // G#
    "Logic Labyrinth", // A
    "Semantic Sea", // A#
    "Mastery Monolith", // B
];

#[derive(Resource)]
pub struct GradeManager {
    pub active_grade: u32,
    pub unlocked_districts: Vec<String>,
}

impl GradeManager {
    pub fn check_grade_up(&mut self, current_xp: u64) -> bool {
        let target_grade = (current_xp / 1000) as u32 + 1;
        if target_grade > self.active_grade {
            self.active_grade = target_grade;
            true
        } else {
            false
        }
    }

    pub fn get_valid_grade_levels(&self) -> Vec<&'static str> {
        let mut grades = vec!["K-2"];
        if self.active_grade >= 2 {
            grades.push("3-5");
        }
        if self.active_grade >= 3 {
            grades.push("6-8");
            grades.push("6-9");
        }
        if self.active_grade >= 4 {
            grades.push("9-10");
            grades.push("10-12");
            grades.push("11-12");
        }
        if self.active_grade >= 5 {
            grades.push("Graduate");
        }
        grades
    }
}

#[derive(Component)]
pub struct QuestUiPanel;

#[derive(Component)]
pub struct QuestUiText;

impl Default for GradeManager {
    fn default() -> Self {
        Self {
            active_grade: 1,
            unlocked_districts: vec![DISTRICTS[0].to_string()],
        }
    }
}

pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GradeManager>();
        #[cfg(feature = "xr")]
        app.add_systems(OnEnter(GameState::Questing), spawn_quest_ui_xr)
           .add_systems(Update, update_quest_ui_xr.run_if(in_state(GameState::Questing)))
           .add_systems(OnExit(GameState::Questing), cleanup_quest_ui_xr);

        #[cfg(not(feature = "xr"))]
        app.add_systems(OnEnter(GameState::Questing), spawn_quest_ui_2d)
           .add_systems(Update, update_quest_ui_2d.run_if(in_state(GameState::Questing)))
           .add_systems(OnExit(GameState::Questing), cleanup_quest_ui_2d);
    }
}

#[cfg(feature = "xr")]
fn spawn_quest_ui_xr(
    mut commands: Commands,
    session: Option<Res<QuestSession>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    let panel_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.05, 0.05, 0.1, 0.95),
        emissive: Color::srgba(0.05, 0.05, 0.1, 0.95).to_srgba().into(),
        metallic: 0.1,
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let panel = commands.spawn((
        QuestUiPanel,
        Mesh3d(meshes.add(Cuboid::new(3.0, 1.2, 0.05))),
        MeshMaterial3d(panel_mat),
        Transform::from_xyz(0.0, 2.0, -1.8),
    )).id();

    let text_entity = commands.spawn((
        QuestUiText,
        Text2d::new(format!("Quest: {}\n\n{}", session.title, get_display_sentence(&session))),
        TextFont { font_size: 26.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.0, 0.03),
    )).id();

    commands.entity(panel).add_child(text_entity);
}

fn get_display_sentence(session: &QuestSession) -> String {
    let mut display_text = session.template.clone();
    for i in 0..session.slots.len() {
        let placeholder = format!("{{{}}}", session.slots[i]);
        let replacement = if let Some((word, _)) = session.filled_slots.get(&i) {
            word.to_uppercase()
        } else {
            format!("[{}]", session.slots[i])
        };
        display_text = display_text.replace(&placeholder, &replacement);
    }
    display_text
}

#[cfg(feature = "xr")]
fn update_quest_ui_xr(
    session: Option<Res<QuestSession>>,
    mut text_query: Query<&mut Text2d, With<QuestUiText>>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    for mut text in &mut text_query {
        text.0 = format!("Quest: {}\n\n{}", session.title, get_display_sentence(&session));
    }
}

#[cfg(feature = "xr")]
fn cleanup_quest_ui_xr(
    mut commands: Commands,
    query: Query<Entity, Or<(With<QuestUiPanel>, With<QuestUiText>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "xr"))]
fn spawn_quest_ui_2d(
    mut commands: Commands,
    session: Option<Res<QuestSession>>,
    #[cfg(not(feature = "flat2d"))] asset_server: Res<AssetServer>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    #[cfg(feature = "flat2d")]
    commands.spawn((
        QuestUiPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-300.0)),
            padding: UiRect::all(Val::Px(60.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.2)),
        BorderColor::all(Color::srgb(0.4, 0.4, 0.9)),
    )).with_children(|parent| {
        parent.spawn((
            QuestUiText,
            Text::new(format!("Quest: {}\n\n{}\n\n[Click cards & click Play Card button to place, click Play Card to Submit when full]", session.title, get_display_sentence(&session))),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });

    #[cfg(not(feature = "flat2d"))]
    commands.spawn((
        QuestUiPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-300.0)),
            padding: UiRect::all(Val::Px(60.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode::new(asset_server.load(crate::asset_catalog::QUEST_BOARD)),
    )).with_children(|parent| {
        parent.spawn((
            QuestUiText,
            Text::new(format!("Quest: {}\n\n{}\n\n[Click cards & click Play Card button to place, click Play Card to Submit when full]", session.title, get_display_sentence(&session))),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

#[cfg(not(feature = "xr"))]
fn update_quest_ui_2d(
    session: Option<Res<QuestSession>>,
    mut text_query: Query<&mut Text, With<QuestUiText>>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    for mut text in &mut text_query {
        text.0 = format!("Quest: {}\n\n{}\n\n[Click cards & click Play Card button to place, click Play Card to Submit when full]", session.title, get_display_sentence(&session));
    }
}

#[cfg(not(feature = "xr"))]
fn cleanup_quest_ui_2d(
    mut commands: Commands,
    query: Query<Entity, With<QuestUiPanel>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_slot_inserts_word_when_in_bounds() {
        let mut session = QuestSession {
            title: "Test".to_string(),
            template: "I feel {ADJECTIVE}.".to_string(),
            slots: vec!["ADJECTIVE".to_string()],
            filled_slots: HashMap::new(),
            xp_reward: 10,
        };
        fill_slot(0, "brave", None, &mut session);
        assert_eq!(session.filled_slots.get(&0).unwrap().0, "brave");
    }

    #[test]
    fn fill_slot_ignores_out_of_bounds_index() {
        let mut session = QuestSession {
            title: "Test".to_string(),
            template: "I feel {ADJECTIVE}.".to_string(),
            slots: vec!["ADJECTIVE".to_string()],
            filled_slots: HashMap::new(),
            xp_reward: 10,
        };
        fill_slot(5, "brave", None, &mut session);
        assert!(session.filled_slots.is_empty());
    }

    #[test]
    fn get_display_sentence_replaces_unfilled_with_placeholder() {
        let session = QuestSession {
            title: "Test".to_string(),
            template: "I feel {ADJECTIVE}.".to_string(),
            slots: vec!["ADJECTIVE".to_string()],
            filled_slots: HashMap::new(),
            xp_reward: 10,
        };
        let text = get_display_sentence(&session);
        assert_eq!(text, "I feel [ADJECTIVE].");
    }

    #[test]
    fn get_display_sentence_fills_known_slots_in_uppercase() {
        let mut session = QuestSession {
            title: "Test".to_string(),
            template: "I feel {ADJECTIVE}.".to_string(),
            slots: vec!["ADJECTIVE".to_string()],
            filled_slots: HashMap::new(),
            xp_reward: 10,
        };
        fill_slot(0, "brave", None, &mut session);
        let text = get_display_sentence(&session);
        assert_eq!(text, "I feel BRAVE.");
    }

    #[test]
    fn grade_manager_default_is_rank_one() {
        let gm = GradeManager::default();
        assert_eq!(gm.active_grade, 1);
        assert_eq!(gm.unlocked_districts, vec!["Garden District"]);
    }

    #[test]
    fn check_grade_up_advances_at_thresholds() {
        let mut gm = GradeManager::default();
        assert!(!gm.check_grade_up(500));
        assert!(gm.check_grade_up(1000));
        assert_eq!(gm.active_grade, 2);
    }

    #[test]
    fn get_valid_grade_levels_unlocks_progressively() {
        let mut gm = GradeManager::default();
        assert_eq!(gm.get_valid_grade_levels(), vec!["K-2"]);
        gm.active_grade = 2;
        assert!(gm.get_valid_grade_levels().contains(&"3-5"));
        gm.active_grade = 3;
        assert!(gm.get_valid_grade_levels().contains(&"6-8"));
    }
}
