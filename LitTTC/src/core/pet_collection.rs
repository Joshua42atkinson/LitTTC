// pet_collection.rs — Pet collection gallery screen
use bevy::prelude::*;
use crate::components::{GameState, SpellBook, SpellBookEntry};
use crate::generated_assets::GeneratedAssets;

pub struct PetCollectionPlugin;

#[derive(Component)]
pub struct PetCollectionUiRoot;

#[derive(Component)]
struct SortButton(SortKey);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SortKey {
    #[default]
    Word,
    Element,
    Mastery,
}

#[derive(Component)]
struct PetGridItem(usize); // index into sorted entries

#[derive(Component)]
struct DetailPanel;

#[derive(Component)]
struct CompanionButton;

#[derive(Resource, Default, Debug, Clone, Copy)]
struct CollectionState {
    sort: SortKey,
    selected: Option<usize>, // original spellbook index
}

impl Plugin for PetCollectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollectionState>()
           .add_systems(OnEnter(GameState::PetCollection), spawn_pet_collection_ui)
           .add_systems(Update, pet_collection_interaction.run_if(in_state(GameState::PetCollection)))
           .add_systems(OnExit(GameState::PetCollection), cleanup_pet_collection_ui);
    }
}

fn spawn_pet_collection_ui(
    mut commands: Commands,
    spellbook: Res<SpellBook>,
    state: Res<CollectionState>,
    assets: Res<GeneratedAssets>,
) {
    let sorted = sorted_indices(&spellbook.entries, state.sort);

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.04, 0.08, 1.0)),
        PetCollectionUiRoot,
    )).with_children(|parent| {
        // Header
        parent.spawn((
            Text::new("PET COLLECTION"),
            TextFont { font_size: 50.0, ..default() },
            TextColor(Color::srgb(0.8, 0.7, 1.0)),
            Node { margin: UiRect::all(Val::Px(20.0)), ..default() },
        ));

        // Sort buttons
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        )).with_children(|row| {
            for (label, key) in [
                ("Word", SortKey::Word),
                ("Element", SortKey::Element),
                ("Mastery", SortKey::Mastery),
            ] {
                row.spawn((
                    Button,
                    SortButton(key),
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(44.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::horizontal(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                )).with_children(|p| {
                    p.spawn((
                        Text::new(label),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });

        // Summary
        parent.spawn((
            Text::new(format!("Words collected: {}", spellbook.entries.len())),
            TextFont { font_size: 22.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.8)),
            Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
        ));

        // Main content: grid + detail side by side
        parent.spawn((
            Node {
                width: Val::Percent(90.0),
                height: Val::Px(480.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        )).with_children(|content| {
            // Scrollable grid placeholder — uses a wrapping row of buttons
            content.spawn((
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: bevy::ui::FlexWrap::Wrap,
                    overflow: bevy::ui::Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 1.0)),
            )).with_children(|grid| {
                for original_idx in &sorted {
                    let entry = &spellbook.entries[*original_idx];
                    let label = pet_label(entry);
                    grid.spawn((
                        Button,
                        PetGridItem(*original_idx),
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(if state.selected == Some(*original_idx) {
                            Color::srgb(0.3, 0.25, 0.5)
                        } else {
                            Color::srgb(0.2, 0.2, 0.25)
                        }),
                    )).with_children(|p| {
                        p.spawn((
                            Text::new(label),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });

            // Detail panel
            let detail_text = if let Some(idx) = state.selected {
                detail_text(&spellbook.entries[idx], &assets)
            } else {
                "Select a pet to view details.".to_string()
            };
            content.spawn((
                DetailPanel,
                Node {
                    width: Val::Percent(38.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.12, 0.18, 1.0)),
            )).with_children(|panel| {
                panel.spawn((
                    Text::new(detail_text),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                    Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
                ));
                panel.spawn((
                    Button,
                    CompanionButton,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(44.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.5, 0.3)),
                )).with_children(|p| {
                    p.spawn((
                        Text::new("Set Companion"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });

        // Main Menu
        parent.spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        )).with_children(|p| {
            p.spawn((
                Text::new("Main Menu"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn pet_label(entry: &SpellBookEntry) -> String {
    let element = entry.element.map(|e| format!("{:?}", e)).unwrap_or_else(|| "?".to_string());
    let role = entry.role.map(|r| format!("{:?}", r)).unwrap_or_else(|| "?".to_string());
    format!("{}\n{} | {}", entry.word, element, role)
}

fn detail_text(entry: &SpellBookEntry, assets: &GeneratedAssets) -> String {
    let element = entry.element.map(|e| format!("{:?}", e)).unwrap_or_else(|| "Unknown".to_string());
    let role = entry.role.map(|r| format!("{:?}", r)).unwrap_or_else(|| "Unknown".to_string());
    let companion_line = if entry.companion { "\n⭐ Companion" } else { "" };
    let lore_line = assets.lore(&entry.word).map_or_else(String::new, |l| {
        let mut s = String::new();
        if !l.title.is_empty() {
            s.push_str(&format!("\nTitle: {}", l.title));
        }
        if !l.habitat.is_empty() {
            s.push_str(&format!("\nHome: {}", l.habitat));
        }
        if !l.fun_fact.is_empty() {
            s.push_str(&format!("\nQuirk: {}", l.fun_fact));
        }
        s
    });

    if let Some(stats) = entry.stats {
        format!(
            "{}\nElement: {}\nRole: {}\nMastery: {:?}\nTimes: {}\nLogos: {:.1}\nPathos: {:.1}\nEthos: {:.1}\nSpeed: {:.1}{}{}",
            entry.word, element, role, entry.mastery, entry.times_encountered,
            stats.logos, stats.pathos, stats.ethos, stats.speed, companion_line, lore_line
        )
    } else {
        format!(
            "{}\nElement: {}\nRole: {}\nMastery: {:?}\nTimes: {}{}{}",
            entry.word, element, role, entry.mastery, entry.times_encountered, companion_line, lore_line
        )
    }
}

fn sorted_indices(entries: &[SpellBookEntry], key: SortKey) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..entries.len()).collect();
    match key {
        SortKey::Word => indices.sort_by(|a, b| entries[*a].word.cmp(&entries[*b].word)),
        SortKey::Element => indices.sort_by(|a, b| {
            let ea = entries[*a].element.map(|e| format!("{:?}", e)).unwrap_or_default();
            let eb = entries[*b].element.map(|e| format!("{:?}", e)).unwrap_or_default();
            ea.cmp(&eb).then_with(|| entries[*a].word.cmp(&entries[*b].word))
        }),
        SortKey::Mastery => indices.sort_by(|a, b| {
            (entries[*a].mastery as u32).cmp(&(entries[*b].mastery as u32))
                .then_with(|| entries[*a].word.cmp(&entries[*b].word))
        }),
    }
    indices
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn pet_collection_interaction(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut state: ResMut<CollectionState>,
    mut spellbook: ResMut<SpellBook>,
    mut interaction_query: ParamSet<(
        Query<(&Interaction, &SortButton, &mut BackgroundColor), Changed<Interaction>>,
        Query<(&Interaction, &PetGridItem, &mut BackgroundColor), Changed<Interaction>>,
        Query<&Interaction, (Changed<Interaction>, With<CompanionButton>, Without<SortButton>, Without<PetGridItem>)>,
        Query<&Interaction, (Changed<Interaction>, With<Button>, Without<SortButton>, Without<PetGridItem>, Without<CompanionButton>)>,
    )>,
    roots: Query<Entity, With<PetCollectionUiRoot>>,
) {
    let mut changed = false;

    for (interaction, sort, mut color) in interaction_query.p0().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                state.sort = sort.0;
                changed = true;
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.45)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
        }
    }

    for (interaction, item, mut color) in interaction_query.p1().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                state.selected = Some(item.0);
                changed = true;
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.35, 0.3, 0.55)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
        }
    }

    if changed {
        for entity in roots.iter() {
            commands.entity(entity).despawn();
        }
        return; // OnExit will not run; UI is rebuilt next frame by the spawner
    }

    for interaction in interaction_query.p2().iter() {
        if *interaction == Interaction::Pressed {
            if let Some(idx) = state.selected {
                for (i, entry) in spellbook.entries.iter_mut().enumerate() {
                    entry.companion = i == idx;
                }
                info!("Companion set to {}", spellbook.entries[idx].word);
            }
        }
    }

    for interaction in interaction_query.p3().iter() {
        if *interaction == Interaction::Pressed {
            crate::commands::log_state_transition(&GameState::PetCollection, GameState::MainMenu);
            next_state.set(GameState::MainMenu);
        }
    }
}

fn cleanup_pet_collection_ui(mut commands: Commands, query: Query<Entity, With<PetCollectionUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
