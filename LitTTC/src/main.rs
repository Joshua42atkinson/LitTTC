// main.rs — Core Bevy entry point for Desktop target
pub mod core {
    pub mod components;
    pub mod database;
    pub mod asset_catalog;
    pub mod generated_assets;
    pub mod deck;
    pub mod input;
    pub mod render;
    pub mod quest;
    pub mod battle;
    pub mod letter;
    pub mod hand_tracking;
    pub mod save;
    pub mod spatial_ui;
    pub mod chat;
    pub mod hud;
    pub mod menu;
    pub mod tutorial;
    pub mod paywall;
    pub mod settings;
    pub mod difficulty;
    pub mod pet_collection;
    pub mod pet_reveal;
    pub mod music;
    pub mod time_cycle;
    pub mod spatial_deck;
    pub mod altar;
    pub mod dialogue_ui;
    pub mod blocklist;
    pub mod commands;
    pub mod companion;
    pub mod diagnostics;
    pub mod platform_paths;
    pub mod performance;
    pub mod ar_capture;
    pub mod overworld;
}

pub mod bridge {
    pub mod tts_client;
    pub mod url_opener;
}

pub use core::*;

use bevy::prelude::*;
#[cfg(feature = "flat2d")]
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use components::*;
use database::*;
use letter::*;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "flat2d")]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "LitTCG 2D — 8-bit prototype".to_string(),
            resolution: bevy::window::WindowResolution::new(1280, 720),
            mode: bevy::window::WindowMode::Windowed,
            ..default()
        }),
        ..default()
    };

    #[cfg(not(feature = "flat2d"))]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "LitTCG — Literary Trading Card Game".to_string(),
            resolution: bevy::window::WindowResolution::new(1280, 720),
            mode: bevy::window::WindowMode::Windowed,
            ..default()
        }),
        ..default()
    };

    #[cfg(feature = "flat2d")]
    let default_plugins = DefaultPlugins
        .set(window_plugin)
        .set(bevy::image::ImagePlugin::default_nearest())
        .set(bevy::asset::AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        });

    #[cfg(not(feature = "flat2d"))]
    let default_plugins = DefaultPlugins
        .set(window_plugin)
        .set(bevy::asset::AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        });

    app.add_plugins((
            default_plugins,
            render::RenderPlugin,
            chat::ChatPlugin,
            battle::BattlePlugin,
            quest::QuestPlugin,
            hud::HudPlugin,
            menu::MenuPlugin,
            tutorial::TutorialPlugin,
            paywall::PaywallPlugin,
            time_cycle::TimeCyclePlugin,
        ))
        .add_plugins((
            database::DatabasePlugin,
            altar::AltarPlugin,
            ar_capture::ARCapturePlugin,
            companion::CompanionPlugin,
            overworld::OverworldPlugin,
        ));

    #[cfg(not(feature = "flat2d"))]
    app.add_plugins((
        spatial_ui::SpatialUiPlugin,
        spatial_deck::SpatialDeckPlugin,
        dialogue_ui::DialogueUiPlugin,
    ));

    // Music is disabled in the flat2d prototype until chiptune stems replace the current drone.
    #[cfg(not(feature = "flat2d"))]
    app.add_plugins(music::MusicPlugin);

    app.add_plugins((
            performance::PerformancePlugin,
            settings::SettingsPlugin,
            difficulty::DifficultyPlugin,
            pet_collection::PetCollectionPlugin,
            pet_reveal::PetRevealPlugin,
        ))
        .insert_resource(if cfg!(feature = "flat2d") {
            ClearColor(Color::srgb(0.04, 0.03, 0.09))
        } else {
            ClearColor(Color::srgb(0.2, 0.2, 0.3))
        })
        .init_state::<GameState>()
        .add_message::<commands::GameCommand>()
        .init_resource::<commands::LastCommand>()
        .init_resource::<GameDatabase>()
        .init_resource::<Deck>()
        .init_resource::<Hand>()
        .init_resource::<DiscardPile>()
        .init_resource::<input::DragState>()
        .init_resource::<LetterStash>()
        .init_resource::<CurrentSpelling>()
        .init_resource::<CharacterSheet>()
        .init_resource::<SpellBook>()
        .init_resource::<WordTrail>()
        .init_resource::<CurrentSlide>()
        .init_resource::<hand_tracking::HandTrackingState>()
        .init_resource::<diagnostics::FrameDiagnostics>()
        .init_resource::<quest::GradeManager>()
        .init_resource::<hand_tracking::PinchEvents>()
        .init_resource::<crate::components::TimeScale>()
        .init_resource::<ActiveGestures>()
        .insert_resource(crate::settings::GameSettings::load().unwrap_or_default());

    #[cfg(not(feature = "flat2d"))]
    app.add_systems(Startup, render::setup_world);

    // GeneratedAssets is used by HUD/pet info in both 2D and 3D.
    app.add_systems(Startup, generated_assets::load_generated_assets);

    app.add_systems(OnEnter(GameState::Loading), (database::spawn_loading_ui, database::start_loading_database))
        .add_systems(Update, (
            database::update_loading_progress,
            database::check_database_loading,
        ).run_if(in_state(GameState::Loading)))
        .add_systems(OnExit(GameState::Loading), database::cleanup_loading_ui)
        .add_systems(Update, database::hot_reload_database);

    #[cfg(not(feature = "flat2d"))]
    {
        app.add_systems(OnEnter(GameState::Collecting), deck::initialize_player_deck)
           .add_systems(Update, (
                spawn_letter_crystals,
                animate_crystals,
                collect_letters,
                handle_pinch_crystals,
            ).run_if(in_state(GameState::Collecting)));
    }

    #[cfg(feature = "flat2d")]
    {
        app.add_systems(Startup, setup_autoplay)
           .add_systems(Update, hud::toggle_hud_visibility)
           .add_systems(OnEnter(GameState::Exploring), deck::initialize_player_deck)
           .add_systems(OnEnter(GameState::Collecting), (
                deck::initialize_player_deck,
                hud::fill_letter_stash_and_start_constructing,
           ))
           .add_systems(OnEnter(GameState::Constructing), hud::spawn_constructing_ui)
           .add_systems(OnExit(GameState::Constructing), hud::despawn_constructing_ui)
           .add_systems(Update, autoplay_system.before(hud::update_constructing_ui).before(commands::handle_game_commands))
           .add_systems(Update, (
                hud::update_constructing_ui,
                hud::handle_constructing_buttons,
           ).run_if(in_state(GameState::Constructing)).before(commands::handle_game_commands));
    }

    app.add_systems(OnEnter(GameState::Playing), save::auto_save_system);

    app.add_systems(Update, (
        deck::select_card_by_key,
    ).run_if(in_state(GameState::Playing)));

    #[cfg(feature = "xr")]
    {
        app.add_systems(OnEnter(GameState::Reviewing), (hud::spawn_review_ui_xr, save::auto_save_system))
           .add_systems(OnExit(GameState::Reviewing), hud::cleanup_review_ui_xr);
    }

    #[cfg(not(feature = "xr"))]
    {
        app.add_systems(OnEnter(GameState::Reviewing), (hud::spawn_review_ui_2d, save::auto_save_system))
           .add_systems(OnExit(GameState::Reviewing), hud::cleanup_review_ui_2d);
    }

    app.add_systems(Update, hud::review_input_system.run_if(in_state(GameState::Reviewing)))
       .add_systems(Update, commands::handle_game_commands)
       .add_systems(Update, (
            hand_tracking::update_hand_tracking,
            deck::draw_cards,
            diagnostics::update_frame_diagnostics,
            input::drag_start,
            input::drag_move,
            input::drag_end,
            input::keyboard_input,
            input::handle_touch_input,
            input::handle_hand_card_button_interactions,
            input::handle_play_card_button_interactions,
            input::handle_skip_button_interactions,
            input::handle_quest_action_button_interactions,
            input::handle_battle_action_button_interactions,
            hand_tracking::grammar_fusion_system,
            letter::handle_keyboard_spelling,
        ).before(commands::handle_game_commands));

    #[cfg(feature = "xr")]
    {
        app.add_systems(OnEnter(GameState::Constructing), letter::spawn_holographic_stash)
           .add_systems(Update, letter::handle_vr_spelling.run_if(in_state(GameState::Constructing)).before(commands::handle_game_commands))
           .add_systems(OnExit(GameState::Constructing), letter::cleanup_holographic_stash)
           .add_systems(OnEnter(GameState::Questing), hand_tracking::spawn_vr_hand)
           .add_systems(Update, hand_tracking::vr_quest_interaction.run_if(in_state(GameState::Questing)).before(commands::handle_game_commands))
           .add_systems(OnExit(GameState::Questing), hand_tracking::cleanup_vr_hand)
           .add_systems(OnEnter(GameState::Battling), hand_tracking::spawn_vr_hand)
           .add_systems(Update, hand_tracking::vr_battle_interaction.run_if(in_state(GameState::Battling)).before(commands::handle_game_commands))
           .add_systems(OnExit(GameState::Battling), hand_tracking::cleanup_vr_hand);
    }

    #[cfg(not(feature = "xr"))]
    {
        app.add_systems(Update, input::keyboard_quest_interaction.run_if(in_state(GameState::Questing)).before(commands::handle_game_commands))
           .add_systems(Update, input::keyboard_battle_interaction.run_if(in_state(GameState::Battling)).before(commands::handle_game_commands))
           .add_systems(Update, input::keyboard_debug_shortcuts.before(commands::handle_game_commands));
    }

    app.run();
}

