//! Pet card reveal animation (the "Pokémon moment").
//!
//! When a player submits a valid spelling word the game enters [`GameState::RevealingPet`].
//! This module spawns a face-down card, flips it to reveal the pet's element, plays a
//! particle burst + sound, then spawns the actual [`PetAvatar`] and returns to
//! [`GameState::Playing`].
use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use faces_protocol::FacesState;

const DEFAULT_REVEAL_DURATION: f32 = 1.5;
#[cfg(not(feature = "flat2d"))]
const CARD_SIZE: Vec3 = Vec3::new(1.0, 1.4, 0.05);
#[cfg(not(feature = "flat2d"))]
const CARD_POSITION: Vec3 = PET_SPAWN_POSITION;
const PET_POSITION: Vec3 = PET_SPAWN_POSITION;
const FLIP_THRESHOLD: f32 = 0.5; // t value at which particles/sound trigger
const MIN_REVEAL_DURATION: f32 = 0.001; // guard against divide-by-zero in tests
const PARTICLE_COUNT: usize = 12;
const PARTICLE_LIFETIME: f32 = 0.8;
const PARTICLE_RADIUS: f32 = 0.04;
const PARTICLE_GRAVITY: f32 = 1.0;
const PARTICLE_MIN_SPEED: f32 = 0.5;
const PARTICLE_MAX_SPEED: f32 = 2.0;
const PARTICLE_MIN_UPWARD: f32 = 0.5;
const PARTICLE_MAX_UPWARD: f32 = 2.0;
#[cfg(not(feature = "flat2d"))]
const PET_SPHERE_RADIUS: f32 = 0.5;

#[derive(Resource, Debug, Clone)]
pub struct RevealConfig {
    pub duration: f32,
}

impl Default for RevealConfig {
    fn default() -> Self {
        Self { duration: DEFAULT_REVEAL_DURATION }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct PendingReveal {
    pub word: String,
    pub element: Element,
    pub role: Role,
    pub stats: PetStats,
    pub faces_state: FacesState,
    pub pet_type: SummonClass,
}

#[derive(Component)]
pub struct RevealCard {
    pub timer: f32,
    pub word: String,
    pub element: Element,
    pub role: Role,
    pub stats: PetStats,
    pub faces_state: FacesState,
    pub pet_type: SummonClass,
}

#[derive(Component)]
pub struct RevealParticle {
    pub lifetime: f32,
    pub velocity: Vec3,
}

/// UI label that tracks a pet entity and floats above it in screen space.
#[derive(Component)]
pub struct PetNameLabel {
    pub pet: Entity,
}

#[derive(Resource)]
struct RevealSounds {
    pet: Handle<AudioSource>,
}

pub struct PetRevealPlugin;

impl Plugin for PetRevealPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RevealConfig>()
           .add_systems(OnEnter(GameState::RevealingPet), spawn_reveal_card)
           .add_systems(Update, (
               animate_reveal_card,
               finish_reveal,
               update_reveal_particles,
           ).run_if(in_state(GameState::RevealingPet)))
           .add_systems(Update, (
               update_floating_pet_label,
               despawn_orphan_labels,
           ))
           .add_systems(OnExit(GameState::RevealingPet), cleanup_reveal_card);
    }
}

