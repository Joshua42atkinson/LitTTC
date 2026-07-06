// paywall.rs - Demo mode restrictions and Paywall UI
use bevy::prelude::*;
use crate::components::*;

#[derive(Resource)]
pub struct DemoSettings {
    pub is_demo: bool,
    pub max_words: usize,
}

impl Default for DemoSettings {
    fn default() -> Self {
        Self {
            is_demo: true,
            max_words: 10,
        }
    }
}

pub struct PaywallPlugin;

impl Plugin for PaywallPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoSettings>()
           .add_systems(OnEnter(GameState::Paywall), spawn_paywall_ui)
           .add_systems(Update, paywall_interaction.run_if(in_state(GameState::Paywall)))
           .add_systems(OnExit(GameState::Paywall), cleanup_paywall_ui);
    }
}

#[derive(Component)]
pub struct PaywallUiRoot;

fn spawn_paywall_ui(mut commands: Commands) {
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
        PaywallUiRoot,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("DEMO LIMIT REACHED"),
            TextFont { font_size: 50.0, ..default() },
            TextColor(Color::srgb(1.0, 0.3, 0.3)),
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));
        parent.spawn((
            Text::new("You have spelled 10 words and reached the end of the free demo.\nTo unlock unlimited gameplay, quests, and saving, please purchase the full version!"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() },
        ));

        // Main Menu Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Return to Main Menu"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn paywall_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::MainMenu);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }
}

fn cleanup_paywall_ui(
    mut commands: Commands,
    query: Query<Entity, With<PaywallUiRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
