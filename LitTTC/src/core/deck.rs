// deck.rs — Deck, Hand, and Discard mechanics using String word identifiers
use bevy::prelude::*;
use crate::components::*;
use crate::database::GameDatabase;

/// Part of speech for Phase 1 micro-deck words
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
}

/// Micro-deck word for Phase 1 gray-box combat
#[derive(Clone, Debug)]
pub struct MicroDeckWord {
    pub word: String,
    pub part_of_speech: PartOfSpeech,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
}

/// Initialize the micro-deck for Phase 1 gray-box combat
pub fn initialize_micro_deck() -> Vec<MicroDeckWord> {
    vec![
        // Nouns (10)
        MicroDeckWord {
            word: "wolf".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["beast".to_string(), "predator".to_string()],
            antonyms: vec!["prey".to_string()],
        },
        MicroDeckWord {
            word: "wall".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["barrier".to_string(), "fortress".to_string()],
            antonyms: vec!["opening".to_string()],
        },
        MicroDeckWord {
            word: "sword".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["blade".to_string(), "weapon".to_string()],
            antonyms: vec!["shield".to_string()],
        },
        MicroDeckWord {
            word: "shield".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["armor".to_string(), "protection".to_string()],
            antonyms: vec!["weapon".to_string()],
        },
        MicroDeckWord {
            word: "dragon".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["beast".to_string(), "monster".to_string()],
            antonyms: vec![],
        },
        MicroDeckWord {
            word: "castle".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["fortress".to_string(), "stronghold".to_string()],
            antonyms: vec![],
        },
        MicroDeckWord {
            word: "river".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["stream".to_string(), "waterway".to_string()],
            antonyms: vec![],
        },
        MicroDeckWord {
            word: "mountain".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["peak".to_string(), "summit".to_string()],
            antonyms: vec!["valley".to_string()],
        },
        MicroDeckWord {
            word: "forest".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["woods".to_string(), "jungle".to_string()],
            antonyms: vec![],
        },
        MicroDeckWord {
            word: "star".to_string(),
            part_of_speech: PartOfSpeech::Noun,
            synonyms: vec!["sun".to_string(), "celestial".to_string()],
            antonyms: vec![],
        },
        // Verbs (5)
        MicroDeckWord {
            word: "strikes".to_string(),
            part_of_speech: PartOfSpeech::Verb,
            synonyms: vec!["attacks".to_string(), "hits".to_string()],
            antonyms: vec!["defends".to_string()],
        },
        MicroDeckWord {
            word: "defends".to_string(),
            part_of_speech: PartOfSpeech::Verb,
            synonyms: vec!["protects".to_string(), "guards".to_string()],
            antonyms: vec!["attacks".to_string()],
        },
        MicroDeckWord {
            word: "heals".to_string(),
            part_of_speech: PartOfSpeech::Verb,
            synonyms: vec!["cures".to_string(), "restores".to_string()],
            antonyms: vec!["wounds".to_string()],
        },
        MicroDeckWord {
            word: "burns".to_string(),
            part_of_speech: PartOfSpeech::Verb,
            synonyms: vec!["scorches".to_string(), "ignites".to_string()],
            antonyms: vec!["freezes".to_string()],
        },
        MicroDeckWord {
            word: "freezes".to_string(),
            part_of_speech: PartOfSpeech::Verb,
            synonyms: vec!["chills".to_string(), "ices".to_string()],
            antonyms: vec!["burns".to_string()],
        },
        // Adjectives (5)
        MicroDeckWord {
            word: "searing".to_string(),
            part_of_speech: PartOfSpeech::Adjective,
            synonyms: vec!["scorching".to_string(), "blazing".to_string()],
            antonyms: vec!["freezing".to_string()],
        },
        MicroDeckWord {
            word: "impregnable".to_string(),
            part_of_speech: PartOfSpeech::Adjective,
            synonyms: vec!["unbreakable".to_string(), "invincible".to_string()],
            antonyms: vec!["weak".to_string()],
        },
        MicroDeckWord {
            word: "swift".to_string(),
            part_of_speech: PartOfSpeech::Adjective,
            synonyms: vec!["fast".to_string(), "quick".to_string()],
            antonyms: vec!["slow".to_string()],
        },
        MicroDeckWord {
            word: "ancient".to_string(),
            part_of_speech: PartOfSpeech::Adjective,
            synonyms: vec!["old".to_string(), "aged".to_string()],
            antonyms: vec!["modern".to_string()],
        },
        MicroDeckWord {
            word: "radiant".to_string(),
            part_of_speech: PartOfSpeech::Adjective,
            synonyms: vec!["bright".to_string(), "glowing".to_string()],
            antonyms: vec!["dark".to_string()],
        },
    ]
}

