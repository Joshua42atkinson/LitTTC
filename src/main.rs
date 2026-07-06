// main.rs — Core Bevy entry point for Desktop target
mod components;
mod database;
mod deck;
mod input;
mod render;
mod quest;
mod battle;
mod letter;
mod hand_tracking;
mod save;
mod spatial_ui;
mod chat;
mod hud;
mod menu;
mod tutorial;
mod paywall;
mod time_cycle;

use std::collections::HashMap;
use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use bevy::asset::AssetEvent;
use components::*;
use database::*;
use letter::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Communication Class — Bevy VR EdTech".to_string(),
                    resolution: bevy::window::WindowResolution::new(1280, 720),
                    canvas: Some("#daydream-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
            render::RenderPlugin,
            chat::ChatPlugin,
            battle::BattlePlugin,
            quest::QuestPlugin,
            hud::HudPlugin,
            menu::MenuPlugin,
            tutorial::TutorialPlugin,
            paywall::PaywallPlugin,
            time_cycle::TimeCyclePlugin,
            spatial_ui::SpatialUiPlugin,
            database::DatabasePlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.3)))
        .init_state::<GameState>()
        .init_resource::<Deck>()
        .init_resource::<Hand>()
        .init_resource::<DiscardPile>()
        .init_resource::<input::DragState>()
        .init_resource::<input::PendingSwipe>()
        .init_resource::<LetterStash>()
        .init_resource::<CurrentSpelling>()
        .init_resource::<CharacterSheet>()
        .init_resource::<SpellBook>()
        .init_resource::<StudentTrail>()
        .init_resource::<CurrentSlide>()
        .init_resource::<hand_tracking::HandTrackingState>()
        .init_resource::<FrameDiagnostics>()
        .init_resource::<quest::CurriculumManager>()
        .init_resource::<hand_tracking::PinchEvents>()
        .init_resource::<crate::components::TimeScale>()
        .add_systems(Startup, setup_world)
        .add_systems(OnEnter(GameState::Loading), start_loading_database)
        .add_systems(Update, check_database_loading.run_if(in_state(GameState::Loading)))
        .add_systems(Update, hot_reload_database)
        .add_systems(OnEnter(GameState::Collecting), initialize_player_deck)
        .add_systems(Update, (
            spawn_letter_crystals,
            animate_crystals,
            collect_letters,
        ).run_if(in_state(GameState::Collecting)))
        .add_systems(OnEnter(GameState::Playing), save::auto_save_system)
        .add_systems(Update, (
            deck::select_card_by_key,
        ).run_if(in_state(GameState::Playing)));

    #[cfg(feature = "xr")]
    {
        app.add_systems(OnEnter(GameState::Reviewing), (spawn_review_ui_xr, save::auto_save_system))
           .add_systems(OnExit(GameState::Reviewing), cleanup_review_ui_xr);
    }

    #[cfg(not(feature = "xr"))]
    {
        app.add_systems(OnEnter(GameState::Reviewing), (spawn_review_ui_2d, save::auto_save_system))
           .add_systems(OnExit(GameState::Reviewing), cleanup_review_ui_2d);
    }

    app.add_systems(Update, review_input_system.run_if(in_state(GameState::Reviewing)))
       .add_systems(Update, (
            hand_tracking::update_hand_tracking,
            deck::draw_cards,
            update_frame_diagnostics,
            input::drag_start,
            input::drag_move,
            input::drag_end,
            input::keyboard_input,
            input::handle_ui_button_interactions,
            hand_tracking::grammar_fusion_system,
        ));

    #[cfg(feature = "xr")]
    {
        app.add_systems(OnEnter(GameState::Constructing), spawn_holographic_stash)
           .add_systems(Update, handle_vr_spelling.run_if(in_state(GameState::Constructing)))
           .add_systems(OnExit(GameState::Constructing), cleanup_holographic_stash)
           .add_systems(OnEnter(GameState::Questing), spawn_vr_hand)
           .add_systems(Update, vr_quest_interaction.run_if(in_state(GameState::Questing)))
           .add_systems(OnExit(GameState::Questing), cleanup_vr_hand)
           .add_systems(OnEnter(GameState::Battling), spawn_vr_hand)
           .add_systems(Update, vr_battle_interaction.run_if(in_state(GameState::Battling)))
           .add_systems(OnExit(GameState::Battling), cleanup_vr_hand);
    }

    #[cfg(not(feature = "xr"))]
    {
        app.add_systems(Update, handle_keyboard_spelling.run_if(in_state(GameState::Constructing)))
           .add_systems(Update, keyboard_quest_interaction.run_if(in_state(GameState::Questing)))
           .add_systems(Update, keyboard_battle_interaction.run_if(in_state(GameState::Battling)));
    }

    app.run();
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn camera with HDR, Bloom, and Screen-Space Ambient Occlusion (SSAO)
    commands.spawn((
        Camera3d::default(),
        bevy::render::view::Hdr,
        bevy::post_process::bloom::Bloom::NATURAL,
        bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(10.0, 10.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.22, 0.25),
            perceptual_roughness: 0.9,
            ..default()
        })),
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        crate::render::SkyLight,
    ));

    info!("3D Environment initialized successfully!");
}

