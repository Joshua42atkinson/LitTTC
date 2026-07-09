// hud.rs - HUD Overlay for Character Sheet, XP, Mastery, and Stash
use bevy::prelude::*;
#[cfg(feature = "flat2d")]
use crate::commands::{GameCommand, log_state_transition};
use crate::components::*;
#[cfg(not(feature = "flat2d"))]
use crate::generated_assets::GeneratedAssets;
use crate::letter::{LetterStash, CurrentSpelling};

#[derive(Component)]
pub struct HudRoot;

/// Marker for the left-side Explore/Talk action buttons.
/// Hidden while in the 2D overworld so the player uses the world instead of menus.
#[derive(Component)]
pub struct ActionMenuPanel;

#[cfg(feature = "flat2d")]
#[derive(Component)]
pub struct ConstructingRoot;

#[cfg(feature = "flat2d")]
#[derive(Component)]
pub struct ConstructingStashText;

#[cfg(feature = "flat2d")]
#[derive(Component)]
pub struct ConstructingSpellingText;

#[cfg(feature = "flat2d")]
#[derive(Component)]
pub struct ConstructingSubmitButton;

#[cfg(feature = "flat2d")]
#[derive(Component)]
pub struct ConstructingBackspaceButton;

#[derive(Component)]
pub struct StatsText;

#[derive(Component)]
pub struct BadgeNode;

#[derive(Component)]
pub struct BadgeText;

#[derive(Component)]
pub struct StashText;

#[derive(Component)]
pub struct SpellingText;

#[derive(Component)]
pub struct HandUiRoot;

#[derive(Component)]
pub struct HandCardUi(pub usize);

#[derive(Component)]
pub struct PlayCardButton;

#[derive(Component)]
pub struct SkipButton;

#[derive(Component)]
pub struct QuestActionButton;

#[derive(Component)]
pub struct BattleActionButton;

#[derive(Component)]
pub struct DeckCounterText;

#[derive(Component)]
pub struct XpProgressBarFill;

#[derive(Component)]
pub struct ActivePetText;

#[derive(Component)]
pub struct PetPortrait;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "flat2d")]
        {
            app.add_systems(Startup, setup_hud)
               .add_systems(Update, (
                   update_stats_ui,
                   update_stash_ui,
                   update_spelling_ui,
                   update_hand_ui,
                   update_deck_counter_ui,
                   update_xp_progress_bar,
                   update_active_pet_ui,
                   update_badge_ui,
                   toggle_action_menu_visibility,
               ));
        }
        #[cfg(not(feature = "flat2d"))]
        {
            app.add_systems(Startup, setup_hud)
               .add_systems(Update, (
                   update_stats_ui,
                   update_stash_ui,
                   update_spelling_ui,
                   update_hand_ui,
                   update_deck_counter_ui,
                   update_xp_progress_bar,
                   update_active_pet_ui,
                   update_badge_ui,
               ));
        }
    }
}

