// integration_tests.rs — End-to-End Integration and Pedagogical Tests
#![allow(unused)]
use lit_ttc::components::*;
use lit_ttc::database::GameDatabase;
use lit_ttc::quest::{self, QuestSession};
use lit_ttc::battle::{self, BattleSession};
use lit_ttc::save::{self, SaveData};
use lit_ttc::blocklist;
use lit_ttc::commands::{GameCommand, LastCommand, handle_game_commands};
use lit_ttc::chat;
use lit_ttc::letter;
use lit_ttc::paywall;
use bevy::prelude::*;
use std::collections::HashMap;

#[test]
fn test_database_loading() {
    let db = GameDatabase::load_from_embedded();
    assert!(db.is_ok(), "Embedded JSON database should load successfully");
    let database = db.unwrap();
    
    assert!(database.words.contains_key("abandoned"), "Database should contain 'abandoned'");
    assert!(database.synonyms.contains_key("abandoned"), "Synonyms should contain 'abandoned'");
    assert!(database.etymology.roots.contains_key("Ignis"), "Etymology roots should contain 'Ignis'");
    assert!(database.quests.npc_chains.contains_key("Barnaby"), "Quests should contain 'Barnaby' chain");
}

#[test]
fn test_database_schema_validation() {
    let db = GameDatabase::load_from_embedded().expect("Database should load");
    
    // Validate WordStats constraints
    for (word, stats) in &db.words {
        assert!(stats.concreteness >= 0.0 && stats.concreteness <= 5.0, "Word '{}' concreteness out of bounds: {}", word, stats.concreteness);
        assert!(stats.valence >= 0.0 && stats.valence <= 9.0, "Word '{}' valence out of bounds: {}", word, stats.valence);
        assert!(stats.intensity >= 0.0 && stats.intensity <= 9.0, "Word '{}' intensity out of bounds: {}", word, stats.intensity);
        assert!(stats.dominance >= 0.0 && stats.dominance <= 9.0, "Word '{}' dominance out of bounds: {}", word, stats.dominance);
        assert!(!stats.grade_level.is_empty(), "Word '{}' missing grade level", word);
    }
    
    // Validate Synonym constraints
    for (word, syn_data) in &db.synonyms {
        assert!(!syn_data.element.is_empty(), "Synonym entry '{}' missing element", word);
        for syn in &syn_data.synonyms {
            assert!(!syn.is_empty(), "Synonym for '{}' is empty", word);
        }
    }

    // Validate Etymology constraints
    for (root, data) in &db.etymology.roots {
        assert!(!data.element.is_empty(), "Root '{}' missing element", root);
        assert!(!data.stat_focus.is_empty(), "Root '{}' missing stat focus", root);
        assert!(data.color.len() == 3, "Root '{}' color must be RGB", root);
    }
}

#[test]
fn test_quest_progression() {
    let mut db = GameDatabase::load_from_embedded().unwrap();
    let mut queue1 = bevy::ecs::world::CommandQueue::default();
    let mut world1 = World::new();
    let mut commands = Commands::new(&mut queue1, &world1);
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.insert_resource(db);
    app.insert_resource(CharacterSheet::default());
    app.insert_resource(SpellBook::default());
    
    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    
    // Simulate starting quest
    let npc_quest = app.world().resource::<GameDatabase>().quests.npc_chains.get("Barnaby").unwrap().first().unwrap().clone();
    
    let mut session = QuestSession {
        npc_name: "Barnaby".to_string(),
        title: npc_quest.title.clone(),
        template: npc_quest.template.clone(),
        slots: vec!["ADJECTIVE".to_string()],
        filled_slots: HashMap::new(),
        xp_reward: npc_quest.rewards.xp,
        evolution_reward: npc_quest.rewards.evolution_points,
    };
    
    // Fill slot and complete
    quest::fill_slot(0, "patience", None, &mut session);
    assert_eq!(session.filled_slots.get(&0).unwrap().0, "patience");
    
    let mut sheet = CharacterSheet::default();
    let mut spellbook = SpellBook::default();
    let mut test_next_state = NextState::default();
    let mut queue2 = bevy::ecs::world::CommandQueue::default();
    let mut world2 = World::new();
    let mut test_commands = Commands::new(&mut queue2, &world2);
    let mut curriculum = quest::CurriculumManager::default();
    let state = State::new(GameState::Questing);

    quest::complete_quest(
        &session,
        &mut sheet,
        &mut spellbook,
        &mut curriculum,
        &mut test_next_state,
        &mut test_commands,
        &state,
    );
    
    assert_eq!(sheet.total_xp, npc_quest.rewards.xp as u64);
    assert_eq!(sheet.words_encountered, 1);
}

