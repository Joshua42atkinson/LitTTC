// render.rs — AAA+ Procedural 3D Pet, Emissive Stage, and Helical Aura Particle Renderer
use bevy::prelude::*;
#[cfg(not(feature = "flat2d"))]
use faces_protocol::{Container, Focus, Action};
use crate::components::*;

#[cfg(not(feature = "flat2d"))]
const AURA_PARTICLE_COUNT: usize = 10;
#[cfg(not(feature = "flat2d"))]
const BURST_PARTICLE_COUNT: usize = 20;

#[cfg(not(feature = "flat2d"))]
pub struct RenderPlugin;

#[cfg(not(feature = "flat2d"))]
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_world, spawn_visual_stage))
           .add_systems(Update, (
                spawn_avatar_visuals,
                animate_avatars,
                animate_rings,
                animate_wings,
                animate_ears,
                animate_particles,
                update_head_materials,
                update_pet_eyes,
                update_pet_mouths,
                scale_gltf_face_nodes,
                apply_screen_shake,
                animate_burst_particles,
                update_sky_lighting,
                animate_unstable_mutants,
           ));
    }
}

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct SkyLight;

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct ScreenShake {
    pub timer: f32,
    pub intensity: f32,
}

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct BurstParticle {
    pub velocity: Vec3,
    pub timer: f32,
}

#[cfg(not(feature = "flat2d"))]
// Marker components for sub-parts of the pet
#[derive(Component)]
pub struct PetEye;

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct PetMouth;

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct PetWing {
    pub side: f32, // -1.0 for left, 1.0 for right
}

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct PetEar {
    pub side: f32, // -1.0 for left, 1.0 for right
}

#[cfg(not(feature = "flat2d"))]
#[derive(Component)]
pub struct AuraParticle {
    pub index: usize,
    pub speed: f32,
}

#[cfg(not(feature = "flat2d"))]
// Spawns the neon holographic stage and boundary pillars
fn spawn_visual_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Spawning AAA+ Holographic Stage Environment...");

    // Holographic stage disc at origin
    let stage_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.6, 0.9, 0.4),
        emissive: Color::srgb(0.0, 0.4, 0.8).into(),
        metallic: 0.9,
        perceptual_roughness: 0.1,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(2.2, 0.05))),
        MeshMaterial3d(stage_mat),
        Transform::from_xyz(0.0, 0.02, -2.0),
    ));

    // Outer glow boundary ring
    let ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.0, 0.8, 0.6),
        emissive: Color::srgb(0.6, 0.0, 0.6).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(0.02, 2.22))),
        MeshMaterial3d(ring_mat),
        Transform::from_xyz(0.0, 0.03, -2.0).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ));

    // Four corner pillars emitting light
    let pillar_positions = [
        Vec3::new(-2.5, 1.5, -4.5),
        Vec3::new(2.5, 1.5, -4.5),
        Vec3::new(-2.5, 1.5, 0.5),
        Vec3::new(2.5, 1.5, 0.5),
    ];

    let pillar_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.12, 0.18),
        metallic: 0.9,
        perceptual_roughness: 0.2,
        ..default()
    });

    let crystal_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::srgb(0.0, 0.8, 0.8).into(),
        ..default()
    });

    for pos in pillar_positions {
        // Base post
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.12, 3.0))),
            MeshMaterial3d(pillar_mat.clone()),
            Transform::from_translation(pos),
        ));

        // Floating glowing crystal cap
        commands.spawn((
            Mesh3d(meshes.add(Cone::new(0.2, 0.5))),
            MeshMaterial3d(crystal_mat.clone()),
            Transform::from_translation(pos + Vec3::new(0.0, 1.8, 0.0)),
        ));

        // Neon glow point light on each pillar
        commands.spawn((
            PointLight {
                color: Color::srgb(0.0, 0.8, 0.8),
                intensity: 60000.0,
                range: 12.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_translation(pos + Vec3::new(0.0, 1.5, 0.0)),
        ));
    }
}

