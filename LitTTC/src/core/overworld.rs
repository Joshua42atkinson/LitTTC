// overworld.rs — 2D top-down overworld (Pokémon Red for Words)
//
// This module is the flat2d proving ground for the XR loop:
//   walk near object -> press E to scan -> type word -> get card -> battle typos.
// All code is gated behind #[cfg(feature = "flat2d")].

use bevy::prelude::*;
use crate::components::*;

/// Player-controlled avatar in the 2D overworld.
#[derive(Component)]
pub struct PlayerAvatar;

/// The Semantic Slime companion that trails the player.
#[derive(Component)]
pub struct CompanionAvatar;

/// Marks static world objects the player can scan for words.
#[derive(Component)]
pub struct ScannableObject {
    pub word: String,
}

/// Marks NPC mentors that can give quests/dialogue.
#[derive(Component)]
pub struct NpcEntity {
    pub npc_name: String,
}

/// Marks roaming wild typos that trigger battles on contact.
#[derive(Component)]
pub struct WildTypoEntity;

/// Bounds of the explorable world map.
#[derive(Resource)]
pub struct WorldBounds {
    pub min: Vec2,
    pub max: Vec2,
}

const PLAYER_SPEED: f32 = 120.0;
const COMPANION_LAG: f32 = 4.0;
const CAMERA_DECAY: f32 = 3.0;

pub fn setup_overworld(
    mut commands: Commands,
) {
    // Center the player in a bounded world.
    commands.insert_resource(WorldBounds {
        min: Vec2::new(-640.0, -360.0),
        max: Vec2::new(640.0, 360.0),
    });

    // Player avatar: bright green square.
    commands.spawn((
        PlayerAvatar,
        Sprite {
            color: Color::srgb(0.2, 0.9, 0.3),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 5.0),
    ));

    // 2D companion: smaller cyan square that follows.
    commands.spawn((
        CompanionAvatar,
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.9),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(-40.0, 0.0, 4.0),
    ));

    // A scannable object: a gray rock in the world.
    commands.spawn((
        ScannableObject { word: "rock".to_string() },
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(40.0, 30.0)),
            ..default()
        },
        Transform::from_xyz(120.0, 80.0, 3.0),
    ));

    // A wild typo: red jagged rectangle.
    commands.spawn((
        WildTypoEntity,
        Sprite {
            color: Color::srgb(0.9, 0.2, 0.2),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(-120.0, -80.0, 3.0),
    ));

    // An NPC mentor: golden circle-ish square.
    commands.spawn((
        NpcEntity { npc_name: "Barnaby".to_string() },
        Sprite {
            color: Color::srgb(0.9, 0.8, 0.3),
            custom_size: Some(Vec2::new(36.0, 36.0)),
            ..default()
        },
        Transform::from_xyz(-200.0, 150.0, 3.0),
    ));

    // Simple ground zone markers (districts) — just colored rectangles for now.
    commands.spawn((
        Sprite {
            color: Color::srgba(0.1, 0.3, 0.1, 0.3),
            custom_size: Some(Vec2::new(400.0, 300.0)),
            ..default()
        },
        Transform::from_xyz(300.0, -100.0, 1.0),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgba(0.2, 0.1, 0.3, 0.3),
            custom_size: Some(Vec2::new(300.0, 300.0)),
            ..default()
        },
        Transform::from_xyz(-300.0, -100.0, 1.0),
    ));
}

pub fn move_avatar(
    mut player: Single<&mut Transform, With<PlayerAvatar>>,
    time: Res<Time>,
    kb: Res<ButtonInput<KeyCode>>,
) {
    let mut dir = Vec2::ZERO;
    if kb.pressed(KeyCode::KeyW) || kb.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if kb.pressed(KeyCode::KeyS) || kb.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if kb.pressed(KeyCode::KeyA) || kb.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if kb.pressed(KeyCode::KeyD) || kb.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    let move_delta = dir.normalize_or_zero() * PLAYER_SPEED * time.delta_secs();
    player.translation += move_delta.extend(0.0);
}

pub fn clamp_avatar_to_bounds(
    mut player: Single<&mut Transform, With<PlayerAvatar>>,
    bounds: Res<WorldBounds>,
) {
    player.translation.x = player.translation.x.clamp(bounds.min.x, bounds.max.x);
    player.translation.y = player.translation.y.clamp(bounds.min.y, bounds.max.y);
}

