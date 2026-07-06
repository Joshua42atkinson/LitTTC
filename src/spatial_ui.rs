// spatial_ui.rs — 3D Spatial UI panels and holographic button pointer observers
#![allow(dead_code)]
use bevy::prelude::*;

pub const COLOR_BTN_NORMAL: Color = Color::srgba(0.1, 0.15, 0.25, 0.8);
pub const COLOR_BTN_HOVER: Color = Color::srgba(0.2, 0.4, 0.6, 0.95);
pub const COLOR_BTN_PRESS: Color = Color::srgba(0.3, 0.6, 0.8, 1.0);
pub const COLOR_SAO_CYAN: Color = Color::srgb(0.0, 1.0, 1.0);

pub struct SpatialUiPlugin;

impl Plugin for SpatialUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_system_menu, setup_action_console))
           .add_systems(Update, (toggle_system_menu, handle_spatial_button_pinches));
    }
}

#[derive(Component)]
pub struct SystemMenuPanel;

#[derive(Component)]
pub struct SpatialButton {
    pub base_scale: Vec3,
}

#[derive(Component)]
pub struct ButtonHovered;

#[derive(Component)]
pub struct ButtonAction(pub String);

pub fn spawn_spatial_panel(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    translation: Vec3,
    title: &str,
) -> Entity {
    // Glassmorphism SAO Panel
    let panel_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.05, 0.05, 0.08, 0.85),
        emissive: Color::srgba(0.0, 0.5, 1.0, 0.1).into(),
        metallic: 0.3,
        perceptual_roughness: 0.2,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 2.0, 0.05))),
        MeshMaterial3d(panel_mat),
        Transform::from_translation(translation),
    )).with_children(|parent| {
        // Title Text
        parent.spawn((
            Text2d::new(title),
            TextFont { font_size: 40.0, ..default() },
            TextColor(COLOR_SAO_CYAN),
            Transform::from_xyz(0.0, 0.8, 0.03),
        ));
    }).id()
}

pub fn spawn_holographic_button(
    commands: &mut Commands,
    parent_entity: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    translation: Vec3,
    label: &str,
) {
    let btn_mat = materials.add(StandardMaterial {
        base_color: COLOR_BTN_NORMAL,
        emissive: (COLOR_BTN_NORMAL.to_srgba() * 0.5).into(),
        metallic: 0.8,
        perceptual_roughness: 0.1,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let btn_entity = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.3, 0.05))),
        MeshMaterial3d(btn_mat),
        Transform::from_translation(translation),
        SpatialButton { base_scale: Vec3::ONE },
        ButtonAction(label.to_string()),
    ))
    .with_children(|inner| {
        inner.spawn((
            Text2d::new(label),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 0.0, 0.03),
        ));
    }).id();

    commands.entity(parent_entity).add_child(btn_entity);
}

// System to spawn the SAO System Menu
fn setup_system_menu(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let panel_id = spawn_spatial_panel(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(3.5, 2.0, -2.5),
        "SYSTEM MENU",
    );

    // Make it hidden by default
    commands.entity(panel_id).insert((SystemMenuPanel, Visibility::Hidden));

    spawn_holographic_button(
        &mut commands,
        panel_id,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.2, 0.05),
        "Character Sheet",
    );

    spawn_holographic_button(
        &mut commands,
        panel_id,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, -0.2, 0.05),
        "Mastery Logs",
    );

    spawn_holographic_button(
        &mut commands,
        panel_id,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, -0.6, 0.05),
        "Settings",
    );
}

// System to toggle the menu with Tab
fn toggle_system_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Visibility, With<SystemMenuPanel>>,
) {
    if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::Escape) {
        for mut vis in &mut query {
            *vis = match *vis {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

// Action Console for VR gameplay loops
fn setup_action_console(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let panel_id = spawn_spatial_panel(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 1.2, -1.0), // Floating right in front of the player
        "ACTION CONSOLE",
    );

    spawn_holographic_button(&mut commands, panel_id, &mut meshes, &mut materials, Vec3::new(-1.2, -0.2, 0.05), "Construct");
    spawn_holographic_button(&mut commands, panel_id, &mut meshes, &mut materials, Vec3::new(0.0, -0.2, 0.05), "Quest");
    spawn_holographic_button(&mut commands, panel_id, &mut meshes, &mut materials, Vec3::new(1.2, -0.2, 0.05), "Battle");
}

fn handle_spatial_button_pinches(
    pinch_events: Res<crate::hand_tracking::PinchEvents>,
    button_query: Query<(&GlobalTransform, &ButtonAction)>,
    mut next_state: ResMut<NextState<crate::components::GameState>>,
    state: Res<State<crate::components::GameState>>,
    db: Res<crate::database::GameDatabase>,
    curriculum: Res<crate::quest::CurriculumManager>,
    mut commands: Commands,
) {
    for event in &pinch_events.events {
        for (transform, action) in &button_query {
            let button_pos = transform.translation();
            // Distance check for pinch collision (VR physical button press)
            if event.position.distance(button_pos) < 0.4 {
                info!("Pinched button: {}", action.0);
                match action.0.as_str() {
                    "Construct" => {
                        if *state.get() == crate::components::GameState::Playing {
                            next_state.set(crate::components::GameState::Constructing);
                        }
                    }
                    "Quest" => {
                        if *state.get() == crate::components::GameState::Playing {
                            crate::quest::start_quest("Barnaby", &db, &curriculum, &mut commands, &mut next_state);
                        }
                    }
                    "Battle" => {
                        if *state.get() == crate::components::GameState::Playing {
                            crate::battle::start_battle(&mut commands, &db, &mut next_state);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// End of spatial_ui.rs