#[cfg(not(feature = "flat2d"))]
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn camera with HDR, Bloom, and Screen-Space Ambient Occlusion (SSAO) for Desktop
    #[cfg(all(not(feature = "xr"), not(target_arch = "wasm32")))]
    commands.spawn((
        Camera3d::default(),
        bevy::render::view::Hdr,
        bevy::post_process::bloom::Bloom::NATURAL,
        bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));

    // Spawn lightweight camera for XR or WASM
    #[cfg(any(feature = "xr", target_arch = "wasm32"))]
    commands.spawn((
        Camera3d::default(),
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
        SkyLight,
    ));

    info!("3D Environment initialized successfully!");
}

// Helper to convert ANSI 256 color index to Color
pub fn ansi_to_color(idx: u8) -> Color {
    if idx < 8 {
        match idx {
            0 => Color::BLACK,
            1 => Color::srgb(0.8, 0.0, 0.0), // Red
            2 => Color::srgb(0.0, 0.8, 0.0), // Green
            3 => Color::srgb(0.8, 0.8, 0.0), // Yellow
            4 => Color::srgb(0.0, 0.0, 0.8), // Blue
            5 => Color::srgb(0.8, 0.0, 0.8), // Magenta
            6 => Color::srgb(0.0, 0.8, 0.8), // Cyan
            _ => Color::srgb(0.8, 0.8, 0.8), // White
        }
    } else if idx < 16 {
        match idx {
            8 => Color::srgb(0.3, 0.3, 0.3), // Bright Black
            9 => Color::srgb(1.0, 0.3, 0.3), // Bright Red
            10 => Color::srgb(0.3, 1.0, 0.3), // Bright Green
            11 => Color::srgb(1.0, 1.0, 0.3), // Bright Yellow
            12 => Color::srgb(0.3, 0.3, 1.0), // Bright Blue
            13 => Color::srgb(1.0, 0.3, 1.0), // Bright Magenta
            14 => Color::srgb(0.3, 1.0, 1.0), // Bright Cyan
            _ => Color::WHITE,
        }
    } else if idx < 232 {
        let val = idx - 16;
        let r = ((val / 36) as f32) / 5.0;
        let g = (((val / 6) % 6) as f32) / 5.0;
        let b = ((val % 6) as f32) / 5.0;
        Color::srgb(r, g, b)
    } else {
        let val = (idx - 232) as f32 / 24.0;
        Color::srgb(val, val, val)
    }
}