#[derive(Resource)]
struct LoadingDatabases {
    words: Handle<RawJsonAsset>,
    syns: Handle<RawJsonAsset>,
    etym: Handle<RawJsonAsset>,
    quest: Handle<RawJsonAsset>,
    npcs: Handle<RawJsonAsset>,
}

fn start_loading_database(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("Starting database asset loading...");
    commands.insert_resource(LoadingDatabases {
        words: asset_server.load("word_database.json"),
        syns: asset_server.load("synonym_database.json"),
        etym: asset_server.load("etymology_db.json"),
        quest: asset_server.load("quest_data.json"),
        npcs: asset_server.load("lore_db.json"),
    });
}

fn check_database_loading(
    mut commands: Commands,
    loading: Res<LoadingDatabases>,
    assets: Res<Assets<RawJsonAsset>>,
    mut next_state: ResMut<NextState<GameState>>,
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
        let etymology: EtymologyDB = serde_json::from_str(&e.text).expect("Failed to parse etymology");
        let quests: QuestData = serde_json::from_str(&q.text).expect("Failed to parse quests");
        let npcs: HashMap<String, NpcData> = serde_json::from_str(&n.text).expect("Failed to parse npcs");

        commands.insert_resource(GameDatabase {
            words,
            synonyms,
            etymology,
            quests,
            npcs,
        });

        info!("Database parsed successfully. Transitioning to MainMenu.");
        next_state.set(GameState::MainMenu);
    }
}

fn hot_reload_database(
    mut events: MessageReader<AssetEvent<RawJsonAsset>>,
    loading: Option<Res<LoadingDatabases>>,
    assets: Res<Assets<RawJsonAsset>>,
    mut db: Option<ResMut<GameDatabase>>,
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

fn initialize_player_deck(
    db: Res<GameDatabase>,
    mut deck: ResMut<Deck>,
    mut spellbook: ResMut<SpellBook>,
    mut stash: ResMut<LetterStash>,
    mut sheet: ResMut<CharacterSheet>,
    mut trail: ResMut<StudentTrail>,
) {
    info!("Initializing player deck from loaded curriculum database...");
    
    if let Ok(data) = save::load_game() {
        *sheet = data.character_sheet;
        *spellbook = data.spellbook;
        *trail = data.student_trail;
        for entry in &spellbook.entries {
            deck.cards.push(entry.word.clone());
        }
        info!("Loaded save file!");
    } else {
        let mut pool: Vec<String> = db.synonyms.keys().cloned().collect();
        pool.sort();

        if !pool.is_empty() {
            for word in pool.iter().take(15) {
                deck.cards.push(word.clone());
                spellbook.record_encounter(word, Channel::Mind);
            }
        } else {
            let default_words = ["abandoned", "abc", "ability", "patience", "clarity", "courage", "wisdom", "strength"];
            for &word in &default_words {
                deck.cards.push(word.to_string());
                spellbook.record_encounter(word, Channel::Mind);
            }
        }
    }

    stash.letters.extend("PATIENCECLARITYCOURAGEWISDOMSTRENGTH".chars());
}

// Keyboard triggers removed in favor of VR Pinch events

#[derive(Component)]
pub struct VrHandCard(pub usize);

#[derive(Component)]
pub struct VrSubmitButton;

fn spawn_vr_hand(
    mut commands: Commands,
    hand: Res<Hand>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<State<GameState>>,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.5),
        ..default()
    });

    let count = hand.cards.len();
    for (i, word) in hand.cards.iter().enumerate() {
        let spacing = 0.6;
        let start_x = -((count as f32 - 1.0) * spacing) / 2.0;
        let x = start_x + (i as f32 * spacing);
        let pos = Vec3::new(x, 1.0, -1.0);

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.4, 0.6, 0.02))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(pos),
            VrHandCard(i),
        )).with_children(|inner| {
            inner.spawn((
                Text2d::new(word.clone()),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::BLACK),
                Transform::from_xyz(0.0, 0.0, 0.02),
            ));
        });
    }

    if *state.get() == GameState::Questing {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 0.3, 0.02))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.8, 0.3),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, -1.0),
            VrSubmitButton,
        )).with_children(|inner| {
            inner.spawn((
                Text2d::new("Complete Quest"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.02),
            ));
        });
    }
}

