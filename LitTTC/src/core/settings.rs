// settings.rs — Settings screen and persistent preferences
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use bevy::prelude::*;
use crate::components::GameState;
use crate::platform_paths::data_dir;

pub struct SettingsPlugin;

#[derive(Component)]
pub struct SettingsUiRoot;

/// Persistent game preferences.
#[derive(Resource, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GameSettings {
    pub sound_volume: f32,
    pub music_volume: f32,
    pub tts_enabled: bool,
    pub hints_enabled: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sound_volume: 1.0,
            music_volume: 0.05,
            tts_enabled: true,
            hints_enabled: true,
        }
    }
}

impl GameSettings {
    pub const SAVE_PATH: &str = "settings.json";

    pub fn save_path() -> PathBuf {
        data_dir().join(Self::SAVE_PATH)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string_pretty(self)?;
        let path = Self::save_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn load() -> Result<Self, std::io::Error> {
        let mut file = File::open(Self::save_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let settings: GameSettings = serde_json::from_str(&contents)?;
        Ok(settings)
    }
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Settings), spawn_settings_ui)
           .add_systems(Update, settings_interaction.run_if(in_state(GameState::Settings)))
           .add_systems(OnExit(GameState::Settings), cleanup_settings_ui);
    }
}

fn spawn_settings_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
        SettingsUiRoot,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("SETTINGS"),
            TextFont { font_size: 50.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
        ));
        parent.spawn((
            Text::new("Audio, graphics, and accessibility options will live here."),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() },
        ));
        // Toggle rows
        let settings = [
            ("TTS", ToggleButton::Tts),
            ("Hints", ToggleButton::Hints),
        ];
        for (label, toggle) in settings {
            parent.spawn((
                Button,
                toggle,
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            )).with_children(|p| {
                p.spawn((
                    Text::new(format!("{}: ON", label)),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        }

        parent.spawn((
            Button,
            ResetSaveButton,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
        )).with_children(|p| {
            p.spawn((
                Text::new("Reset Save"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        parent.spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        )).with_children(|p| {
            p.spawn((
                Text::new("Main Menu"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

#[derive(Component, Clone, Copy, Debug)]
enum ToggleButton {
    Tts,
    Hints,
}

#[derive(Component)]
struct ResetSaveButton;

#[allow(clippy::type_complexity)]
fn settings_interaction(
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
    mut toggle_buttons: Query<(&Interaction, &ToggleButton, &mut BackgroundColor, &Children), Changed<Interaction>>,
    mut text_query: Query<&mut Text>,
    reset_buttons: Query<&Interaction, (Changed<Interaction>, With<ResetSaveButton>, Without<ToggleButton>)>,
    menu_buttons: Query<&Interaction, (Changed<Interaction>, With<Button>, Without<ToggleButton>, Without<ResetSaveButton>)>,
) {
    for (interaction, toggle, mut color, children) in &mut toggle_buttons {
        match *interaction {
            Interaction::Pressed => {
                let (label, new_value) = match toggle {
                    ToggleButton::Tts => {
                        settings.tts_enabled = !settings.tts_enabled;
                        ("TTS", settings.tts_enabled)
                    }
                    ToggleButton::Hints => {
                        settings.hints_enabled = !settings.hints_enabled;
                        ("Hints", settings.hints_enabled)
                    }
                };
                if let Err(e) = settings.save() {
                    warn!("Failed to save settings: {}", e);
                }
                let state_label = if new_value { "ON" } else { "OFF" };
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        *text = Text::new(format!("{}: {}", label, state_label));
                    }
                }
                info!("{} toggled to {}", label, state_label);
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        }
    }

    for interaction in &reset_buttons {
        if *interaction == Interaction::Pressed {
            if let Err(e) = std::fs::remove_file(crate::save::SaveData::SAVE_PATH) {
                warn!("No save file to reset (or error): {}", e);
            }
            info!("Save data reset.");
        }
    }

    for interaction in &menu_buttons {
        if *interaction == Interaction::Pressed {
            crate::commands::log_state_transition(&GameState::Settings, GameState::MainMenu);
            next_state.set(GameState::MainMenu);
        }
    }
}

fn cleanup_settings_ui(mut commands: Commands, query: Query<Entity, With<SettingsUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