#[cfg(not(feature = "flat2d"))]
// Spawns the meshes for the pet's container, eyes, and mouth when a PetAvatar is spawned
fn spawn_avatar_visuals(
    mut commands: Commands,
    query: Query<(Entity, &PetAvatar, &PetFacesState, &Element), Added<PetAvatar>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let _ = asset_server;
    for (entity, avatar, faces_state, element) in query.iter() {
        info!("Spawning visuals for pet: {}", avatar.word);

        let head_color = ansi_to_color(faces_state.0.aura.index());
        let metallic_val = 0.8;
        let roughness_val = 0.15;
        
        match avatar.pet_type {
            SummonClass::SemanticSlime => {}
        }

        let head_mat = materials.add(StandardMaterial {
            base_color: head_color,
            emissive: (head_color.to_srgba() * 0.5).into(),
            metallic: metallic_val,
            perceptual_roughness: roughness_val,
            ..default()
        });

        // Determine Head Mesh based on Container and Pet Type
        let head_mesh = match (avatar.pet_type, faces_state.0.container) {
            (SummonClass::SemanticSlime, Container::Neutral) => meshes.add(Sphere::new(0.5).mesh().ico(4).unwrap()),
            (SummonClass::SemanticSlime, Container::Rigid) => meshes.add(Cuboid::new(0.8, 0.8, 0.8)),
            (SummonClass::SemanticSlime, Container::Fluid) => meshes.add(Torus::new(0.15, 0.45)),
            (SummonClass::SemanticSlime, Container::Defensive) => meshes.add(Cylinder::new(0.4, 0.8)),
            (SummonClass::SemanticSlime, Container::Sharp) => meshes.add(Cone::new(0.5, 0.8)),
        };

        let lower_archetype = match faces_state.0.container {
            Container::Neutral => "slime",
            Container::Rigid => "golem",
            Container::Fluid => "robot",
            Container::Defensive => "defender",
            Container::Sharp => "sharp",
        };
        let asset_relative = format!("pets/{}.glb", lower_archetype);

        #[cfg(not(target_arch = "wasm32"))]
        let has_gltf = {
            let asset_full = format!("{}{}", crate::asset_catalog::ASSETS_DIR, asset_relative);
            std::path::Path::new(&asset_full).exists()
        };
        #[cfg(target_arch = "wasm32")]
        let has_gltf = false;

        if has_gltf {
            commands.entity(entity).insert(SceneRoot(asset_server.load(format!("{}#Scene0", asset_relative))));
        } else {
            commands.entity(entity).insert((
                Mesh3d(head_mesh),
                MeshMaterial3d(head_mat),
            ));
        }

        commands.entity(entity).with_children(|parent| {
            // Spawn inner glow core
            let core_color = element.color();
            let core_mat = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: core_color.into(),
                ..default()
            });
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.18).mesh().ico(2).unwrap())),
                MeshMaterial3d(core_mat),
            ));

            // Dynamic Point Light parented to core for glowing stage casting
            parent.spawn((
                PointLight {
                    color: head_color,
                    intensity: 90000.0,
                    shadows_enabled: true,
                    range: 6.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));

            // Spawn Eyes
            let eye_mat = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: Color::WHITE.into(),
                ..default()
            });

            for side in [-1.0f32, 1.0f32] {
                parent.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.06).mesh().ico(2).unwrap())),
                    MeshMaterial3d(eye_mat.clone()),
                    Transform::from_xyz(side * 0.2, 0.1, -0.45),
                    PetEye,
                ));
            }

            // Spawn Mouth
            let mouth_mat = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                emissive: Color::WHITE.into(),
                ..default()
            });

            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.2, 0.03, 0.03))),
                MeshMaterial3d(mouth_mat),
                Transform::from_xyz(0.0, -0.15, -0.45),
                PetMouth,
            ));

            // Spawn dynamic flapping wings
            let wing_mat = materials.add(StandardMaterial {
                base_color: Color::srgba(0.9, 0.9, 0.9, 0.5),
                emissive: (core_color.to_srgba() * 0.4).into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            for side in [-1.0f32, 1.0f32] {
                parent.spawn((
                    Mesh3d(meshes.add(Cone::new(0.15, 0.6))),
                    MeshMaterial3d(wing_mat.clone()),
                    Transform::from_xyz(side * 0.65, 0.0, 0.0)
                        .with_rotation(Quat::from_rotation_z(side * std::f32::consts::FRAC_PI_3)),
                    PetWing { side },
                ));
            }

            // Spawn dynamic ears
            let ear_mat = materials.add(StandardMaterial {
                base_color: head_color,
                emissive: (head_color.to_srgba() * 0.3).into(),
                ..default()
            });

            for side in [-1.0f32, 1.0f32] {
                parent.spawn((
                    Mesh3d(meshes.add(Cone::new(0.08, 0.35))),
                    MeshMaterial3d(ear_mat.clone()),
                    Transform::from_xyz(side * 0.38, 0.45, 0.0)
                        .with_rotation(Quat::from_rotation_z(-side * 0.3)),
                    PetEar { side },
                ));
            }

            // Spawn orbital rings
            let ring_mat = materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.4),
                emissive: (head_color.to_srgba() * 0.3).into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            parent.spawn((
                Mesh3d(meshes.add(Torus::new(0.02, 0.75))),
                MeshMaterial3d(ring_mat),
                Transform::from_xyz(0.0, 0.0, 0.0),
                OrbitalRing,
            ));

            // Spawn spiraling aura particles (glowing spheres)
            let particle_mat = materials.add(StandardMaterial {
                base_color: core_color,
                emissive: (core_color.to_srgba() * 1.5).into(),
                ..default()
            });

            for i in 0..AURA_PARTICLE_COUNT {
                parent.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.03).mesh().ico(1).unwrap())),
                    MeshMaterial3d(particle_mat.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    AuraParticle { index: i, speed: 2.0 },
                ));
            }
        });

        // Spawn Burst Particles independently
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..BURST_PARTICLE_COUNT {
            let vx = rng.gen_range(-3.0..3.0);
            let vy = rng.gen_range(1.0..5.0);
            let vz = rng.gen_range(-3.0..3.0);
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.05).mesh().ico(1).unwrap())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: element.color(),
                    emissive: (element.color().to_srgba() * 2.0).into(),
                    ..default()
                })),
                Transform::from_xyz(0.0, 1.0, -1.0),
                BurstParticle {
                    velocity: Vec3::new(vx, vy, vz),
                    timer: 1.5,
                }
            ));
        }
    }
}