fn cleanup_vr_hand(
    mut commands: Commands,
    query: Query<Entity, Or<(With<VrHandCard>, With<VrSubmitButton>)>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn vr_quest_interaction(
    pinch_events: Res<hand_tracking::PinchEvents>,
    hand: Res<Hand>,
    session: Option<ResMut<quest::QuestSession>>,
    mut sheet: ResMut<CharacterSheet>,
    mut spellbook: ResMut<SpellBook>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    card_query: Query<(&GlobalTransform, &VrHandCard)>,
    submit_query: Query<&GlobalTransform, With<VrSubmitButton>>,
) {
    let mut session = match session {
        Some(s) => s,
        None => return,
    };

    for event in &pinch_events.events {
        // Check submit button
        for transform in &submit_query {
            if event.position.distance(transform.translation()) < 0.4 {
                if session.filled_slots.len() >= session.slots.len() {
                    quest::complete_quest(&session, &mut sheet, &mut spellbook, &mut next_state, &mut commands);
                } else {
                    warn!("Cannot complete quest yet, fill all slots!");
                }
                return;
            }
        }

        // Check hand cards
        for (transform, card) in &card_query {
            if event.position.distance(transform.translation()) < 0.3 {
                if card.0 < hand.cards.len() {
                    let word = &hand.cards[card.0];
                    let slots_count = session.slots.len();
                    for i in 0..slots_count {
                        if !session.filled_slots.contains_key(&i) {
                            quest::fill_slot(i, word, None, &mut session);
                            break;
                        }
                    }
                }
                return;
            }
        }
    }
}

fn vr_battle_interaction(
    pinch_events: Res<hand_tracking::PinchEvents>,
    mut hand: ResMut<Hand>,
    session: Option<ResMut<battle::BattleSession>>,
    db: Res<GameDatabase>,
    mut spellbook: ResMut<SpellBook>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera>>,
    card_query: Query<(&GlobalTransform, &VrHandCard)>,
) {
    let mut session = match session {
        Some(s) => s,
        None => return,
    };

    for event in &pinch_events.events {
        for (transform, card) in &card_query {
            if event.position.distance(transform.translation()) < 0.3 {
                if card.0 < hand.cards.len() {
                    let played_word = hand.cards.remove(card.0);
                    let is_correct = battle::play_battle_card(&played_word, &mut session, &db, &mut spellbook, &mut next_state);
                    
                    if is_correct {
                        for entity in camera_query.iter() {
                            commands.entity(entity).insert(crate::render::ScreenShake { timer: 0.3, intensity: 0.2 });
                        }
                    }
                }
                return;
            }
        }
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct FrameDiagnostics {
    pub fps: f32,
    pub frame_count: u32,
}

fn update_frame_diagnostics(
    time: Res<Time>,
    mut diagnostics: ResMut<FrameDiagnostics>,
) {
    diagnostics.frame_count += 1;
    let delta = time.delta_secs();
    if delta > 0.0 {
        diagnostics.fps = 1.0 / delta;
    }
    if diagnostics.frame_count % 120 == 0 {
        info!("FPS Diagnostic Overlay: {:.1} fps", diagnostics.fps);
    }
}

#[derive(Component)]
pub struct ReviewUiPanel;

#[derive(Component)]
pub struct ReviewUiText;

#[cfg(feature = "xr")]
fn spawn_review_ui_xr(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spellbook: Res<SpellBook>,
) {
    let panel_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.02, 0.08, 0.05, 0.95),
        emissive: Color::srgba(0.02, 0.15, 0.05, 1.0).to_srgba().into(),
        metallic: 0.1,
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let panel = commands.spawn((
        ReviewUiPanel,
        Mesh3d(meshes.add(Cuboid::new(3.0, 1.5, 0.05))),
        MeshMaterial3d(panel_mat),
        Transform::from_xyz(0.0, 2.0, -1.8),
    )).id();

    let mut mastered_text = "Mastered Words:\n".to_string();
    let mut count = 0;
    for entry in spellbook.entries.iter() {
        if count < 5 {
            mastered_text.push_str(&format!("- {}: {:?}\n", entry.word, entry.mastery));
            count += 1;
        }
    }
    if count == 0 {
        mastered_text.push_str("(No words registered in SpellBook yet)\n");
    }

    let text_entity = commands.spawn((
        ReviewUiText,
        Text2d::new(format!("VICTORY & REVIEW\n\n{}\nPress ENTER to continue exploration", mastered_text)),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.0, 0.03),
    )).id();

    commands.entity(panel).add_child(text_entity);
}

fn review_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

#[cfg(feature = "xr")]
fn cleanup_review_ui_xr(
    mut commands: Commands,
    query: Query<Entity, Or<(With<ReviewUiPanel>, With<ReviewUiText>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "xr"))]
