// lib.rs — Android NDK entry point and shared game structures
pub mod components;
pub mod database;
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
pub mod time_cycle;



#[cfg(target_os = "android")]
#[bevy_main]
fn main() {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    info!("Initializing Communication Class Android XR Entry Point...");

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
            .add_plugins((render::RenderPlugin, chat::ChatPlugin, battle::BattlePlugin, quest::QuestPlugin))
            .run();
    }

    #[cfg(not(feature = "xr"))]
    {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins((render::RenderPlugin, chat::ChatPlugin, battle::BattlePlugin, quest::QuestPlugin))
            .run();
    }
}
