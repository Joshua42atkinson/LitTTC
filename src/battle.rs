// battle.rs — Turn-based synonym/antonym card combat against wild typos
#![allow(dead_code)]
use bevy::prelude::*;
use crate::components::*;
use crate::database::*;

#[derive(Resource, Debug, Clone)]
pub struct BattleSession {
    pub typo_word: String,
    pub typo_health: i32,
    pub player_health: i32,
}

#[derive(Component)]
pub struct CriticalHitTrigger;

pub fn semantic_distance(a: &WordStats, b: &WordStats) -> f32 {
    let dc = a.concreteness - b.concreteness;
    let dv = a.valence - b.valence;
    let dd = a.dominance - b.dominance;
    let da = a.intensity - b.intensity;
    (dc*dc + dv*dv + dd*dd + da*da).sqrt()
}

pub fn start_battle(
    commands: &mut Commands,
    db: &GameDatabase,
    curriculum: &crate::quest::CurriculumManager,
    next_state: &mut NextState<GameState>,
    state: &State<GameState>,
) {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let valid_grades = curriculum.get_valid_grade_levels();

    let valid_words: Vec<&String> = db.words.iter()
        .filter(|(_, stats)| valid_grades.contains(&stats.grade_level.as_str()))
        .map(|(word, _)| word)
        .collect();

    let mut typo_word = "inferno".to_string(); // fallback
    if let Some(&word) = valid_words.choose(&mut thread_rng()) {
        typo_word = word.clone();
    }

    commands.insert_resource(BattleSession {
        typo_word: typo_word.clone(),
        typo_health: 50,
        player_health: 100,
    });

    info!("A wild Typo ({}) emerges! Deduce its semantic weakness based on its stats!", typo_word.to_uppercase());
    crate::commands::log_state_transition(state.get(), GameState::Battling);
    next_state.set(GameState::Battling);
}

pub struct BattleResult {
    pub is_effective: bool,
    pub social_combat_triggered: bool,
    pub is_synonym_logic: bool,
}

pub fn play_battle_card(
    played_word: &str,
    session: &mut BattleSession,
    db: &GameDatabase,
    spellbook: &mut SpellBook,
    next_state: &mut NextState<GameState>,
    sheet: &CharacterSheet,
    state: &State<GameState>,
) -> BattleResult {
    let lower_typo = session.typo_word.to_lowercase();
    let lower_played = played_word.to_lowercase();

    let mut damage_multiplier = 1.0;
    let mut is_effective = false;
    let mut social_combat_triggered = false;
    let mut is_synonym_logic = false;

    if let (Some(typo_stats), Some(played_stats)) = (db.words.get(&lower_typo), db.words.get(&lower_played)) {
        if sheet.active_summon_class == SummonClass::RhetoricRobot {
            let logos_diff = (typo_stats.concreteness - played_stats.concreteness).abs();
            let pathos_diff = (((typo_stats.intensity + typo_stats.valence) / 2.0) - ((played_stats.intensity + played_stats.valence) / 2.0)).abs();
            let ethos_diff = (typo_stats.dominance - played_stats.dominance).abs();

            social_combat_triggered = true;

            if logos_diff > pathos_diff && logos_diff > ethos_diff {
                // Structural Paradox (Logos dominant)
                is_synonym_logic = false;
                damage_multiplier = 2.5;
                is_effective = true;
            } else {
                // Semantic Equivalence (Pathos/Ethos dominant)
                is_synonym_logic = true;
                damage_multiplier = 2.5;
                is_effective = true;
            }
        } else if sheet.active_summon_class == SummonClass::GrammarGolem {
            let mut shared_root = false;
            let mut shared_suffix = false;
            let mut has_own_root = false;
            let mut has_own_suffix = false;

            // Check roots
            for root in db.etymology.roots.keys() {
                let root_lower = root.to_lowercase();
                let played_has = lower_played.contains(&root_lower);
                let typo_has = lower_typo.contains(&root_lower);
                if played_has {
                    has_own_root = true;
                }
                if played_has && typo_has {
                    shared_root = true;
                }
            }

            // Check suffixes
            for suffix in db.etymology.suffixes.keys() {
                let suffix_lower = suffix.to_lowercase();
                let played_has = lower_played.ends_with(&suffix_lower);
                let typo_has = lower_typo.ends_with(&suffix_lower);
                if played_has {
                    has_own_suffix = true;
                }
                if played_has && typo_has {
                    shared_suffix = true;
                }
            }

            if shared_root { damage_multiplier += 0.5; }
            if shared_suffix { damage_multiplier += 0.5; }

            if has_own_root && has_own_suffix {
                // Perfect grammatical integrity!
                damage_multiplier += 0.5; 
            }
            
            if damage_multiplier > 1.0 {
                is_effective = true;
            } else {
                damage_multiplier = 0.5;
            }
        } else {
            let distance = semantic_distance(typo_stats, played_stats);
            
            // Emergent counters: high distance = opposing concepts (e.g. Fire vs Water)
            if distance > 4.0 {
                damage_multiplier = 1.5 + (distance - 4.0) * 0.2;
                is_effective = true;
            } else if distance < 2.0 {
                damage_multiplier = 0.5;
            }
        }
    }

    let base_damage = 25.0;
    let final_damage = (base_damage * damage_multiplier) as i32;

    if is_effective {
        session.typo_health -= final_damage;
        info!("CRITICAL HIT! Semantic distance multiplier: {:.2}x. Typo health: {}", damage_multiplier, session.typo_health);
        spellbook.upgrade_mastery(played_word, MasteryLevel::Owned);
    } else {
        session.typo_health -= final_damage;
        session.player_health -= 20;
        warn!("INEFFECTIVE! Damage multiplier: {:.2}x. Typo counters! Player health: {}", damage_multiplier, session.player_health);
    }

    if session.typo_health <= 0 {
        info!("Victory! The Typo has been corrected and clean spelling returns to the sector.");
        spellbook.upgrade_mastery(played_word, MasteryLevel::Mastered);
        crate::commands::log_state_transition(state.get(), GameState::Reviewing);
        next_state.set(GameState::Reviewing);
    } else if session.player_health <= 0 {
        warn!("Defeat! The Typo overrode your spelling defense. Retreating to town square.");
        crate::commands::log_state_transition(state.get(), GameState::Playing);
        next_state.set(GameState::Playing);
    }

    BattleResult {
        is_effective,
        social_combat_triggered,
        is_synonym_logic,
    }
}

