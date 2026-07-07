// commands.rs — GameCommand message bridge: input decoupled from game logic
use bevy::prelude::*;
use crate::components::{GameState, SwipeChoice};

/// A player-intent message fired by input systems and consumed by command handlers.
///
/// This enum decouples raw input (mouse clicks, keyboard, VR pinches, swipes)
/// from the game systems that mutate state. Every variant captures the data
/// needed to perform a high-level action without knowing which input device
/// triggered it.
///
/// Variants are intentionally allowed as dead code during Phase 1.1; they will
/// be wired to input systems in P1.2 and P1.3.
#[allow(dead_code)]
#[derive(Message, Clone, Debug, PartialEq)]
pub enum GameCommand {
    // ─── SPELLING / WORD CONSTRUCTION ─────────────────────────────────
    /// Submit the word currently being typed in the spelling pad.
    SubmitSpelling,
    /// Add a single letter to the current spelling.
    AddLetter(char),
    /// Remove the last letter from the current spelling.
    Backspace,
    /// Clear the entire current spelling.
    ClearSpelling,

    // ─── HAND / CARD INTERACTIONS ───────────────────────────────────────
    /// Select the card at the given index in the player's hand.
    SelectCard(usize),
    /// Play the currently selected card (state-dependent interpretation).
    PlayCard,
    /// Skip / cancel the current action (state-dependent interpretation).
    SkipCard,

    // ─── BATTLE ─────────────────────────────────────────────────────────
    /// Start a battle against a wild Typo.
    StartBattle,
    /// Play the card at the given index as a battle attack.
    PlayBattleCard(usize),
    /// Retreat from the current battle.
    FleeBattle,

    // ─── QUEST ──────────────────────────────────────────────────────────
    /// Start a quest from the named NPC.
    StartQuest(String),
    /// Fill the next empty quest slot with the card at the given index.
    FillQuestSlot(usize),
    /// Mark the current quest as complete.
    CompleteQuest,
    /// Cancel the active quest.
    CancelQuest,

    // ─── SWIPE / DIALOGUE ──────────────────────────────────────────────
    /// Commit to a swipe choice in a dialogue encounter.
    Swipe(SwipeChoice),

    // ─── REVIEW ─────────────────────────────────────────────────────────
    /// Dismiss the post-battle review screen and return to exploration.
    DismissReview,

    // ─── MENU ───────────────────────────────────────────────────────────
    /// Start a new game (clear save, begin tutorial, go to Collecting).
    NewGame,
    /// Continue from an existing save file.
    ContinueGame,
    /// Open the settings screen.
    OpenSettings,

    // ─── DIRECT STATE (used sparingly for transitions without extra data) ─
    /// Transition to a specific game state. Prefer semantic variants above.
    TransitionTo(GameState),
}

/// Resource that tracks the most recently fired command for debugging and replay.
#[allow(dead_code)]
#[derive(Resource, Default, Debug, Clone)]
pub struct LastCommand(pub Option<GameCommand>);

/// Logs a game state transition in a consistent format across systems.
pub fn log_state_transition(current: &GameState, next: GameState) {
    info!("State transition: {:?} -> {:?}", current, next);
}

