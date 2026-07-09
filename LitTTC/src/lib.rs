// lib.rs — Android NDK entry point
#![warn(clippy::all)]

pub mod core {
    pub mod asset_catalog;
    pub mod components;
    pub mod database;
    pub mod deck;
    pub mod generated_assets;
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
    pub mod time_cycle;
    pub mod spatial_deck;
    pub mod altar;
    pub mod dialogue_ui;
    pub mod diagnostics;
    pub mod blocklist;
    pub mod commands;
    pub mod music;
    pub mod companion;
    pub mod platform_paths;
    pub mod performance;
    pub mod ar_capture;
}

pub mod bridge {
    pub mod tts_client;
    pub mod url_opener;
}

pub use core::*;



#[cfg(target_os = "android")]
use bevy::prelude::*;
#[cfg(target_os = "android")]
use components::*;
#[cfg(target_os = "android")]
use database::*;
#[cfg(target_os = "android")]
use letter::*;

#[cfg(target_os = "android")]
fn init_common_resources(app: &mut App) {
    app.add_message::<commands::GameCommand>()
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
        .init_resource::<ActiveGestures>();
}

#[cfg(target_os = "android")]
struct CommonResourcesPlugin;

#[cfg(target_os = "android")]
impl Plugin for CommonResourcesPlugin {
    fn build(&self, app: &mut App) {
        init_common_resources(app);
    }
}

#[cfg(target_os = "android")]
#[bevy_main]
fn main() {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    info!("Initializing LitTCG Android Entry Point...");

    #[cfg(feature = "xr")]
    {
        use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
        use bevy_mod_openxr::{add_xr_plugins, resources::OxrSessionConfig, types::EnvironmentBlendMode};

        App::new()
            .add_plugins(add_xr_plugins(
                DefaultPlugins.build().disable::<PipelinedRenderingPlugin>(),
            ))
            .insert_resource(OxrSessionConfig {
                blend_mode_preference: vec![
                    EnvironmentBlendMode::ALPHA_BLEND,
                    EnvironmentBlendMode::OPAQUE,
                ],
                ..default()
            })
            .insert_resource(ClearColor(Color::NONE))
            .insert_resource(crate::settings::GameSettings::load().unwrap_or_default())
            .init_state::<GameState>()
            .add_plugins(performance::PerformancePlugin)
            .add_plugins(CommonResourcesPlugin)
            .add_plugins((
                render::RenderPlugin, chat::ChatPlugin, battle::BattlePlugin, quest::QuestPlugin,
                hud::HudPlugin, menu::MenuPlugin, tutorial::TutorialPlugin, paywall::PaywallPlugin,
                time_cycle::TimeCyclePlugin, spatial_ui::SpatialUiPlugin, database::DatabasePlugin,
                spatial_deck::SpatialDeckPlugin, altar::AltarPlugin, dialogue_ui::DialogueUiPlugin,
                pet_reveal::PetRevealPlugin, ar_capture::ARCapturePlugin,
                companion::CompanionPlugin,
                music::MusicPlugin,
            ))
            .add_systems(Startup, generated_assets::load_generated_assets)
            .add_systems(OnEnter(GameState::Loading), (database::spawn_loading_ui, database::start_loading_database))
            .add_systems(Update, (
                database::update_loading_progress,
                database::check_database_loading,
            ).run_if(in_state(GameState::Loading)))
            .add_systems(OnExit(GameState::Loading), database::cleanup_loading_ui)
            .run();
    }

    #[cfg(not(feature = "xr"))]
    {
        App::new()
            .add_plugins(DefaultPlugins)
            .insert_resource(crate::settings::GameSettings::load().unwrap_or_default())
            .init_state::<GameState>()
            .add_plugins(performance::PerformancePlugin)
            .add_plugins(CommonResourcesPlugin)
            .add_plugins((
                render::RenderPlugin, chat::ChatPlugin, battle::BattlePlugin, quest::QuestPlugin,
                hud::HudPlugin, menu::MenuPlugin, tutorial::TutorialPlugin, paywall::PaywallPlugin,
                time_cycle::TimeCyclePlugin, spatial_ui::SpatialUiPlugin, database::DatabasePlugin,
                spatial_deck::SpatialDeckPlugin, altar::AltarPlugin, dialogue_ui::DialogueUiPlugin,
                pet_reveal::PetRevealPlugin,
                companion::CompanionPlugin,
                music::MusicPlugin,
            ))
            .add_systems(Startup, generated_assets::load_generated_assets)
            .add_systems(OnEnter(GameState::Loading), (database::spawn_loading_ui, database::start_loading_database))
            .add_systems(Update, (
                database::update_loading_progress,
                database::check_database_loading,
            ).run_if(in_state(GameState::Loading)))
            .add_systems(OnExit(GameState::Loading), database::cleanup_loading_ui)
            .run();
    }
}