#[derive(Component)]
pub struct PlayerHealthBar;

#[derive(Component)]
pub struct EnemyHealthBar;

#[derive(Component)]
pub struct BattleUiMarker;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "xr")]
        app.add_systems(OnEnter(GameState::Battling), (spawn_battle_ui_xr, set_pet_battle_state))
           .add_systems(Update, update_battle_hp_bars_xr.run_if(in_state(GameState::Battling)))
           .add_systems(OnExit(GameState::Battling), (cleanup_battle_ui_xr, set_pet_idle_state));

        #[cfg(not(feature = "flat2d"))]
        app.add_systems(Update, handle_critical_hit_effects);

        #[cfg(not(feature = "xr"))]
        app.add_systems(OnEnter(GameState::Battling), (spawn_battle_ui_2d, set_pet_battle_state))
           .add_systems(Update, update_battle_hp_bars_2d.run_if(in_state(GameState::Battling)))
           .add_systems(OnExit(GameState::Battling), (cleanup_battle_ui_2d, set_pet_idle_state));
    }
}

#[cfg(not(feature = "flat2d"))]
pub fn handle_critical_hit_effects(
    trigger_query: Query<Entity, With<CriticalHitTrigger>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    for trigger_entity in trigger_query.iter() {
        commands.entity(trigger_entity).despawn();
        
        for entity in &camera_query {
            commands.entity(entity).insert(crate::render::ScreenShake { timer: 0.3, intensity: 0.2 });
        }
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..30 {
            let vx = rng.gen_range(-4.0..4.0);
            let vy = rng.gen_range(2.0..6.0);
            let vz = rng.gen_range(-4.0..4.0);
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.06).mesh().ico(1).unwrap())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.9, 0.1),
                    emissive: Color::srgb(2.0, 1.8, 0.2).into(),
                    ..default()
                })),
                Transform::from_xyz(0.0, 1.5, -2.0),
                crate::render::BurstParticle {
                    velocity: Vec3::new(vx, vy, vz),
                    timer: 1.5,
                }
            ));
        }
    }
}

fn set_pet_battle_state(
    mut query: Query<&mut PetVisualState, With<PetAvatar>>,
) {
    for mut state in &mut query {
        *state = PetVisualState::Battle;
    }
}