/// Central command handler: turns high-level player-intent messages into game state changes.
///
/// Input systems should remain thin after Phase 1.3 — they only send `GameCommand` messages.
/// This system is the single place where those messages are interpreted and executed.
///
/// Resources are grouped into tuples to stay within Bevy's system-parameter limit.
pub fn handle_game_commands(
    mut commands: Commands,
    mut messages: MessageReader<GameCommand>,
    mut last: ResMut<LastCommand>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sheet: ResMut<crate::components::CharacterSheet>,
    mut trail: ResMut<crate::components::StudentTrail>,
    mut session_battle: Option<ResMut<crate::battle::BattleSession>>,
    mut session_quest: Option<ResMut<crate::quest::QuestSession>>,
    card_resources: (ResMut<crate::components::Hand>, ResMut<crate::components::SpellBook>),
    combat_resources: (
        Res<crate::database::GameDatabase>,
        ResMut<crate::quest::CurriculumManager>,
        Res<Time>,
        Option<Res<AssetServer>>,
        ResMut<crate::chat::ChatLog>,
    ),
    spelling_resources: (
        ResMut<crate::letter::CurrentSpelling>,
        ResMut<crate::letter::LetterStash>,
        Option<ResMut<Assets<Mesh>>>,
        Option<ResMut<Assets<StandardMaterial>>>,
        Res<crate::paywall::DemoSettings>,
    ),
) {
    let (mut hand, mut spellbook) = card_resources;
    let (db, mut curriculum, time, asset_server, mut chat_log) = combat_resources;
    let (mut current_spelling, mut letter_stash, mut meshes, mut materials, demo) = spelling_resources;

    for msg in messages.read() {
        last.0 = Some(msg.clone());
        match msg {
            GameCommand::SelectCard(idx) => {
                if *idx < hand.cards.len() {
                    hand.selected = Some(*idx);
                    info!("Selected card index: {}", idx);
                } else {
                    warn!("SelectCard index {} out of bounds (hand has {} cards)", idx, hand.cards.len());
                }
            }

            GameCommand::StartBattle => {
                if *state.get() == GameState::Playing {
                    crate::battle::start_battle(&mut commands, &db, &curriculum, &mut next_state, &state);
                }
            }

            GameCommand::StartQuest(npc) => {
                if *state.get() == GameState::Playing {
                    crate::quest::start_quest(npc, &db, &curriculum, &mut commands, &mut next_state, &state);
                }
            }

            GameCommand::PlayCard => {
                match *state.get() {
                    GameState::Playing => {
                        if hand.selected.is_some() {
                            crate::battle::start_battle(&mut commands, &db, &curriculum, &mut next_state, &state);
                        } else {
                            warn!("Select a card first!");
                        }
                    }
                    GameState::Battling => {
                        if let Some(ref mut session) = session_battle {
                            if let Some(idx) = hand.selected {
                                if idx < hand.cards.len() {
                                    let played_word = hand.cards.remove(idx);
                                    let typo_word = session.typo_word.clone();
                                    let result = crate::battle::play_battle_card(
                                        &played_word,
                                        session,
                                        &db,
                                        &mut spellbook,
                                        &mut next_state,
                                        &sheet,
                                        &state,
                                    );
                                    if result.is_effective {
                                        commands.spawn(crate::battle::CriticalHitTrigger);
                                    }
                                    if result.social_combat_triggered {
                                        if let Some(ref asset_server) = asset_server {
                                            crate::chat::trigger_social_combat(
                                                &played_word,
                                                &typo_word,
                                                result.is_synonym_logic,
                                                time.elapsed_secs(),
                                                &mut chat_log,
                                                &mut commands,
                                                asset_server,
                                            );
                                        }
                                    }
                                    hand.selected = None;
                                }
                            } else {
                                warn!("Select a card first to play!");
                            }
                        }
                    }
                    GameState::Questing => {
                        if let Some(ref mut session) = session_quest {
                            if session.filled_slots.len() >= session.slots.len() {
                                crate::quest::complete_quest(session, &mut sheet, &mut spellbook, &mut curriculum, &mut next_state, &mut commands, &state);
                            } else if let Some(idx) = hand.selected {
                                if idx < hand.cards.len() {
                                    let word = &hand.cards[idx];
                                    let slots_count = session.slots.len();
                                    for i in 0..slots_count {
                                        if !session.filled_slots.contains_key(&i) {
                                            crate::quest::fill_slot(i, word, Some(sheet.active_summon_class), session);
                                            break;
                                        }
                                    }
                                    hand.selected = None;
                                }
                            } else {
                                warn!("Select a card first or complete quest if full!");
                            }
                        }
                    }
                    _ => {}
                }
            }

            GameCommand::SkipCard => {
                match *state.get() {
                    GameState::Playing => {
                        crate::quest::start_quest("Barnaby", &db, &curriculum, &mut commands, &mut next_state, &state);
                    }
                    GameState::Battling => {
                        info!("Retreating from battle!");
                        commands.remove_resource::<crate::battle::BattleSession>();
                        log_state_transition(state.get(), GameState::Playing);
                        next_state.set(GameState::Playing);
                    }
                    GameState::Questing => {
                        info!("Canceling quest!");
                        commands.remove_resource::<crate::quest::QuestSession>();
                        log_state_transition(state.get(), GameState::Playing);
                        next_state.set(GameState::Playing);
                    }
                    _ => {}
                }
            }

            GameCommand::PlayBattleCard(idx) => {
                if *state.get() == GameState::Battling {
                    if let Some(ref mut session) = session_battle {
                        if *idx < hand.cards.len() {
                            let played_word = hand.cards.remove(*idx);
                            let typo_word = session.typo_word.clone();
                            let result = crate::battle::play_battle_card(
                                &played_word,
                                session,
                                &db,
                                &mut spellbook,
                                &mut next_state,
                                &sheet,
                                &state,
                            );
                            if result.is_effective {
                                commands.spawn(crate::battle::CriticalHitTrigger);
                            }
                            if result.social_combat_triggered {
                                if let Some(ref asset_server) = asset_server {
                                    crate::chat::trigger_social_combat(
                                        &played_word,
                                        &typo_word,
                                        result.is_synonym_logic,
                                        time.elapsed_secs(),
                                        &mut chat_log,
                                        &mut commands,
                                        asset_server,
                                    );
                                }
                            }
                            hand.selected = None;
                        } else {
                            warn!("PlayBattleCard index {} out of bounds", idx);
                        }
                    }
                }
            }

            GameCommand::FleeBattle => {
                if *state.get() == GameState::Battling {
                    commands.remove_resource::<crate::battle::BattleSession>();
                    log_state_transition(state.get(), GameState::Playing);
                    next_state.set(GameState::Playing);
                }
            }

            GameCommand::CancelQuest => {
                if *state.get() == GameState::Questing {
                    commands.remove_resource::<crate::quest::QuestSession>();
                    log_state_transition(state.get(), GameState::Playing);
                    next_state.set(GameState::Playing);
                }
            }

            GameCommand::CompleteQuest => {
                if let Some(ref mut session) = session_quest {
                    if session.filled_slots.len() >= session.slots.len() {
                        crate::quest::complete_quest(session, &mut sheet, &mut spellbook, &mut curriculum, &mut next_state, &mut commands, &state);
                    } else {
                        warn!("Cannot complete quest: not all slots are filled!");
                    }
                }
            }

            GameCommand::FillQuestSlot(card_idx) => {
                if *state.get() == GameState::Questing {
                    if let Some(ref mut session) = session_quest {
                        if *card_idx < hand.cards.len() {
                            let word = &hand.cards[*card_idx];
                            let slots_count = session.slots.len();
                            for i in 0..slots_count {
                                if !session.filled_slots.contains_key(&i) {
                                    crate::quest::fill_slot(i, word, Some(sheet.active_summon_class), session);
                                    break;
                                }
                            }
                        } else {
                            warn!("FillQuestSlot card index {} out of bounds", card_idx);
                        }
                    }
                }
            }

            GameCommand::SubmitSpelling => {
                if *state.get() == GameState::Constructing {
                    if let (Some(meshes), Some(materials)) = (meshes.take(), materials.take()) {
                        crate::letter::submit_spelling_word(
                            &mut *current_spelling,
                            &mut *letter_stash,
                            &mut *next_state,
                            &*db,
                            &mut commands,
                            meshes,
                            materials,
                            &*spellbook,
                            &*demo,
                            &*sheet,
                            &state,
                        );
                    } else {
                        warn!("SubmitSpelling skipped: rendering assets not available");
                    }
                    // Mesh/material resources are consumed by submit_spelling_word; return to avoid further use.
                    return;
                }
            }

            GameCommand::AddLetter(c) => {
                let upper_c = c.to_ascii_uppercase();
                if let Some(pos) = letter_stash.letters.iter().position(|&x| x == upper_c) {
                    letter_stash.letters.remove(pos);
                    current_spelling.word.push(upper_c);
                    info!("Current spelling: {}", current_spelling.word);
                }
            }

            GameCommand::Backspace => {
                if let Some(c) = current_spelling.word.pop() {
                    letter_stash.letters.push(c);
                }
            }

            GameCommand::ClearSpelling => {
                for c in current_spelling.word.drain(..) {
                    letter_stash.letters.push(c);
                }
            }

            GameCommand::Swipe(choice) => {
                trail.swipe_history.push(*choice);
                trail.current_word = Some(current_spelling.word.clone());
            }

            GameCommand::DismissReview => {
                if *state.get() == GameState::Reviewing {
                    log_state_transition(state.get(), GameState::Playing);
                    next_state.set(GameState::Playing);
                }
            }

            GameCommand::NewGame => {
                if *state.get() == GameState::MainMenu {
                    let _ = std::fs::remove_file("save.json");
                    commands.insert_resource(crate::tutorial::TutorialState { step: 0, active: true });
                    log_state_transition(state.get(), GameState::Collecting);
                    next_state.set(GameState::Collecting);
                }
            }

            GameCommand::ContinueGame => {
                if *state.get() == GameState::MainMenu {
                    log_state_transition(state.get(), GameState::Collecting);
                    next_state.set(GameState::Collecting);
                }
            }

            GameCommand::OpenSettings => {
                info!("OpenSettings command received (not implemented yet)");
            }

            GameCommand::TransitionTo(target) => {
                log_state_transition(state.get(), target.clone());
                next_state.set(target.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_command_variants_round_trip() {
        let commands = vec![
            GameCommand::SubmitSpelling,
            GameCommand::SelectCard(3),
            GameCommand::PlayCard,
            GameCommand::StartBattle,
            GameCommand::PlayBattleCard(1),
            GameCommand::FleeBattle,
            GameCommand::StartQuest("Barnaby".to_string()),
            GameCommand::FillQuestSlot(0),
            GameCommand::CompleteQuest,
            GameCommand::CancelQuest,
            GameCommand::Swipe(SwipeChoice::Yes),
            GameCommand::DismissReview,
            GameCommand::NewGame,
            GameCommand::ContinueGame,
            GameCommand::TransitionTo(GameState::Playing),
        ];
        // Every variant must be PartialEq so this compiles and asserts equality.
        assert_eq!(commands, commands.clone());
    }
}