#[test]
fn test_battle_combat_mechanics() {
    let db = GameDatabase::load_from_embedded().unwrap();
    let mut session = BattleSession {
        typo_word: "abandoned".to_string(),
        typo_health: 100,
        player_health: 100,
    };
    
    let mut spellbook = SpellBook::default();
    let mut next_state = NextState::default();
    let sheet = CharacterSheet::default();
    let state = State::new(GameState::Battling);

    // Play a word with high semantic distance (effective): "abc" vs "abandoned"
    let result_1 = battle::play_battle_card("abc", &mut session, &db, &mut spellbook, &mut next_state, &sheet, &state);
    assert!(result_1.is_effective, "Playing abc should be semantically effective");
    assert_eq!(session.typo_health, 32, "Effective card should damage typo");
    assert_eq!(session.player_health, 100, "Effective card should not damage player");

    // Play a word with low semantic distance (ineffective): "abandoned" vs "abandoned"
    let result_2 = battle::play_battle_card("abandoned", &mut session, &db, &mut spellbook, &mut next_state, &sheet, &state);
    assert!(!result_2.is_effective, "Playing identical word should be ineffective");
    assert_eq!(session.typo_health, 20, "Ineffective card should do minor damage");
    assert_eq!(session.player_health, 80, "Ineffective card should result in typo counter-attack");
}

#[test]
fn test_rhetoric_robot_combat_mechanics() {
    let db = GameDatabase::load_from_embedded().unwrap();
    let mut session = BattleSession {
        typo_word: "abandoned".to_string(),
        typo_health: 100,
        player_health: 100,
    };
    
    let mut spellbook = SpellBook::default();
    let mut next_state = NextState::default();
    let mut sheet = CharacterSheet::default();
    sheet.active_summon_class = SummonClass::RhetoricRobot;
    let state = State::new(GameState::Battling);

    // Play a word. Rhetoric Robot triggers social combat.
    let result = battle::play_battle_card("abc", &mut session, &db, &mut spellbook, &mut next_state, &sheet, &state);
    
    assert!(result.social_combat_triggered, "Rhetoric Robot should trigger social combat");
    assert!(result.is_effective, "Rhetoric attack should be highly effective");
    assert_eq!(session.typo_health, 38, "Rhetoric attack deals massive 2.5x damage (25 * 2.5 = 62. 100 - 62 = 38)");
}

#[test]
fn test_local_save_system() {
    let sheet = CharacterSheet {
        mind_attunement: 0.55,
        heart_attunement: 0.2,
        body_attunement: 0.1,
        action_attunement: 0.05,
        emergent_class: "The Oracle".to_string(),
        words_encountered: 12,
        total_deeper_swipes: 4,
        total_xp: 450,
        active_summon_class: SummonClass::SemanticSlime,
        arm_length: 0.65,
    };
    
    let mut spellbook = SpellBook::default();
    spellbook.record_encounter("clarity", Channel::Mind);
    spellbook.upgrade_mastery("clarity", MasteryLevel::Mastered);
    
    let trail = StudentTrail {
        visited_words: vec!["clarity".to_string(), "abandoned".to_string()],
        swipe_history: vec![],
        current_word: Some("clarity".to_string()),
    };
    
    // Test saving
    let save_res = save::save_game(&sheet, &spellbook, &trail);
    assert!(save_res.is_ok(), "Saving progress to disk should succeed");
    
    // Test loading
    let load_res = save::load_game();
    assert!(load_res.is_ok(), "Loading progress from disk should succeed");
    
    let loaded = load_res.unwrap();
    assert_eq!(loaded.character_sheet.emergent_class, "The Oracle");
    assert_eq!(loaded.character_sheet.total_xp, 450);
    assert_eq!(loaded.student_trail.visited_words.len(), 2);
    
    // Clean up temporary save file
    let _ = std::fs::remove_file("save.json");
}

#[test]
fn test_pet_chat_taming() {
    use lit_ttc::chat::{self, ChatLog};
    use faces_protocol::{Focus, Action};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ChatLog>();

    let mut chat_log = app.world().resource::<ChatLog>().clone();
    assert_eq!(chat_log.messages.len(), 0);

    // Simulate taming actions directly
    let focus = Focus::Happy;
    let action = Action::Playful;
    let element = Element::Normal;

    let dialogue = chat::get_pet_dialogue("wisdom", focus, action, element);
    chat_log.add_message("wisdom", &dialogue, 0.0);

    assert_eq!(chat_log.messages.len(), 1);
    assert!(chat_log.messages[0].text.contains("wisdom"));
    assert!(chat_log.messages[0].text.contains("bounce-bounce-bouncy"));
}

