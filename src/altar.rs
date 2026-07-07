// altar.rs — 3D Holographic Programmer Art for the Summoning Altar
#![allow(dead_code)]
use bevy::prelude::*;
use crate::components::*;
use crate::letter::CurrentSpelling;

#[derive(Component)]
pub struct AltarUi;

#[derive(Component)]
pub struct AltarSpellingText;

#[derive(Component)]
pub struct AltarSummonButton;

pub struct AltarPlugin;

impl Plugin for AltarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_altar)
           .add_systems(Update, (update_altar_text, handle_summon_pinch));
    }
}

fn setup_altar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let altar_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0.1, 0.1, 0.95),
        emissive: Color::srgb(0.0, 0.2, 0.4).into(),
        metallic: 0.9,
        perceptual_roughness: 0.1,
        ..default()
    });

    let button_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.2, 0.2, 0.95),
        emissive: Color::srgb(0.5, 0.0, 0.0).into(),
        ..default()
    });

    // Spawn Altar Base (Cylinder)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.8, 1.0))),
        MeshMaterial3d(altar_mat),
        Transform::from_xyz(0.0, 0.5, -2.0),
        AltarUi,
    )).with_children(|parent| {
        // Holographic Text above altar
        parent.spawn((
            Text2d::new(""),
            TextFont { font_size: 60.0, ..default() },
            TextColor(Color::srgb(0.0, 1.0, 1.0)),
            Transform::from_xyz(0.0, 1.2, 0.0),
            AltarSpellingText,
        ));

        // Summon Button on top of altar
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.6, 0.1, 0.4))),
            MeshMaterial3d(button_mat),
            Transform::from_xyz(0.0, 0.55, 0.0),
            AltarSummonButton,
        )).with_children(|btn| {
            btn.spawn((
                Text2d::new("SUMMON"),
                TextFont { font_size: 30.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.06, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ));
        });
    });
}

fn update_altar_text(
    spelling: Res<CurrentSpelling>,
    mut query: Query<&mut Text2d, With<AltarSpellingText>>,
) {
    if spelling.is_changed() {
        for mut text in query.iter_mut() {
            text.0 = spelling.word.clone();
        }
    }
}

fn handle_summon_pinch(
    pinch_events: Res<crate::hand_tracking::PinchEvents>,
    mut buttons: Query<(&GlobalTransform, &mut Transform), With<AltarSummonButton>>,
    spelling: Res<CurrentSpelling>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    if *state.get() != GameState::Collecting && *state.get() != GameState::Constructing {
        return;
    }

    for pinch in &pinch_events.events {
        for (global_tf, mut tf) in buttons.iter_mut() {
            if global_tf.translation().distance(pinch.position) < 0.3 {
                // Button pressed!
                info!("Altar Summon Button Pressed! Word: {}", spelling.word);
                // Sink button visually
                tf.translation.y = 0.5;
                
                // If word is not empty, trigger summon
                if !spelling.word.is_empty() {
                    crate::commands::log_state_transition(state.get(), GameState::Reviewing);
                    next_state.set(GameState::Reviewing);
                }
            } else {
                // Reset button height
                tf.translation.y = 0.55;
            }
        }
    }
}