#[allow(unused_variables, unused_mut)]
fn spawn_reveal_card(
    mut commands: Commands,
    pending: Option<Res<PendingReveal>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // PendingReveal is consumed on entry so the card can store all data needed
    // to spawn the final pet at the end of the animation.
    let pending = match pending {
        Some(p) => p.clone(),
        None => {
            warn!("Entered RevealingPet without a PendingReveal resource");
            return;
        }
    };

    #[cfg(not(feature = "flat2d"))]
    // Card back material
    let back_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.1, 0.4),
        emissive: Color::srgb(0.1, 0.05, 0.2).into(),
        metallic: 0.8,
        perceptual_roughness: 0.2,
        ..default()
    });

    #[cfg(not(feature = "flat2d"))]
    commands.spawn((
        Name::new("Reveal Card"),
        Mesh3d(meshes.add(Cuboid::new(CARD_SIZE.x, CARD_SIZE.y, CARD_SIZE.z))),
        MeshMaterial3d(back_mat),
        Transform::from_translation(CARD_POSITION)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        RevealCard {
            timer: 0.0,
            word: pending.word,
            element: pending.element,
            role: pending.role,
            stats: pending.stats,
            faces_state: pending.faces_state,
            pet_type: pending.pet_type,
        },
    ));

    // 2D variant: a sprite that "flips" via scale.x rather than 3D rotation.
    #[cfg(feature = "flat2d")]
    commands.spawn((
        Name::new("Reveal Card 2D"),
        Sprite {
            color: Color::srgb(0.2, 0.1, 0.4),
            custom_size: Some(Vec2::new(256.0, 358.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)).with_scale(Vec3::new(-1.0, 1.0, 1.0)),
        RevealCard {
            timer: 0.0,
            word: pending.word,
            element: pending.element,
            role: pending.role,
            stats: pending.stats,
            faces_state: pending.faces_state,
            pet_type: pending.pet_type,
        },
    ));

    let attune_handle: Handle<AudioSource> = asset_server.load(crate::asset_catalog::SOUND_ATTUNE);
    let pet_handle: Handle<AudioSource> = asset_server.load(crate::asset_catalog::SOUND_PET);
    commands.insert_resource(RevealSounds {
        pet: pet_handle.clone(),
    });

    commands.spawn((
        AudioPlayer::<AudioSource>(attune_handle),
        PlaybackSettings::DESPAWN,
    ));
}