#[cfg(feature = "flat2d")]
#[derive(Resource)]
struct Autoplay {
    enabled: bool,
    paywall_test: bool,
    continue_test: bool,
    last_state: Option<GameState>,
    timer: Timer,
    capture: bool,
    exiting: bool,
    step: usize,
}

#[cfg(feature = "flat2d")]
fn setup_autoplay(mut commands: Commands) {
    let enabled = std::env::var("LITTCG_AUTOPLAY").is_ok();
    let paywall_test = std::env::var("LITTCG_AUTOPLAY_PAYWALL").is_ok();
    let continue_test = std::env::var("LITTCG_AUTOPLAY_CONTINUE").is_ok();
    commands.insert_resource(Autoplay {
        enabled,
        paywall_test,
        continue_test,
        last_state: None,
        timer: Timer::from_seconds(1.0, TimerMode::Once),
        capture: false,
        exiting: false,
        step: 0,
    });
}

#[cfg(feature = "flat2d")]
fn autoplay_system(
    mut commands: Commands,
    mut autoplay: ResMut<Autoplay>,
    state: Res<State<GameState>>,
    mut stash: ResMut<crate::letter::LetterStash>,
    mut demo: ResMut<crate::paywall::DemoSettings>,
    time: Res<Time>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
    mut exit_writer: MessageWriter<AppExit>,
    quest_session: Option<Res<crate::quest::QuestSession>>,
    battle_session: Option<Res<crate::battle::BattleSession>>,
) {
    if !autoplay.enabled {
        return;
    }
    let current = state.get().clone();

    if autoplay.last_state != Some(current.clone()) {
        autoplay.last_state = Some(current.clone());
        autoplay.timer.set_duration(std::time::Duration::from_secs(1));
        autoplay.timer.reset();
        autoplay.capture = true;
        autoplay.exiting = false;
    }

    autoplay.timer.tick(time.delta());
    if !autoplay.timer.just_finished() {
        return;
    }

    if autoplay.capture {
        autoplay.capture = false;
        let path = format!("/home/joshua/LitTCG/flat2d_autoplay_{:?}.png", current);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }

    match current {
        GameState::MainMenu => {
            if autoplay.continue_test && autoplay.step == 0 {
                writer.write(crate::commands::GameCommand::ContinueGame);
                autoplay.step = 4;
            } else if autoplay.paywall_test && autoplay.step >= 100 {
                exit_writer.write(AppExit::Success);
            } else {
                match autoplay.step {
                    0 => {
                        writer.write(crate::commands::GameCommand::OpenSettings);
                        autoplay.step = 1;
                    }
                    1 => {
                        writer.write(crate::commands::GameCommand::OpenDifficulty);
                        autoplay.step = 2;
                    }
                    2 => {
                        writer.write(crate::commands::GameCommand::OpenPetCollection);
                        autoplay.step = 3;
                    }
                    _ => {
                        writer.write(crate::commands::GameCommand::NewGame);
                        autoplay.step = 4;
                    }
                }
            }
            autoplay.timer.set_duration(std::time::Duration::from_secs(1));
            autoplay.timer.reset();
        }
        GameState::Settings => {
            writer.write(crate::commands::GameCommand::ReturnToMenu);
            autoplay.timer.set_duration(std::time::Duration::from_secs(1));
            autoplay.timer.reset();
        }
        GameState::Difficulty => {
            writer.write(crate::commands::GameCommand::ReturnToMenu);
            autoplay.timer.set_duration(std::time::Duration::from_secs(1));
            autoplay.timer.reset();
        }
        GameState::PetCollection => {
            writer.write(crate::commands::GameCommand::ReturnToMenu);
            autoplay.timer.set_duration(std::time::Duration::from_secs(1));
            autoplay.timer.reset();
        }
        GameState::Constructing => {
            let demo_word = "ACT";
            stash.letters = demo_word.chars().collect();
            for c in demo_word.chars() {
                writer.write(crate::commands::GameCommand::AddLetter(c));
            }
            if autoplay.paywall_test {
                demo.words_used = 10;
                autoplay.step = 100;
            }
            writer.write(crate::commands::GameCommand::SubmitSpelling);
        }
        GameState::Paywall => {
            if autoplay.exiting {
                exit_writer.write(AppExit::Success);
            } else {
                autoplay.exiting = true;
                autoplay.timer.set_duration(std::time::Duration::from_secs(1));
                autoplay.timer.reset();
            }
        }
        GameState::Questing => {
            if let Some(session) = quest_session.as_ref() {
                if session.filled_slots.len() >= session.slots.len() {
                    writer.write(crate::commands::GameCommand::PlayCard);
                } else {
                    writer.write(crate::commands::GameCommand::SelectCard(0));
                    writer.write(crate::commands::GameCommand::PlayCard);
                }
                autoplay.timer.set_duration(std::time::Duration::from_secs(1));
                autoplay.timer.reset();
            }
        }
        GameState::Battling => {
            if battle_session.is_some() {
                writer.write(crate::commands::GameCommand::SelectCard(0));
                writer.write(crate::commands::GameCommand::PlayCard);
                autoplay.timer.set_duration(std::time::Duration::from_secs(1));
                autoplay.timer.reset();
            }
        }
        GameState::Reviewing => {
            writer.write(crate::commands::GameCommand::DismissReview);
        }
        GameState::Exploring => {
            match autoplay.step {
                4 => {
                    writer.write(crate::commands::GameCommand::StartQuest("Barnaby".to_string()));
                    autoplay.step = 5;
                    autoplay.timer.set_duration(std::time::Duration::from_secs(1));
                    autoplay.timer.reset();
                }
                5 => {
                    writer.write(crate::commands::GameCommand::StartBattle);
                    autoplay.step = 6;
                    autoplay.timer.set_duration(std::time::Duration::from_secs(1));
                    autoplay.timer.reset();
                }
                _ => {
                    if autoplay.exiting {
                        exit_writer.write(AppExit::Success);
                    } else {
                        autoplay.exiting = true;
                        autoplay.timer.set_duration(std::time::Duration::from_secs(2));
                        autoplay.timer.reset();
                    }
                }
            }
        }
        _ => {}
    }
}