fn setup_hud(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // HUD Root Node (Absolute, full screen)
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        HudRoot,
    ))
    .with_children(|parent| {
        // Left Box: Stats
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
        )).with_children(|stats_parent| {
            stats_parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
            )).with_children(|row| {
                row.spawn((
                    Text::new("Class: Newcomer\nRank: 1\nXP: 0\nWords: 0"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::WHITE),
                    StatsText,
                ));
                row.spawn((
                    Node {
                        width: Val::Px(44.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.8, 0.6, 0.2)),
                    BackgroundColor(Color::srgba(1.0, 0.8, 0.0, 0.8)),
                    BadgeNode,
                )).with_children(|badge| {
                    badge.spawn((
                        Text::new("N"), // Initial badge char
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(Color::BLACK),
                        BadgeText,
                    ));
                });
            });

            // Deck counter
            stats_parent.spawn((
                Text::new("Deck: 0 cards"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                DeckCounterText,
            ));

            // XP Progress Bar label
            stats_parent.spawn((
                Text::new("Ink Progress:"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // XP Progress Bar container
            stats_parent.spawn((
                Node {
                    width: Val::Px(260.0),
                    height: Val::Px(20.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
            )).with_children(|bar| {
                bar.spawn((
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                    XpProgressBarFill,
                ));
            });

            // Active Pet Text
            stats_parent.spawn((
                Text::new("Pet: None"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.4, 0.6, 0.9)),
                ActivePetText,
            ));

            // AI-generated pet portrait (fallback to Barnaby avatar until a pet spawns)
            #[cfg(not(feature = "flat2d"))]
            stats_parent.spawn((
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(120.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.8, 0.6, 0.2)),
                ImageNode::new(_asset_server.load(crate::asset_catalog::BARNABY_AVATAR)),
                PetPortrait,
            ));

            #[cfg(feature = "flat2d")]
            stats_parent.spawn((
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(120.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.8, 0.6, 0.2)),
                BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                PetPortrait,
            ));
        });

        // Right Box: Stash & Controls
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(15.0)),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
        )).with_children(|stash_parent| {
            stash_parent.spawn((
                Text::new("Stash: []"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(0.4, 0.8, 0.4)),
                StashText,
            ));
            stash_parent.spawn((
                Text::new("[P] Pet  [F] Feed  [T] Attune"),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
    });

    // Left Side: Action Menu (Quest / Battle)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Percent(40.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        HudRoot,
        ActionMenuPanel,
    )).with_children(|parent| {
        // Battle Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(180.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.9, 0.4, 0.4)),
            BackgroundColor(Color::srgba(0.3, 0.1, 0.1, 0.9)),
            BattleActionButton,
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Explore (Battle)"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // Quest Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(180.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.4, 0.4, 0.9)),
            BackgroundColor(Color::srgba(0.1, 0.1, 0.3, 0.9)),
            QuestActionButton,
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Talk (Quest)"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });

    // Bottom Center: Controls (Play / Skip buttons)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(180.0),
            left: Val::Percent(50.0),
            width: Val::Px(300.0),
            margin: UiRect::left(Val::Px(-150.0)),
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(20.0),
            ..default()
        },
        HudRoot,
    )).with_children(|parent| {
        // Play Card Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(160.0),
                height: Val::Px(52.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::srgba(0.2, 0.6, 0.2, 0.9)),
            PlayCardButton,
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Play Card"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // Skip Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(160.0),
                height: Val::Px(52.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::srgba(0.6, 0.2, 0.2, 0.9)),
            SkipButton,
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Skip"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });

    // Bottom Center: Spelling text (absolute)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(230.0),
            left: Val::Percent(45.0),
            ..default()
        },
        HudRoot,
    )).with_children(|bottom_parent| {
        bottom_parent.spawn((
            Text::new(""),
            TextFont { font_size: 40.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.2)),
            SpellingText,
        ));
    });

    // Bottom: Cards container
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            column_gap: Val::Px(15.0),
            ..default()
        },
        HudRoot,
        HandUiRoot,
    ));
}

fn update_stats_ui(
    sheet: Res<CharacterSheet>,
    mut query: Query<&mut Text, With<StatsText>>,
) {
    if sheet.is_changed() {
        for mut text in &mut query {
            text.0 = format!("Class: {}\nRank: {}\nXP: {}\nWords: {}", 
                sheet.emergent_class,
                (sheet.total_xp / 1000) + 1,
                sheet.total_xp,
                sheet.words_encountered
            );
        }
    }
}

fn update_badge_ui(
    sheet: Res<CharacterSheet>,
    mut text_query: Query<&mut Text, With<BadgeText>>,
    mut node_query: Query<(&mut BackgroundColor, &mut BorderColor), With<BadgeNode>>,
) {
    if sheet.is_changed() {
        let (initial, bg_color, border_color) = match sheet.emergent_class.as_str() {
            "Newcomer" => ("N", Color::srgba(0.8, 0.8, 0.8, 0.8), Color::WHITE),
            "The Oracle" => ("O", Color::srgba(0.2, 0.8, 1.0, 0.9), Color::srgb(0.0, 0.5, 1.0)),
            "The Bard" => ("B", Color::srgba(1.0, 0.4, 0.8, 0.9), Color::srgb(1.0, 0.2, 0.5)),
            "The Scholar" => ("S", Color::srgba(0.9, 0.9, 0.2, 0.9), Color::srgb(1.0, 0.8, 0.0)),
            _ => (&sheet.emergent_class[0..1], Color::srgba(1.0, 0.8, 0.0, 0.8), Color::WHITE),
        };

        for mut text in &mut text_query {
            text.0 = initial.to_string();
        }
        for (mut bg, mut border) in &mut node_query {
            bg.0 = bg_color;
            *border = BorderColor::all(border_color);
        }
    }
}