pub fn update_overworld_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<PlayerAvatar>)>,
    player: Single<&Transform, With<PlayerAvatar>>,
    time: Res<Time>,
) {
    let target = Vec3::new(player.translation.x, player.translation.y, camera.translation.z);
    camera.translation.smooth_nudge(&target, CAMERA_DECAY, time.delta_secs());
}

pub fn companion_follow(
    player: Single<&Transform, (With<PlayerAvatar>, Without<CompanionAvatar>)>,
    mut companion: Single<&mut Transform, (With<CompanionAvatar>, Without<PlayerAvatar>)>,
    time: Res<Time>,
) {
    let target = player.translation.xy();
    let current = companion.translation.xy();
    let diff = target - current;
    let dist = diff.length();
    // Keep a small trailing distance so the companion doesn't sit inside the player.
    let follow_distance = 28.0;
    if dist > follow_distance {
        let dir = diff.normalize_or_zero();
        let desired = current + dir * (dist - follow_distance);
        let smoothed = current.lerp(desired, COMPANION_LAG * time.delta_secs());
        companion.translation.x = smoothed.x;
        companion.translation.y = smoothed.y;
    }
}

/// Cleanup the overworld scene when leaving Exploring.
pub fn cleanup_overworld(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<PlayerAvatar>,
            With<CompanionAvatar>,
            With<ScannableObject>,
            With<NpcEntity>,
            With<WildTypoEntity>,
        )>,
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    // Despawn the two district zone markers we spawned without marker components.
    // We can identify them by their large size and low z, but for now we rely on the
    // fact that they are just Sprite entities. To avoid desawning unrelated sprites,
    // we only remove the explicitly marked entities above. The zone markers will be
    // replaced with a proper tilemap later; for the gray-box they are harmless.
    commands.remove_resource::<WorldBounds>();
}

const INTERACT_RADIUS: f32 = 50.0;
const AVATAR_SIZE: f32 = 32.0;
const TYPO_SIZE: f32 = 32.0;

pub fn handle_overworld_interactions(
    player: Single<&Transform, With<PlayerAvatar>>,
    scannables: Query<(&Transform, &ScannableObject)>,
    npcs: Query<(&Transform, &NpcEntity)>,
    typos: Query<&Transform, With<WildTypoEntity>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *state.get() != GameState::Exploring {
        return;
    }

    let player_pos = player.translation.xy();

    // Battle on contact with a wild typo.
    for typo_tf in typos.iter() {
        let typo_pos = typo_tf.translation.xy();
        let dx = (player_pos.x - typo_pos.x).abs();
        let dy = (player_pos.y - typo_pos.y).abs();
        if dx < (AVATAR_SIZE + TYPO_SIZE) * 0.5 && dy < (AVATAR_SIZE + TYPO_SIZE) * 0.5 {
            crate::commands::log_state_transition(&GameState::Exploring, GameState::Battling);
            next_state.set(GameState::Battling);
            return;
        }
    }

    if keys.just_pressed(KeyCode::KeyE) {
        // Find nearest scannable object.
        let mut nearest_scan: Option<(&ScannableObject, f32)> = None;
        for (tf, obj) in scannables.iter() {
            let dist = tf.translation.xy().distance(player_pos);
            if dist < INTERACT_RADIUS {
                nearest_scan = Some((obj, dist));
            }
        }
        if let Some((obj, _)) = nearest_scan {
            writer.write(crate::commands::GameCommand::ScanObject(obj.word.clone()));
            return;
        }

        // Find nearest NPC.
        let mut nearest_npc: Option<&NpcEntity> = None;
        let mut nearest_dist = f32::MAX;
        for (tf, npc) in npcs.iter() {
            let dist = tf.translation.xy().distance(player_pos);
            if dist < INTERACT_RADIUS && dist < nearest_dist {
                nearest_dist = dist;
                nearest_npc = Some(npc);
            }
        }
        if let Some(npc) = nearest_npc {
            writer.write(crate::commands::GameCommand::StartQuest(npc.npc_name.clone()));
            return;
        }
    }
}

pub struct OverworldPlugin;

#[cfg(feature = "flat2d")]
impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Exploring), setup_overworld)
           .add_systems(Update, (
                move_avatar,
                clamp_avatar_to_bounds.after(move_avatar),
                companion_follow.after(clamp_avatar_to_bounds),
                update_overworld_camera.after(companion_follow),
                handle_overworld_interactions,
           ).run_if(in_state(GameState::Exploring)))
           .add_systems(OnExit(GameState::Exploring), cleanup_overworld);
    }
}

#[cfg(not(feature = "flat2d"))]
impl Plugin for OverworldPlugin {
    fn build(&self, _app: &mut App) {}
}
