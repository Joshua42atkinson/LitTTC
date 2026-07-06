// deck.rs — Deck, Hand, and Discard mechanics using String word identifiers
use bevy::prelude::*;
use crate::components::*;

pub fn draw_cards(
    mut deck: ResMut<Deck>,
    mut hand: ResMut<Hand>,
    mut spellbook: ResMut<SpellBook>,
    mut sheet: ResMut<CharacterSheet>,
    mut trail: ResMut<StudentTrail>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    if *state.get() != GameState::Playing && *state.get() != GameState::Battling {
        return;
    }

    // Refill hand up to max size
    while hand.cards.len() < hand.max_size {
        if let Some(word) = deck.cards.pop() {
            hand.cards.push(word.clone());

            // Record encounter in SpellBook & CharacterSheet
            // For now, we attune to Mind by default, or look it up
            sheet.engage_channel(&Channel::Mind);
            sheet.words_encountered += 1;
            spellbook.record_encounter(&word, Channel::Mind);

            if !trail.visited_words.contains(&word) {
                trail.visited_words.push(word);
            }
        } else {
            break;
        }
    }

    if hand.cards.is_empty() {
        next_state.set(GameState::Reviewing);
    }
}

pub fn select_card_by_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut hand: ResMut<Hand>,
) {
    let key_map = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
    ];

    for (key, index) in key_map {
        if keys.just_pressed(key) && index < hand.cards.len() {
            hand.selected = Some(index);
            return;
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        hand.selected = None;
    }
}