fn update_stash_ui(
    stash: Res<LetterStash>,
    mut query: Query<&mut Text, With<StashText>>,
) {
    if stash.is_changed() {
        for mut text in &mut query {
            let stash_str: String = stash.letters.iter().collect();
            text.0 = format!("Stash: [{}]", stash_str);
        }
    }
}

fn update_spelling_ui(
    spelling: Res<CurrentSpelling>,
    mut query: Query<&mut Text, With<SpellingText>>,
) {
    if spelling.is_changed() {
        for mut text in &mut query {
            text.0 = spelling.word.clone();
        }
    }
}

fn update_hand_ui(
    mut commands: Commands,
    hand: Res<crate::components::Hand>,
    root_query: Query<Entity, With<HandUiRoot>>,
    card_query: Query<Entity, With<HandCardUi>>,
    _asset_server: Res<AssetServer>,
) {
    if hand.is_changed() {
        for entity in &card_query {
            commands.entity(entity).despawn();
        }

        if let Some(root) = root_query.iter().next() {
            commands.entity(root).with_children(|parent| {
                for (i, word) in hand.cards.iter().enumerate() {
                    let is_selected = hand.selected == Some(i);
                    let border_color = if is_selected { Color::srgb(1.0, 0.84, 0.0) } else { Color::srgba(0.0, 0.0, 0.0, 0.0) };
                    parent.spawn((
                        Button,
                        HandCardUi(i),
                        DraggableCard { touch_id: None },
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(180.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor::all(border_color),
                        #[cfg(not(feature = "flat2d"))]
                        ImageNode::new(_asset_server.load(crate::asset_catalog::CARD_BACKGROUND)),
                        #[cfg(feature = "flat2d")]
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
                    )).with_children(|card| {
                        card.spawn((
                            Text::new(format!("[{}]\n\n{}", i + 1, word)),
                            TextFont { font_size: 20.0, ..default() },
                            TextColor(Color::WHITE),
                            TextLayout::new_with_justify(Justify::Center),
                        ));
                    });
                }
            });
        }
    }
}

fn update_deck_counter_ui(
    deck: Res<Deck>,
    mut query: Query<&mut Text, With<DeckCounterText>>,
) {
    if deck.is_changed() {
        let deck_name = match deck.active_summon_class {
            Some(SummonClass::SemanticSlime) => "Slime Deck",
            None => "Deck",
        };
        for mut text in &mut query {
            text.0 = format!("{}: {} cards", deck_name, deck.cards.len());
        }
    }
}

fn update_xp_progress_bar(
    time: Res<Time>,
    sheet: Res<CharacterSheet>,
    mut query: Query<&mut Node, With<XpProgressBarFill>>,
) {
    let target_progress = (sheet.total_xp % 1000) as f32 / 1000.0;
    for mut node in &mut query {
        let current_width = match node.width {
            Val::Percent(p) => p / 100.0,
            _ => 0.0,
        };
        
        let dt = time.delta_secs();
        let mut new_progress = current_width + (target_progress - current_width) * 5.0 * dt;
        
        if target_progress < current_width - 0.5 {
            new_progress = target_progress;
        }

        node.width = Val::Percent(new_progress * 100.0);
    }
}

#[cfg(not(feature = "flat2d"))]
fn update_active_pet_ui(
    pet_query: Query<(&PetAvatar, &Element), Changed<PetAvatar>>,
    mut hud_query: Query<&mut Text, With<ActivePetText>>,
    mut portrait_query: Query<&mut ImageNode, With<PetPortrait>>,
    assets: Res<GeneratedAssets>,
    asset_server: Res<AssetServer>,
) {
    if pet_query.is_empty() {
        return;
    }

    let Some((pet, element)) = pet_query.iter().next() else {
        return;
    };

    let lore_line = assets.lore(&pet.word).map_or_else(String::new, |l| {
        let mut extra = String::new();
        if !l.title.is_empty() {
            extra.push_str(&format!("\n{}", l.title));
        }
        if !l.fun_fact.is_empty() {
            extra.push_str(&format!("\n{}", l.fun_fact));
        }
        extra
    });

    for mut text in &mut hud_query {
        text.0 = format!("Pet: {} ({:?}){}", pet.word.to_uppercase(), element, lore_line);
    }

    for mut portrait in &mut portrait_query {
        let path = assets.portrait_path_or_fallback(&pet.word, &format!("{:?}", element));
        portrait.image = asset_server.load(&path);
    }
}

#[cfg(feature = "flat2d")]
fn update_active_pet_ui(
    pet_query: Query<(&PetAvatar, &Element), Changed<PetAvatar>>,
    mut hud_query: Query<&mut Text, With<ActivePetText>>,
    mut portrait_query: Query<&mut BackgroundColor, With<PetPortrait>>,
) {
    if pet_query.is_empty() {
        return;
    }

    let Some((pet, element)) = pet_query.iter().next() else {
        return;
    };

    let lore_line = assets_lore(&pet.word);

    for mut text in &mut hud_query {
        text.0 = format!("Pet: {} ({:?}){}", pet.word.to_uppercase(), element, lore_line);
    }

    for mut portrait in &mut portrait_query {
        portrait.0 = element.color();
    }
}

fn assets_lore(_word: &str) -> String {
    // GeneratedAssets is loaded, but as a resource we can't easily get here without
    // importing it. For flat2d we just return an empty string for now.
    String::new()
}

#[cfg(feature = "flat2d")]
pub fn toggle_hud_visibility(
    state: Res<State<GameState>>,
    mut query: Query<&mut Visibility, With<HudRoot>>,
) {
    let show = matches!(
        state.get(),
        GameState::Playing | GameState::Battling | GameState::Questing | GameState::Reviewing | GameState::Exploring
    );
    for mut vis in &mut query {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
    }
}

#[cfg(feature = "flat2d")]
pub fn toggle_action_menu_visibility(
    state: Res<State<GameState>>,
    mut query: Query<&mut Visibility, With<ActionMenuPanel>>,
) {
    // The old Explore/Talk buttons only make sense in the legacy Playing state or in menus.
    // In the 2D overworld the player walks to objects/NPCs directly.
    let show = matches!(
        state.get(),
        GameState::Playing | GameState::Battling | GameState::Questing | GameState::Reviewing
    );
    for mut vis in &mut query {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
    }
}

#[derive(Component)]
pub struct ReviewUiPanel;

#[derive(Component)]
pub struct ReviewUiText;

#[cfg(feature = "xr")]
pub fn spawn_review_ui_xr(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spellbook: Res<SpellBook>,
) {
    let panel_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.02, 0.08, 0.05, 0.95),
        emissive: Color::srgba(0.02, 0.15, 0.05, 1.0).to_srgba().into(),
        metallic: 0.1,
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let panel = commands.spawn((
        ReviewUiPanel,
        Mesh3d(meshes.add(Cuboid::new(3.0, 1.5, 0.05))),
        MeshMaterial3d(panel_mat),
        Transform::from_xyz(0.0, 2.0, -1.8),
    )).id();

    let mut mastered_text = "Mastered Words:\n".to_string();
    let mut count = 0;
    for entry in spellbook.entries.iter() {
        if count < 5 {
            mastered_text.push_str(&format!("- {}: {:?}\n", entry.word, entry.mastery));
            count += 1;
        }
    }
    if count == 0 {
        mastered_text.push_str("(No words registered in SpellBook yet)\n");
    }

    let text_entity = commands.spawn((
        ReviewUiText,
        Text2d::new(format!("VICTORY & REVIEW\n\n{}\nPress ENTER to continue exploration", mastered_text)),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.0, 0.03),
    )).id();

    commands.entity(panel).add_child(text_entity);
}