#[cfg(not(feature = "flat2d"))]
fn animate_avatars(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut AvatarAnimation, &PetVisualState)>,
) {
    for (mut tf, mut anim, state) in &mut query {
        anim.time += time.delta_secs();

        let t = 1.0; // Simplification of ease transition
        match state {
            PetVisualState::Idle => {
                tf.translation.y = anim.base_y + (anim.time * 1.5).sin() * 0.08 * t;
                tf.rotate_y(time.delta_secs() * 0.2);
            }
            PetVisualState::Alert => {
                tf.translation.y = anim.base_y + 0.15 * t;
                tf.rotate_y(time.delta_secs() * 1.2);
            }
            PetVisualState::Battle => {
                let jitter = (anim.time * 25.0).sin() * 0.015 * t;
                tf.translation.x = jitter;
                tf.rotate_y(time.delta_secs() * 2.0);
            }
            PetVisualState::Happy => {
                tf.translation.y = anim.base_y + (anim.time * 6.0).sin().abs() * 0.2 * t;
                tf.rotate_y(time.delta_secs() * 0.5);
            }
        }
    }
}

#[cfg(not(feature = "flat2d"))]
fn animate_wings(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PetWing)>,
) {
    let t = time.elapsed_secs();
    for (mut tf, wing) in &mut query {
        let flap = (t * 6.0).sin() * 0.3 * wing.side;
        tf.rotation = Quat::from_rotation_z(wing.side * std::f32::consts::FRAC_PI_3 + flap);
    }
}

#[cfg(not(feature = "flat2d"))]
fn animate_ears(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PetEar)>,
) {
    let t = time.elapsed_secs();
    for (mut tf, ear) in &mut query {
        let wiggle = (t * 4.0).sin() * 0.12 * ear.side;
        tf.rotation = Quat::from_rotation_z(-ear.side * 0.3 + wiggle);
    }
}

#[cfg(not(feature = "flat2d"))]
fn animate_particles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &AuraParticle)>,
) {
    let t = time.elapsed_secs();
    let count = AURA_PARTICLE_COUNT as f32;
    for (mut tf, particle) in &mut query {
        let angle = t * particle.speed + (particle.index as f32) * (2.0 * std::f32::consts::PI / count);
        let radius = 0.95 + (t + particle.index as f32).sin() * 0.08;
        let y_pos = ((particle.index as f32 / count) - 0.5) + (t * 0.6).sin() * 0.12;
        
        tf.translation.x = radius * angle.cos();
        tf.translation.z = radius * angle.sin();
        tf.translation.y = y_pos;
    }
}

#[cfg(not(feature = "flat2d"))]
fn animate_rings(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<OrbitalRing>>,
) {
    for mut tf in &mut query {
        tf.rotate_y(time.delta_secs() * 0.8);
    }
}

#[cfg(not(feature = "flat2d"))]
fn update_head_materials(
    mut materials: ResMut<Assets<StandardMaterial>>,
    heads: Query<(&PetFacesState, &MeshMaterial3d<StandardMaterial>)>,
) {
    for (faces_state, mat_handle) in heads.iter() {
        if let Some(mat) = materials.get_mut(mat_handle) {
            let color = ansi_to_color(faces_state.0.aura.index());
            mat.base_color = color;
            mat.emissive = (color.to_srgba() * 0.5).into();
        }
    }
}

#[cfg(not(feature = "flat2d"))]
fn update_pet_eyes(
    all_heads: Query<&PetFacesState>,
    mut eyes: Query<(&ChildOf, &mut Transform), With<PetEye>>,
) {
    for (parent, mut transform) in &mut eyes {
        if let Ok(faces_state) = all_heads.get(parent.parent()) {
            let scale = match faces_state.0.focus {
                Focus::Neutral => Vec3::splat(1.0),
                Focus::Intense => Vec3::new(1.0, 0.3, 1.0), // Squint
                Focus::Open => Vec3::splat(1.8), // Wide eyes
                Focus::Distant => Vec3::splat(0.6), // Dim
                Focus::Happy => Vec3::new(1.2, 1.2, 0.4), // Arch
                Focus::Tired => Vec3::new(1.0, 0.1, 1.0), // Flat
            };
            transform.scale = scale;
        }
    }
}