/// Get micro-deck words as strings for UI
pub fn get_micro_deck_words() -> Vec<String> {
    initialize_micro_deck().iter().map(|w| w.word.clone()).collect()
}

/// Refills the player's hand from the deck and records encounters in the spellbook.
pub fn draw_cards(
    mut deck: ResMut<Deck>,
    mut hand: ResMut<Hand>,
    mut spellbook: ResMut<SpellBook>,
    mut sheet: ResMut<CharacterSheet>,
    mut trail: ResMut<WordTrail>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    let allowed_states = [
        GameState::Playing,
        GameState::Battling,
        GameState::Exploring,
    ];
    if !allowed_states.contains(state.get()) {
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
            spellbook.record_encounter(&word, Channel::Mind, None, None, None);

            if !trail.visited_words.contains(&word) {
                trail.visited_words.push(word);
            }
        } else {
            break;
        }
    }

    if hand.cards.is_empty() && deck.cards.is_empty() {
        crate::commands::log_state_transition(state.get(), GameState::Collecting);
        next_state.set(GameState::Collecting);
    }
}

/// Selects or deselects a hand card using number keys (1-5) or Escape.
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

/// Initializes the player's deck from the save file or rank lexicon on first enter.
pub fn initialize_player_deck(
    db: Res<GameDatabase>,
    grade_manager: Res<crate::quest::GradeManager>,
    mut deck: ResMut<Deck>,
    mut spellbook: ResMut<SpellBook>,
    mut stash: ResMut<crate::letter::LetterStash>,
    mut sheet: ResMut<CharacterSheet>,
    mut trail: ResMut<WordTrail>,
) {
    info!("Shuffling deck from rank lexicon...");

    if let Ok(data) = crate::save::load_game() {
        *sheet = data.character_sheet;
        *spellbook = data.spellbook;
        *trail = data.word_trail;
        for entry in &spellbook.entries {
            deck.cards.push(entry.word.clone());
        }
        info!("Loaded chronicle!");
    } else {
        let valid_grades = grade_manager.get_valid_grade_levels();
        let mut pool: Vec<String> = db.words.iter()
            .filter(|(_, stats): &(&String, &crate::database::WordStats)| valid_grades.contains(&stats.grade_level.as_str()))
            .map(|(word, _): (&String, &crate::database::WordStats)| word.clone())
            .collect();
        pool.sort();

        if !pool.is_empty() {
            for word in pool.iter().take(15) {
                deck.cards.push(word.clone());
                spellbook.record_encounter(word, Channel::Mind, None, None, None);
            }
        } else {
            let default_words = ["abandoned", "abc", "ability", "patience", "clarity", "courage", "wisdom", "strength"];
            for &word in &default_words {
                deck.cards.push(word.to_string());
                spellbook.record_encounter(word, Channel::Mind, None, None, None);
            }
        }
    }

    stash.letters.extend("PATIENCECLARITYCOURAGEWISDOMSTRENGTH".chars());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_defaults_are_empty_and_unselected() {
        let hand = Hand::default();
        assert!(hand.cards.is_empty());
        assert_eq!(hand.max_size, 3);
        assert!(hand.selected.is_none());
    }

    #[test]
    fn deck_defaults_with_empty_cards() {
        let deck = Deck::default();
        assert!(deck.cards.is_empty());
        assert!(deck.active_summon_class.is_none());
    }
}