#[cfg(feature = "xr")]
pub fn cleanup_review_ui_xr(
    mut commands: Commands,
    query: Query<Entity, Or<(With<ReviewUiPanel>, With<ReviewUiText>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(not(feature = "xr"))]
pub fn spawn_review_ui_2d(
    mut commands: Commands,
    spellbook: Res<SpellBook>,
) {
    let mut mastered_text = "Mastered Words:\n".to_string();
    let mut count = 0;
    for entry in spellbook.entries.iter() {
        if count < 5 {
            mastered_text.push_str(&format!("- {}: {:?}\n", entry.word, entry.mastery));
            count += 1;
        }
    }
    if count == 0 {
        mastered_text.push_str("(No words registered in SpellBook yet)\n");
    }

    commands.spawn((
        ReviewUiPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(20.0),
            left: Val::Percent(30.0),
            width: Val::Percent(40.0),
            height: Val::Percent(60.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(30.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.15, 0.1, 0.95)),
    )).with_children(|parent| {
        parent.spawn((
            ReviewUiText,
            Text::new(format!("VICTORY & REVIEW\n\n{}\n\n[Press ENTER to continue]", mastered_text)),
            TextFont { font_size: 28.0, ..default() },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
        ));
    });
}

#[cfg(not(feature = "xr"))]
pub fn cleanup_review_ui_2d(
    mut commands: Commands,
    query: Query<Entity, With<ReviewUiPanel>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(feature = "flat2d")]
pub fn fill_letter_stash_and_start_constructing(
    mut stash: ResMut<crate::letter::LetterStash>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    stash.letters.clear();
    for _ in 0..10 {
        stash.letters.push((rng.gen_range(0u8..26) + b'A') as char);
    }
    log_state_transition(&GameState::Collecting, GameState::Constructing);
    next_state.set(GameState::Constructing);
}

#[cfg(feature = "flat2d")]
pub fn spawn_constructing_ui(mut commands: Commands) {
    commands.spawn((
        ConstructingRoot,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(15.0),
            left: Val::Percent(15.0),
            width: Val::Percent(70.0),
            height: Val::Percent(70.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(24.0),
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.95)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Construct a Word"),
            TextFont { font_size: 42.0, ..default() },
            TextColor(Color::srgb(0.9, 0.8, 0.3)),
        ));

        parent.spawn((
            ConstructingStashText,
            Text::new("Stash: []"),
            TextFont { font_size: 26.0, ..default() },
            TextColor(Color::srgb(0.4, 0.8, 0.4)),
        ));

        parent.spawn((
            ConstructingSpellingText,
            Text::new("_"),
            TextFont { font_size: 56.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.2)),
        ));

        parent.spawn((
            Text::new("Type A-Z, Enter to submit, Backspace to undo"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));

        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                ..default()
            },
        )).with_children(|row| {
            row.spawn((
                Button,
                ConstructingSubmitButton,
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::srgb(0.8, 0.6, 0.2)),
                BackgroundColor(Color::srgba(0.2, 0.5, 0.2, 0.9)),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new("Submit"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });

            row.spawn((
                Button,
                ConstructingBackspaceButton,
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::srgb(0.6, 0.2, 0.2)),
                BackgroundColor(Color::srgba(0.4, 0.15, 0.15, 0.9)),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new("Backspace"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

#[cfg(feature = "flat2d")]
pub fn despawn_constructing_ui(
    mut commands: Commands,
    root_query: Query<Entity, With<ConstructingRoot>>,
    children_query: Query<&Children>,
) {
    fn despawn_node(cmds: &mut Commands, entity: Entity, children_query: &Query<&Children>) {
        if let Ok(kids) = children_query.get(entity) {
            for child in kids.iter() {
                despawn_node(cmds, child, children_query);
            }
        }
        cmds.entity(entity).despawn();
    }

    for root in &root_query {
        despawn_node(&mut commands, root, &children_query);
    }
}

#[cfg(feature = "flat2d")]
pub fn update_constructing_ui(
    stash: Res<crate::letter::LetterStash>,
    spelling: Res<crate::letter::CurrentSpelling>,
    mut query: Query<(&mut Text, Option<&ConstructingStashText>, Option<&ConstructingSpellingText>)>,
) {
    let stash_str: String = stash.letters.iter().collect();
    let display = if spelling.word.is_empty() {
        "_".to_string()
    } else {
        spelling.word.clone()
    };

    for (mut text, stash_marker, spelling_marker) in &mut query {
        if stash_marker.is_some() {
            text.0 = format!("Stash: [{}]", stash_str);
        }
        if spelling_marker.is_some() {
            text.0 = display.clone();
        }
    }
}

#[cfg(feature = "flat2d")]
pub fn handle_constructing_buttons(
    mut writer: MessageWriter<GameCommand>,
    mut submit_query: Query<&Interaction, With<ConstructingSubmitButton>>,
    mut backspace_query: Query<&Interaction, With<ConstructingBackspaceButton>>,
) {
    for interaction in &mut submit_query {
        if *interaction == Interaction::Pressed {
            writer.write(GameCommand::SubmitSpelling);
        }
    }
    for interaction in &mut backspace_query {
        if *interaction == Interaction::Pressed {
            writer.write(GameCommand::Backspace);
        }
    }
}

pub fn review_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<crate::commands::GameCommand>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        writer.write(crate::commands::GameCommand::DismissReview);
    }
}

