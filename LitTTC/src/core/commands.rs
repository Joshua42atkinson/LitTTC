// commands.rs — GameCommand message bridge: input decoupled from game logic
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::components::{GameState, SwipeChoice};

/// A player-intent message fired by input systems and consumed by command handlers.
///
/// This enum decouples raw input (mouse clicks, keyboard, VR pinches, swipes)
/// from the game systems that mutate state. Every variant captures the data
/// needed to perform a high-level action without knowing which input device
/// triggered it.
#[derive(Message, Clone, Debug, PartialEq)]
pub enum GameCommand {
    // ─── SPELLING / WORD CONSTRUCTION ─────────────────────────────────
    /// Submit the word currently being typed in the spelling pad.
    SubmitSpelling,
    /// Add a single letter to the current spelling.
    AddLetter(char),
    /// Remove the last letter from the current spelling.
    Backspace,
    /// Scan a world object in the 2D overworld (flat2d) to harvest its word.
    ScanObject(String),

    // ─── HAND / CARD INTERACTIONS ───────────────────────────────────────
    /// Select the card at the given index in the player's hand.
    SelectCard(usize),
    /// Play the currently selected card (state-dependent interpretation).
    PlayCard,

    // ─── BATTLE ─────────────────────────────────────────────────────────
    /// Start a battle against a wild Typo.
    StartBattle,
    /// Play the card at the given index as a battle attack.
    PlayBattleCard(usize),
    /// Retreat from the current battle.
    FleeBattle,

    // ─── THESAURUS DANCE (SENTENCE CRAFTING) ────────────────────────────
    /// Add the card at the given hand index to the battle Plot.
    AddToPlot(usize),
    /// Remove the card at the given Plot index.
    RemoveFromPlot(usize),
    /// Cast the constructed sentence as a single attack.
    CastSentence,

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

    // ─── DEBUG / COMBAT SIMULATION ───────────────────────────────────────
    /// Start a debug battle with specific enemy word
    DebugBattle(String),
    /// Set the active face for combat testing
    SetFace(String),
    /// Print current VAAM metrics
    PrintVaam,

    // ─── MENU ───────────────────────────────────────────────────────────
    /// Start a new game (clear save, begin tutorial, go to Collecting).
    NewGame,
    /// Continue from an existing save file.
    ContinueGame,
    /// Open the settings screen.
    OpenSettings,
    /// Open the difficulty selection screen.
    OpenDifficulty,
    /// Open the pet collection gallery.
    OpenPetCollection,
    /// Return to the main menu from settings/gallery/difficulty/paywall.
    ReturnToMenu,
}

/// Resource that tracks the most recently fired command for debugging and replay.
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
/// Resources are grouped into a single SystemParam to stay within Bevy's system-parameter limit.
#[derive(SystemParam)]
pub struct CommandContext<'w> {
    pub sheet: ResMut<'w, crate::components::CharacterSheet>,
    pub trail: ResMut<'w, crate::components::WordTrail>,
    pub session_battle: Option<ResMut<'w, crate::battle::BattleSession>>,
    pub plot: Option<ResMut<'w, crate::battle::Plot>>,
    pub session_quest: Option<ResMut<'w, crate::quest::QuestSession>>,
    pub hand: ResMut<'w, crate::components::Hand>,
    pub deck: Option<ResMut<'w, crate::components::Deck>>,
    pub spellbook: ResMut<'w, crate::components::SpellBook>,
    pub db: Res<'w, crate::database::GameDatabase>,
    pub grade_manager: ResMut<'w, crate::quest::GradeManager>,
    pub _time: Res<'w, Time>,
    pub _asset_server: Option<Res<'w, AssetServer>>,
    pub _chat_log: ResMut<'w, crate::chat::ChatLog>,
    pub current_spelling: ResMut<'w, crate::letter::CurrentSpelling>,
    pub letter_stash: ResMut<'w, crate::letter::LetterStash>,
    pub meshes: Option<ResMut<'w, Assets<Mesh>>>,
    pub materials: Option<ResMut<'w, Assets<StandardMaterial>>>,
    pub demo: ResMut<'w, crate::paywall::DemoSettings>,
    pub active_face: Option<Res<'w, crate::components::ActiveFace>>,
    pub vaam_metrics: Option<ResMut<'w, crate::battle::VaamMetrics>>,
}

