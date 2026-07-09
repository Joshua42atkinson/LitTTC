// companion.rs — Persistent player companion that floats alongside the player camera.
//
// The companion is the emotional anchor of the Tao of Fun: a pet the child chooses
// from their collection and which follows them through the world.

use bevy::prelude::*;

#[cfg(not(feature = "flat2d"))]
use crate::components::*;
#[cfg(not(feature = "flat2d"))]
use crate::pet_reveal::PetNameLabel;
#[cfg(not(feature = "flat2d"))]
use faces_protocol::detect::detect_scored;

pub struct CompanionPlugin;

/// Tracks the in-world companion entity so it is not duplicated every state transition.
#[derive(Resource, Default, Debug, Clone)]
pub struct CompanionState {
    pub entity: Option<Entity>,
    pub word: Option<String>,
}

/// Marker component for the persistent companion entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Companion;

impl Plugin for CompanionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompanionState>();
        #[cfg(not(feature = "flat2d"))]
        build_3d_systems(app);
    }
}

#[cfg(not(feature = "flat2d"))]
fn build_3d_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), spawn_companion)
       .add_systems(Update, follow_camera.run_if(in_state(GameState::Playing)));
}

#[cfg(not(feature = "flat2d"))]
const COMPANION_OFFSET: Vec3 = Vec3::new(1.4, -0.3, -1.2);

#[cfg(not(feature = "flat2d"))]
const COMPANION_FOLLOW_SPEED: f32 = 3.0;

#[cfg(not(feature = "flat2d"))]
const COMPANION_BASE_Y: f32 = 1.3;

#[cfg(not(feature = "flat2d"))]
fn spawn_companion(
    mut commands: Commands,
    mut state: ResMut<CompanionState>,
    spellbook: Res<SpellBook>,
    existing: Query<(Entity, &PetAvatar), With<Companion>>,
    children_query: Query<&Children>,
) {
    let selected_word = spellbook.entries.iter().find(|e| e.companion).map(|e| e.word.clone());

    if state.word.as_deref() == selected_word.as_deref() {
        if let Some(entity) = state.entity {
            if existing.get(entity).is_ok() {
                return;
            }
        }
    }

    for (entity, _) in existing.iter() {
        despawn_with_children(&mut commands, entity, &children_query);
    }
    state.word = None;
    state.entity = None;

    fn despawn_with_children(commands: &mut Commands, entity: Entity, children_query: &Query<&Children>) {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                despawn_with_children(commands, child, children_query);
            }
        }
        commands.entity(entity).despawn();
    }

    let Some(word) = selected_word else { return };
    let Some(entry) = spellbook.entries.iter().find(|e| e.word == word) else { return };

    let detected = detect_scored(&word.to_lowercase());
    let element = entry.element.unwrap_or(Element::Normal);
    let role = entry.role.unwrap_or(Role::Bruiser);
    let stats = entry.stats.unwrap_or(PetStats {
        logos: 10.0,
        pathos: 10.0,
        ethos: 10.0,
        speed: 10.0,
    });

    let start_pos = Vec3::new(COMPANION_OFFSET.x, COMPANION_BASE_Y, COMPANION_OFFSET.z);
    let companion_entity = commands.spawn((
        Name::new(format!("Companion - {}", word)),
        Transform::from_translation(start_pos),
        Companion,
        PetAvatar {
            word: word.clone(),
            pet_type: SummonClass::SemanticSlime,
        },
        PetFacesState(detected.state),
        PetVisualState::Idle,
        element,
        role,
        stats,
    )).id();

    let label_text = format!("{} | {:?} | {:?}", word, element, role);
    commands.spawn((
        Name::new("Companion Label"),
        PetNameLabel { pet: companion_entity },
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        Text::new(label_text),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::WHITE),
    ));

    state.entity = Some(companion_entity);
    state.word = Some(word);
}

#[cfg(not(feature = "flat2d"))]
fn follow_camera(
    time: Res<Time>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut companion: Query<&mut Transform, With<Companion>>,
) {
    let Some((_, camera_tf)) = cameras.iter().next() else { return };

    let camera_transform = camera_tf.compute_transform();
    let target = camera_transform.translation + camera_transform.rotation * COMPANION_OFFSET;

    let dt = time.delta_secs();
    let alpha = (dt * COMPANION_FOLLOW_SPEED).min(1.0);
    for mut tf in companion.iter_mut() {
        tf.translation = tf.translation.lerp(target, alpha);
    }
}