#[allow(clippy::type_complexity)]
fn animate_reveal_card(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<RevealConfig>,
    sounds: Res<RevealSounds>,
    mut cards: Query<(
        Entity,
        &mut Transform,
        &mut RevealCard,
        Option<&mut MeshMaterial3d<StandardMaterial>>,
        Option<&mut Sprite>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let duration = config.duration.max(MIN_REVEAL_DURATION);
    for (_entity, mut transform, mut card, _mat_handle, _sprite) in cards.iter_mut() {
        card.timer += time.delta_secs();
        let t = (card.timer / duration).clamp(0.0, 1.0);

        // 3D: flip from back (rotated 180°, t=0) to front (rotated 0°, t=1).
        #[cfg(not(feature = "flat2d"))]
        {
            let angle = std::f32::consts::PI * (1.0 - t);
            transform.rotation = Quat::from_rotation_y(angle);
        }

        // 2D: simulate a flip by scaling x from -1 (back) to 1 (front).
        #[cfg(feature = "flat2d")]
        {
            transform.scale.x = -1.0 + 2.0 * t;
        }

        // Spawn particles halfway through the flip.
        let prev_t = ((card.timer - time.delta_secs()) / duration).clamp(0.0, 1.0);
        if prev_t < FLIP_THRESHOLD && t >= FLIP_THRESHOLD {
            spawn_particle_burst(&mut commands, &mut meshes, &mut materials, transform.translation, card.element);
            commands.spawn((
                AudioPlayer::<AudioSource>(sounds.pet.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }

        // Switch to front-face material/color once we're facing the player.
        // Skip on the final frame so we don't queue a change on a despawning entity.
        if t > FLIP_THRESHOLD && card.timer < duration {
            #[cfg(not(feature = "flat2d"))]
            if let Some(mut mat_handle) = _mat_handle {
                mat_handle.0 = materials.add(card.element.material());
            }
            #[cfg(feature = "flat2d")]
            if let Some(mut sprite) = _sprite {
                sprite.color = card.element.color();
            }
        }
    }
}

fn finish_reveal(
    mut commands: Commands,
    config: Res<RevealConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cards: Query<(Entity, &RevealCard)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let duration = config.duration.max(MIN_REVEAL_DURATION);
    for (entity, card) in cards.iter() {
        if card.timer < duration {
            continue;
        }
        spawn_revealed_pet(&mut commands, &mut meshes, &mut materials, card);
        commands.entity(entity).despawn();
        commands.remove_resource::<PendingReveal>();
        let next = if cfg!(feature = "flat2d") { GameState::Exploring } else { GameState::Playing };
        crate::commands::log_state_transition(&GameState::RevealingPet, next.clone());
        next_state.set(next);
    }
}

#[allow(unused_variables)]
fn spawn_revealed_pet(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    card: &RevealCard,
) {
    #[cfg(not(feature = "flat2d"))]
    let main_mat = materials.add(card.element.material());

    #[cfg(not(feature = "flat2d"))]
    let mut entity = commands.spawn((
        Name::new(format!("Pet Avatar - {}", card.word)),
        Transform::from_translation(PET_POSITION),
        PetAvatar {
            word: card.word.clone(),
            pet_type: card.pet_type,
        },
        PetFacesState(card.faces_state),
        PetVisualState::Happy,
        AvatarAnimation {
            time: 0.0,
            base_y: PET_POSITION.y,
        },
        card.stats,
        card.element,
        card.role,
    ));

    #[cfg(feature = "flat2d")]
    let entity = commands.spawn((
        Name::new(format!("Pet Avatar - {}", card.word)),
        Transform::from_translation(PET_POSITION),
        PetAvatar {
            word: card.word.clone(),
            pet_type: card.pet_type,
        },
        PetFacesState(card.faces_state),
        PetVisualState::Happy,
        card.stats,
        card.element,
        card.role,
    ));

    #[cfg(not(feature = "flat2d"))]
    entity.insert((
        Mesh3d(meshes.add(Sphere::new(PET_SPHERE_RADIUS).mesh().ico(4).unwrap())),
        MeshMaterial3d(main_mat),
    ));

    let pet_entity = entity.id();
    let label_text = format!("{} | {:?} | {:?}", card.word, card.element, card.role);
    commands.spawn((
        Name::new("Floating Pet Label"),
        PetNameLabel { pet: pet_entity },
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        Text::new(label_text),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

#[allow(unused_variables)]
fn spawn_particle_burst(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    origin: Vec3,
    element: Element,
) {
    let color = element.color();

    #[cfg(not(feature = "flat2d"))]
    let mat = materials.add(StandardMaterial {
        base_color: color,
        emissive: (color.to_srgba() * 0.8).into(),
        ..default()
    });

    let mut rng = rand::thread_rng();
    for _ in 0..PARTICLE_COUNT {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(PARTICLE_MIN_SPEED..PARTICLE_MAX_SPEED);
        let upward = rng.gen_range(PARTICLE_MIN_UPWARD..PARTICLE_MAX_UPWARD);
        let velocity = Vec3::new(angle.cos() * speed, upward, angle.sin() * speed);

        #[cfg(not(feature = "flat2d"))]
        commands.spawn((
            Name::new("Reveal Particle"),
            Mesh3d(meshes.add(Sphere::new(PARTICLE_RADIUS))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(origin),
            RevealParticle { lifetime: PARTICLE_LIFETIME, velocity },
        ));

        #[cfg(feature = "flat2d")]
        {
            let pixel_velocity = Vec3::new(velocity.x * 100.0, velocity.y * 100.0, 0.0);
            commands.spawn((
                Name::new("Reveal Particle 2D"),
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(PARTICLE_RADIUS * 200.0)),
                    ..default()
                },
                Transform::from_translation(origin),
                RevealParticle { lifetime: PARTICLE_LIFETIME, velocity: pixel_velocity },
            ));
        }
    }
}

fn update_reveal_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut RevealParticle)>,
) {
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        particle.lifetime -= time.delta_secs();
        transform.translation += particle.velocity * time.delta_secs();
        particle.velocity.y -= PARTICLE_GRAVITY * time.delta_secs(); // gravity
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_reveal_card(
    mut commands: Commands,
    cards: Query<Entity, With<RevealCard>>,
    particles: Query<Entity, With<RevealParticle>>,
) {
    for entity in cards.iter() {
        commands.entity(entity).despawn();
    }
    for entity in particles.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<RevealSounds>();
    commands.remove_resource::<PendingReveal>();
}

/// Projects each pet's world position to screen space and places its label above it.
fn update_floating_pet_label(
    mut labels: Query<(&PetNameLabel, &mut Node)>,
    pets: Query<&GlobalTransform, With<PetAvatar>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = match camera.single() {
        Ok(c) => c,
        Err(_) => return,
    };
    for (label, mut node) in labels.iter_mut() {
        let Ok(pet_transform) = pets.get(label.pet) else { continue };
        let pet_pos = pet_transform.translation();
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, pet_pos) else { continue };
        // Offset slightly above the pet in screen pixels.
        node.left = Val::Px(viewport_pos.x);
        node.top = Val::Px(viewport_pos.y - 40.0);
    }
}

/// Removes labels whose pet entity no longer exists.
fn despawn_orphan_labels(
    mut commands: Commands,
    labels: Query<(Entity, &PetNameLabel)>,
    pets: Query<(), With<PetAvatar>>,
) {
    for (entity, label) in labels.iter() {
        if pets.get(label.pet).is_err() {
            commands.entity(entity).despawn();
        }
    }
}
