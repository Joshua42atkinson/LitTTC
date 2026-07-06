// tutorial.rs - 3 Step Onboarding
use bevy::prelude::*;
use crate::components::*;

#[derive(Resource, Default)]
pub struct TutorialState {
    pub step: usize,
    pub active: bool,
}

#[derive(Component)]
pub struct TutorialUi;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>();
        
        #[cfg(not(feature = "xr"))]
        app.add_systems(Update, update_tutorial_2d.run_if(in_state(GameState::Collecting)));
        
        #[cfg(feature = "xr")]
        app.add_systems(Update, update_tutorial_xr.run_if(in_state(GameState::Collecting)));
    }
}

#[cfg(not(feature = "xr"))]
fn update_tutorial_2d(
    mut commands: Commands,
    mut state: ResMut<TutorialState>,
    pinch_events: Res<crate::hand_tracking::PinchEvents>, // Simulates mouse clicks on desktop
    query: Query<Entity, With<TutorialUi>>,
) {
    if !state.active {
        return;
    }

    if !pinch_events.events.is_empty() {
        state.step += 1;
        for e in &query {
            commands.entity(e).despawn();
        }
    }

    if query.is_empty() && state.step < 4 {
        let text = match state.step {
            0 => "Welcome to Communication Class!\nLook around the VR environment.",
            1 => "Reach out and Pinch the blue crystals\nto collect letters for your Stash.",
            2 => "Pinch the 'Construct' floating button\nto spell a word with your physical blocks.",
            3 => "Pinch 'Quest' or 'Battle' to use your\nwords in action! (Pinch the air to close)",
            _ => "",
        };

        commands.spawn((
            TutorialUi,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-250.0)),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.4, 0.8, 0.8)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new(format!("Tutorial Step {}/4\n\n{}\n\n[Pinch the air to continue]", state.step + 1, text)),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    } else if state.step >= 4 {
        state.active = false;
        for e in &query {
            commands.entity(e).despawn();
        }
    }
}

#[cfg(feature = "xr")]
fn update_tutorial_xr(
    mut commands: Commands,
    mut state: ResMut<TutorialState>,
    pinch_events: Res<crate::hand_tracking::PinchEvents>,
    query: Query<Entity, With<TutorialUi>>,
    button_query: Query<(&GlobalTransform, &crate::spatial_ui::ButtonAction)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !state.active {
        return;
    }

    let mut next_clicked = false;
    for event in &pinch_events.events {
        for (transform, action) in &button_query {
            if action.0 == "Next" && event.position.distance(transform.translation()) < 0.4 {
                next_clicked = true;
            }
        }
    }

    if next_clicked {
        state.step += 1;
        for e in &query {
            commands.entity(e).despawn();
        }
    }

    if query.is_empty() && state.step < 4 {
        let text = match state.step {
            0 => "Welcome to Communication Class!\nLook around the VR environment.",
            1 => "Reach out and Pinch the blue crystals\nto collect letters for your Stash.",
            2 => "Pinch the 'Construct' floating button\nto spell a word with your blocks.",
            3 => "Pinch 'Quest' or 'Battle' to use your\nwords in action!",
            _ => "",
        };

        let panel_id = crate::spatial_ui::spawn_spatial_panel(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(0.0, 1.5, -2.0),
            &format!("Tutorial Step {}/4", state.step + 1),
        );

        commands.entity(panel_id).insert(TutorialUi);

        commands.entity(panel_id).with_children(|parent| {
            parent.spawn((
                Text2d::new(text),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, -0.1, 0.03),
            ));
        });

        crate::spatial_ui::spawn_holographic_button(
            &mut commands,
            panel_id,
            &mut meshes,
            &mut materials,
            Vec3::new(0.0, -0.7, 0.05),
            "Next",
        );
    } else if state.step >= 4 {
        state.active = false;
        for e in &query {
            commands.entity(e).despawn();
        }
    }
}