fn spawn_review_ui_2d(
    mut commands: Commands,
    spellbook: Res<SpellBook>,
) {
    let mut mastered_text = "Mastered Words:\n".to_string();
    let mut count = 0;
    for entry in spellbook.entries.iter() {
        if count < 5 {
            mastered_text.push_str(&format!("- {}: {:?}\n", entry.word, entry.mastery));
            count += 1;
        }
    }
    if count == 0 {
        mastered_text.push_str("(No words registered in SpellBook yet)\n");
    }

    commands.spawn((
        ReviewUiPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(20.0),
            left: Val::Percent(30.0),
            width: Val::Percent(40.0),
            height: Val::Percent(60.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(30.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.15, 0.1, 0.95)),
    )).with_children(|parent| {
        parent.spawn((
            ReviewUiText,
            Text::new(format!("VICTORY & REVIEW\n\n{}\n\n[Press ENTER to continue]", mastered_text)),
            TextFont { font_size: 28.0, ..default() },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
        ));
    });
}

#[cfg(not(feature = "xr"))]
fn cleanup_review_ui_2d(
    mut commands: Commands,
    query: Query<Entity, With<ReviewUiPanel>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "xr"))]
fn keyboard_quest_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    hand: Res<Hand>,
    session: Option<ResMut<quest::QuestSession>>,
    mut sheet: ResMut<CharacterSheet>,
    mut spellbook: ResMut<SpellBook>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let mut session = match session {
        Some(s) => s,
        None => return,
    };

    if keys.just_pressed(KeyCode::Enter) {
        if session.filled_slots.len() >= session.slots.len() {
            quest::complete_quest(&session, &mut sheet, &mut spellbook, &mut next_state, &mut commands);
        } else {
            info!("Cannot complete quest yet, fill all slots!");
        }
        return;
    }

    let pressed_idx = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ].iter().position(|&k| keys.just_pressed(k));

    if let Some(idx) = pressed_idx {
        if idx < hand.cards.len() {
            let word = &hand.cards[idx];
            let slots_count = session.slots.len();
            for i in 0..slots_count {
                if !session.filled_slots.contains_key(&i) {
                    quest::fill_slot(i, word, None, &mut session);
                    break;
                }
            }
        }
    }
}

#[cfg(not(feature = "xr"))]
fn keyboard_battle_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    mut hand: ResMut<Hand>,
    session: Option<ResMut<battle::BattleSession>>,
    db: Res<GameDatabase>,
    mut spellbook: ResMut<SpellBook>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera>>,
) {
    let mut session = match session {
        Some(s) => s,
        None => return,
    };

    let pressed_idx = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ].iter().position(|&k| keys.just_pressed(k));

    if let Some(idx) = pressed_idx {
        if idx < hand.cards.len() {
            let played_word = hand.cards.remove(idx);
            let is_correct = battle::play_battle_card(&played_word, &mut session, &db, &mut spellbook, &mut next_state);
            
            if is_correct {
                for entity in camera_query.iter() {
                    commands.entity(entity).insert(crate::render::ScreenShake { timer: 0.3, intensity: 0.2 });
                }
            }
        }
    }
}