#[test]
fn test_curriculum_and_dialogue() {
    use lit_ttc::quest::{self, CurriculumManager};

    let db = GameDatabase::load_from_embedded().unwrap();
    
    // 1. Verify dynamic NPC dialogue matching
    let dialogue_dawn = quest::get_npc_dialogue("Peter", &db, "Dawn");
    let dialogue_night = quest::get_npc_dialogue("Peter", &db, "Night");
    
    assert!(!dialogue_dawn.is_empty());
    assert!(!dialogue_night.is_empty());

    // 2. Verify CurriculumManager grade mapping
    let mut curriculum = CurriculumManager {
        active_grade: 1,
        progress_xp: 0,
        unlocked_districts: vec!["The Phoneme Forest".to_string()],
    };
    
    assert_eq!(curriculum.get_grade_level(), "Grade 1");
    
    // Gain 1200 XP (threshold for Grade 2 is 1000 XP)
    let graded_up = curriculum.check_grade_up(1200);
    assert!(graded_up);
    assert_eq!(curriculum.get_grade_level(), "Grade 2");
}

#[test]
fn test_blocklist_filters_inappropriate_words() {
    // Clean words pass
    assert!(blocklist::is_clean("happy"), "happy should be clean");
    assert!(blocklist::is_clean("serenity"), "serenity should be clean");
    assert!(blocklist::is_clean("dragon"), "dragon should be clean");
    assert!(blocklist::is_clean("knowledge"), "knowledge should be clean");

    // Profanity blocked
    assert!(!blocklist::is_clean("damn"), "damn should be blocked");
    assert!(!blocklist::is_clean("shit"), "shit should be blocked");
    assert!(!blocklist::is_clean("fuck"), "fuck should be blocked");

    // Case insensitive
    assert!(!blocklist::is_clean("SHIT"), "SHIT should be blocked");
    assert!(!blocklist::is_clean("Fuck"), "Fuck should be blocked");

    // Compound words with banned substrings
    assert!(!blocklist::is_clean("fuckface"), "fuckface should be blocked");
    assert!(!blocklist::is_clean("shithead"), "shithead should be blocked");

    // False positive check — legitimate words that contain "ass" etc.
    assert!(blocklist::is_clean("assassin"), "assassin should be clean");
    assert!(blocklist::is_clean("classroom"), "classroom should be clean");
    assert!(blocklist::is_clean("glass"), "glass should be clean");
    assert!(blocklist::is_clean("grass"), "grass should be clean");
}

#[test]
fn test_game_command_messages() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.add_message::<GameCommand>();
    app.init_resource::<LastCommand>();

    // System that records the last command so we can assert it was received.
    app.add_systems(
        Update,
        |mut messages: MessageReader<GameCommand>, mut last: ResMut<LastCommand>| {
            for msg in messages.read() {
                last.0 = Some(msg.clone());
            }
        },
    );

    // Send a StartQuest command and advance the app schedule.
    let cmd = GameCommand::StartQuest("Barnaby".to_string());
    app.world_mut().write_message(cmd.clone());
    app.update();

    let last = app.world().resource::<LastCommand>();
    assert_eq!(last.0, Some(cmd), "GameCommand should be received and recorded");
}

#[test]
fn test_command_handler_select_card() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.add_message::<GameCommand>();
    app.init_resource::<LastCommand>();
    app.init_resource::<Hand>();

    app.add_systems(Update, handle_game_commands);

    // Minimal resources required by the handler signature.
    app.init_resource::<GameDatabase>();
    app.init_resource::<quest::CurriculumManager>();
    app.init_resource::<SpellBook>();
    app.init_resource::<CharacterSheet>();
    app.init_resource::<StudentTrail>();
    app.init_resource::<chat::ChatLog>();
    app.init_resource::<letter::CurrentSpelling>();
    app.init_resource::<letter::LetterStash>();
    app.init_resource::<paywall::DemoSettings>();

    // Pre-populate the hand.
    {
        let mut hand = app.world_mut().resource_mut::<Hand>();
        hand.cards = vec!["wisdom".to_string(), "courage".to_string(), "patience".to_string()];
    }

    app.world_mut().write_message(GameCommand::SelectCard(1));
    app.update();

    let hand = app.world().resource::<Hand>();
    assert_eq!(hand.selected, Some(1), "Handler should select card index 1");
}

#[test]
fn test_command_handler_dismiss_review() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.add_message::<GameCommand>();
    app.init_resource::<LastCommand>();
    // Minimal resources required by the handler signature.
    app.init_resource::<GameDatabase>();
    app.init_resource::<quest::CurriculumManager>();
    app.init_resource::<Hand>();
    app.init_resource::<SpellBook>();
    app.init_resource::<CharacterSheet>();
    app.init_resource::<StudentTrail>();
    app.init_resource::<chat::ChatLog>();
    app.init_resource::<letter::CurrentSpelling>();
    app.init_resource::<letter::LetterStash>();
    app.init_resource::<paywall::DemoSettings>();

    app.add_systems(Update, handle_game_commands);

    // Move state into Reviewing.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Reviewing);
    app.update();
    assert_eq!(*app.world().resource::<State<GameState>>().get(), GameState::Reviewing);

    // Dismiss the review.
    app.world_mut().write_message(GameCommand::DismissReview);
    app.update();
    app.update(); // Second update applies the NextState transition set by the handler.

    assert_eq!(*app.world().resource::<State<GameState>>().get(), GameState::Playing);
}