#[cfg(not(feature = "flat2d"))]
fn update_pet_mouths(
    all_heads: Query<&PetFacesState>,
    mut mouths: Query<(&ChildOf, &mut Transform), With<PetMouth>>,
) {
    for (parent, mut transform) in &mut mouths {
        if let Ok(faces_state) = all_heads.get(parent.parent()) {
            let scale = match faces_state.0.action {
                Action::Withheld => Vec3::new(1.0, 0.2, 1.0),
                Action::Assertive => Vec3::new(0.6, 1.0, 1.0),
                Action::Playful => Vec3::new(1.2, 0.6, 1.0),
                Action::Thoughtful => Vec3::new(0.8, 0.3, 1.0),
                Action::Hesitant => Vec3::splat(0.4),
            };
            transform.scale = scale;
        }
    }
}

#[cfg(not(feature = "flat2d"))]
fn scale_gltf_face_nodes(
    mut query: Query<(&Name, &ChildOf, &mut Transform)>,
    all_heads: Query<&PetFacesState>,
    all_parents: Query<&ChildOf>,
) {
    for (name, child_of, mut transform) in &mut query {
        let name_str = name.as_str();
        if name_str == "Eye_Left" || name_str == "Eye_Right" || name_str == "Mouth" {
            let mut curr = child_of.parent();
            let mut faces_state = None;
            for _ in 0..10 {
                if let Ok(state) = all_heads.get(curr) {
                    faces_state = Some(state);
                    break;
                }
                if let Ok(parent_child) = all_parents.get(curr) {
                    curr = parent_child.parent();
                } else {
                    break;
                }
            }

            if let Some(faces_state) = faces_state {
                if name_str == "Eye_Left" || name_str == "Eye_Right" {
                    let scale = match faces_state.0.focus {
                        Focus::Neutral => Vec3::splat(1.0),
                        Focus::Intense => Vec3::new(1.0, 0.3, 1.0),
                        Focus::Open => Vec3::splat(1.8),
                        Focus::Distant => Vec3::splat(0.6),
                        Focus::Happy => Vec3::new(1.2, 1.2, 0.4),
                        Focus::Tired => Vec3::new(1.0, 0.1, 1.0),
                    };
                    transform.scale = scale;
                } else if name_str == "Mouth" {
                    let scale = match faces_state.0.action {
                        Action::Withheld => Vec3::new(1.0, 0.2, 1.0),
                        Action::Assertive => Vec3::new(0.6, 1.0, 1.0),
                        Action::Playful => Vec3::new(1.2, 0.6, 1.0),
                        Action::Thoughtful => Vec3::new(0.8, 0.3, 1.0),
                        Action::Hesitant => Vec3::splat(0.4),
                    };
                    transform.scale = scale;
                }
            }
        }
    }
}

#[cfg(feature = "flat2d")]
pub struct RenderPlugin;

#[cfg(feature = "flat2d")]
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_2d_camera_and_background)
           .add_systems(Update, (
               spawn_2d_pet_avatars,
               update_2d_pet_expressions,
           ));
    }
}

#[cfg(feature = "flat2d")]
fn setup_2d_camera_and_background(mut commands: Commands) {
    // Camera2d automatically sets the correct 2D orthographic projection and Core2d render graph.
    // IsDefaultUiCamera makes the main menu / HUD render to this camera.
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
    ));

    // Spawn simple 2D dark background behind everything at z=-1.
    commands.spawn((
        Sprite {
            color: Color::srgba(0.02, 0.02, 0.08, 1.0),
            custom_size: Some(Vec2::new(3000.0, 3000.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

#[cfg(feature = "flat2d")]
fn spawn_2d_pet_avatars(
    mut commands: Commands,
    query: Query<(Entity, &PetAvatar, &PetFacesState, &Element), Added<PetAvatar>>,
) {
    for (entity, avatar, _faces_state, element) in query.iter() {
        info!("Spawning flat 2D visuals for pet: {}", avatar.word);
        
        let color = element.color();
        
        // Spawn a flat circular/square sprite for the pet, scaled for the 256x224 viewport.
        commands.entity(entity).insert((
            Sprite {
                color,
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        )).with_children(|parent| {
            // Spawn eyes
            parent.spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(-5.0, 2.0, 1.0),
            ));
            parent.spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(5.0, 2.0, 1.0),
            ));

            // Spawn mouth
            parent.spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(6.0, 2.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, -4.0, 1.0),
            ));

            // Label showing pet name
            parent.spawn((
                Text2d::new(&avatar.word),
                TextFont { font_size: 4.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 16.0, 2.0),
            ));
        });
    }
}