pub fn handle_game_commands(
    mut commands: Commands,
    mut messages: MessageReader<GameCommand>,
    mut last: ResMut<LastCommand>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    ctx: CommandContext,
) {
    let CommandContext {
        mut sheet,
        mut trail,
        mut session_battle,
        mut plot,
        mut session_quest,
        mut hand,
        mut deck,
        mut spellbook,
        db,
        mut grade_manager,
        _time,
        _asset_server,
        _chat_log,
        mut current_spelling,
        mut letter_stash,
        mut meshes,
        mut materials,
        mut demo,
        active_face,
        mut vaam_metrics,
    } = ctx;

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
                let allowed = if cfg!(feature = "flat2d") {
                    *state.get() == GameState::Playing || *state.get() == GameState::Exploring
                } else {
                    *state.get() == GameState::Playing
                };
                if allowed {
                    crate::battle::start_battle(&mut commands, &db, &grade_manager, &mut next_state, &state);
                }
            }

            GameCommand::StartQuest(npc) => {
                let allowed = if cfg!(feature = "flat2d") {
                    *state.get() == GameState::Playing || *state.get() == GameState::Exploring
                } else {
                    *state.get() == GameState::Playing
                };
                if allowed {
                    crate::quest::start_quest(npc, &db, &grade_manager, &mut commands, &mut next_state, &state);
                }
            }

            GameCommand::PlayCard => {
                match *state.get() {
                    GameState::Playing => {
                        if hand.selected.is_some() {
                            crate::battle::start_battle(&mut commands, &db, &grade_manager, &mut next_state, &state);
                        } else {
                            warn!("Select a card first!");
                        }
                    }
                    GameState::Battling => {
                        if let Some(ref mut session) = session_battle {
                            if let Some(idx) = hand.selected {
                                if idx < hand.cards.len() {
                                    let played_word = hand.cards.remove(idx);
                                    let _typo_word = session.typo_word.clone();
                                    let face_ref = active_face.as_deref();
                                    let vaam_ref = vaam_metrics.as_deref_mut();
                                    let result = crate::battle::play_battle_card(
                                        &played_word,
                                        session,
                                        &db,
                                        &mut spellbook,
                                        &mut next_state,
                                        &sheet,
                                        &state,
                                        face_ref,
                                        vaam_ref,
                                    );
                                    if result.is_effective {
                                        commands.spawn(crate::battle::CriticalHitTrigger);
                                    }
                                    // Tutor Loop: if player defeated, start targeted quest
                                    if session.player_health <= 0 {
                                        crate::battle::start_tutor_loop(&mut commands, &db, &grade_manager, session, &mut next_state, &state);
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
                                crate::quest::complete_quest(session, &mut sheet, &mut spellbook, &mut grade_manager, &mut next_state, &mut commands, &state);
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

            GameCommand::PlayBattleCard(idx) => {
                if *state.get() == GameState::Battling {
                    if let Some(ref mut session) = session_battle {
                        if *idx < hand.cards.len() {
                            let played_word = hand.cards.remove(*idx);
                            let _typo_word = session.typo_word.clone();
                            let face_ref = active_face.as_deref();
                            let vaam_ref = vaam_metrics.as_deref_mut();
                            let result = crate::battle::play_battle_card(
                                &played_word,
                                session,
                                &db,
                                &mut spellbook,
                                &mut next_state,
                                &sheet,
                                &state,
                                face_ref,
                                vaam_ref,
                            );
                            if result.is_effective {
                                commands.spawn(crate::battle::CriticalHitTrigger);
                            }
                            // Tutor Loop: if player defeated, start targeted quest
                            if session.player_health <= 0 {
                                crate::battle::start_tutor_loop(&mut commands, &db, &grade_manager, session, &mut next_state, &state);
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
                    commands.remove_resource::<crate::battle::Plot>();
                    let next = if cfg!(feature = "flat2d") { GameState::Exploring } else { GameState::Playing };
                    log_state_transition(state.get(), next.clone());
                    next_state.set(next);
                }
            }

            GameCommand::AddToPlot(idx) => {
                if *state.get() == GameState::Battling {
                    if let Some(ref mut p) = plot {
                        if *idx < hand.cards.len() && p.cards.len() < p.max_size {
                            let word = hand.cards.remove(*idx);
                            p.cards.push(word.clone());
                            info!("Added '{}' to the Plot. Sentence: {}", word, p.sentence_preview());
                            hand.selected = None;
                        } else {
                            warn!("AddToPlot index {} out of bounds or Plot full", idx);
                        }
                    }
                }
            }

            GameCommand::RemoveFromPlot(idx) => {
                if *state.get() == GameState::Battling {
                    if let Some(ref mut p) = plot {
                        if *idx < p.cards.len() {
                            let word = p.cards.remove(*idx);
                            hand.cards.push(word.clone());
                            info!("Removed '{}' from the Plot. Sentence: {}", word, p.sentence_preview());
                        } else {
                            warn!("RemoveFromPlot index {} out of bounds", idx);
                        }
                    }
                }
            }

            GameCommand::CastSentence => {
                if *state.get() == GameState::Battling {
                    if let Some(ref mut session) = session_battle {
                        let face_ref = active_face.as_deref();
                        let vaam_ref = vaam_metrics.as_deref_mut();
                        crate::battle::cast_sentence(
                            session,
                            plot.as_deref_mut(),
                            &db,
                            &mut spellbook,
                            &mut next_state,
                            &sheet,
                            &state,
                            face_ref,
                            vaam_ref,
                        );
                    }
                }
            }

            GameCommand::CancelQuest => {
                if *state.get() == GameState::Questing {
                    commands.remove_resource::<crate::quest::QuestSession>();
                    let next = if cfg!(feature = "flat2d") { GameState::Exploring } else { GameState::Playing };
                    log_state_transition(state.get(), next.clone());
                    next_state.set(next);
                }
            }

            GameCommand::CompleteQuest => {
                if let Some(ref mut session) = session_quest {
                    if session.filled_slots.len() >= session.slots.len() {
                        crate::quest::complete_quest(session, &mut sheet, &mut spellbook, &mut grade_manager, &mut next_state, &mut commands, &state);
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

            GameCommand::DebugBattle(enemy_word) => {
                if *state.get() == GameState::Playing {
                    info!("DEBUG: Starting battle against '{}'", enemy_word);
                    commands.insert_resource(crate::battle::BattleSession {
                        typo_word: enemy_word.clone(),
                        typo_health: 100,
                        player_health: 100,
                        failed_word: None,
                    });
                    // Initialize VAAM metrics if not present
                    if vaam_metrics.is_none() {
                        commands.insert_resource(crate::battle::VaamMetrics::default());
                    }
                    // Initialize ActiveFace if not present
                    if active_face.is_none() {
                        commands.insert_resource(crate::components::ActiveFace::default());
                    }
                    crate::commands::log_state_transition(state.get(), GameState::Battling);
                    next_state.set(GameState::Battling);
                }
            }

            GameCommand::SetFace(face_name) => {
                let face = match face_name.to_lowercase().as_str() {
                    "fierce" => crate::components::SlimeFace::Fierce,
                    "joyful" => crate::components::SlimeFace::Joyful,
                    "calm" => crate::components::SlimeFace::Calm,
                    "angry" => crate::components::SlimeFace::Angry,
                    _ => {
                        warn!("Unknown face: '{}'. Use: fierce, joyful, calm, angry", face_name);
                        return;
                    }
                };
                commands.insert_resource(crate::components::ActiveFace { face });
                info!("DEBUG: Set active face to {:?}", face);
            }

            GameCommand::PrintVaam => {
                if let Some(ref metrics) = vaam_metrics {
                    info!("=== VAAM METRICS ===");
                    info!("{}", metrics.get_summary());
                    info!("====================");
                } else {
                    warn!("No VAAM metrics resource found. Start a battle first.");
                }
            }

            GameCommand::SubmitSpelling => {
                if *state.get() == GameState::Constructing {
                    if let (Some(mut meshes), Some(mut materials)) = (meshes.take(), materials.take()) {
                        let data = crate::letter::SpellingData {
                            db: &db,
                            sheet: &sheet,
                        };
                        let mut assets = crate::letter::SpellingAssets {
                            meshes: &mut meshes,
                            materials: &mut materials,
                        };
                        crate::letter::submit_spelling_word(
                            &mut current_spelling,
                            &mut next_state,
                            &mut commands,
                            &data,
                            &mut demo,
                            &mut spellbook,
                            &mut assets,
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

            GameCommand::ScanObject(word) => {
                if cfg!(feature = "flat2d") && *state.get() == GameState::Exploring {
                    let upper = word.to_uppercase();
                    letter_stash.letters.clear();
                    current_spelling.word.clear();
                    letter_stash.letters.extend(upper.chars());
                    // Make sure the harvested word is available in the deck after spelling.
                    if let Some(ref mut d) = deck {
                        if !d.cards.contains(&upper) {
                            d.cards.push(upper.clone());
                        }
                    }
                    log_state_transition(state.get(), GameState::Constructing);
                    next_state.set(GameState::Constructing);
                }
            }

            GameCommand::Swipe(choice) => {
                trail.swipe_history.push(*choice);
                trail.current_word = Some(current_spelling.word.clone());
            }

            GameCommand::DismissReview => {
                if *state.get() == GameState::Reviewing {
                    let next = if cfg!(feature = "flat2d") { GameState::Exploring } else { GameState::Playing };
                    log_state_transition(state.get(), next.clone());
                    next_state.set(next);
                }
            }

            GameCommand::NewGame => {
                if *state.get() == GameState::MainMenu {
                    if std::path::Path::new("save.json").exists() {
                        info!("Existing chronicle found; archiving to save.json.bak before a new journey.");
                        let _ = std::fs::rename("save.json", "save.json.bak");
                    }
                    commands.insert_resource(crate::tutorial::TutorialState { step: 0, active: true });
                    let next = if cfg!(feature = "flat2d") { GameState::Exploring } else { GameState::Collecting };
                    log_state_transition(state.get(), next.clone());
                    next_state.set(next);
                }
            }

            GameCommand::ContinueGame => {
                if *state.get() == GameState::MainMenu {
                    log_state_transition(state.get(), GameState::Collecting);
                    next_state.set(GameState::Collecting);
                }
            }

            GameCommand::OpenSettings => {
                if *state.get() == GameState::MainMenu {
                    log_state_transition(state.get(), GameState::Settings);
                    next_state.set(GameState::Settings);
                }
            }

            GameCommand::OpenDifficulty => {
                if *state.get() == GameState::MainMenu {
                    log_state_transition(state.get(), GameState::Difficulty);
                    next_state.set(GameState::Difficulty);
                }
            }

            GameCommand::OpenPetCollection => {
                if *state.get() == GameState::MainMenu {
                    log_state_transition(state.get(), GameState::PetCollection);
                    next_state.set(GameState::PetCollection);
                }
            }

            GameCommand::ReturnToMenu => {
                if matches!(*state.get(), GameState::Settings | GameState::Difficulty | GameState::PetCollection | GameState::Paywall) {
                    log_state_transition(state.get(), GameState::MainMenu);
                    next_state.set(GameState::MainMenu);
                }
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
            GameCommand::OpenSettings,
            GameCommand::OpenDifficulty,
            GameCommand::OpenPetCollection,
            GameCommand::ReturnToMenu,
        ];
        // Every variant must be PartialEq so this compiles and asserts equality.
        assert_eq!(commands, commands.clone());
    }
}