fn set_pet_idle_state(
    mut query: Query<&mut PetVisualState, With<PetAvatar>>,
) {
    for mut state in &mut query {
        *state = PetVisualState::Idle;
    }
}

#[cfg(feature = "xr")]
fn spawn_battle_ui_xr(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    session: Res<BattleSession>,
) {
    let instruction_text = format!("WILD TYPO: {}", session.typo_word.to_uppercase());
    commands.spawn((
        BattleUiMarker,
        Text2d::new(instruction_text),
        TextFont { font_size: 36.0, ..default() },
        TextColor(Color::srgb(0.9, 0.9, 0.2)),
        Transform::from_xyz(0.0, 2.5, -2.0),
    ));
    // Player HP bar
    let player_bar = commands.spawn((
        PlayerHealthBar,
        BattleUiMarker,
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.1, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            ..default()
        })),
        Transform::from_xyz(-1.5, 1.8, -2.0),
    )).id();

    let player_text = commands.spawn((
        BattleUiMarker,
        Text2d::new("Player HP: 100"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.15, 0.02),
    )).id();
    commands.entity(player_bar).add_child(player_text);

    // Enemy HP bar
    let enemy_bar = commands.spawn((
        EnemyHealthBar,
        BattleUiMarker,
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.1, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(1.5, 1.8, -2.0),
    )).id();

    let enemy_text = commands.spawn((
        BattleUiMarker,
        Text2d::new("Typo HP: 50"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.15, 0.02),
    )).id();
    commands.entity(enemy_bar).add_child(enemy_text);
}

#[cfg(feature = "xr")]
fn update_battle_hp_bars_xr(
    session: Option<Res<BattleSession>>,
    mut player_bar: Query<(&mut Transform, &Children), (With<PlayerHealthBar>, Without<EnemyHealthBar>)>,
    mut enemy_bar: Query<(&mut Transform, &Children), (With<EnemyHealthBar>, Without<PlayerHealthBar>)>,
    mut text_query: Query<&mut Text2d>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    for (mut transform, children) in &mut player_bar {
        let ratio = (session.player_health as f32 / 100.0).clamp(0.0, 1.0);
        transform.scale.x = ratio;
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("Player HP: {}", session.player_health);
            }
        }
    }

    for (mut transform, children) in &mut enemy_bar {
        let ratio = (session.typo_health as f32 / 50.0).clamp(0.0, 1.0);
        transform.scale.x = ratio;
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("Typo HP: {}", session.typo_health);
            }
        }
    }
}

#[cfg(feature = "xr")]
fn cleanup_battle_ui_xr(
    mut commands: Commands,
    query: Query<Entity, With<BattleUiMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "xr"))]
fn spawn_battle_ui_2d(
    mut commands: Commands,
    session: Res<BattleSession>,
) {
    let instruction_text = format!("WILD TYPO: {}", session.typo_word.to_uppercase());
    
    commands.spawn((
        BattleUiMarker,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(instruction_text),
            TextFont { font_size: 36.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.2)),
        ));

        // Container for HP Bars
        parent.spawn((
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Px(400.0),
                ..default()
            },
        )).with_children(|bars| {
            bars.spawn((
                PlayerHealthBar,
                Text::new("Player HP: 100"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.2, 0.8, 0.2)),
            ));

            bars.spawn((
                EnemyHealthBar,
                Text::new("Typo HP: 50"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.8, 0.2, 0.2)),
            ));
        });
    });
}

#[cfg(not(feature = "xr"))]
fn update_battle_hp_bars_2d(
    session: Option<Res<BattleSession>>,
    mut player_bar: Query<&mut Text, (With<PlayerHealthBar>, Without<EnemyHealthBar>)>,
    mut enemy_bar: Query<&mut Text, (With<EnemyHealthBar>, Without<PlayerHealthBar>)>,
) {
    let session = match session {
        Some(s) => s,
        None => return,
    };

    if let Some(mut text) = player_bar.iter_mut().next() {
        text.0 = format!("Player HP: {}", session.player_health);
    }
    
    if let Some(mut text) = enemy_bar.iter_mut().next() {
        text.0 = format!("Typo HP: {}", session.typo_health);
    }
}

#[cfg(not(feature = "xr"))]
fn cleanup_battle_ui_2d(
    mut commands: Commands,
    query: Query<Entity, With<BattleUiMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