#[cfg(feature = "flat2d")]
fn update_2d_pet_expressions(
    mut query: Query<(&PetFacesState, &mut Sprite), Changed<PetFacesState>>,
) {
    for (faces_state, mut sprite) in &mut query {
        let color = ansi_to_color(faces_state.0.aura.index());
        sprite.color = color;
    }
}

#[cfg(not(feature = "flat2d"))]
fn apply_screen_shake(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ScreenShake), With<Camera>>,
) {
    for (entity, mut tf, mut shake) in &mut query {
        shake.timer -= time.delta_secs();
        if shake.timer <= 0.0 {
            tf.translation = Vec3::new(0.0, 2.0, 5.0);
            commands.entity(entity).remove::<ScreenShake>();
        } else {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let offset_x = rng.gen_range(-shake.intensity..shake.intensity);
            let offset_y = rng.gen_range(-shake.intensity..shake.intensity);
            tf.translation = Vec3::new(offset_x, 2.0 + offset_y, 5.0);
        }
    }
}

#[cfg(not(feature = "flat2d"))]
fn animate_burst_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BurstParticle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut burst) in &mut query {
        burst.timer -= dt;
        if burst.timer <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            tf.translation += burst.velocity * dt;
            burst.velocity *= 0.95; // drag
        }
    }
}

#[cfg(not(feature = "flat2d"))]
pub fn update_sky_lighting(
    cycle: Res<crate::time_cycle::DayNightCycle>,
    mut query: Query<&mut DirectionalLight, With<SkyLight>>,
    mut clear_color: ResMut<ClearColor>,
) {
    use crate::time_cycle::TimeOfDay;
    
    let t = cycle.time_elapsed / cycle.phase_duration;
    
    let (c1, c2, l1, l2) = match cycle.current_phase {
        TimeOfDay::Dawn => (
            Color::srgb(0.05, 0.05, 0.1), Color::srgb(0.4, 0.4, 0.6),
            Color::srgb(0.2, 0.2, 0.5), Color::srgb(0.8, 0.8, 0.9)
        ),
        TimeOfDay::Day => (
            Color::srgb(0.4, 0.4, 0.6), Color::srgb(0.2, 0.3, 0.5),
            Color::srgb(0.8, 0.8, 0.9), Color::srgb(1.0, 1.0, 0.9)
        ),
        TimeOfDay::Dusk => (
            Color::srgb(0.2, 0.3, 0.5), Color::srgb(0.1, 0.05, 0.1),
            Color::srgb(1.0, 1.0, 0.9), Color::srgb(0.8, 0.3, 0.1)
        ),
        TimeOfDay::Night => (
            Color::srgb(0.1, 0.05, 0.1), Color::srgb(0.05, 0.05, 0.1),
            Color::srgb(0.8, 0.3, 0.1), Color::srgb(0.2, 0.2, 0.5)
        ),
    };

    let bg = c1.mix(&c2, t);
    let light_col = l1.mix(&l2, t);

    clear_color.0 = bg;
    for mut light in &mut query {
        light.color = light_col;
    }
}

#[cfg(not(feature = "flat2d"))]
pub fn animate_unstable_mutants(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut crate::components::UnstableWord, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for (entity, mut tf, mut unstable, mat_handle) in &mut query {
        unstable.health -= time.delta_secs() * 10.0;
        
        if unstable.health <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        tf.scale = Vec3::splat(1.0 + rng.gen_range(-0.2..0.2));
        tf.translation.x += rng.gen_range(-0.05..0.05);
        tf.translation.y += rng.gen_range(-0.05..0.05);
        
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            if rng.gen_bool(0.1) {
                mat.base_color = Color::srgb(rng.gen(), rng.gen(), rng.gen());
            }
        }
    }
}
