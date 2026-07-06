// menu.rs - Main Menu UI
use bevy::prelude::*;
use crate::components::*;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    Continue,
    Settings,
    Difficulty,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
           .add_systems(Update, menu_interaction.run_if(in_state(GameState::MainMenu)))
           .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu);
    }
}

fn spawn_main_menu(
    mut commands: Commands,
) {
    let font_size = 32.0;
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(15.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.9)),
        MainMenuRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("COMMUNICATION CLASS"),
            TextFont { font_size: 50.0, ..default() },
            TextColor(Color::srgb(0.2, 0.8, 0.9)),
            Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() },
        ));

        let button_node = Node {
            width: Val::Px(250.0),
            height: Val::Px(55.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };

        // Play Button
        parent.spawn((
            Button,
            button_node.clone(),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            MenuButtonAction::Play,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("New Game"),
                TextFont { font_size, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // Continue Button
        parent.spawn((
            Button,
            button_node.clone(),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            MenuButtonAction::Continue,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Continue"),
                TextFont { font_size, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // Settings Button
        parent.spawn((
            Button,
            button_node.clone(),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            MenuButtonAction::Settings,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Settings"),
                TextFont { font_size, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // Difficulty Button
        parent.spawn((
            Button,
            button_node.clone(),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            MenuButtonAction::Difficulty,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Difficulty"),
                TextFont { font_size, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn menu_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                match action {
                    MenuButtonAction::Play => {
                        let _ = std::fs::remove_file("save.json");
                        commands.insert_resource(crate::tutorial::TutorialState { step: 0, active: true });
                        next_state.set(GameState::Collecting);
                    }
                    MenuButtonAction::Continue => {
                        next_state.set(GameState::Collecting);
                    }
                    MenuButtonAction::Settings => {
                        info!("Settings menu clicked (not implemented)");
                    }
                    MenuButtonAction::Difficulty => {
                        info!("Difficulty menu clicked (not implemented)");
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}

fn cleanup_main_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenuRoot>>,
) {
    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}
