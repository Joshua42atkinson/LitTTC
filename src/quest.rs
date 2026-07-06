// quest.rs — Mad-Lib Quest systems and NPC dialogue management
#![allow(dead_code)]
use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::*;
use crate::database::*;

#[derive(Resource, Debug, Clone)]
pub struct QuestSession {
    pub npc_name: String,
    pub title: String,
    pub template: String,
    pub slots: Vec<String>, // e.g. ["ADJECTIVE", "NOUN", "VERB"]
    pub filled_slots: HashMap<usize, (String, Option<SummonClass>)>, // slot_index -> (word, summon_class)
    pub xp_reward: u32,
    pub evolution_reward: u32,
}

pub fn start_quest(
    npc_name: &str,
    db: &GameDatabase,
    curriculum: &CurriculumManager,
    commands: &mut Commands,
    next_state: &mut NextState<GameState>,
) {
    if let Some(quests) = db.quests.npc_chains.get(npc_name) {
        let target_diff = curriculum.active_grade;
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
            npc_name: npc_name.to_string(),
            title: quest.title.clone(),
            template: quest.template.clone(),
            slots,
            filled_slots: HashMap::new(),
            xp_reward: quest.rewards.xp,
            evolution_reward: quest.rewards.evolution_points,
        });

        info!("Started quest: {} from NPC: {}", quest.title, npc_name);
        next_state.set(GameState::Questing);
    }
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
    next_state: &mut NextState<GameState>,
    commands: &mut Commands,
) {
    if session.filled_slots.len() < session.slots.len() {
        warn!("Cannot complete quest: not all slots are filled!");
        return;
    }

    // Reconstruct the final text
    let mut final_text = session.template.clone();
    let mut bonus_xp = 0;
    #[allow(unused_variables)]
    let mut bonus_evolution = 0;

    for i in 0..session.slots.len() {
        let placeholder = format!("{{{}}}", session.slots[i]);
        let (replacement, summon_class) = session.filled_slots.get(&i).cloned().unwrap_or_default();
        final_text = final_text.replace(&placeholder, &replacement);
        
        // Upgrade mastery for the used words
        spellbook.upgrade_mastery(&replacement, MasteryLevel::Experienced);

        // Meme Template Logic based on SummonClass
        if let Some(s_class) = summon_class {
            match s_class {
                SummonClass::SemanticSlime => {
                    info!("Semantic Slime consumed a word for evolution!");
                    bonus_evolution += 5; // Extra evolution points for fluid semantics
                },
                SummonClass::GrammarGolem => {
                    info!("Grammar Golem reinforces syntax!");
                    bonus_xp += 10; // Extra XP for correct grammar usage
                },
                SummonClass::RhetoricRobot => {
                    info!("Rhetoric Robot applies a social buff!");
                    bonus_xp += 15; // Represents manipulating social combat
                }
            }
        }
    }

    info!("Meme Template Completed! Sentence: '{}'", final_text);
    
    // Award rewards
    sheet.total_xp += (session.xp_reward + bonus_xp) as u64;
    sheet.words_encountered += 1;

    commands.remove_resource::<QuestSession>();
    next_state.set(GameState::Playing);
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

#[derive(Resource, Debug, Clone, Default)]
pub struct CurriculumManager {
    pub active_grade: u32,
    pub progress_xp: u64,
}

impl CurriculumManager {
    pub fn get_grade_level(&self) -> String {
        format!("Grade {}", self.active_grade)
    }

    pub fn check_grade_up(&mut self, current_xp: u64) -> bool {
        let target_grade = (current_xp / 1000) as u32 + 1;
        if target_grade > self.active_grade {
            self.active_grade = target_grade;
            true
        } else {
            false
        }
    }
}

#[derive(Component)]
pub struct QuestUiPanel;

#[derive(Component)]
pub struct QuestUiText;

pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
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
    asset_server: Res<AssetServer>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

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
        ImageNode::new(asset_server.load("ui/quest_board.png")),
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
